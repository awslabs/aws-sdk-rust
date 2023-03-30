/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_smithy_async::rt::sleep::TokioSleep;
use aws_smithy_client::test_connection::wire_mock::{
    check_matches, ReplayedEvent, WireLevelTestConnection,
};
use aws_smithy_client::{ev, match_events};
use aws_smithy_types::retry::{ReconnectMode, RetryConfig};
use aws_types::region::Region;
use aws_types::SdkConfig;
use std::sync::Arc;

#[tokio::test]
/// test that disabling reconnects on retry config disables them for the client
async fn disable_reconnects() {
    let mock = WireLevelTestConnection::spinup(vec![
        ReplayedEvent::status(503),
        ReplayedEvent::status(503),
        ReplayedEvent::with_body("here-is-your-object"),
    ])
    .await;

    let sdk_config = SdkConfig::builder()
        .region(Region::from_static("us-east-2"))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .sleep_impl(Arc::new(TokioSleep::new()))
        .endpoint_url(mock.endpoint_url())
        .http_connector(mock.http_connector())
        .retry_config(
            RetryConfig::standard().with_reconnect_mode(ReconnectMode::ReuseAllConnections),
        )
        .build();
    let client = aws_sdk_s3::Client::new(&sdk_config);
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
async fn reconnect_on_503() {
    let mock = WireLevelTestConnection::spinup(vec![
        ReplayedEvent::status(503),
        ReplayedEvent::status(503),
        ReplayedEvent::with_body("here-is-your-object"),
    ])
    .await;

    let sdk_config = SdkConfig::builder()
        .region(Region::from_static("us-east-2"))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .sleep_impl(Arc::new(TokioSleep::new()))
        .endpoint_url(mock.endpoint_url())
        .http_connector(mock.http_connector())
        .retry_config(RetryConfig::standard())
        .build();
    let client = aws_sdk_s3::Client::new(&sdk_config);
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
