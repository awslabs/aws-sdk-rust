/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! What a `ChecksumCapture` attached to a GetObject call reports.

#![cfg(feature = "test-util")]

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::{Credentials, Region, ResponseChecksumValidation};
use aws_sdk_s3::types::ChecksumMode;
use aws_sdk_s3::{Client, Config};
use aws_smithy_http_client::test_util::{NeverClient, ReplayEvent, StaticReplayClient};
use aws_smithy_observability::unit_of_work::checksums::{
    ChecksumCapture, ChecksumOutcome, IncompleteCause, NotValidatedReason,
    ResponseChecksumAlgorithm,
};
use aws_smithy_observability::unit_of_work::Capture;
use aws_smithy_runtime_api::client::http::HttpClient;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::retry::RetryConfig;
use std::time::Duration;

/// The CRC32 of "Hello world", in base64.
const HELLO_WORLD_CRC32: &str = "i9aeUg==";

fn response(status: u16, headers: &[(&str, &str)], body: &'static str) -> ReplayEvent {
    let mut response = http_1x::Response::builder().status(status);
    for (name, value) in headers {
        response = response.header(*name, *value);
    }
    ReplayEvent::new(
        http_1x::Request::builder()
            .uri("https://test-bucket.s3.us-east-1.amazonaws.com/test.txt?x-id=GetObject")
            .body(SdkBody::empty())
            .unwrap(),
        response.body(SdkBody::from(body)).unwrap(),
    )
}

fn hello_world(status: u16, crc32: &str) -> ReplayEvent {
    response(
        status,
        &[("content-length", "11"), ("x-amz-checksum-crc32", crc32)],
        "Hello world",
    )
}

fn slow_down() -> ReplayEvent {
    // The checksum header is wrong for this body. Error responses are never validated.
    response(
        503,
        &[("x-amz-checksum-crc32", "AAAAAA==")],
        "<Error><Code>SlowDown</Code><Message>Please reduce your request rate.</Message></Error>",
    )
}

fn client(http_client: impl HttpClient + 'static) -> Client {
    client_with(Config::builder().http_client(http_client))
}

fn client_with(config: aws_sdk_s3::config::Builder) -> Client {
    Client::from_conf(
        config
            .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
            .region(Region::new("us-east-1"))
            .retry_config(RetryConfig::standard().with_initial_backoff(Duration::from_millis(1)))
            .with_test_defaults()
            .build(),
    )
}

/// Sends a GetObject with `captures` attached and reads its body. Returns the output's body error.
async fn get_and_read(client: &Client, captures: &[&ChecksumCapture]) -> Option<String> {
    let mut request = client
        .get_object()
        .bucket("test-bucket")
        .key("test.txt")
        .checksum_mode(ChecksumMode::Enabled)
        .customize();
    for checksum in captures {
        request = request.interceptor(Capture::new().with(*checksum));
    }
    let output = request.send().await.expect("GetObject succeeds");
    output.body.collect().await.err().map(|e| e.to_string())
}

fn validated() -> Option<ChecksumOutcome> {
    Some(ChecksumOutcome::Validated {
        algorithm: ResponseChecksumAlgorithm::Crc32,
        computed: HELLO_WORLD_CRC32.into(),
    })
}

fn not_validated(reason: NotValidatedReason) -> Option<ChecksumOutcome> {
    Some(ChecksumOutcome::NotValidated { reason })
}

#[tokio::test]
async fn the_successful_attempt_after_two_503s_validates() {
    let http_client = StaticReplayClient::new(vec![
        slow_down(),
        slow_down(),
        hello_world(206, HELLO_WORLD_CRC32),
    ]);
    let checksum = ChecksumCapture::new();

    let body_error = get_and_read(&client(http_client.clone()), &[&checksum]).await;
    assert_eq!(body_error, None);
    assert_eq!(http_client.actual_requests().count(), 3);
    assert_eq!(checksum.try_outcome(), validated());
}

#[tokio::test]
async fn an_error_response_with_a_checksum_header_keeps_its_modeled_error() {
    let http_client = StaticReplayClient::new(vec![response(
        404,
        &[("x-amz-checksum-crc32", "AAAAAA==")],
        "<Error><Code>NoSuchKey</Code><Message>The specified key does not exist.</Message></Error>",
    )]);
    let checksum = ChecksumCapture::new();

    let error = client(http_client.clone())
        .get_object()
        .bucket("test-bucket")
        .key("test.txt")
        .checksum_mode(ChecksumMode::Enabled)
        .customize()
        .interceptor(Capture::new().with(&checksum))
        .send()
        .await
        .expect_err("the object does not exist");
    assert!(
        error.into_service_error().is_no_such_key(),
        "a checksum mismatch replaced the modeled error"
    );
    assert_eq!(http_client.actual_requests().count(), 1);
    assert_eq!(
        checksum.try_outcome(),
        not_validated(NotValidatedReason::ErrorResponse)
    );
}

#[tokio::test]
async fn a_mismatch_is_in_the_capture_when_the_body_error_reaches_the_caller() {
    let http_client = StaticReplayClient::new(vec![hello_world(200, "AAAAAA==")]);
    let checksum = ChecksumCapture::new();

    let output = client(http_client)
        .get_object()
        .bucket("test-bucket")
        .key("test.txt")
        .checksum_mode(ChecksumMode::Enabled)
        .customize()
        .interceptor(Capture::new().with(&checksum))
        .send()
        .await
        .unwrap();
    assert_eq!(checksum.try_outcome(), None, "the body has not been read");

    output
        .body
        .collect()
        .await
        .expect_err("the checksum is wrong");
    assert_eq!(
        checksum.try_outcome(),
        Some(ChecksumOutcome::Mismatch {
            algorithm: ResponseChecksumAlgorithm::Crc32,
            expected: "AAAAAA==".into(),
            computed: HELLO_WORLD_CRC32.into(),
        })
    );
}

#[tokio::test]
async fn a_body_shorter_than_its_content_length_reads_incomplete() {
    // If content-length enforcement wrapped the checksum body instead of sitting inside it, the
    // checksum would match first and this would read `Validated`.
    let http_client = StaticReplayClient::new(vec![response(
        200,
        &[
            ("content-length", "20"),
            ("x-amz-checksum-crc32", HELLO_WORLD_CRC32),
        ],
        "Hello world",
    )]);
    let checksum = ChecksumCapture::new();

    let body_error = get_and_read(&client(http_client), &[&checksum]).await;
    assert!(body_error.is_some(), "the body is short");
    assert_eq!(
        checksum.try_outcome(),
        Some(ChecksumOutcome::Incomplete {
            algorithm: ResponseChecksumAlgorithm::Crc32,
            cause: IncompleteCause::BodyError,
        })
    );
}

#[tokio::test]
async fn a_checksum_the_sdk_does_not_compute_reads_unsupported_algorithm() {
    let http_client = StaticReplayClient::new(vec![response(
        200,
        &[
            ("content-length", "11"),
            ("x-amz-checksum-md5", "PiWWCnnbxptnTNTsZ6csYg=="),
        ],
        "Hello world",
    )]);
    let checksum = ChecksumCapture::new();

    let body_error = get_and_read(&client(http_client), &[&checksum]).await;
    assert_eq!(body_error, None);
    assert_eq!(
        checksum.try_outcome(),
        not_validated(NotValidatedReason::UnsupportedAlgorithm)
    );
}

#[tokio::test]
async fn two_captures_on_one_call_both_validate() {
    let http_client = StaticReplayClient::new(vec![hello_world(200, HELLO_WORLD_CRC32)]);
    let (first, second) = (ChecksumCapture::new(), ChecksumCapture::new());

    let body_error = get_and_read(&client(http_client), &[&first, &second]).await;
    assert_eq!(body_error, None);
    assert_eq!(first.try_outcome(), validated());
    assert_eq!(second.try_outcome(), validated());
}

#[tokio::test]
async fn the_same_handle_in_two_captures_records_the_call_once() {
    let http_client = StaticReplayClient::new(vec![hello_world(200, HELLO_WORLD_CRC32)]);
    let checksum = ChecksumCapture::new();

    let body_error = get_and_read(&client(http_client), &[&checksum, &checksum]).await;
    assert_eq!(body_error, None);
    assert_eq!(checksum.try_outcome(), validated());
}

#[tokio::test]
async fn a_client_level_capture_never_reports_one_call_for_another() {
    let http_client = StaticReplayClient::new(vec![
        hello_world(200, HELLO_WORLD_CRC32),
        hello_world(200, HELLO_WORLD_CRC32),
    ]);
    let checksum = ChecksumCapture::new();
    let client = client_with(
        Config::builder()
            .http_client(http_client)
            .interceptor(Capture::new().with(&checksum)),
    );

    get_and_read(&client, &[]).await;
    assert_eq!(checksum.try_outcome(), validated());
    get_and_read(&client, &[]).await;
    assert_eq!(
        checksum.try_outcome(),
        not_validated(NotValidatedReason::Reused)
    );
}

#[tokio::test]
async fn a_cancelled_call_reads_no_response() {
    let checksum = ChecksumCapture::new();
    let send = client(NeverClient::new())
        .get_object()
        .bucket("test-bucket")
        .key("test.txt")
        .customize()
        .interceptor(Capture::new().with(&checksum))
        .send();

    tokio::time::timeout(Duration::from_millis(100), send)
        .await
        .expect_err("the HTTP client never responds");
    assert_eq!(
        checksum.try_outcome(),
        not_validated(NotValidatedReason::NoResponse)
    );
}

#[tokio::test]
async fn validation_turned_off_on_the_client_reads_disabled() {
    let http_client = StaticReplayClient::new(vec![hello_world(200, HELLO_WORLD_CRC32)]);
    let checksum = ChecksumCapture::new();
    let client = client_with(
        Config::builder()
            .http_client(http_client)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired),
    );

    let output = client
        .get_object()
        .bucket("test-bucket")
        .key("test.txt")
        .customize()
        .interceptor(Capture::new().with(&checksum))
        .send()
        .await
        .unwrap();
    output.body.collect().await.unwrap();
    assert_eq!(
        checksum.try_outcome(),
        not_validated(NotValidatedReason::Disabled)
    );
}
