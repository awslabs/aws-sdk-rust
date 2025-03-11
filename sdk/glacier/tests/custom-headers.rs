/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_glacier::config::{Credentials, Region};
use aws_sdk_glacier::primitives::ByteStream;
use aws_smithy_http_client::test_util::capture_request;
use aws_smithy_protocol_test::{assert_ok, validate_headers};

#[tokio::test]
async fn set_correct_headers() {
    let (http_client, handler) = capture_request(None);
    let conf = aws_sdk_glacier::Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .build();

    let client = aws_sdk_glacier::Client::from_conf(conf);
    let _resp = client
        .upload_archive()
        .vault_name("vault")
        .body(ByteStream::from_path("tests/test-file.txt").await.unwrap())
        .send()
        .await;
    let req = handler.expect_request();
    assert_ok(validate_headers(
        req.headers(),
        [
            (
                "x-amz-sha256-tree-hash",
                "2af02ea61585d13604b26ae314a99fc8e972d1f11daba655a68681843cfced9f",
            ),
            (
                "x-amz-content-sha256",
                "2af02ea61585d13604b26ae314a99fc8e972d1f11daba655a68681843cfced9f",
            ),
        ],
    ));
}

#[tokio::test]
async fn autofill_account_id() {
    let (http_client, handler) = capture_request(None);
    let conf = aws_sdk_glacier::Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .build();

    let client = aws_sdk_glacier::Client::from_conf(conf);
    let _resp = client
        .abort_multipart_upload()
        .vault_name("vault")
        .upload_id("some/upload/id")
        .send()
        .await;
    let req = handler.expect_request();
    assert_eq!(
        "https://glacier.us-east-1.amazonaws.com/-/vaults/vault/multipart-uploads/some%2Fupload%2Fid",
        req.uri()
    );
}

#[tokio::test]
async fn api_version_set() {
    let (http_client, handler) = capture_request(None);
    let conf = aws_sdk_glacier::Config::builder()
        .region(Region::new("us-east-1"))
        .credentials_provider(Credentials::for_tests())
        .http_client(http_client)
        .build();

    let client = aws_sdk_glacier::Client::from_conf(conf);
    let _resp = client
        .abort_multipart_upload()
        .vault_name("vault")
        .upload_id("some/upload/id")
        .send()
        .await;
    let req = handler.expect_request();
    assert_ok(validate_headers(
        req.headers(),
        [("x-amz-glacier-version", "2012-06-01")],
    ));
}
