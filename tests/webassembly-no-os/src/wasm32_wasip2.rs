/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(test)]
mod test {
    use aws_config::timeout::TimeoutConfig;
    use aws_config::{retry::RetryConfig, BehaviorVersion};
    use aws_sdk_s3::Client;
    use aws_sdk_s3::{config::Region, Config};
    use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_wasm::wasi::WasiHttpClientBuilder;

    // Test constructing an operation using an SdkConfig with a WASI http client
    // We do not send the request to keep these tests sandboxable, a full test of
    // the client is in the SDK canary.
    #[tokio::test]
    async fn test_operation_construction() {
        let http_client = WasiHttpClientBuilder::new().build();
        let wasi_config = aws_config::from_env()
            .region("us-east-2")
            .timeout_config(TimeoutConfig::disabled())
            .retry_config(RetryConfig::disabled())
            .no_credentials()
            .http_client(http_client)
            .load()
            .await;

        let client = Client::new(&wasi_config);
        let operation = client
            .list_objects_v2()
            .bucket("nara-national-archives-catalog")
            .delimiter("/")
            .prefix("authority-records/organization/")
            .max_keys(5);

        assert_eq!(
            operation.get_bucket(),
            &Some("nara-national-archives-catalog".to_string())
        );
    }

    #[tokio::test]
    async fn basic_operation_with_retries_no_sleep_no_time() {
        let http_client = StaticReplayClient::new(vec![
            ReplayEvent::new(
                http_1x::Request::builder()
                    .uri("https://www.doesntmatter.com")
                    .body(SdkBody::empty())
                    .unwrap(),
                http_1x::Response::builder()
                    .status(500)
                    .body(SdkBody::from("This was an error"))
                    .unwrap(),
            ),
            ReplayEvent::new(
                http_1x::Request::builder()
                    .uri("https://www.doesntmatter.com")
                    .body(SdkBody::empty())
                    .unwrap(),
                http_1x::Response::builder()
                    .status(500)
                    .body(SdkBody::from("This was an error"))
                    .unwrap(),
            ),
            ReplayEvent::new(
                http_1x::Request::builder()
                    .uri("https://www.doesntmatter.com")
                    .body(SdkBody::empty())
                    .unwrap(),
                http_1x::Response::builder()
                    .status(200)
                    .body(SdkBody::from(
                        r#"<?xml version="1.0" encoding="UTF-8"?>
                    <ListBucketResult>
                        <Name>test-bucket</Name>
                        <Prefix>prefix~</Prefix>
                        <KeyCount>1</KeyCount>
                        <MaxKeys>1000</MaxKeys>
                        <IsTruncated>false</IsTruncated>
                        <Contents>
                            <Key>some-file.file</Key>
                            <LastModified>2009-10-12T17:50:30.000Z</LastModified>
                            <Size>434234</Size>
                            <StorageClass>STANDARD</StorageClass>
                        </Contents>
                    </ListBucketResult>
                    "#,
                    ))
                    .unwrap(),
            ),
        ]);

        let config = Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new("us-east-1"))
            .http_client(http_client)
            .retry_config(RetryConfig::standard())
            .build();

        let client = Client::from_conf(config);

        let result = client
            .list_objects_v2()
            .bucket("test-bucket")
            .send()
            .await
            .unwrap();

        assert_eq!(result.name.unwrap(), "test-bucket");
        assert_eq!(result.prefix.unwrap(), "prefix~");
    }
}
