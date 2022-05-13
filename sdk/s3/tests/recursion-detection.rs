/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::{Credentials, Region};
use aws_smithy_client::test_connection::capture_request;
use http::HeaderValue;

#[tokio::test]
async fn recursion_detection_applied() {
    std::env::set_var("AWS_LAMBDA_FUNCTION_NAME", "some-function");
    std::env::set_var("_X_AMZN_TRACE_ID", "traceid");
    let (conn, captured_request) = capture_request(None);

    let creds = Credentials::new(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
        None,
        "test",
    );
    let conf = aws_sdk_s3::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let client = aws_sdk_s3::Client::from_conf_conn(conf, conn);
    let _response = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request
            .expect_request()
            .headers()
            .get("x-amzn-trace-id"),
        Some(&HeaderValue::from_static("traceid"))
    );
}
