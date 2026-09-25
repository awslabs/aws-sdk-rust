/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types for HTTP headers

use crate::http::error::{HttpError, NonUtf8Header};
use std::borrow::Cow;
use std::fmt::Debug;
use std::str::FromStr;

/// Header names whose values must be redacted in Debug output to prevent
/// credential / session-token / customer-key leakage via tracing.
const DENYLIST: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "x-amz-security-token",
    "cookie",
    "set-cookie",
    "x-amz-server-side-encryption-customer-key",
    "x-amz-server-side-encryption-customer-key-md5",
    "x-amz-copy-source-server-side-encryption-customer-key",
    "x-amz-copy-source-server-side-encryption-customer-key-md5",
];

fn is_sensitive(name: &str) -> bool {
    DENYLIST.iter().any(|d| name.eq_ignore_ascii_case(d))
}

/// An immutable view of headers
///
/// Header values are stored exactly as received and are *not* required to be valid UTF-8: an HTTP
/// header value may contain any octet in `0x80..=0xFF` (obs-text, RFC 7230), and an arbitrary
/// sequence of those is not necessarily valid UTF-8. The string-typed accessors ([`get`](Headers::get), [`get_all`](Headers::get_all),
/// [`iter`](Headers::iter), [`remove`](Headers::remove)) therefore yield only values that are
/// valid UTF-8, and skip those that are not. Use the corresponding byte accessors
/// ([`get_bytes`](Headers::get_bytes), [`get_all_bytes`](Headers::get_all_bytes),
/// [`iter_bytes`](Headers::iter_bytes)) to observe every value.
///
/// Consequently [`len`](Headers::len) and [`contains_key`](Headers::contains_key) count and
/// report values the string accessors skip.
#[derive(Clone, Default)]
pub struct Headers {
    pub(super) headers: http_1x::HeaderMap<HeaderValue>,
}

impl Debug for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (key, value) in self.headers.iter() {
            let name = key.as_str();
            if is_sensitive(name) {
                map.entry(
                    &name,
                    &format_args!("** redacted (length={}) **", value.as_bytes().len()),
                );
            } else {
                match value.try_as_str() {
                    Some(value) => map.entry(&name, &value),
                    None => map.entry(
                        &name,
                        &format_args!("** non-utf8 (length={}) **", value.as_bytes().len()),
                    ),
                };
            }
        }
        map.finish()
    }
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a str, &'a str);
    type IntoIter = HeadersIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeadersIter {
            inner: self.headers.iter(),
        }
    }
}

/// An Iterator over headers
pub struct HeadersIter<'a> {
    inner: http_1x::header::Iter<'a, HeaderValue>,
}

impl<'a> Iterator for HeadersIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        // Values that are not valid UTF-8 are skipped; use `Headers::iter_bytes` to see them.
        loop {
            let (name, value) = self.inner.next()?;
            if let Some(value) = value.try_as_str() {
                return Some((name.as_str(), value));
            }
        }
    }
}

impl Headers {
    /// Create an empty header map
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "http-1x")]
    pub(crate) fn http1_headermap(self) -> http_1x::HeaderMap {
        let mut headers = http_1x::HeaderMap::new();
        headers.reserve(self.headers.len());
        headers.extend(self.headers.into_iter().map(|(k, v)| (k, v.into_http1x())));
        headers
    }

    #[cfg(feature = "http-02x")]
    pub(crate) fn http0_headermap(self) -> http_02x::HeaderMap {
        let mut headers = http_02x::HeaderMap::new();
        headers.reserve(self.headers.len());
        headers.extend(self.headers.into_iter().map(|(k, v)| {
            (
                k.map(|n| {
                    http_02x::HeaderName::from_bytes(n.as_str().as_bytes()).expect("proven valid")
                }),
                v.into_http02x(),
            )
        }));
        headers
    }

    /// Returns the value for a given key
    ///
    /// Returns `None` if the header is absent, or if its value is not valid UTF-8; use
    /// [`get_bytes`](Self::get_bytes) to read a value of any encoding.
    ///
    /// If multiple values are associated, the first value is returned
    /// See [HeaderMap::get](http_1x::HeaderMap::get)
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.headers.get(key.as_ref()).and_then(|v| v.try_as_str())
    }

    /// Returns the value for a given key, distinguishing an unreadable value from an absent header
    ///
    /// `Some(Ok(_))` is a value that is valid UTF-8, `Some(Err(_))` is the raw octets of one that is
    /// not, and `None` means the header is absent. [`get`](Self::get) collapses the first two into
    /// `None`, so use this where the difference matters.
    ///
    /// If multiple values are associated, the first value is returned.
    pub fn try_get(&self, key: impl AsRef<str>) -> Option<Result<&str, &[u8]>> {
        self.headers
            .get(key.as_ref())
            .map(|value| match value.try_as_str() {
                Some(value) => Ok(value),
                None => Err(value.as_bytes()),
            })
    }

    /// Returns all values for a given key
    ///
    /// Values that are not valid UTF-8 are skipped; use
    /// [`get_all_bytes`](Self::get_all_bytes) to read values of any encoding.
    pub fn get_all(&self, key: impl AsRef<str>) -> impl Iterator<Item = &str> {
        self.headers
            .get_all(key.as_ref())
            .iter()
            .filter_map(|v| v.try_as_str())
    }

    /// Returns the value for a given key as raw bytes
    ///
    /// Unlike [`get`](Self::get), the returned bytes are not required to be valid UTF-8.
    ///
    /// If multiple values are associated, the first value is returned.
    pub fn get_bytes(&self, key: impl AsRef<str>) -> Option<&[u8]> {
        self.headers.get(key.as_ref()).map(|v| v.as_bytes())
    }

    /// Returns all values for a given key as raw bytes
    ///
    /// Unlike [`get_all`](Self::get_all), the returned bytes are not required to be valid UTF-8.
    pub fn get_all_bytes(&self, key: impl AsRef<str>) -> impl Iterator<Item = &[u8]> {
        self.headers
            .get_all(key.as_ref())
            .iter()
            .map(|v| v.as_bytes())
    }

    /// Returns an iterator over the headers
    pub fn iter(&self) -> HeadersIter<'_> {
        HeadersIter {
            inner: self.headers.iter(),
        }
    }

    /// Returns an iterator over the headers, pairing each name with its raw value bytes
    ///
    /// Unlike [`iter`](Self::iter), the returned bytes are not required to be valid UTF-8.
    pub fn iter_bytes(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.headers.iter().map(|(k, v)| (k.as_str(), v.as_bytes()))
    }

    /// Returns the total number of **values** stored in the map
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Returns true if there are no headers
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if this header is present
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.headers.contains_key(key.as_ref())
    }

    /// Insert a value into the headers structure.
    ///
    /// This will *replace* any existing value for this key. Returns the previous associated value if any.
    ///
    /// # Panics
    /// If the key is not valid ASCII, or if the value is not valid UTF-8, this function will panic.
    pub fn insert(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Option<String> {
        let key = header_name(key, false).unwrap();
        let value = header_value(value.into_maybe_static().unwrap(), false).unwrap();
        self.headers
            .insert(key, value)
            .and_then(|old_value| old_value.try_as_str().map(str::to_string))
    }

    /// Insert a value into the headers structure.
    ///
    /// This will *replace* any existing value for this key. Returns the previous associated value if any.
    ///
    /// If the key is not valid ASCII, or if the value is not valid UTF-8, this function will return an error.
    pub fn try_insert(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Result<Option<String>, HttpError> {
        let key = header_name(key, true)?;
        let value = header_value(value.into_maybe_static()?, true)?;
        Ok(self
            .headers
            .insert(key, value)
            .and_then(|old_value| old_value.try_as_str().map(str::to_string)))
    }

    /// Appends a value to a given key
    ///
    /// # Panics
    /// If the key is not valid ASCII, or if the value is not valid UTF-8, this function will panic.
    pub fn append(&mut self, key: impl AsHeaderComponent, value: impl AsHeaderComponent) -> bool {
        let key = header_name(key.into_maybe_static().unwrap(), false).unwrap();
        let value = header_value(value.into_maybe_static().unwrap(), false).unwrap();
        self.headers.append(key, value)
    }

    /// Appends a value to a given key
    ///
    /// If the key is not valid ASCII, or if the value is not valid UTF-8, this function will return an error.
    pub fn try_append(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Result<bool, HttpError> {
        let key = header_name(key.into_maybe_static()?, true)?;
        let value = header_value(value.into_maybe_static()?, true)?;
        Ok(self.headers.append(key, value))
    }

    /// Removes all headers with a given key
    ///
    /// If there are multiple entries for this key, the first entry is returned. Returns `None`
    /// if the first value is not valid UTF-8; the headers are removed either way.
    pub fn remove(&mut self, key: impl AsRef<str>) -> Option<String> {
        self.headers
            .remove(key.as_ref())
            .and_then(|h| h.try_as_str().map(str::to_string))
    }
}

#[cfg(feature = "http-02x")]
impl TryFrom<http_02x::HeaderMap> for Headers {
    type Error = HttpError;

    fn try_from(value: http_02x::HeaderMap) -> Result<Self, Self::Error> {
        // Values are admitted regardless of encoding; see `Headers` for how non-UTF-8 values
        // surface to readers.
        //
        // `http` 0.2.x accepts some header names that `http` 1.x rejects (for example names
        // containing `"`). Convert fallibly and surface an error instead of panicking.
        //
        // A `None` key in `HeaderMap`'s iterator means "same name as the previous entry"
        // (multi-value headers), so the converted names are collected in order before being
        // extended into the map to preserve that association.
        let converted: Vec<(Option<http_1x::HeaderName>, HeaderValue)> = value
            .into_iter()
            .map(|(k, v)| {
                let name = k
                    .map(|n| http_1x::HeaderName::from_bytes(n.as_str().as_bytes()))
                    .transpose()
                    .map_err(HttpError::invalid_header_name)?;
                Ok((name, HeaderValue::from_http02x(v)))
            })
            .collect::<Result<_, HttpError>>()?;
        let mut headers: http_1x::HeaderMap<HeaderValue> = Default::default();
        headers.extend(converted);
        Ok(Headers { headers })
    }
}

#[cfg(feature = "http-1x")]
impl TryFrom<http_1x::HeaderMap> for Headers {
    type Error = HttpError;

    fn try_from(value: http_1x::HeaderMap) -> Result<Self, Self::Error> {
        // Values are admitted regardless of encoding; see `Headers` for how non-UTF-8 values
        // surface to readers. This conversion is infallible, but the signature is retained
        // because header names may be rejected by the `http` 0.2.x conversion above.
        let mut headers: http_1x::HeaderMap<HeaderValue> = Default::default();
        headers.extend(
            value
                .into_iter()
                .map(|(k, v)| (k, HeaderValue::from_http1x(v))),
        );
        Ok(Headers { headers })
    }
}

use sealed::AsHeaderComponent;

mod sealed {
    use super::*;
    /// Trait defining things that may be converted into a header component (name or value)
    pub trait AsHeaderComponent {
        /// If the component can be represented as a Cow<'static, str>, return it
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError>;

        /// Return a string reference to this header
        fn as_str(&self) -> Result<&str, HttpError>;

        /// If a component is already internally represented as a `http_1x::HeaderName`, return it
        fn repr_as_http1x_header_name(self) -> Result<http_1x::HeaderName, Self>
        where
            Self: Sized,
        {
            Err(self)
        }
    }

    impl AsHeaderComponent for &'static str {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Borrowed(self))
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            Ok(self)
        }
    }

    impl AsHeaderComponent for String {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Owned(self))
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            Ok(self)
        }
    }

    impl AsHeaderComponent for Cow<'static, str> {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(self)
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            Ok(self.as_ref())
        }
    }

    #[cfg(feature = "http-02x")]
    impl AsHeaderComponent for http_02x::HeaderValue {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Owned(
                std::str::from_utf8(self.as_bytes())
                    .map_err(|err| {
                        HttpError::non_utf8_header(NonUtf8Header::new(
                            self.as_bytes().to_vec(),
                            err,
                        ))
                    })?
                    .to_string(),
            ))
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            std::str::from_utf8(self.as_bytes()).map_err(|err| {
                HttpError::non_utf8_header(NonUtf8Header::new(self.as_bytes().to_vec(), err))
            })
        }
    }

    #[cfg(feature = "http-02x")]
    impl AsHeaderComponent for http_02x::HeaderName {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(self.to_string().into())
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            Ok(self.as_ref())
        }
    }

    impl AsHeaderComponent for http_1x::HeaderName {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(self.to_string().into())
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            Ok(self.as_ref())
        }

        fn repr_as_http1x_header_name(self) -> Result<http_1x::HeaderName, Self>
        where
            Self: Sized,
        {
            Ok(self)
        }
    }

    impl AsHeaderComponent for http_1x::HeaderValue {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Owned(
                std::str::from_utf8(self.as_bytes())
                    .map_err(|err| {
                        HttpError::non_utf8_header(NonUtf8Header::new(
                            self.as_bytes().to_vec(),
                            err,
                        ))
                    })?
                    .to_string(),
            ))
        }

        fn as_str(&self) -> Result<&str, HttpError> {
            std::str::from_utf8(self.as_bytes()).map_err(|err| {
                HttpError::non_utf8_header(NonUtf8Header::new(self.as_bytes().to_vec(), err))
            })
        }
    }
}

mod header_value {
    use super::*;

    /// HeaderValue type
    ///
    /// **Note**: Unlike `HeaderValue` in `http`, this only supports UTF-8 header values
    #[derive(Debug, Clone)]
    pub struct HeaderValue {
        _private: Inner,
    }

    #[derive(Debug, Clone)]
    enum Inner {
        #[cfg(feature = "http-02x")]
        H0(http_02x::HeaderValue),
        H1(http_1x::HeaderValue),
    }

    impl HeaderValue {
        // Encoding is not validated here. Callers that require UTF-8 go through
        // `AsHeaderComponent`, which validates before reaching this point.
        #[cfg(feature = "http-02x")]
        pub(crate) fn from_http02x(value: http_02x::HeaderValue) -> Self {
            Self {
                _private: Inner::H0(value),
            }
        }

        // Encoding is not validated here; see `from_http02x`.
        pub(crate) fn from_http1x(value: http_1x::HeaderValue) -> Self {
            Self {
                _private: Inner::H1(value),
            }
        }

        #[cfg(feature = "http-02x")]
        pub(crate) fn into_http02x(self) -> http_02x::HeaderValue {
            match self._private {
                Inner::H0(v) => v,
                Inner::H1(v) => http_02x::HeaderValue::from_maybe_shared(v).expect("unreachable"),
            }
        }

        #[allow(dead_code)]
        pub(crate) fn into_http1x(self) -> http_1x::HeaderValue {
            match self._private {
                Inner::H1(v) => v,
                #[cfg(feature = "http-02x")]
                Inner::H0(v) => http_1x::HeaderValue::from_maybe_shared(v).expect("unreachable"),
            }
        }
    }

    impl AsRef<str> for HeaderValue {
        /// # Panics
        /// If the value is not valid UTF-8. See [`HeaderValue::as_str`].
        fn as_ref(&self) -> &str {
            std::str::from_utf8(self.as_bytes()).expect("header value is not valid UTF-8")
        }
    }

    impl From<HeaderValue> for String {
        fn from(value: HeaderValue) -> Self {
            value.as_ref().to_string()
        }
    }

    impl HeaderValue {
        /// Returns the string representation of this header value
        ///
        /// # Panics
        /// If the value is not valid UTF-8. A `HeaderValue` stored in a [`Headers`] may be of any
        /// encoding, so prefer [`try_as_str`](Self::try_as_str) or
        /// [`as_bytes`](Self::as_bytes) unless the value is one you constructed yourself, which
        /// is necessarily valid UTF-8 because the only public constructors ([`FromStr`] and
        /// [`TryFrom<String>`]) take a `str`.
        ///
        /// No accessor on [`Headers`] hands out a `HeaderValue`, so this is not reachable through
        /// one. Note [`From<HeaderValue> for String`](String::from) panics for the same reason.
        pub fn as_str(&self) -> &str {
            self.as_ref()
        }

        /// Returns the bytes of this header value exactly as they were received
        ///
        /// Unlike [`as_str`](Self::as_str), this is always available.
        pub fn as_bytes(&self) -> &[u8] {
            match &self._private {
                #[cfg(feature = "http-02x")]
                Inner::H0(v) => v.as_bytes(),
                Inner::H1(v) => v.as_bytes(),
            }
        }

        /// Returns the string representation of this header value, or `None` if it is not
        /// valid UTF-8
        pub fn try_as_str(&self) -> Option<&str> {
            std::str::from_utf8(self.as_bytes()).ok()
        }
    }

    impl FromStr for HeaderValue {
        type Err = HttpError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            HeaderValue::try_from(s.to_string())
        }
    }

    impl TryFrom<String> for HeaderValue {
        type Error = HttpError;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            Ok(HeaderValue::from_http1x(
                http_1x::HeaderValue::try_from(value).map_err(HttpError::invalid_header_value)?,
            ))
        }
    }
}

pub use header_value::HeaderValue;

type MaybeStatic = Cow<'static, str>;

fn header_name(
    name: impl AsHeaderComponent,
    panic_safe: bool,
) -> Result<http_1x::HeaderName, HttpError> {
    name.repr_as_http1x_header_name().or_else(|name| {
        name.into_maybe_static().and_then(|mut cow| {
            if cow.chars().any(|c| c.is_ascii_uppercase()) {
                cow = Cow::Owned(cow.to_ascii_uppercase());
            }
            match cow {
                Cow::Borrowed(s) if panic_safe => {
                    http_1x::HeaderName::try_from(s).map_err(HttpError::invalid_header_name)
                }
                Cow::Borrowed(static_s) => Ok(http_1x::HeaderName::from_static(static_s)),
                Cow::Owned(s) => {
                    http_1x::HeaderName::try_from(s).map_err(HttpError::invalid_header_name)
                }
            }
        })
    })
}

fn header_value(value: MaybeStatic, panic_safe: bool) -> Result<HeaderValue, HttpError> {
    let header = match value {
        Cow::Borrowed(b) if panic_safe => {
            http_1x::HeaderValue::try_from(b).map_err(HttpError::invalid_header_value)?
        }
        Cow::Borrowed(b) => http_1x::HeaderValue::from_static(b),
        Cow::Owned(s) => {
            http_1x::HeaderValue::try_from(s).map_err(HttpError::invalid_header_value)?
        }
    };
    // `value` is a `Cow<'static, str>`, so the result is valid UTF-8 by construction.
    Ok(HeaderValue::from_http1x(header))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_can_be_any_string() {
        let _: HeaderValue = "😹".parse().expect("can be any string");
        let _: HeaderValue = "abcd".parse().expect("can be any string");
        let _ = "a\nb"
            .parse::<HeaderValue>()
            .expect_err("cannot contain control characters");
    }

    #[test]
    fn no_panic_insert_upper_case_header_name() {
        let mut headers = Headers::new();
        headers.insert("I-Have-Upper-Case", "foo");
    }
    #[test]
    fn no_panic_append_upper_case_header_name() {
        let mut headers = Headers::new();
        headers.append("I-Have-Upper-Case", "foo");
    }

    #[test]
    #[should_panic]
    fn panic_insert_invalid_ascii_key() {
        let mut headers = Headers::new();
        headers.insert("💩", "foo");
    }
    #[test]
    #[should_panic]
    fn panic_insert_invalid_header_value() {
        let mut headers = Headers::new();
        headers.insert("foo", "💩");
    }
    #[test]
    #[should_panic]
    fn panic_append_invalid_ascii_key() {
        let mut headers = Headers::new();
        headers.append("💩", "foo");
    }
    #[test]
    #[should_panic]
    fn panic_append_invalid_header_value() {
        let mut headers = Headers::new();
        headers.append("foo", "💩");
    }

    #[test]
    fn no_panic_try_insert_invalid_ascii_key() {
        let mut headers = Headers::new();
        assert!(headers.try_insert("💩", "foo").is_err());
    }
    #[test]
    fn no_panic_try_insert_invalid_header_value() {
        let mut headers = Headers::new();
        assert!(headers
            .try_insert(
                "foo",
                // Valid header value with invalid UTF-8
                http_1x::HeaderValue::from_bytes(&[0xC0, 0x80]).unwrap()
            )
            .is_err());
    }
    #[test]
    fn no_panic_try_append_invalid_ascii_key() {
        let mut headers = Headers::new();
        assert!(headers.try_append("💩", "foo").is_err());
    }
    #[test]
    fn no_panic_try_append_invalid_header_value() {
        let mut headers = Headers::new();
        assert!(headers
            .try_append(
                "foo",
                // Valid header value with invalid UTF-8
                http_1x::HeaderValue::from_bytes(&[0xC0, 0x80]).unwrap()
            )
            .is_err());
    }

    #[test]
    fn header_value_exposes_bytes_and_checked_str() {
        let value: HeaderValue = "hello".parse().expect("valid");
        assert_eq!(b"hello", value.as_bytes());
        assert_eq!(Some("hello"), value.try_as_str());
        assert_eq!("hello", value.as_str());
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn byte_accessors_agree_with_str_accessors() {
        let mut map = http_1x::HeaderMap::new();
        map.append("single", http_1x::HeaderValue::from_static("v1"));
        map.append("multi", http_1x::HeaderValue::from_static("m1"));
        map.append("multi", http_1x::HeaderValue::from_static("m2"));
        let headers = Headers::try_from(map).expect("all values are valid UTF-8");

        assert_eq!(Some(b"v1".as_slice()), headers.get_bytes("single"));
        assert_eq!(
            headers.get("single").map(str::as_bytes),
            headers.get_bytes("single")
        );

        let all_bytes: Vec<_> = headers.get_all_bytes("multi").collect();
        assert_eq!(vec![b"m1".as_slice(), b"m2".as_slice()], all_bytes);
        let all_str: Vec<_> = headers.get_all("multi").map(str::as_bytes).collect();
        assert_eq!(all_str, all_bytes);

        assert_eq!(None, headers.get_bytes("absent"));

        let mut from_bytes: Vec<_> = headers.iter_bytes().collect();
        from_bytes.sort();
        let mut from_str: Vec<_> = headers.iter().map(|(k, v)| (k, v.as_bytes())).collect();
        from_str.sort();
        assert_eq!(from_str, from_bytes);
    }

    // Reported in review: `insert`/`try_insert` return the previous value, which reaches the
    // panicking `AsRef<str>` when that value was admitted as non-UTF-8.
    #[cfg(feature = "http-1x")]
    #[test]
    fn replacing_a_non_utf8_value_does_not_panic() {
        for replace in [
            (|h: &mut Headers| {
                h.insert("bad", "replacement");
            }) as fn(&mut Headers),
            |h: &mut Headers| {
                h.try_insert("bad", "replacement").expect("valid");
            },
        ] {
            let mut map = http_1x::HeaderMap::new();
            map.insert("bad", non_utf8_header_value());
            let mut headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");
            let res =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| replace(&mut headers)));
            assert!(res.is_ok(), "replacing an admitted value must not panic");
        }
    }

    proptest::proptest! {
        #[test]
        fn insert_header_prop_test(input in ".*") {
            let mut headers = Headers::new();
            let _ = headers.try_insert(input.clone(), input);
        }

        #[test]
        fn append_header_prop_test(input in ".*") {
            let mut headers = Headers::new();
            let _ = headers.try_append(input.clone(), input);
        }
    }

    // `http` 0.2.x accepts header names (e.g. containing `"`) that `http` 1.x rejects. Converting
    // such a map must return an `Err`, not panic.
    #[cfg(feature = "http-02x")]
    #[test]
    fn converting_an_http02x_headermap_never_panics() {
        let name = http_02x::HeaderName::from_bytes(b"a\"b").expect("http 0.2.x accepts this");
        let mut map = http_02x::HeaderMap::new();
        map.insert(name, http_02x::HeaderValue::from_static("v"));
        let res = std::panic::catch_unwind(|| Headers::try_from(map));
        assert!(
            res.is_ok(),
            "TryFrom<http_02x::HeaderMap> for Headers panicked on a name that http 0.2.x \
             considers valid but http 1.x does not; a TryFrom should return Err"
        );
        assert!(
            res.unwrap().is_err(),
            "expected an Err for a header name that http 1.x rejects"
        );
    }

    // A lone 0xE9 is a valid HTTP header octet (obs-text per RFC 7230) but is not valid UTF-8.
    // Every user of this fixture builds a `HeaderMap` from one of the `http` crates, so it is dead
    // code when neither is enabled.
    #[cfg(any(feature = "http-1x", feature = "http-02x"))]
    const NON_UTF8_VALUE: &[u8] = b"value-\xe9";

    #[cfg(feature = "http-1x")]
    fn non_utf8_header_value() -> http_1x::HeaderValue {
        http_1x::HeaderValue::from_bytes(NON_UTF8_VALUE).expect("valid header octets")
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn non_utf8_values_are_admitted_and_readable_as_bytes() {
        let mut map = http_1x::HeaderMap::new();
        map.insert("ok", http_1x::HeaderValue::from_static("v"));
        map.insert("bad", non_utf8_header_value());
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        // The value is present...
        assert!(headers.contains_key("bad"));
        assert_eq!(Some(NON_UTF8_VALUE), headers.get_bytes("bad"));
        // ...but is not offered as a string.
        assert_eq!(None, headers.get("bad"));

        assert_eq!(Some("v"), headers.get("ok"));
    }

    #[cfg(feature = "http-02x")]
    #[test]
    fn non_utf8_values_are_admitted_from_an_http02x_headermap() {
        let mut map = http_02x::HeaderMap::new();
        map.insert(
            "bad",
            http_02x::HeaderValue::from_bytes(NON_UTF8_VALUE).expect("valid header octets"),
        );
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");
        assert_eq!(Some(NON_UTF8_VALUE), headers.get_bytes("bad"));
        assert_eq!(None, headers.get("bad"));
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn a_non_utf8_value_does_not_hide_the_other_values_of_its_header() {
        let mut map = http_1x::HeaderMap::new();
        map.append("multi", http_1x::HeaderValue::from_static("v1"));
        map.append("multi", non_utf8_header_value());
        map.append("multi", http_1x::HeaderValue::from_static("v3"));
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        assert_eq!(
            vec!["v1", "v3"],
            headers.get_all("multi").collect::<Vec<_>>()
        );
        assert_eq!(
            vec![b"v1".as_slice(), NON_UTF8_VALUE, b"v3".as_slice()],
            headers.get_all_bytes("multi").collect::<Vec<_>>()
        );
        // `get` returns the first value, which here is valid UTF-8.
        assert_eq!(Some("v1"), headers.get("multi"));
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn try_get_distinguishes_unreadable_from_absent() {
        let mut map = http_1x::HeaderMap::new();
        map.insert("ok", http_1x::HeaderValue::from_static("v"));
        map.insert("bad", non_utf8_header_value());
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        assert_eq!(Some(Ok("v")), headers.try_get("ok"));
        assert_eq!(Some(Err(NON_UTF8_VALUE)), headers.try_get("bad"));
        assert_eq!(None, headers.try_get("absent"));

        // `get` cannot tell the last two apart, which is why `try_get` exists.
        assert_eq!(None, headers.get("bad"));
        assert_eq!(None, headers.get("absent"));
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn iter_skips_non_utf8_values_and_iter_bytes_does_not() {
        let mut map = http_1x::HeaderMap::new();
        map.insert("a", non_utf8_header_value());
        map.insert("b", http_1x::HeaderValue::from_static("v"));
        map.insert("c", non_utf8_header_value());
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        assert_eq!(vec![("b", "v")], headers.iter().collect::<Vec<_>>());
        assert_eq!(3, headers.iter_bytes().count());
        // `len` counts stored values, including those `iter` skips.
        assert_eq!(3, headers.len());
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn remove_drops_a_non_utf8_value_and_reports_none() {
        let mut map = http_1x::HeaderMap::new();
        map.insert("bad", non_utf8_header_value());
        let mut headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        assert_eq!(None, headers.remove("bad"));
        assert!(
            !headers.contains_key("bad"),
            "the header is removed regardless"
        );
    }

    #[cfg(feature = "http-1x")]
    #[test]
    fn debug_marks_non_utf8_values_without_panicking() {
        let mut map = http_1x::HeaderMap::new();
        map.insert("bad", non_utf8_header_value());
        let headers = Headers::try_from(map).expect("non-UTF-8 values are admitted");

        let output = format!("{headers:?}");
        assert!(output.contains("bad"), "{output}");
        assert!(output.contains("non-utf8"), "{output}");
    }

    // Multi-value headers rely on the `None`-key semantics of `HeaderMap`'s iterator; make sure the
    // fallible conversion preserves all values for a repeated name.
    #[cfg(feature = "http-02x")]
    #[test]
    fn converting_an_http02x_headermap_preserves_multi_value_headers() {
        let mut map = http_02x::HeaderMap::new();
        map.append("multi", http_02x::HeaderValue::from_static("v1"));
        map.append("multi", http_02x::HeaderValue::from_static("v2"));
        let headers = Headers::try_from(map).expect("valid headers");
        let values: Vec<_> = headers.get_all("multi").collect();
        assert_eq!(values, vec!["v1", "v2"]);
    }
}

#[cfg(test)]
mod redaction_tests {
    use super::*;

    #[test]
    fn debug_redacts_authorization() {
        let mut headers = Headers::new();
        headers.insert(
            "authorization",
            "AWS4-HMAC-SHA256 Credential=AKIAXXX/.../Signature=SECRETSIGMARKER",
        );
        let output = format!("{:?}", headers);
        assert!(!output.contains("SECRETSIGMARKER"));
        assert!(output.contains("authorization"));
        assert!(output.contains("** redacted"));
    }

    #[test]
    fn debug_redacts_security_token() {
        let mut headers = Headers::new();
        headers.insert("x-amz-security-token", "IQoJb3JpZ2luSECRETTOKENMARKERzzz");
        let output = format!("{:?}", headers);
        assert!(!output.contains("SECRETTOKENMARKER"));
        assert!(output.contains("x-amz-security-token"));
        assert!(output.contains("length="));
    }

    #[test]
    fn debug_redacts_mixed_case_header_name() {
        let mut headers = Headers::new();
        headers.insert(
            "Authorization",
            "AWS4-HMAC-SHA256 Credential=AKIAXXX/.../Signature=SECRETSIGMARKER",
        );
        let output = format!("{:?}", headers);
        assert!(!output.contains("SECRETSIGMARKER"));
        assert!(output.contains("** redacted"));
    }

    #[test]
    fn debug_preserves_non_sensitive_headers() {
        let mut headers = Headers::new();
        headers.insert("host", "example.com");
        headers.insert("x-amz-user-agent", "aws-sdk-rust/1.0");
        let output = format!("{:?}", headers);
        assert!(output.contains("example.com"));
        assert!(output.contains("aws-sdk-rust/1.0"));
    }

    #[test]
    fn debug_handles_sse_customer_key() {
        let mut headers = Headers::new();
        headers.insert(
            "x-amz-server-side-encryption-customer-key",
            "BASE64KEYMARKER_DO_NOT_LOG",
        );
        let output = format!("{:?}", headers);
        assert!(!output.contains("BASE64KEYMARKER_DO_NOT_LOG"));
        assert!(output.contains("x-amz-server-side-encryption-customer-key"));
        assert!(output.contains("** redacted"));
    }

    #[test]
    fn debug_includes_length() {
        let value = "exactly-twenty-chars";
        assert_eq!(value.len(), 20);
        let mut headers = Headers::new();
        headers.insert("authorization", value);
        let output = format!("{:?}", headers);
        assert!(output.contains("length=20"));
    }
}
