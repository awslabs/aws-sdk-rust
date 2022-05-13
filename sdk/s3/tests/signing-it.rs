/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::middleware::DefaultMiddleware;
use aws_sdk_s3::operation::ListObjectsV2;
use aws_sdk_s3::{Credentials, Region};
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_client::Client as CoreClient;
use aws_smithy_http::body::SdkBody;
use std::time::{Duration, UNIX_EPOCH};
pub type Client<C> = CoreClient<C, DefaultMiddleware>;

#[tokio::test]
async fn test_signer() -> Result<(), aws_sdk_s3::Error> {
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
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210618/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token;x-amz-user-agent, Signature=6233614b69271e15db079287874a654183916e509909b5719b00cd8d5f31299e")
            .uri("https://s3.us-east-1.amazonaws.com/test-bucket?list-type=2&prefix=prefix~")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder().status(200).body("").unwrap(),
    )]);
    let client = Client::new(conn.clone());
    let mut op = ListObjectsV2::builder()
        .bucket("test-bucket")
        .prefix("prefix~")
        .build()
        .unwrap()
        .make_operation(&conf)
        .await
        .unwrap();
    op.properties_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
    op.properties_mut().insert(AwsUserAgent::for_tests());

    client.call(op).await.expect_err("empty response");
    conn.assert_requests_match(&[]);
    Ok(())
}
