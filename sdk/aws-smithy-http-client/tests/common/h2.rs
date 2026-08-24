/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Scripted HTTP/2 server support for connection behavior tests.
//!
//! This fixture is intentionally test-local. The project keeps its TLS setup surface
//! out of the public API while still testing HTTP/2 connection-pool behavior against
//! a real TLS+h2 server.

use super::client::WAIT;
use super::tls;
use aws_smithy_http_client::test_util::wire::connection::{ConnectionCloseReason, GateWaiter};
use bytes::Bytes;
use h2::server::SendResponse;
use h2::Reason;
use http_1x::{Method, Response, StatusCode};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::future::poll_fn;
use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::{JoinHandle, JoinSet};
use tokio_rustls::TlsAcceptor;

const H2_FRAME_HEADER_LENGTH: usize = 9;
const GOAWAY_FRAME_TYPE: u8 = 0x7;
const GOAWAY_PAYLOAD_LENGTH: usize = 8;
/// Mask for the 31-bit stream identifier in a GOAWAY payload; the high bit is reserved.
const GOAWAY_LAST_STREAM_ID_MASK: u32 = 0x7fff_ffff;

/// Test-local error for the H2 fixture. Structurally identical to the public
/// `HarnessError` but kept local because `HarnessError::new` is not public.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct H2HarnessError {
    message: Arc<str>,
}

impl H2HarnessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: Arc::from(message.into()),
        }
    }
}

impl fmt::Display for H2HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for H2HarnessError {}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct H2ConnectionId(u64);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum H2Event {
    TcpAccepted {
        connection_id: H2ConnectionId,
        peer_addr: SocketAddr,
    },
    TlsNegotiated {
        connection_id: H2ConnectionId,
        alpn: Option<Vec<u8>>,
    },
    TlsHandshakeAbandoned {
        connection_id: H2ConnectionId,
        error: String,
    },
    H2HandshakeAbandoned {
        connection_id: H2ConnectionId,
        error: String,
    },
    H2Ready {
        connection_id: H2ConnectionId,
    },
    StreamAccepted {
        connection_id: H2ConnectionId,
        stream_id: u32,
        method: Method,
        path: String,
    },
    ResponseCompleted {
        connection_id: H2ConnectionId,
        stream_id: u32,
    },
    ResetSent {
        connection_id: H2ConnectionId,
        stream_id: u32,
        reason: Reason,
    },
    ClientResetObserved {
        connection_id: H2ConnectionId,
        stream_id: u32,
        reason: Reason,
    },
    GoAwaySent {
        connection_id: H2ConnectionId,
        last_stream_id: u32,
        reason: Reason,
    },
    ConnectionClosed {
        connection_id: H2ConnectionId,
        reason: ConnectionCloseReason,
    },
}

#[derive(Clone, Debug)]
pub(crate) enum H2BodyPlan {
    Complete(Bytes),
    Gated {
        before: Bytes,
        gate: GateWaiter,
        after: Bytes,
    },
    ResetAfter {
        before: Bytes,
        gate: GateWaiter,
        reason: Reason,
    },
    AwaitClientReset {
        before: Bytes,
        expected_reason: Reason,
    },
}

impl H2BodyPlan {
    pub(crate) fn complete(body: impl Into<Bytes>) -> Self {
        Self::Complete(body.into())
    }

    pub(crate) fn gated(
        before: impl Into<Bytes>,
        gate: GateWaiter,
        after: impl Into<Bytes>,
    ) -> Self {
        Self::Gated {
            before: before.into(),
            gate,
            after: after.into(),
        }
    }

    pub(crate) fn reset_after(body: impl Into<Bytes>, gate: GateWaiter, reason: Reason) -> Self {
        Self::ResetAfter {
            before: body.into(),
            gate,
            reason,
        }
    }

    pub(crate) fn await_client_reset(body: impl Into<Bytes>, expected_reason: Reason) -> Self {
        Self::AwaitClientReset {
            before: body.into(),
            expected_reason,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct H2Response {
    status: StatusCode,
    body: H2BodyPlan,
}

impl H2Response {
    pub(crate) fn new(status: StatusCode) -> Self {
        Self {
            status,
            body: H2BodyPlan::complete(Bytes::new()),
        }
    }

    pub(crate) fn ok(body: impl Into<Bytes>) -> Self {
        Self::new(StatusCode::OK).body(H2BodyPlan::complete(body))
    }

    pub(crate) fn body(mut self, body: H2BodyPlan) -> Self {
        self.body = body;
        self
    }
}

#[derive(Clone, Debug)]
pub(crate) enum H2StreamScript {
    Respond(H2Response),
    Reset(Reason),
}

impl H2StreamScript {
    pub(crate) fn respond(response: H2Response) -> Self {
        Self::Respond(response)
    }

    pub(crate) fn reset(reason: Reason) -> Self {
        Self::Reset(reason)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct H2ConnectionScript {
    routes: HashMap<String, H2StreamScript>,
    fallback: Option<H2StreamScript>,
    allow_handshake_abandonment: bool,
}

impl H2ConnectionScript {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn route(mut self, path: impl Into<String>, script: H2StreamScript) -> Self {
        let path = path.into();
        assert!(
            self.routes.insert(path.clone(), script).is_none(),
            "duplicate H2 route {path:?}"
        );
        self
    }

    pub(crate) fn fallback(mut self, script: H2StreamScript) -> Self {
        self.fallback = Some(script);
        self
    }

    pub(crate) fn allow_handshake_abandonment(mut self) -> Self {
        self.allow_handshake_abandonment = true;
        self
    }

    fn script_for(&self, path: &str) -> Option<H2StreamScript> {
        self.routes.get(path).or(self.fallback.as_ref()).cloned()
    }
}

#[derive(Clone, Debug)]
pub(crate) enum H2ConnectionPlan {
    Queue(VecDeque<H2ConnectionScript>),
    Unbounded(H2ConnectionScript),
}

impl H2ConnectionPlan {
    pub(crate) fn queue(scripts: impl IntoIterator<Item = H2ConnectionScript>) -> Self {
        Self::Queue(scripts.into_iter().collect())
    }

    pub(crate) fn unbounded(script: H2ConnectionScript) -> Self {
        Self::Unbounded(script)
    }

    fn next_script(&mut self) -> Option<H2ConnectionScript> {
        match self {
            Self::Queue(scripts) => scripts.pop_front(),
            Self::Unbounded(script) => Some(script.clone()),
        }
    }
}

#[derive(Debug)]
struct RecordedState {
    events: Vec<H2Event>,
    failures: Vec<H2HarnessError>,
    generation: u64,
}

#[derive(Debug)]
struct SharedState {
    recorded: Mutex<RecordedState>,
    changed: watch::Sender<u64>,
    controls: Mutex<HashMap<H2ConnectionId, mpsc::UnboundedSender<ConnectionCommand>>>,
}

impl SharedState {
    fn new() -> Self {
        let (changed, _) = watch::channel(0);
        Self {
            recorded: Mutex::new(RecordedState {
                events: Vec::new(),
                failures: Vec::new(),
                generation: 0,
            }),
            changed,
            controls: Mutex::new(HashMap::new()),
        }
    }

    fn record_event(&self, event: H2Event) {
        let generation = {
            let mut recorded = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
            recorded.events.push(event);
            recorded.generation += 1;
            recorded.generation
        };
        self.changed.send_replace(generation);
    }

    fn record_failure(&self, failure: H2HarnessError) {
        let generation = {
            let mut recorded = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
            recorded.failures.push(failure);
            recorded.generation += 1;
            recorded.generation
        };
        self.changed.send_replace(generation);
    }

    fn failure(&self) -> Option<H2HarnessError> {
        let recorded = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
        match recorded.failures.as_slice() {
            [] => None,
            [failure] => Some(failure.clone()),
            failures => Some(H2HarnessError::new(format!(
                "{} H2 harness failures: {}",
                failures.len(),
                failures
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ))),
        }
    }

    fn events(&self) -> Vec<H2Event> {
        self.recorded
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .events
            .clone()
    }

    fn insert_control(
        &self,
        connection_id: H2ConnectionId,
        control: mpsc::UnboundedSender<ConnectionCommand>,
    ) {
        self.controls
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(connection_id, control);
    }

    fn remove_control(&self, connection_id: H2ConnectionId) {
        self.controls
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .remove(&connection_id);
    }

    fn control(
        &self,
        connection_id: H2ConnectionId,
    ) -> Option<mpsc::UnboundedSender<ConnectionCommand>> {
        self.controls
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .get(&connection_id)
            .cloned()
    }
}

#[derive(Debug)]
enum ConnectionCommand {
    GracefulShutdown(oneshot::Sender<()>),
}

#[derive(Debug, Default)]
pub(crate) struct H2ServerBuilder {
    connections: Option<H2ConnectionPlan>,
}

impl H2ServerBuilder {
    pub(crate) fn connections(mut self, connections: H2ConnectionPlan) -> Self {
        self.connections = Some(connections);
        self
    }

    pub(crate) async fn start(self) -> Result<H2TestServer, H2HarnessError> {
        let connections = self
            .connections
            .ok_or_else(|| H2HarnessError::new("an H2 connection plan is required"))?;
        let tls_acceptor = tls::server_tls_acceptor(&[b"h2"])
            .map_err(|err| H2HarnessError::new(format!("failed to configure TLS: {err}")))?;
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|err| H2HarnessError::new(format!("failed to bind H2 listener: {err}")))?;
        let listen_addr = listener.local_addr().map_err(|err| {
            H2HarnessError::new(format!("failed to read H2 listener address: {err}"))
        })?;
        let state = Arc::new(SharedState::new());
        let (shutdown, shutdown_rx) = watch::channel(false);
        let listener_state = state.clone();
        let listener_task = tokio::spawn(run_listener(
            listener,
            tls_acceptor,
            connections,
            listener_state,
            shutdown_rx,
        ));

        Ok(H2TestServer {
            listen_addr,
            state,
            shutdown,
            listener_task: Some(listener_task),
        })
    }
}

#[derive(Debug)]
pub(crate) struct H2TestServer {
    listen_addr: SocketAddr,
    state: Arc<SharedState>,
    shutdown: watch::Sender<bool>,
    listener_task: Option<JoinHandle<()>>,
}

impl H2TestServer {
    pub(crate) fn builder() -> H2ServerBuilder {
        H2ServerBuilder::default()
    }

    pub(crate) fn url(&self, path: &str) -> String {
        assert!(path.starts_with('/'), "H2 test path must start with '/'");
        format!("https://localhost:{}{path}", self.listen_addr.port())
    }

    pub(crate) fn events(&self) -> Vec<H2Event> {
        self.state.events()
    }

    pub(crate) fn connection_count(&self) -> usize {
        self.events()
            .iter()
            .filter(|event| matches!(event, H2Event::TcpAccepted { .. }))
            .count()
    }

    pub(crate) fn stream_count(&self) -> usize {
        self.events()
            .iter()
            .filter(|event| matches!(event, H2Event::StreamAccepted { .. }))
            .count()
    }

    pub(crate) async fn wait_for_event<F>(
        &self,
        timeout: Duration,
        predicate: F,
    ) -> Result<H2Event, H2HarnessError>
    where
        F: Fn(&H2Event) -> bool,
    {
        let mut changed = self.state.changed.subscribe();
        let wait = async {
            loop {
                if let Some(failure) = self.state.failure() {
                    return Err(failure);
                }
                if let Some(event) = self.state.events().into_iter().find(&predicate) {
                    return Ok(event);
                }
                changed.changed().await.map_err(|_| {
                    H2HarnessError::new("H2 event notification closed while waiting")
                })?;
            }
        };

        tokio::time::timeout(timeout, wait).await.map_err(|_| {
            H2HarnessError::new(format!("timed out after {timeout:?} waiting for H2 event"))
        })?
    }

    pub(crate) async fn send_graceful_goaway(
        &self,
        connection_id: H2ConnectionId,
    ) -> Result<(), H2HarnessError> {
        let control = self.state.control(connection_id).ok_or_else(|| {
            H2HarnessError::new(format!("H2 connection {connection_id:?} is not active"))
        })?;
        let (acknowledged, ack) = oneshot::channel();
        control
            .send(ConnectionCommand::GracefulShutdown(acknowledged))
            .map_err(|_| {
                H2HarnessError::new(format!(
                    "failed to send graceful shutdown to H2 connection {connection_id:?}"
                ))
            })?;
        tokio::time::timeout(WAIT, ack)
            .await
            .map_err(|_| {
                H2HarnessError::new(format!(
                    "timed out waiting for H2 connection {connection_id:?} to start graceful shutdown"
                ))
            })?
            .map_err(|_| {
                H2HarnessError::new(format!(
                    "H2 connection {connection_id:?} closed before graceful shutdown started"
                ))
            })
    }

    pub(crate) async fn shutdown(mut self) -> Result<(), H2HarnessError> {
        self.shutdown.send_replace(true);
        let mut listener_task = self
            .listener_task
            .take()
            .expect("listener task should exist until shutdown");
        match tokio::time::timeout(WAIT, &mut listener_task).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => {
                return Err(H2HarnessError::new(format!(
                    "H2 listener task failed: {err}"
                )))
            }
            Err(_) => {
                listener_task.abort();
                return Err(H2HarnessError::new(
                    "timed out waiting for H2 listener and connection tasks to stop",
                ));
            }
        }
        if let Some(failure) = self.state.failure() {
            return Err(failure);
        }
        Ok(())
    }
}

impl Drop for H2TestServer {
    fn drop(&mut self) {
        self.shutdown.send_replace(true);
        if let Some(listener_task) = self.listener_task.take() {
            listener_task.abort();
        }
    }
}

async fn run_listener(
    listener: TcpListener,
    tls_acceptor: TlsAcceptor,
    connections: H2ConnectionPlan,
    state: Arc<SharedState>,
    mut shutdown: watch::Receiver<bool>,
) {
    let connection_ids = AtomicU64::new(1);
    let connections = Mutex::new(connections);
    let mut connection_tasks = JoinSet::new();

    loop {
        tokio::select! {
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
            accepted = listener.accept() => {
                let (tcp_stream, peer_addr) = match accepted {
                    Ok(accepted) => accepted,
                    Err(err) => {
                        state.record_failure(H2HarnessError::new(format!(
                            "H2 listener accept failed: {err}"
                        )));
                        break;
                    }
                };
                let connection_id = H2ConnectionId(connection_ids.fetch_add(1, Ordering::Relaxed));
                state.record_event(H2Event::TcpAccepted {
                    connection_id,
                    peer_addr,
                });
                let script = connections
                    .lock()
                    .unwrap_or_else(|err| err.into_inner())
                    .next_script();
                let Some(script) = script else {
                    state.record_failure(H2HarnessError::new(format!(
                        "accepted unexpected H2 connection {connection_id:?} after the plan was exhausted"
                    )));
                    drop(tcp_stream);
                    continue;
                };
                let (control, control_rx) = mpsc::unbounded_channel();
                state.insert_control(connection_id, control);
                connection_tasks.spawn(run_connection(
                    tcp_stream,
                    tls_acceptor.clone(),
                    connection_id,
                    script,
                    state.clone(),
                    shutdown.clone(),
                    control_rx,
                ));
            }
            completed = connection_tasks.join_next(), if !connection_tasks.is_empty() => {
                record_connection_task_result(&state, completed);
            }
        }
    }

    while let Some(completed) = connection_tasks.join_next().await {
        record_connection_task_result(&state, Some(completed));
    }
}

fn record_connection_task_result(
    state: &SharedState,
    completed: Option<Result<Result<(), H2HarnessError>, tokio::task::JoinError>>,
) {
    match completed {
        Some(Ok(Ok(()))) | None => {}
        Some(Ok(Err(err))) => state.record_failure(err),
        Some(Err(err)) => state.record_failure(H2HarnessError::new(format!(
            "H2 connection task failed: {err}"
        ))),
    }
}

async fn run_connection(
    tcp_stream: TcpStream,
    tls_acceptor: TlsAcceptor,
    connection_id: H2ConnectionId,
    script: H2ConnectionScript,
    state: Arc<SharedState>,
    shutdown: watch::Receiver<bool>,
    control: mpsc::UnboundedReceiver<ConnectionCommand>,
) -> Result<(), H2HarnessError> {
    let result = drive_connection(
        tcp_stream,
        tls_acceptor,
        connection_id,
        script,
        state.clone(),
        shutdown,
        control,
    )
    .await;
    state.remove_control(connection_id);
    let reason = match &result {
        Ok(reason) => *reason,
        Err(_) => ConnectionCloseReason::ScriptFailed,
    };
    state.record_event(H2Event::ConnectionClosed {
        connection_id,
        reason,
    });
    result.map(|_| ())
}

async fn drive_connection(
    tcp_stream: TcpStream,
    tls_acceptor: TlsAcceptor,
    connection_id: H2ConnectionId,
    script: H2ConnectionScript,
    state: Arc<SharedState>,
    mut shutdown: watch::Receiver<bool>,
    mut control: mpsc::UnboundedReceiver<ConnectionCommand>,
) -> Result<ConnectionCloseReason, H2HarnessError> {
    let tls_stream = match tokio::time::timeout(WAIT, tls_acceptor.accept(tcp_stream)).await {
        Err(_) => return Err(H2HarnessError::new("timed out waiting for TLS handshake")),
        Ok(Ok(tls_stream)) => tls_stream,
        Ok(Err(err)) if script.allow_handshake_abandonment => {
            state.record_event(H2Event::TlsHandshakeAbandoned {
                connection_id,
                error: err.to_string(),
            });
            return Ok(ConnectionCloseReason::ClientClosed);
        }
        Ok(Err(err)) => return Err(H2HarnessError::new(format!("TLS handshake failed: {err}"))),
    };
    let alpn = tls_stream
        .get_ref()
        .1
        .alpn_protocol()
        .map(ToOwned::to_owned);
    state.record_event(H2Event::TlsNegotiated {
        connection_id,
        alpn: alpn.clone(),
    });
    if alpn.as_deref() != Some(b"h2") {
        return Err(H2HarnessError::new(format!(
            "connection {connection_id:?} negotiated unexpected ALPN {alpn:?}"
        )));
    }

    let observed = ObservedIo::new(tls_stream, connection_id, state.clone());
    let mut connection =
        match tokio::time::timeout(WAIT, h2::server::Builder::new().handshake(observed)).await {
            Err(_) => return Err(H2HarnessError::new("timed out waiting for H2 handshake")),
            Ok(Ok(connection)) => connection,
            Ok(Err(err)) if script.allow_handshake_abandonment => {
                state.record_event(H2Event::H2HandshakeAbandoned {
                    connection_id,
                    error: err.to_string(),
                });
                return Ok(ConnectionCloseReason::ClientClosed);
            }
            Ok(Err(err)) => return Err(H2HarnessError::new(format!("H2 handshake failed: {err}"))),
        };
    state.record_event(H2Event::H2Ready { connection_id });

    let mut stream_tasks = JoinSet::new();
    let mut shutting_down = false;
    let mut graceful_shutdown = false;
    let mut control_open = true;
    // Tracks whether the connection ended because the client closed it (accept returned None)
    // vs. a scripted GOAWAY completing.
    let mut client_initiated_close = false;

    loop {
        tokio::select! {
            changed = shutdown.changed(), if !shutting_down => {
                if changed.is_err() || *shutdown.borrow() {
                    shutting_down = true;
                    connection.abrupt_shutdown(Reason::NO_ERROR);
                }
            }
            command = control.recv(), if control_open => {
                match command {
                    Some(ConnectionCommand::GracefulShutdown(acknowledged)) => {
                        graceful_shutdown = true;
                        connection.graceful_shutdown();
                        let _ = acknowledged.send(());
                    }
                    None => control_open = false,
                }
            }
            accepted = connection.accept() => {
                match accepted {
                    Some(Ok((request, mut respond))) => {
                        let stream_id = respond.stream_id().as_u32();
                        let method = request.method().clone();
                        let path = request.uri().path().to_string();
                        state.record_event(H2Event::StreamAccepted {
                            connection_id,
                            stream_id,
                            method,
                            path: path.clone(),
                        });
                        let Some(stream_script) = script.script_for(&path) else {
                            respond.send_reset(Reason::PROTOCOL_ERROR);
                            return Err(H2HarnessError::new(format!(
                                "connection {connection_id:?} received unscripted H2 path {path:?}"
                            )));
                        };
                        // Scripts cover the response side only, so the request body is
                        // never read. Dropping the `RecvStream` marks the stream as no
                        // longer receiving: h2 then discards incoming DATA without
                        // charging the stream window and releases the connection
                        // capacity, so a client streaming a request body runs to
                        // completion. Holding it instead would charge the window on
                        // every DATA frame and never refund it, stalling the client
                        // once the send window is exhausted.
                        //
                        // TODO(test-utils): to script request bodies, pass
                        // `request.into_body()` into `run_stream` and drive it there,
                        // calling `release_capacity()` as chunks are consumed.
                        drop(request);
                        stream_tasks.spawn(run_stream(
                            connection_id,
                            stream_id,
                            stream_script,
                            respond,
                            state.clone(),
                        ));
                    }
                    Some(Err(err)) => {
                        if shutting_down
                            || (graceful_shutdown && err.reason() == Some(Reason::NO_ERROR))
                        {
                            break;
                        }
                        return Err(H2HarnessError::new(format!(
                            "H2 connection {connection_id:?} failed while accepting a stream: {err}"
                        )));
                    }
                    None => {
                        client_initiated_close = true;
                        break;
                    }
                }
            }
            completed = stream_tasks.join_next(), if !stream_tasks.is_empty() => {
                record_stream_task_result(&state, completed);
            }
        }
    }

    if shutting_down {
        stream_tasks.abort_all();
        while stream_tasks.join_next().await.is_some() {}
        Ok(ConnectionCloseReason::HarnessShutdown)
    } else {
        let drain = async {
            while let Some(completed) = stream_tasks.join_next().await {
                record_stream_task_result(&state, Some(completed));
            }
        };
        if tokio::time::timeout(WAIT, drain).await.is_err() {
            state.record_failure(H2HarnessError::new(format!(
                "H2 connection {connection_id:?} closed before its stream scripts completed"
            )));
            stream_tasks.abort_all();
            while stream_tasks.join_next().await.is_some() {}
        }
        if client_initiated_close {
            Ok(ConnectionCloseReason::ClientClosed)
        } else {
            Ok(ConnectionCloseReason::ScriptCompleted)
        }
    }
}

fn record_stream_task_result(
    state: &SharedState,
    completed: Option<Result<Result<(), H2HarnessError>, tokio::task::JoinError>>,
) {
    match completed {
        Some(Ok(Ok(()))) | None => {}
        Some(Ok(Err(err))) => state.record_failure(err),
        Some(Err(err)) => {
            state.record_failure(H2HarnessError::new(format!("H2 stream task failed: {err}")))
        }
    }
}

async fn run_stream(
    connection_id: H2ConnectionId,
    stream_id: u32,
    script: H2StreamScript,
    mut respond: SendResponse<Bytes>,
    state: Arc<SharedState>,
) -> Result<(), H2HarnessError> {
    match script {
        H2StreamScript::Reset(reason) => {
            respond.send_reset(reason);
            state.record_event(H2Event::ResetSent {
                connection_id,
                stream_id,
                reason,
            });
        }
        H2StreamScript::Respond(response) => {
            let end_stream =
                matches!(&response.body, H2BodyPlan::Complete(body) if body.is_empty());
            let response_head = Response::builder()
                .status(response.status)
                .body(())
                .map_err(|err| {
                    H2HarnessError::new(format!("failed to build H2 response: {err}"))
                })?;
            let mut send = respond
                .send_response(response_head, end_stream)
                .map_err(|err| H2HarnessError::new(format!("failed to send H2 response: {err}")))?;
            if end_stream {
                state.record_event(H2Event::ResponseCompleted {
                    connection_id,
                    stream_id,
                });
                return Ok(());
            }

            match response.body {
                H2BodyPlan::Complete(body) => {
                    send.send_data(body, true).map_err(|err| {
                        H2HarnessError::new(format!("failed to send H2 response body: {err}"))
                    })?;
                    state.record_event(H2Event::ResponseCompleted {
                        connection_id,
                        stream_id,
                    });
                }
                H2BodyPlan::Gated {
                    before,
                    gate,
                    after,
                } => {
                    if !before.is_empty() {
                        send.send_data(before, false).map_err(|err| {
                            H2HarnessError::new(format!(
                                "failed to send H2 response body prefix: {err}"
                            ))
                        })?;
                    }
                    gate.wait().await.map_err(|err| {
                        H2HarnessError::new(format!("H2 response gate failed: {err}"))
                    })?;
                    send.send_data(after, true).map_err(|err| {
                        H2HarnessError::new(format!(
                            "failed to send H2 response body suffix: {err}"
                        ))
                    })?;
                    state.record_event(H2Event::ResponseCompleted {
                        connection_id,
                        stream_id,
                    });
                }
                H2BodyPlan::ResetAfter {
                    before,
                    gate,
                    reason,
                } => {
                    if !before.is_empty() {
                        send.send_data(before, false).map_err(|err| {
                            H2HarnessError::new(format!(
                                "failed to send H2 response body prefix: {err}"
                            ))
                        })?;
                    }
                    gate.wait().await.map_err(|err| {
                        H2HarnessError::new(format!("H2 reset gate failed: {err}"))
                    })?;
                    send.send_reset(reason);
                    state.record_event(H2Event::ResetSent {
                        connection_id,
                        stream_id,
                        reason,
                    });
                }
                H2BodyPlan::AwaitClientReset {
                    before,
                    expected_reason,
                } => {
                    if !before.is_empty() {
                        send.send_data(before, false).map_err(|err| {
                            H2HarnessError::new(format!(
                                "failed to send H2 response body prefix: {err}"
                            ))
                        })?;
                    }
                    let reason = poll_fn(|cx| send.poll_reset(cx)).await.map_err(|err| {
                        H2HarnessError::new(format!("failed while awaiting client reset: {err}"))
                    })?;
                    if reason != expected_reason {
                        return Err(H2HarnessError::new(format!(
                            "expected client reset {expected_reason:?}, got {reason:?}"
                        )));
                    }
                    state.record_event(H2Event::ClientResetObserved {
                        connection_id,
                        stream_id,
                        reason,
                    });
                }
            }
        }
    }
    Ok(())
}

struct ObservedIo<T> {
    inner: T,
    observer: FrameObserver,
}

impl<T> ObservedIo<T> {
    fn new(inner: T, connection_id: H2ConnectionId, state: Arc<SharedState>) -> Self {
        Self {
            inner,
            observer: FrameObserver::new(connection_id, state),
        }
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for ObservedIo<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for ObservedIo<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match Pin::new(&mut self.inner).poll_write(cx, buf) {
            Poll::Ready(Ok(written)) => {
                self.observer.observe(&buf[..written]);
                Poll::Ready(Ok(written))
            }
            result => result,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

struct FrameObserver {
    connection_id: H2ConnectionId,
    state: Arc<SharedState>,
    pending: Vec<u8>,
}

impl FrameObserver {
    fn new(connection_id: H2ConnectionId, state: Arc<SharedState>) -> Self {
        Self {
            connection_id,
            state,
            pending: Vec::new(),
        }
    }

    fn observe(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
        loop {
            if self.pending.len() < H2_FRAME_HEADER_LENGTH {
                return;
            }
            let payload_length = ((self.pending[0] as usize) << 16)
                | ((self.pending[1] as usize) << 8)
                | self.pending[2] as usize;
            let frame_length = H2_FRAME_HEADER_LENGTH + payload_length;
            if self.pending.len() < frame_length {
                return;
            }
            if self.pending[3] == GOAWAY_FRAME_TYPE {
                if payload_length < GOAWAY_PAYLOAD_LENGTH {
                    self.state.record_failure(H2HarnessError::new(format!(
                        "connection {:?} emitted a GOAWAY frame with a {}-byte payload",
                        self.connection_id, payload_length
                    )));
                } else {
                    let payload = &self.pending[H2_FRAME_HEADER_LENGTH..frame_length];
                    let last_stream_id =
                        u32::from_be_bytes(payload[0..4].try_into().expect("four bytes"))
                            & GOAWAY_LAST_STREAM_ID_MASK;
                    let reason = Reason::from(u32::from_be_bytes(
                        payload[4..8].try_into().expect("four bytes"),
                    ));
                    self.state.record_event(H2Event::GoAwaySent {
                        connection_id: self.connection_id,
                        last_stream_id,
                        reason,
                    });
                }
            }
            self.pending.drain(..frame_length);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_observer_records_fragmented_goaway() {
        let state = Arc::new(SharedState::new());
        let connection_id = H2ConnectionId(7);
        let mut observer = FrameObserver::new(connection_id, state.clone());
        let mut frame = vec![0, 0, 8, GOAWAY_FRAME_TYPE, 0, 0, 0, 0, 0];
        frame.extend_from_slice(&3u32.to_be_bytes());
        frame.extend_from_slice(&u32::from(Reason::NO_ERROR).to_be_bytes());

        observer.observe(&frame[..5]);
        assert!(state.events().is_empty());
        observer.observe(&frame[5..]);

        assert_eq!(
            state.events(),
            vec![H2Event::GoAwaySent {
                connection_id,
                last_stream_id: 3,
                reason: Reason::NO_ERROR,
            }]
        );
    }

    #[test]
    fn test_connection_script_routes_by_path() {
        let script = H2ConnectionScript::new()
            .route("/reset", H2StreamScript::reset(Reason::CANCEL))
            .fallback(H2StreamScript::respond(H2Response::ok("ok")));

        assert!(matches!(
            script.script_for("/reset"),
            Some(H2StreamScript::Reset(Reason::CANCEL))
        ));
        assert!(matches!(
            script.script_for("/other"),
            Some(H2StreamScript::Respond(_))
        ));
    }
}
