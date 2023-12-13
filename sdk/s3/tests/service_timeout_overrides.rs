/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;
use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::region::Region;
use aws_types::SdkConfig;
use std::time::Duration;
use tokio::time::Instant;

/// Use a 5 second operation timeout on SdkConfig and a 0ms connect timeout on the service config
#[tokio::test]
async fn timeouts_can_be_set_by_service() {
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::from_static("us-east-1"))
        .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::from_secs(5))
                .build(),
        )
        // ip that
        .endpoint_url(
            // Emulate a connect timeout error by hitting an unroutable IP
            "http://172.255.255.0:18104",
        )
        .build();
    let config = aws_sdk_s3::config::Builder::from(&sdk_config)
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(Duration::from_secs(0))
                .build(),
        )
        .build();
    let client = aws_sdk_s3::Client::from_conf(config);
    let start = Instant::now();
    let err = client
        .get_object()
        .key("foo")
        .bucket("bar")
        .send()
        .await
        .expect_err("unroutable IP should timeout");
    match err {
        SdkError::DispatchFailure(err) => assert!(err.is_timeout()),
        // if the connect timeout is not respected, this times out after 1 second because of the operation timeout with `SdkError::Timeout`
        _other => panic!("unexpected error: {:?}", _other),
    }
    // there should be a 0ms timeout, we gotta set some stuff up. Just want to make sure
    // it's shorter than the 5 second timeout if the test is broken
    assert!(start.elapsed() < Duration::from_millis(500));
}
