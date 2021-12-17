/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_sdk_iam as iam;

#[tokio::main]
async fn main() -> Result<(), iam::Error> {
    tracing_subscriber::fmt::init();
    let shared_config = aws_config::load_from_env().await;
    let client = iam::Client::new(&shared_config);
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
