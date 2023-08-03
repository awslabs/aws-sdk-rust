/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::auth::{AuthOptionResolver, AuthOptionResolverParams, HttpAuthSchemes};
use crate::client::identity::IdentityResolvers;
use crate::client::interceptors::context::{Error, Input, Output};
use crate::client::retries::RetryClassifiers;
use crate::client::retries::RetryStrategy;
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_http::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, Layer};
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

pub trait Connection: Send + Sync + fmt::Debug {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse>;
}

impl Connection for Box<dyn Connection> {
    fn call(&self, request: HttpRequest) -> BoxFuture<HttpResponse> {
        (**self).call(request)
    }
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

pub trait EndpointResolver: Send + Sync + fmt::Debug {
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Result<Endpoint, BoxError>;
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

pub trait Settable {
    fn layer(&mut self) -> &mut Layer;
    fn put<T: Send + Sync + Debug + 'static>(&mut self, value: T) {
        self.layer().put(value);
    }
}

pub trait Gettable {
    fn config_bag(&self) -> &ConfigBag;
    fn get<T: Send + Sync + Debug + 'static>(&self) -> Option<&T> {
        self.config_bag().get::<T>()
    }
}

impl Settable for Layer {
    fn layer(&mut self) -> &mut Layer {
        self
    }
}

impl Gettable for ConfigBag {
    fn config_bag(&self) -> &ConfigBag {
        self
    }
}

pub trait ConfigBagAccessors {
    fn auth_option_resolver_params(&self) -> &AuthOptionResolverParams
    where
        Self: Gettable,
    {
        self.config_bag()
            .get::<AuthOptionResolverParams>()
            .expect("auth option resolver params must be set")
    }
    fn set_auth_option_resolver_params(
        &mut self,
        auth_option_resolver_params: AuthOptionResolverParams,
    ) where
        Self: Settable,
    {
        self.put::<AuthOptionResolverParams>(auth_option_resolver_params);
    }

    fn auth_option_resolver(&self) -> &dyn AuthOptionResolver
    where
        Self: Gettable,
    {
        &**self
            .config_bag()
            .get::<Box<dyn AuthOptionResolver>>()
            .expect("an auth option resolver must be set")
    }

    fn set_auth_option_resolver(&mut self, auth_option_resolver: impl AuthOptionResolver + 'static)
    where
        Self: Settable,
    {
        self.put::<Box<dyn AuthOptionResolver>>(Box::new(auth_option_resolver));
    }

    fn endpoint_resolver_params(&self) -> &EndpointResolverParams
    where
        Self: Gettable,
    {
        self.config_bag()
            .get::<EndpointResolverParams>()
            .expect("endpoint resolver params must be set")
    }

    fn set_endpoint_resolver_params(&mut self, endpoint_resolver_params: EndpointResolverParams)
    where
        Self: Settable,
    {
        self.put::<EndpointResolverParams>(endpoint_resolver_params);
    }

    fn endpoint_resolver(&self) -> &dyn EndpointResolver
    where
        Self: Gettable,
    {
        &**self
            .config_bag()
            .get::<Box<dyn EndpointResolver>>()
            .expect("an endpoint resolver must be set")
    }

    fn set_endpoint_resolver(&mut self, endpoint_resolver: impl EndpointResolver + 'static)
    where
        Self: Settable,
    {
        self.put::<Box<dyn EndpointResolver>>(Box::new(endpoint_resolver));
    }

    fn identity_resolvers(&self) -> &IdentityResolvers
    where
        Self: Gettable,
    {
        self.config_bag()
            .get::<IdentityResolvers>()
            .expect("identity resolvers must be configured")
    }

    fn set_identity_resolvers(&mut self, identity_resolvers: IdentityResolvers)
    where
        Self: Settable,
    {
        self.put::<IdentityResolvers>(identity_resolvers);
    }

    fn connection(&self) -> &dyn Connection
    where
        Self: Gettable,
    {
        &**self
            .config_bag()
            .get::<Box<dyn Connection>>()
            .expect("missing connector")
    }

    fn set_connection(&mut self, connection: impl Connection + 'static)
    where
        Self: Settable,
    {
        self.put::<Box<dyn Connection>>(Box::new(connection));
    }

    fn http_auth_schemes(&self) -> &HttpAuthSchemes
    where
        Self: Gettable,
    {
        self.config_bag()
            .get::<HttpAuthSchemes>()
            .expect("auth schemes must be set")
    }
    fn set_http_auth_schemes(&mut self, http_auth_schemes: HttpAuthSchemes)
    where
        Self: Settable,
    {
        self.put::<HttpAuthSchemes>(http_auth_schemes);
    }

    fn request_serializer(&self) -> Arc<dyn RequestSerializer>
    where
        Self: Gettable,
    {
        self.get::<Arc<dyn RequestSerializer>>()
            .expect("missing request serializer")
            .clone()
    }
    fn set_request_serializer(&mut self, request_serializer: impl RequestSerializer + 'static)
    where
        Self: Settable,
    {
        self.put::<Arc<dyn RequestSerializer>>(Arc::new(request_serializer));
    }

    fn response_deserializer(&self) -> &dyn ResponseDeserializer
    where
        Self: Gettable,
    {
        &**self
            .get::<Box<dyn ResponseDeserializer>>()
            .expect("missing response deserializer")
    }
    fn set_response_deserializer(
        &mut self,
        response_deserializer: impl ResponseDeserializer + 'static,
    ) where
        Self: Settable,
    {
        self.put::<Box<dyn ResponseDeserializer>>(Box::new(response_deserializer));
    }

    fn retry_classifiers(&self) -> &RetryClassifiers
    where
        Self: Gettable,
    {
        self.get::<RetryClassifiers>()
            .expect("retry classifiers must be set")
    }
    fn set_retry_classifiers(&mut self, retry_classifiers: RetryClassifiers)
    where
        Self: Settable,
    {
        self.put::<RetryClassifiers>(retry_classifiers);
    }

    fn retry_strategy(&self) -> Option<&dyn RetryStrategy>
    where
        Self: Gettable,
    {
        self.get::<Box<dyn RetryStrategy>>().map(|rs| &**rs)
    }
    fn set_retry_strategy(&mut self, retry_strategy: impl RetryStrategy + 'static)
    where
        Self: Settable,
    {
        self.put::<Box<dyn RetryStrategy>>(Box::new(retry_strategy));
    }

    fn request_time(&self) -> Option<SharedTimeSource>
    where
        Self: Gettable,
    {
        self.get::<SharedTimeSource>().cloned()
    }
    fn set_request_time(&mut self, time_source: impl TimeSource + 'static)
    where
        Self: Settable,
    {
        self.put::<SharedTimeSource>(SharedTimeSource::new(time_source));
    }

    fn sleep_impl(&self) -> Option<SharedAsyncSleep>
    where
        Self: Gettable,
    {
        self.get::<SharedAsyncSleep>().cloned()
    }
    fn set_sleep_impl(&mut self, async_sleep: Option<SharedAsyncSleep>)
    where
        Self: Settable,
    {
        if let Some(sleep_impl) = async_sleep {
            self.put::<SharedAsyncSleep>(sleep_impl);
        } else {
            self.layer().unset::<SharedAsyncSleep>();
        }
    }

    fn loaded_request_body(&self) -> &LoadedRequestBody
    where
        Self: Gettable,
    {
        self.get::<LoadedRequestBody>().unwrap_or(&NOT_NEEDED)
    }
    fn set_loaded_request_body(&mut self, loaded_request_body: LoadedRequestBody)
    where
        Self: Settable,
    {
        self.put::<LoadedRequestBody>(loaded_request_body);
    }
}

const NOT_NEEDED: LoadedRequestBody = LoadedRequestBody::NotNeeded;

impl ConfigBagAccessors for ConfigBag {}
impl ConfigBagAccessors for Layer {}
