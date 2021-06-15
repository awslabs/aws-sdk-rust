/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), aws_sdk_ec2::Error> {
    let client = aws_sdk_ec2::Client::from_env();
    let rsp = client.describe_regions().send().await?;
    for region in rsp.regions.unwrap_or_default() {
        println!("region: {:#?}", region.region_name.unwrap());
    }

    Ok(())
}
