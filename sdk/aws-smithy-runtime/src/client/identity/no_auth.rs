/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::identity::{Identity, IdentityResolver};
use aws_smithy_runtime_api::client::orchestrator::Future;
use aws_smithy_types::config_bag::ConfigBag;

/// Identity for the [`NoAuthScheme`](crate::client::auth::no_auth::NoAuthScheme) auth scheme.
#[derive(Debug, Default)]
pub struct NoAuthIdentity;

impl NoAuthIdentity {
    /// Creates a new `NoAuthIdentity`.
    pub fn new() -> Self {
        Self
    }
}

/// Identity resolver for the [`NoAuthScheme`](crate::client::auth::no_auth::NoAuthScheme) auth scheme.
#[derive(Debug, Default)]
pub struct NoAuthIdentityResolver;

impl NoAuthIdentityResolver {
    /// Creates a new `NoAuthIdentityResolver`.
    pub fn new() -> Self {
        Self
    }
}

impl IdentityResolver for NoAuthIdentityResolver {
    fn resolve_identity(&self, _: &ConfigBag) -> Future<Identity> {
        Future::ready(Ok(Identity::new(NoAuthIdentity::new(), None)))
    }
}
