/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3control::config::{Credentials, Region};
use aws_sdk_s3control::{Client, Config};
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use http_1x::header::AUTHORIZATION;

#[tokio::test]
async fn test_signer() {
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http_1x::Request::builder()
        .header("authorization",
                    "AWS4-HMAC-SHA256 Credential=ANOTREAL/20090213/us-east-1/s3/aws4_request, \
                    SignedHeaders=host;x-amz-account-id;x-amz-content-sha256;x-amz-date;x-amz-user-agent, \
                    Signature=0102a74cb220f8445c4efada17660572ff813e07b524032ec831e8c2514be903")
            .uri("https://test-bucket.s3-control.us-east-1.amazonaws.com/v20180820/accesspoint")
            .body(SdkBody::empty())
            .unwrap(),
        http_1x::Response::builder().status(200).body(SdkBody::empty()).unwrap(),
    )]);
    let config = Config::builder()
        .credentials_provider(SharedCredentialsProvider::new(
            Credentials::for_tests_with_session_token(),
        ))
        .http_client(http_client.clone())
        .region(Region::new("us-east-1"))
        .with_test_defaults()
        .build();
    let client = Client::from_conf(config);

    let _ = client
        .list_access_points()
        .account_id("test-bucket")
        .send()
        .await
        .expect_err("empty response");

    http_client.assert_requests_match(&[AUTHORIZATION.as_str()]);
}
