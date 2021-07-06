/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::{self, ProvideRegion};
use cognitoidentityprovider::{Client, Config, Error, Region};
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

/// Lists your Amazon Cognito user pools
/// # Arguments
///
/// * `[-r REGION]` - The region containing the buckets.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-g]` - Whether to display buckets in all regions.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt { region, verbose } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    if verbose {
        println!(
            "Cognito client version: {}",
            cognitoidentityprovider::PKG_VERSION
        );
        println!("Region:                 {:?}", region_provider.region());
        println!();
    }

    let config = Config::builder().region(region_provider).build();
    let client = Client::from_conf(config);

    let response = client.list_user_pools().max_results(10).send().await?;
    if let Some(pools) = response.user_pools {
        println!("User pools:");
        for pool in pools {
            println!("  ID:              {}", pool.id.unwrap_or_default());
            println!("  Name:            {}", pool.name.unwrap_or_default());
            println!("  Status:          {:?}", pool.status);
            println!("  Lambda Config:   {:?}", pool.lambda_config);
            println!("  Last modified:   {:?}", pool.last_modified_date);
            println!("  Creation date:   {:?}", pool.creation_date);
            println!();
        }
    }
    println!("Next token: {:?}", response.next_token);

    Ok(())
}
