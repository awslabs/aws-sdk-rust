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
#[path = "common/runtime.rs"]
mod runtime;

use aws_smithy_http_client::pool::{
    Client as PoolClient, ConnectionPool, ConnectionReuseScope, DriverSpawner, Partition,
    PartitionId,
};
use aws_smithy_http_client::test_util::wire::connection::{ConnectionCloseReason, ManualGate};
use aws_smithy_http_client::tls;
use aws_smithy_runtime_api::client::connection::{
    CaptureSmithyConnection, ConnectionMetadata as SmithyConnectionMetadata,
};
use aws_smithy_runtime_api::client::http::telemetry::{
    CaptureHttpAttemptTelemetry, ConnectionUsage,
};
use aws_smithy_runtime_api::client::http::{SharedHttpClient, SharedHttpConnector};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use bytes::Bytes;
use common::client as test_client;
use common::client::{
    BackendConfig, HttpsClientBackend, HyperUtilLegacyPool, PartitionedConnectionPool,
};
use common::h2::{
    H2BodyPlan, H2ConnectionId, H2ConnectionPlan, H2ConnectionScript, H2Event, H2Response,
    H2StreamScript, H2TestServer,
};
use common::tls as test_tls;
use h2::Reason;
use http_body_util::BodyExt;
use std::error::Error;

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
        test_tls::SERVER_IDENTITY.client_context(),
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

    #[tokio::test]
    async fn test_sequential_requests_reuse_connection_with_partitioned_pool() {
        sequential_requests_reuse_connection(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_concurrent_requests_multiplex_on_warmed_connection_with_partitioned_pool() {
        concurrent_requests_multiplex_on_warmed_connection(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_concurrent_cold_start_converges_on_one_h2_connection_with_partitioned_pool() {
        concurrent_cold_start_converges_on_one_h2_connection(&PartitionedConnectionPool).await;
    }
}

mod connection_metadata {
    use super::*;
    use aws_smithy_async::test_util::ManualTimeSource;
    use std::time::{Duration, UNIX_EPOCH};

    async fn capture_attempt(
        connector: &SharedHttpConnector,
        url: &str,
    ) -> aws_smithy_runtime_api::client::http::telemetry::HttpAttemptTelemetry {
        let capture = CaptureHttpAttemptTelemetry::new();
        let mut request = HttpRequest::get(url).expect("valid HTTP request");
        request.add_extension(capture.clone());
        let (status, body) = test_client::send_and_collect(connector, request).await;
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));
        capture.get()
    }

    async fn attempt_telemetry_matches_backend_capability(
        backend: &dyn HttpsClientBackend,
        captures_selection: bool,
    ) {
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([
                H2ConnectionScript::new().fallback(H2StreamScript::respond(H2Response::ok("ok")))
            ]))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(backend);
        let connector = test_client::connector(&client);

        let first = capture_attempt(&connector, &server.url("/first")).await;
        let second = capture_attempt(&connector, &server.url("/second")).await;
        assert!(first.connector_call_duration().is_some());
        assert!(second.connector_call_duration().is_some());

        if captures_selection {
            assert_eq!(
                first.acquisition().expect("first acquisition").usage(),
                ConnectionUsage::Fresh
            );
            assert_eq!(
                second.acquisition().expect("second acquisition").usage(),
                ConnectionUsage::Reused
            );
            let first_connection = first.connection().expect("first connection metadata");
            let second_connection = second.connection().expect("second connection metadata");
            assert_eq!(
                first_connection.connection_id(),
                second_connection.connection_id()
            );
            assert_eq!(
                first_connection.establishment(),
                second_connection.establishment(),
                "reuse must retain the establishment that created the connection"
            );
            assert!(first_connection.establishment().is_some());
        } else {
            assert!(first.acquisition().is_none());
            assert!(first.connection().is_none());
            assert!(second.acquisition().is_none());
            assert!(second.connection().is_none());
        }

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn test_attempt_telemetry_with_hyper_util_legacy_pool() {
        attempt_telemetry_matches_backend_capability(&HyperUtilLegacyPool, false).await;
    }

    #[tokio::test]
    async fn test_attempt_telemetry_with_partitioned_pool() {
        attempt_telemetry_matches_backend_capability(&PartitionedConnectionPool, true).await;
    }

    #[tokio::test]
    async fn h2_acquisition_ends_before_response_headers() {
        let response_gate = ManualGate::new();
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([H2ConnectionScript::new()
                .fallback(H2StreamScript::respond_after(
                    H2Response::ok("ok"),
                    response_gate.waiter(),
                ))]))
            .start()
            .await
            .expect("H2 server should start");
        let time = ManualTimeSource::new(UNIX_EPOCH);
        let pool = ConnectionPool::builder()
            .tls_provider(rustls_aws_lc())
            .tls_context(test_tls::SERVER_IDENTITY.client_context())
            .time_source(time.clone())
            .build_https()
            .expect("valid partitioned HTTPS pool");
        let client =
            SharedHttpClient::new(PoolClient::new(&pool).expect("anonymous partition exists"));
        let connector = test_client::connector_with_time_source(&client, time.clone());
        let capture = CaptureHttpAttemptTelemetry::new();
        let mut request = HttpRequest::get(server.url("/gated")).expect("valid HTTP request");
        request.add_extension(capture.clone());
        let send = tokio::spawn({
            let connector = connector.clone();
            async move { test_client::send_and_collect(&connector, request).await }
        });

        response_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("server should accept the H2 stream");
        time.advance(Duration::from_secs(10));
        response_gate.release();
        let (status, body) = send.await.expect("request task");
        assert_eq!((status, body.as_slice()), (200, b"ok".as_slice()));

        let telemetry = capture.get();
        assert_eq!(
            telemetry.acquisition().expect("acquisition").duration(),
            Some(Duration::ZERO)
        );
        assert_eq!(
            telemetry.connector_call_duration(),
            Some(Duration::from_secs(10))
        );

        drop(connector);
        drop(client);
        drop(pool);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

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

    #[tokio::test]
    async fn test_poisoned_connection_is_not_reused_with_partitioned_pool() {
        poisoned_connection_is_not_reused(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_stream_reset_does_not_retire_connection_with_partitioned_pool() {
        stream_reset_does_not_retire_connection(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_dropping_response_body_cancels_only_stream_with_partitioned_pool() {
        dropping_response_body_cancels_only_stream(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_graceful_goaway_preserves_in_flight_stream_and_replaces_connection_with_partitioned_pool(
    ) {
        graceful_goaway_preserves_in_flight_stream_and_replaces_connection(
            &PartitionedConnectionPool,
        )
        .await;
    }

    #[tokio::test]
    async fn goaway_before_first_stream_has_bounded_reacquisition() {
        let script = H2ConnectionScript::new()
            .goaway_on_ready()
            .fallback(H2StreamScript::respond(H2Response::ok("unexpected")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::unbounded(script))
            .start()
            .await
            .expect("H2 server should start");
        let client = h2_client(&PartitionedConnectionPool);
        let connector = test_client::connector(&client);

        let outcome = tokio::time::timeout(
            test_client::WAIT,
            test_client::send_request(
                &connector,
                HttpRequest::get(server.url("/before-first-stream")).expect("valid HTTP request"),
            ),
        )
        .await
        .expect("GOAWAY before dispatch did not terminate");
        assert!(
            outcome.is_err(),
            "a request succeeded after GOAWAY excluded its stream"
        );
        assert!(
            server.connection_count() <= 3,
            "GOAWAY before dispatch created {} connections",
            server.connection_count()
        );

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }
}

mod abrupt_transport_failure {
    use super::*;
    use aws_smithy_http_client::pool::ConnectionEvent;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum PoolEvent {
        Opened(ConnectionId),
        LogicalClose(ConnectionId),
        PhysicalClose(ConnectionId),
    }

    /// Raw transport loss terminates every accepted stream and later demand uses
    /// a replacement connection.
    async fn abrupt_transport_failure_terminates_streams_and_replaces_connection(
        client: SharedHttpClient,
    ) -> aws_smithy_http_client::pool::OriginKey {
        let body_gate = ManualGate::new();
        let held = |body| {
            H2StreamScript::respond(
                H2Response::new(http_1x::StatusCode::OK).body(H2BodyPlan::gated(
                    body,
                    body_gate.waiter(),
                    "-never",
                )),
            )
        };
        let first = H2ConnectionScript::new()
            .route("/warm", H2StreamScript::respond(H2Response::ok("warm")))
            .route("/one", held("one"))
            .route("/two", held("two"))
            .route("/three", held("three"));
        let second = H2ConnectionScript::new().route(
            "/replacement",
            H2StreamScript::respond(H2Response::ok("replacement")),
        );
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([first, second]))
            .start()
            .await
            .expect("H2 server should start");
        let connector = test_client::connector(&client);

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/warm")).await;
        assert_eq!((status, body.as_slice()), (200, b"warm".as_slice()));
        let (one, two, three) = tokio::join!(
            test_client::send_request(
                &connector,
                HttpRequest::get(server.url("/one")).expect("valid HTTP request"),
            ),
            test_client::send_request(
                &connector,
                HttpRequest::get(server.url("/two")).expect("valid HTTP request"),
            ),
            test_client::send_request(
                &connector,
                HttpRequest::get(server.url("/three")).expect("valid HTTP request"),
            ),
        );
        let responses = [
            one.expect("first response headers should arrive"),
            two.expect("second response headers should arrive"),
            three.expect("third response headers should arrive"),
        ];
        body_gate
            .wait_for_arrivals(3, test_client::WAIT)
            .await
            .expect("all response streams should reach the body gate");

        let original = single_stream_connection(&server, "/warm");
        assert_eq!(original, single_stream_connection(&server, "/one"));
        assert_eq!(original, single_stream_connection(&server, "/two"));
        assert_eq!(original, single_stream_connection(&server, "/three"));
        server
            .abort_transport(original)
            .await
            .expect("the original transport should abort");

        for response in responses {
            let body_result =
                tokio::time::timeout(test_client::WAIT, response.into_body().collect())
                    .await
                    .expect("aborted response body should terminate");
            assert!(
                body_result.is_err(),
                "abrupt transport loss completed an accepted response body"
            );
        }
        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ConnectionClosed {
                        connection_id,
                        reason: ConnectionCloseReason::ScriptedTransportAbort,
                    } if *connection_id == original
                )
            })
            .await
            .expect("the harness should record raw transport loss");

        let replacement_url = server.url("/replacement");
        let origin = aws_smithy_http_client::pool::OriginKey::from_uri(
            &replacement_url.parse().expect("valid replacement URI"),
        )
        .expect("replacement URI should name an origin");
        let (status, body) = test_client::get_and_collect(&connector, &replacement_url).await;
        assert_eq!((status, body.as_slice()), (200, b"replacement".as_slice()));
        assert_ne!(
            original,
            single_stream_connection(&server, "/replacement"),
            "later demand reused the aborted connection"
        );
        assert_eq!(2, server.connection_count());

        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
        origin
    }

    async fn wait_for_pool_events(
        events: &Arc<Mutex<Vec<PoolEvent>>>,
        predicate: impl Fn(&[PoolEvent]) -> bool,
    ) {
        tokio::time::timeout(test_client::WAIT, async {
            loop {
                if predicate(&events.lock().unwrap()) {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("pool lifecycle events should converge");
    }

    #[tokio::test]
    async fn test_abrupt_transport_failure_terminates_streams_and_replaces_connection_with_hyper_util_legacy_pool(
    ) {
        abrupt_transport_failure_terminates_streams_and_replaces_connection(h2_client(
            &HyperUtilLegacyPool,
        ))
        .await;
    }

    #[tokio::test]
    async fn test_abrupt_transport_failure_terminates_streams_and_replaces_connection_with_partitioned_pool(
    ) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = events.clone();
        let pool = ConnectionPool::builder()
            .tls_provider(rustls_aws_lc())
            .tls_context(test_tls::SERVER_IDENTITY.client_context())
            .max_connections_per_host(1)
            .event_listener(move |event: &ConnectionEvent<'_>| {
                let event = match event {
                    ConnectionEvent::Opened(opened) => {
                        Some(PoolEvent::Opened(opened.connection().id()))
                    }
                    ConnectionEvent::LogicalClose(closed) => {
                        Some(PoolEvent::LogicalClose(closed.connection().id()))
                    }
                    ConnectionEvent::PhysicalClose(closed) => {
                        Some(PoolEvent::PhysicalClose(closed.connection().id()))
                    }
                    ConnectionEvent::EstablishmentFailed(_) => None,
                    _ => None,
                };
                if let Some(event) = event {
                    observed.lock().unwrap().push(event);
                }
            })
            .build_https()
            .expect("valid partitioned HTTPS pool");
        let client = SharedHttpClient::new(
            PoolClient::new(&pool).expect("anonymous partition should resolve"),
        );

        let origin =
            abrupt_transport_failure_terminates_streams_and_replaces_connection(client).await;
        wait_for_pool_events(&events, |events| {
            events
                .iter()
                .filter(|event| matches!(event, PoolEvent::PhysicalClose(_)))
                .count()
                >= 1
        })
        .await;

        let events = events.lock().unwrap();
        let opened: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                PoolEvent::Opened(connection) => Some(*connection),
                _ => None,
            })
            .collect();
        assert_eq!(2, opened.len(), "expected original and replacement opens");
        assert_ne!(opened[0], opened[1]);
        assert_eq!(
            1,
            events
                .iter()
                .filter(|event| matches!(event, PoolEvent::LogicalClose(id) if *id == opened[0]))
                .count(),
            "the aborted generation should close logically once"
        );
        assert_eq!(
            1,
            events
                .iter()
                .filter(|event| matches!(event, PoolEvent::PhysicalClose(id) if *id == opened[0]))
                .count(),
            "the aborted transport should close physically once"
        );
        drop(events);

        let stats = pool
            .partition_stats(PartitionId::ANONYMOUS, &origin)
            .expect("anonymous partition should retain origin statistics");
        assert_eq!(0, stats.pending_acquisitions());
        assert_eq!(0, stats.establishing_connections());
        assert_eq!(0, stats.h2().active_requests());
        assert_eq!(1, stats.h2().accepting());
        assert_eq!(0, stats.h2().draining());
        assert_eq!(1, stats.physically_live_connections());
        let capacity = pool
            .origin_stats(&origin)
            .capacity()
            .copied()
            .expect("origin should have bounded capacity");
        assert_eq!(1, capacity.limit());
        assert_eq!(
            1,
            capacity.in_use(),
            "replacement connection should own the returned slot"
        );
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

    #[tokio::test]
    async fn test_s2n_negotiates_h2_and_reuses_connection_with_partitioned_pool() {
        s2n_negotiates_h2_and_reuses_connection(&PartitionedConnectionPool).await;
    }
}

mod idle_timeout {
    use super::*;
    use aws_smithy_types::body::SdkBody;
    use http_body_1x::{Body, Frame, SizeHint};
    use std::convert::Infallible;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;
    use tokio::sync::oneshot;

    const IDLE_TIMEOUT: Duration = Duration::from_millis(100);

    /// Streaming request body that remains open until the test releases it.
    struct HeldUpload {
        finish: oneshot::Receiver<()>,
        complete: bool,
    }

    impl Body for HeldUpload {
        type Data = Bytes;
        type Error = Infallible;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
            if self.complete {
                return Poll::Ready(None);
            }
            match Pin::new(&mut self.finish).poll(cx) {
                Poll::Ready(_) => {
                    self.complete = true;
                    Poll::Ready(None)
                }
                Poll::Pending => Poll::Pending,
            }
        }

        fn is_end_stream(&self) -> bool {
            self.complete
        }

        fn size_hint(&self) -> SizeHint {
            SizeHint::default()
        }
    }

    fn held_upload() -> (oneshot::Sender<()>, SdkBody) {
        let (finish, finished) = oneshot::channel();
        let body = HeldUpload {
            finish: finished,
            complete: false,
        };
        (finish, SdkBody::from_body_1_x(body))
    }

    fn client_with_idle_timeout(backend: &dyn HttpsClientBackend) -> SharedHttpClient {
        backend.build_https(
            BackendConfig {
                pool_idle_timeout: Some(IDLE_TIMEOUT),
                ..Default::default()
            },
            rustls_aws_lc(),
            test_tls::SERVER_IDENTITY.client_context(),
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

    #[tokio::test]
    async fn test_idle_connection_is_evicted_after_timeout_with_partitioned_pool() {
        idle_connection_is_evicted_after_timeout(&PartitionedConnectionPool).await;
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

    #[tokio::test]
    async fn test_active_stream_survives_idle_timeout_but_later_request_uses_replacement_with_partitioned_pool(
    ) {
        active_stream_survives_idle_timeout_but_later_request_uses_replacement(
            &PartitionedConnectionPool,
        )
        .await;
    }

    /// Idle expiration retains a physical connection while its upload remains active.
    #[tokio::test]
    async fn response_completion_does_not_close_active_upload_at_idle_timeout() {
        let held_request_gate = ManualGate::new();
        let script = H2ConnectionScript::new()
            .route(
                "/upload",
                H2StreamScript::respond_before_receiving_request_body(
                    H2Response::ok("early response"),
                    held_request_gate.waiter(),
                ),
            )
            .route("/reuse", H2StreamScript::respond(H2Response::ok("reused")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let client = client_with_idle_timeout(&PartitionedConnectionPool);
        let connector = test_client::connector(&client);

        let (finish_upload, upload) = held_upload();
        let mut request = HttpRequest::new(upload);
        request.set_method("POST").expect("valid HTTP method");
        request
            .set_uri(server.url("/upload"))
            .expect("valid HTTP URI");
        let response = test_client::send_request(&connector, request)
            .await
            .expect("response should arrive before upload completion");
        let (status, body) = test_client::collect_response(response).await;
        assert_eq!(
            (status, body.as_slice()),
            (200, b"early response".as_slice())
        );

        held_request_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("server should retain the active request body");
        let upload_connection = single_stream_connection(&server, "/upload");
        tokio::time::sleep(IDLE_TIMEOUT * 2).await;
        assert!(
            !server.events().iter().any(|event| {
                matches!(
                    event,
                    H2Event::ConnectionClosed { connection_id, .. }
                        if *connection_id == upload_connection
                )
            }),
            "idle expiration closed a connection with an active upload"
        );

        let (status, body) = test_client::get_and_collect(&connector, &server.url("/reuse")).await;
        assert_eq!((status, body.as_slice()), (200, b"reused".as_slice()));
        assert_ne!(
            upload_connection,
            single_stream_connection(&server, "/reuse"),
            "an expired HTTP/2 generation accepted a new request"
        );
        assert_eq!(server.connection_count(), 2);

        finish_upload
            .send(())
            .expect("request body disappeared before upload completion");
        held_request_gate.release();
        server
            .wait_for_event(test_client::WAIT, |event| {
                matches!(
                    event,
                    H2Event::ConnectionClosed { connection_id, .. }
                        if *connection_id == upload_connection
                )
            })
            .await
            .expect("draining connection should close after upload completion");
        drop(connector);
        drop(client);
        server.shutdown().await.expect("clean H2 server shutdown");
    }
}

mod partition_reuse {
    use super::*;
    use std::time::Duration;

    fn partitioned_clients(
        scope: ConnectionReuseScope,
    ) -> (ConnectionPool, SharedHttpConnector, SharedHttpConnector) {
        let first = PartitionId::from_index(1);
        let second = PartitionId::from_index(2);
        let pool = ConnectionPool::builder()
            .tls_provider(rustls_aws_lc())
            .tls_context(test_tls::SERVER_IDENTITY.client_context())
            .partitions([
                Partition::new(
                    first,
                    DriverSpawner::tokio(tokio::runtime::Handle::current()),
                ),
                Partition::new(
                    second,
                    DriverSpawner::tokio(tokio::runtime::Handle::current()),
                ),
            ])
            .connection_reuse_scope(scope)
            .max_connections_per_host(1)
            .build_https()
            .expect("valid partitioned HTTPS pool");
        let first_client = SharedHttpClient::new(
            PoolClient::from_partition(&pool, first).expect("first partition should resolve"),
        );
        let second_client = SharedHttpClient::new(
            PoolClient::from_partition(&pool, second).expect("second partition should resolve"),
        );
        (
            pool,
            test_client::connector(&first_client),
            test_client::connector(&second_client),
        )
    }

    async fn eligible_partition_reuses_peer_h2(scope: ConnectionReuseScope) {
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([H2ConnectionScript::new()
                .fallback(H2StreamScript::respond(H2Response::ok("shared")))]))
            .start()
            .await
            .expect("H2 server should start");
        let (pool, first, second) = partitioned_clients(scope);

        let first_result = test_client::get_and_collect(&first, &server.url("/first")).await;
        let second_result = test_client::get_and_collect(&second, &server.url("/second")).await;

        assert_eq!(first_result, (200, b"shared".to_vec()));
        assert_eq!(second_result, (200, b"shared".to_vec()));
        assert_eq!(server.connection_count(), 1);
        assert_eq!(
            single_stream_connection(&server, "/first"),
            single_stream_connection(&server, "/second"),
            "eligible partitions should dispatch through the connection-owning generation"
        );

        drop(first);
        drop(second);
        drop(pool);
        server.shutdown().await.expect("clean H2 server shutdown");
    }

    #[tokio::test]
    async fn pool_scope_reuses_a_peer_h2_generation() {
        eligible_partition_reuses_peer_h2(ConnectionReuseScope::Pool).await;
    }

    #[tokio::test]
    async fn matching_network_interface_scope_reuses_a_peer_h2_generation() {
        eligible_partition_reuses_peer_h2(ConnectionReuseScope::NetworkInterface).await;
    }

    #[tokio::test]
    async fn partition_scope_waits_until_out_of_scope_h2_releases_capacity() {
        let script = H2ConnectionScript::new()
            .fallback(H2StreamScript::respond(H2Response::ok("partition")));
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([script.clone(), script]))
            .start()
            .await
            .expect("H2 server should start");
        let (pool, first, second) = partitioned_clients(ConnectionReuseScope::Partition);

        let (status, body, metadata) =
            get_and_collect_with_capture(&first, &server.url("/first")).await;
        assert_eq!((status, body.as_slice()), (200, b"partition".as_slice()));
        let first_connection = single_stream_connection(&server, "/first");

        let second_url = server.url("/second");
        let mut pending =
            tokio::spawn(async move { test_client::get_and_collect(&second, &second_url).await });
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut pending)
                .await
                .is_err(),
            "an out-of-scope generation must not satisfy partition-local demand"
        );
        assert_eq!(server.connection_count(), 1);

        metadata.poison();
        let (status, body) = tokio::time::timeout(test_client::WAIT, pending)
            .await
            .expect("released capacity should wake partition-local demand")
            .expect("request task should not panic");
        assert_eq!((status, body.as_slice()), (200, b"partition".as_slice()));
        let second_connection = single_stream_connection(&server, "/second");
        assert_ne!(first_connection, second_connection);
        assert_eq!(server.connection_count(), 2);

        drop(metadata);
        drop(first);
        drop(pool);
        server.shutdown().await.expect("clean H2 server shutdown");
    }
}

mod runtime_placement {
    use super::*;
    use aws_smithy_http_client::pool::ConnectionEvent;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use runtime::DrivenRuntime;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::runtime::{Handle, Id};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum EventKind {
        Opened,
        LogicalClose,
        PhysicalClose,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct ObservedEvent {
        kind: EventKind,
        connection: ConnectionId,
        partition: PartitionId,
        runtime: Option<Id>,
    }

    async fn wait_for_event(
        events: &Arc<Mutex<Vec<ObservedEvent>>>,
        predicate: impl Fn(&ObservedEvent) -> bool,
    ) -> ObservedEvent {
        tokio::time::timeout(test_client::WAIT, async {
            loop {
                if let Some(event) = events.lock().unwrap().iter().copied().find(&predicate) {
                    return event;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("connection event should arrive")
    }

    #[tokio::test]
    async fn peer_h2_reuse_keeps_connection_work_on_the_owner_runtime() {
        let first_id = PartitionId::from_index(1);
        let second_id = PartitionId::from_index(2);
        let first_runtime = DrivenRuntime::start("h2-owner-one");
        let first_runtime_id = first_runtime.id();
        let second_runtime = DrivenRuntime::start("h2-owner-two");
        let second_runtime_id = second_runtime.id();
        let peer_body_gate = ManualGate::new();
        let first_script = H2ConnectionScript::new()
            .abort_streams_on_client_close()
            .route("/first", H2StreamScript::respond(H2Response::ok("first")))
            .route(
                "/peer",
                H2StreamScript::respond(
                    H2Response::new(http_1x::StatusCode::OK).body(H2BodyPlan::gated(
                        "peer-",
                        peer_body_gate.waiter(),
                        "never",
                    )),
                ),
            );
        let second_script = H2ConnectionScript::new().route(
            "/replacement",
            H2StreamScript::respond(H2Response::ok("replacement")),
        );
        let server = H2TestServer::builder()
            .connections(H2ConnectionPlan::queue([first_script, second_script]))
            .start()
            .await
            .expect("H2 server should start");

        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = events.clone();
        let pool = ConnectionPool::builder()
            .tls_provider(rustls_aws_lc())
            .tls_context(test_tls::SERVER_IDENTITY.client_context())
            .partitions([
                Partition::new(first_id, first_runtime.driver_spawner()),
                Partition::new(second_id, second_runtime.driver_spawner()),
            ])
            .connection_reuse_scope(ConnectionReuseScope::Pool)
            .max_connections_per_host(1)
            .event_listener(move |event: &ConnectionEvent<'_>| {
                let observed_event = match event {
                    ConnectionEvent::Opened(opened) => {
                        Some((EventKind::Opened, opened.connection()))
                    }
                    ConnectionEvent::LogicalClose(closed) => {
                        Some((EventKind::LogicalClose, closed.connection()))
                    }
                    ConnectionEvent::PhysicalClose(closed) => {
                        Some((EventKind::PhysicalClose, closed.connection()))
                    }
                    ConnectionEvent::EstablishmentFailed(_) => None,
                    _ => None,
                };
                if let Some((kind, connection)) = observed_event {
                    observed.lock().unwrap().push(ObservedEvent {
                        kind,
                        connection: connection.id(),
                        partition: connection.owner_partition(),
                        runtime: Handle::try_current().ok().map(|handle| handle.id()),
                    });
                }
            })
            .build_https()
            .expect("valid partitioned HTTPS pool");
        let first_client = SharedHttpClient::new(
            PoolClient::from_partition(&pool, first_id).expect("first partition should resolve"),
        );
        let second_client = SharedHttpClient::new(
            PoolClient::from_partition(&pool, second_id).expect("second partition should resolve"),
        );
        let first = test_client::connector(&first_client);
        let second = test_client::connector(&second_client);

        let first_request = first.clone();
        let first_url = server.url("/first");
        let (status, body) = first_runtime
            .spawn(async move { test_client::get_and_collect(&first_request, &first_url).await })
            .await
            .expect("first partition request task should not panic");
        assert_eq!((status, body.as_slice()), (200, b"first".as_slice()));
        let original_server_connection = single_stream_connection(&server, "/first");
        let original_open = wait_for_event(&events, |event| {
            event.kind == EventKind::Opened && event.partition == first_id
        })
        .await;
        assert_eq!(Some(first_runtime_id), original_open.runtime);
        assert!(first_runtime.submitted_tasks() > 0);

        let peer_request = second.clone();
        let peer_url = server.url("/peer");
        let peer_response = second_runtime
            .spawn(async move {
                test_client::send_request(
                    &peer_request,
                    HttpRequest::get(peer_url).expect("valid HTTP request"),
                )
                .await
            })
            .await
            .expect("peer request task should not panic")
            .expect("peer response headers should arrive");
        peer_body_gate
            .wait_until_reached(test_client::WAIT)
            .await
            .expect("peer response should reach its body gate");
        assert_eq!(
            original_server_connection,
            single_stream_connection(&server, "/peer"),
            "peer demand should use the first partition's generation"
        );
        assert!(
            !events
                .lock()
                .unwrap()
                .iter()
                .any(|event| { event.kind == EventKind::Opened && event.partition == second_id }),
            "peer reuse should not establish a requester-owned connection"
        );

        first_runtime.shutdown();
        let body_result =
            tokio::time::timeout(test_client::WAIT, peer_response.into_body().collect())
                .await
                .expect("owner-runtime shutdown should terminate the accepted response");
        assert!(
            body_result.is_err(),
            "accepted response completed after its owner runtime stopped"
        );
        let logical_close = wait_for_event(&events, |event| {
            event.kind == EventKind::LogicalClose && event.connection == original_open.connection
        })
        .await;
        assert_eq!(first_id, logical_close.partition);
        let physical_close = wait_for_event(&events, |event| {
            event.kind == EventKind::PhysicalClose && event.connection == original_open.connection
        })
        .await;
        assert_eq!(first_id, physical_close.partition);

        let replacement_request = second.clone();
        let replacement_url = server.url("/replacement");
        let (status, body) = second_runtime
            .spawn(async move {
                test_client::get_and_collect(&replacement_request, &replacement_url).await
            })
            .await
            .expect("replacement request task should not panic");
        assert_eq!((status, body.as_slice()), (200, b"replacement".as_slice()));
        assert_ne!(
            original_server_connection,
            single_stream_connection(&server, "/replacement")
        );
        let replacement_open = wait_for_event(&events, |event| {
            event.kind == EventKind::Opened && event.partition == second_id
        })
        .await;
        assert_eq!(Some(second_runtime_id), replacement_open.runtime);
        assert_ne!(original_open.connection, replacement_open.connection);
        assert!(second_runtime.submitted_tasks() > 0);

        let origin = aws_smithy_http_client::pool::OriginKey::from_uri(
            &server.url("/").parse().expect("valid server URI"),
        )
        .expect("server URI should name an origin");
        let first_stats = pool
            .partition_stats(first_id, &origin)
            .expect("first partition should retain origin statistics");
        assert_eq!(0, first_stats.h2().active_requests());
        assert_eq!(0, first_stats.physically_live_connections());
        let second_stats = pool
            .partition_stats(second_id, &origin)
            .expect("second partition should retain origin statistics");
        assert_eq!(1, second_stats.h2().accepting());
        assert_eq!(0, second_stats.h2().active_requests());
        assert_eq!(1, second_stats.physically_live_connections());
        let capacity = pool
            .origin_stats(&origin)
            .capacity()
            .copied()
            .expect("origin should have bounded capacity");
        assert_eq!(1, capacity.limit());
        assert_eq!(1, capacity.in_use());

        drop(first);
        drop(second);
        drop(first_client);
        drop(second_client);
        drop(pool);
        second_runtime.shutdown();
        server.shutdown().await.expect("clean H2 server shutdown");
    }
}
