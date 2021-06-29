/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

/// Lists your AWS Elemental MediaPackage endpoint URLs.
use aws_types::region::ProvideRegion;
use mediapackage::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists your AWS Elemental MediaPackage endpoint descriptions and URLs.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("MediaPackage version: {}", PKG_VERSION);
        println!("Region:               {:?}", &region);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    let or_endpoints = client.list_origin_endpoints().send().await?;

    for e in or_endpoints.origin_endpoints.unwrap_or_default() {
        let endpoint_url = e.url.as_deref().unwrap_or_default();
        let endpoint_description = e.description.as_deref().unwrap_or_default();
        println!(
            "Endpoint Description: {}, Endpoint URL : {}",
            endpoint_description, endpoint_url
        );
    }

    Ok(())
}
