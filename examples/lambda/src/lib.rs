/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_sdk_lambda::{config::Region, meta::PKG_VERSION, Client};
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    pub region: Option<String>,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Parser)]
pub struct ArnOpt {
    #[structopt(flatten)]
    pub base: Opt,

    /// The AWS Lambda function's Amazon Resource Name (ARN).
    #[structopt(short, long)]
    pub arn: String,
}

pub fn make_region_provider(opt: Option<String>) -> RegionProviderChain {
    RegionProviderChain::first_try(opt.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"))
}

pub async fn make_config(opt: Opt) -> SdkConfig {
    let region_provider = make_region_provider(opt.region);
    println!();

    if opt.verbose {
        println!("Lambda client version: {}", PKG_VERSION);
        println!(
            "Region:                {}",
            region_provider.region().await.unwrap().as_ref()
        );
        // println!("Lambda function ARN:  {}", &arn);
        println!();
    }

    aws_config::from_env().region(region_provider).load().await
}

pub async fn make_client(opt: Opt) -> Client {
    Client::new(&make_config(opt).await)
}
