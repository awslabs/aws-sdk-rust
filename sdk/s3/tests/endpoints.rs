/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_sdk_s3::config::Builder;
use aws_sdk_s3::config::{Credentials, Region};
use aws_sdk_s3::Client;
use aws_smithy_client::test_connection::{capture_request, CaptureRequestReceiver};
use std::convert::Infallible;
use std::time::{Duration, UNIX_EPOCH};

fn test_client(update_builder: fn(Builder) -> Builder) -> (CaptureRequestReceiver, Client) {
    let (conn, captured_request) = capture_request(None);
    let sdk_config = SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(Credentials::for_tests()))
        .region(Region::new("us-west-4"))
        .http_connector(conn)
        .build();
    let client = Client::from_conf(update_builder(Builder::from(&sdk_config)).build());
    (captured_request, client)
}

#[tokio::test]
async fn virtual_hosted_buckets() {
    let (captured_request, client) = test_client(|b| b);
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3.us-west-4.amazonaws.com/?list-type=2"
    );
}

#[tokio::test]
async fn force_path_style() {
    let (captured_request, client) = test_client(|b| b.force_path_style(true));
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://s3.us-west-4.amazonaws.com/test-bucket/?list-type=2"
    );
}

#[tokio::test]
async fn fips() {
    let (captured_request, client) = test_client(|b| b.use_fips(true));
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3-fips.us-west-4.amazonaws.com/?list-type=2"
    );
}

#[tokio::test]
async fn dual_stack() {
    let (captured_request, client) = test_client(|b| b.use_dual_stack(true));
    let _ = client.list_objects_v2().bucket("test-bucket").send().await;
    assert_eq!(
        captured_request.expect_request().uri().to_string(),
        "https://test-bucket.s3.dualstack.us-west-4.amazonaws.com/?list-type=2"
    );
}

#[tokio::test]
async fn multi_region_access_points() {
    let (_captured_request, client) = test_client(|b| b);
    let response = client
        .get_object()
        .bucket("arn:aws:s3::123456789012:accesspoint/mfzwi23gnjvgw.mrap")
        .key("blah")
        .send()
        .await;
    let error = response.expect_err("should fail—sigv4a is not supported");
    assert!(
        dbg!(format!("{:?}", error)).contains("No auth schemes were supported"),
        "message should contain the correct error, found: {:?}",
        error
    );
}

#[tokio::test]
async fn s3_object_lambda() {
    let (captured_request, client) = test_client(|b| b);
    let _ = client
        .get_object()
        .bucket("arn:aws:s3-object-lambda:us-east-100:123412341234:accesspoint/myolap")
        .key("s3.txt")
        .customize()
        .await
        .unwrap()
        .map_operation(|mut op| {
            op.properties_mut()
                .insert(UNIX_EPOCH + Duration::from_secs(1234567890));
            Result::<_, Infallible>::Ok(op)
        })
        .unwrap()
        .send()
        .await
        .unwrap();
    let captured_request = captured_request.expect_request();
    assert_eq!(captured_request.uri().to_string(), "https://myolap-123412341234.s3-object-lambda.us-east-100.amazonaws.com/s3.txt?x-id=GetObject");
    let auth_header = captured_request.headers().get("AUTHORIZATION").unwrap();
    let auth_header = auth_header.to_str().unwrap();
    // verifies that both the signing scope (s3-object-lambda) has been set as well as the ARN region
    // us-east-100
    let expected_start =
        "AWS4-HMAC-SHA256 Credential=ANOTREAL/20090213/us-east-100/s3-object-lambda/aws4_request";

    assert!(
        auth_header.starts_with(expected_start),
        "expected auth header to start with {} but it was {}",
        expected_start,
        auth_header
    );
}

#[tokio::test]
async fn s3_object_lambda_no_cross_region() {
    let (_, client) = test_client(|b| b.use_arn_region(false));
    let err = client
        .get_object()
        .bucket("arn:aws:s3-object-lambda:us-east-1:123412341234:accesspoint/myolap")
        .key("s3.txt")
        .send()
        .await
        .expect_err("should fail—cross region invalid arn");
    assert!(
        format!("{:?}", err).contains(
            "Invalid configuration: region from ARN `us-east-1` \
    does not match client region `us-west-4` and UseArnRegion is `false`"
        ),
        "{}",
        err
    );
}
