/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use polly::model::{Engine, OutputFormat, VoiceId};
use std::error::Error;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let client = polly::Client::from_env();
    let resp = client
        .synthesize_speech()
        .voice_id(VoiceId::Emma)
        .engine(Engine::Neural)
        .output_format(OutputFormat::Mp3)
        .text("Hello, I am polly!")
        .send()
        .await?;
    let audio = resp.audio_stream.expect("data should be included");
    let mut file = File::create("audio.mp3").await?;
    file.write_all(audio.as_ref()).await?;
    println!(
        "Audio written to audio.mp3 ({} bytes)",
        audio.as_ref().len()
    );
    Ok(())
}
