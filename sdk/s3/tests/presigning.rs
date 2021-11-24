/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_s3 as s3;
use s3::presigning::config::PresigningConfig;
use std::error::Error;
use std::time::{Duration, SystemTime};

#[tokio::test]
async fn test_presigning() -> Result<(), Box<dyn Error>> {
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

    let input = s3::input::GetObjectInput::builder()
        .bucket("test-bucket")
        .key("test-key")
        .build()?;

    let presigned = input
        .presigned(
            &config,
            PresigningConfig::builder()
                .start_time(SystemTime::UNIX_EPOCH + Duration::from_secs(1234567891))
                .expires_in(Duration::from_secs(30))
                .build()
                .unwrap(),
        )
        .await?;

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
