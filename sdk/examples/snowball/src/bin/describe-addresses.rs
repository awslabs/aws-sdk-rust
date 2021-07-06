/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_snowball::{Config, Region};
use aws_types::region;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), aws_sdk_snowball::Error> {
    tracing_subscriber::fmt::init();

    let Opt { region } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-east-1"));

    let conf = Config::builder().region(region_provider).build();
    let client = aws_sdk_snowball::Client::from_conf(conf);

    let addresses = client.describe_addresses().send().await?;
    for address in addresses.addresses.unwrap() {
        println!("Address: {:?}", address);
    }

    Ok(())
}
