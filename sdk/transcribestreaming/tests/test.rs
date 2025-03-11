/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use async_stream::stream;
use aws_sdk_transcribestreaming::config::{Credentials, Region};
use aws_sdk_transcribestreaming::error::SdkError;
use aws_sdk_transcribestreaming::operation::start_stream_transcription::StartStreamTranscriptionOutput;
#[allow(unused)] // making sure `EventReceiver` is re-exported
use aws_sdk_transcribestreaming::primitives::event_stream::EventReceiver;
use aws_sdk_transcribestreaming::primitives::Blob;
use aws_sdk_transcribestreaming::types::error::{AudioStreamError, TranscriptResultStreamError};
use aws_sdk_transcribestreaming::types::{
    AudioEvent, AudioStream, LanguageCode, MediaEncoding, TranscriptResultStream,
};
use aws_sdk_transcribestreaming::{Client, Config};
use aws_smithy_eventstream::test_util::validate_body;
use aws_smithy_http_client::test_util::dvr::{Event, ReplayingClient};
use bytes::BufMut;
use futures_core::Stream;
use std::error::Error as StdError;

const CHUNK_SIZE: usize = 8192;

#[tokio::test]
async fn test_success() {
    let input_stream = stream! {
        let pcm = pcm_data();
        for chunk in pcm.chunks(CHUNK_SIZE) {
            yield Ok(AudioStream::AudioEvent(AudioEvent::builder().audio_chunk(Blob::new(chunk)).build()));
        }
    };
    let (replayer, mut output) =
        start_request("us-west-2", include_str!("success.json"), input_stream).await;

    let mut full_message = String::new();
    while let Some(event) = output.transcript_result_stream.recv().await.unwrap() {
        match event {
            TranscriptResultStream::TranscriptEvent(transcript_event) => {
                let transcript = transcript_event.transcript.unwrap();
                for result in transcript.results.unwrap_or_default() {
                    if !result.is_partial {
                        let first_alternative = &result.alternatives.as_ref().unwrap()[0];
                        full_message += first_alternative.transcript.as_ref().unwrap();
                        full_message.push('\n');
                    }
                }
            }
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }

    // Validate the requests
    replayer
        .validate(&["content-type", "content-length"], validate_success_body)
        .await
        .unwrap();

    // Validate the responses
    assert_eq!(
        "Good day to you transcribe.\nThis is Polly talking to you from the Rust ST K.\n",
        full_message
    );
}

#[tokio::test]
async fn test_error() {
    let input_stream = stream! {
        let pcm = pcm_data();
        for chunk in pcm.chunks(CHUNK_SIZE).take(1) {
            yield Ok(AudioStream::AudioEvent(AudioEvent::builder().audio_chunk(Blob::new(chunk)).build()));
        }
    };
    let (replayer, mut output) =
        start_request("us-east-1", include_str!("error.json"), input_stream).await;

    match output.transcript_result_stream.recv().await {
        Err(SdkError::ServiceError(context)) => match context.err() {
            TranscriptResultStreamError::BadRequestException(err) => {
                assert_eq!(
                    Some("A complete signal was sent without the preceding empty frame."),
                    err.message()
                );
            }
            otherwise => panic!("Expected BadRequestException, got: {:?}", otherwise),
        },
        otherwise => panic!("Expected BadRequestException, got: {:?}", otherwise),
    }

    // Validate the requests
    replayer
        .validate(&["content-type", "content-length"], validate_error_body)
        .await
        .unwrap();
}

async fn start_request(
    region: &'static str,
    events_json: &str,
    input_stream: impl Stream<Item = Result<AudioStream, AudioStreamError>> + Send + Sync + 'static,
) -> (ReplayingClient, StartStreamTranscriptionOutput) {
    let events: Vec<Event> = serde_json::from_str(events_json).unwrap();
    let replayer = ReplayingClient::new(events);

    let region = Region::from_static(region);
    let config = Config::builder()
        .region(region)
        .http_client(replayer.clone())
        .credentials_provider(Credentials::for_tests())
        .build();
    let client = Client::from_conf(config);

    let output = client
        .start_stream_transcription()
        .language_code(LanguageCode::EnGb)
        .media_sample_rate_hertz(8000)
        .media_encoding(MediaEncoding::Pcm)
        .audio_stream(input_stream.into())
        .send()
        .await
        .unwrap();

    (replayer, output)
}

fn validate_success_body(
    expected_body: &[u8],
    actual_body: &[u8],
) -> Result<(), Box<dyn StdError>> {
    validate_body(expected_body, actual_body, true)
}

// For the error test, the second request frame may not be sent by the client depending on when
// the error response is parsed and bubbled up to the user.
fn validate_error_body(expected_body: &[u8], actual_body: &[u8]) -> Result<(), Box<dyn StdError>> {
    validate_body(expected_body, actual_body, false)
}

fn pcm_data() -> Vec<u8> {
    let audio = include_bytes!("hello-transcribe-8000.wav");
    let reader = hound::WavReader::new(&audio[..]).unwrap();
    let samples_result: hound::Result<Vec<i16>> = reader.into_samples::<i16>().collect();

    let mut pcm: Vec<u8> = Vec::new();
    for sample in samples_result.unwrap() {
        pcm.put_i16_le(sample);
    }
    pcm
}
