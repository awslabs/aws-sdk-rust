/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_sdk_dynamodb::{error::DisplayErrorContext, Client};
use clap::Parser;
use dynamodb_code_examples::{
    make_config, scenario::delete::delete_table, scenario::error::Error, Opt as BaseOpt,
};
use std::process;

#[derive(Debug, Parser)]
struct Opt {
    /// The name of the table.
    #[structopt(short, long)]
    table: String,

    #[structopt(flatten)]
    base: BaseOpt,
}

/// Deletes a DynamoDB table.
/// # Arguments
///
/// * `-t TABLE` - The name of the table.
/// * `[-r REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() {
    if let Err(err) = run_example(Opt::parse()).await {
        eprintln!("Error: {}", DisplayErrorContext(err));
        process::exit(1);
    }
}

async fn run_example(Opt { table, base }: Opt) -> Result<(), Error> {
    let shared_config = make_config(base).await?;
    let client = Client::new(&shared_config);

    delete_table(&client, &table).await?;

    Ok(())
}
