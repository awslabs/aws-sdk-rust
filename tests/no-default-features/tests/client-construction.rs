/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::IdentityCache;

use aws_sdk_s3::config::{
    retry::RetryConfig, timeout::TimeoutConfig, BehaviorVersion, Config, Credentials, Region,
    SharedAsyncSleep, Sleep, StalledStreamProtectionConfig,
};
use aws_sdk_s3::primitives::SdkBody;
use aws_smithy_http_client::test_util::infallible_client_fn;

use aws_sdk_s3::error::DisplayErrorContext;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_http_client::test_util::capture_request;
use aws_smithy_runtime::test_util::capture_test_logs::capture_test_logs;
use std::time::Duration;

// This will fail due to lack of a connector when constructing the SDK Config
// If this test doesn't panic, you may have accidentally unified features, resulting in
// the connector being enabled transitively
#[tokio::test]
#[should_panic(
    expected = "Enable the `default-https-client` crate feature or configure an HTTP client to fix this."
)]
async fn test_clients_from_sdk_config() {
    aws_config::load_defaults(BehaviorVersion::latest()).await;
}

// This will fail due to lack of a connector when constructing the service client
#[tokio::test]
async fn test_clients_from_service_config() {
    use aws_sdk_s3::config::Region;

    #[derive(Clone, Debug)]
    struct StubSleep;
    impl AsyncSleep for StubSleep {
        fn sleep(&self, _duration: Duration) -> Sleep {
            Sleep::new(Box::pin(async { /* no-op */ }))
        }
    }

    let config = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .sleep_impl(SharedAsyncSleep::new(StubSleep))
        .behavior_version(BehaviorVersion::latest())
        .build();
    // Creating the client shouldn't panic or error since presigning doesn't require a connector
    let client = aws_sdk_s3::Client::from_conf(config);

    let err = client
        .list_buckets()
        .send()
        .await
        .expect_err("it should fail to send a request because there is no HTTP client");
    let msg = format!("{}", DisplayErrorContext(err));
    assert!(
        msg.contains("No HTTP client was available to send this request. Enable the `default-https-client` crate feature or configure an HTTP client to fix this."),
        "expected '{msg}' to contain 'No HTTP client was available to send this request. Enable the `default-https-client` crate feature or set an HTTP client to fix this.'"
    );
}

#[tokio::test]
#[should_panic(expected = "Invalid client configuration: A behavior major version must be set")]
async fn test_missing_behavior_version() {
    use aws_sdk_s3::config::Region;
    let http_client =
        infallible_client_fn(|_req| http_1x::Response::builder().body(SdkBody::empty()).unwrap());

    let config = Config::builder()
        .region(Region::new("us-east-1"))
        .identity_cache(IdentityCache::no_cache())
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .build();
    // This line panics
    let _client = aws_sdk_s3::Client::from_conf(config);
}

#[tokio::test]
#[should_panic(
    expected = "Invalid client configuration: An async sleep implementation is required for retry to work."
)]
async fn test_missing_async_sleep_time_source_retries() {
    let _logs = capture_test_logs();
    let (http_client, _) = capture_request(None);

    // Configure retry and timeouts without providing a sleep impl
    let config = Config::builder()
        .http_client(http_client)
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .retry_config(RetryConfig::standard())
        .timeout_config(TimeoutConfig::disabled())
        .behavior_version(BehaviorVersion::latest())
        .build();

    // should panic with a validation error
    let _client = aws_sdk_s3::Client::from_conf(config);
}

#[tokio::test]
#[should_panic(
    expected = "Invalid client configuration: An async sleep implementation is required for timeouts to work."
)]
async fn test_missing_async_sleep_time_source_timeouts() {
    let _logs = capture_test_logs();
    let (http_client, _) = capture_request(None);

    // Configure retry and timeouts without providing a sleep impl
    let config = Config::builder()
        .http_client(http_client)
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .retry_config(RetryConfig::disabled())
        .behavior_version(BehaviorVersion::latest())
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::from_secs(5))
                .build(),
        )
        .build();

    // should panic with a validation error
    let _client = aws_sdk_s3::Client::from_conf(config);
}

#[tokio::test]
#[should_panic(
    expected = "Invalid client configuration: Lazy identity caching requires an async sleep implementation to be configured."
)]
async fn test_time_source_for_identity_cache() {
    let _logs = capture_test_logs();
    let (http_client, _) = capture_request(None);

    // Configure an identity cache without providing a sleep impl or time source
    let config = Config::builder()
        .http_client(http_client)
        .region(Region::new("us-east-1"))
        .identity_cache(IdentityCache::lazy().build())
        .credentials_provider(Credentials::for_tests())
        .retry_config(RetryConfig::disabled())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .timeout_config(TimeoutConfig::disabled())
        .behavior_version(BehaviorVersion::latest())
        .build();

    // should panic with a validation error
    let _client = aws_sdk_s3::Client::from_conf(config);
}

#[allow(deprecated)] // intentionally testing an old behavior version
#[tokio::test]
async fn behavior_mv_from_aws_config() {
    let (http_client, req) = capture_request(None);
    let cfg = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .http_client(http_client)
        .retry_config(RetryConfig::disabled())
        .credentials_provider(Credentials::for_tests())
        .identity_cache(IdentityCache::no_cache())
        .timeout_config(TimeoutConfig::disabled())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .region(Region::new("us-west-2"))
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&cfg);
    let _err = s3_client
        .list_buckets()
        .send()
        .await
        .expect_err("it should fail to send a request because there is no HTTP client");
    assert!(req
        .expect_request()
        .uri()
        .starts_with("https://s3.us-west-2.amazonaws.com/"));
}

#[allow(deprecated)] // intentionally testing an old behavior version
#[tokio::test]
async fn behavior_mv_from_client_construction() {
    let (http_client, req) = capture_request(None);
    let cfg = aws_config::SdkConfig::builder()
        .http_client(http_client)
        .retry_config(RetryConfig::disabled())
        .identity_cache(IdentityCache::no_cache())
        .timeout_config(TimeoutConfig::disabled())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .region(Region::new("us-west-2"))
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(
        aws_sdk_s3::config::Builder::from(&cfg)
            .credentials_provider(Credentials::for_tests())
            .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2023_11_09())
            .build(),
    );
    let _err = dbg!(s3_client
        .list_buckets()
        .send()
        .await
        .expect_err("it should fail to send a request because there is no HTTP client"));
    assert!(req
        .expect_request()
        .uri()
        .starts_with("https://s3.us-west-2.amazonaws.com/"));
}
