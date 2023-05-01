/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{BoxError, EndpointResolver, HttpRequest};
use http::Uri;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct StaticUriEndpointResolver {
    uri: Uri,
}

impl StaticUriEndpointResolver {
    pub fn localhost(port: u16) -> Self {
        Self {
            uri: Uri::from_str(&format!("https://localhost:{port}"))
                .expect("all u16 values are valid ports"),
        }
    }

    pub fn uri(uri: Uri) -> Self {
        Self { uri }
    }
}

impl Default for StaticUriEndpointResolver {
    fn default() -> Self {
        StaticUriEndpointResolver::localhost(3000)
    }
}

impl EndpointResolver for StaticUriEndpointResolver {
    fn resolve_and_apply_endpoint(&self, request: &mut HttpRequest) -> Result<(), BoxError> {
        *request.uri_mut() = self.uri.clone();

        Ok(())
    }
}
