/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(missing_docs)]

//! A deterministic harness for testing connection-level client behavior.
//!
//! Each endpoint assigns one complete [`ConnectionScript`] to each accepted
//! connection. An [`Http1Script`] parses requests and emits typed responses,
//! while a [`SocketScript`] runs ordered byte-level actions for malformed
//! framing, partial I/O, resets, and other transport behavior.
//!
//! [`ManualGate`] synchronizes a test with a script without relying on elapsed
//! time. Reaching a gate records an arrival and blocks the script until the
//! gate is released. Release is permanent, so current and future waiters all
//! pass.
//!
//! The harness records DNS lookups, accepted connections, parsed HTTP/1
//! requests, and connection closure. Its wait methods observe those events and
//! also surface failures from endpoint and connection tasks. Call
//! [`ConnectionTestHarness::shutdown`] at the end of a test to join those tasks
//! and report any background failure; dropping the harness only aborts them.
//!
//! # Script ownership
//!
//! Scripts describe connections, not requests. A queued endpoint plan moves
//! one script to each connection in order. Repeated plans clone a complete
//! script for each connection. Requests served over one keep-alive connection
//! remain within that connection's script.
//!
//! # Extension via `SocketScript`
//!
//! [`SocketScript`] is the extension point for framing the typed API does not
//! model; interim 1xx exchanges, for example, are expressible by adding
//! byte-level actions without changing existing scripts.

use aws_smithy_runtime_api::client::dns::{DnsFuture, ResolveDns, ResolveDnsError};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;
use tokio::task::{JoinHandle, JoinSet};

const MAX_HTTP1_HEADER_BYTES: usize = 64 * 1024;
const MAX_HTTP1_BODY_BYTES: usize = 8 * 1024 * 1024;
const READ_CHUNK_SIZE: usize = 8 * 1024;

/// An error produced by the connection test harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessError {
    message: Arc<str>,
}

impl HarnessError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: Arc::from(message.into()),
        }
    }
}

impl fmt::Display for HarnessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for HarnessError {}

/// The identity assigned to an accepted connection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Returns the numeric connection identity.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Why a connection task stopped.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionCloseReason {
    /// The peer closed the connection.
    ClientClosed,
    /// The script reached its end or explicitly closed the connection.
    ScriptCompleted,
    /// The script reset the connection.
    Reset,
    /// The harness was shut down.
    HarnessShutdown,
    /// The script failed.
    ScriptFailed,
}

/// An event recorded by the harness.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectionEvent {
    /// A DNS lookup was performed.
    DnsLookup {
        /// The requested hostname.
        hostname: String,
    },
    /// A TCP connection was accepted.
    TcpAccepted {
        /// The accepted connection's identity.
        connection_id: ConnectionId,
        /// The endpoint that accepted the connection.
        endpoint_addr: SocketAddr,
    },
    /// A complete HTTP/1 request was received.
    Http1Request {
        /// The connection carrying the request.
        connection_id: ConnectionId,
        /// The endpoint that accepted the connection.
        endpoint_addr: SocketAddr,
        /// The request method.
        method: String,
        /// The request target from the request line.
        target: String,
        /// The Host header, when present.
        host: Option<String>,
    },
    /// A connection task stopped.
    ConnectionClosed {
        /// The connection that stopped.
        connection_id: ConnectionId,
        /// Why the connection stopped.
        reason: ConnectionCloseReason,
    },
}

#[derive(Debug)]
struct RecordedState {
    events: Vec<ConnectionEvent>,
    failures: Vec<HarnessError>,
    generation: u64,
}

// Mutations advance a generation and notify watchers after releasing the
// mutex. This lets event waits sleep without holding the recorded state.
#[derive(Debug)]
struct SharedState {
    recorded: Mutex<RecordedState>,
    changed: watch::Sender<u64>,
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
        }
    }

    fn record_event(&self, event: ConnectionEvent) {
        let generation = {
            let mut state = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
            state.events.push(event);
            state.generation += 1;
            state.generation
        };
        self.changed.send_replace(generation);
    }

    fn record_failure(&self, failure: HarnessError) {
        let generation = {
            let mut state = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
            state.failures.push(failure);
            state.generation += 1;
            state.generation
        };
        self.changed.send_replace(generation);
    }

    fn events(&self) -> Vec<ConnectionEvent> {
        self.recorded
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .events
            .clone()
    }

    fn failure(&self) -> Option<HarnessError> {
        let state = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
        match state.failures.as_slice() {
            [] => None,
            [failure] => Some(failure.clone()),
            failures => Some(HarnessError::new(format!(
                "{} harness failures: {}",
                failures.len(),
                failures
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; ")
            ))),
        }
    }

    async fn wait_for<F>(
        &self,
        description: &str,
        timeout: Duration,
        predicate: F,
    ) -> Result<(), HarnessError>
    where
        F: Fn(&[ConnectionEvent]) -> bool,
    {
        let mut changed = self.changed.subscribe();
        let wait = async {
            loop {
                {
                    let state = self.recorded.lock().unwrap_or_else(|err| err.into_inner());
                    if let Some(failure) = state.failures.first() {
                        return Err(failure.clone());
                    }
                    if predicate(&state.events) {
                        return Ok(());
                    }
                }
                changed.changed().await.map_err(|_| {
                    HarnessError::new(format!(
                        "event notification closed while waiting for {description}"
                    ))
                })?;
            }
        };

        tokio::time::timeout(timeout, wait).await.map_err(|_| {
            HarnessError::new(format!(
                "timed out after {timeout:?} waiting for {description}"
            ))
        })?
    }
}

/// A one-shot broadcast gate used to synchronize a test with one or more scripts.
///
/// Each script-side wait records an arrival. [`ManualGate::release`] is
/// idempotent and lets both current and future waiters proceed.
#[derive(Clone, Debug)]
pub struct ManualGate {
    state: Arc<GateState>,
}

#[derive(Debug)]
struct GateState {
    snapshot: watch::Sender<GateSnapshot>,
}

#[derive(Clone, Copy, Debug)]
struct GateSnapshot {
    arrivals: usize,
    released: bool,
}

impl ManualGate {
    /// Creates an unreleased gate with no arrivals.
    pub fn new() -> Self {
        let (snapshot, _) = watch::channel(GateSnapshot {
            arrivals: 0,
            released: false,
        });
        Self {
            state: Arc::new(GateState { snapshot }),
        }
    }

    /// Returns a script-side waiter that shares this gate's state.
    pub fn waiter(&self) -> GateWaiter {
        GateWaiter {
            state: self.state.clone(),
        }
    }

    /// Returns the number of calls to [`GateWaiter::wait`] that have reached the gate.
    ///
    /// Calling `wait` more than once on the same waiter records each arrival.
    pub fn arrivals(&self) -> usize {
        self.state.snapshot.borrow().arrivals
    }

    /// Waits up to `timeout` for at least one script to reach the gate.
    pub async fn wait_until_reached(&self, timeout: Duration) -> Result<(), HarnessError> {
        self.wait_for_arrivals(1, timeout).await
    }

    /// Waits up to `timeout` for at least `expected` script-side waits to arrive.
    pub async fn wait_for_arrivals(
        &self,
        expected: usize,
        timeout: Duration,
    ) -> Result<(), HarnessError> {
        let mut snapshot = self.state.snapshot.subscribe();
        let wait = async {
            loop {
                if snapshot.borrow().arrivals >= expected {
                    return Ok(());
                }
                snapshot.changed().await.map_err(|_| {
                    HarnessError::new("gate notification closed while waiting for arrivals")
                })?;
            }
        };

        tokio::time::timeout(timeout, wait).await.map_err(|_| {
            HarnessError::new(format!(
                "timed out after {timeout:?} waiting for {expected} gate arrivals; observed {}",
                self.arrivals()
            ))
        })?
    }

    /// Permanently releases every current and future waiter.
    pub fn release(&self) {
        self.state
            .snapshot
            .send_modify(|snapshot| snapshot.released = true);
    }
}

impl Default for ManualGate {
    fn default() -> Self {
        Self::new()
    }
}

/// A script-side handle for a [`ManualGate`].
///
/// A waiter owns the shared gate state, so dropping the [`ManualGate`]
/// controller does not cancel an outstanding wait.
#[derive(Clone, Debug)]
pub struct GateWaiter {
    state: Arc<GateState>,
}

impl GateWaiter {
    /// Records one arrival and waits until the gate is released.
    ///
    /// This wait has no timeout. The controlling test should use
    /// [`ManualGate::wait_until_reached`] or [`ManualGate::wait_for_arrivals`]
    /// with a timeout before releasing the gate.
    pub async fn wait(&self) -> Result<(), HarnessError> {
        let mut snapshot = self.state.snapshot.subscribe();
        self.state
            .snapshot
            .send_modify(|snapshot| snapshot.arrivals += 1);
        loop {
            if snapshot.borrow().released {
                return Ok(());
            }
            snapshot
                .changed()
                .await
                .map_err(|_| HarnessError::new("gate notification closed before release"))?;
        }
    }
}

/// What a finite HTTP/1 script does after sending its final response.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Finish {
    /// Wait for the client to close and fail if it sends another request.
    #[default]
    AwaitClientClose,
    /// Close the connection normally.
    Close,
    /// Reset the connection.
    Reset,
}

/// A complete HTTP/1 response body, optionally paused at a gate.
///
/// A gated body advertises the combined byte length of its parts. The script
/// writes the bytes before the gate, records an arrival, waits for release,
/// and then writes the remaining bytes.
#[derive(Clone, Debug)]
pub struct BodyPlan {
    parts: Vec<BodyPart>,
    length: usize,
}

#[derive(Clone, Debug)]
enum BodyPart {
    Bytes(Vec<u8>),
    Wait(GateWaiter),
}

impl BodyPlan {
    /// Creates a complete response body.
    pub fn complete(body: impl AsRef<[u8]>) -> Self {
        let body = body.as_ref().to_vec();
        Self {
            length: body.len(),
            parts: vec![BodyPart::Bytes(body)],
        }
    }

    /// Creates a body that pauses after `before` and resumes with `after`.
    pub fn split_at_gate(
        before: impl AsRef<[u8]>,
        gate: GateWaiter,
        after: impl AsRef<[u8]>,
    ) -> Self {
        let before = before.as_ref().to_vec();
        let after = after.as_ref().to_vec();
        Self {
            length: before.len() + after.len(),
            parts: vec![
                BodyPart::Bytes(before),
                BodyPart::Wait(gate),
                BodyPart::Bytes(after),
            ],
        }
    }
}

impl Default for BodyPlan {
    fn default() -> Self {
        Self::complete([])
    }
}

/// One HTTP/1 response emitted by a script.
#[derive(Clone, Debug)]
pub struct Http1Response {
    status: u16,
    headers: Vec<(String, String)>,
    body: BodyPlan,
    close: bool,
}

impl Http1Response {
    /// Creates a `200 OK` response with an empty body.
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Creates a response with the given status and an empty body.
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: BodyPlan::default(),
            close: false,
        }
    }

    /// Adds a response header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Sets a complete response body.
    pub fn body(mut self, body: impl AsRef<[u8]>) -> Self {
        self.body = BodyPlan::complete(body);
        self
    }

    /// Sets a response body plan.
    pub fn body_plan(mut self, body: BodyPlan) -> Self {
        self.body = body;
        self
    }

    /// Sends `Connection: close` and closes after this response.
    pub fn connection_close(mut self) -> Self {
        self.close = true;
        self
    }

    fn validate(&self) -> Result<(), HarnessError> {
        http_1x::StatusCode::from_u16(self.status)
            .map_err(|_| HarnessError::new(format!("invalid HTTP status {}", self.status)))?;
        for (name, value) in &self.headers {
            if name.is_empty() || name.contains(['\r', '\n', ':']) || value.contains(['\r', '\n']) {
                return Err(HarnessError::new(format!(
                    "invalid HTTP response header {name:?}: {value:?}"
                )));
            }
            if name.eq_ignore_ascii_case("content-length")
                || name.eq_ignore_ascii_case("connection")
            {
                return Err(HarnessError::new(format!(
                    "{name} is managed by Http1Response; use SocketScript for raw framing"
                )));
            }
        }
        Ok(())
    }

    fn actions(&self) -> Vec<Action> {
        let reason = http_1x::StatusCode::from_u16(self.status)
            .ok()
            .and_then(|code| code.canonical_reason())
            .unwrap_or("Response");
        let mut head = String::new();
        let _ = write!(
            head,
            "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: {}\r\n",
            self.status,
            reason,
            self.body.length,
            if self.close { "close" } else { "keep-alive" }
        );
        for (name, value) in &self.headers {
            let _ = write!(head, "{name}: {value}\r\n");
        }
        head.push_str("\r\n");

        let mut actions = vec![Action::WriteAll(head.into_bytes())];
        for part in &self.body.parts {
            match part {
                BodyPart::Bytes(bytes) if !bytes.is_empty() => {
                    actions.push(Action::WriteAll(bytes.clone()));
                }
                BodyPart::Bytes(_) => {}
                BodyPart::Wait(waiter) => actions.push(Action::Wait(waiter.clone())),
            }
        }
        if self.close {
            actions.push(Action::Close);
        }
        actions
    }
}

/// A typed HTTP/1 script for one connection.
///
/// Each response consumes and records one complete request before it is sent.
/// Request parsing supports fixed bodies framed by `Content-Length`; use
/// [`SocketScript`] when testing transfer encoding or custom framing. Request
/// headers are limited to 64 fields and 64 KiB, and bodies are limited to 8 MiB,
/// so malformed input cannot grow the harness without bound.
#[derive(Clone, Debug)]
pub struct Http1Script {
    responses: Http1Responses,
    finish: Finish,
}

#[derive(Clone, Debug)]
enum Http1Responses {
    Finite(Vec<Http1Response>),
    Repeated(Http1Response),
}

impl Http1Script {
    /// Creates a finite script with no responses.
    ///
    /// Unless a different [`Finish`] is selected, the script waits for the
    /// client to close without sending a request.
    pub fn new() -> Self {
        Self {
            responses: Http1Responses::Finite(Vec::new()),
            finish: Finish::default(),
        }
    }

    /// Creates a finite sequence that serves one response per request.
    ///
    /// Responses are emitted in iteration order.
    pub fn responses<I>(responses: I) -> Self
    where
        I: IntoIterator<Item = Http1Response>,
    {
        Self {
            responses: Http1Responses::Finite(responses.into_iter().collect()),
            finish: Finish::default(),
        }
    }

    /// Serves the same response for every request until the client closes.
    pub fn serve(response: Http1Response) -> Self {
        Self {
            responses: Http1Responses::Repeated(response),
            finish: Finish::default(),
        }
    }

    /// Appends one response to a finite script.
    ///
    /// # Panics
    ///
    /// Panics if called on a repeating script created with [`Http1Script::serve`],
    /// which already answers every request.
    pub fn respond(mut self, response: Http1Response) -> Self {
        match &mut self.responses {
            Http1Responses::Finite(responses) => responses.push(response),
            Http1Responses::Repeated(_) => panic!(
                "cannot append a response to a repeating Http1Script (created with Http1Script::serve)"
            ),
        }
        self
    }

    /// Selects what happens after the final finite response.
    ///
    /// # Panics
    ///
    /// Panics if called on a repeating script created with [`Http1Script::serve`],
    /// which runs until the client closes and so has no final response.
    pub fn finish(mut self, finish: Finish) -> Self {
        assert!(
            !matches!(&self.responses, Http1Responses::Repeated(_)),
            "cannot set a finite finish policy on a repeating Http1Script (created with Http1Script::serve)"
        );
        self.finish = finish;
        self
    }

    fn validate(&self) -> Result<(), HarnessError> {
        match &self.responses {
            Http1Responses::Finite(responses) => {
                for (index, response) in responses.iter().enumerate() {
                    response.validate()?;
                    if response.close && index + 1 != responses.len() {
                        return Err(HarnessError::new(
                            "a connection-closing response must be the final response",
                        ));
                    }
                }
                if responses.last().is_some_and(|response| response.close)
                    && self.finish != Finish::AwaitClientClose
                {
                    return Err(HarnessError::new(
                        "a connection-closing response cannot also have a finish policy",
                    ));
                }
            }
            Http1Responses::Repeated(response) => {
                response.validate()?;
            }
        }
        Ok(())
    }
}

impl Default for Http1Script {
    fn default() -> Self {
        Self::new()
    }
}

/// An ordered sequence of low-level socket actions for one connection.
///
/// Actions execute in insertion order. [`SocketScript::await_client_close`],
/// [`SocketScript::close`], and [`SocketScript::reset`] are terminal and must
/// be the final action. Reaching the end without a terminal action completes
/// the script and closes the socket normally.
#[derive(Clone, Debug, Default)]
pub struct SocketScript {
    actions: Vec<Action>,
}

impl SocketScript {
    /// Creates an empty script.
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads and records one complete fixed-length HTTP/1 request.
    ///
    /// This uses the same bounded, `Content-Length`-only parser as
    /// [`Http1Script`].
    pub fn read_http1_request(mut self) -> Self {
        self.actions.push(Action::ReadHttp1Request);
        self
    }

    /// Reads and discards through `delimiter`.
    ///
    /// The action fails if the delimiter is not found within `limit` bytes.
    pub fn read_until(mut self, delimiter: impl AsRef<[u8]>, limit: usize) -> Self {
        self.actions.push(Action::ReadUntil {
            delimiter: delimiter.as_ref().to_vec(),
            limit,
        });
        self
    }

    /// Reads and discards exactly `length` bytes, including any buffered bytes.
    pub fn read_exact(mut self, length: usize) -> Self {
        self.actions.push(Action::ReadExact(length));
        self
    }

    /// Reads exactly `expected` and fails if the bytes differ.
    pub fn expect_bytes(mut self, expected: impl AsRef<[u8]>) -> Self {
        self.actions
            .push(Action::ExpectBytes(expected.as_ref().to_vec()));
        self
    }

    /// Writes all given bytes.
    pub fn write_all(mut self, bytes: impl AsRef<[u8]>) -> Self {
        self.actions.push(Action::WriteAll(bytes.as_ref().to_vec()));
        self
    }

    /// Records an arrival at `gate` and waits without a timeout for release.
    pub fn wait(mut self, gate: GateWaiter) -> Self {
        self.actions.push(Action::Wait(gate));
        self
    }

    /// Delays the next action by `duration`.
    ///
    /// Prefer [`ManualGate`] when elapsed time is not itself under test.
    pub fn delay(mut self, duration: Duration) -> Self {
        self.actions.push(Action::Delay(duration));
        self
    }

    /// Shuts down the socket's write half.
    pub fn shutdown_write(mut self) -> Self {
        self.actions.push(Action::ShutdownWrite);
        self
    }

    /// Waits for the client to close and fails if it sends more bytes.
    ///
    /// This is a terminal action.
    ///
    /// Bytes already buffered by a preceding read also cause this action to fail.
    pub fn await_client_close(mut self) -> Self {
        self.actions.push(Action::AwaitClientClose);
        self
    }

    /// Closes the connection normally as the script's terminal action.
    pub fn close(mut self) -> Self {
        self.actions.push(Action::Close);
        self
    }

    /// Resets the connection using `SO_LINGER=0` as the script's terminal action.
    pub fn reset(mut self) -> Self {
        self.actions.push(Action::Reset);
        self
    }

    fn validate(&self) -> Result<(), HarnessError> {
        for (index, action) in self.actions.iter().enumerate() {
            if let Action::ReadUntil { delimiter, limit } = action {
                if delimiter.is_empty() {
                    return Err(HarnessError::new(
                        "SocketScript::read_until delimiter must not be empty",
                    ));
                }
                if *limit < delimiter.len() {
                    return Err(HarnessError::new(
                        "SocketScript::read_until limit is shorter than its delimiter",
                    ));
                }
            }
            if matches!(action, Action::AwaitClientClose) && index + 1 != self.actions.len() {
                return Err(HarnessError::new(
                    "SocketScript::await_client_close must be the final action",
                ));
            }
            if matches!(action, Action::Close | Action::Reset) && index + 1 != self.actions.len() {
                return Err(HarnessError::new(
                    "SocketScript close and reset actions must be final",
                ));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum Action {
    ReadHttp1Request,
    ReadUntil { delimiter: Vec<u8>, limit: usize },
    ReadExact(usize),
    ExpectBytes(Vec<u8>),
    WriteAll(Vec<u8>),
    Wait(GateWaiter),
    Delay(Duration),
    ShutdownWrite,
    AwaitClientClose,
    Close,
    Reset,
}

/// A complete high-level or byte-level script for one accepted connection.
///
/// Convert an [`Http1Script`] or [`SocketScript`] directly when no explicit
/// distinction is needed at the call site.
#[derive(Clone, Debug)]
pub struct ConnectionScript {
    kind: ConnectionScriptKind,
}

#[derive(Clone, Debug)]
enum ConnectionScriptKind {
    Http1(Http1Script),
    Socket(SocketScript),
}

impl ConnectionScript {
    /// Creates a high-level HTTP/1 script.
    pub fn http1(script: Http1Script) -> Self {
        Self {
            kind: ConnectionScriptKind::Http1(script),
        }
    }

    /// Creates a low-level socket script.
    pub fn socket(script: SocketScript) -> Self {
        Self {
            kind: ConnectionScriptKind::Socket(script),
        }
    }

    fn validate(&self) -> Result<(), HarnessError> {
        match &self.kind {
            ConnectionScriptKind::Http1(script) => script.validate(),
            ConnectionScriptKind::Socket(script) => script.validate(),
        }
    }
}

impl From<Http1Script> for ConnectionScript {
    fn from(script: Http1Script) -> Self {
        Self::http1(script)
    }
}

impl From<SocketScript> for ConnectionScript {
    fn from(script: SocketScript) -> Self {
        Self::socket(script)
    }
}

/// Assigns a complete [`ConnectionScript`] to each accepted connection.
///
/// Queue plans consume scripts in order, while repeated plans clone a script
/// for each connection. Accepting more connections than a finite plan provides
/// is a harness failure: the extra connection is closed, and the failure is
/// returned by event waits and [`ConnectionTestHarness::shutdown`].
#[derive(Clone, Debug)]
pub struct EndpointPlan {
    kind: EndpointPlanKind,
}

#[derive(Clone, Debug)]
enum EndpointPlanKind {
    Queue(VecDeque<ConnectionScript>),
    Repeat {
        script: ConnectionScript,
        remaining: Option<usize>,
    },
}

impl EndpointPlan {
    /// Assigns one complete script to each accepted connection in iteration order.
    pub fn queue<I, S>(scripts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<ConnectionScript>,
    {
        Self {
            kind: EndpointPlanKind::Queue(scripts.into_iter().map(Into::into).collect()),
        }
    }

    /// Assigns a clone of `script` to exactly `accepts` connections.
    pub fn repeat_n(accepts: usize, script: impl Into<ConnectionScript>) -> Self {
        Self {
            kind: EndpointPlanKind::Repeat {
                script: script.into(),
                remaining: Some(accepts),
            },
        }
    }

    /// Assigns a clone of `script` to every accepted connection.
    pub fn unbounded(script: impl Into<ConnectionScript>) -> Self {
        Self {
            kind: EndpointPlanKind::Repeat {
                script: script.into(),
                remaining: None,
            },
        }
    }

    fn next_script(&mut self) -> Option<ConnectionScript> {
        match &mut self.kind {
            EndpointPlanKind::Queue(scripts) => scripts.pop_front(),
            EndpointPlanKind::Repeat { script, remaining } => match remaining {
                Some(0) => None,
                Some(remaining) => {
                    *remaining -= 1;
                    Some(script.clone())
                }
                None => Some(script.clone()),
            },
        }
    }

    fn validate(&self) -> Result<(), HarnessError> {
        match &self.kind {
            EndpointPlanKind::Queue(scripts) => {
                for script in scripts {
                    script.validate()?;
                }
            }
            EndpointPlanKind::Repeat { script, .. } => script.validate()?,
        }
        Ok(())
    }
}

impl From<ConnectionScript> for EndpointPlan {
    fn from(script: ConnectionScript) -> Self {
        Self::queue([script])
    }
}

impl From<Http1Script> for EndpointPlan {
    fn from(script: Http1Script) -> Self {
        ConnectionScript::from(script).into()
    }
}

impl From<SocketScript> for EndpointPlan {
    fn from(script: SocketScript) -> Self {
        ConnectionScript::from(script).into()
    }
}

/// A bound endpoint managed by a [`ConnectionTestHarness`].
#[derive(Debug)]
pub struct TestEndpoint {
    addr: SocketAddr,
}

impl TestEndpoint {
    /// Returns the endpoint's IP address.
    pub fn ip(&self) -> IpAddr {
        self.addr.ip()
    }

    /// Returns the endpoint's TCP port.
    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    /// Returns the endpoint's socket address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns an HTTP URL for the endpoint.
    pub fn endpoint_url(&self) -> String {
        format!("http://{}/", self.addr)
    }
}

/// A DNS resolver backed by entries configured on the harness.
///
/// Every lookup records a [`ConnectionEvent::DnsLookup`] regardless of
/// outcome. Names without a configured entry return a [`ResolveDnsError`]
/// so that hostname typos in tests surface immediately rather than
/// manifesting as confusing downstream connect failures.
#[derive(Clone, Debug)]
pub struct MockDnsResolver {
    entries: Arc<HashMap<String, Vec<IpAddr>>>,
    state: Arc<SharedState>,
}

impl ResolveDns for MockDnsResolver {
    fn resolve_dns<'a>(&'a self, name: &'a str) -> DnsFuture<'a> {
        self.state.record_event(ConnectionEvent::DnsLookup {
            hostname: name.to_owned(),
        });
        match self.entries.get(name) {
            Some(addrs) => DnsFuture::ready(Ok(addrs.clone())),
            None => DnsFuture::ready(Err(ResolveDnsError::new(std::io::Error::other(format!(
                "no DNS entry configured for {name:?}"
            ))))),
        }
    }
}

/// Configures endpoints and DNS entries for a [`ConnectionTestHarness`].
///
/// Endpoints bind in configuration order. The first endpoint chooses an
/// ephemeral port, and every later endpoint binds that same port.
#[derive(Debug, Default)]
pub struct HarnessBuilder {
    endpoints: Vec<EndpointConfig>,
    dns: Vec<DnsConfig>,
}

#[derive(Debug)]
struct EndpointConfig {
    ip: IpAddr,
    plan: EndpointPlan,
}

#[derive(Debug)]
enum DnsConfig {
    Explicit(String, Vec<IpAddr>),
    All(String),
}

impl HarnessBuilder {
    /// Adds a scripted TCP endpoint at `ip`.
    ///
    /// The endpoint receives the next script from `plan` for each connection.
    pub fn endpoint(mut self, ip: IpAddr, plan: impl Into<EndpointPlan>) -> Self {
        self.endpoints.push(EndpointConfig {
            ip,
            plan: plan.into(),
        });
        self
    }

    /// Maps `hostname` to the given addresses in iteration order.
    ///
    /// Unregistered names produce a [`ResolveDnsError`] at resolution time.
    pub fn dns<I>(mut self, hostname: impl Into<String>, ips: I) -> Self
    where
        I: IntoIterator<Item = IpAddr>,
    {
        self.dns.push(DnsConfig::Explicit(
            hostname.into(),
            ips.into_iter().collect(),
        ));
        self
    }

    /// Maps `hostname` to every configured endpoint address in endpoint order.
    ///
    /// Unregistered names produce a [`ResolveDnsError`] at resolution time.
    pub fn dns_all(mut self, hostname: impl Into<String>) -> Self {
        self.dns.push(DnsConfig::All(hostname.into()));
        self
    }

    /// Binds all endpoints and starts their background tasks.
    pub async fn build(self) -> Result<ConnectionTestHarness, HarnessError> {
        if self.endpoints.is_empty() {
            return Err(HarnessError::new(
                "a connection test harness requires at least one endpoint",
            ));
        }
        for config in &self.endpoints {
            config.plan.validate()?;
        }

        let mut bound = Vec::with_capacity(self.endpoints.len());
        let mut port = 0;
        for config in self.endpoints {
            let requested = SocketAddr::new(config.ip, port);
            let listener = TcpListener::bind(requested).await.map_err(|err| {
                HarnessError::new(format!("failed to bind endpoint {requested}: {err}"))
            })?;
            let addr = listener.local_addr().map_err(|err| {
                HarnessError::new(format!("failed to read endpoint address: {err}"))
            })?;
            if port == 0 {
                port = addr.port();
            }
            bound.push((listener, addr, config.plan));
        }

        let state = Arc::new(SharedState::new());
        let next_connection_id = Arc::new(AtomicU64::new(1));
        let (shutdown, _) = watch::channel(false);
        let mut endpoints = Vec::with_capacity(bound.len());
        let mut endpoint_tasks = Vec::with_capacity(bound.len());
        for (listener, addr, plan) in bound {
            endpoints.push(TestEndpoint { addr });
            endpoint_tasks.push(tokio::spawn(run_endpoint(
                listener,
                addr,
                plan,
                state.clone(),
                next_connection_id.clone(),
                shutdown.subscribe(),
            )));
        }

        let all_ips = endpoints.iter().map(TestEndpoint::ip).collect::<Vec<_>>();
        let mut dns_entries = HashMap::new();
        for config in self.dns {
            match config {
                DnsConfig::Explicit(hostname, ips) => {
                    dns_entries.insert(hostname, ips);
                }
                DnsConfig::All(hostname) => {
                    dns_entries.insert(hostname, all_ips.clone());
                }
            }
        }
        let dns_resolver = MockDnsResolver {
            entries: Arc::new(dns_entries),
            state: state.clone(),
        };

        Ok(ConnectionTestHarness {
            endpoints,
            state,
            dns_resolver,
            shutdown,
            endpoint_tasks,
        })
    }
}

/// Running scripted endpoints with recorded connection events and mock DNS.
///
/// Endpoint tasks own all accepted connection tasks. Use
/// [`ConnectionTestHarness::shutdown`] to stop and join them and to surface
/// script failures. Dropping the harness requests shutdown and aborts endpoint
/// tasks without waiting for their result.
#[derive(Debug)]
pub struct ConnectionTestHarness {
    endpoints: Vec<TestEndpoint>,
    state: Arc<SharedState>,
    dns_resolver: MockDnsResolver,
    shutdown: watch::Sender<bool>,
    endpoint_tasks: Vec<JoinHandle<()>>,
}

impl ConnectionTestHarness {
    /// Creates a harness builder.
    pub fn builder() -> HarnessBuilder {
        HarnessBuilder::default()
    }

    /// Returns all configured endpoints.
    pub fn endpoints(&self) -> &[TestEndpoint] {
        &self.endpoints
    }

    /// Returns an endpoint by configuration order.
    pub fn endpoint(&self, index: usize) -> Option<&TestEndpoint> {
        self.endpoints.get(index)
    }

    /// Returns the TCP port shared by all endpoints.
    pub fn port(&self) -> u16 {
        self.endpoints[0].port()
    }

    /// Returns an HTTP URL for the first endpoint.
    pub fn endpoint_url(&self) -> String {
        self.endpoints[0].endpoint_url()
    }

    /// Returns a clone of the configured DNS resolver.
    pub fn dns_resolver(&self) -> MockDnsResolver {
        self.dns_resolver.clone()
    }

    /// Returns a snapshot of all events recorded so far.
    ///
    /// Events remain ordered by when they were recorded across all endpoints.
    pub fn events(&self) -> Vec<ConnectionEvent> {
        self.state.events()
    }

    /// Returns the number of accepted TCP connections.
    pub fn tcp_accepted_count(&self) -> usize {
        self.events()
            .iter()
            .filter(|event| matches!(event, ConnectionEvent::TcpAccepted { .. }))
            .count()
    }

    /// Returns the number of accepted TCP connections for `ip`.
    pub fn tcp_accepted_by(&self, ip: IpAddr) -> usize {
        self.events()
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    ConnectionEvent::TcpAccepted { endpoint_addr, .. }
                        if endpoint_addr.ip() == ip
                )
            })
            .count()
    }

    /// Returns the number of DNS lookups.
    pub fn dns_lookup_count(&self) -> usize {
        self.events()
            .iter()
            .filter(|event| matches!(event, ConnectionEvent::DnsLookup { .. }))
            .count()
    }

    /// Returns recorded HTTP request targets and Host headers.
    pub fn http_requests(&self) -> Vec<(String, Option<String>)> {
        self.events()
            .into_iter()
            .filter_map(|event| match event {
                ConnectionEvent::Http1Request { target, host, .. } => Some((target, host)),
                _ => None,
            })
            .collect()
    }

    /// Waits up to `timeout` for at least `expected` accepted TCP connections.
    ///
    /// A background harness failure is returned immediately.
    pub async fn wait_for_tcp_accepts(
        &self,
        expected: usize,
        timeout: Duration,
    ) -> Result<(), HarnessError> {
        self.state
            .wait_for("TCP accepts", timeout, |events| {
                events
                    .iter()
                    .filter(|event| matches!(event, ConnectionEvent::TcpAccepted { .. }))
                    .count()
                    >= expected
            })
            .await
    }

    /// Waits up to `timeout` for at least `expected` complete HTTP/1 requests.
    ///
    /// A background harness failure is returned immediately.
    pub async fn wait_for_http_requests(
        &self,
        expected: usize,
        timeout: Duration,
    ) -> Result<(), HarnessError> {
        self.state
            .wait_for("HTTP/1 requests", timeout, |events| {
                events
                    .iter()
                    .filter(|event| matches!(event, ConnectionEvent::Http1Request { .. }))
                    .count()
                    >= expected
            })
            .await
    }

    /// Waits up to `timeout` until an event matches `predicate`.
    ///
    /// A background harness failure is returned immediately.
    pub async fn wait_for_event<F>(
        &self,
        timeout: Duration,
        predicate: F,
    ) -> Result<(), HarnessError>
    where
        F: Fn(&ConnectionEvent) -> bool,
    {
        self.state
            .wait_for("matching event", timeout, |events| {
                events.iter().any(&predicate)
            })
            .await
    }

    /// Requests shutdown, joins every endpoint and connection task, and reports failures.
    ///
    /// Failures recorded before or during shutdown are combined into the
    /// returned [`HarnessError`].
    ///
    /// Drop any client holding connections to this harness *before* calling this.
    /// Shutdown cancels connection tasks promptly, including one parked in
    /// [`SocketScript::await_client_close`]; a script waiting there can only
    /// observe bytes the client should not have sent while the connection is
    /// still live, so shutting down with the client alive can mask that failure.
    pub async fn shutdown(mut self) -> Result<(), HarnessError> {
        self.shutdown.send_replace(true);
        for task in self.endpoint_tasks.drain(..) {
            if let Err(err) = task.await {
                self.state.record_failure(HarnessError::new(format!(
                    "endpoint task failed while shutting down: {err}"
                )));
            }
        }
        match self.state.failure() {
            Some(failure) => Err(failure),
            None => Ok(()),
        }
    }
}

impl Drop for ConnectionTestHarness {
    fn drop(&mut self) {
        // A test that panics never reaches its `shutdown()` call, so a recorded
        // background failure would otherwise be lost -- and that failure is often
        // the actual explanation for the panic.
        if std::thread::panicking() {
            if let Some(failure) = self.state.failure() {
                eprintln!(
                    "\n[ConnectionTestHarness] background failure during panic:\n  {failure}\n"
                );
            }
        }
        self.shutdown.send_replace(true);
        for task in &self.endpoint_tasks {
            task.abort();
        }
    }
}

async fn run_endpoint(
    listener: TcpListener,
    endpoint_addr: SocketAddr,
    mut plan: EndpointPlan,
    state: Arc<SharedState>,
    next_connection_id: Arc<AtomicU64>,
    mut shutdown: watch::Receiver<bool>,
) {
    // The endpoint owns every connection task and drains the set before
    // returning, including during harness shutdown.
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = wait_for_shutdown(&mut shutdown) => break,
            completed = connections.join_next(), if !connections.is_empty() => {
                if let Some(Err(err)) = completed {
                    state.record_failure(HarnessError::new(format!(
                        "connection task at {endpoint_addr} failed: {err}"
                    )));
                }
            }
            accepted = listener.accept() => {
                let (stream, _) = match accepted {
                    Ok(accepted) => accepted,
                    Err(err) => {
                        state.record_failure(HarnessError::new(format!(
                            "failed to accept a connection at {endpoint_addr}: {err}"
                        )));
                        break;
                    }
                };
                let connection_id =
                    ConnectionId(next_connection_id.fetch_add(1, Ordering::Relaxed));
                state.record_event(ConnectionEvent::TcpAccepted {
                    connection_id,
                    endpoint_addr,
                });
                let Some(script) = plan.next_script() else {
                    state.record_failure(HarnessError::new(format!(
                        "endpoint {endpoint_addr} accepted connection {connection_id} after its plan was exhausted"
                    )));
                    drop(stream);
                    continue;
                };

                let state = state.clone();
                let connection_shutdown = shutdown.clone();
                connections.spawn(async move {
                    run_connection_task(
                        stream,
                        script,
                        connection_id,
                        endpoint_addr,
                        state,
                        connection_shutdown,
                    )
                    .await;
                });
            }
        }
    }

    while let Some(result) = connections.join_next().await {
        if let Err(err) = result {
            state.record_failure(HarnessError::new(format!(
                "connection task at {endpoint_addr} failed while shutting down: {err}"
            )));
        }
    }
}

async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow() {
            return;
        }
        if shutdown.changed().await.is_err() {
            return;
        }
    }
}

async fn run_connection_task(
    stream: TcpStream,
    script: ConnectionScript,
    connection_id: ConnectionId,
    endpoint_addr: SocketAddr,
    state: Arc<SharedState>,
    mut shutdown: watch::Receiver<bool>,
) {
    let result = tokio::select! {
        biased;
        _ = wait_for_shutdown(&mut shutdown) => Ok(ConnectionCloseReason::HarnessShutdown),
        result = run_connection(stream, script, connection_id, endpoint_addr, &state) => result,
    };
    let reason = match result {
        Ok(reason) => reason,
        Err(err) => {
            state.record_failure(HarnessError::new(format!(
                "connection {connection_id} at {endpoint_addr}: {err}"
            )));
            ConnectionCloseReason::ScriptFailed
        }
    };
    state.record_event(ConnectionEvent::ConnectionClosed {
        connection_id,
        reason,
    });
}

async fn run_connection(
    stream: TcpStream,
    script: ConnectionScript,
    connection_id: ConnectionId,
    endpoint_addr: SocketAddr,
    state: &SharedState,
) -> Result<ConnectionCloseReason, HarnessError> {
    let mut executor = ScriptExecutor {
        stream,
        pending: Vec::new(),
        connection_id,
        endpoint_addr,
        state,
    };
    match script.kind {
        ConnectionScriptKind::Socket(script) => Ok(executor
            .execute(&script.actions)
            .await?
            .unwrap_or(ConnectionCloseReason::ScriptCompleted)),
        ConnectionScriptKind::Http1(script) => match script.responses {
            Http1Responses::Finite(responses) => {
                let mut actions = Vec::new();
                for response in responses {
                    actions.push(Action::ReadHttp1Request);
                    actions.extend(response.actions());
                }
                if !actions
                    .last()
                    .is_some_and(|action| matches!(action, Action::Close | Action::Reset))
                {
                    actions.push(match script.finish {
                        Finish::AwaitClientClose => Action::AwaitClientClose,
                        Finish::Close => Action::Close,
                        Finish::Reset => Action::Reset,
                    });
                }
                Ok(executor
                    .execute(&actions)
                    .await?
                    .unwrap_or(ConnectionCloseReason::ScriptCompleted))
            }
            Http1Responses::Repeated(response) => loop {
                match executor.read_http1_request().await {
                    Ok(request) => executor.record_request(request),
                    Err(ReadRequestError::ClientClosed) => {
                        return Ok(ConnectionCloseReason::ClientClosed);
                    }
                    Err(ReadRequestError::Failed(err)) => return Err(err),
                }
                if let Some(reason) = executor.execute(&response.actions()).await? {
                    return Ok(reason);
                }
            },
        },
    }
}

struct ScriptExecutor<'a> {
    stream: TcpStream,
    pending: Vec<u8>,
    connection_id: ConnectionId,
    endpoint_addr: SocketAddr,
    state: &'a SharedState,
}

impl ScriptExecutor<'_> {
    async fn execute(
        &mut self,
        actions: &[Action],
    ) -> Result<Option<ConnectionCloseReason>, HarnessError> {
        for action in actions {
            match action {
                Action::ReadHttp1Request => {
                    let request = self.read_http1_request().await.map_err(|err| match err {
                        ReadRequestError::ClientClosed => {
                            HarnessError::new("client closed before the expected HTTP/1 request")
                        }
                        ReadRequestError::Failed(err) => err,
                    })?;
                    self.record_request(request);
                }
                Action::ReadUntil { delimiter, limit } => {
                    self.read_until(delimiter, *limit).await?;
                }
                Action::ReadExact(length) => {
                    self.fill_pending(*length).await?;
                    self.pending.drain(..*length);
                }
                Action::ExpectBytes(expected) => {
                    self.fill_pending(expected.len()).await?;
                    if self.pending[..expected.len()] != expected[..] {
                        return Err(HarnessError::new(format!(
                            "socket bytes differed: expected {expected:?}, got {:?}",
                            &self.pending[..expected.len()]
                        )));
                    }
                    self.pending.drain(..expected.len());
                }
                Action::WriteAll(bytes) => {
                    self.stream
                        .write_all(bytes)
                        .await
                        .map_err(|err| HarnessError::new(format!("failed to write: {err}")))?;
                }
                Action::Wait(gate) => gate.wait().await?,
                Action::Delay(duration) => tokio::time::sleep(*duration).await,
                Action::ShutdownWrite => {
                    self.stream
                        .shutdown()
                        .await
                        .map_err(|err| HarnessError::new(format!("failed to shut down: {err}")))?;
                }
                Action::AwaitClientClose => {
                    if !self.pending.is_empty() {
                        return Err(HarnessError::new(
                            "client sent bytes after the scripted HTTP/1 responses were exhausted",
                        ));
                    }
                    let mut byte = [0u8; 1];
                    return match self.stream.read(&mut byte).await {
                        Ok(0) => Ok(Some(ConnectionCloseReason::ClientClosed)),
                        Ok(_) => Err(HarnessError::new(
                            "client sent another request after the HTTP/1 script was exhausted",
                        )),
                        Err(err) if peer_close_error(&err) => {
                            Ok(Some(ConnectionCloseReason::ClientClosed))
                        }
                        Err(err) => Err(HarnessError::new(format!(
                            "failed while waiting for the client to close: {err}"
                        ))),
                    };
                }
                Action::Close => {
                    return Ok(Some(ConnectionCloseReason::ScriptCompleted));
                }
                Action::Reset => {
                    socket2::SockRef::from(&self.stream)
                        .set_linger(Some(Duration::ZERO))
                        .map_err(|err| {
                            HarnessError::new(format!("failed to configure TCP reset: {err}"))
                        })?;
                    return Ok(Some(ConnectionCloseReason::Reset));
                }
            }
        }
        Ok(None)
    }

    async fn read_until(&mut self, delimiter: &[u8], limit: usize) -> Result<(), HarnessError> {
        loop {
            if let Some(index) = find_bytes(&self.pending, delimiter) {
                let consumed = index + delimiter.len();
                if consumed > limit {
                    return Err(HarnessError::new(format!(
                        "read_until exceeded its {limit}-byte limit"
                    )));
                }
                self.pending.drain(..consumed);
                return Ok(());
            }
            if self.pending.len() >= limit {
                return Err(HarnessError::new(format!(
                    "read_until did not find its delimiter within {limit} bytes"
                )));
            }
            self.read_more().await?;
        }
    }

    async fn fill_pending(&mut self, length: usize) -> Result<(), HarnessError> {
        while self.pending.len() < length {
            self.read_more().await?;
        }
        Ok(())
    }

    async fn read_more(&mut self) -> Result<(), HarnessError> {
        let mut chunk = [0u8; READ_CHUNK_SIZE];
        match self.stream.read(&mut chunk).await {
            Ok(0) => Err(HarnessError::new(
                "client closed while the script was reading",
            )),
            Ok(read) => {
                self.pending.extend_from_slice(&chunk[..read]);
                Ok(())
            }
            Err(err) => Err(HarnessError::new(format!(
                "failed to read from client: {err}"
            ))),
        }
    }

    async fn read_http1_request(&mut self) -> Result<ParsedRequest, ReadRequestError> {
        loop {
            let parsed = parse_request_head(&self.pending).map_err(ReadRequestError::Failed)?;
            if let Some(mut request) = parsed {
                let total_length = request
                    .header_length
                    .checked_add(request.body_length)
                    .ok_or_else(|| {
                        ReadRequestError::Failed(HarnessError::new(
                            "HTTP/1 request length overflow",
                        ))
                    })?;
                if request.body_length > MAX_HTTP1_BODY_BYTES {
                    return Err(ReadRequestError::Failed(HarnessError::new(format!(
                        "HTTP/1 request body exceeds {MAX_HTTP1_BODY_BYTES} bytes"
                    ))));
                }
                while self.pending.len() < total_length {
                    self.read_more().await.map_err(ReadRequestError::Failed)?;
                }
                self.pending.drain(..total_length);
                request.header_length = 0;
                request.body_length = 0;
                return Ok(request);
            }
            if self.pending.len() >= MAX_HTTP1_HEADER_BYTES {
                return Err(ReadRequestError::Failed(HarnessError::new(format!(
                    "HTTP/1 request headers exceed {MAX_HTTP1_HEADER_BYTES} bytes"
                ))));
            }

            let mut chunk = [0u8; READ_CHUNK_SIZE];
            match self.stream.read(&mut chunk).await {
                Ok(0) if self.pending.is_empty() => return Err(ReadRequestError::ClientClosed),
                Ok(0) => {
                    return Err(ReadRequestError::Failed(HarnessError::new(
                        "client closed during HTTP/1 request headers",
                    )))
                }
                Ok(read) => self.pending.extend_from_slice(&chunk[..read]),
                Err(err) if self.pending.is_empty() && peer_close_error(&err) => {
                    return Err(ReadRequestError::ClientClosed)
                }
                Err(err) => {
                    return Err(ReadRequestError::Failed(HarnessError::new(format!(
                        "failed to read HTTP/1 request: {err}"
                    ))))
                }
            }
        }
    }

    fn record_request(&self, request: ParsedRequest) {
        self.state.record_event(ConnectionEvent::Http1Request {
            connection_id: self.connection_id,
            endpoint_addr: self.endpoint_addr,
            method: request.method,
            target: request.target,
            host: request.host,
        });
    }
}

enum ReadRequestError {
    ClientClosed,
    Failed(HarnessError),
}

struct ParsedRequest {
    method: String,
    target: String,
    host: Option<String>,
    header_length: usize,
    body_length: usize,
}

fn parse_request_head(bytes: &[u8]) -> Result<Option<ParsedRequest>, HarnessError> {
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut request = httparse::Request::new(&mut headers);
    let header_length = match request
        .parse(bytes)
        .map_err(|err| HarnessError::new(format!("invalid HTTP/1 request: {err}")))?
    {
        httparse::Status::Partial => return Ok(None),
        httparse::Status::Complete(length) => length,
    };
    if header_length > MAX_HTTP1_HEADER_BYTES {
        return Err(HarnessError::new(format!(
            "HTTP/1 request headers exceed {MAX_HTTP1_HEADER_BYTES} bytes"
        )));
    }
    let method = request
        .method
        .ok_or_else(|| HarnessError::new("HTTP/1 request has no method"))?
        .to_owned();
    let target = request
        .path
        .ok_or_else(|| HarnessError::new("HTTP/1 request has no target"))?
        .to_owned();
    let mut host = None;
    let mut content_length = None;
    for header in request.headers.iter() {
        if header.name.eq_ignore_ascii_case("host") {
            host = Some(
                std::str::from_utf8(header.value)
                    .map_err(|_| HarnessError::new("Host header is not valid UTF-8"))?
                    .trim()
                    .to_owned(),
            );
        } else if header.name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(HarnessError::new(
                    "multiple Content-Length headers are not supported",
                ));
            }
            let value = std::str::from_utf8(header.value)
                .map_err(|_| HarnessError::new("Content-Length is not valid ASCII"))?
                .trim();
            content_length = Some(
                value
                    .parse::<usize>()
                    .map_err(|_| HarnessError::new(format!("invalid Content-Length {value:?}")))?,
            );
        } else if header.name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(HarnessError::new(
                "Transfer-Encoding is not supported by read_http1_request; use raw socket actions",
            ));
        }
    }

    Ok(Some(ParsedRequest {
        method,
        target,
        host,
        header_length,
        body_length: content_length.unwrap_or(0),
    }))
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn peer_close_error(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::ConnectionAborted
            | std::io::ErrorKind::ConnectionReset
            | std::io::ErrorKind::BrokenPipe
            | std::io::ErrorKind::NotConnected
    )
}
