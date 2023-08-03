/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::auth::{
    AuthOptionResolver, AuthOptionResolverParams, DynAuthOptionResolver, HttpAuthSchemes,
};
use crate::client::identity::IdentityResolvers;
use crate::client::interceptors::context::{Error, Input, Output};
use crate::client::retries::RetryStrategy;
use crate::client::retries::{DynRetryStrategy, RetryClassifiers};
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_http::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer, Storable, StoreReplace};
use aws_smithy_types::endpoint::Endpoint;
use aws_smithy_types::type_erasure::{TypeErasedBox, TypedBox};
use bytes::Bytes;
use std::fmt;
use std::fmt::Debug;
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

pub trait Connection: Send + Sync + fmt::Debug {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse>;
}

#[derive(Debug)]
pub struct DynConnection(Box<dyn Connection>);

impl DynConnection {
    pub fn new(connection: impl Connection + 'static) -> Self {
        Self(Box::new(connection))
    }
}

impl Connection for DynConnection {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
        (*self.0).call(request)
    }
}

impl Storable for DynConnection {
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
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Result<Endpoint, BoxError>;
}

#[derive(Debug)]
pub struct DynEndpointResolver(Box<dyn EndpointResolver>);

impl DynEndpointResolver {
    pub fn new(endpoint_resolver: impl EndpointResolver + 'static) -> Self {
        Self(Box::new(endpoint_resolver))
    }
}

impl EndpointResolver for DynEndpointResolver {
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Result<Endpoint, BoxError> {
        self.0.resolve_endpoint(params)
    }
}

impl Storable for DynEndpointResolver {
    type Storer = StoreReplace<Self>;
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

// Place traits in a private module so that they can be used in the public API without being a part of the public API.
mod internal {
    use aws_smithy_types::config_bag::{
        ConfigBag, FrozenLayer, Layer, Storable, Store, StoreReplace,
    };
    use std::fmt::Debug;

    pub trait Settable {
        fn unset<T: Send + Sync + Clone + Debug + 'static>(&mut self);

        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>>;
    }

    impl Settable for Layer {
        fn unset<T: Send + Sync + Clone + Debug + 'static>(&mut self) {
            Layer::unset::<T>(self);
        }

        fn store_put<T>(&mut self, value: T)
        where
            T: Storable<Storer = StoreReplace<T>>,
        {
            Layer::store_put(self, value);
        }
    }

    pub trait Gettable {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_>;
    }

    impl Gettable for ConfigBag {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            ConfigBag::load::<T>(self)
        }
    }

    impl Gettable for Layer {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            Layer::load::<T>(self)
        }
    }

    impl Gettable for FrozenLayer {
        fn load<T: Storable>(&self) -> <T::Storer as Store>::ReturnedType<'_> {
            Layer::load::<T>(self)
        }
    }
}
use internal::{Gettable, Settable};

pub trait ConfigBagAccessors {
    fn auth_option_resolver_params(&self) -> &AuthOptionResolverParams
    where
        Self: Gettable,
    {
        self.load::<AuthOptionResolverParams>()
            .expect("auth option resolver params must be set")
    }
    fn set_auth_option_resolver_params(
        &mut self,
        auth_option_resolver_params: AuthOptionResolverParams,
    ) where
        Self: Settable,
    {
        self.store_put::<AuthOptionResolverParams>(auth_option_resolver_params);
    }

    fn auth_option_resolver(&self) -> &dyn AuthOptionResolver
    where
        Self: Gettable,
    {
        self.load::<DynAuthOptionResolver>()
            .expect("an auth option resolver must be set")
    }

    fn set_auth_option_resolver(&mut self, auth_option_resolver: DynAuthOptionResolver)
    where
        Self: Settable,
    {
        self.store_put::<DynAuthOptionResolver>(auth_option_resolver);
    }

    fn endpoint_resolver_params(&self) -> &EndpointResolverParams
    where
        Self: Gettable,
    {
        self.load::<EndpointResolverParams>()
            .expect("endpoint resolver params must be set")
    }

    fn set_endpoint_resolver_params(&mut self, endpoint_resolver_params: EndpointResolverParams)
    where
        Self: Settable,
    {
        self.store_put::<EndpointResolverParams>(endpoint_resolver_params);
    }

    fn endpoint_resolver(&self) -> &dyn EndpointResolver
    where
        Self: Gettable,
    {
        self.load::<DynEndpointResolver>()
            .expect("an endpoint resolver must be set")
    }

    fn set_endpoint_resolver(&mut self, endpoint_resolver: DynEndpointResolver)
    where
        Self: Settable,
    {
        self.store_put::<DynEndpointResolver>(endpoint_resolver);
    }

    fn identity_resolvers(&self) -> &IdentityResolvers
    where
        Self: Gettable,
    {
        self.load::<IdentityResolvers>()
            .expect("identity resolvers must be configured")
    }

    fn set_identity_resolvers(&mut self, identity_resolvers: IdentityResolvers)
    where
        Self: Settable,
    {
        self.store_put::<IdentityResolvers>(identity_resolvers);
    }

    fn connection(&self) -> &dyn Connection
    where
        Self: Gettable,
    {
        self.load::<DynConnection>().expect("missing connector")
    }

    fn set_connection(&mut self, connection: DynConnection)
    where
        Self: Settable,
    {
        self.store_put::<DynConnection>(connection);
    }

    fn http_auth_schemes(&self) -> &HttpAuthSchemes
    where
        Self: Gettable,
    {
        self.load::<HttpAuthSchemes>()
            .expect("auth schemes must be set")
    }
    fn set_http_auth_schemes(&mut self, http_auth_schemes: HttpAuthSchemes)
    where
        Self: Settable,
    {
        self.store_put::<HttpAuthSchemes>(http_auth_schemes);
    }

    fn request_serializer(&self) -> SharedRequestSerializer
    where
        Self: Gettable,
    {
        self.load::<SharedRequestSerializer>()
            .expect("missing request serializer")
            .clone()
    }
    fn set_request_serializer(&mut self, request_serializer: SharedRequestSerializer)
    where
        Self: Settable,
    {
        self.store_put::<SharedRequestSerializer>(request_serializer);
    }

    fn response_deserializer(&self) -> &dyn ResponseDeserializer
    where
        Self: Gettable,
    {
        self.load::<DynResponseDeserializer>()
            .expect("missing response deserializer")
    }
    fn set_response_deserializer(&mut self, response_deserializer: DynResponseDeserializer)
    where
        Self: Settable,
    {
        self.store_put::<DynResponseDeserializer>(response_deserializer);
    }

    fn retry_classifiers(&self) -> &RetryClassifiers
    where
        Self: Gettable,
    {
        self.load::<RetryClassifiers>()
            .expect("retry classifiers must be set")
    }
    fn set_retry_classifiers(&mut self, retry_classifiers: RetryClassifiers)
    where
        Self: Settable,
    {
        self.store_put::<RetryClassifiers>(retry_classifiers);
    }

    fn retry_strategy(&self) -> Option<&dyn RetryStrategy>
    where
        Self: Gettable,
    {
        self.load::<DynRetryStrategy>().map(|rs| rs as _)
    }
    fn set_retry_strategy(&mut self, retry_strategy: DynRetryStrategy)
    where
        Self: Settable,
    {
        self.store_put::<DynRetryStrategy>(retry_strategy);
    }

    fn request_time(&self) -> Option<SharedTimeSource>
    where
        Self: Gettable,
    {
        self.load::<SharedTimeSource>().cloned()
    }
    fn set_request_time(&mut self, time_source: impl TimeSource + 'static)
    where
        Self: Settable,
    {
        self.store_put::<SharedTimeSource>(SharedTimeSource::new(time_source));
    }

    fn sleep_impl(&self) -> Option<SharedAsyncSleep>
    where
        Self: Gettable,
    {
        self.load::<SharedAsyncSleep>().cloned()
    }
    fn set_sleep_impl(&mut self, async_sleep: Option<SharedAsyncSleep>)
    where
        Self: Settable,
    {
        if let Some(sleep_impl) = async_sleep {
            self.store_put::<SharedAsyncSleep>(sleep_impl);
        } else {
            self.unset::<SharedAsyncSleep>();
        }
    }

    fn loaded_request_body(&self) -> &LoadedRequestBody
    where
        Self: Gettable,
    {
        self.load::<LoadedRequestBody>().unwrap_or(&NOT_NEEDED)
    }
    fn set_loaded_request_body(&mut self, loaded_request_body: LoadedRequestBody)
    where
        Self: Settable,
    {
        self.store_put::<LoadedRequestBody>(loaded_request_body);
    }
}

const NOT_NEEDED: LoadedRequestBody = LoadedRequestBody::NotNeeded;

impl ConfigBagAccessors for ConfigBag {}
impl ConfigBagAccessors for FrozenLayer {}
impl ConfigBagAccessors for Layer {}
