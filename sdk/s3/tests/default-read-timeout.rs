/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! S3's exemption from the per-service default read timeout, against the generated `aws-sdk-s3`
//! crate.
//!
//! S3 is fully exempt in the shared timeout-defaults artifact: a large `PutObject` cannot produce
//! response headers until the whole body has been sent, so any read bound would fire on normal
//! usage. The assertion runs at `latest()`, which is what makes it meaningful — it proves the
//! exemption holds rather than merely that the gate is closed.

use std::time::Duration;

use aws_sdk_s3::config::Region;
use aws_sdk_s3::{Client, Config};
use aws_smithy_http_client::test_util::NeverClient;
use aws_smithy_types::retry::RetryConfig;

#[tokio::test]
async fn s3_stays_exempt_at_latest() {
    use aws_smithy_runtime_api::client::http::{http_client_fn, HttpClient};
    use std::sync::{Arc, Mutex};

    let captured: Arc<Mutex<Option<Option<Duration>>>> = Default::default();
    let sink = captured.clone();
    let inner = NeverClient::new();
    let http_client = http_client_fn(move |settings, components| {
        *sink.lock().unwrap() = Some(settings.read_timeout());
        inner.http_connector(settings, components)
    });

    let config = Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-1"))
        .retry_config(RetryConfig::disabled())
        .http_client(http_client)
        .build();
    let client = Client::from_conf(config);

    // `NeverClient` never responds, so bound the call; the settings are captured before the
    // connector is ever polled.
    let _ = tokio::time::timeout(
        Duration::from_millis(300),
        client.get_object().bucket("test").key("test").send(),
    )
    .await;

    let seen = captured
        .lock()
        .unwrap()
        .take()
        .expect("the HTTP client must have been asked for a connector");

    assert_eq!(
        seen, None,
        "S3 is fully exempt in the artifact, so no default read timeout may be applied"
    );
}
