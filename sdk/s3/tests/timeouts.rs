/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_sdk_s3::config::{timeout::TimeoutConfig, Region};
use aws_sdk_s3::error::DisplayErrorContext;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{
    CompressionType, CsvInput, CsvOutput, ExpressionType, FileHeaderInfo, InputSerialization,
    OutputSerialization,
};
use aws_sdk_s3::{Client, Config};
use aws_smithy_async::assert_elapsed;
use aws_smithy_http_client::test_util::NeverClient;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

#[tokio::test(start_paused = true)]
async fn test_event_stream_request_times_out_if_server_is_unresponsive() {
    let config = Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-2"))
        .http_client(NeverClient::new())
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::from_millis(500))
                .build(),
        )
        .build();
    let client = Client::from_conf(config);

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

    let expected = "operation timeout (all attempts including retries) occurred after 500ms";
    let message = format!("{}", DisplayErrorContext(err));
    assert!(
        message.contains(expected),
        "expected '{message}' to contain '{expected}'"
    );
    assert_elapsed!(now, Duration::from_millis(500));
}

#[tokio::test(start_paused = true)]
async fn test_upload_request_times_out_if_server_is_unresponsive() {
    let config = Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-2"))
        .http_client(NeverClient::new())
        .timeout_config(
            TimeoutConfig::builder()
                .operation_timeout(Duration::from_millis(500))
                .build(),
        )
        .build();
    let client = Client::from_conf(config);

    let now = tokio::time::Instant::now();

    let err = client
        .put_object()
        .bucket("aws-rust-sdk")
        .key("sample_data.csv")
        .body(ByteStream::from_static(b"Hello world!"))
        .send()
        .await
        .unwrap_err();

    let expected = "operation timeout (all attempts including retries) occurred after 500ms";
    let message = format!("{}", DisplayErrorContext(err));
    assert!(
        message.contains(expected),
        "expected '{message}' to contain '{expected}'"
    );
    assert_elapsed!(now, std::time::Duration::from_secs_f32(0.5));
}

#[tokio::test]
async fn test_read_timeout() {
    async fn run_server(
        mut shutdown_receiver: tokio::sync::oneshot::Receiver<()>,
    ) -> (impl Future<Output = ()>, SocketAddr) {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let listener_addr = listener.local_addr().unwrap();

        (
            async move {
                while shutdown_receiver.try_recv().is_err() {
                    if let Ok(Ok((_socket, _))) =
                        timeout(Duration::from_millis(100), listener.accept()).await
                    {
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                }
            },
            listener_addr,
        )
    }
    let (server_shutdown, server_shutdown_receiver) = tokio::sync::oneshot::channel();
    let (server_fut, server_addr) = run_server(server_shutdown_receiver).await;
    let server_handle = tokio::spawn(server_fut);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let config = Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-1"))
        .timeout_config(
            TimeoutConfig::builder()
                .read_timeout(Duration::from_millis(300))
                .build(),
        )
        .endpoint_url(format!("http://{server_addr}"))
        .build();
    let client = Client::from_conf(config);

    if let Ok(result) = timeout(
        Duration::from_millis(1000),
        client.get_object().bucket("test").key("test").send(),
    )
    .await
    {
        match result {
            Ok(_) => panic!("should not have succeeded"),
            Err(err) => {
                let message = format!("{}", DisplayErrorContext(&err));
                let expected = "timeout: HTTP read timeout occurred after 300ms";
                assert!(
                    message.contains(expected),
                    "expected '{message}' to contain '{expected}'"
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
    let config = Config::builder()
        .with_test_defaults()
        .region(Region::new("us-east-1"))
        .timeout_config(
            TimeoutConfig::builder()
                .connect_timeout(Duration::from_millis(300))
                .build(),
        )
        .endpoint_url(
            // Emulate a connect timeout error by hitting an unroutable IP
            "http://172.255.255.0:18104",
        )
        .build();
    let client = Client::from_conf(config);

    if let Ok(result) = timeout(
        Duration::from_millis(1000),
        client.get_object().bucket("test").key("test").send(),
    )
    .await
    {
        match result {
            Ok(_) => panic!("should not have succeeded"),
            Err(err) => {
                let message = format!("{}", DisplayErrorContext(&err));
                let expected =
                    "timeout: client error (Connect): HTTP connect timeout occurred after 300ms";
                assert!(
                    message.contains(expected),
                    "expected '{message}' to contain '{expected}'"
                );
            }
        }
    } else {
        panic!("the client didn't timeout");
    }
}
