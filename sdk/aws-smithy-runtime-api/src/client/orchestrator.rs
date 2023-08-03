/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Errors that can occur while running the orchestrator.
mod error;

use crate::client::auth::{AuthOptionResolver, AuthOptionResolverParams, HttpAuthSchemes};
use crate::client::identity::IdentityResolvers;
use crate::client::interceptors::context::{Error, Input, Output};
use crate::client::retries::RetryClassifiers;
use crate::client::retries::RetryStrategy;
use crate::config_bag::ConfigBag;
use crate::type_erasure::{TypeErasedBox, TypedBox};
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_http::body::SdkBody;
use aws_smithy_types::endpoint::Endpoint;
use bytes::Bytes;
use std::fmt;
use std::future::Future as StdFuture;
use std::pin::Pin;
use std::sync::Arc;

pub use error::OrchestratorError;

pub type HttpRequest = http::Request<SdkBody>;
pub type HttpResponse = http::Response<SdkBody>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
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

pub trait ConfigBagAccessors {
    fn auth_option_resolver_params(&self) -> &AuthOptionResolverParams;
    fn set_auth_option_resolver_params(
        &mut self,
        auth_option_resolver_params: AuthOptionResolverParams,
    );

    fn auth_option_resolver(&self) -> &dyn AuthOptionResolver;
    fn set_auth_option_resolver(&mut self, auth_option_resolver: impl AuthOptionResolver + 'static);

    fn endpoint_resolver_params(&self) -> &EndpointResolverParams;
    fn set_endpoint_resolver_params(&mut self, endpoint_resolver_params: EndpointResolverParams);

    fn endpoint_resolver(&self) -> &dyn EndpointResolver;
    fn set_endpoint_resolver(&mut self, endpoint_resolver: impl EndpointResolver + 'static);

    fn identity_resolvers(&self) -> &IdentityResolvers;
    fn set_identity_resolvers(&mut self, identity_resolvers: IdentityResolvers);

    fn connection(&self) -> &dyn Connection;
    fn set_connection(&mut self, connection: impl Connection + 'static);

    fn http_auth_schemes(&self) -> &HttpAuthSchemes;
    fn set_http_auth_schemes(&mut self, http_auth_schemes: HttpAuthSchemes);

    fn request_serializer(&self) -> Arc<dyn RequestSerializer>;
    fn set_request_serializer(&mut self, request_serializer: impl RequestSerializer + 'static);

    fn response_deserializer(&self) -> &dyn ResponseDeserializer;
    fn set_response_deserializer(
        &mut self,
        response_serializer: impl ResponseDeserializer + 'static,
    );

    fn retry_classifiers(&self) -> &RetryClassifiers;
    fn set_retry_classifiers(&mut self, retry_classifier: RetryClassifiers);

    fn retry_strategy(&self) -> &dyn RetryStrategy;
    fn set_retry_strategy(&mut self, retry_strategy: impl RetryStrategy + 'static);

    fn request_time(&self) -> Option<SharedTimeSource>;
    fn set_request_time(&mut self, time_source: impl TimeSource + 'static);

    fn sleep_impl(&self) -> Option<Arc<dyn AsyncSleep>>;
    fn set_sleep_impl(&mut self, async_sleep: Option<Arc<dyn AsyncSleep>>);

    fn loaded_request_body(&self) -> &LoadedRequestBody;
    fn set_loaded_request_body(&mut self, loaded_request_body: LoadedRequestBody);
}

const NOT_NEEDED: LoadedRequestBody = LoadedRequestBody::NotNeeded;

impl ConfigBagAccessors for ConfigBag {
    fn auth_option_resolver_params(&self) -> &AuthOptionResolverParams {
        self.get::<AuthOptionResolverParams>()
            .expect("auth option resolver params must be set")
    }

    fn set_auth_option_resolver_params(
        &mut self,
        auth_option_resolver_params: AuthOptionResolverParams,
    ) {
        self.put::<AuthOptionResolverParams>(auth_option_resolver_params);
    }

    fn auth_option_resolver(&self) -> &dyn AuthOptionResolver {
        &**self
            .get::<Box<dyn AuthOptionResolver>>()
            .expect("an auth option resolver must be set")
    }

    fn set_auth_option_resolver(
        &mut self,
        auth_option_resolver: impl AuthOptionResolver + 'static,
    ) {
        self.put::<Box<dyn AuthOptionResolver>>(Box::new(auth_option_resolver));
    }

    fn endpoint_resolver_params(&self) -> &EndpointResolverParams {
        self.get::<EndpointResolverParams>()
            .expect("endpoint resolver params must be set")
    }

    fn set_endpoint_resolver_params(&mut self, endpoint_resolver_params: EndpointResolverParams) {
        self.put::<EndpointResolverParams>(endpoint_resolver_params);
    }

    fn endpoint_resolver(&self) -> &dyn EndpointResolver {
        &**self
            .get::<Box<dyn EndpointResolver>>()
            .expect("an endpoint resolver must be set")
    }

    fn set_endpoint_resolver(&mut self, endpoint_resolver: impl EndpointResolver + 'static) {
        self.put::<Box<dyn EndpointResolver>>(Box::new(endpoint_resolver));
    }

    fn identity_resolvers(&self) -> &IdentityResolvers {
        self.get::<IdentityResolvers>()
            .expect("identity resolvers must be configured")
    }

    fn set_identity_resolvers(&mut self, identity_resolvers: IdentityResolvers) {
        self.put::<IdentityResolvers>(identity_resolvers);
    }

    fn connection(&self) -> &dyn Connection {
        &**self
            .get::<Box<dyn Connection>>()
            .expect("missing connector")
    }

    fn set_connection(&mut self, connection: impl Connection + 'static) {
        self.put::<Box<dyn Connection>>(Box::new(connection));
    }

    fn http_auth_schemes(&self) -> &HttpAuthSchemes {
        self.get::<HttpAuthSchemes>()
            .expect("auth schemes must be set")
    }

    fn set_http_auth_schemes(&mut self, http_auth_schemes: HttpAuthSchemes) {
        self.put::<HttpAuthSchemes>(http_auth_schemes);
    }

    fn request_serializer(&self) -> Arc<dyn RequestSerializer> {
        self.get::<Arc<dyn RequestSerializer>>()
            .expect("missing request serializer")
            .clone()
    }

    fn set_request_serializer(&mut self, request_serializer: impl RequestSerializer + 'static) {
        self.put::<Arc<dyn RequestSerializer>>(Arc::new(request_serializer));
    }

    fn response_deserializer(&self) -> &dyn ResponseDeserializer {
        &**self
            .get::<Box<dyn ResponseDeserializer>>()
            .expect("missing response deserializer")
    }

    fn set_response_deserializer(
        &mut self,
        response_deserializer: impl ResponseDeserializer + 'static,
    ) {
        self.put::<Box<dyn ResponseDeserializer>>(Box::new(response_deserializer));
    }

    fn retry_classifiers(&self) -> &RetryClassifiers {
        self.get::<RetryClassifiers>()
            .expect("retry classifiers must be set")
    }

    fn set_retry_classifiers(&mut self, retry_classifiers: RetryClassifiers) {
        self.put::<RetryClassifiers>(retry_classifiers);
    }

    fn retry_strategy(&self) -> &dyn RetryStrategy {
        &**self
            .get::<Box<dyn RetryStrategy>>()
            .expect("a retry strategy must be set")
    }

    fn set_retry_strategy(&mut self, retry_strategy: impl RetryStrategy + 'static) {
        self.put::<Box<dyn RetryStrategy>>(Box::new(retry_strategy));
    }

    fn request_time(&self) -> Option<SharedTimeSource> {
        self.get::<SharedTimeSource>().cloned()
    }

    fn set_request_time(&mut self, request_time: impl TimeSource + 'static) {
        self.put::<SharedTimeSource>(SharedTimeSource::new(request_time));
    }

    fn sleep_impl(&self) -> Option<Arc<dyn AsyncSleep>> {
        self.get::<Arc<dyn AsyncSleep>>().cloned()
    }

    fn set_sleep_impl(&mut self, sleep_impl: Option<Arc<dyn AsyncSleep>>) {
        if let Some(sleep_impl) = sleep_impl {
            self.put::<Arc<dyn AsyncSleep>>(sleep_impl);
        } else {
            self.unset::<Arc<dyn AsyncSleep>>();
        }
    }

    fn loaded_request_body(&self) -> &LoadedRequestBody {
        self.get::<LoadedRequestBody>().unwrap_or(&NOT_NEEDED)
    }

    fn set_loaded_request_body(&mut self, loaded_request_body: LoadedRequestBody) {
        self.put::<LoadedRequestBody>(loaded_request_body);
    }
}
