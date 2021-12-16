/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_cloudwatch::Client;

#[tokio::main]
async fn main() -> Result<(), aws_sdk_cloudwatch::Error> {
    tracing_subscriber::fmt::init();

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let rsp = client.list_metrics().send().await?;
    let metrics = rsp.metrics.unwrap_or_default();
    println!("found {} metric(s)", metrics.len());
    for metric in metrics {
        println!("metric: {:?}", metric);
    }
    Ok(())
}
