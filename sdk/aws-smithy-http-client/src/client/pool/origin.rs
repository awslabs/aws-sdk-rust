/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Canonical HTTP origin identity.
//!
//! [`OriginLookup`] borrows canonical URI host text when possible. A miss
//! materializes an owned [`OriginKey`] for retained pool state; a canonical
//! lookup hit does not allocate host storage.

use http_1x::uri::{Authority, Scheme};
use http_1x::Uri;
use std::borrow::Cow;
#[cfg(test)]
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::net::Ipv6Addr;
use std::num::NonZeroU16;
use std::str::FromStr;
use std::sync::Arc;

#[cfg(test)]
std::thread_local! {
    static OWNED_ORIGIN_KEY_MATERIALIZATIONS: Cell<usize> = const { Cell::new(0) };
}

/// An owned, canonical HTTP or HTTPS origin.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct OriginKey {
    scheme: SchemeKey,
    host: Arc<str>,
    port: Option<NonZeroU16>,
}

impl OriginKey {
    /// Creates an origin from an absolute HTTP or HTTPS URI.
    pub fn from_uri(uri: &Uri) -> Result<Self, InvalidOrigin> {
        OriginLookup::from_uri(uri).map(OriginLookup::into_owned)
    }

    /// Creates an origin from structured scheme, host, and port parts.
    ///
    /// The `host` must not contain user information or a port. IPv6 literals
    /// use the bracketed authority form, for example `[::1]`.
    pub fn from_parts(
        scheme: Scheme,
        host: impl AsRef<str>,
        port: Option<u16>,
    ) -> Result<Self, InvalidOrigin> {
        let scheme = SchemeKey::from_scheme(&scheme)?;
        let host = host.as_ref();
        let authority = Authority::from_str(host).map_err(|source| {
            InvalidOrigin::with_source(format!("invalid origin host {host:?}"), source)
        })?;

        if authority.as_str().contains('@') || authority_port_text(&authority).is_some() {
            return Err(InvalidOrigin::new(
                "origin host must not contain user information or a port",
            ));
        }

        let host = authority.host();
        if host.is_empty() {
            return Err(InvalidOrigin::new("origin host must not be empty"));
        }

        Ok(OriginLookup {
            scheme,
            host: canonical_host(host),
            port: canonical_port(scheme, port)?,
        }
        .into_owned())
    }

    /// Returns this origin's HTTP or HTTPS scheme.
    pub fn scheme(&self) -> &Scheme {
        self.scheme.as_scheme()
    }

    /// Returns this origin's canonical host.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the non-default port, if one was specified.
    pub fn port(&self) -> Option<u16> {
        self.port.map(NonZeroU16::get)
    }

    /// Clones the shared host pointer for use as an index key.
    pub(super) fn shared_host(&self) -> Arc<str> {
        self.host.clone()
    }
}

/// Error returned when a URI or set of parts does not name an HTTP origin.
#[derive(Debug)]
pub struct InvalidOrigin {
    message: String,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl InvalidOrigin {
    /// Creates an origin error with no lower-level cause.
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Creates an origin error that preserves the parsing failure as its source.
    fn with_source(message: impl Into<String>, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for InvalidOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for InvalidOrigin {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn Error + 'static))
    }
}

/// Compact HTTP or HTTPS scheme identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum SchemeKey {
    Http,
    Https,
}

impl SchemeKey {
    /// Accepts HTTP or HTTPS case-insensitively and rejects every other scheme.
    fn from_scheme(scheme: &Scheme) -> Result<Self, InvalidOrigin> {
        if scheme.as_str().eq_ignore_ascii_case("http") {
            Ok(Self::Http)
        } else if scheme.as_str().eq_ignore_ascii_case("https") {
            Ok(Self::Https)
        } else {
            Err(InvalidOrigin::new(format!(
                "unsupported origin scheme {:?}; expected http or https",
                scheme.as_str()
            )))
        }
    }

    /// Returns the canonical static `http` or `https` scheme.
    fn as_scheme(self) -> &'static Scheme {
        match self {
            Self::Http => &Scheme::HTTP,
            Self::Https => &Scheme::HTTPS,
        }
    }
}

/// Borrowed canonical origin used to probe retained cells.
///
/// A lowercase URI host remains borrowed on a lookup hit. After a cell miss,
/// converting it into an [`OriginKey`] allocates the host retained by the new
/// cell.
pub(super) struct OriginLookup<'a> {
    scheme: SchemeKey,
    host: Cow<'a, str>,
    port: Option<NonZeroU16>,
}

impl<'a> OriginLookup<'a> {
    /// Borrows every identity component from an already-owned origin.
    pub(super) fn from_origin(origin: &'a OriginKey) -> Self {
        Self {
            scheme: origin.scheme,
            host: Cow::Borrowed(&origin.host),
            port: origin.port,
        }
    }

    /// Canonicalizes an absolute HTTP or HTTPS URI for retained-state lookup.
    ///
    /// Host storage remains borrowed when the URI already uses its canonical
    /// spelling.
    pub(super) fn from_uri(uri: &'a Uri) -> Result<Self, InvalidOrigin> {
        let scheme = uri
            .scheme()
            .ok_or_else(|| InvalidOrigin::new("origin URI is missing a scheme"))
            .and_then(SchemeKey::from_scheme)?;
        let authority = uri
            .authority()
            .ok_or_else(|| InvalidOrigin::new("origin URI is missing an authority"))?;
        let host = authority.host();
        if host.is_empty() {
            return Err(InvalidOrigin::new("origin URI is missing a host"));
        }

        Ok(Self {
            scheme,
            host: canonical_host(host),
            port: canonical_port(scheme, parse_authority_port(authority)?)?,
        })
    }

    /// Returns the compact canonical scheme used by the first-level index.
    pub(super) fn scheme(&self) -> SchemeKey {
        self.scheme
    }

    /// Returns the canonical host without materializing owned storage.
    pub(super) fn host(&self) -> &str {
        &self.host
    }

    /// Returns the canonical non-default port used by the first-level index.
    pub(super) fn port(&self) -> Option<NonZeroU16> {
        self.port
    }

    /// Materializes the stable identity retained after a lookup miss.
    pub(super) fn into_owned(self) -> OriginKey {
        #[cfg(test)]
        OWNED_ORIGIN_KEY_MATERIALIZATIONS.with(|count| count.set(count.get() + 1));
        OriginKey {
            scheme: self.scheme,
            host: Arc::from(self.host),
            port: self.port,
        }
    }

    #[cfg(test)]
    /// Returns the number of lookup values converted into retained identities.
    pub(super) fn owned_origin_key_materializations_for_test() -> usize {
        OWNED_ORIGIN_KEY_MATERIALIZATIONS.with(Cell::get)
    }
}

/// Returns a non-default port, omitting scheme defaults and rejecting zero.
fn canonical_port(
    scheme: SchemeKey,
    port: Option<u16>,
) -> Result<Option<NonZeroU16>, InvalidOrigin> {
    match (scheme, port) {
        (_, Some(0)) => Err(InvalidOrigin::new("origin port must not be zero")),
        (SchemeKey::Http, Some(80)) | (SchemeKey::Https, Some(443)) | (_, None) => Ok(None),
        (_, Some(port)) => Ok(NonZeroU16::new(port)),
    }
}

/// Parses an explicitly written authority port.
///
/// Invalid and absent ports remain distinct so malformed origins cannot alias
/// an origin with the scheme's default port.
fn parse_authority_port(authority: &Authority) -> Result<Option<u16>, InvalidOrigin> {
    let Some(port) = authority_port_text(authority) else {
        return Ok(None);
    };

    port.parse().map(Some).map_err(|source| {
        InvalidOrigin::with_source(format!("invalid origin port {port:?}"), source)
    })
}

/// Returns the port text following an authority host, if one was written.
///
/// This reads the authority source because `Authority::port_u16` returns
/// `None` for both an absent port and a malformed or out-of-range port.
fn authority_port_text(authority: &Authority) -> Option<&str> {
    let host_and_port = authority
        .as_str()
        .rsplit_once('@')
        .map_or(authority.as_str(), |(_, host_and_port)| host_and_port);

    if host_and_port.starts_with('[') {
        let close = host_and_port.find(']')?;
        host_and_port[close + 1..].strip_prefix(':')
    } else {
        host_and_port.rsplit_once(':').map(|(_, port)| port)
    }
}

/// Canonicalizes DNS names and recognized IPv6 literals, borrowing when unchanged.
///
/// DNS names are ASCII-lowercased. IPv6 addresses use the standard compressed
/// spelling while retaining the zone identifier's case. Unknown IP-literal
/// forms remain byte-distinct because their case semantics are not known.
fn canonical_host(host: &str) -> Cow<'_, str> {
    if let Some(inner) = host
        .strip_prefix('[')
        .and_then(|host| host.strip_suffix(']'))
    {
        let (address, zone) = inner
            .split_once("%25")
            .map_or((inner, None), |(address, zone)| (address, Some(zone)));
        if let Ok(address) = address.parse::<Ipv6Addr>() {
            return if canonical_ipv6_matches(host, address, zone) {
                Cow::Borrowed(host)
            } else {
                Cow::Owned(match zone {
                    Some(zone) => format!("[{address}%25{zone}]"),
                    None => format!("[{address}]"),
                })
            };
        }

        return Cow::Borrowed(host);
    }

    if host.as_bytes().iter().any(u8::is_ascii_uppercase) {
        let mut canonical = host.to_owned();
        canonical.make_ascii_lowercase();
        Cow::Owned(canonical)
    } else {
        Cow::Borrowed(host)
    }
}

/// Compares an IPv6 literal with its canonical formatting without allocating.
fn canonical_ipv6_matches(host: &str, address: Ipv6Addr, zone: Option<&str>) -> bool {
    struct CompareWriter<'a> {
        expected: &'a [u8],
        offset: usize,
        matches: bool,
    }

    impl fmt::Write for CompareWriter<'_> {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.offset.saturating_add(value.len());
            if self.expected.get(self.offset..end) != Some(value.as_bytes()) {
                self.matches = false;
            }
            self.offset = end;
            Ok(())
        }
    }

    let mut writer = CompareWriter {
        expected: host.as_bytes(),
        offset: 0,
        matches: true,
    };
    let result = match zone {
        Some(zone) => fmt::write(&mut writer, format_args!("[{address}%25{zone}]")),
        None => fmt::write(&mut writer, format_args!("[{address}]")),
    };
    result.is_ok() && writer.matches && writer.offset == writer.expected.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_host_borrows_common_case() {
        assert!(matches!(canonical_host("example.com"), Cow::Borrowed(_)));
        assert!(matches!(
            canonical_host("EXAMPLE.com"),
            Cow::Owned(host) if host == "example.com"
        ));
    }

    #[test]
    fn canonicalizes_host_and_default_port() {
        let a = OriginKey::from_uri(&"https://EXAMPLE.com:443/a".parse().unwrap()).unwrap();
        let b = OriginKey::from_uri(&"https://example.com/b".parse().unwrap()).unwrap();

        assert_eq!(a, b);
        assert_eq!("example.com", a.host());
        assert_eq!(None, a.port());
    }

    #[test]
    fn non_default_port_and_scheme_remain_identity() {
        let base = OriginKey::from_uri(&"https://example.com/".parse().unwrap()).unwrap();
        let port = OriginKey::from_uri(&"https://example.com:8443/".parse().unwrap()).unwrap();
        let clear = OriginKey::from_uri(&"http://example.com/".parse().unwrap()).unwrap();

        assert_ne!(base, port);
        assert_ne!(base, clear);
        assert_eq!(Some(8443), port.port());
    }

    #[test]
    fn rejects_invalid_and_zero_ports() {
        for uri in [
            "https://example.com:/",
            "https://example.com:abc/",
            "https://example.com:99999/",
            "https://example.com:0/",
        ] {
            let uri: Uri = uri.parse().unwrap();
            assert!(OriginKey::from_uri(&uri).is_err(), "{uri}");
        }

        assert!(OriginKey::from_parts(Scheme::HTTPS, "example.com", Some(0)).is_err());
        assert!(OriginKey::from_parts(Scheme::HTTPS, "example.com:abc", None).is_err());
        assert!(OriginKey::from_parts(Scheme::HTTPS, "example.com:99999", None).is_err());
    }

    #[test]
    fn trailing_dot_remains_distinct() {
        let plain = OriginKey::from_uri(&"https://example.com/".parse().unwrap()).unwrap();
        let dotted = OriginKey::from_uri(&"https://example.com./".parse().unwrap()).unwrap();
        assert_ne!(plain, dotted);
    }

    #[test]
    fn parts_and_uri_canonicalize_identically() {
        let from_parts = OriginKey::from_parts(Scheme::HTTPS, "EXAMPLE.com", Some(443)).unwrap();
        let from_uri = OriginKey::from_uri(&"https://example.com/".parse().unwrap()).unwrap();
        assert_eq!(from_parts, from_uri);
    }

    #[test]
    fn parts_accept_mixed_case_http_scheme() {
        let scheme = Scheme::from_str("HTTPS").unwrap();
        let origin = OriginKey::from_parts(scheme, "example.com", None).unwrap();
        assert_eq!(&Scheme::HTTPS, origin.scheme());
    }

    #[test]
    fn uri_userinfo_is_not_part_of_the_origin() {
        let with_userinfo =
            OriginKey::from_uri(&"https://user:password@example.com/".parse().unwrap()).unwrap();
        let plain = OriginKey::from_uri(&"https://example.com/".parse().unwrap()).unwrap();
        assert_eq!(plain, with_userinfo);
    }

    #[test]
    fn ipv6_parts_and_uri_canonicalize_identically() {
        let from_parts =
            OriginKey::from_parts(Scheme::HTTPS, "[2001:0db8:0:0:0:0:0:1]", Some(443)).unwrap();
        let from_uri = OriginKey::from_uri(&"https://[2001:db8::1]:443/".parse().unwrap()).unwrap();

        assert_eq!(from_parts, from_uri);
        assert_eq!("[2001:db8::1]", from_parts.host());
        assert_eq!(None, from_parts.port());
    }

    #[test]
    fn canonical_ipv6_lookup_remains_borrowed() {
        assert!(matches!(canonical_host("[2001:db8::1]"), Cow::Borrowed(_)));
        assert!(matches!(
            canonical_host("[fe80::1%25ETH0]"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn ipv6_zone_identifier_case_remains_distinct() {
        let upper =
            OriginKey::from_uri(&"https://[fe80:0:0:0::1%25ETH0]/".parse().unwrap()).unwrap();
        let lower = OriginKey::from_uri(&"https://[fe80::1%25eth0]/".parse().unwrap()).unwrap();

        assert_eq!("[fe80::1%25ETH0]", upper.host());
        assert_ne!(upper, lower);
    }

    #[test]
    fn invalid_origin_preserves_diagnostic_source() {
        let error = OriginKey::from_parts(Scheme::HTTPS, "not a host", None).unwrap_err();
        assert!(error.to_string().contains("invalid origin host"));
        assert!(error.source().is_some());

        let uri: Uri = "https://example.com:abc/".parse().unwrap();
        let error = OriginKey::from_uri(&uri).unwrap_err();
        assert_eq!("invalid origin port \"abc\"", error.to_string());
        assert!(error.source().is_some());
    }

    #[test]
    fn rejects_non_origins() {
        let relative: Uri = "/path".parse().unwrap();
        assert_eq!(
            "origin URI is missing a scheme",
            OriginKey::from_uri(&relative).unwrap_err().to_string()
        );

        let ftp: Uri = "ftp://example.com/".parse().unwrap();
        assert!(OriginKey::from_uri(&ftp)
            .unwrap_err()
            .to_string()
            .contains("unsupported origin scheme"));

        assert_eq!(
            "origin host must not contain user information or a port",
            OriginKey::from_parts(Scheme::HTTPS, "example.com:443", None)
                .unwrap_err()
                .to_string()
        );
    }
}
