/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{config::Credentials, config::Region, Client};
use aws_smithy_client::test_connection::capture_request;
use std::convert::Infallible;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_operation_should_not_normalize_uri_path() {
    let (conn, rx) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();

    let client = Client::new(&sdk_config);

    let bucket_name = "test-bucket-ad7c9f01-7f7b-4669-b550-75cc6d4df0f1";

    client
        .put_object()
        .bucket(bucket_name)
        .key("a/.././b.txt") // object key with dot segments
        .body(ByteStream::from_static("Hello, world".as_bytes()))
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1669257290));
            op.properties_mut().insert(AwsUserAgent::for_tests());

            Ok::<_, Infallible>(op)
        })
        .unwrap()
        .send()
        .await
        .unwrap();

    let request = rx.expect_request();
    let actual_auth =
        std::str::from_utf8(request.headers().get("authorization").unwrap().as_bytes()).unwrap();

    let actual_uri = request.uri().path();
    let expected_uri = "/a/.././b.txt";
    assert_eq!(actual_uri, expected_uri);

    let expected_sig = "Signature=4803b8b8c794b5ecc055933befd7c5547f8bf6585bb18e4ae33ff65220d5cdd7";
    assert!(
        actual_auth.contains(expected_sig),
        "authorization header signature did not match expected signature: expected {} but not found in {}",
        expected_sig,
        actual_auth,
    );
}
