/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_sts::{Credentials, Region};
use aws_smithy_client::test_connection::capture_request;

#[tokio::test]
async fn assume_role_signed() {
    let creds = Credentials::new(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
        None,
        "test",
    );
    let conf = aws_sdk_sts::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let (server, request) = capture_request(None);
    let client = aws_sdk_sts::Client::from_conf_conn(conf, server);
    let _ = client.assume_role().send().await;
    // assume role should have an auth header
    assert_ne!(
        request.expect_request().headers().get("AUTHORIZATION"),
        None
    );
}

#[tokio::test]
async fn web_identity_unsigned() {
    let creds = Credentials::new(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
        None,
        "test",
    );
    let conf = aws_sdk_sts::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let (server, request) = capture_request(None);
    let client = aws_sdk_sts::Client::from_conf_conn(conf, server);
    let _ = client.assume_role_with_web_identity().send().await;
    // web identity should be unsigned
    assert_eq!(
        request.expect_request().headers().get("AUTHORIZATION"),
        None
    );
}

#[tokio::test]
async fn assume_role_saml_unsigned() {
    let conf = aws_sdk_sts::Config::builder()
        .region(Region::new("us-east-1"))
        .build();
    let (server, request) = capture_request(None);
    let client = aws_sdk_sts::Client::from_conf_conn(conf, server);
    let _ = client.assume_role_with_saml().send().await;
    // web identity should be unsigned
    assert_eq!(
        request.expect_request().headers().get("AUTHORIZATION"),
        None
    );
}

#[tokio::test]
async fn web_identity_no_creds() {
    let conf = aws_sdk_sts::Config::builder()
        .region(Region::new("us-east-1"))
        .build();
    let (server, request) = capture_request(None);
    let client = aws_sdk_sts::Client::from_conf_conn(conf, server);
    let _ = client.assume_role_with_web_identity().send().await;
    // web identity should be unsigned and work without credentials
    assert_eq!(
        request.expect_request().headers().get("AUTHORIZATION"),
        None
    );
}
