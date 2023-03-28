/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::provider::SharedCredentialsProvider;
use aws_http::user_agent::AwsUserAgent;
use aws_sdk_s3::{config::Credentials, config::Region, types::ObjectAttributes, Client};
use aws_smithy_client::test_connection::TestConnection;
use aws_smithy_http::body::SdkBody;
use aws_types::SdkConfig;
use http::header::AUTHORIZATION;
use std::{
    convert::Infallible,
    time::{Duration, UNIX_EPOCH},
};

const RESPONSE_BODY_XML: &[u8] = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<GetObjectAttributesResponse xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\"><Checksum><ChecksumSHA1>e1AsOh9IyGCa4hLN+2Od7jlnP14=</ChecksumSHA1></Checksum></GetObjectAttributesResponse>";

#[tokio::test]
async fn ignore_invalid_xml_body_root() {
    let conn = TestConnection::new(vec![
        (http::Request::builder()
             .header("x-amz-object-attributes", "Checksum")
             .header("x-amz-user-agent", "aws-sdk-rust/0.123.test api/test-service/0.123 os/windows/XPSP3 lang/rust/1.50.0")
             .header("x-amz-date", "20210618T170728Z")
             .header("authorization", "AWS4-HMAC-SHA256 Credential=ANOTREAL/20210618/us-east-1/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date;x-amz-object-attributes;x-amz-security-token;x-amz-user-agent, Signature=0e6ec749db5a0af07890a83f553319eda95be0e498d058c64880471a474c5378")
             .header("x-amz-content-sha256", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
             .header("x-amz-security-token", "notarealsessiontoken")
             .uri(http::Uri::from_static("https://some-test-bucket.s3.us-east-1.amazonaws.com/test.txt?attributes"))
             .body(SdkBody::empty())
             .unwrap(),
         http::Response::builder()
             .header(
                 "x-amz-id-2",
                 "rbipIUyF3YKPIcqpz6hrP9x9mzYMSqkHzDEp6TEN/STcKvylDIE/LLN6x9t6EKJRrgctNsdNHWk=",
             )
             .header("x-amz-request-id", "K8036R3D4NZNMMVC")
             .header("date", "Tue, 23 Aug 2022 18:17:23 GMT")
             .header("last-modified", "Tue, 21 Jun 2022 16:30:01 GMT")
             .header("server", "AmazonS3")
             .header("content-length", "224")
             .status(200)
             .body(RESPONSE_BODY_XML)
             .unwrap())
    ]);

    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-east-1"))
        .http_connector(conn.clone())
        .build();
    let client = Client::new(&sdk_config);

    let _ = client
        .get_object_attributes()
        .bucket("some-test-bucket")
        .key("test.txt")
        .object_attributes(ObjectAttributes::Checksum)
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
        .await
        .unwrap();

    conn.assert_requests_match(&[AUTHORIZATION]);
}
