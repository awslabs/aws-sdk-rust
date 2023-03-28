/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3 as s3;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use s3::config::{Credentials, Region};
use s3::operation::get_object::GetObjectInput;
use s3::operation::head_object::HeadObjectInput;
use s3::operation::put_object::PutObjectInput;
use s3::operation::upload_part::UploadPartInput;
use s3::presigning::{PresignedRequest, PresigningConfig};
use std::error::Error;
use std::time::{Duration, SystemTime};

/// Generates a `PresignedRequest` from the given input.
/// Assumes that that input has a `presigned` method on it.
macro_rules! presign_input {
    ($input:expr) => {{
        let creds = Credentials::for_tests();
        let config = s3::Config::builder()
            .credentials_provider(creds)
            .region(Region::new("us-east-1"))
            .build();

        let req: PresignedRequest = $input
            .presigned(
                &config,
                PresigningConfig::builder()
                    .start_time(SystemTime::UNIX_EPOCH + Duration::from_secs(1234567891))
                    .expires_in(Duration::from_secs(30))
                    .build()
                    .unwrap(),
            )
            .await?;
        req
    }};
}

#[tokio::test]
async fn test_presigning() -> Result<(), Box<dyn Error>> {
    let presigned = presign_input!(GetObjectInput::builder()
        .bucket("test-bucket")
        .key("test-key")
        .build()?);

    let pq = presigned.uri().path_and_query().unwrap();
    let path = pq.path();
    let query = pq.query().unwrap();
    let mut query_params: Vec<&str> = query.split('&').collect();
    query_params.sort();

    assert_eq!(
        "test-bucket.s3.us-east-1.amazonaws.com",
        presigned.uri().authority().unwrap()
    );
    assert_eq!("GET", presigned.method().as_str());
    assert_eq!("/test-key", path);
    assert_eq!(
        &[
            "X-Amz-Algorithm=AWS4-HMAC-SHA256",
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=758353318739033a850182c7b3435076eebbbd095f8dcf311383a6a1e124c4cb",
            "X-Amz-SignedHeaders=host",
            "x-id=GetObject"
        ][..],
        &query_params
    );
    assert!(presigned.headers().is_empty());

    Ok(())
}

#[tokio::test]
async fn test_presigning_with_payload_headers() -> Result<(), Box<dyn Error>> {
    let presigned = presign_input!(PutObjectInput::builder()
        .bucket("test-bucket")
        .key("test-key")
        .content_length(12345)
        .content_type("application/x-test")
        .build()?);

    let pq = presigned.uri().path_and_query().unwrap();
    let path = pq.path();
    let query = pq.query().unwrap();
    let mut query_params: Vec<&str> = query.split('&').collect();
    query_params.sort();

    assert_eq!(
        "test-bucket.s3.us-east-1.amazonaws.com",
        presigned.uri().authority().unwrap()
    );
    assert_eq!("PUT", presigned.method().as_str());
    assert_eq!("/test-key", path);
    assert_eq!(
        &[
            "X-Amz-Algorithm=AWS4-HMAC-SHA256",
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=be1d41dc392f7019750e4f5e577234fb9059dd20d15f6a99734196becce55e52",
            "X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost",
            "x-id=PutObject"
        ][..],
        &query_params
    );

    let mut expected_headers = HeaderMap::new();
    expected_headers.insert(CONTENT_LENGTH, HeaderValue::from_static("12345"));
    expected_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-test"));
    assert_eq!(&expected_headers, presigned.headers());

    Ok(())
}

#[tokio::test]
async fn test_presigned_upload_part() -> Result<(), Box<dyn Error>> {
    let presigned = presign_input!(UploadPartInput::builder()
        .content_length(12345)
        .bucket("bucket")
        .key("key")
        .part_number(0)
        .upload_id("upload-id")
        .build()?);
    assert_eq!(
        presigned.uri().to_string(),
        "https://bucket.s3.us-east-1.amazonaws.com/key?x-id=UploadPart&partNumber=0&uploadId=upload-id&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20090213T233131Z&X-Amz-Expires=30&X-Amz-SignedHeaders=content-length%3Bhost&X-Amz-Signature=a702867244f0bd1fb4d161e2a062520dcbefae3b9992d2e5366bcd61a60c6ddd&X-Amz-Security-Token=notarealsessiontoken",
    );
    Ok(())
}

#[tokio::test]
async fn test_presigning_object_lambda() -> Result<(), Box<dyn Error>> {
    let presigned = presign_input!(GetObjectInput::builder()
        .bucket("arn:aws:s3-object-lambda:us-west-2:123456789012:accesspoint:my-banner-ap-name")
        .key("test2.txt")
        .build()
        .unwrap());
    // since the URI is `my-banner-api-name...` we know EP2 is working properly for presigning
    assert_eq!(presigned.uri().to_string(), "https://my-banner-ap-name-123456789012.s3-object-lambda.us-west-2.amazonaws.com/test2.txt?x-id=GetObject&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ANOTREAL%2F20090213%2Fus-west-2%2Fs3-object-lambda%2Faws4_request&X-Amz-Date=20090213T233131Z&X-Amz-Expires=30&X-Amz-SignedHeaders=host&X-Amz-Signature=027976453050b6f9cca7af80a59c05ee572b462e0fc1ef564c59412b903fcdf2&X-Amz-Security-Token=notarealsessiontoken");
    Ok(())
}

#[tokio::test]
async fn test_presigned_head_object() -> Result<(), Box<dyn Error>> {
    let presigned = presign_input!(HeadObjectInput::builder()
        .bucket("bucket")
        .key("key")
        .build()?);

    assert_eq!("HEAD", presigned.method().as_str());
    assert_eq!(
        presigned.uri().to_string(),
        "https://bucket.s3.us-east-1.amazonaws.com/key?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20090213T233131Z&X-Amz-Expires=30&X-Amz-SignedHeaders=host&X-Amz-Signature=6b97012e70d5ee3528b5591e0e90c0f45e0fa303506f854eff50ff922751a193&X-Amz-Security-Token=notarealsessiontoken",
    );
    Ok(())
}
