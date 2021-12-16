/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_ec2 as ec2;
use aws_sdk_lambda as lambda;
use aws_types::region::Region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region in which the client is created.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the Lambda ARNs and runtimes in the given AWS Region.
async fn show_lambdas(verbose: bool, region: &str) {
    let shared_config = aws_config::from_env()
        .region(Region::new(region.to_string()))
        .load()
        .await;
    let client = lambda::Client::new(&shared_config);

    let resp = client.list_functions().send().await;
    let functions = resp.unwrap().functions.unwrap_or_default();
    let num_functions = functions.len();

    if num_functions > 0 || verbose {
        println!("Found {} functions in {}:", num_functions, region);
        println!();
    }

    for function in functions {
        println!("  ARN:     {}", function.function_arn.unwrap());
        println!("  Runtime: {:?}", function.runtime.unwrap());
        println!();
    }
}

/// Lists the ARNs and runtimes of your Lambda functions in all available regions.
/// # Arguments
///
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), lambda::Error> {
    tracing_subscriber::fmt::init();
    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(ec2::Region::new))
        .or_default_provider()
        .or_else(ec2::Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    println!();

    if verbose {
        println!("EC2 client version:    {}", ec2::PKG_VERSION);
        println!("Lambda client version: {}", lambda::PKG_VERSION);
        println!(
            "Region:                {:?}",
            shared_config.region().unwrap()
        );
        println!();
    }

    // Get list of available regions.
    let ec2_client = ec2::Client::new(&shared_config);
    let resp = ec2_client.describe_regions().send().await;

    for region in resp.unwrap().regions.unwrap_or_default() {
        show_lambdas(verbose, &region.region_name.unwrap()).await;
    }

    Ok(())
}
