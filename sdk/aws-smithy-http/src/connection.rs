/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ConnectionMetadata {
    is_proxied: bool,
    remote_addr: Option<SocketAddr>,
    poison_fn: Arc<dyn Fn() + Send + Sync>,
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

#[derive(Clone, Default)]
pub struct CaptureSmithyConnection {
    loader: Arc<Mutex<Option<Box<LoaderFn>>>>,
}

impl CaptureSmithyConnection {
    pub fn new() -> Self {
        Self {
            loader: Default::default(),
        }
    }
    pub fn set_connection_retriever<F>(&self, f: F)
    where
        F: Fn() -> Option<ConnectionMetadata> + Send + Sync + 'static,
    {
        *self.loader.lock().unwrap() = Some(Box::new(f));
    }

    pub fn get(&self) -> Option<ConnectionMetadata> {
        match self.loader.lock().unwrap().as_ref() {
            Some(loader) => loader(),
            None => {
                println!("no loader was set :-/");
                None
            }
        }
    }
}

impl ConnectionMetadata {
    pub fn poison(&self) {
        tracing::info!("smithy connection was poisoned");
        (self.poison_fn)()
    }
}

impl ConnectionMetadata {
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

    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
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
