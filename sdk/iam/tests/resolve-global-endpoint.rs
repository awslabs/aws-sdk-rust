/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_iam::config::{Credentials, Region};
use aws_smithy_http_client::test_util::capture_request;

#[tokio::test]
async fn correct_endpoint_resolver() {
    let (http_client, request) = capture_request(None);
    let conf = aws_sdk_iam::Config::builder()
        .credentials_provider(Credentials::for_tests())
        .use_fips(true)
        .region(Region::new("us-east-1"))
        .http_client(http_client)
        .build();
    let client = aws_sdk_iam::Client::from_conf(conf);
    let _ = dbg!(client.list_roles().send().await);
    let req = request.expect_request();
    assert_eq!(&req.uri().to_string(), "https://iam-fips.amazonaws.com/");
}
