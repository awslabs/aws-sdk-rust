/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_apigateway::config::{Credentials, Region};
use aws_sdk_apigateway::{Client, Config};
use aws_smithy_client::test_connection::capture_request;
use aws_smithy_protocol_test::{assert_ok, validate_headers};

#[tokio::test]
async fn accept_header_is_application_json() {
    let (conn, handler) = capture_request(None);
    let conf = Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_connector(conn)
        .build();

    let client = Client::from_conf(conf);
    let _result = client
        .delete_resource()
        .rest_api_id("some-rest-api-id")
        .resource_id("some-resource-id")
        .send()
        .await;
    let request = handler.expect_request();
    assert_ok(validate_headers(
        request.headers(),
        [("accept", "application/json")],
    ));
}
