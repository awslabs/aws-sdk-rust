/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::{AppName, Credentials, Region};
use aws_smithy_client::test_connection::capture_request;

#[tokio::test]
async fn user_agent_app_name() -> Result<(), aws_sdk_s3::Error> {
    let (conn, handler) = capture_request(None);
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
        .app_name(AppName::new("test-app-name").expect("valid app name")) // set app name in config
        .build();
    let client = aws_sdk_s3::Client::from_conf_conn(conf, conn);
    let _response = client.list_objects_v2().bucket("test-bucket").send().await;

    // verify app name made it to the user agent
    let request = handler.expect_request();
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

    Ok(())
}
