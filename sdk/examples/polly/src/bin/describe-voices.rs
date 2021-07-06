/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use polly::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to isplay additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Displays a list of the voices in the Region.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
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
        println!("Region: {:?}", &region);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);

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
