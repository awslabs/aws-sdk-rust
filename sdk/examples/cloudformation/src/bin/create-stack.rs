/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_cloudformation::{Client, Config, Error, Region, PKG_VERSION};
use aws_types::region;
use aws_types::region::ProvideRegion;
use std::fs;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The name of the AWS CloudFormation stack.
    #[structopt(short, long)]
    stack_name: String,

    /// The name of the file containing the stack template.
    #[structopt(short, long)]
    template_file: String,

    /// Whether to display additional runtime information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Creates a CloudFormation stack in the region.
/// # Arguments
///
/// * `-s STACK-NAME` - The name of the stack.
/// * `-t TEMPLATE-NAME` - The name of the file containing the stack template.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        stack_name,
        template_file,
        verbose,
    } = Opt::from_args();

    let region = region::ChainProvider::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));

    if verbose {
        println!("CloudFormation client version: {}", PKG_VERSION);
        println!(
            "Region:                        {}",
            region.region().unwrap().as_ref()
        );
        println!("Stack:                         {}", &stack_name);
        println!("Template:                      {}", &template_file);
        println!();
    }

    // Get content of template file as a string.
    let contents =
        fs::read_to_string(template_file).expect("Something went wrong reading the file");

    let conf = Config::builder().region(region).build();
    let client = Client::from_conf(conf);

    client
        .create_stack()
        .stack_name(stack_name)
        .template_body(contents)
        .send()
        .await?;

    println!("Stack created.");
    println!("Use describe-stacks with your stack name to see the status of your stack.");
    println!("You cannot use/deploy the stack until the status is 'CreateComplete'.");
    println!();

    Ok(())
}
