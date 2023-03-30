/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_iam::config::{Credentials, Region};
use aws_smithy_client::test_connection::capture_request;

// this test is ignored because pseudoregions have been removed. This test should be re-enabled
// once FIPS support is added in aws-config
#[tokio::test]
#[ignore]
async fn correct_endpoint_resolver() {
    let (conn, request) = capture_request(None);
    let conf = aws_sdk_iam::Config::builder()
        .region(Region::from_static("iam-fips"))
        .credentials_provider(Credentials::for_tests())
        .http_connector(conn)
        .build();
    let client = aws_sdk_iam::Client::from_conf(conf);
    let _ = client.list_roles().send().await;
    let req = request.expect_request();
    assert_eq!(&req.uri().to_string(), "https://iam-fips.amazonaws.com/");
}
