/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::time::Duration;

use aws_credential_types::Credentials;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::Client;
use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
use aws_smithy_http_client::test_util::wire::{ev, match_events, ReplayedEvent, WireMockServer};
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::region::Region;
use bytes::Bytes;

const DYNAMO_THROTTLING_RESPONSE: &str = r#"{"__type":"com.amazonaws.dynamodb.v20120810#ThrottlingException",
"message":"enhance your calm"}"#;

const DYNAMODB_DB_SUCCESS_RESPONSE: &str = r#"{"Count":0,"Items":[],"ScannedCount":2}"#;

#[tokio::test]
async fn test_no_reconnect_500_throttling() {
    assert_error_not_transient(ReplayedEvent::HttpResponse {
        status: 500,
        body: Bytes::from(DYNAMO_THROTTLING_RESPONSE),
    })
    .await
}

#[tokio::test]
async fn test_no_reconnect_429_throttling() {
    assert_error_not_transient(ReplayedEvent::HttpResponse {
        status: 429,
        body: Bytes::from(DYNAMO_THROTTLING_RESPONSE),
    })
    .await
}

async fn assert_error_not_transient(error: ReplayedEvent) {
    let mock = WireMockServer::start(vec![
        error,
        ReplayedEvent::with_body(DYNAMODB_DB_SUCCESS_RESPONSE),
    ])
    .await;

    let config = aws_sdk_dynamodb::Config::builder()
        .region(Region::from_static("us-east-2"))
        .credentials_provider(Credentials::for_tests())
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .endpoint_url(mock.endpoint_url())
        .http_client(mock.http_client())
        .timeout_config(
            TimeoutConfig::builder()
                .operation_attempt_timeout(Duration::from_millis(100))
                .build(),
        )
        .retry_config(RetryConfig::standard())
        .build();
    let client = Client::from_conf(config);
    let _item = client
        .get_item()
        .table_name("arn:aws:dynamodb:us-east-2:333333333333:table/table_name")
        .key("foo", AttributeValue::Bool(true))
        .send()
        .await
        .expect("should succeed");
    match_events!(ev!(dns), ev!(connect), _, ev!(http(200)))(&mock.events());
}
