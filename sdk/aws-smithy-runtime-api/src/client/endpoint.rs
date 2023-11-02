/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! APIs needed to configure endpoint resolution for clients.

use crate::box_error::BoxError;
use crate::client::runtime_components::sealed::ValidateConfig;
use crate::impl_shared_conversions;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::type_erasure::TypeErasedBox;
use std::fmt;
use std::sync::Arc;

new_type_future! {
    #[doc = "Future for [`EndpointResolver::resolve_endpoint`]."]
    pub struct EndpointFuture<'a, Endpoint, BoxError>;
}

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

#[deprecated(note = "Renamed to ResolveEndpoint.")]
pub use ResolveEndpoint as EndpointResolver;

/// Configurable endpoint resolver implementation.
pub trait ResolveEndpoint: Send + Sync + fmt::Debug {
    /// Asynchronously resolves an endpoint to use from the given endpoint parameters.
    fn resolve_endpoint<'a>(&'a self, params: &'a EndpointResolverParams) -> EndpointFuture<'a>;
}

/// Shared endpoint resolver.
///
/// This is a simple shared ownership wrapper type for the [`ResolveEndpoint`] trait.
#[derive(Clone, Debug)]
pub struct SharedEndpointResolver(Arc<dyn ResolveEndpoint>);

impl SharedEndpointResolver {
    /// Creates a new [`SharedEndpointResolver`].
    pub fn new(endpoint_resolver: impl ResolveEndpoint + 'static) -> Self {
        Self(Arc::new(endpoint_resolver))
    }
}

impl ResolveEndpoint for SharedEndpointResolver {
    fn resolve_endpoint<'a>(&'a self, params: &'a EndpointResolverParams) -> EndpointFuture<'a> {
        self.0.resolve_endpoint(params)
    }
}

impl ValidateConfig for SharedEndpointResolver {}

impl_shared_conversions!(convert SharedEndpointResolver from ResolveEndpoint using SharedEndpointResolver::new);
