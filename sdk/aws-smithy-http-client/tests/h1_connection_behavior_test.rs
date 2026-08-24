/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/1.1 connection behavior contracts for Smithy HTTP client implementations.
//!
//! Covers connection reuse and lifecycle, request routing, connection metadata, failure and
//! timeout classification, HTTP semantics that affect reusability, concurrency, and DNS
//! resolution.
//!
//! Each contract has an implementation-neutral scenario followed by an explicit test runner for
//! every backend that must satisfy it.
//!
//! A contract function without a corresponding runner becomes dead code, and CI builds
//! with `RUSTFLAGS: -D warnings` turn that into a compile error. This makes orphaned
//! contracts self-detecting without a test registry. Macros are deliberately avoided so
//! that each runner is independently discoverable by IDEs and `--exact` filtering.

#![cfg(all(feature = "wire-mock", feature = "default-client"))]

// This inline module form selectively includes only the `client` submodule from
// `tests/common/`. Using `mod common;` (which goes through `common/mod.rs`) would
// also pull in the `h2` and `tls` submodules, generating dead-code warnings under
// `--all-features` since those modules contain items only the h2 and TLS tests use.
mod common {
    pub(crate) mod client;
}

use aws_smithy_async::assert_elapsed;
use aws_smithy_http_client::test_util::wire::connection::{
    BodyPlan, ConnectionCloseReason, ConnectionEvent, ConnectionId, ConnectionScript,
    ConnectionTestHarness, EndpointPlan, HarnessError, Http1Response, Http1Script, ManualGate,
    SocketScript,
};
use aws_smithy_http_client::Builder;
use aws_smithy_runtime_api::client::connection::{
    CaptureSmithyConnection, ConnectionMetadata as SmithyConnectionMetadata,
};
use aws_smithy_runtime_api::client::http::{
    HttpConnectorSettings, SharedHttpClient, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::retry::ErrorKind;
use common::client as test_client;
use common::client::{BackendConfig, HyperUtilLegacyPool};
use http_body_util::BodyExt;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

const IP1: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

trait HttpClientBackend {
    fn build(&self, config: BackendConfig) -> SharedHttpClient;
}

impl HttpClientBackend for HyperUtilLegacyPool {
    fn build(&self, config: BackendConfig) -> SharedHttpClient {
        let mut builder = Builder::new();
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.pool_idle_timeout(pool_idle_timeout);
        }
        builder.build_http()
    }
}

fn request_with_body(url: &str, body: &[u8]) -> HttpRequest {
    let mut request = HttpRequest::new(SdkBody::from(body.to_vec()));
    request.set_method("POST").expect("valid HTTP method");
    request.set_uri(url).expect("valid HTTP URI");
    request
        .headers_mut()
        .insert("content-length", body.len().to_string());
    request
}

async fn get_and_collect_with_capture(
    connector: &SharedHttpConnector,
    url: &str,
) -> (u16, Vec<u8>, SmithyConnectionMetadata) {
    let capture = CaptureSmithyConnection::new();
    let mut request = HttpRequest::get(url).expect("valid HTTP request");
    request.add_extension(capture.clone());
    let (status, body) = test_client::send_and_collect(connector, request).await;
    let metadata = capture
        .get()
        .expect("CaptureSmithyConnection should contain connection metadata");
    (status, body, metadata)
}

/// Drops the client-side handles, then shuts the harness down and reports failures.
///
/// The drop must come first. Shutdown cancels connection tasks promptly, so a script
/// parked in `await_client_close()` stops reading as soon as the signal arrives -- and
/// that read is what detects bytes the client should never have sent, such as a
/// pipelined request. Shutting down while the client still holds pooled connections
/// cancels the read before it observes them, and the harness reports success.
async fn shutdown_harness(
    harness: ConnectionTestHarness,
    connector: SharedHttpConnector,
    client: SharedHttpClient,
) -> Result<(), HarnessError> {
    drop(connector);
    drop(client);
    harness.shutdown().await
}

fn http1_request_connection_ids(harness: &ConnectionTestHarness) -> Vec<ConnectionId> {
    harness
        .events()
        .into_iter()
        .filter_map(|event| match event {
            ConnectionEvent::Http1Request { connection_id, .. } => Some(connection_id),
            _ => None,
        })
        .collect()
}

/// Asserts the error was caused by the peer resetting the connection.
///
/// A read that encounters a TCP RST reports `ECONNRESET` on Unix and
/// `WSAECONNRESET` on Windows, both of which map to
/// [`std::io::ErrorKind::ConnectionReset`], so this exact-kind check is portable
/// across every target the suite runs on.
#[track_caller]
fn assert_is_connection_reset(err: &(dyn std::error::Error + 'static)) {
    assert_io_error_kind(err, std::io::ErrorKind::ConnectionReset);
}

/// Asserts the error was caused by the response body ending before its declared
/// length was delivered.
///
/// Hyper's length-delimited body decoder reports a short read as
/// [`std::io::ErrorKind::UnexpectedEof`], which distinguishes a clean server
/// close mid-body from a connection reset.
#[track_caller]
fn assert_is_body_truncation(err: &(dyn std::error::Error + 'static)) {
    assert_io_error_kind(err, std::io::ErrorKind::UnexpectedEof);
}

#[track_caller]
fn assert_io_error_kind(err: &(dyn std::error::Error + 'static), expected: std::io::ErrorKind) {
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(cause) = current {
        if let Some(io_err) = cause.downcast_ref::<std::io::Error>() {
            if io_err.kind() == expected {
                return;
            }
        }
        current = cause.source();
    }
    panic!(
        "expected a {expected:?} io::Error in the source chain, got: {err}\n\
         sources: {}",
        error_chain_display(err)
    );
}

/// Renders an error and its `source()` chain as a single ` -> `-joined line for
/// assertion failure messages.
fn error_chain_display(err: &(dyn std::error::Error + 'static)) -> String {
    let mut chain = String::new();
    let mut current: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(cause) = current {
        if !chain.is_empty() {
            chain.push_str(" -> ");
        }
        chain.push_str(&cause.to_string());
        current = cause.source();
    }
    chain
}

mod reuse_and_lifecycle {
    use super::*;

    /// Fully consuming a response returns its H1 connection to the origin pool for reuse.
    async fn fully_consumed_responses_reuse_connection(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                Http1Script::responses([
                    Http1Response::ok().body("first"),
                    Http1Response::ok().body("second"),
                    Http1Response::ok().body("third"),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        for expected in [b"first".as_slice(), b"second", b"third"] {
            let (status, body) =
                test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
            assert_eq!(status, 200);
            assert_eq!(body, expected);
        }

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 3);
        assert!(
            connection_ids
                .iter()
                .all(|connection_id| *connection_id == connection_ids[0]),
            "fully consumed responses should reuse one connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_fully_consumed_responses_reuse_connection_with_hyper_util_legacy_pool() {
        fully_consumed_responses_reuse_connection(&HyperUtilLegacyPool).await;
    }

    /// An idle pooled connection is closed after its configured timeout and then replaced.
    async fn idle_connection_is_evicted_after_timeout(backend: &dyn HttpClientBackend) {
        let idle_timeout = Duration::from_millis(100);
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([Http1Response::ok().body("first")]),
                    Http1Script::responses([Http1Response::ok().body("second")]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig {
            pool_idle_timeout: Some(idle_timeout),
        });
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));
        let first_connection = http1_request_connection_ids(&harness)[0];

        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ClientClosed,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the client should close the evicted idle connection");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a connection evicted by the idle timeout must not be reused"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_idle_connection_is_evicted_after_timeout_with_hyper_util_legacy_pool() {
        idle_connection_is_evicted_after_timeout(&HyperUtilLegacyPool).await;
    }

    /// Idle eviction does not interrupt a response body that is still being consumed.
    async fn active_response_body_survives_idle_timeout(backend: &dyn HttpClientBackend) {
        let idle_timeout = Duration::from_millis(100);
        let body_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                Http1Script::responses([
                    Http1Response::ok().body_plan(BodyPlan::split_at_gate(
                        "first-",
                        body_gate.waiter(),
                        "body",
                    )),
                    Http1Response::ok().body("second"),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig {
            pool_idle_timeout: Some(idle_timeout),
        });
        let connector = test_client::connector(&client);

        let first_response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("first request should return response headers");

        body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the first response should reach its body gate");

        tokio::time::sleep(idle_timeout * 3).await;
        body_gate.release();
        let (status, body) = test_client::collect_response(first_response).await;
        assert_eq!((status, body.as_slice()), (200, b"first-body".as_slice()));

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "an active response body must survive the pool idle timeout"
        );
        // No accept-count assertion here. Returning a gated body's connection to the
        // pool goes through hyper-util's spawned `on_idle` task, so the pooled entry may
        // not be visible yet when request 2 checks out. hyper-util then races a
        // speculative connect against the checkout, which can accept a second socket
        // even though request 2 is ultimately served on the first connection. The
        // connection-ID equality above is the contract; the accept count is not.

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_active_response_body_survives_idle_timeout_with_hyper_util_legacy_pool() {
        active_response_body_survives_idle_timeout(&HyperUtilLegacyPool).await;
    }

    /// A held H1 response keeps its connection checked out, so another request opens a second.
    async fn held_response_body_allows_second_connection(backend: &dyn HttpClientBackend) {
        let body_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([Http1Response::ok()
                        .body_plan(BodyPlan::split_at_gate("held-", body_gate.waiter(), "body"))]),
                    Http1Script::responses([Http1Response::ok().body("second")]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let first_response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("first request should return response headers");

        body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the first response should reach its body gate");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a held H1 response body should make a second connection available"
        );

        body_gate.release();
        let (status, body) = test_client::collect_response(first_response).await;
        assert_eq!((status, body.as_slice()), (200, b"held-body".as_slice()));

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_held_response_body_allows_second_connection_with_hyper_util_legacy_pool() {
        held_response_body_allows_second_connection(&HyperUtilLegacyPool).await;
    }

    /// Pins an opportunistic optimization in the hyper-util legacy pool: when the
    /// chunk terminator is already buffered in the same TCP segment as the response
    /// head, hyper's single non-blocking drain poll reaches `ChunkedState::End` and
    /// returns the connection to the pool. This is NOT a requirement a replacement
    /// pool must satisfy; the genuine backend-neutral contract is its sibling test
    /// `dropping_unavailable_response_remainder_retires_connection`.
    async fn hyper_drains_a_fully_buffered_chunk_terminator_on_drop(
        backend: &dyn HttpClientBackend,
    ) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    SocketScript::new()
                        .read_http1_request()
                        // The head, chunk data, and terminator must arrive in one TCP
                        // segment. Hyper's Dispatcher makes exactly one non-blocking
                        // poll_read_body attempt on drop (in `Conn::poll_drain_or_close_read`);
                        // it only reclaims the connection if that single poll reaches
                        // ChunkedState::End.
                        .write_all(
                            b"HTTP/1.1 200 OK\r\n\
                              Transfer-Encoding: chunked\r\n\
                              Connection: keep-alive\r\n\
                              \r\n\
                              5\r\nfirst\r\n0\r\n\r\n",
                        )
                        .read_http1_request()
                        .write_all(
                            b"HTTP/1.1 200 OK\r\n\
                              Content-Length: 6\r\n\
                              Connection: keep-alive\r\n\
                              \r\n\
                              second",
                        )
                        .await_client_close(),
                    SocketScript::new().await_client_close(),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut first_response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("first request should succeed");
        assert_eq!(first_response.status().as_u16(), 200);
        let frame = first_response
            .body_mut()
            .frame()
            .await
            .expect("first response should contain a data frame")
            .expect("first response frame should be readable");
        assert_eq!(
            frame
                .into_data()
                .expect("first response frame should contain data"),
            b"first".as_slice()
        );
        drop(first_response);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "Hyper should drain the buffered chunk terminator and reuse the connection"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_hyper_drains_a_fully_buffered_chunk_terminator_on_drop_with_hyper_util_legacy_pool(
    ) {
        hyper_drains_a_fully_buffered_chunk_terminator_on_drop(&HyperUtilLegacyPool).await;
    }

    /// Dropping a response whose declared remainder is unavailable retires the connection
    /// instead of returning it to the pool.
    async fn dropping_unavailable_response_remainder_retires_connection(
        backend: &dyn HttpClientBackend,
    ) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 10\r\n\
                                  Connection: keep-alive\r\n\
                                  \r\n\
                                  first",
                            )
                            .await_client_close(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("second")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut first_response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("first request should return response headers");
        assert_eq!(first_response.status().as_u16(), 200);
        let frame = first_response
            .body_mut()
            .frame()
            .await
            .expect("first response should contain a partial data frame")
            .expect("first response frame should be readable");
        assert_eq!(
            frame
                .into_data()
                .expect("first response frame should contain data"),
            b"first".as_slice()
        );
        let first_connection = http1_request_connection_ids(&harness)[0];
        drop(first_response);

        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ClientClosed,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("dropping the incomplete body should close the connection");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a connection with an unavailable response remainder must be replaced"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_dropping_unavailable_response_remainder_retires_connection_with_hyper_util_legacy_pool(
    ) {
        dropping_unavailable_response_remainder_retires_connection(&HyperUtilLegacyPool).await;
    }

    /// A server-closed idle keep-alive connection is replaced before the next request.
    async fn stale_idle_connection_is_replaced(backend: &dyn HttpClientBackend) {
        let close_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 5\r\n\
                                  Connection: keep-alive\r\n\
                                  \r\n\
                                  first",
                            )
                            .wait(close_gate.waiter())
                            .close(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("second")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));
        close_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the server should be ready to close the idle connection");
        let first_connection = http1_request_connection_ids(&harness)[0];

        close_gate.release();
        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ScriptCompleted,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the server should close the first connection");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a server-closed idle connection must be replaced"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_stale_idle_connection_is_replaced_with_hyper_util_legacy_pool() {
        stale_idle_connection_is_replaced(&HyperUtilLegacyPool).await;
    }

    /// A server close that the pool has not yet observed must not surface an error: the
    /// next request succeeds on a fresh connection.
    ///
    /// This is the un-synchronized counterpart to `stale_idle_connection_is_replaced`,
    /// which waits for the close to be observed before issuing the second request. Here
    /// the second request is issued immediately, so a pool may satisfy the contract by
    /// either of two routes:
    ///
    /// 1. It notices the closed socket during checkout and connects instead.
    /// 2. It checks the socket out, the write fails before any request byte reaches the
    ///    wire, and it retries the unstarted request on a new connection. This is
    ///    hyper-util's `retry_canceled_requests` behavior, which only retries when the
    ///    connection was reused -- a failure on a fresh connection propagates instead.
    ///
    /// Which route runs depends on whether the client's connection task has processed the
    /// peer's FIN yet, which is not observable from outside the pool, so the assertions
    /// deliberately cover only the caller-visible outcome. A replacement pool is free to
    /// take either route.
    async fn request_on_an_unobserved_stale_connection_still_succeeds(
        backend: &dyn HttpClientBackend,
    ) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 5\r\n\
                                  Connection: keep-alive\r\n\
                                  \r\n\
                                  first",
                            )
                            // Close as soon as the response is written, without
                            // waiting for the client to observe it.
                            .close(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("second")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!(
            (status, body.as_slice()),
            (200, b"second".as_slice()),
            "a stale pooled connection must not fail the next request"
        );

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "the second request must be served on a replacement connection"
        );
        // Exactly one replacement socket, whichever route ran. A third accept would also
        // exhaust the endpoint plan, which `shutdown` reports as a harness failure.
        assert_eq!(harness.tcp_accepted_count(), 2);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_request_on_an_unobserved_stale_connection_still_succeeds_with_hyper_util_legacy_pool(
    ) {
        request_on_an_unobserved_stale_connection_still_succeeds(&HyperUtilLegacyPool).await;
    }

    /// A response carrying `Connection: close` prevents subsequent reuse of its connection.
    async fn connection_close_response_is_not_reused(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 7\r\n\
                                  Connection: close\r\n\
                                  \r\n\
                                  closing",
                            )
                            .await_client_close(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("fresh")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"closing".as_slice()));
        let first_connection = http1_request_connection_ids(&harness)[0];
        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ClientClosed,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the client should close a connection marked Connection: close");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"fresh".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a Connection: close response must not be reused"
        );
        assert_eq!(harness.tcp_accepted_count(), 2);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_connection_close_response_is_not_reused_with_hyper_util_legacy_pool() {
        connection_close_response_is_not_reused(&HyperUtilLegacyPool).await;
    }
}

mod routing_and_status {
    use super::*;

    /// A direct request uses origin-form for its target and sends the URI authority in `Host`.
    async fn direct_request_uses_origin_form_and_host_header(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                Http1Script::responses([Http1Response::ok().body("ok")]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);
        let url = format!(
            "{}/some/path?key=value",
            harness.endpoint_url().trim_end_matches('/')
        );

        let (status, body) = test_client::get_and_collect(&connector, &url).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));

        let requests = harness.http_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].0, "/some/path?key=value");
        let expected_host = format!("127.0.0.1:{}", harness.port());
        assert_eq!(requests[0].1.as_deref(), Some(expected_host.as_str()));

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_direct_request_uses_origin_form_and_host_header_with_hyper_util_legacy_pool() {
        direct_request_uses_origin_form_and_host_header(&HyperUtilLegacyPool).await;
    }

    /// Connections are pooled by origin authority even when two origins reach the same endpoint.
    async fn different_origins_do_not_share_connections(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([
                        Http1Response::ok().body("ip-a"),
                        Http1Response::ok().body("ip-b"),
                    ]),
                    Http1Script::responses([
                        Http1Response::ok().body("localhost-a"),
                        Http1Response::ok().body("localhost-b"),
                    ]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);
        let ip_url = harness.endpoint_url();
        let localhost_url = format!("http://localhost:{}/", harness.port());

        for (url, expected) in [
            (&ip_url, b"ip-a".as_slice()),
            (&localhost_url, b"localhost-a".as_slice()),
            (&ip_url, b"ip-b".as_slice()),
            (&localhost_url, b"localhost-b".as_slice()),
        ] {
            let (status, body) = test_client::get_and_collect(&connector, url).await;
            assert_eq!(status, 200);
            assert_eq!(body, expected);
        }

        let requests = harness
            .events()
            .into_iter()
            .filter_map(|event| match event {
                ConnectionEvent::Http1Request {
                    connection_id,
                    host,
                    ..
                } => Some((connection_id, host)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(requests.len(), 4);
        assert_eq!(requests[0].1, Some(format!("127.0.0.1:{}", harness.port())));
        assert_eq!(requests[1].1, Some(format!("localhost:{}", harness.port())));
        assert_eq!(requests[2].1, requests[0].1);
        assert_eq!(requests[3].1, requests[1].1);
        assert_eq!(requests[0].0, requests[2].0);
        assert_eq!(requests[1].0, requests[3].0);
        assert_ne!(
            requests[0].0, requests[1].0,
            "distinct authorities resolving to one endpoint must not share an H1 connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 2);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_different_origins_do_not_share_connections_with_hyper_util_legacy_pool() {
        different_origins_do_not_share_connections(&HyperUtilLegacyPool).await;
    }

    /// An HTTP server-error status does not by itself make the underlying connection unusable.
    async fn raw_server_error_response_does_not_poison_connection(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                Http1Script::responses([
                    Http1Response::new(503).body("unavailable"),
                    Http1Response::ok().body("recovered"),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (503, b"unavailable".as_slice()));

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"recovered".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "a raw server error response must not poison the connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_raw_server_error_response_does_not_poison_connection_with_hyper_util_legacy_pool()
    {
        raw_server_error_response_does_not_poison_connection(&HyperUtilLegacyPool).await;
    }
}

mod connection_metadata {
    use super::*;

    /// Captured metadata reports the socket addresses, and poisoning it prevents connection
    /// reuse.
    async fn captured_connection_addresses_and_poison_prevent_reuse(
        backend: &dyn HttpClientBackend,
    ) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([Http1Response::ok().body("first")]),
                    Http1Script::responses([Http1Response::ok().body("second")]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body, metadata) =
            get_and_collect_with_capture(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));
        assert_eq!(
            metadata.remote_addr(),
            Some(harness.endpoint(0).expect("first endpoint").addr())
        );
        let local_addr = metadata
            .local_addr()
            .expect("direct connection should include its local address");
        assert_eq!(local_addr.ip(), IP1);
        assert_ne!(local_addr.port(), 0);

        metadata.poison();

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "poisoned connection metadata must prevent connection reuse"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_captured_connection_addresses_and_poison_prevent_reuse_with_hyper_util_legacy_pool(
    ) {
        captured_connection_addresses_and_poison_prevent_reuse(&HyperUtilLegacyPool).await;
    }

    /// Poisoning an active connection lets its current body complete but prevents later reuse.
    async fn poisoning_active_connection_allows_body_completion_and_prevents_reuse(
        backend: &dyn HttpClientBackend,
    ) {
        let body_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([Http1Response::ok().body_plan(
                        BodyPlan::split_at_gate("first-", body_gate.waiter(), "body"),
                    )]),
                    Http1Script::responses([Http1Response::ok().body("second")]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);
        let capture = CaptureSmithyConnection::new();
        let mut request = HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request");
        request.add_extension(capture.clone());

        let first_response = test_client::send_request(&connector, request)
            .await
            .expect("first request should return response headers");

        body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the active response should reach its body gate");
        let first_connection = http1_request_connection_ids(&harness)[0];
        capture
            .get()
            .expect("active request should expose connection metadata")
            .poison();

        body_gate.release();
        let (status, body) = test_client::collect_response(first_response).await;
        assert_eq!((status, body.as_slice()), (200, b"first-body".as_slice()));
        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ClientClosed,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the poisoned connection should retire after its active body completes");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));
        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "a connection poisoned while active must not be reused"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_poisoning_active_connection_allows_body_completion_and_prevents_reuse_with_hyper_util_legacy_pool(
    ) {
        poisoning_active_connection_allows_body_completion_and_prevents_reuse(&HyperUtilLegacyPool)
            .await;
    }

    /// Capturing and dropping connection metadata without poisoning it does not affect reuse.
    async fn captured_connection_without_poison_is_reused(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                Http1Script::responses([
                    Http1Response::ok().body("first"),
                    Http1Response::ok().body("second"),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body, metadata) =
            get_and_collect_with_capture(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));
        drop(metadata);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "capturing metadata without poisoning should permit reuse"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_captured_connection_without_poison_is_reused_with_hyper_util_legacy_pool() {
        captured_connection_without_poison_is_reused(&HyperUtilLegacyPool).await;
    }
}

mod failures_and_timeouts {
    use super::*;

    /// A TCP reset immediately after accept is reported as an I/O connector error.
    async fn reset_on_accept_is_io_error(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(IP1, SocketScript::new().reset())
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let error = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect_err("reset on accept should fail the request");
        assert!(error.is_io(), "expected ConnectorError::io, got {error:?}");

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_reset_on_accept_is_io_error_with_hyper_util_legacy_pool() {
        reset_on_accept_is_io_error(&HyperUtilLegacyPool).await;
    }

    /// A reset after the complete request but before response headers is an I/O connector error.
    async fn reset_after_complete_request_is_io_error(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(IP1, SocketScript::new().read_http1_request().reset())
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);
        let request_body = b"complete request body";

        let error = test_client::send_request(
            &connector,
            request_with_body(&harness.endpoint_url(), request_body),
        )
        .await
        .expect_err("reset after the request should fail before response headers");
        assert!(error.is_io(), "expected ConnectorError::io, got {error:?}");

        let requests = harness.events();
        assert!(
            requests.iter().any(|event| matches!(
                event,
                ConnectionEvent::Http1Request { method, .. } if method == "POST"
            )),
            "the harness must receive the complete framed request before resetting"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_reset_after_complete_request_is_io_error_with_hyper_util_legacy_pool() {
        reset_after_complete_request_is_io_error(&HyperUtilLegacyPool).await;
    }

    /// A reset after response headers preserves the response and fails only its body
    /// with a connection-reset I/O error; the reset connection is retired from the pool.
    async fn reset_during_response_body_fails_body_only(backend: &dyn HttpClientBackend) {
        let reset_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 10\r\n\
                                  Connection: keep-alive\r\n\
                                  \r\n\
                                  first",
                            )
                            .wait(reset_gate.waiter())
                            .reset(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("after-reset")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("response headers should complete before the reset");
        assert_eq!(response.status().as_u16(), 200);
        reset_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the response should reach its reset gate");
        let frame = response
            .body_mut()
            .frame()
            .await
            .expect("response should contain a partial data frame")
            .expect("partial response frame should be readable");
        assert_eq!(
            frame
                .into_data()
                .expect("partial response frame should contain data"),
            b"first".as_slice()
        );

        reset_gate.release();
        let body_error = tokio::time::timeout(test_client::WAIT, response.into_body().collect())
            .await
            .expect("response body should fail within the outer deadline")
            .expect_err("reset should fail collection of the remaining response body");
        assert_is_connection_reset(&*body_error);

        let first_connection = http1_request_connection_ids(&harness)[0];
        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::Reset,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the harness should record a Reset close for the first connection");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"after-reset".as_slice()));
        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "the reset connection must be retired, not returned to the pool"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_reset_during_response_body_fails_body_only_with_hyper_util_legacy_pool() {
        reset_during_response_body_fails_body_only(&HyperUtilLegacyPool).await;
    }

    /// A clean EOF (server half-close) after response headers preserves the response and
    /// fails only its body with a truncation error, not a connection-reset; the closed
    /// connection is retired from the pool.
    ///
    /// The script uses `shutdown_write()` rather than `close()` so the EOF is delivered by
    /// a FIN with no pending unread data, which cannot be escalated to an RST by the
    /// kernel. Contrast `reset_during_response_body_fails_body_only`, which asserts the
    /// reset classification.
    async fn clean_eof_during_response_body_fails_body_only(backend: &dyn HttpClientBackend) {
        let close_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    ConnectionScript::socket(
                        SocketScript::new()
                            .read_http1_request()
                            .write_all(
                                b"HTTP/1.1 200 OK\r\n\
                                  Content-Length: 10\r\n\
                                  Connection: keep-alive\r\n\
                                  \r\n\
                                  first",
                            )
                            .wait(close_gate.waiter())
                            .shutdown_write()
                            .await_client_close(),
                    ),
                    ConnectionScript::http1(Http1Script::responses([
                        Http1Response::ok().body("after-close")
                    ])),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("response headers should complete before close");
        assert_eq!(response.status().as_u16(), 200);
        close_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the response should reach its close gate");
        let frame = response
            .body_mut()
            .frame()
            .await
            .expect("response should contain a partial data frame")
            .expect("partial response frame should be readable");
        assert_eq!(
            frame
                .into_data()
                .expect("partial response frame should contain data"),
            b"first".as_slice()
        );

        close_gate.release();
        let body_error = tokio::time::timeout(test_client::WAIT, response.into_body().collect())
            .await
            .expect("response body should fail within the outer deadline")
            .expect_err("close should fail collection of the remaining response body");
        assert_is_body_truncation(&*body_error);

        let first_connection = http1_request_connection_ids(&harness)[0];
        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ClientClosed,
                    } if *connection_id == first_connection
                )
            })
            .await
            .expect("the harness should record the client closing the truncated connection");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"after-close".as_slice()));
        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(
            connection_ids[0], connection_ids[1],
            "the closed connection must be retired, not returned to the pool"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_clean_eof_during_response_body_fails_body_only_with_hyper_util_legacy_pool() {
        clean_eof_during_response_body_fails_body_only(&HyperUtilLegacyPool).await;
    }

    /// A clean EOF before response headers is classified as a transient non-I/O error.
    async fn clean_eof_before_response_is_transient_other(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                SocketScript::new()
                    .read_http1_request()
                    .shutdown_write()
                    .await_client_close(),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let error = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect_err("clean EOF before response headers should fail the request");
        assert!(
            error.is_other(),
            "expected ConnectorError::other, got {error:?}"
        );
        assert_eq!(error.as_other(), Some(ErrorKind::TransientError));

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_clean_eof_before_response_is_transient_other_with_hyper_util_legacy_pool() {
        clean_eof_before_response_is_transient_other(&HyperUtilLegacyPool).await;
    }

    /// Exceeding the configured response-read deadline is reported as a timeout.
    async fn read_timeout_is_timeout_error(backend: &dyn HttpClientBackend) {
        let read_timeout = Duration::from_millis(250);
        let silent_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                SocketScript::new()
                    .read_http1_request()
                    .wait(silent_gate.waiter()),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector_with_settings(
            &client,
            HttpConnectorSettings::builder()
                .read_timeout(read_timeout)
                .build(),
        );
        let url = harness.endpoint_url();
        let start = tokio::time::Instant::now();
        let request_task = tokio::spawn({
            let connector = connector.clone();
            async move {
                test_client::send_request(
                    &connector,
                    HttpRequest::get(url).expect("valid HTTP request"),
                )
                .await
            }
        });

        silent_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the server should receive the request and remain silent");
        let error = tokio::time::timeout(test_client::WAIT, request_task)
            .await
            .expect("request should finish within the outer deadline")
            .expect("request task should not panic")
            .expect_err("the read timeout should fail the request");
        assert!(
            error.is_timeout(),
            "expected ConnectorError::timeout, got {error:?}"
        );
        // The timeout must fire at the configured deadline, not merely before the outer
        // 5s test deadline. The margin absorbs timer and scheduling jitter on a loaded
        // CI host; the request itself does no work, so the deadline dominates.
        assert_elapsed!(start, read_timeout, Duration::from_millis(250));

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_read_timeout_is_timeout_error_with_hyper_util_legacy_pool() {
        read_timeout_is_timeout_error(&HyperUtilLegacyPool).await;
    }
}

mod protocol_edge_cases {
    use super::*;

    /// A HEAD response advertises a body length via Content-Length but transmits no body.
    /// The client must not attempt to read the advertised bytes; the connection must
    /// remain reusable for a subsequent request.
    ///
    /// The typed `Http1Response` API always writes a body matching Content-Length, so this
    /// test uses raw `SocketScript` bytes to emit a HEAD response with
    /// `Content-Length: 100` and zero body bytes.
    async fn head_response_with_content_length_does_not_desync_connection(
        backend: &dyn HttpClientBackend,
    ) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                SocketScript::new()
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 200 OK\r\n\
                          Content-Length: 100\r\n\
                          Connection: keep-alive\r\n\
                          \r\n",
                    )
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 200 OK\r\n\
                          Content-Length: 5\r\n\
                          Connection: keep-alive\r\n\
                          \r\n\
                          hello",
                    )
                    .await_client_close(),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut head_request = HttpRequest::new(SdkBody::empty());
        head_request.set_method("HEAD").expect("valid HTTP method");
        head_request
            .set_uri(harness.endpoint_url())
            .expect("valid HTTP URI");
        let (status, body) = test_client::send_and_collect(&connector, head_request).await;
        assert_eq!(status, 200);
        assert!(
            body.is_empty(),
            "HEAD response must have an empty body, got {} bytes",
            body.len()
        );

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!(status, 200);
        assert_eq!(body, b"hello");

        let methods = harness
            .events()
            .into_iter()
            .filter_map(|event| match event {
                ConnectionEvent::Http1Request { method, .. } => Some(method),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(methods, ["HEAD", "GET"]);

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "a HEAD response must not desync the connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_head_response_with_content_length_does_not_desync_connection_with_hyper_util_legacy_pool(
    ) {
        head_response_with_content_length_does_not_desync_connection(&HyperUtilLegacyPool).await;
    }

    /// A 204 No Content response has no body by definition, and a well-behaved server
    /// omits Content-Length entirely. The client must treat the response as complete and
    /// return the connection to the pool for reuse.
    async fn no_content_response_is_reused(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                SocketScript::new()
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 204 No Content\r\n\
                          Connection: keep-alive\r\n\
                          \r\n",
                    )
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 200 OK\r\n\
                          Content-Length: 6\r\n\
                          Connection: keep-alive\r\n\
                          \r\n\
                          second",
                    )
                    .await_client_close(),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!(status, 204);
        assert!(body.is_empty(), "a 204 response body must be empty");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "a 204 response must not retire the connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_no_content_response_is_reused_with_hyper_util_legacy_pool() {
        no_content_response_is_reused(&HyperUtilLegacyPool).await;
    }

    /// A 304 Not Modified response carries no body even when Content-Length is present
    /// echoing the original resource size. The connection must remain reusable.
    async fn not_modified_response_is_reused(backend: &dyn HttpClientBackend) {
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                SocketScript::new()
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 304 Not Modified\r\n\
                          Content-Length: 500\r\n\
                          Connection: keep-alive\r\n\
                          \r\n",
                    )
                    .read_http1_request()
                    .write_all(
                        b"HTTP/1.1 200 OK\r\n\
                          Content-Length: 5\r\n\
                          Connection: keep-alive\r\n\
                          \r\n\
                          third",
                    )
                    .await_client_close(),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!(status, 304);
        assert!(body.is_empty(), "a 304 response body must be empty");

        let (status, body) =
            test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
        assert_eq!((status, body.as_slice()), (200, b"third".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_eq!(
            connection_ids[0], connection_ids[1],
            "a 304 response must not retire the connection"
        );
        assert_eq!(harness.tcp_accepted_count(), 1);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_not_modified_response_is_reused_with_hyper_util_legacy_pool() {
        not_modified_response_is_reused(&HyperUtilLegacyPool).await;
    }
}

mod concurrency {
    use super::*;

    /// Concurrent HTTP/1.1 requests to one origin each get their own connection.
    ///
    /// HTTP/1.1 has no multiplexing, so each in-flight request needs its own connection.
    /// Contrast `concurrent_cold_start_converges_on_one_h2_connection` in the HTTP/2
    /// suite, where many streams share one connection.
    async fn concurrent_requests_open_distinct_connections(backend: &dyn HttpClientBackend) {
        let gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::repeat_n(
                    4,
                    Http1Script::responses([Http1Response::ok()
                        .body_plan(BodyPlan::split_at_gate("resp-", gate.waiter(), "body"))]),
                ),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let mut handles = Vec::new();
        for _ in 0..4 {
            let connector = connector.clone();
            let url = harness.endpoint_url();
            handles.push(tokio::spawn(async move {
                test_client::get_and_collect(&connector, &url).await
            }));
        }

        // All four requests are in flight once each has reached its body gate.
        gate.wait_for_arrivals(4, test_client::WAIT)
            .await
            .expect("all four requests should reach the body gate");

        let connection_ids = http1_request_connection_ids(&harness)
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(
            connection_ids.len(),
            4,
            "four concurrent H1 requests must each get a distinct connection"
        );

        gate.release();
        for handle in handles {
            let (status, body) = handle.await.expect("request task should not panic");
            assert_eq!((status, body.as_slice()), (200, b"resp-body".as_slice()));
        }

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_concurrent_requests_open_distinct_connections_with_hyper_util_legacy_pool() {
        concurrent_requests_open_distinct_connections(&HyperUtilLegacyPool).await;
    }

    // Not covered here: the idle cap, i.e. dropping a returning connection when the idle
    // list is already at `pool_max_idle_per_host`. The smithy `Builder` does not expose
    // that knob, so the behavior is unreachable through the public API. Add coverage when
    // it is exposed.
    //
    // Also not covered: handoff of a returning connection to a queued waiter. hyper-util
    // has no total-connection cap, so an extra concurrent request opens another connection
    // rather than queuing, and the handoff is not observable from outside the pool.

    /// A second request must not be written onto a connection whose response is still in
    /// flight; it opens its own connection instead.
    ///
    /// This pins the absence of HTTP/1.1 pipelining. The first response body is held at a
    /// gate, then a second request is issued: it must land on a different connection, and
    /// the first connection must still show exactly one request.
    ///
    /// Note that no assertion can prove bytes were never written to the first socket while
    /// it is parked at the gate -- the script performs no reads there, so pipelined bytes
    /// would sit unread in the kernel buffer. The `await_client_close` finish that
    /// `Http1Script` appends is what detects them: it fails the harness if the client sent
    /// anything more, and `shutdown` surfaces that failure.
    async fn request_is_not_written_before_the_previous_response_completes(
        backend: &dyn HttpClientBackend,
    ) {
        let body_gate = ManualGate::new();
        let harness = ConnectionTestHarness::builder()
            .endpoint(
                IP1,
                EndpointPlan::queue([
                    Http1Script::responses([Http1Response::ok()
                        .body_plan(BodyPlan::split_at_gate("slow-", body_gate.waiter(), "resp"))]),
                    Http1Script::responses([Http1Response::ok().body("fast")]),
                ]),
            )
            .build()
            .await
            .expect("harness should start");
        let client = backend.build(BackendConfig::default());
        let connector = test_client::connector(&client);

        let first_response = test_client::send_request(
            &connector,
            HttpRequest::get(harness.endpoint_url()).expect("valid HTTP request"),
        )
        .await
        .expect("first request should return response headers");
        assert_eq!(first_response.status().as_u16(), 200);

        body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("the first response body should reach its gate");
        let first_connection = http1_request_connection_ids(&harness)[0];

        let second_handle = tokio::spawn({
            let connector = connector.clone();
            let url = harness.endpoint_url();
            async move { test_client::get_and_collect(&connector, &url).await }
        });

        harness
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    ConnectionEvent::Http1Request { connection_id, .. }
                        if *connection_id != first_connection
                )
            })
            .await
            .expect("the second request should arrive on a different connection");

        // Still holding the gate: the first connection must show only its own request.
        let requests_on_first = http1_request_connection_ids(&harness)
            .into_iter()
            .filter(|connection_id| *connection_id == first_connection)
            .count();
        assert_eq!(
            requests_on_first, 1,
            "a second request must not be pipelined onto a connection with an \
             incomplete response"
        );

        body_gate.release();
        let (status, body) = test_client::collect_response(first_response).await;
        assert_eq!((status, body.as_slice()), (200, b"slow-resp".as_slice()));
        let (status, body) = second_handle.await.expect("request task should not panic");
        assert_eq!((status, body.as_slice()), (200, b"fast".as_slice()));

        let connection_ids = http1_request_connection_ids(&harness);
        assert_eq!(connection_ids.len(), 2);
        assert_ne!(connection_ids[0], connection_ids[1]);
        assert_eq!(harness.tcp_accepted_count(), 2);

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_request_is_not_written_before_the_previous_response_completes_with_hyper_util_legacy_pool(
    ) {
        request_is_not_written_before_the_previous_response_completes(&HyperUtilLegacyPool).await;
    }
}

// A DNS resolver can only be installed through `Builder::build_with_resolver`, which is
// available once a TLS provider is selected, so these tests need a TLS feature even though
// they speak plaintext HTTP. `CryptoMode::Ring` requires `rustls-ring` specifically.
#[cfg(feature = "rustls-ring")]
mod dns_resolution {
    use super::*;
    use aws_smithy_http_client::tls;

    /// A hostname with no DNS entry fails the request at resolution, before any TCP
    /// connection is attempted.
    async fn unresolvable_hostname_fails_before_connect() {
        // The endpoint is never connected to. It exists because a harness requires at
        // least one endpoint, and it supplies the port used to build the request URL.
        let harness = ConnectionTestHarness::builder()
            .endpoint(IP1, SocketScript::new().await_client_close())
            .build()
            .await
            .expect("harness should start");
        let client = Builder::new()
            .tls_provider(tls::Provider::Rustls(
                tls::rustls_provider::CryptoMode::Ring,
            ))
            .build_with_resolver(harness.dns_resolver());
        let connector = test_client::connector(&client);

        let url = format!("http://unknown.test:{}/", harness.port());
        let error = test_client::send_request(
            &connector,
            HttpRequest::get(&url).expect("valid HTTP request"),
        )
        .await
        .expect_err("an unresolvable hostname must fail the request");
        assert!(
            error.is_io(),
            "expected ConnectorError::io for a DNS failure, got {error:?}"
        );
        let chain = error_chain_display(&error);
        assert!(
            chain.contains("dns error"),
            "the error should identify DNS resolution as the cause, got: {chain}"
        );

        assert_eq!(
            harness.tcp_accepted_count(),
            0,
            "no TCP connection should be attempted when DNS resolution fails"
        );
        let lookups = harness
            .events()
            .into_iter()
            .filter(
                |event| matches!(event, ConnectionEvent::DnsLookup { hostname } if hostname == "unknown.test"),
            )
            .count();
        assert_eq!(
            lookups, 1,
            "the failed lookup should be recorded exactly once"
        );

        shutdown_harness(harness, connector, client)
            .await
            .expect("clean harness shutdown");
    }

    #[tokio::test]
    async fn test_unresolvable_hostname_fails_before_connect_with_hyper_util_legacy_pool() {
        unresolvable_hostname_fails_before_connect().await;
    }
}
