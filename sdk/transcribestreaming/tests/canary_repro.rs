/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Verifies that the schema-based serialization path (via SdkConfig →
//! SharedClientProtocol) produces correct requests for streaming operations.

use async_stream::stream;
use aws_credential_types::Credentials;
use aws_sdk_transcribestreaming::config::Region;
use aws_sdk_transcribestreaming::primitives::Blob;
use aws_sdk_transcribestreaming::types::{
    AudioEvent, AudioStream, LanguageCode, MediaEncoding, TranscriptResultStream,
};
use aws_sdk_transcribestreaming::Client;
use aws_smithy_http_client::test_util::dvr::{Event, ReplayingClient};
use bytes::BufMut;

const CHUNK_SIZE: usize = 8192;

fn client_from_sdk_config(replayer: ReplayingClient) -> Client {
    let sdk_config =
        aws_types::SdkConfig::builder()
            .region(Region::from_static("us-west-2"))
            .credentials_provider(
                aws_credential_types::provider::SharedCredentialsProvider::new(
                    Credentials::for_tests(),
                ),
            )
            .http_client(replayer)
            .build();
    Client::new(&sdk_config)
}

async fn run_transcribe(client: &Client) -> Result<String, Box<dyn std::error::Error>> {
    let input_stream = stream! {
        let pcm = pcm_data();
        for chunk in pcm.chunks(CHUNK_SIZE) {
            yield Ok(AudioStream::AudioEvent(
                AudioEvent::builder().audio_chunk(Blob::new(chunk)).build(),
            ));
        }
    };

    let mut output = client
        .start_stream_transcription()
        .language_code(LanguageCode::EnGb)
        .media_sample_rate_hertz(8000)
        .media_encoding(MediaEncoding::Pcm)
        .audio_stream(input_stream.into())
        .send()
        .await?;

    let mut full_message = String::new();
    while let Some(event) = output.transcript_result_stream.recv().await? {
        match event {
            TranscriptResultStream::TranscriptEvent(transcript_event) => {
                let transcript = transcript_event.transcript.unwrap();
                for result in transcript.results.unwrap_or_default() {
                    if !result.is_partial {
                        let first_alternative = &result.alternatives.as_ref().unwrap()[0];
                        full_message += first_alternative.transcript.as_ref().unwrap();
                        full_message.push(' ');
                    }
                }
            }
            otherwise => panic!("received unexpected event type: {otherwise:?}"),
        }
    }
    Ok(full_message)
}

/// Test that the schema-based path (SdkConfig → SharedClientProtocol) produces
/// correct requests, matching the canary's code path.
#[tokio::test]
async fn test_schema_path_with_replaying_client() {
    let events: Vec<Event> = serde_json::from_str(include_str!("success.json")).unwrap();
    let replayer = ReplayingClient::new(events);
    let client = client_from_sdk_config(replayer.clone());
    let result = run_transcribe(&client)
        .await
        .expect("transcription should succeed");
    assert!(
        !result.trim().is_empty(),
        "transcription result should not be empty"
    );
    replayer
        .validate(&["content-type", "content-length"], |expected, actual| {
            aws_smithy_eventstream::test_util::validate_body(expected, actual, true)
        })
        .await
        .unwrap();
}

/// Verify that event stream requests do NOT have a Content-Length header.
/// The protocol's serialize_request may set Content-Length from the initial
/// empty body, but it must be removed when the body is replaced with the
/// streaming event stream. A stale Content-Length: 0 causes the service to
/// cancel the HTTP/2 stream immediately.
#[tokio::test]
async fn test_event_stream_no_content_length() {
    let events: Vec<Event> = serde_json::from_str(include_str!("success.json")).unwrap();
    let replayer = ReplayingClient::new(events);
    let client = client_from_sdk_config(replayer.clone());
    let _ = run_transcribe(&client).await;
    let requests = replayer.take_requests().await;
    let first = requests.first().expect("should have at least one request");
    assert!(
        first.headers().get("content-length").is_none(),
        "event stream request must not have Content-Length header, \
         but found: {:?}",
        first.headers().get("content-length"),
    );
}

fn pcm_data() -> Vec<u8> {
    let reader = hound::WavReader::new(&include_bytes!("hello-transcribe-8000.wav")[..])
        .expect("valid wav data");
    let samples_result: hound::Result<Vec<i16>> = reader.into_samples::<i16>().collect();
    let mut pcm: Vec<u8> = Vec::new();
    for sample in samples_result.unwrap() {
        pcm.put_i16_le(sample);
    }
    pcm
}
