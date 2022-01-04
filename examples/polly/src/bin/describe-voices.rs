/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_polly::{Client, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to isplay additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Displays a list of the voices in the Region.
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("Polly version: {}", PKG_VERSION);
        println!("Region: {:?}", shared_config.region().unwrap());
        println!();
    }

    let resp = client.describe_voices().send().await?;

    println!("Voices:");

    let voices = resp.voices.unwrap_or_default();
    for voice in &voices {
        println!(
            "  Name:     {}",
            voice.name.as_deref().unwrap_or("No name!")
        );
        println!(
            "  Language: {}",
            voice.language_name.as_deref().unwrap_or("No language!")
        );
    }

    println!();
    println!("Found {} voices", voices.len());
    println!();

    Ok(())
}
