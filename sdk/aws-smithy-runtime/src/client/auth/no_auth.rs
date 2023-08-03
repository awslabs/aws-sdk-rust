/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! The [`NoAuthRuntimePlugin`] and supporting code.

use crate::client::identity::no_auth::NoAuthIdentityResolver;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::{
    AuthSchemeEndpointConfig, AuthSchemeId, HttpAuthScheme, HttpRequestSigner, SharedHttpAuthScheme,
};
use aws_smithy_runtime_api::client::config_bag_accessors::ConfigBagAccessors;
use aws_smithy_runtime_api::client::identity::{
    Identity, IdentityResolvers, SharedIdentityResolver,
};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};

pub const NO_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("no_auth");

/// A [`RuntimePlugin`] that registers a "no auth" identity resolver and auth scheme.
///
/// This plugin can be used to disable authentication in certain cases, such as when there is
/// a Smithy `@optionalAuth` trait.
#[non_exhaustive]
#[derive(Debug)]
pub struct NoAuthRuntimePlugin(FrozenLayer);

impl Default for NoAuthRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl NoAuthRuntimePlugin {
    pub fn new() -> Self {
        let mut cfg = Layer::new("NoAuth");
        cfg.push_identity_resolver(
            NO_AUTH_SCHEME_ID,
            SharedIdentityResolver::new(NoAuthIdentityResolver::new()),
        );
        cfg.push_http_auth_scheme(SharedHttpAuthScheme::new(NoAuthScheme::new()));
        Self(cfg.freeze())
    }
}

impl RuntimePlugin for NoAuthRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        Some(self.0.clone())
    }
}

#[derive(Debug, Default)]
pub struct NoAuthScheme {
    signer: NoAuthSigner,
}

impl NoAuthScheme {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
struct NoAuthSigner;

impl HttpRequestSigner for NoAuthSigner {
    fn sign_request(
        &self,
        _request: &mut HttpRequest,
        _identity: &Identity,
        _auth_scheme_endpoint_config: AuthSchemeEndpointConfig<'_>,
        _config_bag: &ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
}

impl HttpAuthScheme for NoAuthScheme {
    fn scheme_id(&self) -> AuthSchemeId {
        NO_AUTH_SCHEME_ID
    }

    fn identity_resolver(
        &self,
        identity_resolvers: &IdentityResolvers,
    ) -> Option<SharedIdentityResolver> {
        identity_resolvers.identity_resolver(NO_AUTH_SCHEME_ID)
    }

    fn request_signer(&self) -> &dyn HttpRequestSigner {
        &self.signer
    }
}
