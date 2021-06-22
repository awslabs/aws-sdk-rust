/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use cloudwatchlogs::Client;

#[tokio::main]
async fn main() -> Result<(), cloudwatchlogs::Error> {
    tracing_subscriber::fmt::init();

    let client = Client::from_env();
    /* uncomment to create a log group */
    /*
    client
        .create_log_group()
        .log_group_name("test-logs")
        .send()
        .await?;

    client
        .create_log_stream()
        .log_group_name("test-logs")
        .log_stream_name("test-stream")
        .send()
        .await?;
     */
    let log_events = client
        .get_log_events()
        .log_group_name("test-logs")
        .log_stream_name("test-stream")
        .send()
        .await?;
    let events = log_events.events.unwrap_or_default();
    println!("number of events: {}", events.len());
    for event in events {
        println!("message: {}", event.message.unwrap_or_default());
    }
    Ok(())
}
