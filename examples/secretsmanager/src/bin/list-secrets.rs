/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_secretsmanager::{Client, Region};
use std::process;
use structopt::StructOpt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region
    #[structopt(short, long)]
    region: Option<String>,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the names of your secrets.
/// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    let Opt { region, verbose } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let _client = Client::new(&shared_config);

    if verbose {
        println!(
            "SecretsManager client version: {}",
            aws_sdk_secretsmanager::PKG_VERSION
        );
        println!("Region: {:?}", shared_config.region().unwrap());

        SubscriberBuilder::default()
            .with_env_filter("info")
            .with_span_events(FmtSpan::CLOSE)
            .init();
    }

    let client = Client::new(&shared_config);

    match client.list_secrets().send().await {
        Ok(resp) => {
            println!("Secret names:");

            let secrets = resp.secret_list.unwrap_or_default();
            for secret in &secrets {
                println!("  {}", secret.name.as_deref().unwrap_or("No name!"));
            }

            println!("Found {} secrets", secrets.len());
        }
        Err(e) => {
            println!("Got an error listing secrets:");
            println!("{}", e);
            process::exit(1);
        }
    };
}
