/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_autoscalingplans::{Client, Error, Region};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists your Amazon Cognito identities
/// # Arguments
///
/// * `[-r REGION]` - The region containing the buckets.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    if verbose {
        println!(
            "Auto Scaling Plans client version: {}",
            aws_sdk_autoscalingplans::PKG_VERSION
        );
        println!(
            "Region:                            {:?}",
            shared_config.region().unwrap()
        );
        println!();
    }

    let client = Client::new(&shared_config);

    let response = client.describe_scaling_plans().send().await?;
    if let Some(plans) = response.scaling_plans {
        println!("Auto Scaling Plans:");
        for plan in plans {
            println!("{:?}\n", plan);
        }
    }
    println!("Next token: {:?}", response.next_token);

    Ok(())
}
