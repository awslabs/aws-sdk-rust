/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), mediapackage::Error> {
    let client = mediapackage::Client::from_env();
    let list_channels = client.list_channels().send().await?;

    // List out all the mediapackage channels and display their ARN and description.
    for c in list_channels.channels.unwrap_or_default() {
        let description = c.description.as_deref().unwrap_or_default();
        let arn = c.arn.as_deref().unwrap_or_default();

        println!(
            "Channel Description : {}, Channel ARN : {}",
            description, arn
        );
    }

    Ok(())
}
