/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_kms as kms;
use kms::error::CreateAliasError;
use kms::operation::{CreateAlias, GenerateRandom};
use kms::output::GenerateRandomOutput;
use kms::Blob;
use smithy_http::middleware::ResponseBody;
use smithy_http::result::SdkError;
use smithy_http::retry::ClassifyResponse;
use smithy_types::retry::{ErrorKind, RetryKind};

#[test]
fn validate_sensitive_trait() {
    let output = GenerateRandomOutput::builder()
        .plaintext(Blob::new("some output"))
        .build();
    assert_eq!(
        format!("{:?}", output),
        "GenerateRandomOutput { plaintext: \"*** Sensitive Data Redacted ***\" }"
    );
}

fn assert_send_sync<T: Send + Sync + 'static>() {}
fn assert_send_fut<T: Send + 'static>(_: T) {}

#[test]
fn types_are_send_sync() {
    assert_send_sync::<kms::Error>();
    assert_send_sync::<kms::SdkError<CreateAliasError>>();
    assert_send_sync::<kms::error::CreateAliasError>();
    assert_send_sync::<kms::output::CreateAliasOutput>();
    assert_send_sync::<kms::Client>();
    assert_send_sync::<GenerateRandom>();
    assert_send_fut(kms::Client::from_env().list_keys().send());
}

/// Parse a semi-real response body and assert that the correct retry status is returned
#[test]
fn errors_are_retryable() {
    let conf = kms::Config::builder().build();
    let (_, parts) = CreateAlias::builder()
        .build(&conf)
        .expect("valid request")
        .into_request_response();
    let http_response = http::Response::builder()
        .status(400)
        .body(r#"{ "code": "LimitExceededException" }"#)
        .unwrap();
    let err = parts
        .response_handler
        .parse_response(&http_response)
        .map_err(|e| SdkError::ServiceError {
            err: e,
            raw: http_response.map(ResponseBody::from_static),
        });
    let retry_kind = parts.retry_policy.classify(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}

#[test]
fn unmodeled_errors_are_retryable() {
    let conf = kms::Config::builder().build();
    let (_, parts) = CreateAlias::builder()
        .build(&conf)
        .expect("valid request")
        .into_request_response();
    let http_response = http::Response::builder()
        .status(400)
        .body(r#"{ "code": "ThrottlingException" }"#)
        .unwrap();
    let err = parts
        .response_handler
        .parse_response(&http_response)
        .map_err(|e| SdkError::ServiceError {
            err: e,
            raw: http_response.map(ResponseBody::from_static),
        });
    let retry_kind = parts.retry_policy.classify(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}
