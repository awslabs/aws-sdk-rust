/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(test)]
mod test {

    use aws_config::{retry::RetryConfig, BehaviorVersion};
    use aws_sdk_s3::config::{AsyncSleep, Sleep};
    use aws_sdk_s3::Client;
    use aws_sdk_s3::{config::Region, Config};
    use aws_smithy_async::test_util::ManualTimeSource;
    use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
    use aws_smithy_types::body::SdkBody;
    use std::time::UNIX_EPOCH;

    #[derive(Debug, Clone)]
    struct FakeSleep;
    impl AsyncSleep for FakeSleep {
        fn sleep(&self, _duration: std::time::Duration) -> Sleep {
            Sleep::new(async move {
                // This is fake, we do nothing
            })
        }
    }

    #[test]
    fn basic_operation_with_retries() {
        // Since wasm32-unknown-unknown is sandboxed it is hard to know that your tests are actually
        // running. wasmtime will happily exit with 0 even if no tests are run. Enabling this makes
        // it very obvious if they are running or not.
        // panic!("The tests are actually running.");

        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

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
            .time_source(ManualTimeSource::new(UNIX_EPOCH))
            .sleep_impl(FakeSleep)
            .retry_config(RetryConfig::standard())
            .build();

        let client = Client::from_conf(config);

        let result = runtime
            .block_on(client.list_objects_v2().bucket("test-bucket").send())
            .unwrap();

        assert_eq!(result.name.unwrap(), "test-bucket");
        assert_eq!(result.prefix.unwrap(), "prefix~");
    }

    #[test]
    #[should_panic]
    fn no_sleep_impl() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

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
            .time_source(ManualTimeSource::new(UNIX_EPOCH))
            .retry_config(RetryConfig::standard())
            .build();

        let client = Client::from_conf(config);

        let _result = runtime
            .block_on(client.list_objects_v2().bucket("test-bucket").send())
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn no_time_source() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

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
            .sleep_impl(FakeSleep)
            .retry_config(RetryConfig::standard())
            .build();

        let client = Client::from_conf(config);

        let _result = runtime
            .block_on(client.list_objects_v2().bucket("test-bucket").send())
            .unwrap();
    }
}
