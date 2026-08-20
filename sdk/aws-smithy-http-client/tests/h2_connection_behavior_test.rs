/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/2 connection behavior contracts.
//!
//! Each contract is implementation-neutral and has an explicit runner for every
//! HTTP client backend expected to preserve that behavior.

#![cfg(all(feature = "wire-mock", feature = "rustls-aws-lc"))]

mod common;

use aws_smithy_http_client::test_util::wire::connection::{ConnectionCloseReason, ManualGate};
use aws_smithy_http_client::tls;
use aws_smithy_http_client::Builder;
use aws_smithy_runtime_api::client::connection::{
    CaptureSmithyConnection, ConnectionMetadata as SmithyConnectionMetadata,
};
use aws_smithy_runtime_api::client::http::{SharedHttpClient, SharedHttpConnector};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use bytes::Bytes;
use common::client as test_client;
use common::client::{BackendConfig, HyperUtilLegacyPool};
use common::h2::{
    H2BodyPlan, H2ConnectionId, H2ConnectionPlan, H2ConnectionScript, H2Event, H2Response,
    H2StreamScript, H2TestServer,
};
use common::tls as test_tls;
use h2::Reason;
use http_body_util::BodyExt;
use std::error::Error;

trait HttpsClientBackend {
    fn build_https(
        &self,
        config: BackendConfig,
        provider: tls::Provider,
        tls_context: tls::TlsContext,
    ) -> SharedHttpClient;
}

impl HttpsClientBackend for HyperUtilLegacyPool {
    fn build_https(
        &self,
        config: BackendConfig,
        provider: tls::Provider,
        tls_context: tls::TlsContext,
    ) -> SharedHttpClient {
        let mut builder = Builder::new();
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.pool_idle_timeout(pool_idle_timeout);
        }
        builder
            .tls_provider(provider)
            .tls_context(tls_context)
            .build_https()
    }
}

fn rustls_aws_lc() -> tls::Provider {
    tls::Provider::Rustls(tls::rustls_provider::CryptoMode::AwsLc)
}

fn h2_client_with_provider(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) -> SharedHttpClient {
    backend.build_https(
        BackendConfig::default(),
        provider,
        test_tls::server_tls_context(),
    )
}

fn h2_client(backend: &dyn HttpsClientBackend) -> SharedHttpClient {
    h2_client_with_provider(backend, rustls_aws_lc())
}

fn stream_connection_ids(server: &H2TestServer, path: &str) -> Vec<H2ConnectionId> {
    server
        .events()
        .into_iter()
        .filter_map(|event| match event {
            H2Event::StreamAccepted {
                connection_id,
                path: event_path,
                ..
            } if event_path == path => Some(connection_id),
            _ => None,
        })
        .collect()
}

fn single_stream_connection(server: &H2TestServer, path: &str) -> H2ConnectionId {
    let connection_ids = stream_connection_ids(server, path);
    assert_eq!(connection_ids.len(), 1, "expected one stream for {path}");
    connection_ids[0]
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

fn h2_error_reason(error: &(dyn Error + 'static)) -> Option<Reason> {
    let mut current = Some(error);
    while let Some(error) = current {
        if let Some(error) = error.downcast_ref::<h2::Error>() {
            return error.reason();
        }
        current = error.source();
    }
    None
}

mod reuse_and_multiplexing {
    use super::*;

    /// Sequential streams for one origin reuse an established H2 connection.
    async fn sequential_requests_reuse_connection(backend: &dyn HttpsClientBackend) {
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([H2ConnectionScript::new()
                .fallback(H2StreamScript::respond(H2Response::ok("hello")))]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        for request_number in 1..=5 {
            let (status, body) = test_client::get_and_collect(&connector, &server.url("/")).await;
            assert_eq!(status, 200, "request {request_number}");
            assert_eq!(body, b"hello", "request {request_number}");
        }

        assert_eq!(server.connection_count(), 1);
        assert_eq!(server.stream_count(), 5);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_sequential_requests_reuse_connection_with_hyper_util_legacy_pool() {
        sequential_requests_reuse_connection(&HyperUtilLegacyPool).await;
    }

    /// Concurrent requests multiplex as independent streams on a warmed H2 connection.
    async fn concurrent_requests_multiplex_on_warmed_connection(backend: &dyn HttpsClientBackend) {
        let body_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .route("/warm", H2StreamScript::respond(H2Response::ok("warm")))
            .route(
                "/concurrent",
                H2StreamScript::respond(H2Response::new(http_1x::StatusCode::OK).body(
                    H2BodyPlan::gated(Bytes::new(), body_gate.waiter(), "multiplexed"),
                )),
            );
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/warm")).await;
        assert_eq!((status, body.as_slice()), (200, b"warm".as_slice()));

        let mut requests = tokio::task::JoinSet::new();
        for _ in 0..4 {
            let connector = connector.clone();
            let url = server.url("/concurrent");
            requests.spawn(async move { test_client::get_and_collect(&connector, &url).await });
        }

        body_gate
            .wait_for_arrivals(4, test_client::WAIT)
            .await
            .expect("all concurrent H2 streams should reach their body gate");
        assert_eq!(server.connection_count(), 1);
        assert_eq!(server.stream_count(), 5);
        body_gate.release();

        while let Some(result) = requests.join_next().await {
            let (status, body) = result.expect("request task should not panic");
            assert_eq!((status, body.as_slice()), (200, b"multiplexed".as_slice()));
        }

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_concurrent_requests_multiplex_on_warmed_connection_with_hyper_util_legacy_pool() {
        concurrent_requests_multiplex_on_warmed_connection(&HyperUtilLegacyPool).await;
    }

    /// Concurrent cold-start requests converge on one established H2 connection even when
    /// connection attempts race.
    async fn concurrent_cold_start_converges_on_one_h2_connection(
        backend: &dyn HttpsClientBackend,
    ) {
        let body_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .allow_handshake_abandonment()
            .fallback(H2StreamScript::respond(
                H2Response::new(http_1x::StatusCode::OK).body(H2BodyPlan::gated(
                    Bytes::new(),
                    body_gate.waiter(),
                    "cold",
                )),
            ));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::unbounded(script))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);
        let mut requests = tokio::task::JoinSet::new();

        for _ in 0..4 {
            let connector = connector.clone();
            let url = server.url("/cold");
            requests.spawn(async move { test_client::get_and_collect(&connector, &url).await });
        }

        body_gate
            .wait_for_arrivals(4, test_client::WAIT)
            .await
            .expect("all cold-start H2 streams should reach their body gate");
        let ready_connections = server
            .events()
            .into_iter()
            .filter_map(|event| match event {
                H2Event::H2Ready { connection_id } => Some(connection_id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ready_connections.len(), 1);
        let stream_connections = stream_connection_ids(&server, "/cold");
        assert_eq!(stream_connections.len(), 4);
        assert!(
            stream_connections
                .iter()
                .all(|connection_id| *connection_id == ready_connections[0]),
            "all cold-start requests should converge on one established H2 connection"
        );
        body_gate.release();

        while let Some(result) = requests.join_next().await {
            let (status, body) = result.expect("request task should not panic");
            assert_eq!((status, body.as_slice()), (200, b"cold".as_slice()));
        }
        assert_eq!(server.stream_count(), 4);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_concurrent_cold_start_converges_on_one_h2_connection_with_hyper_util_legacy_pool()
    {
        concurrent_cold_start_converges_on_one_h2_connection(&HyperUtilLegacyPool).await;
    }
}

mod connection_metadata {
    use super::*;

    /// Poisoning captured H2 connection metadata moves later streams to a new connection.
    async fn poisoned_connection_is_not_reused(backend: &dyn HttpsClientBackend) {
        let script =
            H2ConnectionScript::new().fallback(H2StreamScript::respond(H2Response::ok("ok")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let (status, body, metadata) =
            get_and_collect_with_capture(&connector, &server.url("/first")).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));
        metadata.poison();

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/second")).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));

        let first_connection = single_stream_connection(&server, "/first");
        let second_connection = single_stream_connection(&server, "/second");
        assert_ne!(first_connection, second_connection);
        assert_eq!(server.connection_count(), 2);

        drop(metadata);
        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_poisoned_connection_is_not_reused_with_hyper_util_legacy_pool() {
        poisoned_connection_is_not_reused(&HyperUtilLegacyPool).await;
    }
}

mod stream_failures {
    use super::*;

    /// A peer reset fails only its H2 stream and leaves the connection available for reuse.
    async fn stream_reset_does_not_retire_connection(backend: &dyn HttpsClientBackend) {
        let reset_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .route(
                "/reset",
                H2StreamScript::respond(H2Response::new(http_1x::StatusCode::OK).body(
                    H2BodyPlan::reset_after("partial", reset_gate.waiter(), Reason::CANCEL),
                )),
            )
            .route("/ok", H2StreamScript::respond(H2Response::ok("reused")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let response = test_client::send_request(
            &connector,
            HttpRequest::get(server.url("/reset")).expect("valid HTTP request"),
        )
        .await
        .expect("response headers should succeed before the stream reset");
        assert_eq!(response.status().as_u16(), 200);
        let mut body = response.into_body();
        let frame = tokio::time::timeout(test_client::WAIT, body.frame())
            .await
            .expect("response body frame should arrive within the outer deadline")
            .expect("response body should contain a frame")
            .expect("response body frame should be readable");
        let data = frame
            .into_data()
            .expect("the first body frame should be data");
        assert_eq!(data, b"partial".as_slice());
        reset_gate.release();
        let error = body
            .collect()
            .await
            .expect_err("the reset response body should fail");
        assert_eq!(h2_error_reason(error.as_ref()), Some(Reason::CANCEL));

        let reset_event = server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ResetSent {
                        reason: Reason::CANCEL,
                        ..
                    }
                )
            })
            .await
            .expect("server should send the scripted stream reset");
        assert!(matches!(reset_event, H2Event::ResetSent { .. }));

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/ok")).await;
        assert_eq!((status, body.as_slice()), (200, b"reused".as_slice()));
        assert_eq!(
            single_stream_connection(&server, "/reset"),
            single_stream_connection(&server, "/ok")
        );
        assert_eq!(server.connection_count(), 1);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_stream_reset_does_not_retire_connection_with_hyper_util_legacy_pool() {
        stream_reset_does_not_retire_connection(&HyperUtilLegacyPool).await;
    }

    /// Dropping an incomplete response body cancels only that stream and permits reuse.
    async fn dropping_response_body_cancels_only_stream(backend: &dyn HttpsClientBackend) {
        let script = H2ConnectionScript::new()
            .route(
                "/drop",
                H2StreamScript::respond(
                    H2Response::new(http_1x::StatusCode::OK)
                        .body(H2BodyPlan::await_client_reset("partial", Reason::CANCEL)),
                ),
            )
            .route("/ok", H2StreamScript::respond(H2Response::ok("reused")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let response = test_client::send_request(
            &connector,
            HttpRequest::get(server.url("/drop")).expect("valid HTTP request"),
        )
        .await
        .expect("response headers should succeed");
        assert_eq!(response.status().as_u16(), 200);
        let mut body = response.into_body();
        let frame = tokio::time::timeout(test_client::WAIT, body.frame())
            .await
            .expect("response body frame should arrive within the outer deadline")
            .expect("response body should contain a frame")
            .expect("response body frame should be readable");
        let data = frame
            .into_data()
            .expect("the first body frame should be data");
        assert_eq!(data, b"partial".as_slice());
        drop(body);

        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ClientResetObserved {
                        reason: Reason::CANCEL,
                        ..
                    }
                )
            })
            .await
            .expect("dropping the body should reset only its H2 stream");

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/ok")).await;
        assert_eq!((status, body.as_slice()), (200, b"reused".as_slice()));
        assert_eq!(
            single_stream_connection(&server, "/drop"),
            single_stream_connection(&server, "/ok")
        );
        assert_eq!(server.connection_count(), 1);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_dropping_response_body_cancels_only_stream_with_hyper_util_legacy_pool() {
        dropping_response_body_cancels_only_stream(&HyperUtilLegacyPool).await;
    }
}

mod goaway_and_replacement {
    use super::*;

    /// Graceful GOAWAY drains an eligible in-flight stream while later streams use a
    /// replacement connection.
    async fn graceful_goaway_preserves_in_flight_stream_and_replaces_connection(
        backend: &dyn HttpsClientBackend,
    ) {
        let held_body_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .route(
                "/held",
                H2StreamScript::respond(H2Response::new(http_1x::StatusCode::OK).body(
                    H2BodyPlan::gated("held-", held_body_gate.waiter(), "complete"),
                )),
            )
            .route(
                "/after",
                H2StreamScript::respond(H2Response::ok("replacement")),
            );
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let held_response = test_client::send_request(
            &connector,
            HttpRequest::get(server.url("/held")).expect("valid HTTP request"),
        )
        .await
        .expect("held response headers should succeed");

        held_body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("held stream should reach its body gate");
        let original_connection = single_stream_connection(&server, "/held");
        let held_stream_id = server
            .events()
            .iter()
            .find_map(|event| match event {
                H2Event::StreamAccepted {
                    connection_id,
                    stream_id,
                    path,
                    ..
                } if *connection_id == original_connection && path == "/held" => Some(*stream_id),
                _ => None,
            })
            .expect("the /held stream should have been accepted");

        server
            .send_graceful_goaway(original_connection)
            .await
            .expect("graceful GOAWAY should start");
        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::GoAwaySent {
                        connection_id,
                        last_stream_id,
                        reason: Reason::NO_ERROR,
                    } if *connection_id == original_connection
                        && *last_stream_id == held_stream_id
                )
            })
            .await
            .expect("the final graceful GOAWAY should be flushed");

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/after")).await;
        assert_eq!((status, body.as_slice()), (200, b"replacement".as_slice()));
        let replacement_connection = single_stream_connection(&server, "/after");
        assert_ne!(original_connection, replacement_connection);
        // Match any close reason: the assertion is "no close happened yet", not "no specific
        // close happened." A more-specific pattern would weaken the negative check.
        assert!(!server.events().iter().any(|event| {
            matches!(
                event,
                H2Event::ConnectionClosed { connection_id, .. }
                    if *connection_id == original_connection
            )
        }));

        held_body_gate.release();
        let (status, body) = test_client::collect_response(held_response).await;
        assert_eq!(
            (status, body.as_slice()),
            (200, b"held-complete".as_slice())
        );
        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ConnectionClosed { connection_id, reason: ConnectionCloseReason::ClientClosed }
                        if *connection_id == original_connection
                )
            })
            .await
            .expect("the original connection should close after its held stream completes");

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_graceful_goaway_preserves_in_flight_stream_and_replaces_connection_with_hyper_util_legacy_pool(
    ) {
        graceful_goaway_preserves_in_flight_stream_and_replaces_connection(&HyperUtilLegacyPool)
            .await;
    }
}

#[cfg(feature = "s2n-tls")]
mod protocol_negotiation {
    use super::*;

    /// The s2n-tls provider negotiates `h2` with ALPN and reuses that connection.
    async fn s2n_negotiates_h2_and_reuses_connection(backend: &dyn HttpsClientBackend) {
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([H2ConnectionScript::new()
                .fallback(H2StreamScript::respond(H2Response::ok("s2n-h2")))]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client_with_provider(backend, tls::Provider::S2nTls);
        let connector = test_client::connector(&client);

        for request_number in 1..=3 {
            let (status, body) = test_client::get_and_collect(&connector, &server.url("/")).await;
            assert_eq!(status, 200, "request {request_number}");
            assert_eq!(body, b"s2n-h2", "request {request_number}");
        }

        let negotiated = server
            .events()
            .into_iter()
            .filter_map(|event| match event {
                H2Event::TlsNegotiated { alpn, .. } => Some(alpn),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(negotiated, vec![Some(b"h2".to_vec())]);
        assert_eq!(server.connection_count(), 1);
        assert_eq!(server.stream_count(), 3);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_s2n_negotiates_h2_and_reuses_connection_with_hyper_util_legacy_pool() {
        s2n_negotiates_h2_and_reuses_connection(&HyperUtilLegacyPool).await;
    }
}

mod idle_timeout {
    use super::*;
    use std::time::Duration;

    const IDLE_TIMEOUT: Duration = Duration::from_millis(100);

    fn client_with_idle_timeout(backend: &dyn HttpsClientBackend) -> SharedHttpClient {
        backend.build_https(
            BackendConfig {
                pool_idle_timeout: Some(IDLE_TIMEOUT),
            },
            rustls_aws_lc(),
            test_tls::server_tls_context(),
        )
    }

    /// An idle H2 connection is closed after its configured timeout and then replaced.
    async fn idle_connection_is_evicted_after_timeout(backend: &dyn HttpsClientBackend) {
        let script =
            H2ConnectionScript::new().fallback(H2StreamScript::respond(H2Response::ok("ok")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = client_with_idle_timeout(backend);
        let connector = test_client::connector(&client);

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/first")).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));
        let first_connection = single_stream_connection(&server, "/first");
        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ConnectionClosed { connection_id, reason: ConnectionCloseReason::ClientClosed }
                        if *connection_id == first_connection
                )
            })
            .await
            .expect("the idle H2 connection should close after its pool timeout");

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/second")).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));
        assert_ne!(
            first_connection,
            single_stream_connection(&server, "/second")
        );

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_idle_connection_is_evicted_after_timeout_with_hyper_util_legacy_pool() {
        idle_connection_is_evicted_after_timeout(&HyperUtilLegacyPool).await;
    }

    /// An active stream survives the idle deadline, but the connection is replaced after the
    /// stream completes.
    async fn active_stream_survives_idle_timeout_but_later_request_uses_replacement(
        backend: &dyn HttpsClientBackend,
    ) {
        let held_body_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .route(
                "/held",
                H2StreamScript::respond(H2Response::new(http_1x::StatusCode::OK).body(
                    H2BodyPlan::gated("held-", held_body_gate.waiter(), "complete"),
                )),
            )
            .route("/second", H2StreamScript::respond(H2Response::ok("second")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = client_with_idle_timeout(backend);
        let connector = test_client::connector(&client);

        let held_response = test_client::send_request(
            &connector,
            HttpRequest::get(server.url("/held")).expect("valid HTTP request"),
        )
        .await
        .expect("held response headers should succeed");

        held_body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("held stream should reach its body gate");
        let first_connection = single_stream_connection(&server, "/held");
        tokio::time::sleep(IDLE_TIMEOUT * 2).await;
        // Match any close reason: the assertion is "no close happened yet", not "no specific
        // close happened." A more-specific pattern would weaken the negative check.
        assert!(!server.events().iter().any(|event| {
            matches!(
                event,
                H2Event::ConnectionClosed { connection_id, .. }
                    if *connection_id == first_connection
            )
        }));

        held_body_gate.release();
        let (status, body) = test_client::collect_response(held_response).await;
        assert_eq!(
            (status, body.as_slice()),
            (200, b"held-complete".as_slice())
        );

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/second")).await;
        assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));
        let second_connection = single_stream_connection(&server, "/second");
        assert_ne!(first_connection, second_connection);

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_active_stream_survives_idle_timeout_but_later_request_uses_replacement_with_hyper_util_legacy_pool(
    ) {
        active_stream_survives_idle_timeout_but_later_request_uses_replacement(
            &HyperUtilLegacyPool,
        )
        .await;
    }
}
