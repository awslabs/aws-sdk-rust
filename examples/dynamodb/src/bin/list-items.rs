/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_sdk_dynamodb::{Client, Error};
use clap::Parser;
use dynamodb_code_examples::{make_config, scenario::list::list_items, Opt as BaseOpt};

#[derive(Debug, Parser)]
struct Opt {
    /// The name of the table.
    #[structopt(short, long)]
    table: String,

    #[structopt(flatten)]
    base: BaseOpt,
}

/// Lists the items in a DynamoDB table.
/// # Arguments
///
/// * `-t TABLE` - The name of the table.
/// * `[-r REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt { table, base } = Opt::parse();

    let shared_config = make_config(base).await?;
    let client = Client::new(&shared_config);

    list_items(&client, &table).await
}
