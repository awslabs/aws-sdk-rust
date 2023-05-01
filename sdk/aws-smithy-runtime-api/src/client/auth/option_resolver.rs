/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{
    AuthOptionResolver, AuthOptionResolverParams, BoxError, HttpAuthOption,
};

#[derive(Debug)]
pub struct StubAuthOptionResolver {}

impl StubAuthOptionResolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl AuthOptionResolver for StubAuthOptionResolver {
    fn resolve_auth_options(
        &self,
        _params: &AuthOptionResolverParams,
    ) -> Result<Vec<HttpAuthOption>, BoxError> {
        Ok(Vec::new())
    }
}

pub struct StubAuthOptionResolverParams {}

impl StubAuthOptionResolverParams {
    pub fn new() -> Self {
        Self {}
    }
}
