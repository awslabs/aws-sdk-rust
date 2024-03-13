/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_experimental::hyper_1_0::CryptoMode;
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;

#[tokio::test]
#[ignore]
async fn hyper_10_end_to_end() {
    let http_client = aws_smithy_experimental::hyper_1_0::HyperClientBuilder::default()
        .crypto_mode(CryptoMode::Ring)
        .build_https();
    let conf = aws_config::defaults(BehaviorVersion::latest())
        .http_client(http_client)
        .load()
        .await;
    let client = aws_sdk_s3::Client::new(&conf);
    let buckets = client
        .list_buckets()
        .send()
        .await
        .expect("failed to list buckets");
    for bucket in buckets.buckets() {
        println!("{}", bucket.name().unwrap());
    }
}
