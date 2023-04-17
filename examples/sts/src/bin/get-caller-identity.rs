/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sts::{config::Region, meta::PKG_VERSION, Client, Error};
use clap::Parser;
use std::fmt::Debug;

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// Displays the STS AssumeRole Arn.
// snippet-start:[sts.rust.get_caller_identity]
async fn get_caller_identity(client: &Client) -> Result<(), Error> {
    let response = client.get_caller_identity().send().await?;

    println!(
        "Success! AccountId = {}",
        response.account().unwrap_or_default()
    );
    println!(
        "Success! AccountArn = {}",
        response.arn().unwrap_or_default()
    );
    println!(
        "Success! UserID = {}",
        response.user_id().unwrap_or_default()
    );

    Ok(())
}
// snippet-end:[sts.rust.get_caller_identity]

/// Displays information about the Amazon API Gateway REST APIs in the Region.
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
    let Opt { region, verbose } = Opt::parse();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("STS client version: {}", PKG_VERSION);
        println!(
            "Region:                    {}",
            region_provider.region().await.unwrap().as_ref()
        );

        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);
    get_caller_identity(&client).await
}
