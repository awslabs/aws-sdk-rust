/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::error::SigningError;
use super::{PayloadChecksumKind, SignatureLocation};
use crate::http_request::canonical_request::header;
use crate::http_request::canonical_request::param;
use crate::http_request::canonical_request::{CanonicalRequest, StringToSign, HMAC_256};
use crate::http_request::SigningParams;
use crate::sign::{calculate_signature, generate_signing_key, sha256_hex_string};
use crate::SigningOutput;
use aws_smithy_http::query_writer::QueryWriter;
use http::header::HeaderValue;
use http::{HeaderMap, Method, Uri};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::str;

/// Represents all of the information necessary to sign an HTTP request.
#[derive(Debug)]
#[non_exhaustive]
pub struct SignableRequest<'a> {
    method: &'a Method,
    uri: &'a Uri,
    headers: &'a HeaderMap<HeaderValue>,
    body: SignableBody<'a>,
}

impl<'a> SignableRequest<'a> {
    /// Creates a new `SignableRequest`. If you have an [`http::Request`], then
    /// consider using [`SignableRequest::from`] instead of `new`.
    pub fn new(
        method: &'a Method,
        uri: &'a Uri,
        headers: &'a HeaderMap<HeaderValue>,
        body: SignableBody<'a>,
    ) -> Self {
        Self {
            method,
            uri,
            headers,
            body,
        }
    }

    /// Returns the signable URI
    pub fn uri(&self) -> &Uri {
        self.uri
    }

    /// Returns the signable HTTP method
    pub fn method(&self) -> &Method {
        self.method
    }

    /// Returns the request headers
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.headers
    }

    /// Returns the signable body
    pub fn body(&self) -> &SignableBody<'_> {
        &self.body
    }
}

impl<'a, B> From<&'a http::Request<B>> for SignableRequest<'a>
where
    B: 'a,
    B: AsRef<[u8]>,
{
    fn from(request: &'a http::Request<B>) -> SignableRequest<'a> {
        SignableRequest::new(
            request.method(),
            request.uri(),
            request.headers(),
            SignableBody::Bytes(request.body().as_ref()),
        )
    }
}

/// A signable HTTP request body
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum SignableBody<'a> {
    /// A body composed of a slice of bytes
    Bytes(&'a [u8]),

    /// An unsigned payload
    ///
    /// UnsignedPayload is used for streaming requests where the contents of the body cannot be
    /// known prior to signing
    UnsignedPayload,

    /// A precomputed body checksum. The checksum should be a SHA256 checksum of the body,
    /// lowercase hex encoded. Eg:
    /// `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
    Precomputed(String),

    /// Set when a streaming body has checksum trailers.
    StreamingUnsignedPayloadTrailer,
}

#[derive(Debug)]
pub struct SigningInstructions {
    headers: Option<HeaderMap<HeaderValue>>,
    params: Option<Vec<(&'static str, Cow<'static, str>)>>,
}

impl SigningInstructions {
    fn new(
        headers: Option<HeaderMap<HeaderValue>>,
        params: Option<Vec<(&'static str, Cow<'static, str>)>>,
    ) -> Self {
        Self { headers, params }
    }

    pub fn headers(&self) -> Option<&HeaderMap<HeaderValue>> {
        self.headers.as_ref()
    }
    pub fn take_headers(&mut self) -> Option<HeaderMap<HeaderValue>> {
        self.headers.take()
    }

    pub fn params(&self) -> Option<&Vec<(&'static str, Cow<'static, str>)>> {
        self.params.as_ref()
    }
    pub fn take_params(&mut self) -> Option<Vec<(&'static str, Cow<'static, str>)>> {
        self.params.take()
    }

    pub fn apply_to_request<B>(mut self, request: &mut http::Request<B>) {
        if let Some(new_headers) = self.take_headers() {
            for (name, value) in new_headers.into_iter() {
                request.headers_mut().insert(name.unwrap(), value);
            }
        }
        if let Some(params) = self.take_params() {
            let mut query = QueryWriter::new(request.uri());
            for (name, value) in params {
                query.insert(name, &value);
            }
            *request.uri_mut() = query.build_uri();
        }
    }
}

/// Produces a signature for the given `request` and returns instructions
/// that can be used to apply that signature to an HTTP request.
pub fn sign<'a>(
    request: SignableRequest<'a>,
    params: &'a SigningParams<'a>,
) -> Result<SigningOutput<SigningInstructions>, SigningError> {
    tracing::trace!(request = ?request, params = ?params, "signing request");
    match params.settings.signature_location {
        SignatureLocation::Headers => {
            let (signing_headers, signature) =
                calculate_signing_headers(&request, params)?.into_parts();
            Ok(SigningOutput::new(
                SigningInstructions::new(Some(signing_headers), None),
                signature,
            ))
        }
        SignatureLocation::QueryParams => {
            let (params, signature) = calculate_signing_params(&request, params)?;
            Ok(SigningOutput::new(
                SigningInstructions::new(None, Some(params)),
                signature,
            ))
        }
    }
}

type CalculatedParams = Vec<(&'static str, Cow<'static, str>)>;

fn calculate_signing_params<'a>(
    request: &'a SignableRequest<'a>,
    params: &'a SigningParams<'a>,
) -> Result<(CalculatedParams, String), SigningError> {
    let creq = CanonicalRequest::from(request, params)?;

    let encoded_creq = &sha256_hex_string(creq.to_string().as_bytes());
    let string_to_sign = StringToSign::new(
        params.time,
        params.region,
        params.service_name,
        encoded_creq,
    )
    .to_string();
    let signing_key = generate_signing_key(
        params.secret_key,
        params.time,
        params.region,
        params.service_name,
    );
    let signature = calculate_signature(signing_key, string_to_sign.as_bytes());
    tracing::trace!(canonical_request = %creq, string_to_sign = %string_to_sign, "calculated signing parameters");

    let values = creq.values.into_query_params().expect("signing with query");
    let mut signing_params = vec![
        (param::X_AMZ_ALGORITHM, Cow::Borrowed(values.algorithm)),
        (param::X_AMZ_CREDENTIAL, Cow::Owned(values.credential)),
        (param::X_AMZ_DATE, Cow::Owned(values.date_time)),
        (param::X_AMZ_EXPIRES, Cow::Owned(values.expires)),
        (
            param::X_AMZ_SIGNED_HEADERS,
            Cow::Owned(values.signed_headers.as_str().into()),
        ),
        (param::X_AMZ_SIGNATURE, Cow::Owned(signature.clone())),
    ];
    if let Some(security_token) = params.security_token {
        signing_params.push((
            param::X_AMZ_SECURITY_TOKEN,
            Cow::Owned(security_token.to_string()),
        ));
    }
    Ok((signing_params, signature))
}

/// Calculates the signature headers that need to get added to the given `request`.
///
/// `request` MUST NOT contain any of the following headers:
/// - x-amz-date
/// - x-amz-content-sha-256
/// - x-amz-security-token
fn calculate_signing_headers<'a>(
    request: &'a SignableRequest<'a>,
    params: &'a SigningParams<'a>,
) -> Result<SigningOutput<HeaderMap<HeaderValue>>, SigningError> {
    // Step 1: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-canonical-request.html.
    let creq = CanonicalRequest::from(request, params)?;
    tracing::trace!(canonical_request = %creq);

    // Step 2: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-string-to-sign.html.
    let encoded_creq = &sha256_hex_string(creq.to_string().as_bytes());
    let sts = StringToSign::new(
        params.time,
        params.region,
        params.service_name,
        encoded_creq,
    );

    // Step 3: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-calculate-signature.html
    let signing_key = generate_signing_key(
        params.secret_key,
        params.time,
        params.region,
        params.service_name,
    );
    let signature = calculate_signature(signing_key, sts.to_string().as_bytes());

    // Step 4: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-add-signature-to-request.html
    let values = creq.values.as_headers().expect("signing with headers");
    let mut headers = HeaderMap::new();
    add_header(&mut headers, header::X_AMZ_DATE, &values.date_time);
    headers.insert(
        "authorization",
        build_authorization_header(params.access_key, &creq, sts, &signature),
    );
    if params.settings.payload_checksum_kind == PayloadChecksumKind::XAmzSha256 {
        add_header(
            &mut headers,
            header::X_AMZ_CONTENT_SHA_256,
            &values.content_sha256,
        );
    }
    if let Some(security_token) = values.security_token {
        add_header(&mut headers, header::X_AMZ_SECURITY_TOKEN, security_token);
    }
    Ok(SigningOutput::new(headers, signature))
}

fn add_header(map: &mut HeaderMap<HeaderValue>, key: &'static str, value: &str) {
    map.insert(key, HeaderValue::try_from(value).expect(key));
}

// add signature to authorization header
// Authorization: algorithm Credential=access key ID/credential scope, SignedHeaders=SignedHeaders, Signature=signature
fn build_authorization_header(
    access_key: &str,
    creq: &CanonicalRequest<'_>,
    sts: StringToSign<'_>,
    signature: &str,
) -> HeaderValue {
    let mut value = HeaderValue::try_from(format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        HMAC_256,
        access_key,
        sts.scope,
        creq.values.signed_headers().as_str(),
        signature
    ))
    .unwrap();
    value.set_sensitive(true);
    value
}

#[cfg(test)]
mod tests {
    use super::{sign, SigningInstructions};
    use crate::date_time::test_parsers::parse_date_time;
    use crate::http_request::sign::SignableRequest;
    use crate::http_request::test::{
        make_headers_comparable, test_request, test_signed_request,
        test_signed_request_query_params,
    };
    use crate::http_request::{SignatureLocation, SigningParams, SigningSettings};
    use http::{HeaderMap, HeaderValue};
    use pretty_assertions::assert_eq;
    use proptest::proptest;
    use std::borrow::Cow;
    use std::time::Duration;

    macro_rules! assert_req_eq {
        ($a:tt, $b:tt) => {
            make_headers_comparable(&mut $a);
            make_headers_comparable(&mut $b);
            assert_eq!(format!("{:?}", $a), format!("{:?}", $b))
        };
    }

    #[test]
    fn test_sign_vanilla_with_headers() {
        let settings = SigningSettings::default();
        let params = SigningParams {
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            security_token: None,
            region: "us-east-1",
            service_name: "service",
            time: parse_date_time("20150830T123600Z").unwrap(),
            settings,
        };

        let original = test_request("get-vanilla-query-order-key-case");
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        assert_eq!(
            "b97d918cfa904a5beff61c982a1b6f458b799221646efd99d3219ec94cdf2500",
            out.signature
        );

        let mut signed = original;
        out.output.apply_to_request(&mut signed);

        let mut expected = test_signed_request("get-vanilla-query-order-key-case");
        assert_req_eq!(expected, signed);
    }

    #[test]
    fn test_sign_url_escape() {
        let test = "double-encode-path";
        let settings = SigningSettings::default();
        let params = SigningParams {
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            security_token: None,
            region: "us-east-1",
            service_name: "service",
            time: parse_date_time("20150830T123600Z").unwrap(),
            settings,
        };

        let original = test_request(test);
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        assert_eq!(
            "6f871eb157f326fa5f7439eb88ca200048635950ce7d6037deda56f0c95d4364",
            out.signature
        );

        let mut signed = original;
        out.output.apply_to_request(&mut signed);

        let mut expected = test_signed_request(test);
        assert_req_eq!(expected, signed);
    }

    #[test]
    fn test_sign_vanilla_with_query_params() {
        let settings = SigningSettings {
            signature_location: SignatureLocation::QueryParams,
            expires_in: Some(Duration::from_secs(35)),
            ..Default::default()
        };
        let params = SigningParams {
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            security_token: None,
            region: "us-east-1",
            service_name: "service",
            time: parse_date_time("20150830T123600Z").unwrap(),
            settings,
        };

        let original = test_request("get-vanilla-query-order-key-case");
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        assert_eq!(
            "f25aea20f8c722ece3b363fc5d60cc91add973f9b64c42ba36fa28d57afe9019",
            out.signature
        );

        let mut signed = original;
        out.output.apply_to_request(&mut signed);

        let mut expected = test_signed_request_query_params("get-vanilla-query-order-key-case");
        assert_req_eq!(expected, signed);
    }

    #[test]
    fn test_sign_headers_utf8() {
        let settings = SigningSettings::default();
        let params = SigningParams {
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            security_token: None,
            region: "us-east-1",
            service_name: "service",
            time: parse_date_time("20150830T123600Z").unwrap(),
            settings,
        };

        let original = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header("some-header", HeaderValue::from_str("テスト").unwrap())
            .body("")
            .unwrap();
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        assert_eq!(
            "4596b207a7fc6bdf18725369bc0cd7022cf20efbd2c19730549f42d1a403648e",
            out.signature
        );

        let mut signed = original;
        out.output.apply_to_request(&mut signed);

        let mut expected = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header("some-header", HeaderValue::from_str("テスト").unwrap())
            .header(
                "x-amz-date",
                HeaderValue::from_str("20150830T123600Z").unwrap(),
            )
            .header(
                "authorization",
                HeaderValue::from_str(
                    "AWS4-HMAC-SHA256 \
                        Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, \
                        SignedHeaders=host;some-header;x-amz-date, \
                        Signature=4596b207a7fc6bdf18725369bc0cd7022cf20efbd2c19730549f42d1a403648e",
                )
                .unwrap(),
            )
            .body("")
            .unwrap();
        assert_req_eq!(expected, signed);
    }

    #[test]
    fn test_sign_headers_space_trimming() {
        let settings = SigningSettings::default();
        let params = SigningParams {
            access_key: "AKIDEXAMPLE",
            secret_key: "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY",
            security_token: None,
            region: "us-east-1",
            service_name: "service",
            time: parse_date_time("20150830T123600Z").unwrap(),
            settings,
        };

        let original = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header(
                "some-header",
                HeaderValue::from_str("  test  test   ").unwrap(),
            )
            .body("")
            .unwrap();
        let signable = SignableRequest::from(&original);
        let out = sign(signable, &params).unwrap();
        assert_eq!(
            "0bd74dbf6f21161f61a1a3a1c313b6a4bc67ec57bf5ea9ae956a63753ca1d7f7",
            out.signature
        );

        let mut signed = original;
        out.output.apply_to_request(&mut signed);

        let mut expected = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .header(
                "some-header",
                HeaderValue::from_str("  test  test   ").unwrap(),
            )
            .header(
                "x-amz-date",
                HeaderValue::from_str("20150830T123600Z").unwrap(),
            )
            .header(
                "authorization",
                HeaderValue::from_str(
                    "AWS4-HMAC-SHA256 \
                        Credential=AKIDEXAMPLE/20150830/us-east-1/service/aws4_request, \
                        SignedHeaders=host;some-header;x-amz-date, \
                        Signature=0bd74dbf6f21161f61a1a3a1c313b6a4bc67ec57bf5ea9ae956a63753ca1d7f7",
                )
                .unwrap(),
            )
            .body("")
            .unwrap();
        assert_req_eq!(expected, signed);
    }

    #[test]
    fn test_sign_headers_returning_expected_error_on_invalid_utf8() {
        let settings = SigningSettings::default();
        let params = SigningParams {
            access_key: "123",
            secret_key: "asdf",
            security_token: None,
            region: "us-east-1",
            service_name: "foo",
            time: std::time::SystemTime::now(),
            settings,
        };

        let req = http::Request::builder()
            .uri("https://foo.com/")
            .header("x-sign-me", HeaderValue::from_bytes(&[0xC0, 0xC1]).unwrap())
            .body(&[])
            .unwrap();

        let creq = crate::http_request::sign(SignableRequest::from(&req), &params);
        assert!(creq.is_err());
    }

    proptest! {
        #[test]
        // Only byte values between 32 and 255 (inclusive) are permitted, excluding byte 127, for
        // [HeaderValue](https://docs.rs/http/latest/http/header/struct.HeaderValue.html#method.from_bytes).
        fn test_sign_headers_no_panic(
            left in proptest::collection::vec(32_u8..=126, 0..100),
            right in proptest::collection::vec(128_u8..=255, 0..100),
        ) {
            let settings = SigningSettings::default();
            let params = SigningParams {
                access_key: "123",
                secret_key: "asdf",
                security_token: None,
                region: "us-east-1",
                service_name: "foo",
                time: std::time::SystemTime::now(),
                settings,
            };

            let bytes = left.iter().chain(right.iter()).cloned().collect::<Vec<_>>();
            let req = http::Request::builder()
                .uri("https://foo.com/")
                .header("x-sign-me", HeaderValue::from_bytes(&bytes).unwrap())
                .body(&[])
                .unwrap();

            // The test considered a pass if the creation of `creq` does not panic.
            let _creq = crate::http_request::sign(
                SignableRequest::from(&req),
                &params);
        }
    }

    #[test]
    fn apply_signing_instructions_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("some-header", HeaderValue::from_static("foo"));
        headers.insert("some-other-header", HeaderValue::from_static("bar"));
        let instructions = SigningInstructions::new(Some(headers), None);

        let mut request = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com")
            .body("")
            .unwrap();

        instructions.apply_to_request(&mut request);

        let get_header = |n: &str| request.headers().get(n).unwrap().to_str().unwrap();
        assert_eq!("foo", get_header("some-header"));
        assert_eq!("bar", get_header("some-other-header"));
    }

    #[test]
    fn apply_signing_instructions_query_params() {
        let params = vec![
            ("some-param", Cow::Borrowed("f&o?o")),
            ("some-other-param?", Cow::Borrowed("bar")),
        ];
        let instructions = SigningInstructions::new(None, Some(params));

        let mut request = http::Request::builder()
            .uri("https://some-endpoint.some-region.amazonaws.com/some/path")
            .body("")
            .unwrap();

        instructions.apply_to_request(&mut request);

        assert_eq!(
            "/some/path?some-param=f%26o%3Fo&some-other-param%3F=bar",
            request.uri().path_and_query().unwrap().to_string()
        );
    }
}
