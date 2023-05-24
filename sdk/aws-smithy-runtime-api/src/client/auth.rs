/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::identity::{Identity, IdentityResolver, IdentityResolvers};
use crate::client::orchestrator::{BoxError, HttpRequest};
use crate::config_bag::ConfigBag;
use crate::type_erasure::{TypeErasedBox, TypedBox};
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

#[cfg(feature = "http-auth")]
pub mod http;

pub mod option_resolver;

/// New type around an auth scheme ID.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct AuthSchemeId {
    scheme_id: &'static str,
}

impl AuthSchemeId {
    /// Creates a new auth scheme ID.
    pub const fn new(scheme_id: &'static str) -> Self {
        Self { scheme_id }
    }

    /// Returns the string equivalent of this auth scheme ID.
    pub const fn as_str(&self) -> &'static str {
        self.scheme_id
    }
}

#[derive(Debug)]
pub struct AuthOptionResolverParams(TypeErasedBox);

impl AuthOptionResolverParams {
    pub fn new<T: fmt::Debug + Send + Sync + 'static>(params: T) -> Self {
        Self(TypedBox::new(params).erase())
    }

    pub fn get<T: fmt::Debug + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

pub trait AuthOptionResolver: Send + Sync + fmt::Debug {
    fn resolve_auth_options(
        &self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError>;
}

impl AuthOptionResolver for Box<dyn AuthOptionResolver> {
    fn resolve_auth_options(
        &self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError> {
        (**self).resolve_auth_options(params)
    }
}

#[derive(Debug)]
struct HttpAuthSchemesInner {
    schemes: Vec<(AuthSchemeId, Box<dyn HttpAuthScheme>)>,
}
#[derive(Debug)]
pub struct HttpAuthSchemes {
    inner: Arc<HttpAuthSchemesInner>,
}

impl HttpAuthSchemes {
    pub fn builder() -> builders::HttpAuthSchemesBuilder {
        Default::default()
    }

    pub fn scheme(&self, scheme_id: AuthSchemeId) -> Option<&dyn HttpAuthScheme> {
        self.inner
            .schemes
            .iter()
            .find(|scheme| scheme.0 == scheme_id)
            .map(|scheme| &*scheme.1)
    }
}

pub trait HttpAuthScheme: Send + Sync + fmt::Debug {
    fn scheme_id(&self) -> AuthSchemeId;

    fn identity_resolver<'a>(
        &self,
        identity_resolvers: &'a IdentityResolvers,
    ) -> Option<&'a dyn IdentityResolver>;

    fn request_signer(&self) -> &dyn HttpRequestSigner;
}

pub trait HttpRequestSigner: Send + Sync + fmt::Debug {
    /// Return a signed version of the given request using the given identity.
    ///
    /// If the provided identity is incompatible with this signer, an error must be returned.
    fn sign_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        config_bag: &ConfigBag,
    ) -> Result<(), BoxError>;
}

pub mod builders {
    use super::*;

    #[derive(Debug, Default)]
    pub struct HttpAuthSchemesBuilder {
        schemes: Vec<(AuthSchemeId, Box<dyn HttpAuthScheme>)>,
    }

    impl HttpAuthSchemesBuilder {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn auth_scheme(
            mut self,
            scheme_id: AuthSchemeId,
            auth_scheme: impl HttpAuthScheme + 'static,
        ) -> Self {
            self.schemes.push((scheme_id, Box::new(auth_scheme) as _));
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
