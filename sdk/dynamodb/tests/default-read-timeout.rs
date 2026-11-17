/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! The per-service default read timeout, against the generated `aws-sdk-dynamodb` crate.
//!
//! The codegen tests cover this with synthetic models. This covers the crate a customer actually
//! gets. DynamoDB is absent from the shared timeout-defaults artifact, so it takes the 5-minute
//! general default, and a caller pinned below the gate keeps the previous behavior.

use std::time::Duration;

use aws_sdk_dynamodb::config::{BehaviorVersion, Region};
use aws_smithy_http_client::test_util::NeverClient;
use aws_smithy_types::retry::RetryConfig;

/// Captures the `read_timeout` the orchestrator hands to the HTTP client for one request.
///
/// The read timeout is enforced inside the connector, so `HttpConnectorSettings` is where a
/// per-service default either arrives or silently does not.
async fn captured_read_timeout(behavior_version: Option<BehaviorVersion>) -> Option<Duration> {
    use aws_smithy_runtime_api::client::http::{http_client_fn, HttpClient};
    use std::sync::{Arc, Mutex};

    let captured: Arc<Mutex<Option<Option<Duration>>>> = Default::default();
    let sink = captured.clone();
    let inner = NeverClient::new();
    let http_client = http_client_fn(move |settings, components| {
        *sink.lock().unwrap() = Some(settings.read_timeout());
        inner.http_connector(settings, components)
    });

    let mut builder = aws_sdk_dynamodb::Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-2"))
        .http_client(http_client)
        .retry_config(RetryConfig::disabled());
    if let Some(bv) = behavior_version {
        builder = builder.behavior_version(bv);
    }
    let conf = builder.build();
    let client = aws_sdk_dynamodb::Client::from_conf(conf);

    // `NeverClient` never responds, so bound the call; the settings are captured before the
    // connector is ever polled.
    let _ = tokio::time::timeout(Duration::from_millis(300), client.list_tables().send()).await;

    let seen = captured.lock().unwrap().take();
    seen.expect("the HTTP client must have been asked for a connector")
}

/// At `latest()` an unlisted service takes the 5-minute general default.
#[tokio::test]
async fn default_is_five_minutes_at_latest() {
    assert_eq!(
        captured_read_timeout(None).await,
        Some(Duration::from_secs(300)),
        "an unlisted service takes the 5-minute general default"
    );
}

/// Pinning below the gate keeps the previous behavior.
#[tokio::test]
#[allow(deprecated)]
async fn absent_below_the_gate() {
    assert_eq!(
        captured_read_timeout(Some(BehaviorVersion::v2026_11_10())).await,
        None,
        "a caller pinned below the gate must get no default read timeout"
    );
}
