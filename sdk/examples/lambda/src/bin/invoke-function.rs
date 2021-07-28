/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_types::region::{self, ProvideRegion};
use lambda::{Client, Config, Error, Region, PKG_VERSION};
use std::str;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The Lambda function's ARN.
    #[structopt(short, long)]
    arn: String,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Invokes a Lambda function by its ARN.
/// # Arguments
///
/// * `-a ARN` - The ARN of the Lambda function.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let Opt {
        arn,
        region,
        verbose,
    } = Opt::from_args();

    let region_provider = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    if verbose {
        println!("Lambda version: {}", PKG_VERSION);
        println!(
            "Region:         {}",
            region_provider.region().unwrap().as_ref()
        );
        println!("Function ARN:   {}", arn);
        println!();
    }

    let config = Config::builder().region(region_provider).build();
    let client = Client::from_conf(config);

    let resp = client.invoke().function_name(arn).send().await?;
    if let Some(blob) = resp.payload {
        let s = str::from_utf8(blob.as_ref()).expect("invalid utf-8");
        println!("Response: {:?}", s);
    }

    Ok(())
}
