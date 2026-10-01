/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Transport connection policy below HTTP protocol establishment.
//!
//! This module owns the boundary from a pool connection attempt to connected
//! I/O with a negotiated protocol. [`TransportConnectContext`] keeps placement,
//! timeout, and ALPN inputs together. A factory may delegate DNS, proxy, TLS,
//! and ALPN to an injected connector, but it must report whether an HTTP/1-only
//! attempt is enforceable before admission may reclaim an idle H2 connection.
//!
//! Cached factories retain configured connector services, not connections or
//! DNS results. Base client validation initializes those services for the
//! selected partition without network I/O. Network I/O begins only when
//! [`TransportFactory::connect`] is called.

use super::super::admission::ProtocolRequirement;
use super::super::registry::PartitionState;
use crate::client::connect::{AsyncConn, BoxConn};
use crate::client::timeout::{self, TimeoutKind};
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_runtime_api::box_error::BoxError;
use http_1x::Uri;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::Arc as StdArc;
use std::time::Duration;
use tower::Service;

/// Timeout policy for one transport connection operation.
///
/// Connector readiness completes before this timeout starts. Keeping the
/// duration and timer together lets request acquisition pass one complete
/// timeout value into transport construction.
#[derive(Clone, Debug)]
pub(in crate::client::pool) struct TransportTimeout {
    /// Maximum duration of the connector's connection future.
    duration: Duration,
    /// Runtime timer used to enforce the deadline.
    sleep: SharedAsyncSleep,
}

impl TransportTimeout {
    /// Creates a timeout enforced by the supplied runtime timer.
    pub(in crate::client::pool) fn new(duration: Duration, sleep: SharedAsyncSleep) -> Self {
        Self { duration, sleep }
    }
}

/// Static ALPN protocol offer used for one transport connection.
pub(in crate::client::pool) type AlpnProtocols = &'static [&'static [u8]];

/// Default offer for requests that may use HTTP/2.
const HTTP_ALPN_PROTOCOLS: AlpnProtocols = &[b"h2", b"http/1.1"];

/// Narrowed offer for requests that require HTTP/1 wire semantics.
const HTTP1_ALPN_PROTOCOLS: AlpnProtocols = &[b"http/1.1"];

/// Inputs needed to create one transport connection.
///
/// This context is the transport-attempt boundary for DNS, socket, proxy, TLS,
/// and ALPN lifecycle observation. It carries no installed connection state
/// because an attempt may fail before a connection exists.
pub(in crate::client::pool) struct TransportConnectContext<'a> {
    /// Partition whose placement policy owns the connection attempt.
    partition: &'a PartitionState,
    /// Absolute URI passed to the connector contract.
    uri: Uri,
    /// Optional deadline for the connector's connection future.
    timeout: Option<TransportTimeout>,
    /// Protocols the default TLS connector may advertise for this attempt.
    alpn_protocols: AlpnProtocols,
}

impl<'a> TransportConnectContext<'a> {
    /// Creates transport inputs after request protocol classification.
    pub(super) fn new(
        partition: &'a PartitionState,
        uri: Uri,
        timeout: Option<TransportTimeout>,
        requirement: ProtocolRequirement,
    ) -> Self {
        Self {
            partition,
            uri,
            timeout,
            alpn_protocols: alpn_protocols(requirement),
        }
    }
}

/// Future returned by a type-erased transport factory.
type TransportFuture = Pin<Box<dyn Future<Output = Result<BoxConn, BoxError>> + Send + 'static>>;

/// Type-erased transport construction below HTTP protocol establishment.
///
/// A pool retains one factory. The concrete connector remains responsible for
/// the transport stages it implements and for returning connector metadata.
/// Protocol establishment consumes the resulting [`BoxConn`].
pub(in crate::client::pool) trait TransportFactory:
    Send + Sync + 'static
{
    /// Initializes connector services used by one partition.
    ///
    /// Initialization must be idempotent and must not perform DNS resolution
    /// or network I/O. Factories without retained initialization state may use
    /// the default no-op implementation.
    fn initialize_for_partition(&self, partition: &PartitionState) -> Result<(), BoxError> {
        let _ = partition;
        Ok(())
    }

    /// Returns whether an H1-required attempt is guaranteed to negotiate H1.
    fn can_guarantee_http1(&self) -> bool;

    /// Creates one partition-bound transport.
    ///
    /// Connector readiness completes before the connection timeout starts.
    fn connect(&self, context: TransportConnectContext<'_>) -> TransportFuture;
}

/// Connector factory whose concrete service is selected by interface.
struct ServiceTransportFactory<F> {
    /// Builds a connector with the selected network-interface binding.
    connector_for_interface: F,
    /// Whether every H1-required connection is guaranteed to negotiate H1.
    can_guarantee_http1: bool,
}

impl<F, C, IO> TransportFactory for ServiceTransportFactory<F>
where
    F: Fn(Option<&str>) -> C + Send + Sync + 'static,
    C: Service<Uri, Response = IO> + Send + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    fn can_guarantee_http1(&self) -> bool {
        self.can_guarantee_http1
    }

    fn connect(&self, context: TransportConnectContext<'_>) -> TransportFuture {
        let TransportConnectContext {
            partition,
            uri,
            timeout,
            alpn_protocols: _alpn_protocols,
        } = context;
        let interface = partition.interface().map(|interface| interface.as_ref());
        let mut connector = (self.connector_for_interface)(interface);
        Box::pin(async move {
            // Readiness follows the existing connector contract and is not timed.
            poll_fn(|cx| connector.poll_ready(cx))
                .await
                .map_err(Into::into)?;
            let connect = connector.call(uri);
            let io = timeout::maybe_timeout_future(
                connect,
                timeout.as_ref().map(|timeout| timeout.duration),
                timeout.as_ref().map(|timeout| &timeout.sleep),
                TimeoutKind::Connect,
            )
            .await?;
            Ok(Box::new(io) as BoxConn)
        })
    }
}

/// Erases an injected connector that owns placement and protocol negotiation.
///
/// The pool cannot apply its interface or ALPN inputs to this connector. It is
/// therefore conservative about H2 reclaim for H1-required demand.
#[cfg(any(
    all(feature = "test-util", aws_sdk_unstable),
    all(test, feature = "rt-tokio")
))]
pub(in crate::client::pool) fn from_connector<C, IO>(connector: C) -> StdArc<dyn TransportFactory>
where
    C: Service<Uri, Response = IO> + Clone + Send + Sync + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    service_factory(move |_| connector.clone(), false)
}

/// Erases a cleartext connector constructor that applies interface placement.
pub(in crate::client::pool) fn from_interface_connector<F, C, IO>(
    connector_for_interface: F,
) -> StdArc<dyn TransportFactory>
where
    F: Fn(Option<&str>) -> C + Send + Sync + 'static,
    C: Service<Uri, Response = IO> + Send + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    service_factory(connector_for_interface, true)
}

/// Erases a service connector and its HTTP/1 negotiation guarantee.
fn service_factory<F, C, IO>(
    connector_for_interface: F,
    can_guarantee_http1: bool,
) -> StdArc<dyn TransportFactory>
where
    F: Fn(Option<&str>) -> C + Send + Sync + 'static,
    C: Service<Uri, Response = IO> + Send + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    StdArc::new(ServiceTransportFactory {
        connector_for_interface,
        can_guarantee_http1,
    })
}

/// Identifies one configured connector service in the TLS transport cache.
///
/// Partition identity is deliberately absent. Partitions with equal interface
/// bindings use the same connector configuration and therefore share entries.
#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct ConnectorCacheKey {
    /// Network-interface binding applied before connect.
    interface: Option<StdArc<str>>,
    /// Protocol offer installed in the TLS connector.
    alpn_protocols: AlpnProtocols,
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
impl ConnectorCacheKey {
    /// Derives the connector configuration used by one partition and offer.
    fn for_partition(partition: &PartitionState, alpn_protocols: AlpnProtocols) -> Self {
        Self {
            interface: partition.interface().cloned(),
            alpn_protocols,
        }
    }

    /// Returns the borrowed interface name expected by connector builders.
    fn interface(&self) -> Option<&str> {
        self.interface.as_deref()
    }
}

/// Retains configured TLS connector services by placement and ALPN offer.
#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
struct CachedTransportFactory<F, C> {
    /// Constructs a connector service for one exact cache key.
    factory: F,
    /// Whether every H1-required connection is guaranteed to negotiate H1.
    can_guarantee_http1: bool,
    /// Connector services constructed during client validation or first use.
    connectors: crate::sync::Mutex<std::collections::HashMap<ConnectorCacheKey, C>>,
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
impl<F, C> CachedTransportFactory<F, C>
where
    F: Fn(Option<&str>, AlpnProtocols) -> C,
    C: Clone,
{
    /// Returns the retained connector for one partition and protocol offer.
    fn connector(&self, partition: &PartitionState, alpn_protocols: AlpnProtocols) -> C {
        let key = ConnectorCacheKey::for_partition(partition, alpn_protocols);
        match self.connectors.lock().entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.get().clone(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let connector = (self.factory)(entry.key().interface(), entry.key().alpn_protocols);
                entry.insert(connector).clone()
            }
        }
    }

    /// Materializes every connector variant this partition may select.
    fn initialize_connectors_for_partition(&self, partition: &PartitionState) {
        drop(self.connector(partition, HTTP_ALPN_PROTOCOLS));
        drop(self.connector(partition, HTTP1_ALPN_PROTOCOLS));
    }
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
impl<F, C, IO> TransportFactory for CachedTransportFactory<F, C>
where
    F: Fn(Option<&str>, AlpnProtocols) -> C + Send + Sync + 'static,
    C: Service<Uri, Response = IO> + Clone + Send + Sync + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    fn initialize_for_partition(&self, partition: &PartitionState) -> Result<(), BoxError> {
        self.initialize_connectors_for_partition(partition);
        Ok(())
    }

    fn can_guarantee_http1(&self) -> bool {
        self.can_guarantee_http1
    }

    fn connect(&self, context: TransportConnectContext<'_>) -> TransportFuture {
        let TransportConnectContext {
            partition,
            uri,
            timeout,
            alpn_protocols,
        } = context;
        let mut connector = self.connector(partition, alpn_protocols);
        Box::pin(async move {
            poll_fn(|cx| connector.poll_ready(cx))
                .await
                .map_err(Into::into)?;
            let connect = connector.call(uri);
            let io = timeout::maybe_timeout_future(
                connect,
                timeout.as_ref().map(|timeout| timeout.duration),
                timeout.as_ref().map(|timeout| &timeout.sleep),
                TimeoutKind::Connect,
            )
            .await?;
            Ok(Box::new(io) as BoxConn)
        })
    }
}

/// Erases and caches configured TLS connector services.
///
/// Interface placement is immutable for a partition. Client validation
/// constructs both ALPN variants for the selected partition while holding the
/// cache lock; later requests clone the retained connector. Provider
/// configuration and certificate loading therefore occur at most once for
/// each placement and offer rather than once per connection.
#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
pub(in crate::client::pool) fn from_cached_interface_connector<F, C, IO>(
    connector_for_interface: F,
    can_guarantee_http1: bool,
) -> StdArc<dyn TransportFactory>
where
    F: Fn(Option<&str>, AlpnProtocols) -> C + Send + Sync + 'static,
    C: Service<Uri, Response = IO> + Clone + Send + Sync + 'static,
    C::Error: Into<BoxError>,
    C::Future: Send + 'static,
    IO: AsyncConn,
{
    StdArc::new(CachedTransportFactory {
        factory: connector_for_interface,
        can_guarantee_http1,
        connectors: crate::sync::Mutex::new(std::collections::HashMap::new()),
    })
}

/// Selects the ALPN offer that preserves the request's wire semantics.
fn alpn_protocols(requirement: ProtocolRequirement) -> AlpnProtocols {
    match requirement {
        ProtocolRequirement::H1Required => HTTP1_ALPN_PROTOCOLS,
        ProtocolRequirement::H1Compatible | ProtocolRequirement::H2Required => HTTP_ALPN_PROTOCOLS,
    }
}

#[cfg(all(test, any(feature = "__rustls", feature = "s2n-tls")))]
mod tests {
    use super::*;
    use crate::client::pool::maintenance::MaintenanceConfig;
    use crate::client::pool::partition::{
        ConnectionReuseScope, DriverSpawner, Partition, PartitionId, Spawn,
    };
    use crate::client::pool::registry::PartitionRegistry;
    use aws_smithy_runtime_api::client::http::HttpClient;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use aws_smithy_types::config_bag::ConfigBag;

    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct TestSpawner;

    impl Spawn for TestSpawner {
        fn spawn(&self, _: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {}
    }

    struct InitializationRecorder {
        partitions: Arc<Mutex<Vec<PartitionId>>>,
    }

    impl TransportFactory for InitializationRecorder {
        fn initialize_for_partition(&self, partition: &PartitionState) -> Result<(), BoxError> {
            self.partitions
                .lock()
                .expect("initialization log is not poisoned")
                .push(partition.id());
            Ok(())
        }

        fn can_guarantee_http1(&self) -> bool {
            true
        }

        fn connect(&self, _: TransportConnectContext<'_>) -> TransportFuture {
            Box::pin(async { panic!("client validation must not start a connection") })
        }
    }

    type Construction = (Option<String>, AlpnProtocols);

    struct CachedFactoryFixture<F> {
        factory: CachedTransportFactory<F, usize>,
        constructions: Arc<Mutex<Vec<Construction>>>,
    }

    fn recording_factory() -> CachedFactoryFixture<impl Fn(Option<&str>, AlpnProtocols) -> usize> {
        let constructions = Arc::new(Mutex::new(Vec::new()));
        let observed = constructions.clone();
        let factory = move |interface: Option<&str>, alpn_protocols: AlpnProtocols| {
            let mut constructions = observed.lock().expect("construction log is not poisoned");
            constructions.push((interface.map(str::to_owned), alpn_protocols));
            constructions.len()
        };
        CachedFactoryFixture {
            factory: CachedTransportFactory {
                factory,
                can_guarantee_http1: true,
                connectors: crate::sync::Mutex::new(std::collections::HashMap::new()),
            },
            constructions,
        }
    }

    fn registry(partitions: Option<Vec<Partition>>) -> PartitionRegistry {
        PartitionRegistry::new(
            partitions,
            ConnectionReuseScope::Partition,
            None,
            MaintenanceConfig::default(),
        )
        .expect("valid partition registry")
    }

    #[test]
    fn client_validation_initializes_its_selected_partition() {
        let selected = PartitionId::from_index(7);
        let initialized = Arc::new(Mutex::new(Vec::new()));
        let pool = crate::client::pool::builder::Builder::default()
            .partitions([Partition::new(selected, DriverSpawner::new(TestSpawner))])
            .build_with_transport_for_test(Arc::new(InitializationRecorder {
                partitions: initialized.clone(),
            }))
            .expect("valid test pool");
        let client = crate::client::pool::Client::from_partition(&pool, selected)
            .expect("selected partition exists");

        client
            .validate_base_client_config(&RuntimeComponentsBuilder::for_tests(), &ConfigBag::base())
            .expect("transport initialization succeeds");

        assert_eq!(
            vec![selected],
            *initialized
                .lock()
                .expect("initialization log is not poisoned")
        );
    }

    #[test]
    fn initialization_constructs_each_alpn_variant_once() {
        let partition = registry(None)
            .partition(PartitionId::ANONYMOUS)
            .expect("anonymous partition exists");
        let fixture = recording_factory();

        fixture
            .factory
            .initialize_connectors_for_partition(&partition);
        fixture
            .factory
            .initialize_connectors_for_partition(&partition);

        let constructions = fixture
            .constructions
            .lock()
            .expect("construction log is not poisoned");
        assert_eq!(2, constructions.len());
        assert_eq!(None, constructions[0].0);
        assert_eq!(HTTP_ALPN_PROTOCOLS, constructions[0].1);
        assert_eq!(HTTP1_ALPN_PROTOCOLS, constructions[1].1);
        assert_eq!(2, fixture.factory.connectors.lock().len());
    }

    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "solaris",
        target_os = "tvos",
        target_os = "visionos",
        target_os = "watchos",
    ))]
    #[test]
    fn equal_interface_bindings_share_connector_entries() {
        let first = PartitionId::from_index(1);
        let second = PartitionId::from_index(2);
        let registry = registry(Some(vec![
            Partition::new(first, DriverSpawner::new(TestSpawner)).interface("interface-a"),
            Partition::new(second, DriverSpawner::new(TestSpawner)).interface("interface-a"),
        ]));
        let fixture = recording_factory();

        fixture.factory.initialize_connectors_for_partition(
            &registry.partition(first).expect("first partition exists"),
        );
        fixture.factory.initialize_connectors_for_partition(
            &registry.partition(second).expect("second partition exists"),
        );

        let constructions = fixture
            .constructions
            .lock()
            .expect("construction log is not poisoned");
        assert_eq!(
            [
                (Some("interface-a".to_string()), HTTP_ALPN_PROTOCOLS),
                (Some("interface-a".to_string()), HTTP1_ALPN_PROTOCOLS),
            ],
            constructions.as_slice()
        );
        assert_eq!(2, fixture.factory.connectors.lock().len());
    }

    #[cfg(any(
        target_os = "android",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "solaris",
        target_os = "tvos",
        target_os = "visionos",
        target_os = "watchos",
    ))]
    #[test]
    fn distinct_interface_bindings_use_distinct_connector_entries() {
        let first = PartitionId::from_index(1);
        let second = PartitionId::from_index(2);
        let registry = registry(Some(vec![
            Partition::new(first, DriverSpawner::new(TestSpawner)).interface("interface-a"),
            Partition::new(second, DriverSpawner::new(TestSpawner)).interface("interface-b"),
        ]));
        let fixture = recording_factory();

        fixture.factory.initialize_connectors_for_partition(
            &registry.partition(first).expect("first partition exists"),
        );
        fixture.factory.initialize_connectors_for_partition(
            &registry.partition(second).expect("second partition exists"),
        );

        let constructions = fixture
            .constructions
            .lock()
            .expect("construction log is not poisoned");
        assert_eq!(
            [
                (Some("interface-a".to_string()), HTTP_ALPN_PROTOCOLS),
                (Some("interface-a".to_string()), HTTP1_ALPN_PROTOCOLS),
                (Some("interface-b".to_string()), HTTP_ALPN_PROTOCOLS),
                (Some("interface-b".to_string()), HTTP1_ALPN_PROTOCOLS),
            ],
            constructions.as_slice()
        );
        assert_eq!(4, fixture.factory.connectors.lock().len());
    }
}
