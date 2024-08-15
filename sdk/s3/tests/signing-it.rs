/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "test-util")]

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::{Client, Config};
use aws_smithy_runtime::client::http::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use http::header::AUTHORIZATION;

#[tokio::test]
async fn test_signer() {
    let http_client = StaticReplayClient::new(vec![ReplayEvent::new(
        http::Request::builder()
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20090213/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-user-agent, Signature=27e3f59ec3cffaa10e4f1c92112e8fb62d468a04cd32be39e68215f830404dbb")
            .uri("https://test-bucket.s3.us-east-1.amazonaws.com/?list-type=2&prefix=prefix~")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder().status(200).body(SdkBody::empty()).unwrap(),
    )]);
    let config = Config::builder()
        .credentials_provider(SharedCredentialsProvider::new(
            Credentials::for_tests_with_session_token(),
        ))
        .region(Region::new("us-east-1"))
        .http_client(http_client.clone())
        .with_test_defaults()
        .build();
    let client = Client::from_conf(config);
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .prefix("prefix~")
        .send()
        .await;

    http_client.assert_requests_match(&[AUTHORIZATION.as_str()]);
}
