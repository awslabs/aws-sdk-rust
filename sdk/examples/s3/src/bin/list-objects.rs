/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use s3::{Client, Config, Region};

use aws_types::region;

use aws_types::region::ProvideRegion;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default region
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The name of the bucket
    #[structopt(short, long)]
    bucket: String,

    /// Whether to display additional information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the objects in an Amazon S3 bucket.
/// # Arguments
///
/// * `-n NAME` - The name of the bucket.
/// * `[-d DEFAULT-REGION]` - The region containing the bucket.
///   If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///   If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt {
        default_region,
        bucket,
        verbose,
    } = Opt::from_args();

    let region = region::ChainProvider::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    tracing_subscriber::fmt::init();

    if verbose {
        println!("S3 client version: {}", s3::PKG_VERSION);
        println!(
            "Region:            {:?}",
            region.region().expect("region must be set")
        );
    }

    let config = Config::builder().region(region).build();

    let client = Client::from_conf(config);

    match client.list_objects().bucket(&bucket).send().await {
        Ok(resp) => {
            println!("Objects:");
            for object in resp.contents.unwrap_or_default() {
                println!(" `{}`", object.key.expect("objects have keys"));
            }
        }
        Err(e) => {
            println!("Got an error retrieving objects for bucket:");
            println!("{}", e);
            process::exit(1);
        }
    }
}
