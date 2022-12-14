/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::{Client, Credentials, Region};
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use aws_types::credentials::SharedCredentialsProvider;
use std::convert::Infallible;
use std::time::{Duration, UNIX_EPOCH};

#[tokio::test]
async fn test_signer() {
    let conn = TestConnection::new(vec![(
        http::Request::builder()
            .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210618/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-security-token;x-amz-user-agent, Signature=6233614b69271e15db079287874a654183916e509909b5719b00cd8d5f31299e")
            .uri("https://s3.us-east-1.amazonaws.com/test-bucket?list-type=2&prefix=prefix~")
            .body(SdkBody::empty())
            .unwrap(),
        http::Response::builder().status(200).body("").unwrap(),
    )]);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "ANOTREAL",
            "notrealrnrELgWzOk3IfjzDKtFBhDby",
            Some("notarealsessiontoken".to_string()),
            None,
            "test",
        )))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();
    let client = Client::new(&sdk_config);
    let _ = client
        .list_objects_v2()
        .bucket("test-bucket")
        .prefix("prefix~")
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1624036048));
            op.properties_mut().insert(AwsUserAgent::for_tests());

            Result::Ok::<_, Infallible>(op)
        })
        .unwrap()
        .send()
        .await;

    conn.assert_requests_match(&[]);
}
