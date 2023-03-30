/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::date_time::{format_date, format_date_time};
use crate::http_request::error::CanonicalRequestError;
use crate::http_request::settings::UriPathNormalizationMode;
use crate::http_request::sign::SignableRequest;
use crate::http_request::uri_path_normalization::normalize_uri_path;
use crate::http_request::url_escape::percent_encode_path;
use crate::http_request::PercentEncodingMode;
use crate::http_request::{PayloadChecksumKind, SignableBody, SignatureLocation, SigningParams};
use crate::sign::sha256_hex_string;
use aws_smithy_http::query_writer::QueryWriter;
use http::header::{AsHeaderName, HeaderName, HOST};
use http::{HeaderMap, HeaderValue, Method, Uri};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use std::time::SystemTime;

pub(crate) mod header {
    pub(crate) const X_AMZ_CONTENT_SHA_256: &str = "x-amz-content-sha256";
    pub(crate) const X_AMZ_DATE: &str = "x-amz-date";
    pub(crate) const X_AMZ_SECURITY_TOKEN: &str = "x-amz-security-token";
    pub(crate) const X_AMZ_USER_AGENT: &str = "x-amz-user-agent";
}

pub(crate) mod param {
    pub(crate) const X_AMZ_ALGORITHM: &str = "X-Amz-Algorithm";
    pub(crate) const X_AMZ_CREDENTIAL: &str = "X-Amz-Credential";
    pub(crate) const X_AMZ_DATE: &str = "X-Amz-Date";
    pub(crate) const X_AMZ_EXPIRES: &str = "X-Amz-Expires";
    pub(crate) const X_AMZ_SECURITY_TOKEN: &str = "X-Amz-Security-Token";
    pub(crate) const X_AMZ_SIGNED_HEADERS: &str = "X-Amz-SignedHeaders";
    pub(crate) const X_AMZ_SIGNATURE: &str = "X-Amz-Signature";
}

pub(crate) const HMAC_256: &str = "AWS4-HMAC-SHA256";

const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";
const STREAMING_UNSIGNED_PAYLOAD_TRAILER: &str = "STREAMING-UNSIGNED-PAYLOAD-TRAILER";

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
            SignatureValues::Headers(values) => Some(values),
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
    ) -> Result<CanonicalRequest<'b>, CanonicalRequestError> {
        // Path encoding: if specified, re-encode % as %25
        // Set method and path into CanonicalRequest
        let path = req.uri().path();
        let path = match params.settings.uri_path_normalization_mode {
            UriPathNormalizationMode::Enabled => normalize_uri_path(path),
            UriPathNormalizationMode::Disabled => Cow::Borrowed(path),
        };
        let path = match params.settings.percent_encoding_mode {
            // The string is already URI encoded, we don't need to encode everything again, just `%`
            PercentEncodingMode::Double => Cow::Owned(percent_encode_path(&path)),
            PercentEncodingMode::Single => path,
        };
        let payload_hash = Self::payload_hash(req.body());

        let date_time = format_date_time(params.time);
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
                    format_date(params.time),
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
    ) -> Result<(Vec<CanonicalHeaderName>, HeaderMap), CanonicalRequestError> {
        // Header computation:
        // The canonical request will include headers not present in the input. We need to clone and
        // normalize the headers from the original request and add:
        // - host
        // - x-amz-date
        // - x-amz-security-token (if provided)
        // - x-amz-content-sha256 (if requested by signing settings)
        let mut canonical_headers = HeaderMap::with_capacity(req.headers().len());
        for (name, value) in req.headers().iter() {
            // Header names and values need to be normalized according to Step 4 of https://docs.aws.amazon.com/general/latest/gr/sigv4-create-canonical-request.html
            // Using append instead of insert means this will not clobber headers that have the same lowercased name
            canonical_headers.append(
                HeaderName::from_str(&name.as_str().to_lowercase())?,
                normalize_header_value(value)?,
            );
        }

        Self::insert_host_header(&mut canonical_headers, req.uri());

        if params.settings.signature_location == SignatureLocation::Headers {
            Self::insert_date_header(&mut canonical_headers, date_time);

            if let Some(security_token) = params.security_token {
                let mut sec_header = HeaderValue::from_str(security_token)?;
                sec_header.set_sensitive(true);
                canonical_headers.insert(header::X_AMZ_SECURITY_TOKEN, sec_header);
            }

            if params.settings.payload_checksum_kind == PayloadChecksumKind::XAmzSha256 {
                let header = HeaderValue::from_str(payload_hash)?;
                canonical_headers.insert(header::X_AMZ_CONTENT_SHA_256, header);
            }
        }

        let mut signed_headers = Vec::with_capacity(canonical_headers.len());
        for name in canonical_headers.keys() {
            if let Some(excluded_headers) = params.settings.excluded_headers.as_ref() {
                if excluded_headers.contains(name) {
                    continue;
                }
            }

            if params.settings.signature_location == SignatureLocation::QueryParams {
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
        // - use `UnsignedPayload` for streaming requests
        // - use `StreamingUnsignedPayloadTrailer` for streaming requests with trailers
        match body {
            SignableBody::Bytes(data) => Cow::Owned(sha256_hex_string(data)),
            SignableBody::Precomputed(digest) => Cow::Borrowed(digest.as_str()),
            SignableBody::UnsignedPayload => Cow::Borrowed(UNSIGNED_PAYLOAD),
            SignableBody::StreamingUnsignedPayloadTrailer => {
                Cow::Borrowed(STREAMING_UNSIGNED_PAYLOAD_TRAILER)
            }
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
                add_param(&mut params, param::X_AMZ_SECURITY_TOKEN, security_token);
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

    fn header_values_for(&self, key: impl AsHeaderName) -> String {
        let values: Vec<&str> = self
            .headers
            .get_all(key)
            .into_iter()
            .map(|value| {
                std::str::from_utf8(value.as_bytes())
                    .expect("SDK request header values are valid UTF-8")
            })
            .collect();
        values.join(",")
    }
}

impl<'a> fmt::Display for CanonicalRequest<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.method)?;
        writeln!(f, "{}", self.path)?;
        writeln!(f, "{}", self.params.as_deref().unwrap_or(""))?;
        // write out _all_ the headers
        for header in &self.values.signed_headers().headers {
            write!(f, "{}:", header.0.as_str())?;
            writeln!(f, "{}", self.header_values_for(&header.0))?;
        }
        writeln!(f)?;
        // write out the signed headers
        write!(f, "{}", self.values.signed_headers().as_str())?;
        writeln!(f)?;
        write!(f, "{}", self.values.content_sha256())?;
        Ok(())
    }
}

/// A regex for matching on 2 or more spaces that acts on bytes.
static MULTIPLE_SPACES: once_cell::sync::Lazy<regex::bytes::Regex> =
    once_cell::sync::Lazy::new(|| regex::bytes::Regex::new(r" {2,}").unwrap());

/// Removes excess spaces before and after a given byte string, and converts multiple sequential
/// spaces to a single space e.g. "  Some  example   text  " -> "Some example text".
///
/// This function ONLY affects spaces and not other kinds of whitespace.
fn trim_all(text: &[u8]) -> Cow<'_, [u8]> {
    // The normal trim function will trim non-breaking spaces and other various whitespace chars.
    // S3 ONLY trims spaces so we use trim_matches to trim spaces only
    let text = trim_spaces_from_byte_string(text);
    MULTIPLE_SPACES.replace_all(text, " ".as_bytes())
}

/// Removes excess spaces before and after a given byte string by returning a subset of those bytes.
/// Will return an empty slice if a string is composed entirely of whitespace.
fn trim_spaces_from_byte_string(bytes: &[u8]) -> &[u8] {
    let starting_index = bytes.iter().position(|b| *b != b' ').unwrap_or(0);
    let ending_offset = bytes.iter().rev().position(|b| *b != b' ').unwrap_or(0);
    let ending_index = bytes.len() - ending_offset;
    &bytes[starting_index..ending_index]
}

/// Works just like [trim_all] but acts on HeaderValues instead of bytes.
/// Will ensure that the underlying bytes are valid UTF-8.
fn normalize_header_value(
    header_value: &HeaderValue,
) -> Result<HeaderValue, CanonicalRequestError> {
    let trimmed_value = trim_all(header_value.as_bytes());
    HeaderValue::from_str(
        std::str::from_utf8(&trimmed_value)
            .map_err(CanonicalRequestError::invalid_utf8_in_header_value)?,
    )
    .map_err(CanonicalRequestError::from)
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
        self.0.as_str().cmp(other.0.as_str())
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(super) struct SigningScope<'a> {
    pub(super) time: SystemTime,
    pub(super) region: &'a str,
    pub(super) service: &'a str,
}

impl<'a> fmt::Display for SigningScope<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}/{}/aws4_request",
            format_date(self.time),
            self.region,
            self.service
        )
    }
}

#[derive(PartialEq, Debug)]
pub(super) struct StringToSign<'a> {
    pub(super) scope: SigningScope<'a>,
    pub(super) time: SystemTime,
    pub(super) region: &'a str,
    pub(super) service: &'a str,
    pub(super) hashed_creq: &'a str,
}

impl<'a> StringToSign<'a> {
    pub(crate) fn new(
        time: SystemTime,
        region: &'a str,
        service: &'a str,
        hashed_creq: &'a str,
    ) -> Self {
        let scope = SigningScope {
            time,
            region,
            service,
        };
        Self {
            scope,
            time,
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
            format_date_time(self.time),
            self.scope,
            self.hashed_creq
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::date_time::test_parsers::parse_date_time;
    use crate::http_request::canonical_request::{
        normalize_header_value, trim_all, CanonicalRequest, SigningScope, StringToSign,
    };
    use crate::http_request::test::{test_canonical_request, test_request, test_sts};
    use crate::http_request::{
        PayloadChecksumKind, SignableBody, SignableRequest, SigningSettings,
    };
    use crate::http_request::{SignatureLocation, SigningParams};
    use crate::sign::sha256_hex_string;
    use aws_smithy_http::query_writer::QueryWriter;
    use http::Uri;
    use http::{header::HeaderName, HeaderValue};
    use pretty_assertions::assert_eq;
    use proptest::{prelude::*, proptest};
    use std::time::Duration;

    fn signing_params(settings: SigningSettings) -> SigningParams<'static> {
        SigningParams {
            access_key: "test-access-key",
            secret_key: "test-secret-key",
            security_token: None,
            region: "test-region",
            service_name: "testservicename",
            time: parse_date_time("20210511T154045Z").unwrap(),
            settings,
        }
    }

    #[test]
    fn test_repeated_header() {
        let mut req = test_request("get-vanilla-query-order-key-case");
        req.headers_mut().append(
            "x-amz-object-attributes",
            HeaderValue::from_static("Checksum"),
        );
        req.headers_mut().append(
            "x-amz-object-attributes",
            HeaderValue::from_static("ObjectSize"),
        );
        let req = SignableRequest::from(&req);
        let settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let signing_params = signing_params(settings);
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();

        assert_eq!(
            creq.values.signed_headers().to_string(),
            "host;x-amz-content-sha256;x-amz-date;x-amz-object-attributes"
        );
        assert_eq!(
            creq.header_values_for("x-amz-object-attributes"),
            "Checksum,ObjectSize",
        );
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
        let scope = SigningScope {
            time: parse_date_time("20150830T123600Z").unwrap(),
            region: "us-east-1",
            service: "iam",
        };
        assert_eq!(format!("{}\n", scope), expected);
    }

    #[test]
    fn test_string_to_sign() {
        let time = parse_date_time("20150830T123600Z").unwrap();
        let creq = test_canonical_request("get-vanilla-query-order-key-case");
        let expected_sts = test_sts("get-vanilla-query-order-key-case");
        let encoded = sha256_hex_string(creq.as_bytes());

        let actual = StringToSign::new(time, "us-east-1", "service", &encoded);
        assert_eq!(expected_sts, actual.to_string());
    }

    #[test]
    fn test_digest_of_canonical_request() {
        let creq = test_canonical_request("get-vanilla-query-order-key-case");
        let expected = "816cd5b414d056048ba4f7c5386d6e0533120fb1fcfa93762cf0fc39e2cf19e0";
        let actual = sha256_hex_string(creq.as_bytes());
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_double_url_encode_path() {
        let req = test_request("double-encode-path");
        let req = SignableRequest::from(&req);
        let signing_params = signing_params(SigningSettings::default());
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();

        let expected = test_canonical_request("double-encode-path");
        let actual = format!("{}", creq);
        assert_eq!(actual, expected);
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

    #[test]
    fn test_signing_urls_with_percent_encoded_query_strings() {
        let all_printable_ascii_chars: String = (32u8..127).map(char::from).collect();
        let uri = Uri::from_static("https://s3.us-east-1.amazonaws.com/my-bucket");

        let mut query_writer = QueryWriter::new(&uri);
        query_writer.insert("list-type", "2");
        query_writer.insert("prefix", &all_printable_ascii_chars);

        let req = http::Request::builder()
            .uri(query_writer.build_uri())
            .body("")
            .unwrap();
        let req = SignableRequest::from(&req);
        let signing_params = signing_params(SigningSettings::default());
        let creq = CanonicalRequest::from(&req, &signing_params).unwrap();

        let expected = "list-type=2&prefix=%20%21%22%23%24%25%26%27%28%29%2A%2B%2C-.%2F0123456789%3A%3B%3C%3D%3E%3F%40ABCDEFGHIJKLMNOPQRSTUVWXYZ%5B%5C%5D%5E_%60abcdefghijklmnopqrstuvwxyz%7B%7C%7D~";
        let actual = creq.params.unwrap();
        assert_eq!(expected, actual);
    }

    // It should exclude user-agent and x-amz-user-agent headers from presigning
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

        let settings = SigningSettings {
            signature_location: SignatureLocation::QueryParams,
            expires_in: Some(Duration::from_secs(30)),
            ..Default::default()
        };

        let signing_params = signing_params(settings);
        let canonical = CanonicalRequest::from(&request, &signing_params).unwrap();

        let values = canonical.values.into_query_params().unwrap();
        assert_eq!(
            "content-length;content-type;host",
            values.signed_headers.as_str()
        );
    }

    proptest! {
       #[test]
       fn presigning_header_exclusion_with_explicit_exclusion_list_specified(
           excluded_headers in prop::collection::vec("[a-z]{1,20}", 1..10),
       ) {
            let mut request_builder = http::Request::builder()
                .uri("https://some-endpoint.some-region.amazonaws.com")
                .header("content-type", "application/xml")
                .header("content-length", "0");
            for key in &excluded_headers {
                request_builder = request_builder.header(key, "value");
            }
            let request = request_builder.body("").unwrap();

            let request = SignableRequest::from(&request);

            let settings = SigningSettings {
                signature_location: SignatureLocation::QueryParams,
                expires_in: Some(Duration::from_secs(30)),
                excluded_headers: Some(
                    excluded_headers
                        .into_iter()
                        .map(|header_string| {
                            HeaderName::from_static(Box::leak(header_string.into_boxed_str()))
                        })
                        .collect(),
                ),
                ..Default::default()
            };

            let signing_params = signing_params(settings);
            let canonical = CanonicalRequest::from(&request, &signing_params).unwrap();

            let values = canonical.values.into_query_params().unwrap();
            assert_eq!(
                "content-length;content-type;host",
                values.signed_headers.as_str()
            );
        }
    }

    #[test]
    fn test_trim_all_handles_spaces_correctly() {
        // Can't compare a byte array to a Cow so we convert both to slices before comparing
        let expected = &b"Some example text"[..];
        let actual = &trim_all(b"  Some  example   text  ")[..];

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_trim_all_ignores_other_forms_of_whitespace() {
        // Can't compare a byte array to a Cow so we convert both to slices before comparing
        let expected = &b"\t\xA0Some\xA0 example \xA0text\xA0\n"[..];
        // \xA0 is a non-breaking space character
        let actual = &trim_all(b"\t\xA0Some\xA0     example   \xA0text\xA0\n")[..];

        assert_eq!(expected, actual);
    }

    #[test]
    fn trim_spaces_works_on_single_characters() {
        assert_eq!(trim_all(b"2").as_ref(), b"2");
    }

    proptest! {
        #[test]
        fn test_trim_all_doesnt_elongate_strings(s in ".*") {
            assert!(trim_all(s.as_bytes()).len() <= s.len())
        }

        #[test]
        fn test_normalize_header_value_works_on_valid_header_value(v in (".*")) {
            if let Ok(header_value) = HeaderValue::from_maybe_shared(v) {
                assert!(normalize_header_value(&header_value).is_ok());
            }
        }

        #[test]
        fn test_trim_all_does_nothing_when_there_are_no_spaces(s in "[^ ]*") {
            assert_eq!(trim_all(s.as_bytes()).as_ref(), s.as_bytes());
        }
    }

    #[test]
    fn test_normalize_header_value_returns_expected_error_on_invalid_utf8() {
        let header_value = HeaderValue::from_bytes(&[0xC0, 0xC1]).unwrap();
        assert!(normalize_header_value(&header_value).is_err());
    }
}
