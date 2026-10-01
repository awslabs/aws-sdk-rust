/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Behavior specific to the partition-aware connection pool.

#![cfg(all(
    feature = "wire-mock",
    feature = "default-client",
    feature = "rt-tokio"
))]

mod common {
    #[allow(dead_code)]
    pub(crate) mod client;
}

#[path = "common/runtime.rs"]
mod runtime;

use aws_smithy_async::test_util::ManualTimeSource;
use aws_smithy_http_client::pool::{
    Client, ConnectPath, ConnectionEstablishmentId, ConnectionEstablishmentStage, ConnectionEvent,
    ConnectionPool, ConnectionProtocol, ConnectionReuseScope, DriverSpawner, OriginKey, Partition,
    PartitionId,
};
use aws_smithy_http_client::test_util::wire::connection::{
    BodyPlan, ConnectionCloseReason, ConnectionEvent as WireConnectionEvent, ConnectionTestHarness,
    EndpointPlan, Http1Response, Http1Script, ManualGate, SocketScript,
};
use aws_smithy_runtime_api::client::connection::{CaptureSmithyConnection, ConnectionId};
use aws_smithy_runtime_api::client::http::telemetry::{
    CaptureHttpAttemptTelemetry, ConnectionUsage,
};
use aws_smithy_runtime_api::client::http::{SharedHttpClient, SharedHttpConnector};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_types::body::SdkBody;
use bytes::Bytes;
use common::client as test_client;
use http_body_1x::{Body, Frame, SizeHint};
use std::convert::Infallible;
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const IP1: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const IP2: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));

#[derive(Debug)]
enum ObservedConnectionEvent {
    Failed {
        establishment: ConnectionEstablishmentId,
        origin: OriginKey,
        stage: ConnectionEstablishmentStage,
        remote_addr: Option<SocketAddr>,
        protocol: Option<ConnectionProtocol>,
        error: String,
    },
    Opened {
        establishment: ConnectionEstablishmentId,
        connection: aws_smithy_runtime_api::client::connection::ConnectionId,
        establishment_partition: PartitionId,
        establishment_origin: OriginKey,
        connection_origin: OriginKey,
        connection_partition: PartitionId,
        remote_addr: Option<SocketAddr>,
        protocol: ConnectionProtocol,
        connect_path: ConnectPath,
        handshake_measured: bool,
    },
}

/// Resolves one declared partition through the public Smithy client boundary.
fn shared_client(pool: &ConnectionPool, partition: PartitionId) -> SharedHttpClient {
    SharedHttpClient::new(
        Client::from_partition(pool, partition).expect("declared partition should resolve"),
    )
}

/// Builds an operation connector with the shared test runtime components.
fn connector(client: &SharedHttpClient) -> SharedHttpConnector {
    test_client::connector(client)
}

#[tokio::test]
async fn connection_listener_observes_each_establishment_terminal_event() {
    let refused_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("ephemeral listener should bind");
    let refused_addr = refused_listener.local_addr().unwrap();
    drop(refused_listener);

    let handshake_listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("handshake listener should bind");
    let closed_peer_addr = handshake_listener.local_addr().unwrap();
    let closed_peer_server = tokio::spawn(async move {
        // HTTP/1 installation completes before the driver observes this close.
        let (stream, _) = handshake_listener.accept().await.unwrap();
        drop(stream);
    });

    let observed = Arc::new(Mutex::new(Vec::new()));
    let pool = ConnectionPool::builder()
        .event_listener({
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                let event = match event {
                    ConnectionEvent::EstablishmentFailed(failed) => {
                        ObservedConnectionEvent::Failed {
                            establishment: failed.establishment().id(),
                            origin: failed.establishment().origin().clone(),
                            stage: failed.stage(),
                            remote_addr: failed.remote_addr(),
                            protocol: failed.protocol(),
                            error: failed.error().to_string(),
                        }
                    }
                    ConnectionEvent::Opened(opened) => ObservedConnectionEvent::Opened {
                        establishment: opened.establishment().id(),
                        connection: opened.connection().id(),
                        establishment_origin: opened.establishment().origin().clone(),
                        connection_origin: opened.connection().origin().clone(),
                        establishment_partition: opened.establishment().partition(),
                        connection_partition: opened.connection().owner_partition(),
                        remote_addr: opened.connection().remote_addr(),
                        protocol: opened.connection().protocol(),
                        connect_path: opened.connection().connect_path(),
                        handshake_measured: opened.stats().protocol_handshake_duration().is_some(),
                    },
                    _ => return,
                };
                observed.lock().unwrap().push(event);
            }
        })
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = connector(&client);

    let refused_url = format!("http://{refused_addr}/");
    test_client::send_request(
        &connector,
        HttpRequest::get(&refused_url).expect("valid refused request"),
    )
    .await
    .expect_err("closed listener should refuse the transport");

    let closed_peer_url = format!("http://{closed_peer_addr}/");
    test_client::send_request(
        &connector,
        HttpRequest::get(closed_peer_url).expect("valid closed-peer request"),
    )
    .await
    .expect_err("closed peer should fail the request after installation");
    closed_peer_server.await.unwrap();

    let success_listener = TcpListener::bind(refused_addr)
        .await
        .expect("failed origin should become reachable");
    let success_server = tokio::spawn(async move {
        let (mut stream, _) = success_listener.accept().await.unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        while !request.windows(4).any(|window| window == b"\r\n\r\n") {
            let count = stream.read(&mut buffer).await.unwrap();
            assert_ne!(0, count, "request ended before its header block");
            request.extend_from_slice(&buffer[..count]);
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 6\r\n\r\nopened")
            .await
            .unwrap();
    });

    let (status, body) = test_client::get_and_collect(&connector, &refused_url).await;
    assert_eq!((status, body.as_slice()), (200, b"opened".as_slice()));
    success_server.await.unwrap();

    let events = observed.lock().unwrap();
    assert_eq!(
        3,
        events.len(),
        "each establishment emits one terminal event"
    );
    let (transport_id, closed_peer_id, opened_id) = match events.as_slice() {
        [ObservedConnectionEvent::Failed {
            establishment: transport_id,
            origin: failed_origin,
            stage: ConnectionEstablishmentStage::Transport,
            remote_addr: None,
            protocol: None,
            error,
        }, ObservedConnectionEvent::Opened {
            establishment: closed_peer_id,
            connection: closed_peer_connection,
            establishment_origin: closed_peer_establishment_origin,
            connection_origin: closed_peer_connection_origin,
            establishment_partition: closed_peer_establishment_partition,
            connection_partition: closed_peer_connection_partition,
            remote_addr: Some(closed_peer_remote_addr),
            protocol: ConnectionProtocol::Http1,
            connect_path: ConnectPath::Direct,
            handshake_measured: true,
        }, ObservedConnectionEvent::Opened {
            establishment: opened_id,
            connection,
            establishment_origin,
            connection_origin,
            establishment_partition,
            connection_partition,
            remote_addr: Some(opened_remote_addr),
            protocol: ConnectionProtocol::Http1,
            connect_path: ConnectPath::Direct,
            handshake_measured: true,
        }] => {
            assert!(!error.is_empty());
            assert_eq!(closed_peer_addr, *closed_peer_remote_addr);
            assert_eq!(
                closed_peer_establishment_origin,
                closed_peer_connection_origin
            );
            assert_eq!(ConnectionId::new(0), *closed_peer_connection);
            assert_eq!(PartitionId::ANONYMOUS, *closed_peer_establishment_partition);
            assert_eq!(
                *closed_peer_establishment_partition,
                *closed_peer_connection_partition
            );
            assert_eq!(refused_addr, *opened_remote_addr);
            assert_eq!(failed_origin, establishment_origin);
            assert_eq!(establishment_origin, connection_origin);
            assert_eq!(PartitionId::ANONYMOUS, *establishment_partition);
            assert_eq!(*establishment_partition, *connection_partition);
            assert_eq!(ConnectionId::new(1), *connection);
            (*transport_id, *closed_peer_id, *opened_id)
        }
        events => panic!("unexpected connection events: {events:#?}"),
    };
    assert_ne!(transport_id, closed_peer_id);
    assert_ne!(transport_id, opened_id);
    assert_ne!(closed_peer_id, opened_id);

    drop(events);
    drop(connector);
    drop(client);
    drop(pool);
}

#[tokio::test]
async fn address_fallback_emits_one_successful_establishment() {
    const HOST: &str = "address-fallback.test";

    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP2,
            Http1Script::responses([Http1Response::ok().body("fallback")]),
        )
        .dns(HOST, [IP1, IP2])
        .build()
        .await
        .expect("harness should start");
    let endpoint = harness.endpoint(0).unwrap().addr();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let pool = ConnectionPool::builder()
        .dns_resolver(harness.dns_resolver())
        .event_listener({
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                let observation = match event {
                    ConnectionEvent::EstablishmentFailed(failed) => (false, failed.remote_addr()),
                    ConnectionEvent::Opened(opened) => (true, opened.connection().remote_addr()),
                    _ => return,
                };
                observed.lock().unwrap().push(observation);
            }
        })
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = connector(&client);
    let url = format!("http://{HOST}:{}/", harness.port());

    let (status, body) = test_client::get_and_collect(&connector, &url).await;
    assert_eq!((status, body.as_slice()), (200, b"fallback".as_slice()));
    assert_eq!(
        &[(true, Some(endpoint))],
        observed.lock().unwrap().as_slice(),
        "a failed address attempt remains inside one successful establishment"
    );

    drop(connector);
    drop(client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn custom_dns_resolver_is_used_for_explicit_partition_connections() {
    const HOST: &str = "partition-dns.test";

    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body("custom dns")]),
        )
        .dns(HOST, [IP1])
        .build()
        .await
        .expect("harness should start");
    let partition = PartitionId::from_index(1);
    let pool = ConnectionPool::builder()
        .partitions([Partition::new(
            partition,
            DriverSpawner::tokio(tokio::runtime::Handle::current()),
        )])
        .dns_resolver(harness.dns_resolver())
        .build_http()
        .expect("valid pool");
    let client = shared_client(&pool, partition);
    let connector = connector(&client);
    let url = format!("http://{HOST}:{}/custom-dns", harness.port());

    let (status, body) = test_client::get_and_collect(&connector, &url).await;
    assert_eq!((status, body.as_slice()), (200, b"custom dns".as_slice()));
    assert_eq!(1, harness.dns_lookup_count());

    drop(connector);
    drop(client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn h1_attempt_telemetry_distinguishes_fresh_and_reused_connections() {
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
    let opened = Arc::new(Mutex::new(None));
    let pool = ConnectionPool::builder()
        .event_listener({
            let opened = opened.clone();
            move |event: &ConnectionEvent<'_>| {
                if let ConnectionEvent::Opened(event) = event {
                    *opened.lock().unwrap() =
                        Some((event.connection().id(), event.stats().total_duration()));
                }
            }
        })
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = connector(&client);

    let first_connection_capture = CaptureSmithyConnection::new();
    let first_capture = CaptureHttpAttemptTelemetry::new();
    let mut first = HttpRequest::get(harness.endpoint_url()).expect("valid request");
    first.add_extension(first_connection_capture.clone());
    first.add_extension(first_capture.clone());
    test_client::send_and_collect(&connector, first).await;

    let second_capture = CaptureHttpAttemptTelemetry::new();
    let mut second = HttpRequest::get(harness.endpoint_url()).expect("valid request");
    second.add_extension(second_capture.clone());
    test_client::send_and_collect(&connector, second).await;

    let first = first_capture.get();
    let second = second_capture.get();
    assert!(first.connector_call_duration().is_some());
    assert!(second.connector_call_duration().is_some());
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
    let captured_connection = first_connection_capture
        .get()
        .expect("connection capture should contain selected metadata");
    assert_eq!(
        captured_connection.connection_id(),
        first_connection.connection_id()
    );
    assert_eq!(
        captured_connection.establishment(),
        first_connection.establishment(),
        "connection and attempt captures must observe the same selected connection"
    );
    assert_eq!(
        first_connection.connection_id(),
        second_connection.connection_id()
    );
    assert_eq!(
        first_connection.establishment(),
        second_connection.establishment(),
        "reuse must retain the establishment that created the connection"
    );
    let first_establishment = first_connection
        .establishment()
        .expect("first connection establishment");
    let (opened_connection, opened_duration) =
        opened.lock().unwrap().expect("opened connection event");
    assert_eq!(
        first_connection.connection_id(),
        Some(opened_connection),
        "request telemetry and lifecycle events must identify the same connection"
    );
    assert_eq!(
        first_establishment.total_duration(),
        opened_duration,
        "request metadata and the opened event must share one establishment measurement"
    );
    assert_eq!(1, harness.tcp_accepted_count());

    drop(connector);
    drop(client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn acquisition_duration_ends_when_hyper_accepts_the_request() {
    let response_gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_http1_request()
                .wait(response_gate.waiter())
                .write_all(
                    "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\nok",
                )
                .await_client_close(),
        )
        .build()
        .await
        .expect("harness should start");
    let pool_time = ManualTimeSource::new(UNIX_EPOCH);
    let attempt_time = ManualTimeSource::new(UNIX_EPOCH);
    let pool = ConnectionPool::builder()
        .time_source(pool_time)
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = test_client::connector_with_time_source(&client, attempt_time.clone());
    let capture = CaptureHttpAttemptTelemetry::new();
    let mut request = HttpRequest::get(harness.endpoint_url()).expect("valid request");
    request.add_extension(capture.clone());

    let send = tokio::spawn({
        let connector = connector.clone();
        async move { test_client::send_and_collect(&connector, request).await }
    });
    response_gate
        .wait_until_reached(test_client::WAIT)
        .await
        .expect("server should receive the accepted request");
    attempt_time.advance(Duration::from_secs(10));
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
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn bounded_waiter_proceeds_after_the_active_h1_returns() {
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
    let attempt_time = ManualTimeSource::new(UNIX_EPOCH);
    let pool = ConnectionPool::builder()
        .max_connections_per_host(1)
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = test_client::connector_with_time_source(&client, attempt_time.clone());

    let first = test_client::send_request(
        &connector,
        HttpRequest::get(harness.endpoint_url()).expect("valid request"),
    )
    .await
    .expect("first request should reach response headers");
    body_gate
        .wait_until_reached(test_client::WAIT)
        .await
        .expect("first response should reach its body gate");

    let second_capture = CaptureHttpAttemptTelemetry::new();
    let mut second_request = HttpRequest::get(harness.endpoint_url()).expect("valid request");
    second_request.add_extension(second_capture.clone());
    let second_connector = connector.clone();
    let mut second = tokio::spawn(async move {
        test_client::send_and_collect(&second_connector, second_request).await
    });
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut second)
            .await
            .is_err(),
        "the second request must wait while the only admitted H1 is active"
    );
    assert_eq!(1, harness.tcp_accepted_count());

    attempt_time.advance(Duration::from_secs(5));
    body_gate.release();
    let (status, body) = test_client::collect_response(first).await;
    assert_eq!((status, body.as_slice()), (200, b"first-body".as_slice()));
    let (status, body) = second.await.expect("second request task should not panic");
    assert_eq!((status, body.as_slice()), (200, b"second".as_slice()));
    assert_eq!(1, harness.tcp_accepted_count());
    let telemetry = second_capture.get();
    let acquisition = telemetry.acquisition().expect("second acquisition");
    assert_eq!(acquisition.duration(), Some(Duration::from_secs(5)));
    assert_eq!(acquisition.usage(), ConnectionUsage::Reused);
    assert_eq!(
        telemetry.connector_call_duration(),
        Some(Duration::from_secs(5))
    );

    drop(connector);
    drop(client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn eligible_partition_borrows_the_peer_h1() {
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
    let first_id = PartitionId::from_index(1);
    let second_id = PartitionId::from_index(2);
    let pool = ConnectionPool::builder()
        .partitions([
            Partition::new(
                first_id,
                DriverSpawner::tokio(tokio::runtime::Handle::current()),
            ),
            Partition::new(
                second_id,
                DriverSpawner::tokio(tokio::runtime::Handle::current()),
            ),
        ])
        .connection_reuse_scope(ConnectionReuseScope::Pool)
        .max_connections_per_host(1)
        .build_http()
        .expect("valid pool");
    let first_client = shared_client(&pool, first_id);
    let second_client = shared_client(&pool, second_id);
    let first_connector = connector(&first_client);
    let second_connector = connector(&second_client);

    test_client::get_and_collect(&first_connector, &harness.endpoint_url()).await;
    test_client::get_and_collect(&second_connector, &harness.endpoint_url()).await;

    let request_connections = harness
        .events()
        .into_iter()
        .filter_map(|event| match event {
            WireConnectionEvent::Http1Request { connection_id, .. } => Some(connection_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(2, request_connections.len());
    assert_eq!(
        request_connections[0], request_connections[1],
        "eligible peer demand should borrow the existing H1"
    );
    assert_eq!(1, harness.tcp_accepted_count());

    drop(first_connector);
    drop(second_connector);
    drop(first_client);
    drop(second_client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

/// Request body that remains open until the test releases it.
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
    (
        finish,
        SdkBody::from_body_1_x(HeldUpload {
            finish: finished,
            complete: false,
        }),
    )
}

#[tokio::test]
async fn peer_h1_reuse_survives_borrower_runtime_shutdown() {
    use runtime::DrivenRuntime;

    let peer_response_gate = ManualGate::new();
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            SocketScript::new()
                .read_http1_request()
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                      Content-Length: 5\r\n\
                      Connection: keep-alive\r\n\
                      \r\n\
                      owner",
                )
                .read_until(b"\r\n\r\n", 16 * 1024)
                .wait(peer_response_gate.waiter())
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                      Content-Length: 4\r\n\
                      Connection: keep-alive\r\n\
                      \r\n\
                      peer",
                )
                .read_until(b"0\r\n\r\n", 1024)
                .read_http1_request()
                .write_all(
                    b"HTTP/1.1 200 OK\r\n\
                      Content-Length: 6\r\n\
                      Connection: keep-alive\r\n\
                      \r\n\
                      reused",
                )
                .await_client_close(),
        )
        .build()
        .await
        .expect("harness should start");
    let first_id = PartitionId::from_index(1);
    let second_id = PartitionId::from_index(2);
    let first_runtime = DrivenRuntime::start("h1-owner-one");
    let second_runtime = DrivenRuntime::start("h1-owner-two");
    assert_ne!(first_runtime.id(), second_runtime.id());
    let pool = ConnectionPool::builder()
        .partitions([
            Partition::new(first_id, first_runtime.driver_spawner()),
            Partition::new(second_id, second_runtime.driver_spawner()),
        ])
        .connection_reuse_scope(ConnectionReuseScope::Pool)
        .max_connections_per_host(1)
        .build_http()
        .expect("valid pool");
    let first_client = shared_client(&pool, first_id);
    let second_client = shared_client(&pool, second_id);
    let first_connector = connector(&first_client);
    let second_connector = connector(&second_client);
    let url = harness.endpoint_url();

    let first_request = first_connector.clone();
    let first_url = url.clone();
    let (status, body) = first_runtime
        .spawn(async move { test_client::get_and_collect(&first_request, &first_url).await })
        .await
        .expect("owner request task should not panic");
    assert_eq!((status, body.as_slice()), (200, b"owner".as_slice()));

    let (finish_upload, upload) = held_upload();
    let mut peer_request = HttpRequest::new(upload);
    peer_request
        .set_uri(url.clone())
        .expect("valid HTTP request URI");
    peer_request.set_method("POST").expect("valid HTTP method");
    let peer_connector = second_connector.clone();
    let peer = second_runtime
        .spawn(async move { test_client::send_and_collect(&peer_connector, peer_request).await });
    peer_response_gate
        .wait_until_reached(test_client::WAIT)
        .await
        .expect("peer request should reach the response gate");
    let owner_tasks_before_peer_completion = first_runtime.submitted_tasks();
    let borrower_tasks_before_peer_completion = second_runtime.submitted_tasks();
    peer_response_gate.release();
    let (status, body) = peer.await.expect("peer request task should not panic");
    assert_eq!((status, body.as_slice()), (200, b"peer".as_slice()));
    assert_eq!(
        owner_tasks_before_peer_completion + 1,
        first_runtime.submitted_tasks(),
        "deferred H1 readiness must use the connection owner's spawner"
    );
    assert_eq!(
        borrower_tasks_before_peer_completion,
        second_runtime.submitted_tasks(),
        "the borrowing partition must not own readiness for a peer connection"
    );
    finish_upload
        .send(())
        .expect("peer upload should remain open until readiness placement is observed");

    second_runtime.shutdown();

    let reused_request = first_connector.clone();
    let reused_url = url.clone();
    let (status, body) = first_runtime
        .spawn(async move { test_client::get_and_collect(&reused_request, &reused_url).await })
        .await
        .expect("reused request task should not panic");
    assert_eq!((status, body.as_slice()), (200, b"reused".as_slice()));

    let request_connections = harness
        .events()
        .into_iter()
        .filter_map(|event| match event {
            WireConnectionEvent::Http1Request { connection_id, .. } => Some(connection_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(2, request_connections.len());
    assert!(
        request_connections
            .iter()
            .all(|connection| *connection == request_connections[0]),
        "owner and peer requests should use one HTTP/1 connection"
    );
    assert_eq!(1, harness.tcp_accepted_count());

    drop(first_connector);
    drop(second_connector);
    drop(first_client);
    drop(second_client);
    drop(pool);
    first_runtime.shutdown();
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn ineligible_partition_reclaims_peer_capacity() {
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
    let first_id = PartitionId::from_index(1);
    let second_id = PartitionId::from_index(2);
    let pool = ConnectionPool::builder()
        .partitions([
            Partition::new(
                first_id,
                DriverSpawner::tokio(tokio::runtime::Handle::current()),
            ),
            Partition::new(
                second_id,
                DriverSpawner::tokio(tokio::runtime::Handle::current()),
            ),
        ])
        .connection_reuse_scope(ConnectionReuseScope::Partition)
        .max_connections_per_host(1)
        .build_http()
        .expect("valid pool");
    let first_client = shared_client(&pool, first_id);
    let second_client = shared_client(&pool, second_id);
    let first_connector = connector(&first_client);
    let second_connector = connector(&second_client);

    test_client::get_and_collect(&first_connector, &harness.endpoint_url()).await;
    let first_connection = harness
        .events()
        .into_iter()
        .find_map(|event| match event {
            WireConnectionEvent::Http1Request { connection_id, .. } => Some(connection_id),
            _ => None,
        })
        .expect("first request should name a connection");
    test_client::get_and_collect(&second_connector, &harness.endpoint_url()).await;

    harness
        .wait_for_event(test_client::WAIT, |event| {
            matches!(
                event,
                WireConnectionEvent::ConnectionClosed {
                    connection_id,
                    reason: ConnectionCloseReason::ClientClosed,
                } if *connection_id == first_connection
            )
        })
        .await
        .expect("reclaim should close the ineligible peer H1");
    assert_eq!(2, harness.tcp_accepted_count());

    drop(first_connector);
    drop(second_connector);
    drop(first_client);
    drop(second_client);
    drop(pool);
    harness.shutdown().await.expect("clean harness shutdown");
}

#[tokio::test]
async fn dropping_the_last_pool_handle_closes_idle_connections() {
    let harness = ConnectionTestHarness::builder()
        .endpoint(
            IP1,
            Http1Script::responses([Http1Response::ok().body("response")]),
        )
        .build()
        .await
        .expect("harness should start");
    let pool = ConnectionPool::builder()
        .idle_timeout(None)
        .build_http()
        .expect("valid pool");
    let client = SharedHttpClient::new(Client::new(&pool).expect("anonymous partition"));
    let connector = connector(&client);

    test_client::get_and_collect(&connector, &harness.endpoint_url()).await;
    let connection = harness
        .events()
        .into_iter()
        .find_map(|event| match event {
            WireConnectionEvent::Http1Request { connection_id, .. } => Some(connection_id),
            _ => None,
        })
        .expect("request should name a connection");

    drop(connector);
    drop(client);
    drop(pool);
    harness
        .wait_for_event(test_client::WAIT, |event| {
            matches!(
                event,
                WireConnectionEvent::ConnectionClosed {
                    connection_id,
                    reason: ConnectionCloseReason::ClientClosed,
                } if *connection_id == connection
            )
        })
        .await
        .expect("pool drop should close its idle connection");
    harness.shutdown().await.expect("clean harness shutdown");
}
