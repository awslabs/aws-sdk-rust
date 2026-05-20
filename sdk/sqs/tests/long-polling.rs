/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_sqs::config::{SharedAsyncSleep, StalledStreamProtectionConfig};
use aws_sdk_sqs::{Client, Config};
use aws_smithy_async::test_util::instant_time_and_sleep;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_runtime::client::retries::{RetryPartition, TokenBucket};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::retry::{RetryConfig, RetrySpec};
use std::time::{Duration, SystemTime};

fn req() -> http_1x::Request<SdkBody> {
    http_1x::Request::builder()
        .body(SdkBody::from("request body"))
        .unwrap()
}

fn transient_err() -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(500)
        .body(SdkBody::from(
            "{\"__type\":\"InternalError\",\"message\":\"Internal error\"}",
        ))
        .unwrap()
}

fn throttling_err() -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(400)
        .body(SdkBody::from(
            "{\"__type\":\"Throttling\",\"message\":\"Rate exceeded\"}",
        ))
        .unwrap()
}

#[derive(Debug)]
struct StaticBackoffInterceptor;

impl Intercept for StaticBackoffInterceptor {
    fn name(&self) -> &'static str {
        "StaticBackoffInterceptor"
    }

    fn modify_before_retry_loop(
        &self,
        _context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if let Some(rc) = cfg.load::<RetryConfig>().cloned() {
            cfg.interceptor_state()
                .store_put(rc.with_use_static_exponential_base(true));
        }
        Ok(())
    }
}

/// Long-Polling Backoff After Transient Error When Token Bucket Empty
/// Given: service=sqs, long_polling=true, initial_retry_tokens=0, exponential_base=1
/// Expected: outcome=retry_quota_exceeded, delay=0.05
#[tokio::test]
async fn long_polling_backoff_after_transient_error_when_token_bucket_empty() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(req(), transient_err())]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_retry_spec(RetrySpec::v2_1()))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(
            RetryPartition::custom("sqs_long_poll_transient")
                .token_bucket(TokenBucket::new(0))
                .build(),
        )
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _err = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .send()
        .await
        .expect_err("should fail — no tokens to retry");

    assert_eq!(http_client.actual_requests().count(), 1);
    assert_eq!(sleep_impl.total_duration(), Duration::from_millis(50));
}

/// Long-Polling Backoff After Throttling Error When Token Bucket Empty
/// Given: service=sqs, long_polling=true, initial_retry_tokens=0, exponential_base=1
/// Expected: outcome=retry_quota_exceeded, delay=1
#[tokio::test]
async fn long_polling_backoff_after_throttling_error_when_token_bucket_empty() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(req(), throttling_err())]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_retry_spec(RetrySpec::v2_1()))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(
            RetryPartition::custom("sqs_long_poll_throttle")
                .token_bucket(TokenBucket::new(0))
                .build(),
        )
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _err = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .send()
        .await
        .expect_err("should fail — no tokens to retry");

    assert_eq!(http_client.actual_requests().count(), 1);
    assert_eq!(sleep_impl.total_duration(), Duration::from_secs(1));
}

/// Long-Polling Max Attempts Exceeded Must NOT Delay
/// Given: service=sqs, long_polling=true, max_attempts=2, exponential_base=1
#[tokio::test]
async fn long_polling_max_attempts_exceeded_must_not_delay() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), transient_err()),
        ReplayEvent::new(req(), transient_err()),
    ]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_retry_spec(RetrySpec::v2_1()))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new("sqs_max_attempts"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _err = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .customize()
        .config_override(
            Config::builder().retry_config(RetryConfig::standard().with_max_attempts(2)),
        )
        .send()
        .await
        .expect_err("should fail — max attempts exceeded");

    assert_eq!(http_client.actual_requests().count(), 2);
    // Only the retry backoff (50ms), no extra delay for max_attempts_exceeded
    assert_eq!(sleep_impl.total_duration(), Duration::from_millis(50));
}

/// Long-Polling Success Must NOT Delay
/// Given: service=sqs, long_polling=true, max_attempts=2, exponential_base=1
#[tokio::test]
async fn long_polling_success_must_not_delay() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), transient_err()),
        ReplayEvent::new(
            req(),
            http_1x::Response::builder()
                .status(200)
                .header("content-type", "application/x-amz-json-1.0")
                .body(SdkBody::from("{\"Messages\":[]}"))
                .unwrap(),
        ),
    ]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_retry_spec(RetrySpec::v2_1()))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new("sqs_success"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _res = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .customize()
        .config_override(
            Config::builder().retry_config(RetryConfig::standard().with_max_attempts(2)),
        )
        .send()
        .await
        .expect("should succeed on retry");

    assert_eq!(http_client.actual_requests().count(), 2);
    // Only the retry backoff (50ms), no extra delay for success
    assert_eq!(sleep_impl.total_duration(), Duration::from_millis(50));
}

/// Long-Polling Non-Retriable Errors Must NOT Delay
/// Given: service=sqs, long_polling=true, max_attempts=2, exponential_base=1
#[tokio::test]
async fn long_polling_non_retryable_errors_must_not_delay() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        req(),
        http_1x::Response::builder()
            .status(404)
            .body(SdkBody::from(
                "{\"__type\":\"QueueDoesNotExist\",\"message\":\"not found\"}",
            ))
            .unwrap(),
    )]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_retry_spec(RetrySpec::v2_1()))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new("sqs_non_retryable"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _err = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .customize()
        .config_override(
            Config::builder().retry_config(RetryConfig::standard().with_max_attempts(2)),
        )
        .send()
        .await
        .expect_err("should fail — non-retryable");

    assert_eq!(http_client.actual_requests().count(), 1);
    assert_eq!(
        sleep_impl.total_duration(),
        Duration::ZERO,
        "non-retryable errors must not delay"
    );
}
