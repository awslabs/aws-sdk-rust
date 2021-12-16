/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_batch::{Client, Region};

#[tokio::main]
async fn main() -> Result<(), aws_sdk_batch::Error> {
    tracing_subscriber::fmt::init();

    let region_provider = RegionProviderChain::default_provider().or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);
    let rsp = client.describe_compute_environments().send().await?;

    let compute_envs = rsp.compute_environments.unwrap_or_default();
    println!("Compute environments ({}):", compute_envs.len());
    for env in compute_envs {
        let arn = env.compute_environment_arn.as_deref().unwrap_or_default();
        let name = env.compute_environment_name.as_deref().unwrap_or_default();

        println!(
            "  Compute Environment Name : {}, Compute Environment ARN : {}",
            name, arn
        );
    }

    Ok(())
}
