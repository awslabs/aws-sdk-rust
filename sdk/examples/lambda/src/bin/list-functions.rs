/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::{self, ProvideRegion};
use lambda::{Client, Config, Error, Region, PKG_VERSION};
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

/// Lists the ARNs of your Lambda functions in the Region.
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
        println!("Lambda version: {}", PKG_VERSION);
        println!(
            "Region:         {}",
            region_provider.region().unwrap().as_ref()
        );
        println!();
    }

    let config = Config::builder().region(region_provider).build();
    let client = Client::from_conf(config);

    let resp = client.list_functions().send().await?;

    println!("Function ARNs:");

    let functions = resp.functions.unwrap_or_default();
    let len = functions.len();

    for function in functions {
        println!("  {}", function.function_arn.unwrap_or_default());
    }

    println!();
    println!("Found {} functions", len);

    Ok(())
}
