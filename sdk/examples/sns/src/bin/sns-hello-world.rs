/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use sns::Region;
use std::process::exit;

#[tokio::main]
async fn main() -> Result<(), sns::Error> {
    tracing_subscriber::fmt::init();
    let conf = sns::Config::builder()
        .region(Region::new("us-east-2"))
        .build();
    let client = sns::Client::from_conf(conf);
    let topics = client.list_topics().send().await?;
    let mut topics = topics.topics.unwrap_or_default();
    let topic_arn = match topics.pop() {
        Some(topic) => topic.topic_arn.expect("topics have ARNs"),
        None => {
            eprintln!("No topics in this account. Please create a topic to proceed");
            exit(1);
        }
    };
    println!("receiving on `{}`", topic_arn);
    let rsp = client
        .subscribe()
        .topic_arn(&topic_arn)
        .protocol("email")
        .endpoint("some.email.address@example.com")
        .send()
        .await?;
    println!("added a subscription: {:?}", rsp);

    let rsp = client
        .publish()
        .topic_arn(&topic_arn)
        .message("hello sns!")
        .send()
        .await?;
    println!("published a message: {:?}", rsp);

    // If you set this to your email address, you should get an email from SNS
    Ok(())
}
