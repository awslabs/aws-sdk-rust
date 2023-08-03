/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::retry::{ReconnectMode, RetryConfig};
use aws_sdk_s3::config::{Credentials, Region, SharedAsyncSleep};
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_smithy_client::test_connection::wire_mock::{
    check_matches, ReplayedEvent, WireLevelTestConnection,
};
use aws_smithy_client::{ev, match_events};

#[tokio::test]
async fn test_disable_reconnect_on_503() {
    let mock = WireLevelTestConnection::spinup(vec![
        ReplayedEvent::status(503),
        ReplayedEvent::status(503),
        ReplayedEvent::with_body("here-is-your-object"),
    ])
    .await;

    let config = aws_sdk_s3::Config::builder()
        .region(Region::from_static("us-east-2"))
        .credentials_provider(Credentials::for_tests())
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .endpoint_url(mock.endpoint_url())
        .http_connector(mock.http_connector())
        .retry_config(
            RetryConfig::standard().with_reconnect_mode(ReconnectMode::ReuseAllConnections),
        )
        .build();
    let client = aws_sdk_s3::Client::from_conf(config);
    let resp = client
        .get_object()
        .bucket("bucket")
        .key("key")
        .send()
        .await
        .expect("succeeds after retries");
    assert_eq!(
        resp.body.collect().await.unwrap().to_vec(),
        b"here-is-your-object"
    );
    match_events!(
        ev!(dns),
        ev!(connect),
        ev!(http(503)),
        ev!(http(503)),
        ev!(http(200))
    )(&mock.events());
}

#[tokio::test]
async fn test_enabling_reconnect_on_503() {
    let mock = WireLevelTestConnection::spinup(vec![
        ReplayedEvent::status(503),
        ReplayedEvent::status(503),
        ReplayedEvent::with_body("here-is-your-object"),
    ])
    .await;

    let config = aws_sdk_s3::Config::builder()
        .region(Region::from_static("us-east-2"))
        .credentials_provider(Credentials::for_tests())
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .endpoint_url(mock.endpoint_url())
        .http_connector(mock.http_connector())
        .retry_config(
            RetryConfig::standard().with_reconnect_mode(ReconnectMode::ReconnectOnTransientError),
        )
        .build();
    let client = aws_sdk_s3::Client::from_conf(config);
    let resp = client
        .get_object()
        .bucket("bucket")
        .key("key")
        .send()
        .await
        .expect("succeeds after retries");
    assert_eq!(
        resp.body.collect().await.unwrap().to_vec(),
        b"here-is-your-object"
    );
    match_events!(
        ev!(dns),
        ev!(connect),
        ev!(http(503)),
        ev!(dns),
        ev!(connect),
        ev!(http(503)),
        ev!(dns),
        ev!(connect),
        ev!(http(200))
    )(&mock.events());
}
