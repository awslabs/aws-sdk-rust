/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sns::{Client, Error, Region, PKG_VERSION};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The email address to subscribe to the topic.
    #[structopt(short, long)]
    email_address: String,

    /// The ARN of the topic.
    #[structopt(short, long)]
    topic_arn: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

/// Subscribes an email address and publishes a message to a topic.
/// If the email address has not been confirmed for the topic,
/// a confirmation request is also sent to the email address.
/// # Arguments
///
/// * `-e EMAIL_ADDRESS` - The email address of a user subscribing to the topic.
/// * `-t TOPIC_ARN` - The ARN of the topic.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        region,
        email_address,
        topic_arn,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    println!();

    if verbose {
        println!("SNS client version:   {}", PKG_VERSION);
        println!("Region:               {}", shared_config.region().unwrap());
        println!("Email address:        {}", &email_address);
        println!("Topic ARN:            {}", &topic_arn);
        println!();
    }

    println!("Receiving on topic with ARN: `{}`", topic_arn);

    let rsp = client
        .subscribe()
        .topic_arn(&topic_arn)
        .protocol("email")
        .endpoint(email_address)
        .send()
        .await?;

    println!("Added a subscription: {:?}", rsp);

    let rsp = client
        .publish()
        .topic_arn(&topic_arn)
        .message("hello sns!")
        .send()
        .await?;

    println!("Published message: {:?}", rsp);

    Ok(())
}
