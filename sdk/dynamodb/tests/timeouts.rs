/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_dynamodb::config::{Credentials, Region, StalledStreamProtectionConfig};
use aws_sdk_dynamodb::error::SdkError;
use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep, Sleep};
use aws_smithy_http_client::test_util::NeverClient;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::SdkConfig;

#[derive(Debug, Clone)]
struct InstantSleep;
impl AsyncSleep for InstantSleep {
    fn sleep(&self, _duration: Duration) -> Sleep {
        Sleep::new(Box::pin(async move {}))
    }
}

#[tokio::test]
async fn api_call_timeout_retries() {
    let http_client = NeverClient::new();
    let conf = SdkConfig::builder()
        .region(Region::new("us-east-2"))
        .http_client(http_client.clone())
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .timeout_config(
            TimeoutConfig::builder()
                .operation_attempt_timeout(Duration::new(123, 0))
                .build(),
        )
        .retry_config(RetryConfig::standard())
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .sleep_impl(SharedAsyncSleep::new(InstantSleep))
        .build();
    let client = aws_sdk_dynamodb::Client::new(&conf);
    let resp = client
        .list_tables()
        .send()
        .await
        .expect_err("call should fail");
    assert_eq!(
        http_client.num_calls(),
        3,
        "client level timeouts should be retried"
    );
    assert!(
        matches!(resp, SdkError::TimeoutError { .. }),
        "expected a timeout error, got: {}",
        resp
    );
}

#[tokio::test]
async fn no_retries_on_operation_timeout() {
    let http_client = NeverClient::new();
    let conf = SdkConfig::builder()
        .region(Region::new("us-east-2"))
        .http_client(http_client.clone())
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::new(123, 0))
                .build(),
        )
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .retry_config(RetryConfig::standard())
        .sleep_impl(SharedAsyncSleep::new(InstantSleep))
        .build();
    let client = aws_sdk_dynamodb::Client::from_conf(aws_sdk_dynamodb::Config::new(&conf));
    let resp = client
        .list_tables()
        .send()
        .await
        .expect_err("call should fail");
    assert_eq!(
        http_client.num_calls(),
        1,
        "operation level timeouts should not be retried"
    );
    assert!(
        matches!(resp, SdkError::TimeoutError { .. }),
        "expected a timeout error, got: {}",
        resp
    );
}
