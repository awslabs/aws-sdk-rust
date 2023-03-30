/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::Region;
use aws_sdk_s3::operation::abort_multipart_upload::AbortMultipartUploadInput;
use aws_smithy_http::operation::error::BuildError;

#[tokio::test]
async fn test_error_when_required_query_param_is_unset() {
    let conf = aws_sdk_s3::Config::builder()
        .region(Region::new("us-east-1"))
        .build();

    let err = AbortMultipartUploadInput::builder()
        .bucket("test-bucket")
        .key("test.txt")
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .unwrap_err();

    assert_eq!(
        BuildError::missing_field("upload_id", "cannot be empty or unset").to_string(),
        err.to_string(),
    )
}

#[tokio::test]
async fn test_error_when_required_query_param_is_set_but_empty() {
    let conf = aws_sdk_s3::Config::builder()
        .region(Region::new("us-east-1"))
        .build();
    let err = AbortMultipartUploadInput::builder()
        .bucket("test-bucket")
        .key("test.txt")
        .upload_id("")
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .unwrap_err();

    assert_eq!(
        BuildError::missing_field("upload_id", "cannot be empty or unset").to_string(),
        err.to_string(),
    )
}
