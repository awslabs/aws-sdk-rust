/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sagemaker as sagemaker;
use sagemaker::{Client, Region};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The region. Overrides environment variable AWS_DEFAULT_REGION.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the name, status, and type of your SageMaker instances in an AWS Region.
/// /// # Arguments
///
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), sagemaker::Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        default_region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    if verbose {
        println!("SageMaker client version: {}", sagemaker::PKG_VERSION);
        println!(
            "Region:                   {:?}",
            shared_config.region().unwrap()
        );
    }

    let notebooks = client.list_notebook_instances().send().await?;

    for n in notebooks.notebook_instances().unwrap_or_default() {
        let n_instance_type = n.instance_type().unwrap();
        let n_status = n.notebook_instance_status().unwrap();
        let n_name = n.notebook_instance_name().unwrap_or_default();

        println!(
            "Notebook Name : {}, Notebook Status : {:#?}, Notebook Instance Type : {:#?}",
            n_name, n_status, n_instance_type
        );
    }

    Ok(())
}
