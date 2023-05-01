/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::orchestrator::{
    BoxError, EndpointResolver, EndpointResolverParams, HttpRequest,
};
use aws_smithy_http::endpoint::error::ResolveEndpointError;
use aws_smithy_http::endpoint::{
    apply_endpoint, EndpointPrefix, ResolveEndpoint, SharedEndpointResolver,
};
use http::header::HeaderName;
use http::{HeaderValue, Uri};
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
    fn resolve_and_apply_endpoint(
        &self,
        _params: &EndpointResolverParams,
        _endpoint_prefix: Option<&EndpointPrefix>,
        request: &mut HttpRequest,
    ) -> Result<(), BoxError> {
        apply_endpoint(request.uri_mut(), &self.endpoint, None)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DefaultEndpointResolver<Params> {
    inner: SharedEndpointResolver<Params>,
}

impl<Params> DefaultEndpointResolver<Params> {
    pub fn new(resolve_endpoint: SharedEndpointResolver<Params>) -> Self {
        Self {
            inner: resolve_endpoint,
        }
    }
}

impl<Params> EndpointResolver for DefaultEndpointResolver<Params>
where
    Params: Debug + Send + Sync + 'static,
{
    fn resolve_and_apply_endpoint(
        &self,
        params: &EndpointResolverParams,
        endpoint_prefix: Option<&EndpointPrefix>,
        request: &mut HttpRequest,
    ) -> Result<(), BoxError> {
        let endpoint = match params.get::<Params>() {
            Some(params) => self.inner.resolve_endpoint(params)?,
            None => {
                return Err(Box::new(ResolveEndpointError::message(
                    "params of expected type was not present",
                )));
            }
        };

        let uri: Uri = endpoint.url().parse().map_err(|err| {
            ResolveEndpointError::from_source("endpoint did not have a valid uri", err)
        })?;

        apply_endpoint(request.uri_mut(), &uri, endpoint_prefix).map_err(|err| {
            ResolveEndpointError::message(format!(
                "failed to apply endpoint `{:?}` to request `{:?}`",
                uri, request,
            ))
            .with_source(Some(err.into()))
        })?;

        for (header_name, header_values) in endpoint.headers() {
            request.headers_mut().remove(header_name);
            for value in header_values {
                request.headers_mut().insert(
                    HeaderName::from_str(header_name).map_err(|err| {
                        ResolveEndpointError::message("invalid header name")
                            .with_source(Some(err.into()))
                    })?,
                    HeaderValue::from_str(value).map_err(|err| {
                        ResolveEndpointError::message("invalid header value")
                            .with_source(Some(err.into()))
                    })?,
                );
            }
        }

        Ok(())
    }
}
