/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sesv2::{Client, Error, Region};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The name of the contact list.
    #[structopt(short, long)]
    contact_list: String,

    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Lists the contacts in a contact list.
/// # Arguments
///
/// * `-c CONTACT-LIST` - The name of the contact list.
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        contact_list,
        default_region,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    if verbose {
        println!("SES client version: {}", aws_sdk_sesv2::PKG_VERSION);
        println!("Region:             {:?}", shared_config.region().unwrap());
        println!("Contact list:       {}", &contact_list);
        println!();
    }

    let client = Client::new(&shared_config);

    let resp = client
        .list_contacts()
        .contact_list_name(contact_list)
        .send()
        .await;

    for contact in resp.unwrap().contacts.unwrap_or_default() {
        println!("{}", contact.email_address.as_deref().unwrap_or_default());
    }

    Ok(())
}
