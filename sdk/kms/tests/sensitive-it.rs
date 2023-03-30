/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::retry::AwsResponseRetryClassifier;
use aws_sdk_kms as kms;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation::{self, Parts};
use aws_smithy_http::response::ParseStrictResponse;
use aws_smithy_http::result::SdkError;
use aws_smithy_http::retry::ClassifyRetry;
use aws_smithy_types::retry::{ErrorKind, RetryKind};
use bytes::Bytes;
use kms::operation::create_alias::{CreateAlias, CreateAliasError, CreateAliasInput};
use kms::operation::generate_random::{GenerateRandom, GenerateRandomOutput};
use kms::primitives::Blob;

#[test]
fn validate_sensitive_trait() {
    let builder = GenerateRandomOutput::builder().plaintext(Blob::new("some output"));
    assert_eq!(
        format!("{:?}", builder),
        "GenerateRandomOutputBuilder { plaintext: \"*** Sensitive Data Redacted ***\", _request_id: None }"
    );
    let output = GenerateRandomOutput::builder()
        .plaintext(Blob::new("some output"))
        .build();
    assert_eq!(
        format!("{:?}", output),
        "GenerateRandomOutput { plaintext: \"*** Sensitive Data Redacted ***\", _request_id: None }"
    );
}

fn assert_send_sync<T: Send + Sync + 'static>() {}
fn assert_send_fut<T: Send + 'static>(_: T) {}
fn assert_debug<T: std::fmt::Debug>() {}

#[tokio::test]
async fn types_are_send_sync() {
    assert_send_sync::<kms::Error>();
    assert_send_sync::<kms::error::SdkError<CreateAliasError>>();
    assert_send_sync::<kms::operation::create_alias::CreateAliasError>();
    assert_send_sync::<kms::operation::create_alias::CreateAliasOutput>();
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

    fn is_clone(it: impl Clone) {
        drop(it)
    }

    is_clone(client);
}

#[test]
fn types_are_debug() {
    assert_debug::<kms::Client>();
    assert_debug::<kms::operation::generate_random::builders::GenerateRandomFluentBuilder>();
    assert_debug::<kms::operation::create_alias::builders::CreateAliasFluentBuilder>();
}

async fn create_alias_op() -> Parts<CreateAlias, AwsResponseRetryClassifier> {
    let conf = kms::Config::builder().build();
    let (_, parts) = CreateAliasInput::builder()
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
    let err = op.response_handler.parse(&http_response).map_err(|e| {
        SdkError::service_error(
            e,
            operation::Response::new(http_response.map(SdkBody::from)),
        )
    });
    let retry_kind = op.retry_classifier.classify_retry(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}

#[tokio::test]
async fn unmodeled_errors_are_retryable() {
    let op = create_alias_op().await;
    let http_response = http::Response::builder()
        .status(400)
        .body(Bytes::from_static(br#"{ "code": "ThrottlingException" }"#))
        .unwrap();
    let err = op.response_handler.parse(&http_response).map_err(|e| {
        SdkError::service_error(
            e,
            operation::Response::new(http_response.map(SdkBody::from)),
        )
    });
    let retry_kind = op.retry_classifier.classify_retry(err.as_ref());
    assert_eq!(retry_kind, RetryKind::Error(ErrorKind::ThrottlingError));
}
