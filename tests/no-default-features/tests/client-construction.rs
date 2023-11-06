/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::IdentityCache;
use aws_sdk_s3::config::{
    retry::RetryConfig, timeout::TimeoutConfig, Config, Credentials, Region, SharedAsyncSleep,
    Sleep,
};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_runtime::client::http::test_util::capture_request;
use aws_smithy_runtime::test_util::capture_test_logs::capture_test_logs;
use std::time::Duration;

// This will fail due to lack of a connector when constructing the SDK Config
// If this test doesn't panic, you may have accidentally unified features, resulting in
// the connector being enabled transitively
#[tokio::test]
#[should_panic(
    expected = "Enable the `rustls` crate feature or configure a HTTP client to fix this."
)]
async fn test_clients_from_sdk_config() {
    aws_config::load_from_env().await;
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
        msg.contains("No HTTP client was available to send this request. Enable the `rustls` crate feature or configure a HTTP client to fix this."),
        "expected '{msg}' to contain 'No HTTP client was available to send this request. Enable the `rustls` crate feature or set a HTTP client to fix this.'"
    );
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
        .timeout_config(TimeoutConfig::disabled())
        .build();

    // should panic with a validation error
    let _client = aws_sdk_s3::Client::from_conf(config);
}
