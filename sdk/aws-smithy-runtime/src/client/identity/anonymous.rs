/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::identity::{Identity, IdentityResolver};
use aws_smithy_runtime_api::client::orchestrator::Future;
use aws_smithy_runtime_api::config_bag::ConfigBag;

#[derive(Debug, Default)]
pub struct AnonymousIdentity;

impl AnonymousIdentity {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Default)]
pub struct AnonymousIdentityResolver;

impl AnonymousIdentityResolver {
    pub fn new() -> Self {
        Self
    }
}

impl IdentityResolver for AnonymousIdentityResolver {
    fn resolve_identity(&self, _: &ConfigBag) -> Future<Identity> {
        Future::ready(Ok(Identity::new(AnonymousIdentity::new(), None)))
    }
}
