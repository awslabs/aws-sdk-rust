/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::{Client, Credentials, Region};
use aws_smithy_client::test_connection::capture_request;

#[tokio::test]
async fn virtual_hosted_buckets() {
    let (conn, captured_request) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "ANOTREAL",
            "notrealrnrELgWzOk3IfjzDKtFBhDby",
            Some("notarealsessiontoken".to_string()),
            None,
            "test",
        )))
        .region(Region::new("us-west-4"))
        .http_connector(conn.clone())
        .build();
    let client = Client::new(&sdk_config);
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3.us-west-4.amazonaws.com/?list-type=2"
    );
}

#[tokio::test]
async fn force_path_style() {
    let (conn, captured_request) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "ANOTREAL",
            "notrealrnrELgWzOk3IfjzDKtFBhDby",
            Some("notarealsessiontoken".to_string()),
            None,
            "test",
        )))
        .region(Region::new("us-west-4"))
        .http_connector(conn.clone())
        .build();
    let force_path_style =
        Client::from_conf(Builder::from(&sdk_config).force_path_style(true).build());
    let _ = force_path_style
        .list_objects_v2()
        .bucket("test-bucket")
        .send()
        .await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://s3.us-west-4.amazonaws.com/test-bucket/?list-type=2"
    );
}
