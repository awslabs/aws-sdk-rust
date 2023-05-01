/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{BoxError, EndpointResolver, HttpRequest};
use aws_smithy_http::endpoint::apply_endpoint;
use http::Uri;
use std::fmt::Debug;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct StaticUriEndpointResolver {
    endpoint: Uri,
}

impl StaticUriEndpointResolver {
    pub fn http_localhost(port: u16) -> Self {
        Self {
            endpoint: Uri::from_str(&format!("http://localhost:{port}"))
                .expect("all u16 values are valid ports"),
        }
    }

    pub fn uri(endpoint: Uri) -> Self {
        Self { endpoint }
    }
}

impl EndpointResolver for StaticUriEndpointResolver {
    fn resolve_and_apply_endpoint(&self, request: &mut HttpRequest) -> Result<(), BoxError> {
        apply_endpoint(request.uri_mut(), &self.endpoint, None)?;
        Ok(())
    }
}
