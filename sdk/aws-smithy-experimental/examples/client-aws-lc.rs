/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_experimental::hyper_1_0::{CryptoMode, HyperClientBuilder};

fn main() {
    let _client = HyperClientBuilder::new()
        .crypto_mode(CryptoMode::AwsLc)
        .build_https();
}
