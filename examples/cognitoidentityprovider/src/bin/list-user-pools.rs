/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_cognitoidentityprovider::error::DisplayErrorContext;
use aws_sdk_cognitoidentityprovider::{config::Region, meta::PKG_VERSION, Client};
use aws_smithy_types_convert::date_time::DateTimeExt;
use clap::Parser;
use cognitoidentityprovider_code_examples::Error;
use std::process;

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// Lists your user pools.
// snippet-start:[cognitoidentityprovider.rust.list-user-pools]
async fn show_pools(client: &Client) -> Result<(), Error> {
    let response = client.list_user_pools().max_results(10).send().await?;
    if let Some(pools) = response.user_pools() {
        println!("User pools:");
        for pool in pools {
            println!("  ID:              {}", pool.id().unwrap_or_default());
            println!("  Name:            {}", pool.name().unwrap_or_default());
            println!("  Status:          {:?}", pool.status());
            println!("  Lambda Config:   {:?}", pool.lambda_config().unwrap());
            println!(
                "  Last modified:   {}",
                pool.last_modified_date().unwrap().to_chrono_utc()?
            );
            println!(
                "  Creation date:   {:?}",
                pool.creation_date().unwrap().to_chrono_utc()
            );
            println!();
        }
    }
    println!("Next token: {}", response.next_token().unwrap_or_default());

    Ok(())
}
// snippet-end:[cognitoidentityprovider.rust.list-user-pools]

/// Lists your Amazon Cognito user pools in the Region.
/// # Arguments
///
/// * `[-r REGION]` - The region containing the buckets.
///   If not supplied, uses the value of the **AWS_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    if let Err(err) = run_example(Opt::parse()).await {
        eprintln!("Error: {}", DisplayErrorContext(err));
        process::exit(1);
    }
}

async fn run_example(Opt { region, verbose }: Opt) -> Result<(), Error> {
    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("Cognito client version: {}", PKG_VERSION);
        println!(
            "Region:                 {}",
            region_provider.region().await.unwrap().as_ref()
        );

        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    show_pools(&client).await
}
