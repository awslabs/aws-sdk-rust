/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// This will fail due to lack of a connector when constructing the SDK Config
#[tokio::test]
#[should_panic(
    expected = "No HTTP connector was available. Enable the `rustls` or `native-tls` crate feature or set a connector to fix this."
)]
async fn test_clients_from_sdk_config() {
    aws_config::load_from_env().await;
}

// This will fail due to lack of a connector when constructing the service client
#[tokio::test]
#[should_panic(
    expected = "No HTTP connector was available. Enable the `rustls` or `native-tls` crate feature or set a connector to fix this."
)]
async fn test_clients_from_service_config() {
    #[derive(Clone, Debug)]
    struct StubSleep;
    impl aws_smithy_async::rt::sleep::AsyncSleep for StubSleep {
        fn sleep(&self, _duration: std::time::Duration) -> aws_sdk_s3::config::Sleep {
            todo!()
        }
    }

    let config = aws_sdk_s3::Config::builder()
        .sleep_impl(std::sync::Arc::new(StubSleep {}))
        .build();
    // This will panic due to the lack of an HTTP connector
    aws_sdk_s3::Client::from_conf(config);
}
