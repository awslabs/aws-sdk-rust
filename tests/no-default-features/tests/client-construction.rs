/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::{Config, Credentials, SharedAsyncSleep, Sleep};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_smithy_async::rt::sleep::AsyncSleep;
use std::time::Duration;

// This will fail due to lack of a connector when constructing the SDK Config
// If this test doesn't panic, you may have accidentally unified features, resulting in
// the connector being enabled transitively
#[tokio::test]
#[should_panic(expected = "Enable the `rustls` crate feature or set a connector to fix this.")]
async fn test_clients_from_sdk_config() {
    aws_config::load_from_env().await;
}

// This will fail due to lack of a connector when constructing the service client
#[cfg(not(aws_sdk_middleware_mode))]
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
        .expect_err("it should fail to send a request because there is no connector");
    let msg = format!("{}", DisplayErrorContext(err));
    assert!(
        msg.contains("No HTTP connector was available to send this request. Enable the `rustls` crate feature or set a connector to fix this."),
        "expected '{msg}' to contain 'No HTTP connector was available to send this request. Enable the `rustls` crate feature or set a connector to fix this.'"
    );
}

// TODO(enableNewSmithyRuntimeMode): Remove this test (covered above for orchestrator)
//
// This will fail due to lack of a connector when constructing the service client
#[cfg(aws_sdk_middleware_mode)]
#[tokio::test]
#[should_panic(
    expected = "No HTTP connector was available. Enable the `rustls` crate feature or set a connector to fix this."
)]
async fn test_clients_from_service_config_middleware() {
    #[derive(Clone, Debug)]
    struct StubSleep;
    impl AsyncSleep for StubSleep {
        fn sleep(&self, _duration: Duration) -> Sleep {
            todo!()
        }
    }

    let config = Config::builder()
        .sleep_impl(SharedAsyncSleep::new(StubSleep {}))
        .build();
    // This will panic due to the lack of an HTTP connector
    aws_sdk_s3::Client::from_conf(config);
}
