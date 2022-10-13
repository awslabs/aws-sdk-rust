/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_config::SdkConfig;
use aws_sdk_s3::model::{
    CompressionType, CsvInput, CsvOutput, ExpressionType, FileHeaderInfo, InputSerialization,
    OutputSerialization,
};
use aws_sdk_s3::{Client, Config, Credentials, Endpoint, Region};
use aws_smithy_async::assert_elapsed;
use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep, TokioSleep};
use aws_smithy_client::never::NeverService;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use aws_smithy_types::timeout::TimeoutConfig;
use aws_types::credentials::SharedCredentialsProvider;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

#[tokio::test(start_paused = true)]
async fn test_timeout_service_ends_request_that_never_completes() {
    let conn: NeverService<http::Request<SdkBody>, http::Response<SdkBody>, ConnectorError> =
        NeverService::new();
    let region = Region::from_static("us-east-2");
    let credentials = Credentials::new("test", "test", None, None, "test");
    let timeout_config = TimeoutConfig::builder()
        .operation_timeout(Duration::from_secs_f32(0.5))
        .build();
    let sleep_impl: Arc<dyn AsyncSleep> = Arc::new(TokioSleep::new());
    let config = Config::builder()
        .region(region)
        .credentials_provider(credentials)
        .timeout_config(timeout_config)
        .sleep_impl(sleep_impl)
        .build();
    let client = Client::from_conf_conn(config, conn.clone());

    let now = tokio::time::Instant::now();

    let err = client
        .select_object_content()
        .bucket("aws-rust-sdk")
        .key("sample_data.csv")
        .expression_type(ExpressionType::Sql)
        .expression("SELECT * FROM s3object s WHERE s.\"Name\" = 'Jane'")
        .input_serialization(
            InputSerialization::builder()
                .csv(
                    CsvInput::builder()
                        .file_header_info(FileHeaderInfo::Use)
                        .build(),
                )
                .compression_type(CompressionType::None)
                .build(),
        )
        .output_serialization(
            OutputSerialization::builder()
                .csv(CsvOutput::builder().build())
                .build(),
        )
        .send()
        .await
        .unwrap_err();

    assert_eq!(format!("{:?}", err), "TimeoutError(RequestTimeoutError { kind: \"operation timeout (all attempts including retries)\", duration: 500ms })");
    assert_elapsed!(now, std::time::Duration::from_secs_f32(0.5));
}

#[tokio::test]
async fn test_read_timeout() {
    async fn run_server(mut shutdown_receiver: tokio::sync::oneshot::Receiver<()>) {
        let listener = TcpListener::bind("127.0.0.1:18103").await.unwrap();
        while shutdown_receiver.try_recv().is_err() {
            if let Ok(result) = timeout(Duration::from_millis(100), listener.accept()).await {
                if let Ok((_socket, _)) = result {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
            }
        }
    }
    let (server_shutdown, server_shutdown_receiver) = tokio::sync::oneshot::channel();
    let server_handle = tokio::spawn(run_server(server_shutdown_receiver));
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = SdkConfig::builder()
        .sleep_impl(default_async_sleep().unwrap())
        .timeout_config(
            TimeoutConfig::builder()
                .read_timeout(Duration::from_millis(300))
                .build(),
        )
        .endpoint_resolver(Endpoint::immutable(
            "http://127.0.0.1:18103".parse().unwrap(),
        ))
        .region(Some(Region::from_static("us-east-1")))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "test", "test", None, None, "test",
        )))
        .build();
    let client = Client::new(&config);

    if let Ok(result) = timeout(
        Duration::from_millis(1000),
        client.get_object().bucket("test").key("test").send(),
    )
    .await
    {
        match result {
            Ok(_) => panic!("should not have succeeded"),
            Err(err) => {
                assert_eq!(
                    "timeout: HTTP read timeout occurred after 300ms",
                    format!("{}", dbg!(err))
                );
            }
        }
    } else {
        panic!("the client didn't timeout");
    }

    server_shutdown.send(()).unwrap();
    server_handle.await.unwrap();
}

#[tokio::test]
async fn test_connect_timeout() {
    let config = SdkConfig::builder()
        .sleep_impl(default_async_sleep().unwrap())
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(Duration::from_millis(300))
                .build(),
        )
        .endpoint_resolver(Endpoint::immutable(
            // Emulate a connect timeout error by hitting an unroutable IP
            "http://172.255.255.0:18104".parse().unwrap(),
        ))
        .region(Some(Region::from_static("us-east-1")))
        .credentials_provider(SharedCredentialsProvider::new(Credentials::new(
            "test", "test", None, None, "test",
        )))
        .build();
    let client = Client::new(&config);

    if let Ok(result) = timeout(
        Duration::from_millis(1000),
        client.get_object().bucket("test").key("test").send(),
    )
    .await
    {
        match result {
            Ok(_) => panic!("should not have succeeded"),
            Err(err) => {
                assert_eq!(
                    "timeout: error trying to connect: HTTP connect timeout occurred after 300ms",
                    format!("{}", dbg!(err))
                );
            }
        }
    } else {
        panic!("the client didn't timeout");
    }
}
