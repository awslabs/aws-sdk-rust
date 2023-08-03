/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::Client;
use aws_smithy_client::test_connection::capture_request;
use aws_smithy_http::operation::error::BuildError;

#[tokio::test]
async fn test_error_when_required_query_param_is_unset() {
    let (conn, _request) = capture_request(None);
    let config = aws_sdk_s3::Config::builder()
        .http_connector(conn)
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .build();
    let client = Client::from_conf(config);

    let err = client
        .abort_multipart_upload()
        .bucket("test-bucket")
        .key("test.txt")
        .send()
        .await
        .unwrap_err();
    let expected = BuildError::missing_field("upload_id", "cannot be empty or unset").to_string();
    let actual = format!("{}", DisplayErrorContext(err));
    assert!(
        actual.contains(&expected),
        "expected error to contain '{expected}', but was '{actual}'",
    )
}

#[tokio::test]
async fn test_error_when_required_query_param_is_set_but_empty() {
    let (conn, _request) = capture_request(None);
    let config = aws_sdk_s3::Config::builder()
        .http_connector(conn)
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .build();
    let client = Client::from_conf(config);

    let err = client
        .abort_multipart_upload()
        .bucket("test-bucket")
        .key("test.txt")
        .upload_id("")
        .send()
        .await
        .unwrap_err();

    let expected = BuildError::missing_field("upload_id", "cannot be empty or unset").to_string();
    let actual = format!("{}", DisplayErrorContext(err));
    assert!(
        actual.contains(&expected),
        "expected error to contain '{expected}', but was '{actual}'",
    )
}
