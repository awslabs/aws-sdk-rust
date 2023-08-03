/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{BoxFuture, HttpRequest, HttpResponse};
use std::fmt;
use std::sync::Arc;

/// Trait with a `call` function that asynchronously converts a request into a response.
///
/// Ordinarily, a connector would use an underlying HTTP library such as [hyper](https://crates.io/crates/hyper),
/// and any associated HTTPS implementation alongside it to service requests.
///
/// However, it can also be useful to create fake connectors implementing this trait
/// for testing.
pub trait HttpConnector: Send + Sync + fmt::Debug {
    /// Asynchronously converts a request into a response.
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse>;
}

/// A shared [`HttpConnector`] implementation.
#[derive(Clone, Debug)]
pub struct SharedHttpConnector(Arc<dyn HttpConnector>);

impl SharedHttpConnector {
    /// Returns a new [`SharedHttpConnector`].
    pub fn new(connection: impl HttpConnector + 'static) -> Self {
        Self(Arc::new(connection))
    }
}

impl HttpConnector for SharedHttpConnector {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
        (*self.0).call(request)
    }
}
