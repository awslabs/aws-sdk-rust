/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::{Credentials, Region};
use aws_smithy_http::endpoint::Endpoint;
use http::Uri;

/// Iterative test of loading clients from shared configuration
#[tokio::test]
async fn endpoints_can_be_overridden_globally() {
    let (conn, request) = aws_smithy_client::test_connection::capture_request(None);
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .http_connector(conn)
        .endpoint_resolver(Endpoint::immutable("http://localhost:8000").expect("valid endpoint"))
        .build();
    let conf = aws_sdk_dynamodb::config::Builder::from(&shared_config)
        .credentials_provider(Credentials::new("asdf", "asdf", None, None, "test"))
        .build();
    let svc = aws_sdk_dynamodb::Client::from_conf(conf);
    let _ = svc.list_tables().send().await;
    assert_eq!(
        request.expect_request().uri(),
        &Uri::from_static("http://localhost:8000")
    );
}

#[tokio::test]
async fn endpoints_can_be_overridden_locally() {
    let (conn, request) = aws_smithy_client::test_connection::capture_request(None);
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .http_connector(conn)
        .build();
    let conf = aws_sdk_dynamodb::config::Builder::from(&shared_config)
        .credentials_provider(Credentials::new("asdf", "asdf", None, None, "test"))
        .endpoint_resolver(Endpoint::immutable("http://localhost:8000").expect("valid endpoint"))
        .build();
    let svc = aws_sdk_dynamodb::Client::from_conf(conf);
    let _ = svc.list_tables().send().await;
    assert_eq!(
        request.expect_request().uri(),
        &Uri::from_static("http://localhost:8000")
    );
}
