/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_sqs::{Client, Error};
use std::process::exit;

/// Sends a message to and receives the message from a FIFO queue.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let queues = client.list_queues().send().await?;
    let mut queue_urls = queues.queue_urls.unwrap_or_default();
    let queue_url = match queue_urls.pop() {
        Some(url) => url,
        None => {
            eprintln!("No queues in this account. Please create a queue to proceed");
            exit(1);
        }
    };

    println!(
        "Sending and receiving messages on with URL: `{}`",
        queue_url
    );

    let rsp = client
        .send_message()
        .queue_url(&queue_url)
        .message_body("hello from my queue")
        // message group id required for FIFO Queue
        // comment out ".message_group_id("MyGroup")" for standard queues
        .message_group_id("MyGroup")
        .send()
        .await?;

    println!("Response from sending a message: {:#?}", rsp);

    let rcv_message_output = client
        .receive_message()
        .queue_url(&queue_url)
        .send()
        .await?;

    for message in rcv_message_output.messages.unwrap_or_default() {
        println!("Got the message: {:#?}", message);
    }

    Ok(())
}
