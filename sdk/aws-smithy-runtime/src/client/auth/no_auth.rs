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
use aws_smithy_runtime_api::client::identity::{Identity, SharedIdentityResolver};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_components::{
    GetIdentityResolver, RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_types::config_bag::ConfigBag;
use std::borrow::Cow;

pub const NO_AUTH_SCHEME_ID: AuthSchemeId = AuthSchemeId::new("no_auth");

/// A [`RuntimePlugin`] that registers a "no auth" identity resolver and auth scheme.
///
/// This plugin can be used to disable authentication in certain cases, such as when there is
/// a Smithy `@optionalAuth` trait.
#[non_exhaustive]
#[derive(Debug)]
pub struct NoAuthRuntimePlugin(RuntimeComponentsBuilder);

impl Default for NoAuthRuntimePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl NoAuthRuntimePlugin {
    pub fn new() -> Self {
        Self(
            RuntimeComponentsBuilder::new("NoAuthRuntimePlugin")
                .with_identity_resolver(
                    NO_AUTH_SCHEME_ID,
                    SharedIdentityResolver::new(NoAuthIdentityResolver::new()),
                )
                .with_http_auth_scheme(SharedHttpAuthScheme::new(NoAuthScheme::new())),
        )
    }
}

impl RuntimePlugin for NoAuthRuntimePlugin {
    fn runtime_components(&self) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&self.0)
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
        _runtime_components: &RuntimeComponents,
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
        identity_resolvers: &dyn GetIdentityResolver,
    ) -> Option<SharedIdentityResolver> {
        identity_resolvers.identity_resolver(NO_AUTH_SCHEME_ID)
    }

    fn request_signer(&self) -> &dyn HttpRequestSigner {
        &self.signer
    }
}
