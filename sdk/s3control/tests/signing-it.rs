/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3control::operation::ListAccessPoints;
use aws_sdk_s3control::{Credentials, Region};
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use std::time::{Duration, UNIX_EPOCH};

use aws_sdk_s3control::middleware::DefaultMiddleware;
use aws_smithy_client::Client as CoreClient;
pub type Client<C> = CoreClient<C, DefaultMiddleware>;

#[tokio::test]
async fn test_signer() -> Result<(), aws_sdk_s3control::Error> {
    let creds = Credentials::new(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
        None,
        "test",
    );
    let conf = aws_sdk_s3control::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("authorization",
                    "AWS4-HMAC-SHA256 Credential=ANOTREAL/20211112/us-east-1/s3/aws4_request, \
                    SignedHeaders=host;x-amz-account-id;x-amz-content-sha256;x-amz-date;x-amz-security-token;x-amz-user-agent, \
                    Signature=ac58c2246428af711ab7bca30c704a2b6a5fd7451cf83f3bceff177f1636e277")
            .uri("https://test-bucket.s3-control.us-east-1.amazonaws.com/v20180820/accesspoint")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder().status(200).body("").unwrap(),
    )]);
    let client = Client::new(conn.clone());
    let mut op = ListAccessPoints::builder()
        .account_id("test-bucket")
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .unwrap();
    op.properties_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1636751225));
    op.properties_mut().insert(AwsUserAgent::for_tests());

    client.call(op).await.expect_err("empty response");
    conn.assert_requests_match(&[]);
    Ok(())
}
