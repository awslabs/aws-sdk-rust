/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3control::config::{Credentials, Region};
use aws_sdk_s3control::Client;
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use aws_types::SdkConfig;
use std::convert::Infallible;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_signer() {
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
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .http_connector(conn.clone())
        .region(Region::new("us-east-1"))
        .build();
    let client = Client::new(&sdk_config);

    let _ = client
        .list_access_points()
        .account_id("test-bucket")
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1636751225));
            op.properties_mut().insert(AwsUserAgent::for_tests());

            Result::Ok::<_, Infallible>(op)
        })
        .unwrap()
        .send()
        .await
        .expect_err("empty response");

    conn.assert_requests_match(&[]);
}
