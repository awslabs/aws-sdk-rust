/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod adapter;
mod default_config;
mod list_buckets;

#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    crate::list_buckets::s3_list_buckets().await
}
