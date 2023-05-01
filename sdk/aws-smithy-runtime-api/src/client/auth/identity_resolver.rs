/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::identity::Identity;
use crate::client::orchestrator::{BoxError, IdentityResolver};
use crate::config_bag::ConfigBag;

#[derive(Debug)]
pub struct AnonymousIdentity {}

impl AnonymousIdentity {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct StubIdentityResolver {}

impl IdentityResolver for StubIdentityResolver {
    fn resolve_identity(&self, _cfg: &ConfigBag) -> Result<Identity, BoxError> {
        Ok(Identity::new(AnonymousIdentity::new(), None))
    }
}
