/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::Credentials;
use aws_sdk_s3::config::{Region, StalledStreamProtectionConfig};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use bytes::BytesMut;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::debug;

// This test doesn't work because we can't count on `hyper` to poll the body,
// regardless of whether we schedule a wake. To make this functionality work,
// we'd have to integrate more closely with the orchestrator.
//
// I'll leave this test here because we do eventually want to support stalled
// stream protection for uploads.
#[ignore]
#[tokio::test]
async fn test_stalled_stream_protection_defaults_for_upload() {
    // We spawn a faulty server that will close the connection after
    // writing half of the response body.
    let (server, server_addr) = start_faulty_upload_server().await;
    let _ = tokio::spawn(server);

    let conf = Config::builder()
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .endpoint_url(format!("http://{server_addr}"))
        // .stalled_stream_protection(StalledStreamProtectionConfig::enabled().build())
        .build();
    let client = Client::from_conf(conf);

    let err = client
        .put_object()
        .bucket("a-test-bucket")
        .key("stalled-stream-test.txt")
        .body(ByteStream::from_static(b"Hello"))
        .send()
        .await
        .expect_err("upload stream stalled out");

    let err = err.source().expect("inner error exists");
    assert_eq!(
        err.to_string(),
        "minimum throughput was specified at 1 B/s, but throughput of 0 B/s was observed"
    );
}

async fn start_faulty_upload_server() -> (impl Future<Output = ()>, SocketAddr) {
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;

    let listener = TcpListener::bind("0.0.0.0:0")
        .await
        .expect("socket is free");
    let bind_addr = listener.local_addr().unwrap();

    async fn process_socket(socket: TcpStream) {
        let mut buf = BytesMut::new();
        let mut time_to_stall = false;

        loop {
            if time_to_stall {
                debug!("faulty server has read partial request, now getting stuck");
                break;
            }

            match socket.try_read_buf(&mut buf) {
                Ok(0) => {
                    unreachable!(
                        "The connection will be closed before this branch is ever reached"
                    );
                }
                Ok(n) => {
                    debug!("read {n} bytes from the socket");

                    // Check to see if we've received some headers
                    if buf.len() >= 128 {
                        let s = String::from_utf8_lossy(&buf);
                        debug!("{s}");

                        time_to_stall = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("reading would block, sleeping for 1ms and then trying again");
                    sleep(Duration::from_millis(1)).await;
                }
                Err(e) => {
                    panic!("{e}")
                }
            }
        }

        loop {
            tokio::task::yield_now().await
        }
    }

    let fut = async move {
        loop {
            let (socket, addr) = listener
                .accept()
                .await
                .expect("listener can accept new connections");
            debug!("server received new connection from {addr:?}");
            let start = std::time::Instant::now();
            process_socket(socket).await;
            debug!(
                "connection to {addr:?} closed after {:.02?}",
                start.elapsed()
            );
        }
    };

    (fut, bind_addr)
}

#[tokio::test]
async fn test_explicitly_configured_stalled_stream_protection_for_downloads() {
    // We spawn a faulty server that will close the connection after
    // writing half of the response body.
    let (server, server_addr) = start_faulty_download_server().await;
    let _ = tokio::spawn(server);

    let conf = Config::builder()
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .endpoint_url(format!("http://{server_addr}"))
        .stalled_stream_protection(
            StalledStreamProtectionConfig::enabled()
                // Fail stalled streams immediately
                .grace_period(Duration::from_secs(0))
                .build(),
        )
        .build();
    let client = Client::from_conf(conf);

    let res = client
        .get_object()
        .bucket("a-test-bucket")
        .key("stalled-stream-test.txt")
        .send()
        .await
        .unwrap();

    let err = res
        .body
        .collect()
        .await
        .expect_err("download stream stalled out");
    let err = err.source().expect("inner error exists");
    assert_eq!(
        err.to_string(),
        "minimum throughput was specified at 1 B/s, but throughput of 0 B/s was observed"
    );
}

#[tokio::test]
async fn test_stalled_stream_protection_for_downloads_can_be_disabled() {
    // We spawn a faulty server that will close the connection after
    // writing half of the response body.
    let (server, server_addr) = start_faulty_download_server().await;
    let _ = tokio::spawn(server);

    let conf = Config::builder()
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .endpoint_url(format!("http://{server_addr}"))
        .stalled_stream_protection(StalledStreamProtectionConfig::disabled())
        .build();
    let client = Client::from_conf(conf);

    let res = client
        .get_object()
        .bucket("a-test-bucket")
        .key("stalled-stream-test.txt")
        .send()
        .await
        .unwrap();

    let timeout_duration = Duration::from_secs(2);
    match tokio::time::timeout(timeout_duration, res.body.collect()).await {
        Ok(_) => panic!("stalled stream protection kicked in but it shouldn't have"),
        // If timeout elapses, then stalled stream protection didn't end the stream early.
        Err(elapsed) => assert_eq!("deadline has elapsed".to_owned(), elapsed.to_string()),
    }
}

// This test will always take as long as whatever grace period is set by default.
#[tokio::test]
async fn test_stalled_stream_protection_for_downloads_is_enabled_by_default() {
    // We spawn a faulty server that will close the connection after
    // writing half of the response body.
    let (server, server_addr) = start_faulty_download_server().await;
    let _ = tokio::spawn(server);

    // Stalled stream protection should be enabled by default.
    let sdk_config = aws_config::from_env()
        .credentials_provider(Credentials::for_tests())
        .region(Region::new("us-east-1"))
        .endpoint_url(format!("http://{server_addr}"))
        .load()
        .await;
    let client = Client::new(&sdk_config);

    let res = client
        .get_object()
        .bucket("a-test-bucket")
        .key("stalled-stream-test.txt")
        .send()
        .await
        .unwrap();

    let start = std::time::Instant::now();
    let err = res
        .body
        .collect()
        .await
        .expect_err("download stream stalled out");
    let err = err.source().expect("inner error exists");
    assert_eq!(
        err.to_string(),
        "minimum throughput was specified at 1 B/s, but throughput of 0 B/s was observed"
    );
    // 1s check interval + 5s grace period
    assert_eq!(start.elapsed().as_secs(), 6);
}

async fn start_faulty_download_server() -> (impl Future<Output = ()>, SocketAddr) {
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::sleep;

    let listener = TcpListener::bind("0.0.0.0:0")
        .await
        .expect("socket is free");
    let bind_addr = listener.local_addr().unwrap();

    async fn process_socket(socket: TcpStream) {
        let mut buf = BytesMut::new();
        let response: &[u8] = br#"HTTP/1.1 200 OK
x-amz-request-id: 4B4NGF0EAWN0GE63
content-length: 12
etag: 3e25960a79dbc69b674cd4ec67a72c62
content-type: application/octet-stream
server: AmazonS3
content-encoding:
last-modified: Tue, 21 Jun 2022 16:29:14 GMT
date: Tue, 21 Jun 2022 16:29:23 GMT
x-amz-id-2: kPl+IVVZAwsN8ePUyQJZ40WD9dzaqtr4eNESArqE68GSKtVvuvCTDe+SxhTT+JTUqXB1HL4OxNM=
accept-ranges: bytes

"#;
        let mut time_to_respond = false;

        loop {
            match socket.try_read_buf(&mut buf) {
                Ok(0) => {
                    unreachable!(
                        "The connection will be closed before this branch is ever reached"
                    );
                }
                Ok(n) => {
                    debug!("read {n} bytes from the socket");

                    // Check for CRLF to see if we've received the entire HTTP request.
                    if buf.ends_with(b"\r\n\r\n") {
                        time_to_respond = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    debug!("reading would block, sleeping for 1ms and then trying again");
                    sleep(Duration::from_millis(1)).await;
                }
                Err(e) => {
                    panic!("{e}")
                }
            }

            if socket.writable().await.is_ok() && time_to_respond {
                // The content length is 12 but we'll only write 5 bytes
                socket.try_write(response).unwrap();
                // We break from the R/W loop after sending a partial response in order to
                // close the connection early.
                debug!("faulty server has written partial response, now getting stuck");
                break;
            }
        }

        loop {
            tokio::task::yield_now().await
        }
    }

    let fut = async move {
        loop {
            let (socket, addr) = listener
                .accept()
                .await
                .expect("listener can accept new connections");
            debug!("server received new connection from {addr:?}");
            let start = std::time::Instant::now();
            process_socket(socket).await;
            debug!(
                "connection to {addr:?} closed after {:.02?}",
                start.elapsed()
            );
        }
    };

    (fut, bind_addr)
}
