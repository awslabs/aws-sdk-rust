/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! [`MapRequest`]-based middleware for resolving and applying a request's endpoint.

use crate::endpoint;
use crate::endpoint::{apply_endpoint, EndpointPrefix, ResolveEndpointError};
use crate::middleware::MapRequest;
use crate::operation::Request;
use http::header::HeaderName;
use http::{HeaderValue, Uri};
use std::str::FromStr;

// TODO(enableNewSmithyRuntimeCleanup): Delete this module

/// Middleware to apply an HTTP endpoint to the request
///
/// This middleware reads [`aws_smithy_types::endpoint::Endpoint`] out of the request properties and applies
/// it to the HTTP request.
#[non_exhaustive]
#[derive(Default, Debug, Clone)]
pub struct SmithyEndpointStage;
impl SmithyEndpointStage {
    /// Create a new `SmithyEndpointStage`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl MapRequest for SmithyEndpointStage {
    type Error = ResolveEndpointError;

    fn name(&self) -> &'static str {
        "resolve_endpoint"
    }

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|mut http_req, props| {
            // we need to do a little dance so that this works with retries.
            // the first pass through, we convert the result into just an endpoint, early returning
            // the error. Put the endpoint back in the bag in case this request gets retried.
            //
            // the next pass through, there is no result, so in that case, we'll look for the
            // endpoint directly.
            //
            // In an ideal world, we would do this in make_operation, but it's much easier for
            // certain protocol tests if we allow requests with invalid endpoint to be constructed.
            if let Some(endpoint) = props.remove::<endpoint::Result>().transpose()? {
                props.insert(endpoint);
            };
            let endpoint = props.get::<aws_smithy_types::endpoint::Endpoint>();
            let endpoint =
                endpoint.ok_or_else(|| ResolveEndpointError::message("no endpoint present"))?;

            let uri: Uri = endpoint.url().parse().map_err(|err| {
                ResolveEndpointError::from_source("endpoint did not have a valid uri", err)
            })?;
            apply_endpoint(http_req.uri_mut(), &uri, props.get::<EndpointPrefix>()).map_err(
                |err| {
                    ResolveEndpointError::message(format!(
                        "failed to apply endpoint `{:?}` to request `{:?}`",
                        uri, http_req
                    ))
                    .with_source(Some(err.into()))
                },
            )?;
            for (header_name, header_values) in endpoint.headers() {
                http_req.headers_mut().remove(header_name);
                for value in header_values {
                    http_req.headers_mut().insert(
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
            Ok(http_req)
        })
    }
}
