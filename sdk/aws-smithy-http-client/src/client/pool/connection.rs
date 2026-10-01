/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Protocol-independent connection lifetime.
//!
//! [`ConnectionState`] serializes dispatch commitment with logical close.
//! [`DispatchGuard`] accounts for accepted requests.
//! [`PhysicalConnectionGuard`] follows root I/O until the client releases its
//! physical connection handle, including transfer through a protocol upgrade.
//! The operating system may continue TCP teardown afterward. Logical close
//! returns bounded capacity before physical completion except while an HTTP/1
//! exchange may still transfer upgraded I/O.

use super::admission::CapacityLease;
use super::events::{LogicalCloseCause, SharedConnectionEventListener};
use super::origin::OriginKey;
#[cfg(test)]
use super::partition::Spawn;
use super::partition::{DriverSpawner, PartitionId};
use super::stats::CellConnectionStats;
use crate::client::connect::{ConnectPath, ConnectPathInner};
use crate::sync::{Arc, Mutex};
pub use aws_smithy_runtime_api::client::connection::ConnectionId;
use aws_smithy_runtime_api::client::connection::{
    ConnectionEstablishmentMetadata, ConnectionMetadata,
};
use http_1x::Extensions;
use hyper::rt::{Read, ReadBufCursor, Write};
use hyper_util::client::legacy::connect::{Connected, Connection, HttpInfo};
use pin_project_lite::pin_project;
use std::fmt;
use std::io::{self, IoSlice};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};

/// Final protocol and ownership classification for a closed connection.
///
/// [`LogicalCloseCause`] reports the first stable reason that pool dispatch
/// ended. This more specific classification is reported at physical close
/// because HTTP/1 upgrade ownership may be resolved after logical close.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CloseReason {
    /// The connection exceeded its configured idle timeout.
    IdleTimeout,
    /// The connection was explicitly marked unsafe for reuse.
    Poisoned,
    /// The protocol driver or peer closed the connection.
    ProtocolClosed,
    /// HTTP/1 did not prove a complete reusable message boundary.
    IncompleteH1Exchange,
    /// The transport left HTTP/1 pool ownership through an upgrade.
    Upgraded,
    /// The connection closed to move bounded capacity to another cell.
    Reclaimed,
    /// The connection pool was dropped.
    PoolDropped,
    /// The runtime driving the connection shut down.
    OwnerRuntimeShutdown,
}

/// Protocol selected for one installed physical connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionProtocol {
    /// HTTP/1.1 with one exclusive request sender.
    Http1,
    /// HTTP/2 with a multiplexed request sender.
    Http2,
}

/// Immutable identity and transport facts for one installed connection.
///
/// Protocol records, request metadata, tracing, and lifecycle events retain the
/// same value. It remains valid after logical close so the final physical-close
/// observation names the same connection.
#[derive(Debug)]
pub struct ConnectionInfo {
    /// Stable identity assigned by the owning pool.
    id: ConnectionId,
    /// Canonical origin this connection may serve.
    origin: OriginKey,
    /// Partition that retains the transport and protocol driver.
    owner_partition: PartitionId,
    /// Protocol selected by configuration or ALPN.
    protocol: ConnectionProtocol,
    /// Local socket address reported by the connector, when available.
    local_addr: Option<SocketAddr>,
    /// Remote socket address reported by the connector, when available.
    remote_addr: Option<SocketAddr>,
    /// Connector-owned path state, including request-time proxy authorization.
    connect_path: ConnectPathInner,
    /// Connector metadata copied into every response on this connection.
    connected: Connected,
    /// Measurements frozen before this connection becomes visible to dispatch.
    establishment: OnceLock<ConnectionEstablishmentMetadata>,
}

impl ConnectionInfo {
    /// Captures immutable facts after transport negotiation selects a protocol.
    pub(super) fn new(
        id: ConnectionId,
        origin: OriginKey,
        owner_partition: PartitionId,
        protocol: ConnectionProtocol,
        connected: Connected,
    ) -> Arc<Self> {
        let mut extras = Extensions::new();
        connected.get_extras(&mut extras);
        let http_info = extras.get::<HttpInfo>();
        let connect_path = ConnectPathInner::from_connected(&connected, &extras);
        Arc::new(Self {
            id,
            origin,
            owner_partition,
            protocol,
            local_addr: http_info.map(HttpInfo::local_addr),
            remote_addr: http_info.map(HttpInfo::remote_addr),
            connect_path,
            connected,
            establishment: OnceLock::new(),
        })
    }

    /// Returns the pool-assigned physical connection identity.
    pub fn id(&self) -> ConnectionId {
        self.id
    }

    /// Returns the canonical origin this connection may serve.
    pub fn origin(&self) -> &OriginKey {
        &self.origin
    }

    /// Returns the partition that owns this connection's I/O and driver.
    pub fn owner_partition(&self) -> PartitionId {
        self.owner_partition
    }

    /// Returns the established HTTP protocol.
    pub fn protocol(&self) -> ConnectionProtocol {
        self.protocol
    }

    /// Returns the connector-reported local socket address, when available.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Returns the connector-reported remote socket address, when available.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns how this connection reaches its origin.
    pub fn connect_path(&self) -> ConnectPath {
        self.connect_path.public()
    }

    /// Returns whether an HTTP proxy participates in this connection path.
    pub fn is_proxied(&self) -> bool {
        self.connect_path().is_proxied()
    }

    /// Returns successful establishment measurements, when installation completed.
    pub fn establishment(&self) -> Option<&ConnectionEstablishmentMetadata> {
        self.establishment.get()
    }

    /// Freezes successful establishment measurements before pool publication.
    pub(super) fn set_establishment(&self, metadata: ConnectionEstablishmentMetadata) {
        assert!(
            self.establishment.set(metadata).is_ok(),
            "connection establishment metadata was set more than once"
        );
    }

    /// Returns connector-owned request-path state.
    pub(super) fn connect_path_inner(&self) -> &ConnectPathInner {
        &self.connect_path
    }

    /// Copies connector-provided values into a response extension map.
    pub(super) fn apply_connector_extras(&self, extensions: &mut Extensions) {
        self.connected.get_extras(extensions);
    }

    /// Builds Smithy metadata with close authority for this H1 record.
    pub(super) fn metadata(&self, close: super::cell::h1::H1CloseHandle) -> ConnectionMetadata {
        let mut builder = ConnectionMetadata::builder()
            .proxied(self.is_proxied())
            .connection_id(self.id)
            .poison_fn(move || {
                close.close(CloseReason::Poisoned);
            });
        builder
            .set_local_addr(self.local_addr)
            .set_remote_addr(self.remote_addr)
            .set_establishment(self.establishment().cloned());
        builder.build()
    }

    /// Builds Smithy metadata with close authority for this H2 generation.
    pub(super) fn h2_metadata(&self, close: super::cell::h2::H2CloseHandle) -> ConnectionMetadata {
        let mut builder = ConnectionMetadata::builder()
            .proxied(self.is_proxied())
            .connection_id(self.id)
            .poison_fn(move || {
                close.close(CloseReason::Poisoned);
            });
        builder
            .set_local_addr(self.local_addr)
            .set_remote_addr(self.remote_addr)
            .set_establishment(self.establishment().cloned());
        builder.build()
    }

    /// Creates synthetic HTTP/1 information for state-machine tests.
    #[cfg(test)]
    pub(super) fn for_test(id: ConnectionId, owner_partition: PartitionId) -> Arc<Self> {
        Self::new(
            id,
            OriginKey::from_parts(http_1x::uri::Scheme::HTTPS, "example.com", None)
                .expect("synthetic test origin is valid"),
            owner_partition,
            ConnectionProtocol::Http1,
            Connected::new(),
        )
    }
}

/// Shared connection ownership from negotiated transport through close.
pub(super) struct ConnectionState {
    /// Identity and transport facts shared with metadata and lifecycle events.
    info: Arc<ConnectionInfo>,
    /// Runtime that owns protocol and follow-up work for this connection.
    owner_spawner: DriverSpawner,
    /// Cell-owned counts for connection lifetimes that outlive protocol records.
    stats: Arc<CellConnectionStats>,
    /// Dispatch, logical-close, and physical-connection completion state.
    lifecycle: Mutex<ConnectionLifecycle>,
}

#[cfg(test)]
#[derive(Debug)]
struct TestDriverSpawner;

#[cfg(test)]
impl Spawn for TestDriverSpawner {
    fn spawn(&self, driver: Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>) {
        drop(driver);
    }
}

/// Connection lifetime state serialized with dispatch commitment and close.
#[derive(Debug)]
struct ConnectionLifecycle {
    /// Dispatch eligibility and ownership of bounded capacity.
    logical: LogicalState,
    /// Requests that committed before logical close.
    in_flight: usize,
    /// Whether the client has released its physical connection handle.
    physical_connection_complete: bool,
    /// Protocol drain or upgrade count held until physical completion.
    counted_close: Option<CountedClose>,
    /// Progress through installed-connection event delivery.
    events: LifecycleEventProgress,
}

/// Whether a connection may accept dispatch and still owns bounded capacity.
#[derive(Debug)]
enum LogicalState {
    /// The connector returned connected I/O and selected the HTTP protocol,
    /// but Hyper has not produced the request handle required for dispatch.
    PendingOpen,
    /// Dispatch may commit.
    Open {
        /// Bounded-origin slot released by logical close, when configured.
        capacity: Option<CapacityLease>,
    },
    /// New dispatch is rejected while accepted work may still drain.
    Closed {
        /// Stable cause recorded by the first logical-close transition.
        cause: LogicalCloseCause,
        /// Protocol-specific final close classification.
        disposition: CloseDisposition,
        /// Capacity retained only while HTTP/1 may transfer upgraded I/O.
        retained_capacity: Option<CapacityLease>,
    },
}

/// Final close classification, including HTTP/1's upgrade handoff window.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CloseDisposition {
    /// HTTP/1 may require exchange-side classification after its driver ends.
    Http1(H1CloseDisposition),
    /// HTTP/2 has no post-driver ownership transfer.
    Http2(CloseReason),
}

impl CloseDisposition {
    /// Returns the final physical close reason once classification completes.
    fn final_reason(self) -> Option<CloseReason> {
        match self {
            Self::Http1(H1CloseDisposition::AwaitingExchange { .. }) => None,
            Self::Http1(H1CloseDisposition::Final(reason)) | Self::Http2(reason) => Some(reason),
        }
    }
}

/// HTTP/1 close classification around a possible upgrade handoff.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum H1CloseDisposition {
    /// The driver ended while one accepted exchange may still prove an upgrade.
    AwaitingExchange {
        /// Final reason when physical ownership ends before an upgrade appears.
        fallback: CloseReason,
    },
    /// Exchange classification is complete.
    Final(CloseReason),
}

/// Relaxed protocol-lifetime count held by one closed physical connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CountedClose {
    H1Draining,
    H1Upgraded,
    H2Draining,
}

impl CountedClose {
    fn start(self, stats: &CellConnectionStats) {
        match self {
            Self::H1Draining => stats.h1_drain_started(),
            Self::H1Upgraded => stats.h1_upgrade_started(),
            Self::H2Draining => stats.h2_drain_started(),
        }
    }

    fn finish(self, stats: &CellConnectionStats) {
        match self {
            Self::H1Draining => stats.h1_drain_finished(),
            Self::H1Upgraded => stats.h1_upgrade_finished(),
            Self::H2Draining => stats.h2_drain_finished(),
        }
    }

    fn h1_upgrade(self, stats: &CellConnectionStats) -> Self {
        assert_eq!(Self::H1Draining, self, "non-HTTP/1 drain became an upgrade");
        stats.h1_drain_upgraded();
        Self::H1Upgraded
    }
}

/// Progress through connection lifecycle event emission.
///
/// `OpenedPending` keeps close events behind the `Opened` callback. The
/// waiting states name the next event and retain the listener after `Opened`
/// returns. `emitting` grants one thread responsibility for emitting every
/// event that becomes ready before it releases that responsibility.
#[derive(Debug)]
enum LifecycleEventProgress {
    /// Pool installation completed, but the `Opened` callback has not returned.
    OpenedPending,
    /// Logical close is the next event.
    WaitingForLogicalClose {
        listener: SharedConnectionEventListener,
        emitting: bool,
    },
    /// Physical close is the next event.
    WaitingForPhysicalClose {
        listener: SharedConnectionEventListener,
        emitting: bool,
    },
    /// No further connection lifecycle event remains.
    Complete,
}

/// Event data copied out of the lifecycle lock for listener invocation.
enum ReadyLifecycleEvent {
    LogicalClose(LogicalCloseCause),
    PhysicalClose(CloseReason),
}

impl LifecycleEventProgress {
    /// Transfers callback ownership from `Opened` to close-event emission.
    fn opened_completed(&mut self, listener: Option<&SharedConnectionEventListener>) -> bool {
        assert!(
            matches!(self, Self::OpenedPending),
            "connection opened event completed more than once"
        );
        let Some(listener) = listener else {
            *self = Self::Complete;
            return false;
        };
        *self = Self::WaitingForLogicalClose {
            listener: listener.clone(),
            emitting: true,
        };
        true
    }

    /// Claims responsibility for emitting newly ready lifecycle events.
    fn claim_emission(&mut self) -> bool {
        let emitting = match self {
            Self::WaitingForLogicalClose { emitting, .. }
            | Self::WaitingForPhysicalClose { emitting, .. } => emitting,
            Self::OpenedPending | Self::Complete => return false,
        };
        if *emitting {
            return false;
        }
        *emitting = true;
        true
    }
}

impl ConnectionLifecycle {
    /// Takes the next ready event or releases emission responsibility.
    fn take_ready_event(&mut self) -> Option<(SharedConnectionEventListener, ReadyLifecycleEvent)> {
        match std::mem::replace(&mut self.events, LifecycleEventProgress::Complete) {
            LifecycleEventProgress::OpenedPending => {
                unreachable!("connection lifecycle event emission began before Opened completed")
            }
            LifecycleEventProgress::WaitingForLogicalClose { listener, emitting } => {
                assert!(
                    emitting,
                    "connection lifecycle event emission was not claimed"
                );
                let cause = match &self.logical {
                    LogicalState::Closed { cause, .. } => *cause,
                    LogicalState::PendingOpen | LogicalState::Open { .. } => {
                        self.events = LifecycleEventProgress::WaitingForLogicalClose {
                            listener,
                            emitting: false,
                        };
                        return None;
                    }
                };
                self.events = LifecycleEventProgress::WaitingForPhysicalClose {
                    listener: listener.clone(),
                    emitting: true,
                };
                Some((listener, ReadyLifecycleEvent::LogicalClose(cause)))
            }
            LifecycleEventProgress::WaitingForPhysicalClose { listener, emitting } => {
                assert!(
                    emitting,
                    "connection lifecycle event emission was not claimed"
                );
                if !self.physical_connection_complete {
                    self.events = LifecycleEventProgress::WaitingForPhysicalClose {
                        listener,
                        emitting: false,
                    };
                    return None;
                }
                let reason = match &self.logical {
                    LogicalState::Closed { disposition, .. } => disposition
                        .final_reason()
                        .expect("physical close completed before H1 classification"),
                    LogicalState::PendingOpen | LogicalState::Open { .. } => {
                        panic!("physical close followed an open connection")
                    }
                };
                Some((listener, ReadyLifecycleEvent::PhysicalClose(reason)))
            }
            LifecycleEventProgress::Complete => None,
        }
    }
}
impl ConnectionState {
    /// Creates state after transport establishment and protocol selection.
    ///
    /// For TLS transports, the connector has completed TLS and ALPN before this
    /// call. Hyper protocol setup and cell installation have not occurred.
    ///
    /// The returned guard uniquely tracks the client's physical connection
    /// handle.
    /// After Hyper returns a request handle, [`Self::open`] attaches optional
    /// bounded capacity before cell installation makes the connection
    /// discoverable.
    pub(super) fn pending_open(
        info: Arc<ConnectionInfo>,
        owner_spawner: DriverSpawner,
        stats: Arc<CellConnectionStats>,
    ) -> (Arc<Self>, PhysicalConnectionGuard) {
        stats.physical_connection_started();
        let connection = Arc::new(Self {
            info,
            owner_spawner,
            stats,
            lifecycle: Mutex::new(ConnectionLifecycle {
                logical: LogicalState::PendingOpen,
                in_flight: 0,
                physical_connection_complete: false,
                counted_close: None,
                events: LifecycleEventProgress::OpenedPending,
            }),
        });
        let physical = PhysicalConnectionGuard {
            connection: connection.clone(),
            active: true,
        };
        (connection, physical)
    }

    #[cfg(test)]
    pub(super) fn pending_open_for_test(
        info: Arc<ConnectionInfo>,
    ) -> (Arc<Self>, PhysicalConnectionGuard) {
        Self::pending_open(
            info,
            DriverSpawner::new(TestDriverSpawner),
            Arc::new(CellConnectionStats::default()),
        )
    }

    /// Returns the runtime that owns this installed connection.
    pub(super) fn owner_spawner(&self) -> DriverSpawner {
        self.owner_spawner.clone()
    }

    /// Opens dispatch commitment and transfers optional bounded capacity.
    ///
    /// The caller performs this transition after Hyper protocol setup and
    /// before publishing the connection through its cell. Opening first
    /// ensures that a newly visible request handle can commit dispatch.
    ///
    /// Returns `lease` when logical close won before opening.
    pub(super) fn open(&self, lease: Option<CapacityLease>) -> Result<(), Option<CapacityLease>> {
        let mut lifecycle = self.lifecycle.lock();
        if !matches!(lifecycle.logical, LogicalState::PendingOpen) {
            return Err(lease);
        }
        lifecycle.logical = LogicalState::Open { capacity: lease };
        Ok(())
    }

    /// Creates a connection whose origin has no admission bound.
    ///
    /// The returned unique guard must move with the root I/O task.
    #[cfg(test)]
    pub(super) fn unbounded(info: Arc<ConnectionInfo>) -> (Arc<Self>, PhysicalConnectionGuard) {
        let (connection, physical) = Self::pending_open_for_test(info);
        connection
            .open(None)
            .expect("new unbounded connection could not open");
        (connection, physical)
    }

    /// Creates a connection that takes ownership of one bounded-origin slot.
    ///
    /// The returned unique guard must move with the root I/O task. Logical
    /// close returns `lease` independently of that guard.
    #[cfg(test)]
    pub(super) fn bounded(
        info: Arc<ConnectionInfo>,
        lease: CapacityLease,
    ) -> (Arc<Self>, PhysicalConnectionGuard) {
        let (connection, physical) = Self::pending_open_for_test(info);
        connection
            .open(Some(lease))
            .expect("new bounded connection could not open");
        (connection, physical)
    }

    /// Returns this connection's stable identity.
    pub(super) fn id(&self) -> ConnectionId {
        self.info.id()
    }

    /// Returns the partition that owns this connection's I/O and driver.
    pub(super) fn owner_partition(&self) -> PartitionId {
        self.info.owner_partition()
    }

    /// Returns immutable identity and transport facts for this connection.
    pub(super) fn info(&self) -> &Arc<ConnectionInfo> {
        &self.info
    }

    /// Attempts to commit one request against logical close.
    ///
    /// Returns a guard and increments the in-flight count while the connection
    /// is open. Returns `None` without changing state after logical close.
    pub(super) fn try_commit_dispatch(connection: &Arc<Self>) -> Option<DispatchGuard> {
        let mut lifecycle = connection.lifecycle.lock();
        if !matches!(lifecycle.logical, LogicalState::Open { .. }) {
            return None;
        }
        lifecycle.in_flight = lifecycle
            .in_flight
            .checked_add(1)
            .expect("in-flight dispatch count exhausted");
        drop(lifecycle);

        Some(DispatchGuard {
            connection: connection.clone(),
            active: true,
        })
    }

    /// Completes the `Opened` callback and reports any close that occurred
    /// while it was running.
    ///
    /// A listener may reenter the pool and close this connection from its
    /// `Opened` callback. The lifecycle transition completes immediately, but
    /// its callbacks are delayed until this method can preserve
    /// `Opened`, logical close, and physical close in that order.
    pub(super) fn complete_opened_event(&self, listener: Option<&SharedConnectionEventListener>) {
        let emit = {
            let mut lifecycle = self.lifecycle.lock();
            assert!(
                !matches!(lifecycle.logical, LogicalState::PendingOpen),
                "connection opened event completed before the connection opened"
            );
            lifecycle.events.opened_completed(listener)
        };

        if emit {
            self.emit_ready_events();
        }
    }

    /// Emits every ready lifecycle event claimed by the current thread.
    ///
    /// Each event is taken in its own statement so the lifecycle lock is
    /// released before the listener runs.
    fn emit_ready_events(&self) {
        loop {
            let ready = self.lifecycle.lock().take_ready_event();
            let Some((listener, event)) = ready else {
                return;
            };
            match event {
                ReadyLifecycleEvent::LogicalClose(cause) => {
                    listener.logical_close(&self.info, cause);
                }
                ReadyLifecycleEvent::PhysicalClose(reason) => {
                    listener.physical_close(&self.info, reason);
                }
            }
        }
    }

    /// Performs the first logical-close transition.
    ///
    /// Returns `true` when this call closes the connection and records
    /// `reason`. Returns `false` when another close already won; the original
    /// logical cause remains unchanged.
    ///
    /// Capacity normally returns after the connection lock is released.
    /// HTTP/1 retains it while an accepted exchange may still transfer
    /// upgraded I/O.
    pub(super) fn logical_close(&self, reason: CloseReason) -> bool {
        let cause = LogicalCloseCause::from_reason(reason);
        let (released_capacity, emit) = {
            let mut lifecycle = self.lifecycle.lock();
            let previous = std::mem::replace(&mut lifecycle.logical, LogicalState::PendingOpen);
            let (disposition, retained_capacity, released_capacity, was_open) = match previous {
                LogicalState::PendingOpen => {
                    let disposition = match self.info.protocol() {
                        ConnectionProtocol::Http1 => {
                            CloseDisposition::Http1(H1CloseDisposition::Final(reason))
                        }
                        ConnectionProtocol::Http2 => CloseDisposition::Http2(reason),
                    };
                    (disposition, None, None, false)
                }
                LogicalState::Open { mut capacity } => match self.info.protocol() {
                    ConnectionProtocol::Http1
                        if lifecycle.in_flight != 0
                            && !lifecycle.physical_connection_complete
                            && matches!(
                                reason,
                                CloseReason::ProtocolClosed | CloseReason::OwnerRuntimeShutdown
                            ) =>
                    {
                        (
                            CloseDisposition::Http1(H1CloseDisposition::AwaitingExchange {
                                fallback: reason,
                            }),
                            capacity.take(),
                            None,
                            true,
                        )
                    }
                    ConnectionProtocol::Http1 => {
                        let retain_for_upgrade = reason == CloseReason::Upgraded
                            && !lifecycle.physical_connection_complete;
                        let retained_capacity =
                            retain_for_upgrade.then(|| capacity.take()).flatten();
                        (
                            CloseDisposition::Http1(H1CloseDisposition::Final(reason)),
                            retained_capacity,
                            capacity,
                            true,
                        )
                    }
                    ConnectionProtocol::Http2 => {
                        (CloseDisposition::Http2(reason), None, capacity, true)
                    }
                },
                closed @ LogicalState::Closed { .. } => {
                    lifecycle.logical = closed;
                    return false;
                }
            };
            if was_open && !lifecycle.physical_connection_complete {
                let counted_close = match self.info.protocol() {
                    ConnectionProtocol::Http1 if reason == CloseReason::Upgraded => {
                        CountedClose::H1Upgraded
                    }
                    ConnectionProtocol::Http1 => CountedClose::H1Draining,
                    ConnectionProtocol::Http2 => CountedClose::H2Draining,
                };
                assert!(
                    lifecycle.counted_close.is_none(),
                    "connection close was counted more than once"
                );
                counted_close.start(&self.stats);
                lifecycle.counted_close = Some(counted_close);
            }
            lifecycle.logical = LogicalState::Closed {
                cause,
                disposition,
                retained_capacity,
            };
            let emit = lifecycle.events.claim_emission();
            (released_capacity, emit)
        };
        drop(released_capacity);
        if emit {
            self.emit_ready_events();
        }
        tracing::debug!(
            connection_id = %self.id(),
            connection_partition = ?self.owner_partition(),
            protocol = ?self.info.protocol(),
            origin_scheme = %self.info.origin().scheme(),
            origin_host = self.info.origin().host(),
            origin_port = ?self.info.origin().port(),
            close_reason = ?reason,
            "connection logically closed"
        );
        true
    }

    /// Finalizes a driver-observed HTTP/1 close from exchange-side evidence.
    ///
    /// A confirmed upgrade retains bounded capacity until root I/O leaves the
    /// client. Every other classification preserves the driver reason and
    /// returns capacity immediately.
    pub(super) fn complete_h1_exchange(&self, exchange_reason: CloseReason) -> bool {
        let (released_capacity, final_reason) = {
            let mut lifecycle = self.lifecycle.lock();
            let physical_connection_complete = lifecycle.physical_connection_complete;
            let (released_capacity, final_reason) = match &mut lifecycle.logical {
                LogicalState::Closed {
                    disposition: CloseDisposition::Http1(state),
                    retained_capacity,
                    ..
                } => {
                    let H1CloseDisposition::AwaitingExchange { fallback } = *state else {
                        return false;
                    };
                    debug_assert!(
                        !physical_connection_complete,
                        "physical close left HTTP/1 exchange classification pending"
                    );
                    let final_reason = if exchange_reason == CloseReason::Upgraded {
                        CloseReason::Upgraded
                    } else {
                        fallback
                    };
                    *state = H1CloseDisposition::Final(final_reason);
                    let release_capacity = final_reason != CloseReason::Upgraded;
                    (
                        release_capacity.then(|| retained_capacity.take()).flatten(),
                        final_reason,
                    )
                }
                _ => return false,
            };
            if final_reason == CloseReason::Upgraded {
                lifecycle.counted_close = Some(
                    lifecycle
                        .counted_close
                        .expect("upgraded HTTP/1 connection had no draining count")
                        .h1_upgrade(&self.stats),
                );
            }
            (released_capacity, final_reason)
        };
        drop(released_capacity);
        tracing::debug!(
            connection_id = %self.id(),
            connection_partition = ?self.owner_partition(),
            origin_scheme = %self.info.origin().scheme(),
            origin_host = self.info.origin().host(),
            origin_port = ?self.info.origin().port(),
            close_reason = ?final_reason,
            exchange_reason = ?exchange_reason,
            "HTTP/1 connection close classification completed"
        );
        true
    }

    /// Removes one dispatch previously committed by [`Self::try_commit_dispatch`].
    fn release_dispatch(&self) {
        let complete_reason = {
            let mut lifecycle = self.lifecycle.lock();
            lifecycle.in_flight = lifecycle
                .in_flight
                .checked_sub(1)
                .expect("completed a dispatch that was not in flight");
            if lifecycle.in_flight == 0 {
                match lifecycle.logical {
                    LogicalState::Closed {
                        disposition:
                            CloseDisposition::Http1(H1CloseDisposition::AwaitingExchange { fallback }),
                        ..
                    } => Some(fallback),
                    _ => None,
                }
            } else {
                None
            }
        };
        if let Some(reason) = complete_reason {
            self.complete_h1_exchange(reason);
        }
    }

    /// Records that the client released its physical connection handle.
    ///
    /// # Panics
    ///
    /// Panics if physical connection ownership completes more than once.
    fn complete_physical_connection(&self) {
        let (released_capacity, emit) = {
            let mut lifecycle = self.lifecycle.lock();
            assert!(
                !lifecycle.physical_connection_complete,
                "physical connection ownership completed more than once"
            );
            lifecycle.physical_connection_complete = true;

            let released_capacity = match &mut lifecycle.logical {
                LogicalState::Closed {
                    disposition,
                    retained_capacity,
                    ..
                } => {
                    if let CloseDisposition::Http1(H1CloseDisposition::AwaitingExchange {
                        fallback,
                    }) = *disposition
                    {
                        *disposition = CloseDisposition::Http1(H1CloseDisposition::Final(fallback));
                    }
                    retained_capacity.take()
                }
                LogicalState::PendingOpen | LogicalState::Open { .. } => None,
            };
            if let Some(counted_close) = lifecycle.counted_close.take() {
                counted_close.finish(&self.stats);
            }
            self.stats.physical_connection_finished();
            let emit = lifecycle.events.claim_emission();
            (released_capacity, emit)
        };
        drop(released_capacity);
        if emit {
            self.emit_ready_events();
        }
        tracing::debug!(
            connection_id = %self.id(),
            connection_partition = ?self.owner_partition(),
            origin_scheme = %self.info.origin().scheme(),
            origin_host = self.info.origin().host(),
            origin_port = ?self.info.origin().port(),
            "physical connection ownership ended"
        );
    }

    /// Returns a consistent lifecycle snapshot.
    #[cfg(test)]
    pub(super) fn probe(&self) -> ConnectionProbe {
        let lifecycle = self.lifecycle.lock();
        let (close_reason, awaiting_h1_exchange) = match &lifecycle.logical {
            LogicalState::PendingOpen | LogicalState::Open { .. } => (None, false),
            LogicalState::Closed { disposition, .. } => (
                disposition.final_reason(),
                matches!(
                    disposition,
                    CloseDisposition::Http1(H1CloseDisposition::AwaitingExchange { .. })
                ),
            ),
        };
        ConnectionProbe {
            close_reason,
            awaiting_h1_exchange,
            in_flight: lifecycle.in_flight,
            physical_connection_complete: lifecycle.physical_connection_complete,
        }
    }
}

impl fmt::Debug for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionState")
            .field("info", &self.info)
            .field("lifecycle", &self.lifecycle)
            .finish()
    }
}

#[cfg(test)]
/// Observable lifecycle state used by protocol coordination and focused tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ConnectionProbe {
    /// Reason logical close won, or `None` while dispatch is accepted.
    pub(super) close_reason: Option<CloseReason>,
    /// Whether an accepted H1 exchange still owes final close classification.
    pub(super) awaiting_h1_exchange: bool,
    /// Number of dispatches that have not completed.
    pub(super) in_flight: usize,
    /// Whether the client released its physical connection handle.
    pub(super) physical_connection_complete: bool,
}

/// One dispatch that committed before logical close.
///
/// Dropping the guard records request completion exactly once.
#[derive(Debug)]
pub(super) struct DispatchGuard {
    /// Shared state whose in-flight count this guard owns.
    connection: Arc<ConnectionState>,
    /// Whether `Drop` still owes request completion.
    active: bool,
}

impl DispatchGuard {
    /// Returns the identity of the connection carrying this dispatch.
    #[cfg(test)]
    pub(super) fn connection_id(&self) -> ConnectionId {
        self.connection.id()
    }

    /// Consumes the guard and records request completion immediately.
    ///
    /// Dropping an uncompleted guard performs the same accounting as a
    /// cancellation fallback.
    pub(super) fn release(mut self) {
        self.active = false;
        self.connection.release_dispatch();
    }
}

impl Drop for DispatchGuard {
    fn drop(&mut self) {
        if self.active {
            self.connection.release_dispatch();
        }
    }
}

/// Tracks the lifetime of the client's physical connection handle.
///
/// The guard is created with the connection and moves with root I/O through
/// protocol drain or upgrade. Dropping it means the client no longer owns its
/// transport handle; the operating system may continue TCP teardown.
#[derive(Debug)]
pub(super) struct PhysicalConnectionGuard {
    /// Shared state whose physical connection lifetime this guard tracks.
    connection: Arc<ConnectionState>,
    /// Whether `Drop` still owes physical connection completion.
    active: bool,
}

impl PhysicalConnectionGuard {
    /// Consumes the guard and records physical connection completion.
    ///
    /// Dropping an uncompleted guard performs the same transition during task
    /// cancellation or runtime shutdown.
    #[cfg(test)]
    pub(super) fn release(mut self) {
        self.active = false;
        self.connection.complete_physical_connection();
    }
}

impl Drop for PhysicalConnectionGuard {
    fn drop(&mut self) {
        if self.active {
            self.connection.complete_physical_connection();
        }
    }
}

pin_project! {
    /// Root transport wrapper that ties physical connection ownership to I/O.
    ///
    /// The wrapper moves intact through Hyper's driver and H1 upgrade path.
    /// Logical close may happen earlier. Dropping the wrapper records that the
    /// client released its physical connection handle after the wrapped I/O
    /// was destroyed; the operating system may continue TCP teardown.
    pub(super) struct ConnectionIo<T> {
        #[pin]
        inner: T,
        // Declared after `inner` so transport destruction precedes the
        // physical-connection completion signal.
        physical: PhysicalConnectionGuard,
    }
}

impl<T> ConnectionIo<T> {
    /// Attaches physical connection lifetime tracking to root transport I/O.
    pub(super) fn new(inner: T, physical: PhysicalConnectionGuard) -> Self {
        Self { inner, physical }
    }

    /// Returns the wrapped transport.
    #[cfg(test)]
    pub(super) fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> fmt::Debug for ConnectionIo<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionIo")
            .field("inner", &self.inner)
            .finish_non_exhaustive()
    }
}

impl<T> Connection for ConnectionIo<T>
where
    T: Connection,
{
    fn connected(&self) -> Connected {
        self.inner.connected()
    }
}

impl<T> Read for ConnectionIo<T>
where
    T: Read,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: ReadBufCursor<'_>,
    ) -> Poll<io::Result<()>> {
        Read::poll_read(self.project().inner, cx, buf)
    }
}

impl<T> Write for ConnectionIo<T>
where
    T: Write,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Write::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Write::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Write::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Write::poll_write_vectored(self.project().inner, cx, bufs)
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use crate::client::pool::admission::OriginAdmission;
    use crate::client::pool::events::{
        ConnectionEvent, ConnectionEvents, SharedConnectionEventListener,
    };
    use aws_smithy_async::time::SharedTimeSource;
    use std::num::NonZeroUsize;
    use std::sync::{Arc as StdArc, Mutex as StdMutex};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum ObservedLifecycleEvent {
        Opened,
        LogicalClose(LogicalCloseCause),
        PhysicalClose(CloseReason),
    }

    fn test_info(id: u64) -> Arc<ConnectionInfo> {
        test_info_for_protocol(id, ConnectionProtocol::Http1)
    }

    fn test_info_for_protocol(id: u64, protocol: ConnectionProtocol) -> Arc<ConnectionInfo> {
        ConnectionInfo::new(
            ConnectionId::new(id),
            OriginKey::from_parts(http_1x::uri::Scheme::HTTPS, "example.com", None)
                .expect("synthetic test origin is valid"),
            PartitionId::from_index(0),
            protocol,
            Connected::new(),
        )
    }

    fn attach_observer(
        connection: &Arc<ConnectionState>,
        observed: StdArc<StdMutex<Vec<ObservedLifecycleEvent>>>,
    ) {
        let events = ConnectionEvents::new(
            Some(SharedConnectionEventListener::new(
                move |event: &ConnectionEvent<'_>| {
                    let event = match event {
                        ConnectionEvent::Opened(_) => ObservedLifecycleEvent::Opened,
                        ConnectionEvent::LogicalClose(closed) => {
                            ObservedLifecycleEvent::LogicalClose(closed.cause())
                        }
                        ConnectionEvent::PhysicalClose(closed) => {
                            ObservedLifecycleEvent::PhysicalClose(closed.reason())
                        }
                        ConnectionEvent::EstablishmentFailed(failed) => {
                            panic!("installed connection failed: {failed:?}")
                        }
                    };
                    observed.lock().unwrap().push(event);
                },
            )),
            SharedTimeSource::default(),
        );
        let establishment = events.establishment_started(
            connection.info().origin(),
            connection.owner_partition(),
            connection.stats.clone(),
        );
        establishment.opened(connection);
    }

    #[test]
    fn lifecycle_callbacks_observe_updated_connection_stats() {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        enum EventKind {
            Opened,
            LogicalClose,
            PhysicalClose,
        }

        let stats = Arc::new(CellConnectionStats::default());
        let observed = StdArc::new(StdMutex::new(Vec::new()));
        let events = ConnectionEvents::new(
            Some(SharedConnectionEventListener::new({
                let stats = stats.clone();
                let observed = observed.clone();
                move |event: &ConnectionEvent<'_>| {
                    let kind = match event {
                        ConnectionEvent::Opened(_) => EventKind::Opened,
                        ConnectionEvent::LogicalClose(_) => EventKind::LogicalClose,
                        ConnectionEvent::PhysicalClose(_) => EventKind::PhysicalClose,
                        ConnectionEvent::EstablishmentFailed(failed) => {
                            panic!("installed connection failed: {failed:?}")
                        }
                    };
                    let snapshot = stats.snapshot(0, 0, 0, 0, 0);
                    observed.lock().unwrap().push((
                        kind,
                        snapshot.establishing_connections(),
                        snapshot.h1().draining(),
                        snapshot.physically_live_connections(),
                    ));
                }
            })),
            SharedTimeSource::default(),
        );
        let mut establishment = events.establishment_started(
            test_info(1).origin(),
            PartitionId::from_index(0),
            stats.clone(),
        );
        establishment.protocol_selected(ConnectionProtocol::Http1);
        let (connection, physical) = ConnectionState::pending_open(
            test_info(1),
            DriverSpawner::new(TestDriverSpawner),
            stats,
        );
        connection.open(None).unwrap();

        establishment.opened(&connection);
        assert!(connection.logical_close(CloseReason::ProtocolClosed));
        physical.release();

        assert_eq!(
            &[
                (EventKind::Opened, 0, 0, 1),
                (EventKind::LogicalClose, 0, 1, 1),
                (EventKind::PhysicalClose, 0, 0, 0),
            ],
            observed.lock().unwrap().as_slice()
        );
    }

    #[test]
    fn logical_close_releases_capacity_before_physical_connection_completion() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);

        assert!(connection.logical_close(CloseReason::Reclaimed));
        assert!(!connection.logical_close(CloseReason::PoolDropped));
        assert_eq!(1, origin.available_capacity_for_test());
        assert!(!connection.probe().physical_connection_complete);
        assert_eq!(
            Some(CloseReason::Reclaimed),
            connection.probe().close_reason
        );

        drop(physical);
        assert!(connection.probe().physical_connection_complete);
    }

    #[test]
    fn installed_events_are_ordered_for_h1_and_h2() {
        for (id, protocol) in [
            (1, ConnectionProtocol::Http1),
            (2, ConnectionProtocol::Http2),
        ] {
            let observed = StdArc::new(StdMutex::new(Vec::new()));
            let (connection, physical) =
                ConnectionState::unbounded(test_info_for_protocol(id, protocol));
            attach_observer(&connection, observed.clone());

            physical.release();
            assert_eq!(
                &[ObservedLifecycleEvent::Opened],
                observed.lock().unwrap().as_slice()
            );

            assert!(connection.logical_close(CloseReason::ProtocolClosed));
            assert_eq!(
                &[
                    ObservedLifecycleEvent::Opened,
                    ObservedLifecycleEvent::LogicalClose(LogicalCloseCause::ProtocolEnded),
                    ObservedLifecycleEvent::PhysicalClose(CloseReason::ProtocolClosed),
                ],
                observed.lock().unwrap().as_slice()
            );
        }
    }

    #[test]
    fn close_before_opened_delivery_is_replayed_in_order() {
        let observed = StdArc::new(StdMutex::new(Vec::new()));
        let (connection, physical) = ConnectionState::unbounded(test_info(1));

        assert!(connection.logical_close(CloseReason::PoolDropped));
        physical.release();
        assert!(observed.lock().unwrap().is_empty());

        attach_observer(&connection, observed.clone());
        assert_eq!(
            &[
                ObservedLifecycleEvent::Opened,
                ObservedLifecycleEvent::LogicalClose(LogicalCloseCause::PoolDropped),
                ObservedLifecycleEvent::PhysicalClose(CloseReason::PoolDropped),
            ],
            observed.lock().unwrap().as_slice()
        );
    }

    #[test]
    fn close_callbacks_may_reenter_connection_state() {
        let observed = StdArc::new(StdMutex::new(Vec::new()));
        let connection_slot = StdArc::new(StdMutex::new(None::<Arc<ConnectionState>>));
        let events = ConnectionEvents::new(
            Some(SharedConnectionEventListener::new({
                let observed = observed.clone();
                let connection_slot = connection_slot.clone();
                move |event: &ConnectionEvent<'_>| {
                    let event = match event {
                        ConnectionEvent::Opened(_) => return,
                        ConnectionEvent::LogicalClose(closed) => {
                            ObservedLifecycleEvent::LogicalClose(closed.cause())
                        }
                        ConnectionEvent::PhysicalClose(closed) => {
                            ObservedLifecycleEvent::PhysicalClose(closed.reason())
                        }
                        ConnectionEvent::EstablishmentFailed(failed) => {
                            panic!("installed connection failed: {failed:?}")
                        }
                    };
                    let connection = connection_slot
                        .lock()
                        .unwrap()
                        .clone()
                        .expect("connection installed before callback");
                    // Recorded after reentry so a panic swallowed by the
                    // listener boundary leaves the observation missing.
                    let closed_again = connection.logical_close(CloseReason::PoolDropped);
                    observed.lock().unwrap().push((event, closed_again));
                }
            })),
            SharedTimeSource::default(),
        );
        let (connection, physical) = ConnectionState::unbounded(test_info(1));
        *connection_slot.lock().unwrap() = Some(connection.clone());
        let establishment = events.establishment_started(
            connection.info().origin(),
            connection.owner_partition(),
            connection.stats.clone(),
        );
        establishment.opened(&connection);

        assert!(connection.logical_close(CloseReason::Poisoned));
        physical.release();
        assert_eq!(
            &[
                (
                    ObservedLifecycleEvent::LogicalClose(LogicalCloseCause::Poisoned),
                    false
                ),
                (
                    ObservedLifecycleEvent::PhysicalClose(CloseReason::Poisoned),
                    false
                ),
            ],
            observed.lock().unwrap().as_slice()
        );
    }

    #[test]
    fn panicking_close_listener_does_not_change_lifecycle_state() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
        let events = ConnectionEvents::new(
            Some(SharedConnectionEventListener::new(
                |_: &ConnectionEvent<'_>| panic!("listener failed"),
            )),
            SharedTimeSource::default(),
        );
        let establishment = events.establishment_started(
            connection.info().origin(),
            connection.owner_partition(),
            connection.stats.clone(),
        );
        establishment.opened(&connection);

        assert!(connection.logical_close(CloseReason::Reclaimed));
        physical.release();

        assert_eq!(1, origin.available_capacity_for_test());
        assert_eq!(
            Some(CloseReason::Reclaimed),
            connection.probe().close_reason
        );
        assert!(connection.probe().physical_connection_complete);
    }
    #[test]
    fn driver_first_upgrade_retains_capacity_until_physical_completion() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();

        assert!(connection.logical_close(CloseReason::ProtocolClosed));
        assert!(connection.probe().awaiting_h1_exchange);
        assert_eq!(0, origin.available_capacity_for_test());

        assert!(connection.complete_h1_exchange(CloseReason::Upgraded));
        assert_eq!(Some(CloseReason::Upgraded), connection.probe().close_reason);
        assert_eq!(0, origin.available_capacity_for_test());
        dispatch.release();

        physical.release();
        assert_eq!(1, origin.available_capacity_for_test());
    }

    #[test]
    fn exchange_first_upgrade_retains_capacity_until_physical_completion() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();

        assert!(connection.logical_close(CloseReason::Upgraded));
        assert_eq!(Some(CloseReason::Upgraded), connection.probe().close_reason);
        assert_eq!(0, origin.available_capacity_for_test());
        dispatch.release();

        physical.release();
        assert_eq!(1, origin.available_capacity_for_test());
    }

    #[test]
    fn non_upgrade_exchange_returns_capacity_before_physical_completion() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();

        assert!(connection.logical_close(CloseReason::ProtocolClosed));
        assert!(connection.complete_h1_exchange(CloseReason::ProtocolClosed));
        assert_eq!(
            Some(CloseReason::ProtocolClosed),
            connection.probe().close_reason
        );
        assert_eq!(1, origin.available_capacity_for_test());
        assert!(!connection.probe().physical_connection_complete);

        dispatch.release();
        physical.release();
    }

    #[test]
    fn non_upgrade_exchange_preserves_owner_runtime_shutdown_reason() {
        let observed = StdArc::new(StdMutex::new(Vec::new()));
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let lease = OriginAdmission::lease_for_test(&origin);
        let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
        attach_observer(&connection, observed.clone());
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();

        assert!(connection.logical_close(CloseReason::OwnerRuntimeShutdown));
        assert!(connection.complete_h1_exchange(CloseReason::ProtocolClosed));
        assert_eq!(
            Some(CloseReason::OwnerRuntimeShutdown),
            connection.probe().close_reason
        );
        assert_eq!(1, origin.available_capacity_for_test());
        dispatch.release();

        physical.release();
        assert_eq!(
            &[
                ObservedLifecycleEvent::Opened,
                ObservedLifecycleEvent::LogicalClose(LogicalCloseCause::OwnerRuntimeShutdown,),
                ObservedLifecycleEvent::PhysicalClose(CloseReason::OwnerRuntimeShutdown,),
            ],
            observed.lock().unwrap().as_slice()
        );
    }

    #[test]
    fn exchange_classification_does_not_replace_a_policy_close() {
        let (connection, _physical) = ConnectionState::unbounded(test_info(1));
        assert!(connection.logical_close(CloseReason::Poisoned));
        assert!(!connection.complete_h1_exchange(CloseReason::Upgraded));
        assert_eq!(Some(CloseReason::Poisoned), connection.probe().close_reason);
    }
    #[test]
    fn committed_dispatch_drains_after_logical_close() {
        let (connection, _physical) = ConnectionState::unbounded(test_info(1));
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();
        assert_eq!(ConnectionId::new(1), dispatch.connection_id());

        assert!(connection.logical_close(CloseReason::ProtocolClosed));
        assert!(ConnectionState::try_commit_dispatch(&connection).is_none());
        assert_eq!(1, connection.probe().in_flight);

        dispatch.release();
        assert_eq!(0, connection.probe().in_flight);
    }

    #[test]
    fn physical_connection_guard_is_created_once_with_the_connection() {
        let (connection, physical) = ConnectionState::unbounded(test_info(1));
        assert_eq!(ConnectionId::new(1), connection.id());
        assert_eq!(PartitionId::from_index(0), connection.owner_partition());
        assert!(!connection.probe().physical_connection_complete);
        physical.release();
        assert!(connection.probe().physical_connection_complete);
    }

    #[test]
    fn connection_state_retains_immutable_connection_info() {
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct ConnectorMarker(&'static str);

        let origin =
            OriginKey::from_parts(http_1x::uri::Scheme::HTTPS, "example.com", None).unwrap();
        let info = ConnectionInfo::new(
            ConnectionId::new(7),
            origin.clone(),
            PartitionId::from_index(2),
            ConnectionProtocol::Http1,
            Connected::new()
                .proxy(true)
                .extra(ConnectorMarker("connector-extra")),
        );
        let (connection, _physical) = ConnectionState::unbounded(info);

        assert_eq!(ConnectionId::new(7), connection.info().id());
        assert_eq!(&origin, connection.info().origin());
        assert_eq!(PartitionId::from_index(2), connection.owner_partition());
        assert_eq!(ConnectionProtocol::Http1, connection.info().protocol());
        assert_eq!(None, connection.info().local_addr());
        assert_eq!(None, connection.info().remote_addr());
        assert_eq!(ConnectPath::ForwardProxy, connection.info().connect_path());
        assert_eq!(None, connection.info().establishment());
        let mut extensions = Extensions::new();
        connection.info().apply_connector_extras(&mut extensions);
        assert_eq!(
            Some(&ConnectorMarker("connector-extra")),
            extensions.get::<ConnectorMarker>()
        );
    }

    #[test]
    fn dropping_connection_io_completes_physical_connection_ownership() {
        let (connection, physical) = ConnectionState::unbounded(test_info(1));
        let io = ConnectionIo::new("transport", physical);
        assert_eq!(&"transport", io.get_ref());
        assert!(!connection.probe().physical_connection_complete);

        drop(io);

        assert!(connection.probe().physical_connection_complete);
    }
}

#[cfg(all(test, smithy_http_client_loom))]
mod loom_tests {
    use super::*;
    use crate::client::pool::admission::OriginAdmission;
    use crate::client::pool::events::{
        ConnectionEvent, ConnectionEvents, SharedConnectionEventListener,
    };
    use aws_smithy_async::time::SharedTimeSource;
    use loom::sync::atomic::{AtomicUsize, Ordering};
    use std::num::NonZeroUsize;

    fn test_info(id: u64) -> Arc<ConnectionInfo> {
        ConnectionInfo::for_test(ConnectionId::new(id), PartitionId::from_index(0))
    }

    /// Races request dispatch commitment with logical close.
    ///
    /// Dispatch either commits before close and drains afterward, or close
    /// rejects it without incrementing the in-flight count.
    #[test]
    fn dispatch_commit_linearizes_against_close() {
        loom::model(|| {
            let (connection, _physical) = ConnectionState::unbounded(test_info(1));

            let dispatch_connection = connection.clone();
            let dispatch = loom::thread::spawn(move || {
                ConnectionState::try_commit_dispatch(&dispatch_connection)
            });
            let close_connection = connection.clone();
            let close = loom::thread::spawn(move || {
                close_connection.logical_close(CloseReason::ProtocolClosed)
            });

            let dispatch = dispatch.join().unwrap();
            assert!(close.join().unwrap());
            assert!(ConnectionState::try_commit_dispatch(&connection).is_none());
            match dispatch {
                Some(dispatch) => {
                    let probe = connection.probe();
                    assert_eq!(None, probe.close_reason);
                    assert!(probe.awaiting_h1_exchange);
                    assert_eq!(1, probe.in_flight);
                    drop(dispatch);
                    let probe = connection.probe();
                    assert_eq!(Some(CloseReason::ProtocolClosed), probe.close_reason);
                    assert!(!probe.awaiting_h1_exchange);
                    assert_eq!(0, probe.in_flight);
                }
                None => {
                    let probe = connection.probe();
                    assert_eq!(Some(CloseReason::ProtocolClosed), probe.close_reason);
                    assert!(!probe.awaiting_h1_exchange);
                    assert_eq!(0, probe.in_flight);
                }
            }
        });
    }

    /// Races two independent logical-close signals for one bounded connection.
    ///
    /// Exactly one reason becomes authoritative and its capacity lease returns once.
    #[test]
    fn concurrent_logical_close_releases_one_capacity_lease() {
        loom::model(|| {
            let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
            let lease = OriginAdmission::lease_for_test(&origin);
            let (connection, _physical) = ConnectionState::bounded(test_info(1), lease);
            let first_connection = connection.clone();
            let first =
                loom::thread::spawn(move || first_connection.logical_close(CloseReason::Poisoned));
            let second_connection = connection.clone();
            let second = loom::thread::spawn(move || {
                second_connection.logical_close(CloseReason::PoolDropped)
            });

            let first = first.join().unwrap();
            let second = second.join().unwrap();
            assert_ne!(first, second);
            assert_eq!(1, origin.available_capacity_for_test());
            let reason = connection.probe().close_reason.unwrap();
            assert!(matches!(
                reason,
                CloseReason::Poisoned | CloseReason::PoolDropped
            ));
        });
    }

    /// Races HTTP/1 upgrade classification with physical connection completion.
    ///
    /// The winning transition determines the final reason while both paths
    /// converge on one capacity return and zero retained lifetime counts.
    #[test]
    fn h1_upgrade_classification_linearizes_against_physical_completion() {
        loom::model(|| {
            let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
            let lease = OriginAdmission::lease_for_test(&origin);
            let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
            let dispatch = ConnectionState::try_commit_dispatch(&connection)
                .expect("open HTTP/1 connection rejected dispatch");
            assert!(connection.logical_close(CloseReason::ProtocolClosed));

            let classify_connection = connection.clone();
            let classify = loom::thread::spawn(move || {
                classify_connection.complete_h1_exchange(CloseReason::Upgraded)
            });
            let complete_physical = loom::thread::spawn(move || physical.release());

            let classified_as_upgrade = classify.join().unwrap();
            complete_physical.join().unwrap();
            dispatch.release();

            let probe = connection.probe();
            assert_eq!(
                Some(if classified_as_upgrade {
                    CloseReason::Upgraded
                } else {
                    CloseReason::ProtocolClosed
                }),
                probe.close_reason
            );
            assert!(!probe.awaiting_h1_exchange);
            assert_eq!(0, probe.in_flight);
            assert!(probe.physical_connection_complete);
            assert_eq!(1, origin.available_capacity_for_test());

            let stats = connection.stats.snapshot(0, 0, 0, 0, 0);
            assert_eq!(0, stats.h1().draining());
            assert_eq!(0, stats.h1().upgraded());
            assert_eq!(0, stats.physically_live_connections());
        });
    }

    /// Races `Opened`, accepted HTTP/1 completion, and both close transitions.
    ///
    /// Each installed event must be delivered once in
    /// `Opened -> LogicalClose -> PhysicalClose` order.
    #[test]
    fn installed_events_remain_ordered_across_concurrent_close() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let event_sequence = Arc::new(AtomicUsize::new(0));
            let observed = event_sequence.clone();
            let events = ConnectionEvents::new(
                Some(SharedConnectionEventListener::new(
                    move |event: &ConnectionEvent<'_>| {
                        let code = match event {
                            ConnectionEvent::Opened(_) => 1,
                            ConnectionEvent::LogicalClose(_) => 2,
                            ConnectionEvent::PhysicalClose(_) => 3,
                            ConnectionEvent::EstablishmentFailed(_) => 4,
                        };
                        observed
                            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |sequence| {
                                Some(sequence * 4 + code)
                            })
                            .expect("event sequence update is infallible");
                    },
                )),
                SharedTimeSource::default(),
            );
            let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
            let lease = OriginAdmission::lease_for_test(&origin);
            let (connection, physical) = ConnectionState::bounded(test_info(1), lease);
            let dispatch = ConnectionState::try_commit_dispatch(&connection)
                .expect("open HTTP/1 connection rejected dispatch");
            let establishment = events.establishment_started(
                connection.info().origin(),
                connection.owner_partition(),
                connection.stats.clone(),
            );

            let opened_connection = connection.clone();
            let opened = loom::thread::spawn(move || establishment.opened(&opened_connection));
            let close_connection = connection.clone();
            let close = loom::thread::spawn(move || {
                close_connection.logical_close(CloseReason::ProtocolClosed)
            });
            let complete_physical = loom::thread::spawn(move || physical.release());
            let complete_dispatch = loom::thread::spawn(move || dispatch.release());

            opened.join().unwrap();
            assert!(close.join().unwrap());
            complete_physical.join().unwrap();
            complete_dispatch.join().unwrap();

            assert_eq!(
                27,
                event_sequence.load(Ordering::SeqCst),
                "installed connection events were missing, duplicated, or reordered"
            );
            assert_eq!(1, origin.available_capacity_for_test());
            let probe = connection.probe();
            assert_eq!(Some(CloseReason::ProtocolClosed), probe.close_reason);
            assert!(!probe.awaiting_h1_exchange);
            assert_eq!(0, probe.in_flight);
            assert!(probe.physical_connection_complete);
        });
    }
}
