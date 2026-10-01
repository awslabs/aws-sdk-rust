/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Connection establishment and installed-lifetime observations.
//!
//! Event payloads borrow transition-owned values for one synchronous callback.
//! Listener invocation occurs after pool state transitions release their locks.
//! One observed establishment emits either [`ConnectionEvent::EstablishmentFailed`]
//! or [`ConnectionEvent::Opened`]. An opened connection then emits
//! [`ConnectionEvent::LogicalClose`] before [`ConnectionEvent::PhysicalClose`].

use super::connection::{CloseReason, ConnectionInfo, ConnectionProtocol, ConnectionState};
use super::origin::OriginKey;
use super::partition::PartitionId;
use super::stats::CellConnectionStats;
use crate::sync::Arc as PoolArc;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_runtime_api::client::result::ConnectorError;
use std::error::Error;
use std::fmt;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Receives connection lifecycle observations from one pool.
///
/// Callbacks run synchronously on the task that performs the observed
/// transition. They must not block on work that requires progress from the
/// same task. Pool locks are released before invocation. On unwind-capable
/// builds, a listener panic is caught after the authoritative transition and
/// cannot roll back pool state or cleanup.
pub trait ConnectionEventListener: Send + Sync + 'static {
    /// Observes one completed lifecycle transition.
    fn on_event(&self, event: &ConnectionEvent<'_>);
}

impl<F> ConnectionEventListener for F
where
    F: for<'a> Fn(&ConnectionEvent<'a>) + Send + Sync + 'static,
{
    fn on_event(&self, event: &ConnectionEvent<'_>) {
        self(event);
    }
}

/// Cloneable, type-erased connection event listener.
///
/// Use this wrapper with mutable builder configuration or when several pools
/// share one listener.
#[derive(Clone)]
pub struct SharedConnectionEventListener(Arc<dyn ConnectionEventListener>);

impl SharedConnectionEventListener {
    /// Creates a shared listener from one concrete implementation.
    pub fn new(listener: impl ConnectionEventListener) -> Self {
        Self(Arc::new(listener))
    }

    /// Invokes the listener without allowing observer panics to alter pool state.
    fn notify(&self, event: &ConnectionEvent<'_>) {
        #[cfg(panic = "unwind")]
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.on_event(event))).is_err()
        {
            tracing::warn!("connection event listener panicked");
        }

        #[cfg(not(panic = "unwind"))]
        self.on_event(event);
    }

    /// Reports the first transition that rejects new dispatch.
    pub(super) fn logical_close(
        &self,
        connection: &PoolArc<ConnectionInfo>,
        cause: LogicalCloseCause,
    ) {
        self.notify(&ConnectionEvent::LogicalClose(ConnectionLogicalClose {
            connection,
            cause,
        }));
    }

    /// Reports release of the client's root transport ownership.
    pub(super) fn physical_close(&self, connection: &PoolArc<ConnectionInfo>, reason: CloseReason) {
        self.notify(&ConnectionEvent::PhysicalClose(ConnectionPhysicalClose {
            connection,
            reason,
        }));
    }
}

impl ConnectionEventListener for SharedConnectionEventListener {
    fn on_event(&self, event: &ConnectionEvent<'_>) {
        self.0.on_event(event);
    }
}

impl fmt::Debug for SharedConnectionEventListener {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SharedConnectionEventListener")
    }
}

/// One connection establishment or installed-lifetime observation.
///
/// Payload references are valid for the synchronous callback. Clone or copy
/// the individual values needed after the callback returns.
#[derive(Debug)]
#[non_exhaustive]
pub enum ConnectionEvent<'a> {
    /// An establishment ended without installing a connection.
    EstablishmentFailed(ConnectionEstablishmentFailed<'a>),
    /// An establishment installed a connection for dispatch.
    Opened(ConnectionOpened<'a>),
    /// An installed connection stopped accepting new dispatch.
    LogicalClose(ConnectionLogicalClose<'a>),
    /// The client released its root transport ownership.
    PhysicalClose(ConnectionPhysicalClose<'a>),
}

/// Opaque identity for one connection establishment.
///
/// Establishment IDs are unique within their assigning pool. A retry that
/// starts another establishment receives another ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionEstablishmentId(u64);

impl fmt::Display for ConnectionEstablishmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Stable identity and routing context for one pool-level establishment.
///
/// Address fallback performed inside a connector remains part of this one
/// establishment.
#[derive(Debug)]
pub struct ConnectionEstablishmentInfo {
    id: ConnectionEstablishmentId,
    origin: OriginKey,
    partition: PartitionId,
}

impl ConnectionEstablishmentInfo {
    /// Returns this establishment's pool-assigned identity.
    pub fn id(&self) -> ConnectionEstablishmentId {
        self.id
    }

    /// Returns the canonical origin being connected.
    pub fn origin(&self) -> &OriginKey {
        &self.origin
    }

    /// Returns the partition that owns the establishment task.
    pub fn partition(&self) -> PartitionId {
        self.partition
    }
}

/// Measurements collected during one pool-level connection establishment.
///
/// A later retry receives a different [`ConnectionEstablishmentId`] and its own
/// measurements.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct ConnectionEstablishmentStats {
    total_duration: Duration,
    transport_duration: Duration,
    protocol_handshake_duration: Option<Duration>,
}

impl ConnectionEstablishmentStats {
    /// Returns elapsed time through successful protocol installation or terminal failure.
    ///
    /// Successful measurements are frozen before the connection is published
    /// to waiting pool demand.
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Returns elapsed time spent in the configured transport connector.
    ///
    /// This may include DNS, socket, proxy, and TLS work. Custom connectors may
    /// perform a different set of transport operations.
    pub fn transport_duration(&self) -> Duration {
        self.transport_duration
    }

    /// Returns elapsed time spent in the Hyper protocol handshake, when started.
    ///
    /// Failures before protocol handshake return `None`.
    pub fn protocol_handshake_duration(&self) -> Option<Duration> {
        self.protocol_handshake_duration
    }

    fn connection_metadata(
        &self,
    ) -> aws_smithy_runtime_api::client::connection::ConnectionEstablishmentMetadata {
        let mut builder =
            aws_smithy_runtime_api::client::connection::ConnectionEstablishmentMetadata::builder()
                .total_duration(self.total_duration)
                .transport_duration(self.transport_duration);
        builder.set_protocol_handshake_duration(self.protocol_handshake_duration);
        builder.build()
    }
}

/// Stage that terminated one failed connection establishment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionEstablishmentStage {
    /// Work performed by the configured transport connector.
    Transport,
    /// Selection of an HTTP protocol compatible with the request.
    ProtocolSelection,
    /// Hyper's HTTP/1 or HTTP/2 client handshake.
    ProtocolHandshake,
    /// Transfer of a handshaken connection into pool-owned protocol state.
    PoolInstallation,
}

/// A terminal failure before a connection became available for pool dispatch.
#[derive(Debug)]
pub struct ConnectionEstablishmentFailed<'a> {
    establishment: &'a ConnectionEstablishmentInfo,
    stats: &'a ConnectionEstablishmentStats,
    stage: ConnectionEstablishmentStage,
    remote_addr: Option<SocketAddr>,
    protocol: Option<ConnectionProtocol>,
    error: &'a (dyn Error + Send + Sync),
}

impl<'a> ConnectionEstablishmentFailed<'a> {
    /// Returns the failed establishment's identity and routing context.
    pub fn establishment(&self) -> &'a ConnectionEstablishmentInfo {
        self.establishment
    }

    /// Returns measurements collected before the failure.
    pub fn stats(&self) -> &'a ConnectionEstablishmentStats {
        self.stats
    }

    /// Returns the stage that terminated the establishment.
    pub fn stage(&self) -> ConnectionEstablishmentStage {
        self.stage
    }

    /// Returns the connector-reported remote address, when transport completed
    /// before a later stage failed.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Returns the selected HTTP protocol, when selection completed.
    pub fn protocol(&self) -> Option<ConnectionProtocol> {
        self.protocol
    }

    /// Returns the terminal error delivered to connection acquisition.
    pub fn error(&self) -> &'a (dyn Error + Send + Sync) {
        self.error
    }
}

/// A connection installed and available for pool dispatch.
#[derive(Debug)]
pub struct ConnectionOpened<'a> {
    establishment: &'a ConnectionEstablishmentInfo,
    stats: &'a ConnectionEstablishmentStats,
    connection: &'a PoolArc<ConnectionInfo>,
}

impl<'a> ConnectionOpened<'a> {
    /// Returns the establishment that created this connection.
    pub fn establishment(&self) -> &'a ConnectionEstablishmentInfo {
        self.establishment
    }

    /// Returns measurements frozen after protocol installation and before pool publication.
    pub fn stats(&self) -> &'a ConnectionEstablishmentStats {
        self.stats
    }

    /// Returns immutable identity and transport facts for the connection.
    pub fn connection(&self) -> &'a PoolArc<ConnectionInfo> {
        self.connection
    }
}

/// Stable cause recorded when pool dispatch ownership ends.
///
/// The later physical-close event may carry a more specific [`CloseReason`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum LogicalCloseCause {
    /// The connection exceeded its configured idle timeout.
    IdleTimeout,
    /// The connection was explicitly marked unsafe for reuse.
    Poisoned,
    /// Hyper's protocol dispatch ownership ended.
    ProtocolEnded,
    /// HTTP/1 did not prove a complete reusable message boundary.
    IncompleteH1Exchange,
    /// The connection closed to move bounded capacity to another cell.
    Reclaimed,
    /// The connection pool was dropped.
    PoolDropped,
    /// The runtime driving the connection shut down.
    OwnerRuntimeShutdown,
}

impl LogicalCloseCause {
    /// Maps the first pool close classification to its stable logical cause.
    pub(super) fn from_reason(reason: CloseReason) -> Self {
        match reason {
            CloseReason::IdleTimeout => Self::IdleTimeout,
            CloseReason::Poisoned => Self::Poisoned,
            CloseReason::ProtocolClosed | CloseReason::Upgraded => Self::ProtocolEnded,
            CloseReason::IncompleteH1Exchange => Self::IncompleteH1Exchange,
            CloseReason::Reclaimed => Self::Reclaimed,
            CloseReason::PoolDropped => Self::PoolDropped,
            CloseReason::OwnerRuntimeShutdown => Self::OwnerRuntimeShutdown,
        }
    }
}

/// Observation that an installed connection stopped accepting new dispatch.
///
/// Accepted HTTP/2 streams, an HTTP/1 exchange, or root transport I/O may
/// remain live after this event.
#[derive(Debug)]
pub struct ConnectionLogicalClose<'a> {
    connection: &'a PoolArc<ConnectionInfo>,
    cause: LogicalCloseCause,
}

impl<'a> ConnectionLogicalClose<'a> {
    /// Returns immutable identity and transport facts for the connection.
    pub fn connection(&self) -> &'a PoolArc<ConnectionInfo> {
        self.connection
    }

    /// Returns the first cause that ended pool dispatch ownership.
    pub fn cause(&self) -> LogicalCloseCause {
        self.cause
    }
}

/// Observation that the client released root transport ownership.
///
/// This is the final event for an installed connection. It does not assert
/// that peer, kernel, or TCP teardown has completed.
#[derive(Debug)]
pub struct ConnectionPhysicalClose<'a> {
    connection: &'a PoolArc<ConnectionInfo>,
    reason: CloseReason,
}

impl<'a> ConnectionPhysicalClose<'a> {
    /// Returns immutable identity and transport facts for the connection.
    pub fn connection(&self) -> &'a PoolArc<ConnectionInfo> {
        self.connection
    }

    /// Returns the final protocol and ownership close classification.
    pub fn reason(&self) -> CloseReason {
        self.reason
    }
}

/// Pool-owned connection event source.
///
/// This owner retains callback delivery, establishment timing, and public
/// establishment identity allocation. Timing is collected for successful
/// connection metadata even when callback delivery is disabled.
#[derive(Debug)]
pub(super) struct ConnectionEvents {
    listener: Option<SharedConnectionEventListener>,
    time_source: SharedTimeSource,
    next_establishment_id: AtomicU64,
}

impl ConnectionEvents {
    /// Creates the event source retained by one connection pool.
    pub(super) fn new(
        listener: Option<SharedConnectionEventListener>,
        time_source: SharedTimeSource,
    ) -> Self {
        Self {
            listener,
            time_source,
            next_establishment_id: AtomicU64::new(0),
        }
    }

    /// Records the start of one connection establishment.
    pub(super) fn establishment_started(
        &self,
        origin: &OriginKey,
        partition: PartitionId,
        connection_stats: PoolArc<CellConnectionStats>,
    ) -> ConnectionEstablishment {
        connection_stats.establishment_started();
        let timing = EstablishmentTiming {
            time_source: self.time_source.clone(),
            started_at: self.time_source.now(),
            protocol_handshake_started_at: None,
            transport_duration: None,
            protocol_handshake_duration: None,
        };
        let observation = self
            .listener
            .as_ref()
            .map(|listener| EstablishmentObservation {
                listener: listener.clone(),
                info: ConnectionEstablishmentInfo {
                    id: ConnectionEstablishmentId(
                        self.next_establishment_id.fetch_add(1, Ordering::Relaxed),
                    ),
                    origin: origin.clone(),
                    partition,
                },
                stage: ConnectionEstablishmentStage::Transport,
                remote_addr: None,
                protocol: None,
            });
        ConnectionEstablishment {
            timing,
            observation,
            successful_stats: None,
            connection_stats: Some(connection_stats),
        }
    }
}

/// Tracks measurements and optional callback state for one establishment.
pub(super) struct ConnectionEstablishment {
    timing: EstablishmentTiming,
    observation: Option<EstablishmentObservation>,
    successful_stats: Option<ConnectionEstablishmentStats>,
    connection_stats: Option<PoolArc<CellConnectionStats>>,
}

/// Timing collected for every connection establishment.
struct EstablishmentTiming {
    time_source: SharedTimeSource,
    started_at: SystemTime,
    protocol_handshake_started_at: Option<SystemTime>,
    transport_duration: Option<Duration>,
    protocol_handshake_duration: Option<Duration>,
}

/// Callback state retained only when a listener is configured.
struct EstablishmentObservation {
    listener: SharedConnectionEventListener,
    info: ConnectionEstablishmentInfo,
    stage: ConnectionEstablishmentStage,
    remote_addr: Option<SocketAddr>,
    protocol: Option<ConnectionProtocol>,
}

impl ConnectionEstablishment {
    /// Records completion of the transport stage.
    pub(super) fn transport_completed(&mut self, remote_addr: Option<SocketAddr>) {
        self.timing.transport_duration = Some(self.timing.elapsed_since(self.timing.started_at));
        if let Some(observation) = &mut self.observation {
            observation.remote_addr = remote_addr;
            observation.stage = ConnectionEstablishmentStage::ProtocolSelection;
        }
    }

    /// Records the protocol selected from connector metadata.
    pub(super) fn protocol_selected(&mut self, protocol: ConnectionProtocol) {
        if let Some(observation) = &mut self.observation {
            observation.protocol = Some(protocol);
        }
    }

    /// Records entry into Hyper's protocol handshake.
    pub(super) fn protocol_handshake_started(&mut self) {
        self.timing.protocol_handshake_started_at = Some(self.timing.time_source.now());
        if let Some(observation) = &mut self.observation {
            observation.stage = ConnectionEstablishmentStage::ProtocolHandshake;
        }
    }

    /// Records a failed Hyper handshake before emitting its terminal event.
    pub(super) fn protocol_handshake_failed(&mut self) {
        self.timing.finish_protocol_handshake();
    }

    /// Records successful Hyper handshake before pool installation.
    pub(super) fn protocol_handshake_completed(&mut self) {
        self.timing.finish_protocol_handshake();
        if let Some(observation) = &mut self.observation {
            observation.stage = ConnectionEstablishmentStage::PoolInstallation;
        }
    }

    /// Freezes successful measurements before the connection becomes visible.
    pub(super) fn installed(&mut self, connection: &PoolArc<ConnectionState>) {
        let stats = self.timing.stats();
        connection
            .info()
            .set_establishment(stats.connection_metadata());
        assert!(
            self.successful_stats.replace(stats).is_none(),
            "connection establishment was installed more than once"
        );
    }

    /// Emits the terminal failure for this establishment.
    pub(super) fn failed(mut self, error: &ConnectorError) {
        let stats = self.timing.stats();
        self.finish_connection_stats();
        let Some(observation) = self.observation.take() else {
            return;
        };
        observation.notify_failure(error, &stats);
    }

    /// Reports successful installation and enables ordered close observations.
    pub(super) fn opened(mut self, connection: &PoolArc<ConnectionState>) {
        let stats = self.successful_stats.take().unwrap_or_else(|| {
            let stats = self.timing.stats();
            connection
                .info()
                .set_establishment(stats.connection_metadata());
            stats
        });
        self.finish_connection_stats();
        let listener = self.observation.take().map(|observation| {
            observation
                .listener
                .notify(&ConnectionEvent::Opened(ConnectionOpened {
                    establishment: &observation.info,
                    stats: &stats,
                    connection: connection.info(),
                }));
            observation.listener
        });
        connection.complete_opened_event(listener.as_ref());
    }

    /// Ends an establishment whose transport lost to existing HTTP/2 supply.
    pub(super) fn superseded(mut self) {
        self.finish_connection_stats();
        self.observation.take();
    }

    fn finish_connection_stats(&mut self) {
        if let Some(stats) = self.connection_stats.take() {
            stats.establishment_finished();
        }
    }
}

impl Drop for ConnectionEstablishment {
    fn drop(&mut self) {
        let stats = self.successful_stats.unwrap_or_else(|| self.timing.stats());
        self.finish_connection_stats();
        let Some(observation) = self.observation.take() else {
            return;
        };
        let error = ConnectorError::io("connection establishment task was dropped".into());
        observation.notify_failure(&error, &stats);
    }
}

impl EstablishmentObservation {
    fn notify_failure(&self, error: &ConnectorError, stats: &ConnectionEstablishmentStats) {
        self.listener.notify(&ConnectionEvent::EstablishmentFailed(
            ConnectionEstablishmentFailed {
                establishment: &self.info,
                stats,
                stage: self.stage,
                remote_addr: self.remote_addr,
                protocol: self.protocol,
                error,
            },
        ));
    }
}

impl EstablishmentTiming {
    fn stats(&self) -> ConnectionEstablishmentStats {
        let total_duration = self.elapsed_since(self.started_at);
        ConnectionEstablishmentStats {
            total_duration,
            transport_duration: self.transport_duration.unwrap_or(total_duration),
            protocol_handshake_duration: self.protocol_handshake_duration.or_else(|| {
                self.protocol_handshake_started_at
                    .map(|started_at| self.elapsed_since(started_at))
            }),
        }
    }

    fn finish_protocol_handshake(&mut self) {
        self.protocol_handshake_duration = self
            .protocol_handshake_started_at
            .take()
            .map(|started_at| self.elapsed_since(started_at));
    }

    fn elapsed_since(&self, started_at: SystemTime) -> Duration {
        match self.time_source.now().duration_since(started_at) {
            Ok(duration) => duration,
            Err(error) => {
                tracing::warn!(?error, "connection establishment clock moved backwards");
                Duration::ZERO
            }
        }
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use aws_smithy_async::{test_util::ManualTimeSource, time::StaticTimeSource};
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;
    use std::time::UNIX_EPOCH;

    fn origin() -> OriginKey {
        OriginKey::from_parts(http_1x::uri::Scheme::HTTPS, "example.com", None).unwrap()
    }

    fn events(listener: Option<SharedConnectionEventListener>) -> ConnectionEvents {
        ConnectionEvents::new(listener, SharedTimeSource::default())
    }

    fn establishment(events: &ConnectionEvents) -> ConnectionEstablishment {
        events.establishment_started(
            &origin(),
            PartitionId::from_index(1),
            PoolArc::new(CellConnectionStats::default()),
        )
    }

    #[test]
    fn disabled_callbacks_do_not_allocate_establishment_ids() {
        let events = events(None);
        events.next_establishment_id.store(7, Ordering::Relaxed);

        establishment(&events).superseded();

        assert_eq!(7, events.next_establishment_id.load(Ordering::Relaxed));
    }

    #[test]
    fn listener_panic_is_isolated() {
        let events = events(Some(SharedConnectionEventListener::new(
            |_: &ConnectionEvent<'_>| panic!("listener failed"),
        )));
        let error = ConnectorError::io("synthetic transport failure".into());

        establishment(&events).failed(&error);
    }

    #[test]
    fn dropping_active_establishment_emits_transport_failure() {
        let observed = Arc::new(Mutex::new(None));
        let events = events(Some(SharedConnectionEventListener::new({
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                let ConnectionEvent::EstablishmentFailed(failed) = event else {
                    panic!("unexpected event: {event:?}");
                };
                *observed.lock().unwrap() = Some((failed.establishment().id(), failed.stage()));
            }
        })));

        drop(establishment(&events));

        assert_eq!(
            Some((
                ConnectionEstablishmentId(0),
                ConnectionEstablishmentStage::Transport
            )),
            *observed.lock().unwrap()
        );
    }

    #[test]
    fn failed_callback_observes_finished_establishment_count() {
        let stats = PoolArc::new(CellConnectionStats::default());
        let observed = Arc::new(Mutex::new(None));
        let events = events(Some(SharedConnectionEventListener::new({
            let stats = stats.clone();
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                let ConnectionEvent::EstablishmentFailed(_) = event else {
                    panic!("unexpected event: {event:?}");
                };
                *observed.lock().unwrap() =
                    Some(stats.snapshot(0, 0, 0, 0, 0).establishing_connections());
            }
        })));
        let establishment =
            events.establishment_started(&origin(), PartitionId::from_index(1), stats.clone());
        assert_eq!(1, stats.snapshot(0, 0, 0, 0, 0).establishing_connections());
        let error = ConnectorError::io("synthetic transport failure".into());

        establishment.failed(&error);

        assert_eq!(Some(0), *observed.lock().unwrap());
    }

    #[test]
    fn failed_event_carries_recorded_stage_and_protocol() {
        let observed = Arc::new(Mutex::new(None));
        let events = events(Some(SharedConnectionEventListener::new({
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                let ConnectionEvent::EstablishmentFailed(failed) = event else {
                    panic!("unexpected event: {event:?}");
                };
                *observed.lock().unwrap() = Some((
                    failed.establishment().id(),
                    failed.stage(),
                    failed.protocol(),
                    failed.stats().protocol_handshake_duration().is_some(),
                ));
            }
        })));
        let mut establishment = establishment(&events);
        establishment.transport_completed(None);
        establishment.protocol_selected(ConnectionProtocol::Http2);
        establishment.protocol_handshake_started();
        establishment.protocol_handshake_failed();
        let error = ConnectorError::io("synthetic handshake failure".into());

        establishment.failed(&error);

        assert_eq!(
            Some((
                ConnectionEstablishmentId(0),
                ConnectionEstablishmentStage::ProtocolHandshake,
                Some(ConnectionProtocol::Http2),
                true,
            )),
            *observed.lock().unwrap()
        );
    }

    #[test]
    fn establishment_stats_measure_recorded_phases() {
        let time = ManualTimeSource::new(UNIX_EPOCH);
        let observed = Arc::new(Mutex::new(None));
        let events = ConnectionEvents::new(
            Some(SharedConnectionEventListener::new({
                let observed = observed.clone();
                move |event: &ConnectionEvent<'_>| {
                    let ConnectionEvent::EstablishmentFailed(failed) = event else {
                        panic!("unexpected event: {event:?}");
                    };
                    *observed.lock().unwrap() = Some(*failed.stats());
                }
            })),
            SharedTimeSource::new(time.clone()),
        );
        let mut establishment = establishment(&events);

        time.advance(Duration::from_secs(2));
        establishment.transport_completed(None);
        establishment.protocol_selected(ConnectionProtocol::Http2);
        establishment.protocol_handshake_started();
        time.advance(Duration::from_secs(3));
        establishment.protocol_handshake_completed();
        time.advance(Duration::from_secs(2));
        establishment.failed(&ConnectorError::io("synthetic installation failure".into()));

        let stats = observed.lock().unwrap().expect("establishment stats");
        assert_eq!(stats.total_duration(), Duration::from_secs(7));
        assert_eq!(stats.transport_duration(), Duration::from_secs(2));
        assert_eq!(
            stats.protocol_handshake_duration(),
            Some(Duration::from_secs(3))
        );
        let metadata = stats.connection_metadata();
        assert_eq!(metadata.total_duration(), Duration::from_secs(7));
        assert_eq!(metadata.transport_duration(), Duration::from_secs(2));
        assert_eq!(
            metadata.protocol_handshake_duration(),
            Some(Duration::from_secs(3))
        );
    }

    #[test]
    fn backwards_clock_saturates_establishment_durations() {
        let timing = EstablishmentTiming {
            time_source: StaticTimeSource::new(UNIX_EPOCH).into(),
            started_at: UNIX_EPOCH + Duration::from_secs(1),
            protocol_handshake_started_at: Some(UNIX_EPOCH + Duration::from_secs(1)),
            transport_duration: None,
            protocol_handshake_duration: None,
        };

        let stats = timing.stats();

        assert_eq!(stats.total_duration(), Duration::ZERO);
        assert_eq!(stats.transport_duration(), Duration::ZERO);
        assert_eq!(stats.protocol_handshake_duration(), Some(Duration::ZERO));
    }

    #[test]
    fn superseded_establishment_emits_no_public_event() {
        let observed = Arc::new(AtomicUsize::new(0));
        let events = events(Some(SharedConnectionEventListener::new({
            let observed = observed.clone();
            move |_: &ConnectionEvent<'_>| {
                observed.fetch_add(1, Ordering::Relaxed);
            }
        })));

        establishment(&events).superseded();

        assert_eq!(0, observed.load(Ordering::Relaxed));
        assert_eq!(1, events.next_establishment_id.load(Ordering::Relaxed));
    }

    #[test]
    fn closure_listener_receives_one_terminal_event() {
        let observed = Arc::new(AtomicUsize::new(0));
        let events = events(Some(SharedConnectionEventListener::new({
            let observed = observed.clone();
            move |event: &ConnectionEvent<'_>| {
                assert!(matches!(event, ConnectionEvent::EstablishmentFailed(_)));
                observed.fetch_add(1, Ordering::Relaxed);
            }
        })));
        let error = ConnectorError::io("synthetic transport failure".into());

        establishment(&events).failed(&error);

        assert_eq!(1, observed.load(Ordering::Relaxed));
    }

    #[test]
    fn logical_close_causes_cover_every_close_reason() {
        assert_eq!(
            [
                LogicalCloseCause::IdleTimeout,
                LogicalCloseCause::Poisoned,
                LogicalCloseCause::ProtocolEnded,
                LogicalCloseCause::ProtocolEnded,
                LogicalCloseCause::IncompleteH1Exchange,
                LogicalCloseCause::Reclaimed,
                LogicalCloseCause::PoolDropped,
                LogicalCloseCause::OwnerRuntimeShutdown,
            ],
            [
                CloseReason::IdleTimeout,
                CloseReason::Poisoned,
                CloseReason::ProtocolClosed,
                CloseReason::Upgraded,
                CloseReason::IncompleteH1Exchange,
                CloseReason::Reclaimed,
                CloseReason::PoolDropped,
                CloseReason::OwnerRuntimeShutdown,
            ]
            .map(LogicalCloseCause::from_reason)
        );
    }
}
