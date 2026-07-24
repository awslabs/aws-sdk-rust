/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_sts as sts;
use aws_sdk_sts::config::retry::RetryConfig;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::error::ErrorMetadata;
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
use sts::operation::assume_role_with_web_identity::AssumeRoleWithWebIdentityError;
use sts::types::error::IdpCommunicationErrorException;

#[tokio::test]
async fn idp_comms_err_retryable() {
    let error = AssumeRoleWithWebIdentityError::IdpCommunicationErrorException(
        IdpCommunicationErrorException::builder()
            .message("test")
            .meta(
                ErrorMetadata::builder()
                    .code("IDPCommunicationError")
                    .message("test")
                    .build(),
            )
            .build(),
    );
    assert_eq!(
        Some(ErrorKind::ServerError),
        error.retryable_error_kind(),
        "IdpCommunicationErrorException should be a retryable server error"
    );
}

fn req() -> http_1x::Request<SdkBody> {
    http_1x::Request::builder()
        .body(SdkBody::from("request"))
        .unwrap()
}

// An `awsQuery` STS error response carrying the given error code with a
// non-retryable HTTP status (400). Using a 400 ensures any retry is caused by
// error-code classification, not by the HTTP status code classifier.
fn sts_error(code: &str) -> http_1x::Response<SdkBody> {
    let body = format!(
        r#"<ErrorResponse xmlns="https://sts.amazonaws.com/doc/2011-06-15/">
    <Error>
        <Type>Sender</Type>
        <Code>{code}</Code>
        <Message>test</Message>
    </Error>
    <RequestId>req-id</RequestId>
</ErrorResponse>"#
    );
    http_1x::Response::builder()
        .status(400)
        .body(SdkBody::from(body))
        .unwrap()
}

// Drives `GetCallerIdentity` (which does NOT model `IDPCommunicationErrorException`)
// against three identical error responses and returns the number of attempts the
// client actually made.
async fn attempts_for_error_code(code: &'static str) -> usize {
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), sts_error(code)),
        ReplayEvent::new(req(), sts_error(code)),
        ReplayEvent::new(req(), sts_error(code)),
    ]);
    let conf = sts::Config::builder()
        .http_client(http_client.clone())
        .retry_config(RetryConfig::standard().with_max_attempts(3))
        .with_test_defaults_v2()
        .build();
    let client = sts::Client::from_conf(conf);

    let _ = client
        .get_caller_identity()
        .send()
        .await
        .expect_err("the mocked response is an error");

    http_client.actual_requests().count()
}

// Unlike `idp_comms_err_retryable` above (which covers the *modeled*
// `IDPCommunicationErrorException` on `AssumeRoleWithWebIdentity` via the
// `@retryable` trait), this exercises the `IDPCommunicationError` *error code* on
// `GetCallerIdentity`, which does NOT model that exception. The only thing that
// makes it retryable is the by-error-code classifier that STS registers for every
// operation, so the client must use all 3 attempts.
#[tokio::test]
async fn idp_communication_error_retried_on_operation_that_does_not_model_it() {
    assert_eq!(
        attempts_for_error_code("IDPCommunicationError").await,
        3,
        "IDPCommunicationError should be retried on all STS operations"
    );
}

// Control: an unrelated, non-retryable error code makes exactly one attempt,
// confirming the retry above is specific to `IDPCommunicationError` and not a
// blanket retry of every error.
#[tokio::test]
async fn unrelated_error_code_is_not_retried() {
    assert_eq!(
        attempts_for_error_code("ValidationError").await,
        1,
        "a non-retryable error code should not be retried"
    );
}
