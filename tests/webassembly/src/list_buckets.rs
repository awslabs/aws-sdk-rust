/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub async fn s3_list_buckets() {
    use aws_sdk_s3::Client;

    use crate::default_config::get_default_config;

    let shared_config = get_default_config().await;
    let client = Client::new(&shared_config);
    let result = client.list_buckets().send().await.unwrap();
    assert_eq!(result.buckets().unwrap().len(), 2)
}

#[tokio::test]
pub async fn test_s3_list_buckets() {
    s3_list_buckets().await
}
