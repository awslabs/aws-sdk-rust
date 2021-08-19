/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Utilities to sign HTTP requests.

use crate::http_request::canonical_request::{CanonicalRequest, StringToSign};
use crate::sign::{calculate_signature, generate_signing_key, sha256_hex_string};
use crate::SigningOutput;
use http::header::{HeaderName, HeaderValue};
use std::error::Error as StdError;
use std::{iter, str};

mod canonical_request;

const HMAC_256: &str = "AWS4-HMAC-SHA256";
const X_AMZ_SECURITY_TOKEN: &str = "x-amz-security-token";
const X_AMZ_DATE: &str = "x-amz-date";
const X_AMZ_CONTENT_SHA_256: &str = "x-amz-content-sha256";

pub type Error = Box<dyn StdError + Send + Sync + 'static>;

/// Signs the given `request` with the signing params.
/// This will directly add the signature headers to the request.
pub fn sign<'a, B>(
    request: &'a mut http::Request<B>,
    params: &'a SigningParams<'a>,
) -> Result<SigningOutput<()>, Error>
where
    B: AsRef<[u8]>,
{
    let signable_body = SignableBody::Bytes(request.body().as_ref());
    let (signing_headers, signature) =
        calculate_signing_headers(&request, signable_body, params)?.into_parts();
    for (header_name, header_value) in signing_headers {
        request
            .headers_mut()
            .append(HeaderName::from_static(header_name), header_value);
    }

    Ok(SigningOutput::new((), signature))
}

pub type SigningParams<'a> = super::SigningParams<'a, SigningSettings>;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct SigningSettings {
    /// We assume the URI will be encoded _once_ prior to transmission. Some services
    /// do not decode the path prior to checking the signature, requiring clients to actually
    /// _double-encode_ the URI in creating the canonical request in order to pass a signature check.
    pub uri_encoding: UriEncoding,

    /// Add an additional checksum header
    pub payload_checksum_kind: PayloadChecksumKind,
}

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum PayloadChecksumKind {
    /// Add x-amz-checksum-sha256 to the canonical request
    ///
    /// This setting is required for S3
    XAmzSha256,

    /// Do not add an additional header when creating the canonical request
    ///
    /// This is "normal mode" and will work for services other than S3
    NoHeader,
}

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum UriEncoding {
    /// Re-encode the resulting URL (eg. %30 becomes `%2530)
    Double,

    /// Take the resulting URL as-is
    Single,
}

impl Default for SigningSettings {
    fn default() -> Self {
        Self {
            uri_encoding: UriEncoding::Double,
            payload_checksum_kind: PayloadChecksumKind::NoHeader,
        }
    }
}

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
}

/// Calculates the signature headers that need to get added to the given `request`.
///
/// `request` MUST NOT contain any of the following headers:
/// - x-amz-date
/// - x-amz-content-sha-256
/// - x-amz-security-token
pub fn calculate_signing_headers<'a, B>(
    request: &'a http::Request<B>,
    body: SignableBody,
    params: &'a SigningParams<'a>,
) -> Result<SigningOutput<impl Iterator<Item = (&'static str, HeaderValue)>>, Error> {
    // Step 1: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-canonical-request.html.
    let SigningParams {
        access_key,
        secret_key,
        security_token,
        region,
        service_name,
        date_time,
        settings,
    } = params;
    let (creq, extra_headers) =
        CanonicalRequest::from(request, body, settings, *date_time, *security_token)?;

    // Step 2: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-string-to-sign.html.
    let encoded_creq = &sha256_hex_string(creq.to_string().as_bytes());
    let sts = StringToSign::new(*date_time, region, service_name, encoded_creq);

    // Step 3: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-calculate-signature.html
    let signing_key = generate_signing_key(secret_key, date_time.date(), region, service_name);
    let signature = calculate_signature(signing_key, &sts.to_string().as_bytes());

    // Step 4: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-add-signature-to-request.html
    let mut authorization: HeaderValue =
        build_authorization_header(access_key, &creq, sts, &signature).parse()?;
    authorization.set_sensitive(true);

    // Construct an iterator of headers that the caller can attach to their request
    // either as headers or as query parameters to create a presigned URL
    let date = (X_AMZ_DATE, extra_headers.x_amz_date);
    let mut security_token = extra_headers
        .x_amz_security_token
        .map(|tok| (X_AMZ_SECURITY_TOKEN, tok));
    let mut content = extra_headers
        .x_amz_content_256
        .map(|content| (X_AMZ_CONTENT_SHA_256, content));
    let auth = iter::once(("authorization", authorization));
    let date = iter::once(date);
    Ok(SigningOutput::new(
        auth.chain(date).chain(iter::from_fn(move || {
            security_token.take().or_else(|| content.take())
        })),
        signature,
    ))
}

// add signature to authorization header
// Authorization: algorithm Credential=access key ID/credential scope, SignedHeaders=SignedHeaders, Signature=signature
fn build_authorization_header(
    access_key: &str,
    creq: &CanonicalRequest,
    sts: StringToSign,
    signature: &str,
) -> String {
    format!(
        "{} Credential={}/{}, SignedHeaders={}, Signature={}",
        HMAC_256,
        access_key,
        sts.scope.to_string(),
        creq.signed_headers,
        signature
    )
}

#[cfg(test)]
mod tests {
    use super::{
        build_authorization_header, Error, PayloadChecksumKind, SignableBody, SigningSettings,
    };
    use crate::date_fmt::{format_date_time, parse_date_time};
    use crate::http_request::canonical_request::{CanonicalRequest, Scope, StringToSign};
    use crate::sign::{calculate_signature, generate_signing_key, sha256_hex_string};
    use http::{HeaderValue, Method, Request, Uri, Version};
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::{convert::TryFrom, str::FromStr};

    macro_rules! assert_req_eq {
        ($a:tt, $b:tt) => {
            assert_eq!(format!("{:?}", $a), format!("{:?}", $b))
        };
    }

    macro_rules! read {
        (req: $case:tt) => {
            fs::read_to_string(format!("./aws-sig-v4-test-suite/{}/{}.req", $case, $case))?
                // this replacement is necessary for tests to pass on Windows, as reading the
                // sigv4 snapshots from the file system results in CRLF line endings being inserted.
                .replace("\r\n", "\n")
        };

        (creq: $case:tt) => {
            fs::read_to_string(format!("./aws-sig-v4-test-suite/{}/{}.creq", $case, $case))?
                .replace("\r\n", "\n")
        };

        (sreq: $case:tt) => {
            fs::read_to_string(format!("./aws-sig-v4-test-suite/{}/{}.sreq", $case, $case))?
                .replace("\r\n", "\n")
        };

        (sts: $case:tt) => {
            fs::read_to_string(format!("./aws-sig-v4-test-suite/{}/{}.sts", $case, $case))?
                .replace("\r\n", "\n")
        };

        (authz: $case:tt) => {
            fs::read_to_string(format!("./aws-sig-v4-test-suite/{}/{}.authz", $case, $case))?
                .replace("\r\n", "\n")
        };
    }

    #[test]
    fn read_request() -> Result<(), Error> {
        //file-name.req—the web request to be signed.
        //file-name.creq—the resulting canonical request.
        //file-name.sts—the resulting string to sign.
        //file-name.authz—the Authorization header.
        //file-name.sreq— the signed request.

        // Step 1: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-canonical-request.html.
        let s = read!(req: "get-vanilla-query-order-key-case");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20150830T123600Z").unwrap();
        let (creq, _) = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body()),
            &SigningSettings::default(),
            date,
            None,
        )?;

        let actual = format!("{}", creq);
        let expected = read!(creq: "get-vanilla-query-order-key-case");
        assert_eq!(actual, expected);

        // Step 2: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-create-string-to-sign.html.
        let encoded_creq = &sha256_hex_string(creq.to_string().as_bytes());
        let sts = StringToSign::new(date, "us-east-1", "service", encoded_creq);

        // Step 3: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-calculate-signature.html
        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";

        let signing_key = generate_signing_key(secret, date.date(), "us-east-1", "service");
        let signature = calculate_signature(signing_key, &sts.to_string().as_bytes());
        let access = "AKIDEXAMPLE";

        // step 4: https://docs.aws.amazon.com/en_pv/general/latest/gr/sigv4-add-signature-to-request.html
        let authorization = build_authorization_header(access, &creq, sts, &signature);
        let x_azn_date = format_date_time(&date);

        let s = read!(req: "get-vanilla-query-order-key-case");
        let mut req = parse_request(s.as_bytes())?;

        let headers = req.headers_mut();
        headers.insert("X-Amz-Date", x_azn_date.parse()?);
        headers.insert("authorization", authorization.parse()?);
        let expected = read!(sreq: "get-vanilla-query-order-key-case");
        let expected = parse_request(expected.as_bytes())?;
        assert_req_eq!(expected, req);

        Ok(())
    }

    #[test]
    fn test_set_xamz_sha_256() -> Result<(), Error> {
        let s = read!(req: "get-vanilla-query-order-key-case");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20150830T123600Z").unwrap();
        let mut signing_settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let (creq, new_headers) = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body()),
            &signing_settings,
            date,
            None,
        )?;
        assert_eq!(
            new_headers.x_amz_content_256,
            Some(HeaderValue::from_static(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            ))
        );
        // assert that the sha256 header was added
        assert_eq!(
            creq.signed_headers.to_string(),
            "host;x-amz-content-sha256;x-amz-date"
        );

        signing_settings.payload_checksum_kind = PayloadChecksumKind::NoHeader;
        let (creq, new_headers) = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body()),
            &signing_settings,
            date,
            None,
        )?;
        assert_eq!(new_headers.x_amz_content_256, None);
        assert_eq!(creq.signed_headers.to_string(), "host;x-amz-date");
        Ok(())
    }

    #[test]
    fn test_unsigned_payload() -> Result<(), Error> {
        let s = read!(req: "get-vanilla-query-order-key-case");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20150830T123600Z").unwrap();
        let signing_settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let (creq, new_headers) = CanonicalRequest::from(
            &req,
            SignableBody::UnsignedPayload,
            &signing_settings,
            date,
            None,
        )?;
        assert_eq!(
            new_headers.x_amz_content_256,
            Some(HeaderValue::from_static("UNSIGNED-PAYLOAD"))
        );
        assert_eq!(creq.payload_hash, "UNSIGNED-PAYLOAD");
        Ok(())
    }

    #[test]
    fn test_precomputed_payload() -> Result<(), Error> {
        let s = read!(req: "get-vanilla-query-order-key-case");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20150830T123600Z").unwrap();
        let signing_settings = SigningSettings {
            payload_checksum_kind: PayloadChecksumKind::XAmzSha256,
            ..Default::default()
        };
        let (creq, new_headers) = CanonicalRequest::from(
            &req,
            SignableBody::Precomputed(String::from(
                "44ce7dd67c959e0d3524ffac1771dfbba87d2b6b4b4e99e42034a8b803f8b072",
            )),
            &signing_settings,
            date,
            None,
        )?;
        assert_eq!(
            new_headers.x_amz_content_256,
            Some(HeaderValue::from_static(
                "44ce7dd67c959e0d3524ffac1771dfbba87d2b6b4b4e99e42034a8b803f8b072"
            ))
        );
        assert_eq!(
            creq.payload_hash,
            "44ce7dd67c959e0d3524ffac1771dfbba87d2b6b4b4e99e42034a8b803f8b072"
        );
        Ok(())
    }

    #[test]
    fn test_build_authorization_header() -> Result<(), Error> {
        let s = read!(req: "get-vanilla-query-order-key-case");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20150830T123600Z").unwrap();
        let creq = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body()),
            &SigningSettings::default(),
            date,
            None,
        )?
        .0;

        let encoded_creq = &sha256_hex_string(creq.to_string().as_bytes());
        let sts = StringToSign::new(date, "us-east-1", "service", encoded_creq);

        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
        let signing_key = generate_signing_key(secret, date.date(), "us-east-1", "service");
        let signature = calculate_signature(signing_key, &sts.to_string().as_bytes());
        let expected_header = read!(authz: "get-vanilla-query-order-key-case");
        let header = build_authorization_header("AKIDEXAMPLE", &creq, sts, &signature);
        assert_eq!(expected_header, header);

        Ok(())
    }

    #[test]
    fn test_generate_scope() -> Result<(), Error> {
        let expected = "20150830/us-east-1/iam/aws4_request\n";
        let date = parse_date_time("20150830T123600Z")?;
        let scope = Scope {
            date: date.date(),
            region: "us-east-1",
            service: "iam",
        };
        assert_eq!(format!("{}\n", scope.to_string()), expected);

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), Error> {
        let buf = read!(req: "post-header-key-case");
        parse_request(buf.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_read_query_params() -> Result<(), Error> {
        let buf = read!(req: "get-vanilla-query-order-key-case");
        parse_request(buf.as_bytes()).unwrap();
        Ok(())
    }

    #[test]
    fn test_parse_headers() {
        let buf = b"Host:example.amazonaws.com\nX-Amz-Date:20150830T123600Z\n\nblah blah";
        let mut headers = [httparse::EMPTY_HEADER; 4];
        assert_eq!(
            httparse::parse_headers(buf, &mut headers),
            Ok(httparse::Status::Complete((
                56,
                &[
                    httparse::Header {
                        name: "Host",
                        value: b"example.amazonaws.com",
                    },
                    httparse::Header {
                        name: "X-Amz-Date",
                        value: b"20150830T123600Z",
                    }
                ][..]
            )))
        );
    }

    #[test]
    fn sign_payload_empty_string() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let actual = sha256_hex_string(&[]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_string_to_sign() -> Result<(), Error> {
        let date = parse_date_time("20150830T123600Z")?;
        let creq = read!(creq: "get-vanilla-query-order-key-case");
        let expected_sts = read!(sts: "get-vanilla-query-order-key-case");
        let encoded = sha256_hex_string(creq.as_bytes());

        let actual = StringToSign::new(date, "us-east-1", "service", &encoded);
        assert_eq!(expected_sts, actual.to_string());

        Ok(())
    }

    #[test]
    fn test_signature_calculation() -> Result<(), Error> {
        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
        let creq =
            std::fs::read_to_string("./aws-sig-v4-test-suite/iam.creq")?.replace("\r\n", "\n");
        let date = parse_date_time("20150830T123600Z")?;

        let derived_key = generate_signing_key(secret, date.date(), "us-east-1", "iam");
        let signature = calculate_signature(derived_key, creq.as_bytes());

        let expected = "5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7";
        assert_eq!(expected, &signature);

        Ok(())
    }

    #[test]
    fn parse_signed_request() -> Result<(), Error> {
        let req = read!(sreq: "post-header-key-case");
        let _: Request<_> = parse_request(req.as_bytes())?;
        Ok(())
    }

    #[test]
    fn read_sts() -> Result<(), Error> {
        let sts = read!(sts: "get-vanilla-query-order-key-case");
        let _ = StringToSign::try_from(sts.as_ref())?;
        Ok(())
    }

    #[test]
    fn test_digest_of_canonical_request() -> Result<(), Error> {
        let creq = read!(creq: "get-vanilla-query-order-key-case");
        let actual = sha256_hex_string(creq.as_bytes());
        let expected = "816cd5b414d056048ba4f7c5386d6e0533120fb1fcfa93762cf0fc39e2cf19e0";

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn test_double_url_encode() -> Result<(), Error> {
        let s = read!(req: "double-url-encode");
        let req = parse_request(s.as_bytes())?;
        let date = parse_date_time("20210511T154045Z")?;
        let creq = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body()),
            &SigningSettings::default(),
            date,
            None,
        )?
        .0;

        let actual = format!("{}", creq);
        let expected = read!(creq: "double-url-encode");
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    fn test_tilde_in_uri() -> Result<(), Error> {
        let req = http::Request::builder()
            .uri("https://s3.us-east-1.amazonaws.com/my-bucket?list-type=2&prefix=~objprefix&single&k=&unreserved=-_.~").body("").unwrap();
        let date = parse_date_time("20210511T154045Z")?;
        let creq = CanonicalRequest::from(
            &req,
            SignableBody::Bytes(req.body().as_ref()),
            &SigningSettings::default(),
            date,
            None,
        )?
        .0;
        assert_eq!(
            creq.params,
            "k=&list-type=2&prefix=~objprefix&single=&unreserved=-_.~"
        );
        Ok(())
    }

    fn parse_request(s: &[u8]) -> Result<Request<bytes::Bytes>, Error> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        // httparse 1.5 requres two trailing newlines to head the header section.
        let mut with_newline = Vec::from(s);
        with_newline.push(b'\n');
        let mut req = httparse::Request::new(&mut headers);
        let _ = req.parse(&with_newline).unwrap();

        let version = match req.version.unwrap() {
            1 => Version::HTTP_11,
            _ => unimplemented!(),
        };

        let method = match req.method.unwrap() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            _ => unimplemented!(),
        };

        let builder = Request::builder();
        let builder = builder.version(version);
        let mut builder = builder.method(method);
        if let Some(path) = req.path {
            builder = builder.uri(Uri::from_str(path)?);
        }
        for header in req.headers {
            let name = header.name.to_lowercase();
            if !name.is_empty() {
                builder = builder.header(&name, header.value);
            }
        }

        let req = builder.body(bytes::Bytes::new())?;
        Ok(req)
    }
}
