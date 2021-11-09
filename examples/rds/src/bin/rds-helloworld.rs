/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use rds::{Client, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Displays information about your RDS instances.
/// # Arguments
///
/// * `[-r REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
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

    if verbose {
        println!("RDS version: {}", PKG_VERSION);
        println!("Region:      {:?}", shared_config.region().unwrap());
        println!();
    }

    let result = client.describe_db_instances().send().await?;

    for db_instance in result.db_instances.unwrap_or_default() {
        println!(
            "DB instance identifier: {:?}",
            db_instance
                .db_instance_identifier
                .expect("instance should have identifiers")
        );
        println!(
            "DB instance class:      {:?}",
            db_instance
                .db_instance_class
                .expect("instance should have class")
        );
        println!(
            "DB instance engine:     {:?}",
            db_instance.engine.expect("instance should have engine")
        );
        println!(
            "DB instance status:     {:?}",
            db_instance
                .db_instance_status
                .expect("instance should have status")
        );
        println!(
            "DB instance endpoint:   {:?}",
            db_instance.endpoint.expect("instance should have endpoint")
        );
    }

    Ok(())
}
