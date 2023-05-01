/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::identity::Identity;
use crate::client::interceptors::context::{Input, OutputOrError};
use crate::client::interceptors::InterceptorContext;
use crate::config_bag::ConfigBag;
use crate::type_erasure::{TypeErasedBox, TypedBox};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::endpoint::EndpointPrefix;
use aws_smithy_http::property_bag::PropertyBag;
use std::any::Any;
use std::borrow::Cow;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type HttpRequest = http::Request<SdkBody>;
pub type HttpResponse = http::Response<SdkBody>;
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxFallibleFut<T> = Pin<Box<dyn Future<Output = Result<T, BoxError>>>>;

pub trait TraceProbe: Send + Sync + Debug {
    fn dispatch_events(&self);
}

pub trait RequestSerializer: Send + Sync + Debug {
    fn serialize_input(&self, input: Input) -> Result<HttpRequest, BoxError>;
}

pub trait ResponseDeserializer: Send + Sync + Debug {
    fn deserialize_streaming(&self, response: &mut HttpResponse) -> Option<OutputOrError> {
        let _ = response;
        None
    }

    fn deserialize_nonstreaming(&self, response: &HttpResponse) -> OutputOrError;
}

pub trait Connection: Send + Sync + Debug {
    fn call(&self, request: HttpRequest) -> BoxFallibleFut<HttpResponse>;
}

impl Connection for Box<dyn Connection> {
    fn call(&self, request: HttpRequest) -> BoxFallibleFut<HttpResponse> {
        (**self).call(request)
    }
}

pub trait RetryStrategy: Send + Sync + Debug {
    fn should_attempt_initial_request(&self, cfg: &ConfigBag) -> Result<(), BoxError>;

    fn should_attempt_retry(
        &self,
        context: &InterceptorContext<HttpRequest, HttpResponse>,
        cfg: &ConfigBag,
    ) -> Result<bool, BoxError>;
}

#[derive(Debug)]
pub struct AuthOptionResolverParams(TypeErasedBox);

impl AuthOptionResolverParams {
    pub fn new<T: Any + Send + Sync + 'static>(params: T) -> Self {
        Self(TypedBox::new(params).erase())
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

pub trait AuthOptionResolver: Send + Sync + Debug {
    fn resolve_auth_options<'a>(
        &'a self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'a, [HttpAuthOption]>, BoxError>;
}

impl AuthOptionResolver for Box<dyn AuthOptionResolver> {
    fn resolve_auth_options<'a>(
        &'a self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'a, [HttpAuthOption]>, BoxError> {
        (**self).resolve_auth_options(params)
    }
}

#[derive(Clone, Debug)]
pub struct HttpAuthOption {
    scheme_id: &'static str,
    properties: Arc<PropertyBag>,
}

impl HttpAuthOption {
    pub fn new(scheme_id: &'static str, properties: Arc<PropertyBag>) -> Self {
        Self {
            scheme_id,
            properties,
        }
    }

    pub fn scheme_id(&self) -> &'static str {
        self.scheme_id
    }

    pub fn properties(&self) -> &PropertyBag {
        &self.properties
    }
}

pub trait IdentityResolver: Send + Sync + Debug {
    fn resolve_identity(&self, identity_properties: &PropertyBag) -> BoxFallibleFut<Identity>;
}

#[derive(Debug)]
pub struct IdentityResolvers {
    identity_resolvers: Vec<(&'static str, Box<dyn IdentityResolver>)>,
}

impl IdentityResolvers {
    pub fn builder() -> builders::IdentityResolversBuilder {
        builders::IdentityResolversBuilder::new()
    }

    pub fn identity_resolver(&self, identity_type: &'static str) -> Option<&dyn IdentityResolver> {
        self.identity_resolvers
            .iter()
            .find(|resolver| resolver.0 == identity_type)
            .map(|resolver| &*resolver.1)
    }
}

#[derive(Debug)]
struct HttpAuthSchemesInner {
    schemes: Vec<(&'static str, Box<dyn HttpAuthScheme>)>,
}
#[derive(Debug)]
pub struct HttpAuthSchemes {
    inner: Arc<HttpAuthSchemesInner>,
}

impl HttpAuthSchemes {
    pub fn builder() -> builders::HttpAuthSchemesBuilder {
        Default::default()
    }

    pub fn scheme(&self, name: &'static str) -> Option<&dyn HttpAuthScheme> {
        self.inner
            .schemes
            .iter()
            .find(|scheme| scheme.0 == name)
            .map(|scheme| &*scheme.1)
    }
}

pub trait HttpAuthScheme: Send + Sync + Debug {
    fn scheme_id(&self) -> &'static str;

    fn identity_resolver<'a>(
        &self,
        identity_resolvers: &'a IdentityResolvers,
    ) -> Option<&'a dyn IdentityResolver>;

    fn request_signer(&self) -> &dyn HttpRequestSigner;
}

pub trait HttpRequestSigner: Send + Sync + Debug {
    /// Return a signed version of the given request using the given identity.
    ///
    /// If the provided identity is incompatible with this signer, an error must be returned.
    fn sign_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        signing_properties: &PropertyBag,
    ) -> Result<(), BoxError>;
}

#[derive(Debug)]
pub struct EndpointResolverParams(TypeErasedBox);

impl EndpointResolverParams {
    pub fn new<T: Any + Send + Sync + 'static>(params: T) -> Self {
        Self(TypedBox::new(params).erase())
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

pub trait EndpointResolver: Send + Sync + Debug {
    fn resolve_and_apply_endpoint(
        &self,
        params: &EndpointResolverParams,
        endpoint_prefix: Option<&EndpointPrefix>,
        request: &mut HttpRequest,
    ) -> Result<(), BoxError>;
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

    fn retry_strategy(&self) -> &dyn RetryStrategy;
    fn set_retry_strategy(&mut self, retry_strategy: impl RetryStrategy + 'static);

    fn trace_probe(&self) -> &dyn TraceProbe;
    fn set_trace_probe(&mut self, trace_probe: impl TraceProbe + 'static);
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

    fn http_auth_schemes(&self) -> &HttpAuthSchemes {
        self.get::<HttpAuthSchemes>()
            .expect("auth schemes must be set")
    }

    fn set_http_auth_schemes(&mut self, http_auth_schemes: HttpAuthSchemes) {
        self.put::<HttpAuthSchemes>(http_auth_schemes);
    }

    fn retry_strategy(&self) -> &dyn RetryStrategy {
        &**self
            .get::<Box<dyn RetryStrategy>>()
            .expect("a retry strategy must be set")
    }

    fn set_retry_strategy(&mut self, retry_strategy: impl RetryStrategy + 'static) {
        self.put::<Box<dyn RetryStrategy>>(Box::new(retry_strategy));
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

    fn trace_probe(&self) -> &dyn TraceProbe {
        &**self
            .get::<Box<dyn TraceProbe>>()
            .expect("missing trace probe")
    }

    fn set_trace_probe(&mut self, trace_probe: impl TraceProbe + 'static) {
        self.put::<Box<dyn TraceProbe>>(Box::new(trace_probe));
    }
}

pub mod builders {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IdentityResolversBuilder {
        identity_resolvers: Vec<(&'static str, Box<dyn IdentityResolver>)>,
    }

    impl IdentityResolversBuilder {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn identity_resolver(
            mut self,
            name: &'static str,
            resolver: impl IdentityResolver + 'static,
        ) -> Self {
            self.identity_resolvers
                .push((name, Box::new(resolver) as _));
            self
        }

        pub fn build(self) -> IdentityResolvers {
            IdentityResolvers {
                identity_resolvers: self.identity_resolvers,
            }
        }
    }

    #[derive(Debug, Default)]
    pub struct HttpAuthSchemesBuilder {
        schemes: Vec<(&'static str, Box<dyn HttpAuthScheme>)>,
    }

    impl HttpAuthSchemesBuilder {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn auth_scheme(
            mut self,
            name: &'static str,
            auth_scheme: impl HttpAuthScheme + 'static,
        ) -> Self {
            self.schemes.push((name, Box::new(auth_scheme) as _));
            self
        }

        pub fn build(self) -> HttpAuthSchemes {
            HttpAuthSchemes {
                inner: Arc::new(HttpAuthSchemesInner {
                    schemes: self.schemes,
                }),
            }
        }
    }
}
