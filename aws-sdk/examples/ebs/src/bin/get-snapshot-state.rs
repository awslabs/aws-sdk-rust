/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2::model::Filter;
use aws_sdk_ec2::{Client, Error, Region, PKG_VERSION};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The ID of the snapshot.
    #[structopt(short, long)]
    snapshot_id: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Retrieves the state of an Amazon Elastic Block Store snapshot using Amazon EC2 API.
/// It must be `completed` before you can use the snapshot.
/// # Arguments
///
/// * `-s SNAPSHOT-ID` - The ID of the snapshot.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        snapshot_id,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("EC2 version: {}", PKG_VERSION);
        println!("Region:      {}", shared_config.region().unwrap());
        println!("Snapshot ID: {}", snapshot_id);
        println!();
    }

    let resp = client
        .describe_snapshots()
        .filters(
            Filter::builder()
                .name("snapshot-id")
                .values(snapshot_id)
                .build(),
        )
        .send()
        .await?;

    println!(
        "State: {}",
        resp.snapshots
            .unwrap()
            .pop()
            .unwrap()
            .state
            .unwrap()
            .as_ref()
    );

    Ok(())
}
