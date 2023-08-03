/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! APIs needed to configure endpoint resolution for clients.

use crate::client::orchestrator::Future;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::type_erasure::TypeErasedBox;
use std::fmt;
use std::sync::Arc;

/// Parameters originating from the Smithy endpoint ruleset required for endpoint resolution.
///
/// The actual endpoint parameters are code generated from the Smithy model, and thus,
/// are not known to the runtime crates. Hence, this struct is really a new-type around
/// a [`TypeErasedBox`] that holds the actual concrete parameters in it.
#[derive(Debug)]
pub struct EndpointResolverParams(TypeErasedBox);

impl EndpointResolverParams {
    /// Creates a new [`EndpointResolverParams`] from a concrete parameters instance.
    pub fn new<T: fmt::Debug + Send + Sync + 'static>(params: T) -> Self {
        Self(TypeErasedBox::new(params))
    }

    /// Attempts to downcast the underlying concrete parameters to `T` and return it as a reference.
    pub fn get<T: fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl Storable for EndpointResolverParams {
    type Storer = StoreReplace<Self>;
}

/// Configurable endpoint resolver implementation.
pub trait EndpointResolver: Send + Sync + fmt::Debug {
    /// Asynchronously resolves an endpoint to use from the given endpoint parameters.
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Future<Endpoint>;
}

/// Shared endpoint resolver.
///
/// This is a simple shared ownership wrapper type for the [`EndpointResolver`] trait.
#[derive(Clone, Debug)]
pub struct SharedEndpointResolver(Arc<dyn EndpointResolver>);

impl SharedEndpointResolver {
    /// Creates a new [`SharedEndpointResolver`].
    pub fn new(endpoint_resolver: impl EndpointResolver + 'static) -> Self {
        Self(Arc::new(endpoint_resolver))
    }
}

impl EndpointResolver for SharedEndpointResolver {
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Future<Endpoint> {
        self.0.resolve_endpoint(params)
    }
}
