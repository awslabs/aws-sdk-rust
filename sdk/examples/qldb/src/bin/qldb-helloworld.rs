/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use qldbsession::model::StartSessionRequest;
use qldbsession::{Client, Config, Error, Region, PKG_VERSION};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The default AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

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
/// * `[-d DEFAULT-REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        ledger,
        default_region,
        verbose,
    } = Opt::from_args();

    let region = default_region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    if verbose {
        println!("OLDB version: {}", PKG_VERSION);
        println!("Region:       {:?}", &region);
        println!("Ledger:       {}", ledger);
        println!();
    }

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

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
