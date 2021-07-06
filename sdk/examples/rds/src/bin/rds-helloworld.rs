/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use rds::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Displays information about your RDS instances.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
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

    if verbose {
        println!("RDS version: {}", PKG_VERSION);
        println!("Region:      {:?}", &region);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

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
