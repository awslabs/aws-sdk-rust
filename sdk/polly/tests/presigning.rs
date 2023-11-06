/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_polly as polly;
use polly::config::{Config, Credentials, Region};
use polly::presigning::PresigningConfig;
use polly::types::{OutputFormat, VoiceId};
use std::time::{Duration, SystemTime};

#[tokio::test]
async fn test_presigning() {
    let config = Config::builder()
        .credentials_provider(Credentials::for_tests_with_session_token())
        .region(Region::new("us-east-1"))
        .build();
    let client = polly::Client::from_conf(config);

    let presigned = client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text("hello, world")
        .voice_id(VoiceId::Joanna)
        .presigned(
            PresigningConfig::builder()
                .start_time(SystemTime::UNIX_EPOCH + Duration::from_secs(1234567891))
                .expires_in(Duration::from_secs(30))
                .build()
                .unwrap(),
        )
        .await
        .expect("success");

    let uri = presigned.uri().parse::<http::Uri>().unwrap();
    let pq = uri.path_and_query().unwrap();
    let path = pq.path();
    let query = pq.query().unwrap();
    let mut query_params: Vec<&str> = query.split('&').collect();
    query_params.sort();

    assert_eq!("GET", presigned.method());
    assert_eq!("/v1/speech", path);
    assert_eq!(
        &[
            "OutputFormat=mp3",
            "Text=hello%2C%20world",
            "VoiceId=Joanna",
            "X-Amz-Algorithm=AWS4-HMAC-SHA256",
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-east-1%2Fpolly%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=79fcf765b063aa29d852fa9d7c2a9ccff94d727d743adbff84a0be6afe9a92e8",
            "X-Amz-SignedHeaders=host",
        ][..],
        &query_params
    );
    assert_eq!(presigned.headers().count(), 0);
}
