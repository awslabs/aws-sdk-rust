/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_sesv2::model::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::{Client, Error, Region};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// The contact list containing email addresses to send the message to.
    #[structopt(short, long)]
    contact_list: String,

    /// The AWS Region.
    #[structopt(short, long)]
    default_region: Option<String>,

    /// The email address of the sender.
    #[structopt(short, long)]
    from_address: String,

    /// The message of the email.
    #[structopt(short, long)]
    message: String,

    /// The subject of the email.
    #[structopt(short, long)]
    subject: String,
    /// Whether to display additional runtime information
    #[structopt(short, long)]
    verbose: bool,
}

/// Sends a message to the email addresses in the contact list.
/// # Arguments
///
/// * `-f FROM-ADDRESS` - The email address of the sender.
/// * `-m MESSAGE` - The email message that is sent.
/// * `-s SUBJECT` - The subject of the email message.
/// * `-c CONTACT-LIST` - The contact list with the email addresses of the recepients.
/// * `[-d DEFAULT-REGION]` - The region in which the client is created.
///    If not supplied, uses the value of the **AWS_DEFAULT_REGION** environment variable.
///    If the environment variable is not set, defaults to **us-west-2**.
/// * `[-v]` - Whether to display additional information.
#[tokio::main]
async fn main() -> Result<(), Error> {
    let Opt {
        contact_list,
        default_region,
        from_address,
        message,
        subject,
        verbose,
    } = Opt::from_args();

    let region_provider = RegionProviderChain::first_try(default_region.map(Region::new))
        .or_default_provider()
        .or_else(Region::new("us-west-2"));
    let shared_config = aws_config::from_env().region(region_provider).load().await;

    if verbose {
        println!("SES client version: {}", aws_sdk_sesv2::PKG_VERSION);
        println!("Region:             {:?}", shared_config.region().unwrap());
        println!("From address:       {}", &from_address);
        println!("Contact list:       {}", &contact_list);
        println!("Subject:            {}", &subject);
        println!("Message:            {}", &message);
        println!();
    }

    let client = Client::new(&shared_config);

    // Get list of email addresses from contact list.
    let resp = client
        .list_contacts()
        .contact_list_name(contact_list)
        .send()
        .await;

    let contacts = resp.unwrap().contacts.unwrap_or_default();

    let cs: String = contacts
        .into_iter()
        .map(|i| i.email_address.unwrap_or_default())
        .collect();

    let dest = Destination::builder().to_addresses(cs).build();
    let subject_content = Content::builder().data(subject).charset("UTF-8").build();
    let body_content = Content::builder().data(message).charset("UTF-8").build();
    let body = Body::builder().text(body_content).build();

    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    let email_content = EmailContent::builder().simple(msg).build();

    match client
        .send_email()
        .from_email_address(from_address)
        .destination(dest)
        .content(email_content)
        .send()
        .await
    {
        Ok(_) => {}
        Err(e) => {
            println!("Got an error sending email: {}", e);
        }
    }

    Ok(())
}
