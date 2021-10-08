/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_config::model::ResourceType;
use aws_sdk_config::{Client, Error, Region, PKG_VERSION};
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

// Lists your resources.
async fn show_resources(
    verbose: bool,
    client: &aws_sdk_config::Client,
) -> Result<(), aws_sdk_config::Error> {
    for value in ResourceType::values() {
        let parsed = ResourceType::from(*value);

        let resp = client
            .list_discovered_resources()
            .resource_type(parsed)
            .send()
            .await?;

        let resources = resp.resource_identifiers.unwrap_or_default();

        if !resources.is_empty() || verbose {
            println!();
            println!("Resources of type {}:", value);
        }

        for resource in resources {
            println!(
                "  Resource ID: {}",
                resource.resource_id.as_deref().unwrap_or_default()
            );
        }
    }

    println!();

    Ok(())
}

/// Lists your AWS Config resources, by resource type, in the Region.
///
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("Config client version: {}", PKG_VERSION);
        println!(
            "Region:                {}",
            region_provider.region().await.unwrap().as_ref()
        );

        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    if !verbose {
        println!("You won't see any output if you don't have any resources defined in the region.");
    }

    show_resources(verbose, &client).await
}
