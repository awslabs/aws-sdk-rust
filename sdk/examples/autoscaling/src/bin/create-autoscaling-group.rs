/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use autoscaling::{Client, Config, Error, Region, PKG_VERSION};
use aws_types::region::{self, ProvideRegion};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The name of the AutoScaling group.
    #[structopt(short, long)]
    autoscaling_name: String,

    /// The ID of the EC2 instance to add to the AutoScaling group.
    #[structopt(short, long)]
    instance_id: String,

    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates an AutoScaling group in the Region.
/// # Arguments
///
/// * `-a AUTOSCALING-NAME` - The name of the AutoScaling group.
/// * `-i INSTANCE-ID` - The ID of the Ec2 instance to add to the AutoScaling group.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        autoscaling_name,
        instance_id,
        region,
        verbose,
    } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    println!();

    if verbose {
        println!("AutoScaling version:    {}", PKG_VERSION);
        println!("Region:                 {:?}", region_provider.region());
        println!("AutoScaling group name: {}", &autoscaling_name);
        println!("Instance ID:            {}", &instance_id);
        println!();
    }

    let conf = Config::builder().region(region_provider).build();
    let client = Client::from_conf(conf);

    client
        .create_auto_scaling_group()
        .auto_scaling_group_name(autoscaling_name)
        .instance_id(instance_id)
        .min_size(1)
        .max_size(5)
        .send()
        .await?;

    println!("Created AutoScaling group");
    Ok(())
}
