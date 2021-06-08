/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[tokio::main]
async fn main() -> Result<(), iam::Error> {
    tracing_subscriber::fmt::init();
    let client = iam::Client::from_env();
    let rsp = client.list_policies().send().await?;
    for policy in rsp.policies.unwrap_or_default() {
        println!(
            "arn: {}; description: {}",
            policy.arn.unwrap(),
            policy.description.unwrap_or_default()
        );
    }
    Ok(())
}
