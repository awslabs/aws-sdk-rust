/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_types::credentials::SharedCredentialsProvider;
use aws_types::region::Region;
use aws_types::Credentials;

// compiling this function validates that fluent builders are cloneable
#[allow(dead_code)]
async fn ensure_builders_clone() {
    let shared_config = aws_types::SdkConfig::builder()
        .region(Region::new("us-east-4"))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "asdf", "asdf", None, None, "test",
        )))
        .build();
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let base_query = client.list_tables();
    let mut tables = vec![];
    for i in 0..100 {
        let query = base_query
            .clone()
            .exclusive_start_table_name(format!("table-{}", i));
        tables.extend(
            query
                .send()
                .await
                .expect("failed")
                .table_names
                .unwrap_or_default(),
        );
    }
}
