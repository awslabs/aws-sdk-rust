/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_polly::input::SynthesizeSpeechInput;
use aws_sdk_polly::model::{OutputFormat, VoiceId};
use aws_sdk_polly::presigning::config::PresigningConfig;
use aws_sdk_polly::{Client, Config, Region, PKG_VERSION};
use std::error::Error;
use std::fs;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The file containing the text to synthesize.
    #[structopt(short, long)]
    filename: String,

    /// How long in seconds before the presigned request should expire.
    #[structopt(short, long)]
    expires_in: Option<u64>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Generates a presigned request to synthesize UTF-8 input, plain text or SSML, to a stream of bytes in a file.
/// # Arguments
///
/// * `-f FILENAME` - The name of the file containing the text to synthesize.
///    The output is saved in MP3 format in a file with the same basename, but with an __mp3__ extension.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-e EXPIRES_IN]` - The amount of time the presigned request should be valid for.
///    If not given, this defaults to 15 minutes.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let Opt {
        filename,
        region,
        expires_in,
        verbose,
    } = Opt::from_args();
    let expires_in = Duration::from_secs(expires_in.unwrap_or(900));

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("Polly version: {}", PKG_VERSION);
        println!("Region:        {:?}", shared_config.region().unwrap());
        println!("Filename:      {}", &filename);
        println!();
    }

    let content = fs::read_to_string(&filename).unwrap();

    // Presigned requests can be made with the client directly
    let presigned_request = client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text(content.clone())
        .voice_id(VoiceId::Joanna)
        .presigned(PresigningConfig::expires_in(expires_in)?)
        .await?;
    println!("From client: {:?}", presigned_request);

    // Or, they can be made directly from an operation input
    let presigned_request = SynthesizeSpeechInput::builder()
        .output_format(OutputFormat::Mp3)
        .text(content)
        .voice_id(VoiceId::Joanna)
        .build()?
        .presigned(
            &Config::from(&shared_config),
            PresigningConfig::expires_in(expires_in)?,
        )
        .await?;
    println!("From operation input: {:?}", presigned_request);

    Ok(())
}
