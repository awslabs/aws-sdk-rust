/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::auth::{AuthOptionResolver, AuthOptionResolverParams, AuthSchemeId};
use std::borrow::Cow;

/// New-type around a `Vec<HttpAuthOption>` that implements `AuthOptionResolver`.
///
/// This is useful for clients that don't require `AuthOptionResolverParams` to resolve auth options.
#[derive(Debug)]
pub struct StaticAuthOptionResolver {
    auth_options: Vec<AuthSchemeId>,
}

impl StaticAuthOptionResolver {
    /// Creates a new instance of `StaticAuthOptionResolver`.
    pub fn new(auth_options: Vec<AuthSchemeId>) -> Self {
        Self { auth_options }
    }
}

impl AuthOptionResolver for StaticAuthOptionResolver {
    fn resolve_auth_options(
        &self,
        _params: &AuthOptionResolverParams,
    ) -> Result<Cow<'_, [AuthSchemeId]>, BoxError> {
        Ok(Cow::Borrowed(&self.auth_options))
    }
}

/// Empty params to be used with [`StaticAuthOptionResolver`].
#[derive(Debug)]
pub struct StaticAuthOptionResolverParams;

impl StaticAuthOptionResolverParams {
    /// Creates a new `StaticAuthOptionResolverParams`.
    pub fn new() -> Self {
        Self
    }
}

impl From<StaticAuthOptionResolverParams> for AuthOptionResolverParams {
    fn from(params: StaticAuthOptionResolverParams) -> Self {
        AuthOptionResolverParams::new(params)
    }
}
