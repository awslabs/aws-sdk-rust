/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::interceptors::context::{Error, Input, Output};
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_http::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::type_erasure::{TypeErasedBox, TypedBox};
use bytes::Bytes;
use std::fmt;
use std::future::Future as StdFuture;
use std::pin::Pin;
use std::sync::Arc;

/// Errors that can occur while running the orchestrator.
mod error;

pub use error::OrchestratorError;

pub type HttpRequest = http::Request<SdkBody>;
pub type HttpResponse = http::Response<SdkBody>;
pub type BoxFuture<T> = Pin<Box<dyn StdFuture<Output = Result<T, BoxError>> + Send>>;
pub type Future<T> = NowOrLater<Result<T, BoxError>, BoxFuture<T>>;

pub trait RequestSerializer: Send + Sync + fmt::Debug {
    fn serialize_input(&self, input: Input, cfg: &mut ConfigBag) -> Result<HttpRequest, BoxError>;
}

#[derive(Clone, Debug)]
pub struct SharedRequestSerializer(Arc<dyn RequestSerializer>);

impl SharedRequestSerializer {
    pub fn new(serializer: impl RequestSerializer + 'static) -> Self {
        Self(Arc::new(serializer))
    }
}

impl RequestSerializer for SharedRequestSerializer {
    fn serialize_input(&self, input: Input, cfg: &mut ConfigBag) -> Result<HttpRequest, BoxError> {
        self.0.serialize_input(input, cfg)
    }
}

impl Storable for SharedRequestSerializer {
    type Storer = StoreReplace<Self>;
}

pub trait ResponseDeserializer: Send + Sync + fmt::Debug {
    fn deserialize_streaming(
        &self,
        response: &mut HttpResponse,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        let _ = response;
        None
    }

    fn deserialize_nonstreaming(
        &self,
        response: &HttpResponse,
    ) -> Result<Output, OrchestratorError<Error>>;
}

#[derive(Debug)]
pub struct DynResponseDeserializer(Box<dyn ResponseDeserializer>);

impl DynResponseDeserializer {
    pub fn new(serializer: impl ResponseDeserializer + 'static) -> Self {
        Self(Box::new(serializer))
    }
}

impl ResponseDeserializer for DynResponseDeserializer {
    fn deserialize_nonstreaming(
        &self,
        response: &HttpResponse,
    ) -> Result<Output, OrchestratorError<Error>> {
        self.0.deserialize_nonstreaming(response)
    }

    fn deserialize_streaming(
        &self,
        response: &mut HttpResponse,
    ) -> Option<Result<Output, OrchestratorError<Error>>> {
        self.0.deserialize_streaming(response)
    }
}

impl Storable for DynResponseDeserializer {
    type Storer = StoreReplace<Self>;
}

#[derive(Debug)]
pub struct EndpointResolverParams(TypeErasedBox);

impl EndpointResolverParams {
    pub fn new<T: fmt::Debug + Send + Sync + 'static>(params: T) -> Self {
        Self(TypedBox::new(params).erase())
    }

    pub fn get<T: fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

impl Storable for EndpointResolverParams {
    type Storer = StoreReplace<Self>;
}

pub trait EndpointResolver: Send + Sync + fmt::Debug {
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Future<Endpoint>;
}

#[derive(Clone, Debug)]
pub struct SharedEndpointResolver(Arc<dyn EndpointResolver>);

impl SharedEndpointResolver {
    pub fn new(endpoint_resolver: impl EndpointResolver + 'static) -> Self {
        Self(Arc::new(endpoint_resolver))
    }
}

impl EndpointResolver for SharedEndpointResolver {
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Future<Endpoint> {
        self.0.resolve_endpoint(params)
    }
}

/// Informs the orchestrator on whether or not the request body needs to be loaded into memory before transmit.
///
/// This enum gets placed into the `ConfigBag` to change the orchestrator behavior.
/// Immediately after serialization (before the `read_after_serialization` interceptor hook),
/// if it was set to `Requested` in the config bag, it will be replaced back into the config bag as
/// `Loaded` with the request body contents for use in later interceptors.
///
/// This all happens before the attempt loop, so the loaded request body will remain available
/// for interceptors that run in any subsequent retry attempts.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum LoadedRequestBody {
    /// Don't attempt to load the request body into memory.
    NotNeeded,
    /// Attempt to load the request body into memory.
    Requested,
    /// The request body is already loaded.
    Loaded(Bytes),
}

impl Storable for LoadedRequestBody {
    type Storer = StoreReplace<Self>;
}

pub(crate) const NOT_NEEDED: LoadedRequestBody = LoadedRequestBody::NotNeeded;
