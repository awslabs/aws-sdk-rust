/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use super::query_writer::QueryWriter;
use super::{Error, PayloadChecksumKind, SignableBody, SignatureLocation, SigningParams};
use crate::date_fmt::{format_date, format_date_time, parse_date, parse_date_time};
use crate::http_request::sign::SignableRequest;
use crate::http_request::PercentEncodingMode;
use crate::sign::sha256_hex_string;
use chrono::{Date, DateTime, Utc};
use http::header::{HeaderName, CONTENT_LENGTH, CONTENT_TYPE, HOST, USER_AGENT};
use http::{HeaderMap, HeaderValue, Method, Uri};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;

pub(crate) mod header {
    pub(crate) const X_AMZ_CONTENT_SHA_256: &str = "x-amz-content-sha256";
    pub(crate) const X_AMZ_DATE: &str = "x-amz-date";
    pub(crate) const X_AMZ_SECURITY_TOKEN: &str = "x-amz-security-token";
    pub(crate) const X_AMZ_USER_AGENT: &str = "x-amz-user-agent";
}

mod param {
    pub(crate) const X_AMZ_ALGORITHM: &str = "X-Amz-Algorithm";
    pub(crate) const X_AMZ_CREDENTIAL: &str = "X-Amz-Credential";
    pub(crate) const X_AMZ_DATE: &str = "X-Amz-Date";
    pub(crate) const X_AMZ_EXPIRES: &str = "X-Amz-Expires";
    pub(crate) const X_AMZ_SIGNED_HEADERS: &str = "X-Amz-SignedHeaders";
}

pub(crate) const HMAC_256: &str = "AWS4-HMAC-SHA256";

const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

#[derive(Debug, PartialEq)]
pub(super) struct HeaderValues<'a> {
    pub(super) content_sha256: Cow<'a, str>,
    pub(super) date_time: String,
    pub(super) security_token: Option<&'a str>,
    pub(super) signed_headers: SignedHeaders,
}

#[derive(Debug, PartialEq)]
pub(super) struct QueryParamValues<'a> {
    pub(super) algorithm: &'static str,
    pub(super) content_sha256: Cow<'a, str>,
    pub(super) credential: String,
    pub(super) date_time: String,
    pub(super) expires: String,
    pub(super) security_token: Option<&'a str>,
    pub(super) signed_headers: SignedHeaders,
}

#[derive(Debug, PartialEq)]
pub(super) enum SignatureValues<'a> {
    Headers(HeaderValues<'a>),
    QueryParams(QueryParamValues<'a>),
}

impl<'a> SignatureValues<'a> {
    pub(super) fn signed_headers(&self) -> &SignedHeaders {
        match self {
            SignatureValues::Headers(values) => &values.signed_headers,
            SignatureValues::QueryParams(values) => &values.signed_headers,
        }
    }

    fn content_sha256(&self) -> &str {
        match self {
            SignatureValues::Headers(values) => &values.content_sha256,
            SignatureValues::QueryParams(values) => &values.content_sha256,
        }
    }

    pub(super) fn as_headers(&self) -> Option<&HeaderValues<'_>> {
        match self {
            SignatureValues::Headers(values) => Some(&values),
            _ => None,
        }
    }

    pub(super) fn into_query_params(self) -> Result<QueryParamValues<'a>, Self> {
        match self {
            SignatureValues::QueryParams(values) => Ok(values),
            _ => Err(self),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct CanonicalRequest<'a> {
    pub(super) method: &'a Method,
    pub(super) path: Cow<'a, str>,
    pub(super) params: Option<String>,
    pub(super) headers: HeaderMap,
    pub(super) values: SignatureValues<'a>,
}

impl<'a> CanonicalRequest<'a> {
    /// Construct a CanonicalRequest from a [`SignableRequest`] and [`SigningParams`].
    ///
    /// The returned canonical request includes information required for signing as well
    /// as query parameters or header values that go along with the signature in a request.
    ///
    /// ## Behavior
    ///
    /// There are several settings which alter signing behavior:
    /// - If a `security_token` is provided as part of the credentials it will be included in the signed headers
    /// - If `settings.percent_encoding_mode` specifies double encoding, `%` in the URL will be re-encoded as `%25`
    /// - If `settings.payload_checksum_kind` is XAmzSha256, add a x-amz-content-sha256 with the body
    ///   checksum. This is the same checksum used as the "payload_hash" in the canonical request
    /// - `settings.signature_location` determines where the signature will be placed in a request,
    ///   and also alters the kinds of signing values that go along with it in the request.
    pub(super) fn from<'b>(
        req: &'b SignableRequest<'b>,
        params: &'b SigningParams<'b>,
    ) -> Result<CanonicalRequest<'b>, Error> {
        // Path encoding: if specified, re-encode % as %25
        // Set method and path into CanonicalRequest
        let path = req.uri().path();
        let path = match params.settings.percent_encoding_mode {
            // The string is already URI encoded, we don't need to encode everything again, just `%`
            PercentEncodingMode::Double => Cow::Owned(path.replace('%', "%25")),
            PercentEncodingMode::Single => Cow::Borrowed(path),
        };
        let payload_hash = Self::payload_hash(req.body());

        let date_time = format_date_time(&params.date_time);
        let (signed_headers, canonical_headers) =
            Self::headers(req, params, &payload_hash, &date_time)?;
        let signed_headers = SignedHeaders::new(signed_headers);
        let values = match params.settings.signature_location {
            SignatureLocation::Headers => SignatureValues::Headers(HeaderValues {
                content_sha256: payload_hash,
                date_time,
                security_token: params.security_token,
                signed_headers,
            }),
            SignatureLocation::QueryParams => SignatureValues::QueryParams(QueryParamValues {
                algorithm: "AWS4-HMAC-SHA256",
                content_sha256: payload_hash,
                credential: format!(
                    "{}/{}/{}/{}/aws4_request",
                    params.access_key,
                    format_date(&params.date_time.date()),
                    params.region,
                    params.service_name,
                ),
                date_time,
                expires: params
                    .settings
                    .expires_in
                    .expect("presigning requires expires_in")
                    .as_secs()
                    .to_string(),
                security_token: params.security_token,
                signed_headers,
            }),
        };
        let creq = CanonicalRequest {
            method: req.method(),
            path,
            params: Self::params(req.uri(), &values),
            headers: canonical_headers,
            values,
        };
        Ok(creq)
    }

    fn headers(
        req: &SignableRequest<'_>,
        params: &SigningParams<'_>,
        payload_hash: &str,
        date_time: &str,
    ) -> Result<(Vec<CanonicalHeaderName>, HeaderMap), Error> {
        // Header computation:
        // The canonical request will include headers not present in the input. We need to clone
        // the headers from the original request and add:
        // - host
        // - x-amz-date
        // - x-amz-security-token (if provided)
        // - x-amz-content-sha256 (if requested by signing settings)
        let mut canonical_headers = req.headers().clone();
        Self::insert_host_header(&mut canonical_headers, req.uri());

        if params.settings.signature_location == SignatureLocation::Headers {
            Self::insert_date_header(&mut canonical_headers, &date_time);

            if let Some(security_token) = params.security_token {
                let mut sec_header = HeaderValue::from_str(security_token)?;
                sec_header.set_sensitive(true);
                canonical_headers.insert(header::X_AMZ_SECURITY_TOKEN, sec_header);
            }

            if params.settings.payload_checksum_kind == PayloadChecksumKind::XAmzSha256 {
                let header = HeaderValue::from_str(&payload_hash)?;
                canonical_headers.insert(header::X_AMZ_CONTENT_SHA_256, header);
            }
        }

        let mut signed_headers = Vec::with_capacity(canonical_headers.len());
        for (name, _) in &canonical_headers {
            // The user agent header should not be signed because it may be altered by proxies
            if name == USER_AGENT {
                continue;
            }
            if params.settings.signature_location == SignatureLocation::QueryParams {
                // Exclude content-length and content-type for query param signatures since the
                // body is unsigned for these use-cases, and the size is not known up-front.
                if name == CONTENT_LENGTH || name == CONTENT_TYPE {
                    continue;
                }
                // The X-Amz-User-Agent header should not be signed if this is for a presigned URL
                if name == HeaderName::from_static(header::X_AMZ_USER_AGENT) {
                    continue;
                }
            }
            signed_headers.push(CanonicalHeaderName(name.clone()));
        }
        Ok((signed_headers, canonical_headers))
    }

    fn payload_hash<'b>(body: &'b SignableBody<'b>) -> Cow<'b, str> {
        // Payload hash computation
        //
        // Based on the input body, set the payload_hash of the canonical request:
        // Either:
        // - compute a hash
        // - use the precomputed hash
        // - use `UnsignedPayload`
        match body {
            SignableBody::Bytes(data) => Cow::Owned(sha256_hex_string(data)),
            SignableBody::Precomputed(digest) => Cow::Borrowed(digest.as_str()),
            SignableBody::UnsignedPayload => Cow::Borrowed(UNSIGNED_PAYLOAD),
        }
    }

    fn params(uri: &Uri, values: &SignatureValues<'_>) -> Option<String> {
        let mut params: Vec<(Cow<'_, str>, Cow<'_, str>)> =
            form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes()).collect();
        fn add_param<'a>(params: &mut Vec<(Cow<'a, str>, Cow<'a, str>)>, k: &'a str, v: &'a str) {
            params.push((Cow::Borrowed(k), Cow::Borrowed(v)));
        }

        if let SignatureValues::QueryParams(values) = values {
            add_param(&mut params, param::X_AMZ_DATE, &values.date_time);
            add_param(&mut params, param::X_AMZ_EXPIRES, &values.expires);
            add_param(&mut params, param::X_AMZ_ALGORITHM, values.algorithm);
            add_param(&mut params, param::X_AMZ_CREDENTIAL, &values.credential);
            add_param(
                &mut params,
                param::X_AMZ_SIGNED_HEADERS,
                values.signed_headers.as_str(),
            );
            if let Some(security_token) = values.security_token {
                add_param(&mut params, header::X_AMZ_SECURITY_TOKEN, security_token);
            }
        }
        // Sort by param name, and then by param value
        params.sort();

        let mut query = QueryWriter::new(uri);
        query.clear_params();
        for (key, value) in params {
            query.insert(&key, &value);
        }

        let query = query.build_query();
        if query.is_empty() {
            None
        } else {
            Some(query)
        }
    }

    fn insert_host_header(
        canonical_headers: &mut HeaderMap<HeaderValue>,
        uri: &Uri,
    ) -> HeaderValue {
        match canonical_headers.get(&HOST) {
            Some(header) => header.clone(),
            None => {
                let authority = uri
                    .authority()
                    .expect("request uri authority must be set for signing");
                let header = HeaderValue::try_from(authority.as_str())
                    .expect("endpoint must contain valid header characters");
                canonical_headers.insert(HOST, header.clone());
                header
            }
        }
    }

    fn insert_date_header(
        canonical_headers: &mut HeaderMap<HeaderValue>,
        date_time: &str,
    ) -> HeaderValue {
        let x_amz_date = HeaderName::from_static(header::X_AMZ_DATE);
        let date_header = HeaderValue::try_from(date_time).expect("date is valid header value");
        canonical_headers.insert(x_amz_date, date_header.clone());
        date_header
    }
}

impl<'a> fmt::Display for CanonicalRequest<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.method)?;
        writeln!(f, "{}", self.path)?;
        writeln!(f, "{}", self.params.as_deref().unwrap_or(""))?;
        // write out _all_ the headers
        for header in &self.values.signed_headers().headers {
            // a missing header is a bug, so we should panic.
            let value = &self.headers[&header.0];
            write!(f, "{}:", header.0.as_str())?;
            writeln!(
                f,
                "{}",
                std::str::from_utf8(value.as_bytes())
                    .expect("SDK request header values are valid UTF-8")
            )?;
        }
        writeln!(f)?;
        // write out the signed headers
        write!(f, "{}", self.values.signed_headers().as_str())?;
        writeln!(f)?;
        write!(f, "{}", self.values.content_sha256())?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Default)]
pub(super) struct SignedHeaders {
    headers: Vec<CanonicalHeaderName>,
    formatted: String,
}

impl SignedHeaders {
    fn new(mut headers: Vec<CanonicalHeaderName>) -> Self {
        headers.sort();
        let formatted = Self::fmt(&headers);
        SignedHeaders { headers, formatted }
    }

    fn fmt(headers: &[CanonicalHeaderName]) -> String {
        let mut value = String::new();
        let mut iter = headers.iter().peekable();
        while let Some(next) = iter.next() {
            value += next.0.as_str();
            if iter.peek().is_some() {
                value.push(';');
            }
        }
        value
    }

    pub(super) fn as_str(&self) -> &str {
        &self.formatted
    }
}

impl fmt::Display for SignedHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct CanonicalHeaderName(HeaderName);

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
pub(super) struct SigningScope<'a> {
    pub(super) date: Date<Utc>,
    pub(super) region: &'a str,
    pub(super) service: &'a str,
}

impl<'a> fmt::Display for SigningScope<'a> {
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

impl<'a> TryFrom<&'a str> for SigningScope<'a> {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<SigningScope<'a>, Self::Error> {
        let mut scopes = s.split('/');
        let date = parse_date(scopes.next().expect("missing date"))?;
        let region = scopes.next().expect("missing region");
        let service = scopes.next().expect("missing service");

        let scope = SigningScope {
            date,
            region,
            service,
        };

        Ok(scope)
    }
}

#[derive(PartialEq, Debug)]
pub(super) struct StringToSign<'a> {
    pub(super) scope: SigningScope<'a>,
    pub(super) date: DateTime<Utc>,
    pub(super) region: &'a str,
    pub(super) service: &'a str,
    pub(super) hashed_creq: &'a str,
}

impl<'a> TryFrom<&'a str> for StringToSign<'a> {
    type Error = Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let lines = s.lines().collect::<Vec<&str>>();
        let date = parse_date_time(&lines[1])?;
        let scope: SigningScope<'_> = TryFrom::try_from(lines[2])?;
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
        let scope = SigningScope {
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

#[cfg(test)]
mod tests {
    use crate::date_fmt::parse_date_time;
    use crate::http_request::canonical_request::{CanonicalRequest, SigningScope, StringToSign};
    use crate::http_request::test::{test_canonical_request, test_request, test_sts};
    use crate::http_request::{
        PayloadChecksumKind, SignableBody, SignableRequest, SigningSettings,
    };
    use crate::http_request::{SignatureLocation, SigningParams};
    use crate::sign::sha256_hex_string;
    use pretty_assertions::assert_eq;
    use std::convert::TryFrom;
    use std::time::Duration;

    fn signing_params(settings: SigningSettings) -> SigningParams<'static> {
        SigningParams {
            access_key: "test-access-key",
            secret_key: "test-secret-key",
            security_token: None,
            region: "test-region",
            service_name: "testservicename",
            date_time: parse_date_time("20210511T154045Z").unwrap(),
            settings,
        }
    }

    #[test]
    fn test_set_xamz_sha_256() {
        let req = test_request("get-vanilla-query-order-key-case");
        let req = SignableRequest::from(&req);
        let settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let mut signing_params = signing_params(settings);
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();
        assert_eq!(
            creq.values.content_sha256(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        // assert that the sha256 header was added
        assert_eq!(
            creq.values.signed_headers().as_str(),
            "host;x-amz-content-sha256;x-amz-date"
        );

        signing_params.settings.payload_checksum_kind = PayloadChecksumKind::NoHeader;
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();
        assert_eq!(creq.values.signed_headers().as_str(), "host;x-amz-date");
    }

    #[test]
    fn test_unsigned_payload() {
        let req = test_request("get-vanilla-query-order-key-case");
        let req = SignableRequest::new(
            req.method(),
            req.uri(),
            req.headers(),
            SignableBody::UnsignedPayload,
        );
        let settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let signing_params = signing_params(settings);
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();
        assert_eq!(creq.values.content_sha256(), "UNSIGNED-PAYLOAD");
        assert!(creq.to_string().ends_with("UNSIGNED-PAYLOAD"));
    }

    #[test]
    fn test_precomputed_payload() {
        let payload_hash = "44ce7dd67c959e0d3524ffac1771dfbba87d2b6b4b4e99e42034a8b803f8b072";
        let req = test_request("get-vanilla-query-order-key-case");
        let req = SignableRequest::new(
            req.method(),
            req.uri(),
            req.headers(),
            SignableBody::Precomputed(String::from(payload_hash)),
        );
        let settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let signing_params = signing_params(settings);
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();
        assert_eq!(creq.values.content_sha256(), payload_hash);
        assert!(creq.to_string().ends_with(payload_hash));
    }

    #[test]
    fn test_generate_scope() {
        let expected = "20150830/us-east-1/iam/aws4_request\n";
        let date = parse_date_time("20150830T123600Z").unwrap();
        let scope = SigningScope {
            date: date.date(),
            region: "us-east-1",
            service: "iam",
        };
        assert_eq!(format!("{}\n", scope.to_string()), expected);
    }

    #[test]
    fn test_string_to_sign() {
        let date = parse_date_time("20150830T123600Z").unwrap();
        let creq = test_canonical_request("get-vanilla-query-order-key-case");
        let expected_sts = test_sts("get-vanilla-query-order-key-case");
        let encoded = sha256_hex_string(creq.as_bytes());

        let actual = StringToSign::new(date, "us-east-1", "service", &encoded);
        assert_eq!(expected_sts, actual.to_string());
    }

    #[test]
    fn read_sts() {
        let sts = test_sts("get-vanilla-query-order-key-case");
        StringToSign::try_from(sts.as_ref()).unwrap();
    }

    #[test]
    fn test_digest_of_canonical_request() {
        let creq = test_canonical_request("get-vanilla-query-order-key-case");
        let expected = "816cd5b414d056048ba4f7c5386d6e0533120fb1fcfa93762cf0fc39e2cf19e0";
        let actual = sha256_hex_string(creq.as_bytes());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_double_url_encode() {
        let req = test_request("double-url-encode");
        let req = SignableRequest::from(&req);
        let signing_params = signing_params(SigningSettings::default());
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();

        let expected = test_canonical_request("double-url-encode");
        let actual = format!("{}", creq);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_tilde_in_uri() {
        let req = http::Request::builder()
            .uri("https://s3.us-east-1.amazonaws.com/my-bucket?list-type=2&prefix=~objprefix&single&k=&unreserved=-_.~").body("").unwrap();
        let req = SignableRequest::from(&req);
        let signing_params = signing_params(SigningSettings::default());
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();
        assert_eq!(
            Some("k=&list-type=2&prefix=~objprefix&single=&unreserved=-_.~"),
            creq.params.as_deref(),
        );
    }

    // It should exclude user-agent, content-type, content-length, and x-amz-user-agent headers from presigning
    #[test]
    fn presigning_header_exclusion() {
        let request = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header("content-type", "application/xml")
            .header("content-length", "0")
            .header("user-agent", "test-user-agent")
            .header("x-amz-user-agent", "test-user-agent")
            .body("")
            .unwrap();
        let request = SignableRequest::from(&request);

        let mut settings = SigningSettings::default();
        settings.signature_location = SignatureLocation::QueryParams;
        settings.expires_in = Some(Duration::from_secs(30));

        let signing_params = signing_params(settings);
        let canonical = CanonicalRequest::from(&request, &signing_params).unwrap();

        let values = canonical.values.into_query_params().unwrap();
        assert_eq!("host", values.signed_headers.as_str());
    }
}
