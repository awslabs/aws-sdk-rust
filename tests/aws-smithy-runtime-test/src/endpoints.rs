/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime::{BoxError, EndpointOrchestrator};
use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugin;

#[derive(Debug)]
pub struct GetObjectEndpointOrc {}

impl GetObjectEndpointOrc {
    pub fn new() -> Self {
        Self {}
    }
}

impl RuntimePlugin for GetObjectEndpointOrc {
    fn configure(&self, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
        todo!()
    }
}

impl EndpointOrchestrator<http::Request<SdkBody>> for GetObjectEndpointOrc {
    fn resolve_and_apply_endpoint(
        &self,
        _req: &mut http::Request<SdkBody>,
        _cfg: &ConfigBag,
    ) -> Result<(), BoxError> {
        todo!()
        // let endpoint = endpoint_resolver.resolve_endpoint(&endpoint_parameters)?;
        // let (tx_req, props) = ctx
        //     .tx_request_mut()
        //     .expect("We call this after setting the tx request");
        //
        // // Apply the endpoint
        // let uri: Uri = endpoint.url().parse().map_err(|err| {
        //     ResolveEndpointError::from_source("endpoint did not have a valid uri", err)
        // })?;
        // apply_endpoint(tx_req.uri_mut(), &uri, props.get::<EndpointPrefix>()).map_err(|err| {
        //     ResolveEndpointError::message(format!(
        //         "failed to apply endpoint `{:?}` to request `{:?}`",
        //         uri, tx_req
        //     ))
        //         .with_source(Some(err.into()))
        // })?;
        // for (header_name, header_values) in endpoint.headers() {
        //     tx_req.headers_mut().remove(header_name);
        //     for value in header_values {
        //         tx_req.headers_mut().insert(
        //             HeaderName::from_str(header_name).map_err(|err| {
        //                 ResolveEndpointError::message("invalid header name")
        //                     .with_source(Some(err.into()))
        //             })?,
        //             HeaderValue::from_str(value).map_err(|err| {
        //                 ResolveEndpointError::message("invalid header value")
        //                     .with_source(Some(err.into()))
        //             })?,
        //         );
        //     }
        // }
    }

    fn resolve_auth_schemes(&self) -> Result<Vec<String>, BoxError> {
        todo!()

        // let endpoint = endpoint_resolver
        //     .resolve_endpoint(params)
        //     .map_err(SdkError::construction_failure)?;
        // let auth_schemes = match endpoint.properties().get("authSchemes") {
        //     Some(Document::Array(schemes)) => schemes,
        //     None => {
        //         return Ok(vec![]);
        //     }
        //     _other => {
        //         return Err(SdkError::construction_failure(
        //             "expected bad things".to_string(),
        //         ));
        //     }
        // };
        // let auth_schemes = auth_schemes
        //     .iter()
        //     .flat_map(|doc| match doc {
        //         Document::Object(map) => Some(map),
        //         _ => None,
        //     })
        //     .map(|it| {
        //         let name = match it.get("name") {
        //             Some(Document::String(s)) => Some(s.as_str()),
        //             _ => None,
        //         };
        //         AuthSchemeOptions::new(
        //             name.unwrap().to_string(),
        //             /* there are no identity properties yet */
        //             None,
        //             Some(Document::Object(it.clone())),
        //         )
        //     })
        //     .collect::<Vec<_>>();
        // Ok(auth_schemes)
    }
}
