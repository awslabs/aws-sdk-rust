/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Self-tests for the connection test harness.
//!
//! These tests verify the public test utility's scripting, synchronization, event recording,
//! failure reporting, and cleanup behavior. They do not define production HTTP client or
//! connection pool behavior.

#![cfg(all(feature = "wire-mock", feature = "default-client"))]

use aws_smithy_http_client::test_util::wire::connection::{
    BodyPlan, ConnectionCloseReason, ConnectionEvent, ConnectionScript, ConnectionTestHarness,
    EndpointPlan, Finish, Http1Response, Http1Script, ManualGate, SocketScript,
};
use aws_smithy_runtime_api::client::dns::ResolveDns;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::Instant;

const IP1: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const IP2: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
const WAIT: Duration = Duration::from_secs(5);

async fn is_bindable(ip: IpAddr) -> bool {
    tokio::net::TcpListener::bind((ip, 0)).await.is_ok()
}

async fn connect(addr: SocketAddr) -> TcpStream {
    tokio::time::timeout(WAIT, TcpStream::connect(addr))
        .await
        .expect("connect timed out")
        .expect("connect failed")
}

async fn write_request(stream: &mut TcpStream, target: &str, host: &str) {
    stream
        .write_all(format!("GET {target} HTTP/1.1\r\nHost: {host}\r\n\r\n").as_bytes())
        .await
        .expect("request write failed");
}

async fn read_response(stream: &mut TcpStream) -> (u16, HashMap<String, String>, Vec<u8>) {
    let head = read_through(stream, b"\r\n\r\n", 64 * 1024).await;
    let head = std::str::from_utf8(&head).expect("response head was not UTF-8");
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .expect("missing status line")
        .split_whitespace()
        .nth(1)
        .expect("missing status")
        .parse()
        .expect("invalid status");
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<HashMap<_, _>>();
    let body_length = headers
        .get("content-length")
        .expect("missing Content-Length")
        .parse()
        .expect("invalid Content-Length");
    let mut body = vec![0; body_length];
    stream
        .read_exact(&mut body)
        .await
        .expect("response body read failed");
    (status, headers, body)
}

async fn read_through(stream: &mut TcpStream, delimiter: &[u8], limit: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(delimiter) {
        assert!(bytes.len() < limit, "delimiter was not found within limit");
        let mut byte = [0];
        let read = tokio::time::timeout(WAIT, stream.read(&mut byte))
            .await
            .expect("read timed out")
            .expect("read failed");
        assert_ne!(read, 0, "connection closed before delimiter");
        bytes.push(byte[0]);
    }
    bytes
}

async fn assert_reset_or_eof(stream: &mut TcpStream) {
    let mut byte = [0];
    match tokio::time::timeout(WAIT, stream.read(&mut byte))
        .await
        .expect("read timed out")
    {
        Ok(0) => {}
        Err(err)
            if matches!(
                err.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::BrokenPipe
            ) => {}
        result => panic!("expected reset or EOF, got {result:?}"),
    }
}

#[tokio::test]
async fn multi_ip_endpoints_share_a_port() {
    assert!(
        is_bindable(IP2).await,
        "127.0.0.2 is not bindable; multi-address loopback tests require this address \
         (see the setup instructions in README.md)"
    );

    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body("one")]).finish(Finish::Close),
        )
        .endpoint(
            IP2,
            Http1Script::responses([Http1Response::ok().body("two")]).finish(Finish::Close),
        )
        .build()
        .await
        .unwrap();

    assert_eq!(harness.endpoints().len(), 2);
    assert_eq!(harness.endpoints()[0].port(), harness.endpoints()[1].port());
    assert_ne!(harness.endpoints()[0].ip(), harness.endpoints()[1].ip());
    assert_eq!(
        harness.endpoint_url(),
        harness.endpoints()[0].endpoint_url()
    );

    for (endpoint, expected) in harness.endpoints().iter().zip(["one", "two"]) {
        let mut stream = connect(endpoint.addr()).await;
        write_request(&mut stream, "/", "example.test").await;
        assert_eq!(read_response(&mut stream).await.2, expected.as_bytes());
    }
    harness.wait_for_tcp_accepts(2, WAIT).await.unwrap();
    assert_eq!(harness.tcp_accepted_by(IP1), 1);
    assert_eq!(harness.tcp_accepted_by(IP2), 1);
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn dns_entries_record_lookups_and_unregistered_names_fail() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, SocketScript::new().close())
        .dns("explicit.test", [IP1])
        .dns_all("all.test")
        .build()
        .await
        .unwrap();
    let resolver = harness.dns_resolver();

    assert_eq!(resolver.resolve_dns("explicit.test").await.unwrap(), [IP1]);
    assert_eq!(resolver.resolve_dns("all.test").await.unwrap(), [IP1]);
    let err = resolver
        .resolve_dns("unknown.test")
        .await
        .expect_err("unregistered hostname should return a DNS resolution error");
    assert!(
        err.to_string().contains("failed to perform DNS lookup"),
        "unexpected error message: {err}"
    );
    assert_eq!(harness.dns_lookup_count(), 3);
    assert!(matches!(
        &harness.events()[2],
        ConnectionEvent::DnsLookup { hostname } if hostname == "unknown.test"
    ));
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn one_connection_serves_an_ordered_keep_alive_sequence() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([
                Http1Response::ok().body("first"),
                Http1Response::new(201).body("second"),
                Http1Response::new(202).body("third"),
            ]),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    for (index, (status, body)) in [(200, "first"), (201, "second"), (202, "third")]
        .into_iter()
        .enumerate()
    {
        write_request(
            &mut stream,
            &format!("/request/{index}?source=test"),
            "pool.example:8080",
        )
        .await;
        let response = read_response(&mut stream).await;
        assert_eq!(response.0, status);
        assert_eq!(response.2, body.as_bytes());
    }
    drop(stream);

    harness.wait_for_http_requests(3, WAIT).await.unwrap();
    assert_eq!(harness.tcp_accepted_count(), 1);
    assert_eq!(
        harness.http_requests(),
        [
            (
                "/request/0?source=test".to_owned(),
                Some("pool.example:8080".to_owned())
            ),
            (
                "/request/1?source=test".to_owned(),
                Some("pool.example:8080".to_owned())
            ),
            (
                "/request/2?source=test".to_owned(),
                Some("pool.example:8080".to_owned())
            ),
        ]
    );
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn connection_close_response_closes_after_one_request() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body("done").connection_close()]),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    write_request(&mut stream, "/", "close.test").await;
    let (_, headers, body) = read_response(&mut stream).await;
    assert_eq!(headers.get("connection").unwrap(), "close");
    assert_eq!(body, b"done");
    assert_reset_or_eof(&mut stream).await;
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn body_can_be_split_around_a_manual_gate() {
    let gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body_plan(BodyPlan::split_at_gate(
                "before-",
                gate.waiter(),
                "after",
            ))])
            .finish(Finish::Close),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;
    write_request(&mut stream, "/", "gate.test").await;

    let head = read_through(&mut stream, b"\r\n\r\n", 64 * 1024).await;
    assert!(std::str::from_utf8(&head)
        .unwrap()
        .contains("Content-Length: 12"));
    let mut first = [0; 7];
    stream.read_exact(&mut first).await.unwrap();
    assert_eq!(&first, b"before-");
    gate.wait_until_reached(WAIT).await.unwrap();
    gate.release();
    gate.release();
    let mut last = [0; 5];
    stream.read_exact(&mut last).await.unwrap();
    assert_eq!(&last, b"after");

    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn server_idle_close_is_controlled_by_a_gate() {
    let gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_http1_request()
                .write_all(
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok",
                )
                .wait(gate.waiter())
                .close(),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;
    write_request(&mut stream, "/", "idle-close.test").await;

    assert_eq!(read_response(&mut stream).await.2, b"ok");
    gate.wait_until_reached(WAIT).await.unwrap();
    gate.release();
    assert_reset_or_eof(&mut stream).await;
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn socket_actions_cover_framing_delay_half_close_and_close() {
    let delay = Duration::from_millis(30);
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_until("\r\n", 32)
                .expect_bytes("AB")
                .read_exact(2)
                .delay(delay)
                .write_all("ok")
                .shutdown_write()
                .close(),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;
    let started = Instant::now();
    stream.write_all(b"prefix\r\nABCD").await.unwrap();

    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    assert_eq!(response, b"ok");
    assert!(started.elapsed() >= delay);
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn reset_on_connect_is_observable() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, SocketScript::new().reset())
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    assert_reset_or_eof(&mut stream).await;
    harness
        .wait_for_event(WAIT, |event| {
            matches!(
                event,
                ConnectionEvent::ConnectionClosed {
                    reason: ConnectionCloseReason::Reset,
                    ..
                }
            )
        })
        .await
        .unwrap();
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn reset_after_a_complete_request_records_the_request() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, SocketScript::new().read_http1_request().reset())
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    write_request(&mut stream, "/reset", "reset.test").await;
    assert_reset_or_eof(&mut stream).await;
    harness.wait_for_http_requests(1, WAIT).await.unwrap();
    assert_eq!(
        harness.http_requests(),
        [("/reset".to_owned(), Some("reset.test".to_owned()))]
    );
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn reset_during_a_response_body_preserves_the_partial_bytes() {
    let gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_http1_request()
                .write_all(
                    "HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: keep-alive\r\n\r\nabc",
                )
                .wait(gate.waiter())
                .reset(),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    write_request(&mut stream, "/", "reset.test").await;
    let head = read_through(&mut stream, b"\r\n\r\n", 64 * 1024).await;
    assert!(std::str::from_utf8(&head)
        .unwrap()
        .contains("Content-Length: 10"));
    let mut partial = [0; 3];
    stream.read_exact(&mut partial).await.unwrap();
    assert_eq!(&partial, b"abc");
    gate.wait_until_reached(WAIT).await.unwrap();
    gate.release();
    assert_reset_or_eof(&mut stream).await;
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn fragmented_request_headers_and_body_are_framed_before_recording() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body("accepted")]).finish(Finish::Close),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;

    for fragment in [
        b"POST /fragmented?yes HTTP/1.1\r\n".as_slice(),
        b"Ho",
        b"st: fragmented.test\r\nContent-Len",
        b"gth: 4\r\n\r",
        b"\nbo",
        b"dy",
    ] {
        stream.write_all(fragment).await.unwrap();
        tokio::task::yield_now().await;
    }

    assert_eq!(read_response(&mut stream).await.2, b"accepted");
    harness.wait_for_http_requests(1, WAIT).await.unwrap();
    assert_eq!(
        harness.http_requests(),
        [(
            "/fragmented?yes".to_owned(),
            Some("fragmented.test".to_owned())
        )]
    );
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn repeated_scripts_handle_concurrent_connections_independently() {
    let gate = ManualGate::new();
    let template = SocketScript::new()
        .read_http1_request()
        .wait(gate.waiter())
        .write_all("HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
        .close();
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, EndpointPlan::repeat_n(3, template))
        .build()
        .await
        .unwrap();

    let mut streams = Vec::new();
    for index in 0..3 {
        let mut stream = connect(harness.endpoints()[0].addr()).await;
        write_request(&mut stream, &format!("/{index}"), "repeat.test").await;
        streams.push(stream);
    }
    gate.wait_for_arrivals(3, WAIT).await.unwrap();
    assert_eq!(gate.arrivals(), 3);
    gate.release();

    for stream in &mut streams {
        assert_eq!(read_response(stream).await.2, b"ok");
    }
    harness.wait_for_http_requests(3, WAIT).await.unwrap();
    assert_eq!(harness.tcp_accepted_count(), 3);
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn unbounded_http_service_handles_multiple_connections() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            EndpointPlan::unbounded(ConnectionScript::http1(Http1Script::serve(
                Http1Response::ok().body("ok").connection_close(),
            ))),
        )
        .build()
        .await
        .unwrap();

    for _ in 0..4 {
        let mut stream = connect(harness.endpoints()[0].addr()).await;
        write_request(&mut stream, "/", "service.test").await;
        assert_eq!(read_response(&mut stream).await.2, b"ok");
    }
    harness.wait_for_tcp_accepts(4, WAIT).await.unwrap();
    harness.shutdown().await.unwrap();
}

#[tokio::test]
async fn await_client_close_must_be_final() {
    let error = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .await_client_close()
                .write_all("unreachable"),
        )
        .build()
        .await
        .expect_err("actions after await_client_close must be rejected");

    assert_eq!(
        error.to_string(),
        "SocketScript::await_client_close must be the final action"
    );
}

#[tokio::test]
async fn accepting_after_plan_exhaustion_surfaces_the_background_failure() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, EndpointPlan::queue([SocketScript::new().close()]))
        .build()
        .await
        .unwrap();

    let mut first = connect(harness.endpoints()[0].addr()).await;
    assert_reset_or_eof(&mut first).await;
    let mut second = connect(harness.endpoints()[0].addr()).await;
    assert_reset_or_eof(&mut second).await;

    let error = harness
        .wait_for_tcp_accepts(2, WAIT)
        .await
        .expect_err("exhausted endpoint plan should fail");
    assert!(error.to_string().contains("plan was exhausted"), "{error}");
    let shutdown_error = harness
        .shutdown()
        .await
        .expect_err("shutdown should report the background failure");
    assert!(
        shutdown_error.to_string().contains("plan was exhausted"),
        "{shutdown_error}"
    );
}

#[tokio::test]
async fn script_failures_surface_through_waits_and_shutdown() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(IP1, SocketScript::new().expect_bytes("expected").close())
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;
    stream.write_all(b"different").await.unwrap();
    assert_reset_or_eof(&mut stream).await;

    let error = harness
        .wait_for_http_requests(1, WAIT)
        .await
        .expect_err("script mismatch should fail the wait");
    assert!(
        error.to_string().contains("socket bytes differed"),
        "{error}"
    );
    let shutdown_error = harness
        .shutdown()
        .await
        .expect_err("shutdown should report the script failure");
    assert!(
        shutdown_error.to_string().contains("socket bytes differed"),
        "{shutdown_error}"
    );
}

#[tokio::test]
async fn explicit_shutdown_cancels_connections_blocked_at_a_gate() {
    let gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_http1_request()
                .wait(gate.waiter())
                .write_all("unreachable"),
        )
        .build()
        .await
        .unwrap();
    let mut stream = connect(harness.endpoints()[0].addr()).await;
    write_request(&mut stream, "/", "shutdown.test").await;
    gate.wait_until_reached(WAIT).await.unwrap();

    tokio::time::timeout(WAIT, harness.shutdown())
        .await
        .expect("harness shutdown timed out")
        .expect("harness shutdown failed");
    assert_reset_or_eof(&mut stream).await;
}

#[test]
#[should_panic(expected = "cannot append a response")]
fn respond_on_repeating_script_panics() {
    let _ = Http1Script::serve(Http1Response::ok().body("ok"))
        .respond(Http1Response::ok().body("extra"));
}

#[test]
#[should_panic(expected = "cannot set a finite finish policy")]
fn finish_on_repeating_script_panics() {
    let _ = Http1Script::serve(Http1Response::ok().body("ok")).finish(Finish::Close);
}
