/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types related to connection monitoring and management.

use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Metadata that tracks the state of an active connection.
#[derive(Clone)]
pub struct ConnectionMetadata {
    is_proxied: bool,
    remote_addr: Option<SocketAddr>,
    poison_fn: Arc<dyn Fn() + Send + Sync>,
}

impl ConnectionMetadata {
    /// Poison this connection, ensuring that it won't be reused.
    pub fn poison(&self) {
        tracing::info!("smithy connection was poisoned");
        (self.poison_fn)()
    }

    /// Create a new [`ConnectionMetadata`].
    pub fn new(
        is_proxied: bool,
        remote_addr: Option<SocketAddr>,
        poison: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            is_proxied,
            remote_addr,
            poison_fn: Arc::new(poison),
        }
    }

    /// Get the remote address for this connection, if one is set.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }
}

impl Debug for ConnectionMetadata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmithyConnection")
            .field("is_proxied", &self.is_proxied)
            .field("remote_addr", &self.remote_addr)
            .finish()
    }
}

type LoaderFn = dyn Fn() -> Option<ConnectionMetadata> + Send + Sync;

/// State for a middleware that will monitor and manage connections.
#[allow(missing_debug_implementations)]
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

#[cfg(test)]
mod test {
    use crate::connection::{CaptureSmithyConnection, ConnectionMetadata};

    #[test]
    fn retrieve_connection_metadata() {
        let retriever = CaptureSmithyConnection::new();
        let retriever_clone = retriever.clone();
        assert!(retriever.get().is_none());
        retriever.set_connection_retriever(|| Some(ConnectionMetadata::new(true, None, || {})));

        assert!(retriever.get().is_some());
        assert!(retriever_clone.get().is_some());
    }
}
