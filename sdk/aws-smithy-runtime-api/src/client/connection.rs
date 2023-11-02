/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types related to connection monitoring and management.

use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;

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
