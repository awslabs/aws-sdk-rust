/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_route53::{Client, Region};
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region. Overrides environment variable AWS_DEFAULT_REGION.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Displays the IDs and names of the hosted zones in the region.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), aws_sdk_route53::Error> {
    let Opt {
        default_region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    if verbose {
        println!("Route53 client version: {}\n", aws_sdk_route53::PKG_VERSION);
        println!(
            "Region:                 {:?}",
            shared_config.region().unwrap()
        );

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let hosted_zone_count = client.get_hosted_zone_count().send().await?;

    println!(
        "\nNumber of hosted zones in region : {}",
        hosted_zone_count.hosted_zone_count.unwrap_or_default(),
    );

    let hosted_zones = client.list_hosted_zones().send().await?;

    for hz in hosted_zones.hosted_zones.unwrap_or_default() {
        let zone_name = hz.name.as_deref().unwrap_or_default();
        let zone_id = hz.id.as_deref().unwrap_or_default();

        println!("Zone ID : {}, Zone Name : {}", zone_id, zone_name);

        let record_sets = client
            .list_resource_record_sets()
            .hosted_zone_id(zone_id)
            .send()
            .await?;
        println!("  Record sets:");
        for set in record_sets.resource_record_sets().unwrap_or_default() {
            println!("    {:?}: {:?}", set.name, set.r#type)
        }
    }

    Ok(())
}
