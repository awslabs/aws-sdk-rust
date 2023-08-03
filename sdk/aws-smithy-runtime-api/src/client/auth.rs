/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::identity::{Identity, IdentityResolvers, SharedIdentityResolver};
use crate::client::orchestrator::HttpRequest;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreAppend, StoreReplace};
use aws_smithy_types::type_erasure::{TypeErasedBox, TypedBox};
use aws_smithy_types::Document;
use std::borrow::Cow;
use std::fmt;
use std::sync::Arc;

#[cfg(feature = "http-auth")]
pub mod http;

pub mod option_resolver;

/// New type around an auth scheme ID.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

impl From<&'static str> for AuthSchemeId {
    fn from(scheme_id: &'static str) -> Self {
        Self::new(scheme_id)
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

impl Storable for AuthOptionResolverParams {
    type Storer = StoreReplace<Self>;
}

pub trait AuthOptionResolver: Send + Sync + fmt::Debug {
    fn resolve_auth_options(
        &self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError>;
}

#[derive(Debug)]
pub struct DynAuthOptionResolver(Box<dyn AuthOptionResolver>);

impl DynAuthOptionResolver {
    pub fn new(auth_option_resolver: impl AuthOptionResolver + 'static) -> Self {
        Self(Box::new(auth_option_resolver))
    }
}

impl Storable for DynAuthOptionResolver {
    type Storer = StoreReplace<Self>;
}

impl AuthOptionResolver for DynAuthOptionResolver {
    fn resolve_auth_options(
        &self,
        params: &AuthOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError> {
        (*self.0).resolve_auth_options(params)
    }
}

pub trait HttpAuthScheme: Send + Sync + fmt::Debug {
    fn scheme_id(&self) -> AuthSchemeId;

    fn identity_resolver(
        &self,
        identity_resolvers: &IdentityResolvers,
    ) -> Option<SharedIdentityResolver>;

    fn request_signer(&self) -> &dyn HttpRequestSigner;
}

/// Container for a shared HTTP auth scheme implementation.
#[derive(Clone, Debug)]
pub struct SharedHttpAuthScheme(Arc<dyn HttpAuthScheme>);

impl SharedHttpAuthScheme {
    /// Creates a new [`SharedHttpAuthScheme`] from the given auth scheme.
    pub fn new(auth_scheme: impl HttpAuthScheme + 'static) -> Self {
        Self(Arc::new(auth_scheme))
    }
}

impl HttpAuthScheme for SharedHttpAuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        self.0.scheme_id()
    }

    fn identity_resolver(
        &self,
        identity_resolvers: &IdentityResolvers,
    ) -> Option<SharedIdentityResolver> {
        self.0.identity_resolver(identity_resolvers)
    }

    fn request_signer(&self) -> &dyn HttpRequestSigner {
        self.0.request_signer()
    }
}

impl Storable for SharedHttpAuthScheme {
    type Storer = StoreAppend<Self>;
}

pub trait HttpRequestSigner: Send + Sync + fmt::Debug {
    /// Return a signed version of the given request using the given identity.
    ///
    /// If the provided identity is incompatible with this signer, an error must be returned.
    fn sign_request(
        &self,
        request: &mut HttpRequest,
        identity: &Identity,
        auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        config_bag: &ConfigBag,
    ) -> Result<(), BoxError>;
}

/// Endpoint configuration for the selected auth scheme.
///
/// This struct gets added to the request state by the auth orchestrator.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct AuthSchemeEndpointConfig<'a>(Option<&'a Document>);

impl<'a> AuthSchemeEndpointConfig<'a> {
    /// Creates a new [`AuthSchemeEndpointConfig`].
    pub fn new(config: Option<&'a Document>) -> Self {
        Self(config)
    }

    /// Creates an empty AuthSchemeEndpointConfig.
    pub fn empty() -> Self {
        Self(None)
    }

    pub fn config(&self) -> Option<&'a Document> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_types::config_bag::{ConfigBag, Layer};

    #[test]
    fn test_shared_http_auth_scheme_configuration() {
        #[derive(Debug)]
        struct TestHttpAuthScheme(&'static str);
        impl HttpAuthScheme for TestHttpAuthScheme {
            fn scheme_id(&self) -> AuthSchemeId {
                AuthSchemeId::new(self.0)
            }

            fn identity_resolver(&self, _: &IdentityResolvers) -> Option<SharedIdentityResolver> {
                unreachable!("this shouldn't get called in this test")
            }

            fn request_signer(&self) -> &dyn HttpRequestSigner {
                unreachable!("this shouldn't get called in this test")
            }
        }

        let mut config_bag = ConfigBag::base();

        let mut layer = Layer::new("first");
        layer.store_append(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_1")));
        config_bag.push_layer(layer);

        let mut layer = Layer::new("second");
        layer.store_append(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_2")));
        layer.store_append(SharedHttpAuthScheme::new(TestHttpAuthScheme("scheme_3")));
        config_bag.push_layer(layer);

        let auth_schemes = config_bag.load::<SharedHttpAuthScheme>();
        let encountered_scheme_ids: Vec<AuthSchemeId> =
            auth_schemes.map(|s| s.scheme_id()).collect();

        assert_eq!(
            vec![
                AuthSchemeId::new("scheme_3"),
                AuthSchemeId::new("scheme_2"),
                AuthSchemeId::new("scheme_1")
            ],
            encountered_scheme_ids
        );
    }
}
