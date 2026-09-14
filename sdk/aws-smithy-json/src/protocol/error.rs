/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared error envelope parsing for AWS JSON protocols.
//!
//! Used by `AwsJsonRpcProtocol` (`awsJson1_0` / `awsJson1_1`) and by
//! `AwsRestJsonProtocol` (`restJson1`) to implement
//! `ClientProtocolInner::parse_error_metadata`.
//!
//! Both protocols use the same envelope shape:
//! - Top-level body keys: `__type` (or `code`) for the error code, and
//!   `message` / `Message` / `errorMessage` for the error message.
//! - Header `X-Amzn-Errortype` takes priority over the body `__type` /
//!   `code` keys.
//! - The error code is sanitized: any namespace prefix (`namespace#Name`)
//!   and any URL suffix (`Name:http://...`) are stripped.
//!
//! `restJson1` does not strictly mandate the body envelope (some services
//! return only the header), but the parsing logic is forgiving — missing
//! fields just produce an empty builder.

use crate::deserialize::token::skip_value;
use crate::deserialize::{json_token_iter, Token};
use aws_smithy_runtime_api::http::Headers;
use aws_smithy_schema::error_envelope::{parse_query_compatible_header, sanitize_error_code};
use aws_smithy_schema::serde::SerdeError;
use aws_smithy_types::error::metadata::{Builder as ErrorMetadataBuilder, ErrorMetadata};
use std::borrow::Cow;

struct ErrorBody<'a> {
    code: Option<Cow<'a, str>>,
    message: Option<Cow<'a, str>>,
}

/// Walks the top-level keys of a JSON object body looking for `code`,
/// `__type`, and `message` / `Message` / `errorMessage`.
///
/// Per the legacy `inlineable/src/json_errors.rs` behavior:
/// - If both `code` and `__type` are present, `code` wins.
/// - Unknown keys are skipped.
/// - An empty body or non-object root is acceptable — both fields stay `None`.
fn parse_error_body(bytes: &[u8]) -> Result<ErrorBody<'_>, SerdeError> {
    let mut tokens = json_token_iter(bytes).peekable();
    let (mut typ, mut code, mut message) = (None, None, None);
    let first = match tokens.next().transpose() {
        Ok(t) => t,
        Err(e) => return Err(invalid_input(format!("malformed JSON error body: {e}"))),
    };
    if let Some(Token::StartObject { .. }) = first {
        loop {
            let next = match tokens.next().transpose() {
                Ok(t) => t,
                Err(e) => return Err(invalid_input(format!("malformed JSON error body: {e}"))),
            };
            match next {
                Some(Token::EndObject { .. }) => break,
                Some(Token::ObjectKey { key, .. }) => {
                    if let Some(Ok(Token::ValueString { value, .. })) = tokens.peek() {
                        match key.as_escaped_str() {
                            "code" => {
                                code = Some(value.to_unescaped().map_err(|e| {
                                    invalid_input(format!(
                                        "malformed string in error body 'code': {e}"
                                    ))
                                })?)
                            }
                            "__type" => {
                                typ = Some(value.to_unescaped().map_err(|e| {
                                    invalid_input(format!(
                                        "malformed string in error body '__type': {e}"
                                    ))
                                })?)
                            }
                            "message" | "Message" | "errorMessage" => {
                                message = Some(value.to_unescaped().map_err(|e| {
                                    invalid_input(format!(
                                        "malformed string in error body 'message': {e}"
                                    ))
                                })?)
                            }
                            _ => {}
                        }
                    }
                    if let Err(e) = skip_value(&mut tokens) {
                        return Err(invalid_input(format!(
                            "malformed JSON error body while skipping value: {e}"
                        )));
                    }
                }
                _ => {
                    return Err(invalid_input(
                        "expected object key or end of object in error body",
                    ));
                }
            }
        }
    }
    Ok(ErrorBody {
        // `code` wins over `__type` when both are present (legacy behavior).
        code: code.or(typ),
        message,
    })
}

fn invalid_input(message: impl Into<String>) -> SerdeError {
    SerdeError::invalid_input(message)
}

/// Extracts canonical error metadata (code + message) from a JSON-protocol
/// response.
///
/// Header `X-Amzn-Errortype` takes priority over the body's `code` /
/// `__type` keys. The resulting code is sanitized via
/// [`sanitize_error_code`].
///
/// Per the contract on
/// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata),
/// the request id is **not** populated here — it's attached separately by
/// the orchestrator's request-id pipeline.
pub(crate) fn parse_error_envelope_metadata(
    payload: &[u8],
    headers: &Headers,
) -> Result<ErrorMetadataBuilder, SerdeError> {
    let ErrorBody { code, message } = parse_error_body(payload)?;

    let mut builder = ErrorMetadata::builder();
    // Header takes precedence over body. Matches the legacy
    // `inlineable/src/json_errors.rs::parse_error_metadata` behavior.
    if let Some(code) = headers
        .get("x-amzn-errortype")
        .or(code.as_deref())
        .map(sanitize_error_code)
    {
        builder = builder.code(code);
    }
    if let Some(message) = message {
        builder = builder.message(message);
    }
    // `@awsQueryCompatible` services emit `X-Amzn-Query-Error: <code>;<type>`
    // and the awsQueryError code (the part before `;`) is the dispatch key
    // baked into generated code. Per the legacy
    // `codegen-core/.../AwsQueryCompatible.kt` wrapper, this header overrides
    // any body-derived code and adds the AWS error type as the `type` extra.
    //
    // Non-queryCompatible services do not emit this header, so the override
    // is a no-op for them.
    if let Some((qc_code, qc_type)) = parse_query_compatible_header(headers) {
        builder = builder.code(qc_code).custom("type", qc_type);
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;

    fn response(headers: &[(&str, &str)], body: &str) -> Response {
        let mut response = Response::new(StatusCode::try_from(400).unwrap(), SdkBody::from(body));
        for (name, value) in headers {
            response
                .headers_mut()
                .insert(name.to_string(), value.to_string());
        }
        response
    }

    #[test]
    fn extracts_code_and_message_from_body() {
        let resp = response(&[], r#"{"__type":"FooError","message":"go to foo"}"#);
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FooError"));
        assert_eq!(meta.message(), Some("go to foo"));
    }

    #[test]
    fn body_code_takes_priority_over_type() {
        let resp = response(&[], r#"{"code":"BarError","__type":"FooError"}"#);
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("BarError"));
    }

    #[test]
    fn header_takes_priority_over_body() {
        let resp = response(
            &[("x-amzn-errortype", "HeaderError")],
            r#"{"__type":"BodyError","message":"hi"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("HeaderError"));
        assert_eq!(meta.message(), Some("hi"));
    }

    #[test]
    fn sanitizes_namespace_prefix() {
        let resp = response(&[], r#"{"__type":"aws.protocoltests.restjson#FooError"}"#);
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn sanitizes_url_suffix() {
        let resp = response(
            &[],
            r#"{"__type":"FooError:http://internal.amazon.com/coral/foo/"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn sanitizes_namespace_and_url() {
        let resp = response(&[("x-amzn-errortype", "ns#FooError:http://example/")], "");
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn alternative_message_keys() {
        // `Message` is the lambda-style alternate spelling.
        let resp = response(
            &[("x-amzn-errortype", "ResourceNotFoundException")],
            r#"{"Type":"User","Message":"Functions from us-west-2 are not reachable from us-east-1"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("ResourceNotFoundException"));
        assert_eq!(
            meta.message(),
            Some("Functions from us-west-2 are not reachable from us-east-1")
        );
    }

    #[test]
    fn empty_body_returns_empty_builder() {
        let resp = response(&[], "");
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert!(meta.code().is_none());
        assert!(meta.message().is_none());
    }

    #[test]
    fn malformed_body_returns_invalid_input() {
        let resp = response(&[], r#"{"__type":"FooError""#); // truncated
        let err = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap_err();
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn ignores_unrecognized_keys() {
        let resp = response(
            &[],
            r#"{"__type":"FooError","extra":5,"nested":{"x":1},"foo":"bar"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn query_compat_header_overrides_body_code() {
        // Legacy `AwsQueryCompatible.kt` wrapper behavior: when
        // X-Amzn-Query-Error is present, its code overrides the body code,
        // and the AWS error type lands in the `type` custom field.
        let resp = response(
            &[("x-amzn-query-error", "Customized;Sender")],
            r#"{"__type":"CustomCodeError","message":"Hi"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("Customized"));
        assert_eq!(meta.message(), Some("Hi"));
        assert_eq!(meta.extra("type"), Some("Sender"));
    }

    #[test]
    fn query_compat_header_overrides_x_amzn_errortype() {
        // queryCompatible header beats the standard error-type header too.
        let resp = response(
            &[
                ("x-amzn-errortype", "FromHeaderType"),
                ("x-amzn-query-error", "FromQuery;Receiver"),
            ],
            "",
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("FromQuery"));
        assert_eq!(meta.extra("type"), Some("Receiver"));
    }

    #[test]
    fn malformed_query_compat_header_is_ignored() {
        // Header without the `;` separator is ignored — body code wins.
        let resp = response(
            &[("x-amzn-query-error", "no-separator-here")],
            r#"{"__type":"BodyError"}"#,
        );
        let meta = parse_error_envelope_metadata(resp.body().bytes().unwrap(), resp.headers())
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("BodyError"));
        assert_eq!(meta.extra("type"), None);
    }
}
