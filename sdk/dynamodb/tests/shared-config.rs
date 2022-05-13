/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::{Credentials, Region};
use http::Uri;

/// Iterative test of loading clients from shared configuration
#[tokio::test]
async fn shared_config_testbed() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .build();
    let conf = aws_sdk_dynamodb::config::Builder::from(&shared_config)
        .credentials_provider(Credentials::new("asdf", "asdf", None, None, "test"))
        .build();
    let (conn, request) = aws_smithy_client::test_connection::capture_request(None);
    let svc = aws_sdk_dynamodb::Client::from_conf_conn(conf, conn);
    let _ = svc.list_tables().send().await;
    assert_eq!(
        request.expect_request().uri(),
        &Uri::from_static("https://dynamodb.us-east-4.amazonaws.com")
    );
}
