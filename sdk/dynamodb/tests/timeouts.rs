/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::types::SdkError;
use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
use aws_smithy_client::never::NeverConnector;
use aws_smithy_types::timeout;
use aws_smithy_types::timeout::Api;
use aws_smithy_types::tristate::TriState;
use aws_types::credentials::SharedCredentialsProvider;
use aws_types::region::Region;
use aws_types::{Credentials, SdkConfig};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
struct InstantSleep;
impl AsyncSleep for InstantSleep {
    fn sleep(&self, _duration: Duration) -> Sleep {
        Sleep::new(Box::pin(async move {}))
    }
}

#[tokio::test]
async fn api_call_timeout_retries() {
    let conn = NeverConnector::new();
    let conf = SdkConfig::builder()
        .region(Region::new("us-east-2"))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "stub", "stub", None, None, "test",
        )))
        .timeout_config(timeout::Config::new().with_api_timeouts(
            Api::new().with_call_attempt_timeout(TriState::Set(Duration::new(123, 0))),
        ))
        .sleep_impl(Arc::new(InstantSleep))
        .build();
    let client = aws_sdk_dynamodb::Client::from_conf_conn(
        aws_sdk_dynamodb::Config::new(&conf),
        conn.clone(),
    );
    let resp = client
        .list_tables()
        .send()
        .await
        .expect_err("call should fail");
    assert_eq!(
        conn.num_calls(),
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
    let conn = NeverConnector::new();
    let conf =
        SdkConfig::builder()
            .region(Region::new("us-east-2"))
            .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
                "stub", "stub", None, None, "test",
            )))
            .timeout_config(timeout::Config::new().with_api_timeouts(
                Api::new().with_call_timeout(TriState::Set(Duration::new(123, 0))),
            ))
            .sleep_impl(Arc::new(InstantSleep))
            .build();
    let client = aws_sdk_dynamodb::Client::from_conf_conn(
        aws_sdk_dynamodb::Config::new(&conf),
        conn.clone(),
    );
    let resp = client
        .list_tables()
        .send()
        .await
        .expect_err("call should fail");
    assert_eq!(
        conn.num_calls(),
        1,
        "operation level timeouts should not be retried"
    );
    assert!(
        matches!(resp, SdkError::TimeoutError { .. }),
        "expected a timeout error, got: {}",
        resp
    );
}
