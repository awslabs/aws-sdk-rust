/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::config::{Credentials, Region, StalledStreamProtectionConfig};
use aws_smithy_http_client::test_util::capture_request;
use http_1x::Uri;

/// Iterative test of loading clients from shared configuration
#[tokio::test]
async fn shared_config_testbed() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .build();
    let (http_client, request) = capture_request(None);
    let conf = aws_sdk_dynamodb::config::Builder::from(&shared_config)
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .endpoint_url("http://localhost:8000")
        .build();
    let svc = aws_sdk_dynamodb::Client::from_conf(conf);
    let _ = svc.list_tables().send().await;
    assert_eq!(
        request.expect_request().uri(),
        &Uri::from_static("http://localhost:8000")
    );
}
