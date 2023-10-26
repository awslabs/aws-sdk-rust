/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![allow(clippy::result_large_err)]

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_kms::types::DataKeySpec;
use aws_sdk_kms::{config::Region, meta::PKG_VERSION, Client, Error};
use clap::Parser;

#[derive(Debug, Parser)]
struct Opt {
    /// The AWS Region.
    #[structopt(short, long)]
    region: Option<String>,

    /// The encryption key.
    #[structopt(short, long)]
    key: String,

    /// Whether to display additional information.
    #[structopt(short, long)]
    verbose: bool,
}

// Create data key.
// snippet-start:[kms.rust.generate-data-key-without-plaintext]
async fn make_key(client: &Client, key: &str) -> Result<(), Error> {
    let resp = client
        .generate_data_key_without_plaintext()
        .key_id(key)
        .key_spec(DataKeySpec::Aes256)
        .send()
        .await?;

    // Did we get an encrypted blob?
    let blob = resp.ciphertext_blob.expect("Could not get encrypted text");
    let bytes = blob.as_ref();

    let s = base64::encode(bytes);

    println!();
    println!("Data key:");
    println!("{}", s);

    Ok(())
}
// snippet-end:[kms.rust.generate-data-key-without-plaintext]

/// Creates an AWS KMS data key without plaintext.
/// # Arguments
///
/// * `[-k KEY]` - The name of the data key.
/// * `[-r REGION]` - The Region in which the client is created.
///    If not supplied, uses the value of the **AWS_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    let Opt {
        key,
        region,
        verbose,
    } = Opt::parse();

    let region_provider = RegionProviderChain::first_try(region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    println!();

    if verbose {
        println!("KMS client version: {}", PKG_VERSION);
        println!(
            "Region:             {}",
            region_provider.region().await.unwrap().as_ref()
        );
        println!("KMS key:            {}", &key);
        println!();
    }

    let shared_config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&shared_config);

    make_key(&client, &key).await
}
