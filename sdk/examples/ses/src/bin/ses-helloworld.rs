/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), aws_sdk_sesv2::Error> {
    let client = aws_sdk_sesv2::Client::from_env();
    tracing_subscriber::fmt::init();
    // Prior to running this example, you need to create a contact list

    let new_contact = client
        .create_contact()
        .contact_list_name("my-contacts")
        .email_address("aws-sdk-rust@amazon.com")
        .send()
        .await;
    match new_contact {
        Ok(_) => println!("contact created"),
        Err(e) => eprintln!("failed to create contact: {}", e),
    };
    let rsp = client
        .list_contacts()
        .contact_list_name("my-contacts")
        .send()
        .await?;
    println!("Contacts: ");
    for contact in rsp.contacts.unwrap_or_default() {
        println!(" - {:#?}", &contact)
    }

    Ok(())
}
