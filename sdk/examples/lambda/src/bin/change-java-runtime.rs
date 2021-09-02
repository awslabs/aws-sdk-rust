/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use lambda::model::Runtime;
use lambda::{Client, Error, Region, PKG_VERSION};
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

/// Sets a Lambda function's Java runtime to Corretto.
/// # Arguments
///
/// * `-a ARN` - The ARN of the Lambda function.
/// * `[-r -REGION]` - The Region in which the client is created.
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

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    println!();

    if verbose {
        println!("Lambda version:      {}", PKG_VERSION);
        println!("Region:              {}", shared_config.region().unwrap());
        println!("Lambda function ARN: {}", &arn);
        println!();
    }

    let client = Client::new(&shared_config);

    // Get function's runtime
    let resp = client.list_functions().send().await?;

    for function in resp.functions.unwrap_or_default() {
        // We only change the runtime for the specified function.
        if arn == function.function_arn.unwrap() {
            let rt = function.runtime.unwrap();
            // We only change the Java runtime.
            if rt == Runtime::Java11 || rt == Runtime::Java8 {
                // Change it to Java8a12 (Corretto).
                println!("Original runtime: {:?}", rt);
                let result = client
                    .update_function_configuration()
                    .function_name(function.function_name.unwrap())
                    .runtime(Runtime::Java8al2)
                    .send()
                    .await?;

                let result_rt = result.runtime.unwrap();
                println!("New runtime: {:?}", result_rt);
            }
        }
    }

    Ok(())
}
