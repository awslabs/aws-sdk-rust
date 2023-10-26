/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_iotsitewise::error::DisplayErrorContext;
use aws_sdk_iotsitewise::{config::Region, meta::PKG_VERSION, Client};
use aws_smithy_types_convert::date_time::DateTimeExt;
use clap::Parser;
use sitewise_code_examples::Error;
use std::process;

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// List the asset models under AWS IoT SiteWise.
// snippet-start:[sitewise.rust.list-asset-models]
async fn list_asset_models(client: &Client) -> Result<(), Error> {
    let resp = client.list_asset_models().send().await?;

    println!("Asset Models:");

    for asset in resp.asset_model_summaries.unwrap() {
        println!("  ID:  {}", asset.id().unwrap_or_default());
        println!("  ARN:  {}", asset.arn().unwrap_or_default());
        println!("  Name:   {}", asset.name().unwrap_or_default());
        println!(
            "  Description:   {}",
            asset.description().unwrap_or_default()
        );
        println!(
            "  Creation Date:   {}",
            asset.creation_date().unwrap().to_chrono_utc()?
        );
        println!(
            "  Last Update Date:   {}",
            asset.last_update_date().unwrap().to_chrono_utc()?
        );
        println!(
            "  Current Status:   {}",
            asset.status().unwrap().state().unwrap().as_str()
        );

        println!();
    }

    println!();

    Ok(())
}
// snippet-end:[sitewise.rust.list-asset-models]

/// Lists the ID, Amazon Resource Name (ARN), name, description, creation_date, last_update_data,
/// and status of your AWS IoT SiteWise asset models in the Region.
///
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(err) = run_example(Opt::parse()).await {
        eprintln!("Error: {}", DisplayErrorContext(err));
        process::exit(1);
    }
}

async fn run_example(Opt { region, verbose }: Opt) -> Result<(), Error> {
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("IoT client version: {}", PKG_VERSION);
        println!(
            "Region:             {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    list_asset_models(&client).await
}
