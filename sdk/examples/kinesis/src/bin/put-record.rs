/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::region::ProvideRegion;
use kinesis::{Client, Config, Error, Region, PKG_VERSION};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The data to add to the stream.
    #[structopt(short, long)]
    data: String,

    /// The name of the partition key.
    #[structopt(short, long)]
    key: String,

    /// The name of the stream.
    #[structopt(short, long)]
    stream_name: String,

    #[structopt(short, long)]
    verbose: bool,
}

/// Adds a record to an Amazon Kinesis data stream.
/// # Arguments
///
/// * `-s STREAM-NAME` - The name of the stream.
/// * `-k KEY-NAME` - The name of the partition key.
/// * `-d DATA` - The data to add.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        data,
        key,
        stream_name,
        region,
        verbose,
    } = Opt::from_args();

    let region = region
        .as_ref()
        .map(|region| Region::new(region.clone()))
        .or_else(|| aws_types::region::default_provider().region())
        .unwrap_or_else(|| Region::new("us-west-2"));

    println!();

    if verbose {
        println!("Kinesis version: {}", PKG_VERSION);
        println!("Region:          {:?}", &region);
        println!("Data:");
        println!();
        println!("{}", &data);
        println!();
        println!("Partition key:   {}", &key);
        println!("Stream name:     {}", &stream_name);
        println!();
    }

    let config = Config::builder().region(region).build();
    let client = Client::from_conf(config);
    let blob = kinesis::Blob::new(data);

    client
        .put_record()
        .data(blob)
        .partition_key(key)
        .stream_name(stream_name)
        .send()
        .await?;

    println!("Put data into stream.");

    Ok(())
}
