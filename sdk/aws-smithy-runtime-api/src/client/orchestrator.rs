/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::auth::{AuthOptionResolver, AuthOptionResolverParams, HttpAuthSchemes};
use crate::client::identity::IdentityResolvers;
use crate::client::interceptors::context::{Error, Input, Output};
use crate::client::retries::RetryClassifiers;
use crate::client::retries::RetryStrategy;
use crate::config_bag::ConfigBag;
use crate::type_erasure::{TypeErasedBox, TypedBox};
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::endpoint::EndpointPrefix;
use std::fmt;
use std::future::Future as StdFuture;
use std::pin::Pin;
use std::sync::Arc;
use std::time::SystemTime;

pub type HttpRequest = http::Request<SdkBody>;
pub type HttpResponse = http::Response<SdkBody>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxFuture<T> = Pin<Box<dyn StdFuture<Output = Result<T, BoxError>>>>;
pub type Future<T> = NowOrLater<Result<T, BoxError>, BoxFuture<T>>;

pub trait RequestSerializer: Send + Sync + fmt::Debug {
    fn serialize_input(&self, input: Input) -> Result<HttpRequest, BoxError>;
}

pub trait ResponseDeserializer: Send + Sync + fmt::Debug {
    fn deserialize_streaming(&self, response: &mut HttpResponse) -> Option<Result<Output, Error>> {
        let _ = response;
        None
    }

    fn deserialize_nonstreaming(&self, response: &HttpResponse) -> Result<Output, Error>;
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
    fn resolve_and_apply_endpoint(
        &self,
        params: &EndpointResolverParams,
        endpoint_prefix: Option<&EndpointPrefix>,
        request: &mut HttpRequest,
    ) -> Result<(), BoxError>;
}

/// Time that the request is being made (so that time can be overridden in the [`ConfigBag`]).
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct RequestTime(SystemTime);

impl Default for RequestTime {
    fn default() -> Self {
        Self(SystemTime::now())
    }
}

impl RequestTime {
    /// Create a new [`RequestTime`].
    pub fn new(time: SystemTime) -> Self {
        Self(time)
    }

    /// Returns the request time as a [`SystemTime`].
    pub fn system_time(&self) -> SystemTime {
        self.0
    }
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

    fn request_serializer(&self) -> &dyn RequestSerializer;
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

    fn request_time(&self) -> Option<RequestTime>;
    fn set_request_time(&mut self, request_time: RequestTime);

    fn sleep_impl(&self) -> Option<Arc<dyn AsyncSleep>>;
    fn set_sleep_impl(&mut self, async_sleep: Option<Arc<dyn AsyncSleep>>);
}

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

    fn request_serializer(&self) -> &dyn RequestSerializer {
        &**self
            .get::<Box<dyn RequestSerializer>>()
            .expect("missing request serializer")
    }

    fn set_request_serializer(&mut self, request_serializer: impl RequestSerializer + 'static) {
        self.put::<Box<dyn RequestSerializer>>(Box::new(request_serializer));
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

    fn request_time(&self) -> Option<RequestTime> {
        self.get::<RequestTime>().cloned()
    }

    fn set_request_time(&mut self, request_time: RequestTime) {
        self.put::<RequestTime>(request_time);
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
}
