/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Connection pooling with explicit runtime and network placement.
//!
//! A [`ConnectionPool`] owns connection policy and a fixed set of partitions.
//! A [`Client`] binds Smithy HTTP operations to one partition. Pools without
//! explicit [`Partition`] values contain one anonymous partition; otherwise a
//! client selects a declared partition by [`PartitionId`].
//!
//! A partition owns connection establishment, protocol drivers, and idle
//! maintenance. [`ConnectionReuseScope`] controls whether another partition may
//! dispatch through those connections. Reuse transfers protocol dispatch
//! authority; it never moves the socket, driver, or capacity accounting.
//! Partitions in the same eligibility group are exactly those whose configured
//! reuse scope permits them to share a connection.
//!
//! Connection establishment and installed connection lifetime are separate
//! ownership phases:
//!
//! ```text
//! establishment task
//!     |-- DNS, socket, proxy, TLS, and ALPN
//!     `-- negotiated transport
//!             `-- PendingOpen
//!                     |-- Hyper protocol setup
//!                     `-- Open
//!                             `-- pool installation
//!                                     `-- discoverable -> draining -> closed
//! ```
//!
//! The establishment task and its permit represent work that may fail before a
//! physical connection exists. `ConnectionState` begins only after the
//! connector returns connected I/O and selects HTTP/1 or HTTP/2. For TLS, that
//! boundary follows TLS and ALPN. Hyper protocol setup moves the state from
//! `PendingOpen` to `Open`; cell installation then makes the connection
//! discoverable. Establishment and connection events can therefore be observed
//! independently without representing failed attempts as installed
//! connections.
//!
//! # State ownership
//!
//! ```text
//! ConnectionPool
//! `-- PoolInner
//!     |-- immutable policy and transport factory
//!     `-- PartitionRegistry
//!         |-- PartitionState per partition
//!         |   |-- runtime placement and idle maintenance
//!         |   `-- OriginCell per canonical origin
//!         |       |-- acquisition queue and supply revisions
//!         |       |-- H1 connection ownership
//!         |       `-- H2 flights, generations, routes, and gates
//!         `-- OriginAdmission per bounded origin
//!             |-- capacity budget
//!             |-- demand schedule
//!             |-- H1 supply index and retained matches
//!             `-- H2 supply index and route/reclaim state
//! ```
//!
//! One `OriginCell` lock owns local acquisition order and protocol state.
//! For a bounded origin, `OriginAdmission` separately owns the origin-wide
//! connection limit and cross-cell matching. An H2 request-claim lock records
//! independent upload and response completion. `ConnectionState` owns logical
//! connection lifetime, and partition maintenance owns its timer state.
//!
//! No two pool locks are held together. Demand snapshots and supply revisions
//! move cell state into admission. Assignments and detached guards carry one
//! selected payload or route back toward a cell. H2 claim completion detaches
//! its dispatch guard before entering connection or cell state.
//! Maintenance detaches cells and wakers before expiration or wake callbacks.
//!
//! # HTTP/1 request lifecycle
//!
//! Hyper represents an HTTP/1 connection with one exclusive
//! `SendRequest<SdkBody>` handle. This module calls that handle the sender. The
//! sender authorizes request dispatch but does not own the socket or protocol
//! driver.
//!
//! ```text
//! Client(partition, request)
//! `-- OriginCell(partition, origin)
//!     |-- local idle sender --------------------------> H1Selection
//!     `-- acquisition queue
//!         |-- returned local or eligible peer sender -> H1Selection
//!         |-- capacity permit -> establish HTTP/1 ---> H1Selection
//!         `-- reclaimed peer capacity -> establish ---> H1Selection
//!
//! H1Selection -- Hyper accepts request --> H1Exchange
//! H1Exchange
//!     |-- complete response + ready --> offer to owning OriginCell
//!     `-- failure, cancellation, or upgrade ----------> retire pool record
//! ```
//!
//! A local hit touches only the cell lock. On a miss, one queued acquisition
//! remains authoritative while a returned sender and establishment race to
//! satisfy it. Bounded origins may borrow an eligible peer sender or reclaim a
//! peer connection and transfer its permit; active connections are not
//! reclaimed.
//!
//! Dispatch commits against logical close before Hyper receives the request.
//! Once accepted, the response lifecycle retains the sender until Hyper proves
//! a complete reusable message boundary. Failure retires the connection, and
//! an upgrade closes the pool record before exposing upgraded root I/O.
//!
//! # HTTP/2 request lifecycle
//!
//! One HTTP/2 connection carries many concurrent request streams. The pool
//! calls one installed incarnation of that connection a generation. A
//! replacement connection receives a new generation identity so delayed
//! close, route, and completion work cannot affect it.
//!
//! The connection-owning cell retains the generation's authoritative Hyper
//! request handle and capacity. A requesting cell may retain only a route that
//! names the owning cell and one exact accepting generation. Each use
//! revalidates that route before cloning a transient request handle.
//!
//! ```text
//! Client(partition, request)
//! `-- OriginCell(partition, origin)
//!     |-- local accepting generation ----------------> H2Activation
//!     `-- acquisition queue
//!         |-- local flight result --------------------> H2Activation
//!         |-- eligible peer generation route --------> H2Activation
//!         `-- capacity permit -> connect + ALPN
//!             |-- HTTP/2 -> join or drive one flight -> H2Activation
//!             `-- HTTP/1 -> H1Selection or incompatible-version error
//!
//! H2Activation -- Hyper accepts request --> H2RequestClaim
//! H2RequestClaim
//!     |-- request body ends or drops -----> upload side complete
//!     `-- response body ends or drops ----> response side complete
//! both sides complete --------------------> release generation request count
//! ```
//!
//! `H2Activation` reserves pool accounting for a prospective stream on one
//! exact generation. It is not yet an HTTP/2 stream. Dropping it before Hyper
//! accepts the request returns its generation-gate turn and request count.
//! Acceptance creates two independent completion sides because upload and response
//! can finish in either order. Logical close stops new activations and releases
//! bounded capacity; accepted streams retain the draining generation until
//! both sides end. Hyper remains responsible for stream identifiers,
//! stream credit, and flow control.
//!
//! A peer route moves only generation identity. The socket, protocol driver,
//! request handle, and capacity remain with the connection-owning partition.
//!
//! `ConnectionState` separates logical close, accepted-request accounting, and
//! physical connection ownership. Logical close rejects new dispatch and
//! normally releases bounded capacity while accepted work drains. An HTTP/1
//! upgrade retains capacity until upgraded root I/O leaves the client.
//! `DispatchGuard` follows an accepted request, while
//! `PhysicalConnectionGuard` follows root I/O until the client releases its
//! transport handle. The operating system may continue TCP teardown afterward.
//! All connection-owned work runs through the partition
//! [`DriverSpawner`].
//!
//! # Observation
//!
//! A pool-wide [`ConnectionEventListener`] observes establishment failure,
//! successful installation, logical close, and release of the client's root
//! transport handle. Callbacks run synchronously after pool locks are released.
//! Observation is disabled unless a listener is configured.
//!
//! [`ConnectionPool::origin_stats`] reports the bounded capacity shared by all
//! partitions for one canonical origin. [`ConnectionPool::partition_stats`]
//! reports request acquisition and connection state for one exact
//! partition-origin cell. These snapshots are diagnostic; the pool does not use
//! them for admission, reuse, reclaim, routing, or dispatch.

#![cfg_attr(
    smithy_http_client_loom,
    allow(
        dead_code,
        reason = "Loom builds replace runtime and transport paths with focused coordination models"
    )
)]

mod admission;
mod builder;
mod cell;
mod client;
mod connection;
mod dispatch;
mod establish;
mod events;
mod maintenance;
mod origin;
mod partition;
mod registry;
mod stats;

pub use builder::{BuildError, Builder};
pub use client::{Client, ClientBuildError};
pub use connection::{CloseReason, ConnectionId, ConnectionInfo, ConnectionProtocol};
pub use events::{
    ConnectionEstablishmentFailed, ConnectionEstablishmentId, ConnectionEstablishmentInfo,
    ConnectionEstablishmentStage, ConnectionEstablishmentStats, ConnectionEvent,
    ConnectionEventListener, ConnectionLogicalClose, ConnectionOpened, ConnectionPhysicalClose,
    LogicalCloseCause, SharedConnectionEventListener,
};
pub use origin::{InvalidOrigin, OriginKey};
pub use partition::{ConnectionReuseScope, DriverSpawner, Partition, PartitionId};
pub use stats::{
    ConnectionCapacityStats, Http1ConnectionStats, Http2ConnectionStats, OriginConnectionStats,
    PartitionConnectionStats,
};

pub use crate::client::connect::ConnectPath;
use crate::sync::Arc;
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_types::body::SdkBody;
use establish::TransportFactory;
use http_1x::{Request, Response};
use registry::{PartitionRegistry, PartitionState};
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicU64;
use std::sync::Arc as StdArc;
use std::time::Duration;

/// Shared connection topology and pooling policy.
///
/// Construct a pool with [`ConnectionPool::builder`], then create [`Client`]
/// handles for its anonymous or explicit partitions. The pool itself owns no
/// request placement; each client supplies that partition choice. Cloning a
/// pool or client shares all retained connections and admission state.
///
/// A pool built with explicit partitions does not also create an anonymous
/// partition. Dropping the final shared owner logically closes retained
/// connections and stops partition maintenance.
#[derive(Clone)]
pub struct ConnectionPool {
    /// Shared pool policy, topology, connector, and connection-ID allocator.
    inner: Arc<PoolInner>,
}

impl ConnectionPool {
    /// Returns a builder for a new connection pool.
    pub fn builder() -> Builder<super::TlsUnset> {
        Builder::default()
    }

    /// Returns current bounded-capacity accounting for one canonical origin.
    ///
    /// The capacity limit applies across every partition. Capacity in use
    /// includes establishments, open connections, and detached HTTP/1 upgrades
    /// that still own root transport I/O. Ordinary draining connections are
    /// excluded after returning their capacity.
    ///
    /// Unbounded origins return statistics without a capacity value. The query
    /// does not create admission state for an unused origin.
    pub fn origin_stats(&self, origin: &OriginKey) -> OriginConnectionStats {
        self.inner.registry.origin_stats(origin)
    }

    /// Returns a diagnostic connection snapshot for one partition and origin.
    ///
    /// An unknown partition returns `None`. A configured partition without a
    /// retained cell for `origin` returns zeroed statistics. Cell-owned values
    /// are read under the existing cell lock. Counts for lifetimes that can
    /// outlive cell records use relaxed atomics and converge after concurrent
    /// transitions settle.
    pub fn partition_stats(
        &self,
        partition: PartitionId,
        origin: &OriginKey,
    ) -> Option<PartitionConnectionStats> {
        self.inner.registry.partition_stats(partition, origin)
    }

    /// Routes one request from its selected partition through pool dispatch.
    pub(in crate::client::pool) async fn send_request(
        &self,
        partition: Arc<PartitionState>,
        request: Request<SdkBody>,
        options: dispatch::RequestOptions,
    ) -> Result<Response<SdkBody>, ConnectorError> {
        dispatch::send(self, partition, request, options).await
    }
}

impl fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionPool")
            .field("partitions", &self.inner.registry)
            .field("idle_timeout", &self.inner.config.idle_timeout)
            .field(
                "max_connections_per_host",
                &self.inner.config.max_connections_per_host,
            )
            .field("reuse_scope", &self.inner.config.reuse_scope)
            .finish_non_exhaustive()
    }
}

/// Immutable pool policy retained with the partition registry.
#[derive(Clone, Debug)]
struct PoolConfig {
    /// Duration after which a reusable idle connection is retired.
    idle_timeout: Option<Duration>,
    /// Optional logical-connection bound shared by every partition for one origin.
    max_connections_per_host: Option<NonZeroUsize>,
    /// Partitions allowed to dispatch through one another's connections.
    reuse_scope: ConnectionReuseScope,
}

/// Shared implementation state behind [`ConnectionPool`].
struct PoolInner {
    /// Immutable settings shared by pool operations.
    config: PoolConfig,
    /// Fixed partitions and lazily created per-origin state.
    registry: PartitionRegistry,
    /// Type-erased construction of one partition-bound transport.
    transport: StdArc<dyn TransportFactory>,
    /// Pool-wide connection lifecycle observation.
    connection_events: events::ConnectionEvents,
    /// Monotonic identity source shared by every physical connection.
    next_connection_id: AtomicU64,
}

impl Drop for PoolInner {
    fn drop(&mut self) {
        self.registry.close_all(CloseReason::PoolDropped);
    }
}

impl fmt::Debug for PoolInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolInner")
            .field("config", &self.config)
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}
