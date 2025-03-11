/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_runtime::user_agent::test_util::assert_ua_contains_metric_values;
use aws_sdk_ec2::{config::Credentials, config::Region, types::InstanceType, Client, Config};
use aws_smithy_http_client::test_util::capture_request;
use aws_smithy_runtime_api::client::http::HttpClient;
use aws_smithy_types::body::SdkBody;
use std::collections::HashSet;

fn stub_config(http_client: impl HttpClient + 'static) -> Config {
    Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .build()
}

fn validate_query_string(expected_str: &str, actual_str: &str) {
    assert_eq!(expected_str.len(), actual_str.len());
    let expected = expected_str.split('&').collect::<HashSet<_>>();
    let actual = actual_str.split('&').collect::<HashSet<_>>();
    assert_eq!(expected, actual);
}

/// See https://github.com/awslabs/aws-sdk-rust/issues/391
///
/// EC2 replies with `<nextToken></nextToken>` which our XML parser parses as empty string and not "none"
#[tokio::test]
async fn paginators_handle_empty_tokens() {
    let response = r#"<?xml version="1.0" encoding="UTF-8"?>
        <DescribeSpotPriceHistoryResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
            <requestId>edf3e86c-4baf-47c1-9228-9a5ea09542e8</requestId>
            <spotPriceHistorySet/>
            <nextToken></nextToken>
        </DescribeSpotPriceHistoryResponse>"#;
    let response = http_1x::Response::builder()
        .status(200)
        .body(SdkBody::from(response))
        .unwrap();
    let (http_client, captured_request) = capture_request(Some(response));
    let client = Client::from_conf(stub_config(http_client.clone()));
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
    let req = captured_request.expect_request();
    let actual_body = std::str::from_utf8(req.body().bytes().unwrap()).unwrap();
    let expected_body = "Action=DescribeSpotPriceHistory&Version=2016-11-15&AvailabilityZone=eu-north-1a&InstanceType.1=g5.48xlarge&ProductDescription.1=Linux%2FUNIX";
    validate_query_string(expected_body, actual_body);
}

/// See https://github.com/awslabs/aws-sdk-rust/issues/405
///
/// EC2 can also reply with the token truly unset which will be interpreted as `None`
#[tokio::test]
async fn paginators_handle_unset_tokens() {
    let response = r#"<?xml version="1.0" encoding="UTF-8"?>
        <DescribeSpotPriceHistoryResponse xmlns="http://ec2.amazonaws.com/doc/2016-11-15/">
            <requestId>edf3e86c-4baf-47c1-9228-9a5ea09542e8</requestId>
            <spotPriceHistorySet/>
        </DescribeSpotPriceHistoryResponse>"#;
    let response = http_1x::Response::builder()
        .status(200)
        .body(SdkBody::from(response))
        .unwrap();
    let (http_client, captured_request) = capture_request(Some(response));
    let client = Client::from_conf(stub_config(http_client.clone()));
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
    let req = captured_request.expect_request();
    let actual_body = std::str::from_utf8(req.body().bytes().unwrap()).unwrap();
    let expected_body = "Action=DescribeSpotPriceHistory&Version=2016-11-15&AvailabilityZone=eu-north-1a&InstanceType.1=g5.48xlarge&ProductDescription.1=Linux%2FUNIX";
    validate_query_string(expected_body, actual_body);
}

#[tokio::test]
async fn should_emit_business_metric_for_paginator_in_user_agent() {
    let (http_client, captured_request) = capture_request(None);
    let client = Client::from_conf(stub_config(http_client.clone()));
    let instance_type = InstanceType::from("g5.48xlarge");
    let _ = client
        .describe_spot_price_history()
        .instance_types(instance_type)
        .product_descriptions("Linux/UNIX")
        .availability_zone("eu-north-1a")
        .into_paginator()
        .items()
        .send()
        .collect::<Vec<_>>()
        .await;
    let expected_req = captured_request.expect_request();
    let user_agent = expected_req.headers().get("x-amz-user-agent").unwrap();
    assert_ua_contains_metric_values(user_agent, &["C"]);
}
