/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_polly as polly;
use aws_smithy_async::time::StaticTimeSource;
use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::date_time::Format;
use aws_smithy_types::DateTime;
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

    let uri = presigned.uri().parse::<http_1x::Uri>().unwrap();
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

#[tokio::test]
async fn test_presigning_customized() {
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
        .customize()
        .config_override(Config::builder().region(Region::new("us-west-1")))
        .mutate_request(|req| req.set_uri(req.uri().to_string() + "&test").expect("valid"))
        .presigned(
            PresigningConfig::builder()
                .start_time(SystemTime::UNIX_EPOCH + Duration::from_secs(1234567891))
                .expires_in(Duration::from_secs(30))
                .build()
                .unwrap(),
        )
        .await
        .expect("success");

    let uri = presigned.uri().parse::<http_1x::Uri>().unwrap();
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
            "X-Amz-Credential=ANOTREAL%2F20090213%2Fus-west-1%2Fpolly%2Faws4_request",
            "X-Amz-Date=20090213T233131Z",
            "X-Amz-Expires=30",
            "X-Amz-Security-Token=notarealsessiontoken",
            "X-Amz-Signature=7cc39d2dfa3b8057f901b2827522790b48c6162571ed7e09c9725178c1cdd1fb",
            "X-Amz-SignedHeaders=host",
            "test",
        ][..],
        &query_params
    );
    assert_eq!(presigned.headers().count(), 0);
}

/// A presigned URL is a function of its inputs. The clock skew offset is state learned from
/// earlier, unrelated calls, so letting it shift the signing time would make identical inputs
/// produce different URLs and move the validity window off what the caller asked for.
///
/// Two ordinary calls first teach the client that the service is an hour ahead. The second
/// request's signing time proves the skew is live, so the presigning assertion can't pass
/// vacuously.
#[tokio::test]
async fn test_presigning_ignores_clock_skew() {
    // Also used as the presigning `start_time`, so `now()` is the same instant either way and the
    // learned skew is the only thing that could move the signing time.
    let client_time = SystemTime::UNIX_EPOCH + Duration::from_secs(1234567891);
    let service_date = DateTime::from(client_time + Duration::from_secs(60 * 60))
        .fmt(Format::HttpDate)
        .unwrap();

    let ok_response = || {
        http_1x::Response::builder()
            .status(200)
            .header("date", service_date.clone())
            .body(SdkBody::from(r#"{"Voices":[]}"#))
            .unwrap()
    };
    let request = || {
        http_1x::Request::builder()
            .uri("https://polly.us-east-1.amazonaws.com/v1/voices")
            .body(SdkBody::empty())
            .unwrap()
    };
    let http_client = StaticReplayClient::new(vec![
        ReplayEvent::new(request(), ok_response()),
        ReplayEvent::new(request(), ok_response()),
    ]);

    let config = Config::builder()
        .credentials_provider(Credentials::for_tests_with_session_token())
        .region(Region::new("us-east-1"))
        .time_source(StaticTimeSource::new(client_time))
        .http_client(http_client.clone())
        .build();
    let client = polly::Client::from_conf(config);

    client.describe_voices().send().await.expect("success");
    client.describe_voices().send().await.expect("success");

    // 23:31:31Z + 1h. The skew reached an ordinary request.
    let requests = http_client.actual_requests().collect::<Vec<_>>();
    assert_eq!(2, requests.len());
    assert_eq!(
        Some("20090214T003131Z"),
        requests[1].headers().get("x-amz-date"),
    );

    let presigned = client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text("hello, world")
        .voice_id(VoiceId::Joanna)
        .presigned(
            PresigningConfig::builder()
                .start_time(client_time)
                .expires_in(Duration::from_secs(30))
                .build()
                .unwrap(),
        )
        .await
        .expect("success");

    let uri = presigned.uri().parse::<http_1x::Uri>().unwrap();
    let pq = uri.path_and_query().unwrap();
    let path = pq.path();
    let query = pq.query().unwrap();
    let mut query_params: Vec<&str> = query.split('&').collect();
    query_params.sort();

    assert_eq!("GET", presigned.method());
    assert_eq!("/v1/speech", path);
    // Identical to `test_presigning`: signed at 23:31:31, not 00:31:31.
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
