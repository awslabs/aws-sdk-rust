/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::retry::RetryConfig;
use aws_sdk_s3::operation::list_objects_v2::builders::ListObjectsV2FluentBuilder;
use aws_sdk_s3::Client;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_smithy_wasm::wasi::WasiHttpClientBuilder;

pub(crate) async fn get_default_wasi_config() -> aws_config::SdkConfig {
    let http_client = WasiHttpClientBuilder::new().build();
    aws_config::from_env()
        .region("us-east-2")
        .timeout_config(TimeoutConfig::disabled())
        .retry_config(RetryConfig::disabled())
        .no_credentials()
        .http_client(http_client)
        .load()
        .await
}

#[tokio::test]
pub async fn test_default_config() {
    let shared_config = get_default_wasi_config().await;
    let client = aws_sdk_s3::Client::new(&shared_config);
    assert_eq!(client.config().region().unwrap().to_string(), "us-east-2")
}

async fn s3_list_objects_operation() -> ListObjectsV2FluentBuilder {
    let shared_config = get_default_wasi_config().await;
    let client = Client::new(&shared_config);
    let operation = client
        .list_objects_v2()
        .bucket("nara-national-archives-catalog")
        .delimiter("/")
        .prefix("authority-records/organization/")
        .max_keys(5);

    operation
}

// Test constructing an operation using an SdkConfig with a WASI http client
// We do not send the request to keep these tests sandboxable, a full test of
// the client is in the SDK canary.
#[tokio::test]
pub async fn test_operation_construction() {
    let operation = s3_list_objects_operation().await;
    assert_eq!(
        operation.get_bucket(),
        &Some("nara-national-archives-catalog".to_string())
    );
}
