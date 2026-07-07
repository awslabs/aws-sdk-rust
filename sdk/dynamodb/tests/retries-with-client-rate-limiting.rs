/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::config::{
    BehaviorVersion, Credentials, Region, SharedAsyncSleep, StalledStreamProtectionConfig,
};
use aws_sdk_dynamodb::{config::retry::RetryConfig, error::ProvideErrorMetadata};
use aws_smithy_async::test_util::instant_time_and_sleep;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_runtime::client::retries::RetryPartition;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::retry::RetrySpec;
use std::time::{Duration, SystemTime};

fn req() -> http_1x::Request<SdkBody> {
    http_1x::Request::builder()
        .body(SdkBody::from("request body"))
        .unwrap()
}

fn ok() -> http_1x::Response<SdkBody> {
    let body = "{ \"TableNames\": [ \"Test\" ] }";
    http_1x::Response::builder()
        .status(200)
        .header("server", "Server")
        .header("content-type", "application/x-amz-json-1.0")
        .header("content-length", body.len().to_string())
        .header("connection", "keep-alive")
        .header("x-amz-crc32", "2335643545")
        .body(SdkBody::from(body))
        .unwrap()
}

fn err() -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(500)
        .body(SdkBody::from("{ \"message\": \"The request has failed because of an unknown error, exception or failure.\", \"code\": \"InternalServerError\" }"))
        .unwrap()
}

fn throttling_err() -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(400)
        .body(SdkBody::from("{ \"message\": \"The request was denied due to request throttling.\", \"code\": \"ThrottlingException\" }"))
        .unwrap()
}

async fn adaptive_retries_no_throttling(
    behavior_version: BehaviorVersion,
    retry_config: RetryConfig,
    partition_name: &str,
    expected_op1_sleep: Duration,
    expected_op2_sleep: Duration,
    expected_op3_sleep: Duration,
) {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);

    let events = vec![
        // First operation
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), ok()),
        // Second operation
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), ok()),
        // Third operation will fail, only errors
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), err()),
    ];

    let http_client = StaticReplayClient::new(events);
    let config = aws_sdk_dynamodb::Config::builder()
        .behavior_version(behavior_version)
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .retry_config(retry_config)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new(partition_name.to_owned()))
        .http_client(http_client.clone())
        .build();
    let expected_table_names = vec!["Test".to_owned()];

    let client = aws_sdk_dynamodb::Client::from_conf(config.clone());
    let res = client.list_tables().send().await.unwrap();
    assert_eq!(sleep_impl.total_duration(), expected_op1_sleep);
    assert_eq!(res.table_names(), expected_table_names.as_slice());
    assert_eq!(http_client.actual_requests().count(), 3);

    let client = aws_sdk_dynamodb::Client::from_conf(config.clone());
    let res = client.list_tables().send().await.unwrap();
    assert_eq!(sleep_impl.total_duration(), expected_op2_sleep);
    assert_eq!(res.table_names(), expected_table_names.as_slice());
    assert_eq!(http_client.actual_requests().count(), 5);

    let client = aws_sdk_dynamodb::Client::from_conf(config);
    let err = client.list_tables().send().await.unwrap_err();
    assert_eq!(sleep_impl.total_duration(), expected_op3_sleep);
    assert_eq!(err.code(), Some("InternalServerError"));
    assert_eq!(http_client.actual_requests().count(), 9);
}

async fn adaptive_retries_with_throttling(
    behavior_version: BehaviorVersion,
    retry_config: RetryConfig,
    partition_name: &str,
    expected_op1_sleep: Duration,
    expected_op2_sleep_min: Duration,
    expected_op2_sleep_max: Duration,
) {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);

    let events = vec![
        // First operation
        ReplayEvent::new(req(), throttling_err()),
        ReplayEvent::new(req(), throttling_err()),
        ReplayEvent::new(req(), ok()),
        // Second operation
        ReplayEvent::new(req(), err()),
        ReplayEvent::new(req(), ok()),
    ];

    let http_client = StaticReplayClient::new(events);
    let config = aws_sdk_dynamodb::Config::builder()
        .behavior_version(behavior_version)
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .retry_config(retry_config)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new(partition_name.to_owned()))
        .http_client(http_client.clone())
        .build();
    let expected_table_names = vec!["Test".to_owned()];

    let client = aws_sdk_dynamodb::Client::from_conf(config.clone());
    let res = client.list_tables().send().await.unwrap();
    assert_eq!(sleep_impl.total_duration(), expected_op1_sleep);
    assert_eq!(res.table_names(), expected_table_names.as_slice());
    assert_eq!(http_client.actual_requests().count(), 3);

    let client = aws_sdk_dynamodb::Client::from_conf(config.clone());
    let res = client.list_tables().send().await.unwrap();
    assert!(sleep_impl.total_duration() > expected_op2_sleep_min);
    assert!(sleep_impl.total_duration() < expected_op2_sleep_max);
    assert_eq!(res.table_names(), expected_table_names.as_slice());
    assert_eq!(http_client.actual_requests().count(), 5);
}

#[tokio::test]
async fn test_adaptive_retries_with_no_throttling_errors() {
    let base_config = RetryConfig::adaptive()
        .with_max_attempts(4)
        .with_use_static_exponential_base(true);

    // Legacy: 1s base backoff → 1+2=3s, 3+1=4s, 4+1+2+4=11s
    #[allow(deprecated)]
    adaptive_retries_no_throttling(
        BehaviorVersion::v2024_03_28(),
        base_config.clone(),
        "no_throttle_legacy",
        Duration::from_secs(3),
        Duration::from_secs(3 + 1),
        Duration::from_secs(3 + 1 + 7),
    )
    .await;

    // Retry 2.1: DynamoDB 25ms base backoff → 25+50=75ms, 75+25=100ms, 100+25+50+100=275ms
    adaptive_retries_no_throttling(
        BehaviorVersion::latest(),
        base_config.with_retry_spec(
            RetrySpec::v2_1().with_non_throttling_initial_backoff(Duration::from_millis(25)),
        ),
        "no_throttle_v2_1",
        Duration::from_millis(75),
        Duration::from_millis(100),
        Duration::from_millis(275),
    )
    .await;
}

#[tokio::test]
async fn test_adaptive_retries_with_throttling_errors() {
    let base_config = RetryConfig::adaptive()
        .with_max_attempts(4)
        .with_use_static_exponential_base(true);

    // Legacy: throttling uses 1s base, rate limiter dominates
    #[allow(deprecated)]
    adaptive_retries_with_throttling(
        BehaviorVersion::v2024_03_28(),
        base_config.clone(),
        "throttle_legacy",
        Duration::from_secs(38),
        Duration::from_secs(47),
        Duration::from_secs(49),
    )
    .await;

    // Retry 2.1: throttling still uses 1s base, rate limiter behavior unchanged
    adaptive_retries_with_throttling(
        BehaviorVersion::latest(),
        base_config.with_retry_spec(
            RetrySpec::v2_1().with_non_throttling_initial_backoff(Duration::from_millis(25)),
        ),
        "throttle_v2_1",
        Duration::from_secs(38),
        Duration::from_secs(47),
        Duration::from_secs(49),
    )
    .await;
}
