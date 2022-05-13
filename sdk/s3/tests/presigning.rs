/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3 as s3;
use aws_sdk_s3::presigning::request::PresignedRequest;
use http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use s3::presigning::config::PresigningConfig;
use std::error::Error;
use std::time::{Duration, SystemTime};

/// Generates a `PresignedRequest` from the given input.
/// Assumes that that input has a `presigned` method on it.
macro_rules! presign_input {
    ($input:expr) => {{
        let creds = s3::Credentials::new(
            "ANOTREAL",
            "notrealrnrELgWzOk3IfjzDKtFBhDby",
            Some("notarealsessiontoken".to_string()),
            None,
            "test",
        );
        let config = s3::Config::builder()
            .credentials_provider(creds)
            .region(s3::Region::new("us-east-1"))
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
    let presigned = presign_input!(s3::input::GetObjectInput::builder()
        .bucket("test-bucket")
        .key("test-key")
        .build()?);

    let pq = presigned.uri().path_and_query().unwrap();
    let path = pq.path();
    let query = pq.query().unwrap();
    let mut query_params: Vec<&str> = query.split('&').collect();
    query_params.sort();

    assert_eq!("GET", presigned.method().as_str());
    assert_eq!("/test-bucket/test-key", path);
    assert_eq!(
        &[
            "X-Amz-Algorithm=AWS4-HMAC-SHA256",
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=b5a3e99da3c8b5ba152d828105afe8efb6ecb2732b5b5175a693fc3902d709c5",
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
    let presigned = presign_input!(s3::input::PutObjectInput::builder()
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

    assert_eq!("PUT", presigned.method().as_str());
    assert_eq!("/test-bucket/test-key", path);
    assert_eq!(
        &[
            "X-Amz-Algorithm=AWS4-HMAC-SHA256",
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=6a22b8bf422d17fe25e7d9fcbd26df31397ca5e3ad07d1cec95326ffdbe4a0a2",
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
    let presigned = presign_input!(s3::input::UploadPartInput::builder()
        .content_length(12345)
        .bucket("bucket")
        .key("key")
        .part_number(0)
        .upload_id("upload-id")
        .build()?);
    assert_eq!(
        presigned.uri().to_string(),
        "https://s3.us-east-1.amazonaws.com/bucket/key?x-id=UploadPart&uploadId=upload-id&X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20090213T233131Z&X-Amz-Expires=30&X-Amz-SignedHeaders=content-length%3Bhost&X-Amz-Signature=e50e1a4d1dae7465bb7731863a565bdf4137393e3ab4119b5764fb49f5f60b14&X-Amz-Security-Token=notarealsessiontoken"
    );
    Ok(())
}
