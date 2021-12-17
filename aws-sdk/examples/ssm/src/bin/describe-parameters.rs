/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ssm::{Client, Region};
use std::process;
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

/// Lists the names of your AWS Systems Manager parameters.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    if verbose {
        println!("SSM client version:   {}", aws_sdk_ssm::PKG_VERSION);
        println!(
            "Region:               {:?}",
            shared_config.region().unwrap()
        );

        tracing_subscriber::fmt::init();
    }

    let client = Client::new(&shared_config);

    println!("Parameter names:");

    match client.describe_parameters().send().await {
        Ok(response) => {
            for param in response.parameters.unwrap().iter() {
                match &param.name {
                    None => {}
                    Some(n) => {
                        println!("  {}", n);
                    }
                }
            }
        }
        Err(error) => {
            println!("Got an error listing the parameter names: {}", error);
            process::exit(1);
        }
    }

    println!();
}
