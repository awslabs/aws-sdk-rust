/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_dynamodb::config::{self, Credentials, Region};
use aws_types::SdkConfig;
use http::Uri;

#[track_caller]
async fn expect_uri(
    conf: SdkConfig,
    uri: &'static str,
    customize: fn(config::Builder) -> config::Builder,
) {
    let (conn, request) = aws_smithy_client::test_connection::capture_request(None);
    let conf = customize(
        aws_sdk_dynamodb::config::Builder::from(&conf)
            .credentials_provider(Credentials::for_tests())
            .http_connector(conn),
    )
    .build();
    let svc = aws_sdk_dynamodb::Client::from_conf(conf);
    // dbg to see why the request failed if this test is failing
    let _ = dbg!(svc.list_tables().send().await);
    assert_eq!(request.expect_request().uri(), &Uri::from_static(uri));
}

/// Integration test of loading clients from shared configuration
#[tokio::test]
async fn endpoints_can_be_overridden_globally() {
    let conf = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .endpoint_url("http://localhost:8000")
        .build();

    expect_uri(conf, "http://localhost:8000", |b| b).await;
}

#[tokio::test]
async fn endpoints_can_be_overridden_locally() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .build();

    expect_uri(shared_config, "http://localhost:8000", |b| {
        b.endpoint_url("http://localhost:8000")
    })
    .await;
}

#[tokio::test]
async fn dual_stack_endpoints() {
    let shared_config = SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .use_dual_stack(true)
        .build();
    expect_uri(shared_config, "https://dynamodb.us-east-4.api.aws/", |b| b).await
}

#[tokio::test]
async fn dual_stack_disabled_locally_endpoints() {
    let shared_config = SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .use_dual_stack(true)
        .build();
    expect_uri(
        shared_config,
        "https://dynamodb.us-east-4.amazonaws.com/",
        |b| b.use_dual_stack(false),
    )
    .await
}

#[tokio::test]
async fn fips_endpoints() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .use_dual_stack(true)
        .use_fips(true)
        .build();

    expect_uri(
        shared_config,
        "https://dynamodb-fips.us-east-4.api.aws/",
        |b| b,
    )
    .await;
}
