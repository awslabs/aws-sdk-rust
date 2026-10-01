/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Connection placement declarations.
//!
//! A [`Partition`] combines stable identity, driver placement, and an optional
//! network-interface binding. It declares where connections live; sharing a
//! connection never moves that connection's I/O or driver.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Declares where a client's connections are established and driven.
#[derive(Clone)]
pub struct Partition {
    /// Stable identity used to resolve clients and index cells.
    id: PartitionId,
    /// Runtime placement for connection-owned tasks.
    spawner: DriverSpawner,
    /// Optional network-interface binding for new connections.
    interface: Option<Arc<str>>,
}

impl Partition {
    /// Creates a partition with a stable identity and driver spawner.
    pub fn new(id: PartitionId, spawner: DriverSpawner) -> Self {
        Self {
            id,
            spawner,
            interface: None,
        }
    }

    /// Binds connections established by this partition to an interface.
    ///
    /// The binding is applied before connect. On Linux, using this setting
    /// sets `SO_BINDTODEVICE` and may require `CAP_NET_RAW` or root
    /// privileges. Apple platforms, illumos, and Solaris use `IP_BOUND_IF`.
    ///
    /// This method is available on Linux and Android, Fuchsia, illumos,
    /// Solaris, macOS, iOS, tvOS, visionOS, and watchOS.
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
    #[cfg_attr(
        docsrs,
        doc(cfg(any(
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
        )))
    )]
    pub fn interface(mut self, interface: impl Into<String>) -> Self {
        self.interface = Some(Arc::from(interface.into()));
        self
    }

    /// Returns the configured network-interface name for validation.
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
    pub(super) fn interface_name(&self) -> Option<&str> {
        self.interface.as_deref()
    }

    /// Returns this partition's declared identity.
    pub(super) fn id(&self) -> PartitionId {
        self.id
    }

    /// Decomposes this declaration for immutable registry storage.
    pub(super) fn into_parts(self) -> (PartitionId, DriverSpawner, Option<Arc<str>>) {
        (self.id, self.spawner, self.interface)
    }
}

impl fmt::Debug for Partition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Partition")
            .field("id", &self.id)
            .field("spawner", &self.spawner)
            .field("interface", &self.interface)
            .finish()
    }
}

/// Identifies one declared connection partition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PartitionId(usize);

impl PartitionId {
    /// Reserved identity for the implicit default partition.
    pub const ANONYMOUS: Self = Self(usize::MAX);

    /// Creates a partition identity from a caller-owned index.
    ///
    /// `usize::MAX` is reserved for [`Self::ANONYMOUS`] and is rejected when
    /// used in an explicit partition declaration.
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Returns whether this is the reserved anonymous partition identity.
    pub const fn is_anonymous(self) -> bool {
        self.0 == Self::ANONYMOUS.0
    }
}

/// Runtime placement for one partition's connection-owned work.
///
/// Protocol drivers, establishment completion, deferred HTTP/1 readiness, and
/// idle maintenance for a partition run through its spawner, even when another
/// partition dispatches on one of its connections. The underlying I/O never
/// moves to the requesting runtime.
///
/// A spawner is placement, not runtime ownership. It does not keep the runtime
/// alive and does not participate in shutdown. When the owning runtime drops
/// spawned work, the pool observes that through the connection's own close
/// path.
///
/// Callers on Tokio use [`DriverSpawner::tokio`] with the handle of the runtime
/// that should own the partition's connections.
#[derive(Clone)]
pub struct DriverSpawner {
    inner: Arc<dyn Spawn>,
}

impl DriverSpawner {
    /// Places connection-owned work on a specific Tokio runtime.
    ///
    /// Work is spawned on `handle` regardless of which thread invokes the
    /// pool. Pass `tokio::runtime::Handle::current()` to use the runtime that
    /// is constructing the partition.
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub fn tokio(handle: tokio::runtime::Handle) -> Self {
        Self::new(TokioSpawn { handle })
    }

    /// Places connection-owned work through a caller-provided spawn function.
    ///
    /// `spawn` receives each connection-owned task and must run it to
    /// completion on the runtime this partition represents. Dropping the task
    /// instead is treated by the pool as that runtime shutting down.
    ///
    /// This is a test hook for observing task placement and is not a supported
    /// extension point.
    #[cfg(feature = "test-util")]
    #[doc(hidden)]
    pub fn from_fn<F>(spawn: F) -> Self
    where
        F: Fn(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send + Sync + 'static,
    {
        Self::new(FnSpawn(spawn))
    }

    /// Wraps one crate-internal placement implementation.
    #[cfg_attr(
        not(any(feature = "rt-tokio", feature = "test-util", test)),
        allow(
            dead_code,
            reason = "no public constructor exists without a runtime feature"
        )
    )]
    pub(in crate::client::pool) fn new(spawn: impl Spawn) -> Self {
        Self {
            inner: Arc::new(spawn),
        }
    }

    /// Spawns connection-owned work on this partition's runtime.
    pub(in crate::client::pool) fn spawn(
        &self,
        driver: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    ) {
        self.inner.spawn(driver);
    }

    /// Returns whether both values name the same underlying spawner.
    #[cfg(test)]
    pub(in crate::client::pool) fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Debug for DriverSpawner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DriverSpawner").field(&self.inner).finish()
    }
}

/// Placement capability behind [`DriverSpawner`].
///
/// This trait is crate-internal so the pool can extend the placement contract
/// without changing the public spawner type.
pub(in crate::client::pool) trait Spawn:
    fmt::Debug + Send + Sync + 'static
{
    /// Spawns connection-owned work on the partition's runtime.
    fn spawn(&self, driver: Pin<Box<dyn Future<Output = ()> + Send + 'static>>);
}

/// Placement on a captured Tokio runtime handle.
#[cfg(feature = "rt-tokio")]
#[derive(Debug)]
struct TokioSpawn {
    /// Runtime that receives connection-owned tasks.
    handle: tokio::runtime::Handle,
}

#[cfg(feature = "rt-tokio")]
impl Spawn for TokioSpawn {
    fn spawn(&self, driver: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        drop(self.handle.spawn(driver));
    }
}

/// Placement through a caller-provided spawn function.
#[cfg(feature = "test-util")]
struct FnSpawn<F>(F);

#[cfg(feature = "test-util")]
impl<F> fmt::Debug for FnSpawn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("FnSpawn")
    }
}

#[cfg(feature = "test-util")]
impl<F> Spawn for FnSpawn<F>
where
    F: Fn(Pin<Box<dyn Future<Output = ()> + Send + 'static>>) + Send + Sync + 'static,
{
    fn spawn(&self, driver: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
        (self.0)(driver);
    }
}

/// Controls which partitions may dispatch through each other's connections.
///
/// The default is [`NetworkInterface`](Self::NetworkInterface): partitions
/// with the same interface binding, including all partitions with no binding,
/// may borrow each other's idle HTTP/1 connections and route requests to each
/// other's HTTP/2 connections. Reuse scope never changes the bounded
/// connection budget, which is shared by every partition for one origin.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionReuseScope {
    /// A connection serves only requests from its owning partition.
    Partition,
    /// Partitions with the same interface binding may share connections.
    #[default]
    NetworkInterface,
    /// Every partition in the pool may share connections.
    Pool,
}

/// Identifies the exact set of partitions allowed to share a connection.
///
/// The configured [`ConnectionReuseScope`] maps each partition to one group:
/// partition-local scope creates a group for one partition, network-interface
/// scope groups equal interface bindings, and pool scope creates one group for
/// the whole pool.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) enum EligibilityGroup {
    /// Only the partition with this identity is eligible.
    Partition(PartitionId),
    /// Partitions with this exact interface binding are eligible.
    NetworkInterface(Option<Arc<str>>),
    /// Every partition in the pool is eligible.
    Pool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "rt-tokio")]
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug)]
    struct TestSpawner;

    impl Spawn for TestSpawner {
        fn spawn(&self, driver: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            drop(driver);
        }
    }

    #[test]
    fn anonymous_identity_is_reserved() {
        assert!(PartitionId::ANONYMOUS.is_anonymous());
        assert!(PartitionId::from_index(usize::MAX).is_anonymous());
        assert!(!PartitionId::from_index(0).is_anonymous());
    }

    #[test]
    fn partition_retains_placement() {
        let spawner = DriverSpawner::new(TestSpawner);
        let partition = Partition::new(PartitionId::from_index(7), spawner.clone());
        let (id, retained, interface) = partition.into_parts();
        assert_eq!(PartitionId::from_index(7), id);
        assert_eq!(None, interface);
        assert!(spawner.ptr_eq(&retained));

        let driver = Box::pin(async {});
        retained.spawn(driver);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn from_fn_receives_each_driver() {
        let submitted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let observed = submitted.clone();
        let spawner = DriverSpawner::from_fn(move |driver| {
            observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            drop(driver);
        });

        spawner.spawn(Box::pin(async {}));
        spawner.spawn(Box::pin(async {}));
        assert_eq!(2, submitted.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!("DriverSpawner(FnSpawn)", format!("{spawner:?}"));
    }

    #[cfg(feature = "rt-tokio")]
    #[test]
    fn tokio_spawner_uses_captured_runtime() {
        let owner = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let owner_id = owner.handle().id();
        let ran = Arc::new(AtomicBool::new(false));
        let task_ran = ran.clone();
        let spawner = DriverSpawner::tokio(owner.handle().clone());

        owner.block_on(async {
            spawner.spawn(Box::pin(async move {
                assert_eq!(owner_id, tokio::runtime::Handle::current().id());
                task_ran.store(true, Ordering::SeqCst);
            }));
            for _ in 0..10 {
                if ran.load(Ordering::SeqCst) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        });
        assert!(ran.load(Ordering::SeqCst));
    }

    #[cfg(feature = "rt-tokio")]
    #[test]
    fn tokio_spawner_accepts_work_from_a_foreign_runtime() {
        let owner = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let foreign = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let owner_id = owner.handle().id();
        let ran = Arc::new(AtomicBool::new(false));
        let task_ran = ran.clone();
        let spawner = DriverSpawner::tokio(owner.handle().clone());

        foreign.block_on(async {
            spawner.spawn(Box::pin(async move {
                assert_eq!(owner_id, tokio::runtime::Handle::current().id());
                task_ran.store(true, Ordering::SeqCst);
            }));
        });
        owner.block_on(async {
            for _ in 0..10 {
                if ran.load(Ordering::SeqCst) {
                    break;
                }
                tokio::task::yield_now().await;
            }
        });
        assert!(ran.load(Ordering::SeqCst));
    }
}
