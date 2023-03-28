/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use tokio_stream::StreamExt;

use aws_sdk_ec2::{config::Credentials, config::Region, types::InstanceType, Client, Config};
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_client::test_connection::TestConnection;

fn stub_config(conn: impl Into<HttpConnector>) -> Config {
    Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_connector(conn)
        .build()
}

/// See https://github.com/awslabs/aws-sdk-rust/issues/391
///
/// EC2 replies with `<nextToken></nextToken>` which our XML parser parses as empty string and not "none"
#[tokio::test]
async fn paginators_handle_empty_tokens() {
    let request= "Action=DescribeSpotPriceHistory&Version=2016-11-15&AvailabilityZone=eu-north-1a&InstanceType.1=g5.48xlarge&ProductDescription.1=Linux%2FUNIX";
    let response = r#"<?xml version="1.0" encoding="UTF-8"?>
        <DescribeSpotPriceHistoryResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
            <requestId>edf3e86c-4baf-47c1-9228-9a5ea09542e8</requestId>
            <spotPriceHistorySet/>
            <nextToken></nextToken>
        </DescribeSpotPriceHistoryResponse>"#;
    let conn = TestConnection::<&str>::new(vec![(
        http::Request::builder()
            .uri("https://ec2.us-east-1.amazonaws.com/")
            .body(request.into())
            .unwrap(),
        http::Response::builder()
            .status(200)
            .body(response)
            .unwrap(),
    )]);
    let client = Client::from_conf(stub_config(conn.clone()));
    let instance_type = InstanceType::from("g5.48xlarge");
    let mut paginator = client
        .describe_spot_price_history()
        .instance_types(instance_type)
        .product_descriptions("Linux/UNIX")
        .availability_zone("eu-north-1a")
        .into_paginator()
        .items()
        .send();
    let first_item = paginator.try_next().await.expect("success");
    assert_eq!(first_item, None);
    conn.assert_requests_match(&[]);
}

/// See https://github.com/awslabs/aws-sdk-rust/issues/405
///
/// EC2 can also reply with the token truly unset which will be interpreted as `None`
#[tokio::test]
async fn paginators_handle_unset_tokens() {
    let request= "Action=DescribeSpotPriceHistory&Version=2016-11-15&AvailabilityZone=eu-north-1a&InstanceType.1=g5.48xlarge&ProductDescription.1=Linux%2FUNIX";
    let response = r#"<?xml version="1.0" encoding="UTF-8"?>
        <DescribeSpotPriceHistoryResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
            <requestId>edf3e86c-4baf-47c1-9228-9a5ea09542e8</requestId>
            <spotPriceHistorySet/>
        </DescribeSpotPriceHistoryResponse>"#;
    let conn = TestConnection::<&str>::new(vec![(
        http::Request::builder()
            .uri("https://ec2.us-east-1.amazonaws.com/")
            .body(request.into())
            .unwrap(),
        http::Response::builder()
            .status(200)
            .body(response)
            .unwrap(),
    )]);
    let client = Client::from_conf(stub_config(conn.clone()));
    let instance_type = InstanceType::from("g5.48xlarge");
    let mut paginator = client
        .describe_spot_price_history()
        .instance_types(instance_type)
        .product_descriptions("Linux/UNIX")
        .availability_zone("eu-north-1a")
        .into_paginator()
        .items()
        .send();
    let first_item = paginator.try_next().await.expect("success");
    assert_eq!(first_item, None);
    conn.assert_requests_match(&[]);
}
