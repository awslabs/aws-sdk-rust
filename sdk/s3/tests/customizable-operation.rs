/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::{Client, Config};
use aws_smithy_runtime::client::http::test_util::capture_request;
use http::HeaderValue;

#[tokio::test]
async fn test_s3_ops_are_customizable() {
    let (http_client, rcvr) = capture_request(None);
    let config = Config::builder()
        .credentials_provider(SharedCredentialsProvider::new(
            Credentials::for_tests_with_session_token(),
        ))
        .region(Region::new("us-east-1"))
        .http_client(http_client.clone())
        .build();

    let client = Client::from_conf(config);

    // The response from the fake connection won't return the expected XML but we don't care about
    // that error in this test
    let _ = client
        .list_buckets()
        .customize()
        .mutate_request(|req| {
            req.headers_mut()
                .append("test-header", HeaderValue::from_static("test-value"));
        })
        .send()
        .await
        .expect_err("this will fail due to not receiving a proper XML response.");

    let expected_req = rcvr.expect_request();
    let test_header = expected_req
        .headers()
        .get("test-header")
        .unwrap()
        .to_owned();

    assert_eq!("test-value", test_header);
}
