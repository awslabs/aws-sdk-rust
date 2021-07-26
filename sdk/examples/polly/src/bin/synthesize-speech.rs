/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_types::region::ProvideRegion;
use polly::model::{OutputFormat, VoiceId};
use polly::{Client, Config, Error, Region, PKG_VERSION};
use std::fs;
use structopt::StructOpt;
use tokio::io::AsyncWriteExt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The file containing the text to synthesize.
    #[structopt(short, long)]
    filename: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Synthesizes UTF-8 input, plain text or SSML, to a stream of bytes in a file.
/// # Arguments
///
/// * `-f FILENAME` - The name of the file containing the text to synthesize.
///    The output is saved in MP3 format in a file with the same basename, but with an __mp3__ extension.
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        filename,
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Polly version: {}", PKG_VERSION);
        println!("Region:        {:?}", &region);
        println!("Filename:      {}", &filename);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

    let content = fs::read_to_string(&filename);

    let resp = client
        .synthesize_speech()
        .output_format(OutputFormat::Mp3)
        .text(content.unwrap())
        .voice_id(VoiceId::Joanna)
        .send()
        .await?;

    // Get MP3 data from response and save it
    let mut blob = resp
        .audio_stream
        .collect()
        .await
        .expect("failed to read data");

    let parts: Vec<&str> = filename.split('.').collect();
    let out_file = format!("{}{}", String::from(parts[0]), ".mp3");

    let mut file = tokio::fs::File::create(out_file)
        .await
        .expect("failed to create file");

    file.write_all_buf(&mut blob)
        .await
        .expect("failed to write to file");

    Ok(())
}
