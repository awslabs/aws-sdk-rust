/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_sqs::config::{SharedAsyncSleep, StalledStreamProtectionConfig};
use aws_sdk_sqs::{Client, Config};
use aws_smithy_async::test_util::instant_time_and_sleep;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_runtime::client::retries::RetryPartition;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::retry::RetryConfig;
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

fn ok() -> http_1x::Response<SdkBody> {
    http_1x::Response::builder()
        .status(200)
        .body(SdkBody::from("{}"))
        .unwrap()
}

// Forces a static exponential base so backoff is deterministic.
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

// A customer who sets RetryConfig::standard() leaves the retry spec unset, so the latest client
// fills it with Retry 2.1 from the default layer: non-throttling backoff is 50ms, not the legacy 1s.
#[tokio::test]
async fn explicit_standard_gets_2_1_backoff() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), transient_err()),
        ReplayEvent::new(req(), transient_err()),
        ReplayEvent::new(req(), ok()),
    ]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new("sqs_explicit_standard"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _ = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .send()
        .await;

    assert_eq!(http_client.actual_requests().count(), 3);
    // 50ms * 2^0 + 50ms * 2^1 = 150ms
    assert_eq!(sleep_impl.total_duration(), Duration::from_millis(150));
}

// Setting only max_attempts keeps the 2.1 backoff for the one retry it allows.
#[tokio::test]
async fn explicit_max_attempts_gets_2_1_backoff() {
    let (time_source, sleep_impl) = instant_time_and_sleep(SystemTime::UNIX_EPOCH);
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(req(), transient_err()),
        ReplayEvent::new(req(), ok()),
    ]);

    let config = Config::builder()
        .with_test_defaults_v2()
        .retry_config(RetryConfig::standard().with_max_attempts(2))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .interceptor(StaticBackoffInterceptor)
        .time_source(SharedTimeSource::new(time_source))
        .sleep_impl(SharedAsyncSleep::new(sleep_impl.clone()))
        .retry_partition(RetryPartition::new("sqs_explicit_max_attempts"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);
    let _ = client
        .receive_message()
        .queue_url("https://sqs.us-east-1.amazonaws.com/123456789012/test-queue")
        .send()
        .await;

    assert_eq!(http_client.actual_requests().count(), 2);
    // one retry at the 2.1 50ms backoff
    assert_eq!(sleep_impl.total_duration(), Duration::from_millis(50));
}
