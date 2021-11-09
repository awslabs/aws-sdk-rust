/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use qldbsession::model::StartSessionRequest;
use qldbsession::{Client, Error, Region, PKG_VERSION};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The name of the ledger.
    #[structopt(short, long)]
    ledger: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates a low-level Amazon QLDB session.
/// # Arguments
///
/// * `-l LEDGER` - The name of the ledger to start a new session against.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        ledger,
        region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    if verbose {
        println!("OLDB version: {}", PKG_VERSION);
        println!("Region:       {:?}", shared_config.region().unwrap());
        println!("Ledger:       {}", ledger);
        println!();
    }

    let result = client
        .send_command()
        .start_session(StartSessionRequest::builder().ledger_name(ledger).build())
        .send()
        .await?;

    println!(
        "Session id: {:?}",
        result.start_session.unwrap().session_token
    );

    Ok(())
}
