/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sesv2::{Client, Error, Region};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists your contact lists (there should only be one).
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        default_region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    if verbose {
        println!("SES client version: {}", aws_sdk_sesv2::PKG_VERSION);
        println!("Region:             {:?}", shared_config.region().unwrap());
        println!();
    }

    let client = Client::new(&shared_config);

    let resp = client.list_contact_lists().send().await;

    for list in resp.unwrap().contact_lists.unwrap_or_default() {
        println!("{}", list.contact_list_name.as_deref().unwrap_or_default());
    }

    Ok(())
}
