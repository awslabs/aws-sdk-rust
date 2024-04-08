/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_experimental::hyper_1_0::{CryptoMode, HyperClientBuilder};
#[tokio::main]

async fn main() {
    // feature = crypto-aws-lc
    let _client = HyperClientBuilder::new()
        .crypto_mode(CryptoMode::AwsLc)
        .build_https();

    // feature = crypto-aws-lc-fips
    // A FIPS client can also be created. Note that this has a more complex build environment required.
    let _client = HyperClientBuilder::new()
        .crypto_mode(CryptoMode::AwsLcFips)
        .build_https();
}
