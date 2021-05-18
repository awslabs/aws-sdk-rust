/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::process;

use lambda::{Client, Config, Region};

use aws_types::region::ProvideRegion;

use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::SubscriberBuilder;

#[tokio::main]
async fn main() {
    let region = aws_types::region::default_provider()
        .region()
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!("Lambda client version: {}", lambda::PKG_VERSION);
    println!("Region:      {:?}", &region);

    SubscriberBuilder::default()
        .with_env_filter("info")
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let config = Config::builder().region(region).build();

    let client = Client::from_conf(config);

    match client.list_functions().send().await {
        Ok(resp) => {
            println!("Functions:");

            let functions = resp.functions.unwrap_or_default();

            for function in &functions {
                println!("  {:?}", function.function_name);
            }

            println!("Found {} functions", functions.len());
        }
        Err(e) => {
            println!("Got an error listing functions:");
            println!("{}", e);
            process::exit(1);
        }
    };
}
