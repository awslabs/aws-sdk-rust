/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_sdk_dynamodb::{Client, Error};
use clap::Parser;
use dynamodb_code_examples::{make_config, scenario::list::list_tables_limit_10, Opt};

/// Lists your DynamoDB tables.
/// # Arguments
///
/// * `[-r REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let shared_config = make_config(Opt::parse()).await?;
    let client = Client::new(&shared_config);

    list_tables_limit_10(&client).await
}
