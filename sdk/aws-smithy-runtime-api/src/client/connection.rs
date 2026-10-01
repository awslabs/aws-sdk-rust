/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types related to connection monitoring and management.

mod establishment;

pub use establishment::{ConnectionEstablishmentMetadata, ConnectionEstablishmentMetadataBuilder};

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Opaque identifier for a physical HTTP connection.
///
/// The assigning HTTP client defines the scope of an ID. IDs from different
/// clients or pools are not comparable.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ConnectionId(u64);

impl ConnectionId {
    /// Creates an ID from a value assigned by an HTTP client.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Metadata that tracks the state of an active connection.
#[derive(Clone)]
pub struct ConnectionMetadata {
    is_proxied: bool,
    remote_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,
    poison_fn: Arc<dyn Fn() + Send + Sync>,
    connection_id: Option<ConnectionId>,
    establishment: Option<ConnectionEstablishmentMetadata>,
}

impl ConnectionMetadata {
    /// Poison this connection, ensuring that it won't be reused.
    pub fn poison(&self) {
        tracing::debug!(
            see_for_more_info = "https://smithy-lang.github.io/smithy-rs/design/client/detailed_error_explanations.html",
            "Connection encountered an issue and should not be re-used. Marking it for closure"
        );
        (self.poison_fn)()
    }

    /// Create a new [`ConnectionMetadata`].
    #[deprecated(
        since = "1.1.0",
        note = "`ConnectionMetadata::new` is deprecated in favour of `ConnectionMetadata::builder`."
    )]
    pub fn new(
        is_proxied: bool,
        remote_addr: Option<SocketAddr>,
        poison: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            is_proxied,
            remote_addr,
            // need to use builder to set this field
            local_addr: None,
            poison_fn: Arc::new(poison),
            connection_id: None,
            establishment: None,
        }
    }

    /// Builder for this connection metadata
    pub fn builder() -> ConnectionMetadataBuilder {
        ConnectionMetadataBuilder::new()
    }

    /// Get the remote address for this connection, if one is set.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Get the local address for this connection, if one is set.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Get the ID assigned to this connection, if the HTTP client provides one.
    ///
    /// Clients that do not track physical connection identity leave this unset.
    pub fn connection_id(&self) -> Option<ConnectionId> {
        self.connection_id
    }

    /// Get measurements from the establishment that installed this connection.
    ///
    /// Clients that do not expose successful establishment measurements leave
    /// this unset.
    pub fn establishment(&self) -> Option<&ConnectionEstablishmentMetadata> {
        self.establishment.as_ref()
    }
}

impl fmt::Debug for ConnectionMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SmithyConnection")
            .field("is_proxied", &self.is_proxied)
            .field("remote_addr", &self.remote_addr)
            .field("local_addr", &self.local_addr)
            .field("connection_id", &self.connection_id)
            .field("establishment", &self.establishment)
            .finish()
    }
}

/// Builder type that is used to construct a [`ConnectionMetadata`] value.
#[derive(Default)]
pub struct ConnectionMetadataBuilder {
    is_proxied: Option<bool>,
    remote_addr: Option<SocketAddr>,
    local_addr: Option<SocketAddr>,
    poison_fn: Option<Arc<dyn Fn() + Send + Sync>>,
    connection_id: Option<ConnectionId>,
    establishment: Option<ConnectionEstablishmentMetadata>,
}

impl fmt::Debug for ConnectionMetadataBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConnectionMetadataBuilder")
            .field("is_proxied", &self.is_proxied)
            .field("remote_addr", &self.remote_addr)
            .field("local_addr", &self.local_addr)
            .field("connection_id", &self.connection_id)
            .field("establishment", &self.establishment)
            .finish()
    }
}

impl ConnectionMetadataBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether or not the associated connection is to an HTTP proxy.
    pub fn proxied(mut self, proxied: bool) -> Self {
        self.set_proxied(Some(proxied));
        self
    }

    /// Set whether or not the associated connection is to an HTTP proxy.
    pub fn set_proxied(&mut self, proxied: Option<bool>) -> &mut Self {
        self.is_proxied = proxied;
        self
    }

    /// Set the remote address of the connection used.
    pub fn remote_addr(mut self, remote_addr: SocketAddr) -> Self {
        self.set_remote_addr(Some(remote_addr));
        self
    }

    /// Set the remote address of the connection used.
    pub fn set_remote_addr(&mut self, remote_addr: Option<SocketAddr>) -> &mut Self {
        self.remote_addr = remote_addr;
        self
    }

    /// Set the local address of the connection used.
    pub fn local_addr(mut self, local_addr: SocketAddr) -> Self {
        self.set_local_addr(Some(local_addr));
        self
    }

    /// Set the local address of the connection used.
    pub fn set_local_addr(&mut self, local_addr: Option<SocketAddr>) -> &mut Self {
        self.local_addr = local_addr;
        self
    }

    /// Set the [`ConnectionId`] assigned by the HTTP client.
    pub fn connection_id(mut self, connection_id: ConnectionId) -> Self {
        self.set_connection_id(Some(connection_id));
        self
    }

    /// Set the [`ConnectionId`] assigned by the HTTP client.
    pub fn set_connection_id(&mut self, connection_id: Option<ConnectionId>) -> &mut Self {
        self.connection_id = connection_id;
        self
    }

    /// Sets measurements from the establishment that installed this connection.
    pub fn establishment(mut self, establishment: ConnectionEstablishmentMetadata) -> Self {
        self.set_establishment(Some(establishment));
        self
    }

    /// Sets measurements from the establishment that installed this connection.
    pub fn set_establishment(
        &mut self,
        establishment: Option<ConnectionEstablishmentMetadata>,
    ) -> &mut Self {
        self.establishment = establishment;
        self
    }

    /// Set a closure which will poison the associated connection.
    ///
    /// A poisoned connection will not be reused for subsequent requests by the pool
    pub fn poison_fn(mut self, poison_fn: impl Fn() + Send + Sync + 'static) -> Self {
        self.set_poison_fn(Some(poison_fn));
        self
    }

    /// Set a closure which will poison the associated connection.
    ///
    /// A poisoned connection will not be reused for subsequent requests by the pool
    pub fn set_poison_fn(
        &mut self,
        poison_fn: Option<impl Fn() + Send + Sync + 'static>,
    ) -> &mut Self {
        self.poison_fn =
            poison_fn.map(|poison_fn| Arc::new(poison_fn) as Arc<dyn Fn() + Send + Sync>);
        self
    }

    /// Build a [`ConnectionMetadata`] value.
    ///
    /// # Panics
    ///
    /// If either the `is_proxied` or `poison_fn` has not been set, then this method will panic
    pub fn build(self) -> ConnectionMetadata {
        ConnectionMetadata {
            is_proxied: self
                .is_proxied
                .expect("is_proxied should be set for ConnectionMetadata"),
            remote_addr: self.remote_addr,
            local_addr: self.local_addr,
            poison_fn: self
                .poison_fn
                .expect("poison_fn should be set for ConnectionMetadata"),
            connection_id: self.connection_id,
            establishment: self.establishment,
        }
    }
}

type LoaderFn = dyn Fn() -> Option<ConnectionMetadata> + Send + Sync;

/// State for a middleware that will monitor and manage connections.
#[derive(Clone, Default)]
pub struct CaptureSmithyConnection {
    loader: Arc<Mutex<Option<Box<LoaderFn>>>>,
}

impl CaptureSmithyConnection {
    /// Create a new connection monitor.
    pub fn new() -> Self {
        Self {
            loader: Default::default(),
        }
    }

    /// Set the retriever that will capture the `hyper` connection.
    pub fn set_connection_retriever<F>(&self, f: F)
    where
        F: Fn() -> Option<ConnectionMetadata> + Send + Sync + 'static,
    {
        *self.loader.lock().unwrap() = Some(Box::new(f));
    }

    /// Get the associated connection metadata.
    pub fn get(&self) -> Option<ConnectionMetadata> {
        match self.loader.lock().unwrap().as_ref() {
            Some(loader) => loader(),
            None => {
                tracing::debug!("no loader was set on the CaptureSmithyConnection");
                None
            }
        }
    }
}

impl fmt::Debug for CaptureSmithyConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CaptureSmithyConnection")
    }
}

impl Storable for CaptureSmithyConnection {
    type Storer = StoreReplace<Self>;
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv6Addr},
        sync::Mutex,
        time::Duration,
    };

    use super::*;

    const TEST_SOCKET_ADDR: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 100);

    #[test]
    #[should_panic]
    fn builder_panic_missing_proxied() {
        ConnectionMetadataBuilder::new()
            .poison_fn(|| {})
            .local_addr(TEST_SOCKET_ADDR)
            .remote_addr(TEST_SOCKET_ADDR)
            .build();
    }

    #[test]
    #[should_panic]
    fn builder_panic_missing_poison_fn() {
        ConnectionMetadataBuilder::new()
            .proxied(true)
            .local_addr(TEST_SOCKET_ADDR)
            .remote_addr(TEST_SOCKET_ADDR)
            .build();
    }

    #[test]
    fn builder_all_fields_successful() {
        let mutable_flag = Arc::new(Mutex::new(false));
        let establishment = ConnectionEstablishmentMetadata::builder()
            .total_duration(Duration::from_millis(13))
            .transport_duration(Duration::from_millis(8))
            .protocol_handshake_duration(Duration::from_millis(5))
            .dns_duration(Duration::from_millis(1))
            .socket_connect_duration(Duration::from_millis(2))
            .proxy_duration(Duration::from_millis(3))
            .tls_duration(Duration::from_millis(4))
            .build();

        let connection_metadata = ConnectionMetadataBuilder::new()
            .proxied(true)
            .local_addr(TEST_SOCKET_ADDR)
            .remote_addr(TEST_SOCKET_ADDR)
            .connection_id(ConnectionId::new(17))
            .establishment(establishment.clone())
            .poison_fn({
                let mutable_flag = Arc::clone(&mutable_flag);
                move || {
                    let mut guard = mutable_flag.lock().unwrap();
                    *guard = !*guard;
                }
            })
            .build();

        assert!(connection_metadata.is_proxied);
        assert_eq!(connection_metadata.remote_addr(), Some(TEST_SOCKET_ADDR));
        assert_eq!(connection_metadata.local_addr(), Some(TEST_SOCKET_ADDR));
        assert_eq!(
            connection_metadata.connection_id(),
            Some(ConnectionId::new(17))
        );
        assert_eq!(connection_metadata.establishment(), Some(&establishment));
        assert_eq!(establishment.total_duration(), Duration::from_millis(13));
        assert_eq!(establishment.transport_duration(), Duration::from_millis(8));
        assert_eq!(
            establishment.protocol_handshake_duration(),
            Some(Duration::from_millis(5))
        );
        assert_eq!(establishment.dns_duration(), Some(Duration::from_millis(1)));
        assert_eq!(
            establishment.socket_connect_duration(),
            Some(Duration::from_millis(2))
        );
        assert_eq!(
            establishment.proxy_duration(),
            Some(Duration::from_millis(3))
        );
        assert_eq!(establishment.tls_duration(), Some(Duration::from_millis(4)));
        assert_eq!("17", ConnectionId::new(17).to_string());
        assert!(!(*mutable_flag.lock().unwrap()));
        connection_metadata.poison();
        assert!(*mutable_flag.lock().unwrap());
    }

    #[test]
    fn builder_optional_fields_translate() {
        let metadata1 = ConnectionMetadataBuilder::new()
            .proxied(true)
            .poison_fn(|| {})
            .build();

        assert_eq!(metadata1.local_addr(), None);
        assert_eq!(metadata1.remote_addr(), None);
        assert_eq!(metadata1.connection_id(), None);
        assert_eq!(metadata1.establishment(), None);

        let metadata2 = ConnectionMetadataBuilder::new()
            .proxied(true)
            .poison_fn(|| {})
            .local_addr(TEST_SOCKET_ADDR)
            .build();

        assert_eq!(metadata2.local_addr(), Some(TEST_SOCKET_ADDR));
        assert_eq!(metadata2.remote_addr(), None);

        let metadata3 = ConnectionMetadataBuilder::new()
            .proxied(true)
            .poison_fn(|| {})
            .remote_addr(TEST_SOCKET_ADDR)
            .build();

        assert_eq!(metadata3.local_addr(), None);
        assert_eq!(metadata3.remote_addr(), Some(TEST_SOCKET_ADDR));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn retrieve_connection_metadata() {
        let retriever = CaptureSmithyConnection::new();
        let retriever_clone = retriever.clone();
        assert!(retriever.get().is_none());
        retriever.set_connection_retriever(|| {
            Some(
                ConnectionMetadata::builder()
                    .proxied(true)
                    .poison_fn(|| {})
                    .build(),
            )
        });

        assert!(retriever.get().is_some());
        assert!(retriever_clone.get().is_some());
    }
}
