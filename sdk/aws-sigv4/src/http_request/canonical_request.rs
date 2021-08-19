/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use super::{
    Error, PayloadChecksumKind, SignableBody, SigningSettings, UriEncoding, HMAC_256,
    X_AMZ_CONTENT_SHA_256, X_AMZ_DATE, X_AMZ_SECURITY_TOKEN,
};
use crate::date_fmt::{format_date, format_date_time, parse_date, parse_date_time};
use crate::sign::sha256_hex_string;
use chrono::{Date, DateTime, Utc};
use http::header::{HeaderName, USER_AGENT};
use http::{HeaderMap, HeaderValue, Method, Request};
use percent_encoding::{AsciiSet, CONTROLS};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;

const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

/// base set of characters that must be URL encoded
const BASE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'/')
    // RFC-3986 §3.3 allows sub-delims (defined in section2.2) to be in the path component.
    // This includes both colon ':' and comma ',' characters.
    // Smithy protocol tests & AWS services percent encode these expected values. Signing
    // will fail if these values are not percent encoded
    .add(b':')
    .add(b',')
    .add(b'?')
    .add(b'#')
    .add(b'[')
    .add(b']')
    .add(b'@')
    .add(b'!')
    .add(b'$')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b';')
    .add(b'=')
    .add(b'%');

fn percent_encode(value: &str) -> String {
    percent_encoding::percent_encode(&value.as_bytes(), BASE_SET).to_string()
}

pub struct AddedHeaders {
    pub x_amz_date: HeaderValue,
    pub x_amz_content_256: Option<HeaderValue>,
    pub x_amz_security_token: Option<HeaderValue>,
}

#[derive(Default, Debug, PartialEq)]
pub struct CanonicalRequest {
    pub method: Method,
    pub path: String,
    pub params: String,
    pub headers: HeaderMap,
    pub signed_headers: SignedHeaders,
    pub payload_hash: String,
}

impl CanonicalRequest {
    /// Construct a CanonicalRequest from an HttpRequest and a signable body
    ///
    /// This function returns 2 things:
    /// 1. The canonical request to use for signing
    /// 2. `AddedHeaders`, a struct recording the additional headers that were added. These will
    ///    behavior returned to the top level caller. If the caller wants to create a
    ///    presigned URL, they can apply these parameters to the query string.
    ///
    /// ## Behavior
    /// There are several settings which alter signing behavior:
    /// - If a `security_token` is provided as part of the credentials it will be included in the signed headers
    /// - If `settings.uri_encoding` specifies double encoding, `%` in the URL will be rencoded as
    /// `%25`
    /// - If settings.payload_checksum_kind is XAmzSha256, add a x-amz-content-sha256 with the body
    /// checksum. This is the same checksum used as the "payload_hash" in the canonical request
    pub fn from<B>(
        req: &Request<B>,
        body: SignableBody,
        settings: &SigningSettings,
        date: DateTime<Utc>,
        security_token: Option<&str>,
    ) -> Result<(CanonicalRequest, AddedHeaders), Error> {
        // Path encoding: if specified, rencode % as %25
        // Set method and path into CanonicalRequest
        let path = req.uri().path();
        let path = match settings.uri_encoding {
            // The string is already URI encoded, we don't need to encode everything again, just `%`
            UriEncoding::Double => path.replace('%', "%25"),
            UriEncoding::Single => path.to_string(),
        };
        let mut creq = CanonicalRequest {
            method: req.method().clone(),
            path,
            ..Default::default()
        };

        if let Some(query) = req.uri().query() {
            let mut first = true;
            let mut out = String::new();
            let mut params: Vec<(Cow<str>, Cow<str>)> =
                form_urlencoded::parse(query.as_bytes()).collect();
            // Sort by param name, and then by param value
            params.sort();
            for (key, value) in params {
                if !first {
                    out.push('&');
                }
                first = false;

                out.push_str(&percent_encode(&key));
                out.push('=');
                out.push_str(&percent_encode(&value));
            }
            creq.params = out;
        }

        // Payload hash computation
        //
        // Based on the input body, set the payload_hash of the canonical request:
        // Either:
        // - compute a hash
        // - use the precomputed hash
        // - use `UnsignedPayload`
        let payload_hash = match body {
            SignableBody::Bytes(data) => sha256_hex_string(data),
            SignableBody::Precomputed(digest) => digest,
            SignableBody::UnsignedPayload => UNSIGNED_PAYLOAD.to_string(),
        };
        creq.payload_hash = payload_hash;

        // Header computation:
        // The canonical request will include headers not present in the input. We need to clone
        // the headers from the original request and add:
        // - x-amz-date
        // - x-amz-security-token (if provided)
        // - x-amz-content-sha256 (if requested by signing settings)
        let mut canonical_headers = req.headers().clone();
        let x_amz_date = HeaderName::from_static(X_AMZ_DATE);
        let date_header =
            HeaderValue::try_from(format_date_time(&date)).expect("date is valid header value");
        canonical_headers.insert(x_amz_date, date_header.clone());
        // to return headers to the user, record which headers we added
        let mut out = AddedHeaders {
            x_amz_date: date_header,
            x_amz_content_256: None,
            x_amz_security_token: None,
        };

        if let Some(security_token) = security_token {
            let mut sec_header = HeaderValue::from_str(security_token)?;
            sec_header.set_sensitive(true);
            canonical_headers.insert(X_AMZ_SECURITY_TOKEN, sec_header.clone());
            out.x_amz_security_token = Some(sec_header);
        }

        if settings.payload_checksum_kind == PayloadChecksumKind::XAmzSha256 {
            let header = HeaderValue::from_str(&creq.payload_hash)?;
            canonical_headers.insert(X_AMZ_CONTENT_SHA_256, header.clone());
            out.x_amz_content_256 = Some(header);
        }

        let mut signed_headers = Vec::with_capacity(canonical_headers.len());
        for (name, _) in &canonical_headers {
            // The user agent header should not be signed because it may be altered by proxies
            if name != USER_AGENT {
                signed_headers.push(CanonicalHeaderName(name.clone()));
            }
        }
        creq.signed_headers = SignedHeaders::new(signed_headers);
        creq.headers = canonical_headers;
        Ok((creq, out))
    }
}

impl fmt::Display for CanonicalRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.method)?;
        writeln!(f, "{}", self.path)?;
        writeln!(f, "{}", self.params)?;
        // write out _all_ the headers
        for header in &self.signed_headers.inner {
            // a missing header is a bug, so we should panic.
            let value = &self.headers[&header.0];
            write!(f, "{}:", header.0.as_str())?;
            writeln!(f, "{}", value.to_str().unwrap())?;
        }
        writeln!(f)?;
        // write out the signed headers
        write!(f, "{}", self.signed_headers.to_string())?;
        writeln!(f)?;
        write!(f, "{}", self.payload_hash)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct SignedHeaders {
    inner: Vec<CanonicalHeaderName>,
}

impl SignedHeaders {
    fn new(mut inner: Vec<CanonicalHeaderName>) -> Self {
        inner.sort();
        SignedHeaders { inner }
    }
}

impl fmt::Display for SignedHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.inner.iter().peekable();
        while let Some(next) = iter.next() {
            match iter.peek().is_some() {
                true => write!(f, "{};", next.0.as_str())?,
                false => write!(f, "{}", next.0.as_str())?,
            };
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CanonicalHeaderName(HeaderName);

impl PartialOrd for CanonicalHeaderName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CanonicalHeaderName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(&other.0.as_str())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Scope<'a> {
    pub date: Date<Utc>,
    pub region: &'a str,
    pub service: &'a str,
}

impl<'a> fmt::Display for Scope<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/aws4_request",
            format_date(&self.date),
            self.region,
            self.service
        )
    }
}

impl<'a> TryFrom<&'a str> for Scope<'a> {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Scope<'a>, Self::Error> {
        let mut scopes = s.split('/');
        let date = parse_date(scopes.next().expect("missing date"))?;
        let region = scopes.next().expect("missing region");
        let service = scopes.next().expect("missing service");

        let scope = Scope {
            date,
            region,
            service,
        };

        Ok(scope)
    }
}

#[derive(PartialEq, Debug)]
pub struct StringToSign<'a> {
    pub scope: Scope<'a>,
    pub date: DateTime<Utc>,
    pub region: &'a str,
    pub service: &'a str,
    pub hashed_creq: &'a str,
}

impl<'a> TryFrom<&'a str> for StringToSign<'a> {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let lines = s.lines().collect::<Vec<&str>>();
        let date = parse_date_time(&lines[1])?;
        let scope: Scope = TryFrom::try_from(lines[2])?;
        let hashed_creq = &lines[3];

        let sts = StringToSign {
            date,
            region: scope.region,
            service: scope.service,
            scope,
            hashed_creq,
        };

        Ok(sts)
    }
}

impl<'a> StringToSign<'a> {
    pub(crate) fn new(
        date: DateTime<Utc>,
        region: &'a str,
        service: &'a str,
        hashed_creq: &'a str,
    ) -> Self {
        let scope = Scope {
            date: date.date(),
            region,
            service,
        };
        Self {
            scope,
            date,
            region,
            service,
            hashed_creq,
        }
    }
}

impl<'a> fmt::Display for StringToSign<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}\n{}",
            HMAC_256,
            format_date_time(&self.date),
            self.scope.to_string(),
            self.hashed_creq
        )
    }
}
