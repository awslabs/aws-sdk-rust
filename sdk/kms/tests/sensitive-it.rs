/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::retry::AwsErrorRetryPolicy;
use aws_sdk_kms as kms;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation::{self, Parts};
use aws_smithy_http::response::ParseStrictResponse;
use aws_smithy_http::result::SdkError;
use aws_smithy_http::retry::ClassifyResponse;
use aws_smithy_types::retry::{ErrorKind, RetryKind};
use bytes::Bytes;
use kms::error::CreateAliasError;
use kms::operation::{CreateAlias, GenerateRandom};
use kms::output::GenerateRandomOutput;
use kms::types::Blob;

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
fn assert_debug<T: std::fmt::Debug>() {}

#[tokio::test]
async fn types_are_send_sync() {
    assert_send_sync::<kms::Error>();
    assert_send_sync::<kms::types::SdkError<CreateAliasError>>();
    assert_send_sync::<kms::error::CreateAliasError>();
    assert_send_sync::<kms::output::CreateAliasOutput>();
    assert_send_sync::<kms::Client>();
    assert_send_sync::<GenerateRandom>();
    let conf = kms::Config::builder().build();
    assert_send_fut(kms::Client::from_conf(conf).list_keys().send());
}

#[tokio::test]
async fn client_is_debug() {
    let conf = kms::Config::builder().build();
    let client = kms::Client::from_conf(conf);
    assert_ne!(format!("{:?}", client), "");
}

#[tokio::test]
async fn client_is_clone() {
    let conf = kms::Config::builder().build();
    let client = kms::Client::from_conf(conf);
    let _ = client.clone();
}

#[test]
fn types_are_debug() {
    assert_debug::<kms::Client>();
    assert_debug::<kms::client::fluent_builders::GenerateRandom>();
    assert_debug::<kms::client::fluent_builders::CreateAlias>();
}

async fn create_alias_op() -> Parts<CreateAlias, AwsErrorRetryPolicy> {
    let conf = kms::Config::builder().build();
    let (_, parts) = CreateAlias::builder()
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .expect("valid request")
        .into_request_response();
    parts
}

/// Parse a semi-real response body and assert that the correct retry status is returned
#[tokio::test]
async fn errors_are_retryable() {
    let op = create_alias_op().await;
    let http_response = http::Response::builder()
        .status(400)
        .body(Bytes::from_static(
            br#"{ "code": "LimitExceededException" }"#,
        ))
        .unwrap();
    let err = op
        .response_handler
        .parse(&http_response)
        .map_err(|e| SdkError::ServiceError {
            err: e,
            raw: operation::Response::new(http_response.map(SdkBody::from)),
        });
    let retry_kind = op.retry_policy.classify(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}

#[tokio::test]
async fn unmodeled_errors_are_retryable() {
    let op = create_alias_op().await;
    let http_response = http::Response::builder()
        .status(400)
        .body(Bytes::from_static(br#"{ "code": "ThrottlingException" }"#))
        .unwrap();
    let err = op
        .response_handler
        .parse(&http_response)
        .map_err(|e| SdkError::ServiceError {
            err: e,
            raw: operation::Response::new(http_response.map(SdkBody::from)),
        });
    let retry_kind = op.retry_policy.classify(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}
