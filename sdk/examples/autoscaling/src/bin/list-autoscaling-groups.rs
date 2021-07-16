/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use autoscaling::{Client, Config, Error, Region, PKG_VERSION};
use aws_types::region::{self, ProvideRegion};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists your AutoScaling groups in the Region.
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

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("AutoScaling version: {}", PKG_VERSION);
        println!("Region:              {:?}", region_provider.region());
        println!();
    }

    let conf = Config::builder().region(region_provider).build();
    let client = Client::from_conf(conf);

    let resp = client.describe_auto_scaling_groups().send().await?;

    println!("Groups:");

    let groups = resp.auto_scaling_groups.unwrap_or_default();

    for group in &groups {
        println!(
            "  {}",
            group.auto_scaling_group_name.as_deref().unwrap_or_default()
        );
        println!(
            "  ARN:          {}",
            group.auto_scaling_group_arn.as_deref().unwrap_or_default()
        );
        println!("  Minimum size: {}", group.min_size.unwrap_or_default());
        println!("  Maximum size: {}", group.max_size.unwrap_or_default());
        println!();
    }

    println!("Found {} group(s)", groups.len());
    Ok(())
}
