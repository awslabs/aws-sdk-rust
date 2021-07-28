/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::operation::ListObjectsV2;
use aws_sdk_s3::{Credentials, Region};
use smithy_client::test_connection::TestConnection;
use smithy_http::body::SdkBody;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_signer() -> Result<(), aws_sdk_s3::Error> {
    let creds = Credentials::from_keys(
        "ANOTREAL",
        "notrealrnrELgWzOk3IfjzDKtFBhDby",
        Some("notarealsessiontoken".to_string()),
    );
    let conf = aws_sdk_s3::Config::builder()
        .credentials_provider(creds)
        .region(Region::new("us-east-1"))
        .build();
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210618/us-east-1/s3/aws4_request, SignedHeaders=content-length;content-type;host;x-amz-content-sha256;x-amz-date;x-amz-security-token;x-amz-user-agent, Signature=c3f78ce4969bd55cbb90ba91f46e4fcd14d08dae858f1ac9e508712997eabde7")
            .uri("https://s3.us-east-1.amazonaws.com/test-bucket?list-type=2&prefix=prefix~")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder().status(200).body("").unwrap(),
    )]);
    let client = aws_hyper::Client::new(conn.clone());
    let mut op = ListObjectsV2::builder()
        .bucket("test-bucket")
        .prefix("prefix~")
        .build()
        .unwrap()
        .make_operation(&conf)
        .unwrap();
    op.config_mut()
        .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
    op.config_mut().insert(AwsUserAgent::for_tests());

    client.call(op).await.expect_err("empty response");
    for req in conn.requests().iter() {
        req.assert_matches(vec![]);
    }
    assert_eq!(conn.requests().len(), 1);
    Ok(())
}
