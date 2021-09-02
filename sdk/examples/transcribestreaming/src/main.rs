/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use async_stream::stream;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_transcribestreaming::model::{
    AudioEvent, AudioStream, LanguageCode, MediaEncoding, TranscriptResultStream,
};
use aws_sdk_transcribestreaming::{Blob, Client, Error, Region};
use bytes::BufMut;
use std::time::Duration;

const CHUNK_SIZE: usize = 8192;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    let input_stream = stream! {
        let pcm = pcm_data();
        for chunk in pcm.chunks(CHUNK_SIZE) {
            // Sleeping isn't necessary, but emphasizes the streaming aspect of this
            tokio::time::sleep(Duration::from_millis(100)).await;
            yield Ok(AudioStream::AudioEvent(AudioEvent::builder().audio_chunk(Blob::new(chunk)).build()));
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
                for result in transcript.results.unwrap_or_else(|| Vec::new()) {
                    if result.is_partial {
                        println!("Partial: {:?}", result);
                    } else {
                        let first_alternative = &result.alternatives.as_ref().unwrap()[0];
                        full_message += first_alternative.transcript.as_ref().unwrap();
                        full_message.push('\n');
                    }
                }
            }
            otherwise => panic!("received unexpected event type: {:?}", otherwise),
        }
    }
    println!("\nFully transcribed message:\n\n{}", full_message);
    println!("Done.");

    Ok(())
}

fn pcm_data() -> Vec<u8> {
    let audio = include_bytes!("../audio/hello-transcribe-8000.wav");
    let reader = hound::WavReader::new(&audio[..]).unwrap();
    let samples_result: hound::Result<Vec<i16>> = reader.into_samples::<i16>().collect();

    let mut pcm: Vec<u8> = Vec::new();
    for sample in samples_result.unwrap() {
        pcm.put_i16_le(sample);
    }
    pcm
}
