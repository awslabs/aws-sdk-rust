/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::{AppName, Credentials, Region};
use aws_sdk_s3::Client;
use aws_smithy_client::test_connection::capture_request;

#[tokio::test]
async fn user_agent_app_name() {
    let (conn, rcvr) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .app_name(AppName::new("test-app-name").expect("valid app name")) // set app name in config
        .build();
    let client = Client::new(&sdk_config);
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;

    // verify app name made it to the user agent
    let request = rcvr.expect_request();
    let formatted = std::str::from_utf8(
        request
            .headers()
            .get("x-amz-user-agent")
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
    assert!(
        formatted.ends_with(" app/test-app-name"),
        "'{}' didn't end with the app name",
        formatted
    );
}
