/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::endpoint::error::ResolveEndpointError;
use aws_smithy_http::endpoint::{
    apply_endpoint as apply_endpoint_to_request_uri, EndpointPrefix, ResolveEndpoint,
    SharedEndpointResolver,
};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::endpoint::{EndpointResolver, EndpointResolverParams};
use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{Future, HttpRequest};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::endpoint::Endpoint;
use http::header::HeaderName;
use http::{HeaderValue, Uri};
use std::fmt::Debug;
use std::str::FromStr;
use tracing::trace;

/// An endpoint resolver that uses a static URI.
#[derive(Clone, Debug)]
pub struct StaticUriEndpointResolver {
    endpoint: Uri,
}

impl StaticUriEndpointResolver {
    /// Create a resolver that resolves to `http://localhost:{port}`.
    pub fn http_localhost(port: u16) -> Self {
        Self {
            endpoint: Uri::from_str(&format!("http://localhost:{port}"))
                .expect("all u16 values are valid ports"),
        }
    }

    /// Create a resolver that resolves to the given URI.
    pub fn uri(endpoint: Uri) -> Self {
        Self { endpoint }
    }
}

impl EndpointResolver for StaticUriEndpointResolver {
    fn resolve_endpoint(&self, _params: &EndpointResolverParams) -> Future<Endpoint> {
        Future::ready(Ok(Endpoint::builder()
            .url(self.endpoint.to_string())
            .build()))
    }
}

/// Empty params to be used with [`StaticUriEndpointResolver`].
#[derive(Debug, Default)]
pub struct StaticUriEndpointResolverParams;

impl StaticUriEndpointResolverParams {
    /// Creates a new `StaticUriEndpointResolverParams`.
    pub fn new() -> Self {
        Self
    }
}

impl From<StaticUriEndpointResolverParams> for EndpointResolverParams {
    fn from(params: StaticUriEndpointResolverParams) -> Self {
        EndpointResolverParams::new(params)
    }
}

/// Default implementation of [`EndpointResolver`].
///
/// This default endpoint resolver implements the `EndpointResolver` trait by
/// converting the type-erased [`EndpointResolverParams`] into the concrete
/// endpoint params for the service. It then delegates endpoint resolution
/// to an underlying resolver that is aware of the concrete type.
#[derive(Clone, Debug)]
pub struct DefaultEndpointResolver<Params> {
    inner: SharedEndpointResolver<Params>,
}

impl<Params> Storable for DefaultEndpointResolver<Params>
where
    Params: Debug + Send + Sync + 'static,
{
    type Storer = StoreReplace<Self>;
}

impl<Params> DefaultEndpointResolver<Params> {
    /// Creates a new `DefaultEndpointResolver`.
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
    fn resolve_endpoint(&self, params: &EndpointResolverParams) -> Future<Endpoint> {
        let ep = match params.get::<Params>() {
            Some(params) => self.inner.resolve_endpoint(params).map_err(Box::new),
            None => Err(Box::new(ResolveEndpointError::message(
                "params of expected type was not present",
            ))),
        }
        .map_err(|e| e as _);
        Future::ready(ep)
    }
}

pub(super) async fn orchestrate_endpoint(
    ctx: &mut InterceptorContext,
    runtime_components: &RuntimeComponents,
    cfg: &mut ConfigBag,
) -> Result<(), BoxError> {
    trace!("orchestrating endpoint resolution");

    let params = cfg
        .load::<EndpointResolverParams>()
        .expect("endpoint resolver params must be set");
    let endpoint_prefix = cfg.load::<EndpointPrefix>();
    let request = ctx.request_mut().expect("set during serialization");

    let endpoint = runtime_components
        .endpoint_resolver()
        .resolve_endpoint(params)
        .await?;
    tracing::debug!("will use endpoint {:?}", endpoint);
    apply_endpoint(request, &endpoint, endpoint_prefix)?;

    // Make the endpoint config available to interceptors
    cfg.interceptor_state().store_put(endpoint);
    Ok(())
}

fn apply_endpoint(
    request: &mut HttpRequest,
    endpoint: &Endpoint,
    endpoint_prefix: Option<&EndpointPrefix>,
) -> Result<(), BoxError> {
    let uri: Uri = endpoint.url().parse().map_err(|err| {
        ResolveEndpointError::from_source("endpoint did not have a valid uri", err)
    })?;

    apply_endpoint_to_request_uri(request.uri_mut(), &uri, endpoint_prefix).map_err(|err| {
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
