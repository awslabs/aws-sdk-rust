/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(clippy::derive_partial_eq_without_eq)]

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use aws_smithy_http::endpoint::error::ResolveEndpointError;
use aws_smithy_http::endpoint::ResolveEndpoint;
use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_types::endpoint::Endpoint as SmithyEndpoint;
use aws_smithy_types::Document;

pub use aws_types::endpoint::{AwsEndpoint, BoxError, CredentialScope, ResolveAwsEndpoint};
use aws_types::region::{Region, SigningRegion};
use aws_types::SigningService;

#[doc(hidden)]
pub struct Params {
    region: Option<Region>,
}

impl Params {
    pub fn new(region: Option<Region>) -> Self {
        Self { region }
    }
}

#[doc(hidden)]
pub struct EndpointShim(Arc<dyn ResolveAwsEndpoint>);
impl EndpointShim {
    pub fn from_resolver(resolver: impl ResolveAwsEndpoint + 'static) -> Self {
        Self(Arc::new(resolver))
    }

    pub fn from_arc(arc: Arc<dyn ResolveAwsEndpoint>) -> Self {
        Self(arc)
    }
}

impl<T> ResolveEndpoint<T> for EndpointShim
where
    T: Clone + Into<Params>,
{
    fn resolve_endpoint(&self, params: &T) -> Result<SmithyEndpoint, ResolveEndpointError> {
        let params: Params = params.clone().into();
        let aws_endpoint = self
            .0
            .resolve_endpoint(
                params
                    .region
                    .as_ref()
                    .ok_or_else(|| ResolveEndpointError::message("no region in params"))?,
            )
            .map_err(|err| {
                ResolveEndpointError::message("failure resolving endpoint").with_source(Some(err))
            })?;
        let uri = aws_endpoint.endpoint().uri();
        let mut auth_scheme =
            HashMap::from([("name".to_string(), Document::String("sigv4".into()))]);
        if let Some(region) = aws_endpoint.credential_scope().region() {
            auth_scheme.insert(
                "signingRegion".to_string(),
                region.as_ref().to_string().into(),
            );
        }
        if let Some(service) = aws_endpoint.credential_scope().service() {
            auth_scheme.insert(
                "signingName".to_string(),
                service.as_ref().to_string().into(),
            );
        }
        Ok(SmithyEndpoint::builder()
            .url(uri.to_string())
            .property("authSchemes", vec![Document::Object(auth_scheme)])
            .build())
    }
}

/// Middleware Stage to add authentication information from a Smithy endpoint into the property bag
///
/// AwsAuthStage implements [`MapRequest`](MapRequest). It will:
/// 1. Load an endpoint from the property bag
/// 2. Set the `SigningRegion` and `SigningService` in the property bag to drive downstream
/// signing middleware.
#[derive(Clone, Debug)]
pub struct AwsAuthStage;

#[derive(Debug)]
enum AwsAuthStageErrorKind {
    NoEndpointResolver,
    EndpointResolutionError(BoxError),
}

#[derive(Debug)]
pub struct AwsAuthStageError {
    kind: AwsAuthStageErrorKind,
}

impl fmt::Display for AwsAuthStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AwsAuthStageErrorKind::*;
        match &self.kind {
            NoEndpointResolver => write!(f, "endpoint resolution failed: no endpoint present"),
            EndpointResolutionError(_) => write!(f, "endpoint resolution failed"),
        }
    }
}

impl Error for AwsAuthStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use AwsAuthStageErrorKind::*;
        match &self.kind {
            EndpointResolutionError(source) => Some(source.as_ref() as _),
            NoEndpointResolver => None,
        }
    }
}

impl From<AwsAuthStageErrorKind> for AwsAuthStageError {
    fn from(kind: AwsAuthStageErrorKind) -> Self {
        Self { kind }
    }
}

impl MapRequest for AwsAuthStage {
    type Error = AwsAuthStageError;

    fn name(&self) -> &'static str {
        "resolve_endpoint"
    }

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|http_req, props| {
            let endpoint = props
                .get::<aws_smithy_types::endpoint::Endpoint>()
                .ok_or(AwsAuthStageErrorKind::NoEndpointResolver)?;
            let (signing_scope_override, signing_service_override) = smithy_to_aws(endpoint)
                .map_err(|err| AwsAuthStageErrorKind::EndpointResolutionError(err))?;

            if let Some(signing_scope) = signing_scope_override {
                props.insert(signing_scope);
            }
            if let Some(signing_service) = signing_service_override {
                props.insert(signing_service);
            }
            Ok(http_req)
        })
    }
}

type EndpointMetadata = (Option<SigningRegion>, Option<SigningService>);

fn smithy_to_aws(value: &SmithyEndpoint) -> Result<EndpointMetadata, Box<dyn Error + Send + Sync>> {
    // look for v4 as an auth scheme
    let auth_schemes = match value.properties().get("authSchemes") {
        Some(Document::Array(schemes)) => schemes,
        // no auth schemes:
        None => return Ok((None, None)),
        _other => return Err("expected an array for authSchemes".into()),
    };
    let auth_schemes = auth_schemes
        .iter()
        .flat_map(|doc| match doc {
            Document::Object(map) => Some(map),
            _ => None,
        })
        .map(|it| {
            let name = match it.get("name") {
                Some(Document::String(s)) => Some(s.as_str()),
                _ => None,
            };
            (name, it)
        });
    let (_, v4) = auth_schemes
        .clone()
        .find(|(name, _doc)| name.as_deref() == Some("sigv4"))
        .ok_or_else(|| {
            format!(
                "No auth schemes were supported. The Rust SDK only supports sigv4. \
                The authentication schemes supported by this endpoint were: {:?}",
                auth_schemes.flat_map(|(name, _)| name).collect::<Vec<_>>()
            )
        })?;

    let signing_scope = match v4.get("signingRegion") {
        Some(Document::String(s)) => Some(SigningRegion::from(Region::new(s.clone()))),
        None => None,
        _ => return Err("unexpected type".into()),
    };
    let signing_service = match v4.get("signingName") {
        Some(Document::String(s)) => Some(SigningService::from(s.to_string())),
        None => None,
        _ => return Err("unexpected type".into()),
    };
    Ok((signing_scope, signing_service))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use aws_smithy_types::endpoint::Endpoint;
    use aws_smithy_types::Document;
    use http::header::HOST;

    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;

    use crate::AwsAuthStage;

    #[test]
    fn default_endpoint_updates_request() {
        let endpoint = Endpoint::builder()
            .url("kinesis.us-east-1.amazon.com")
            .build();
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(SigningRegion::from(region.clone()));
            props.insert(SigningService::from_static("kinesis"));
            props.insert(endpoint);
        };
        let req = AwsAuthStage.apply(req).expect("should succeed");
        assert_eq!(req.properties().get(), Some(&SigningRegion::from(region)));
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("kinesis"))
        );

        assert!(req.http().headers().get(HOST).is_none());
        assert!(
            req.properties().get::<Endpoint>().is_some(),
            "Endpoint middleware MUST leave the result in the bag"
        );
    }

    #[test]
    fn sets_service_override_when_set() {
        let endpoint = Endpoint::builder()
            .url("kinesis.us-east-override.amazon.com")
            .property(
                "authSchemes",
                vec![Document::Object({
                    let mut out = HashMap::new();
                    out.insert("name".to_string(), "sigv4".to_string().into());
                    out.insert(
                        "signingName".to_string(),
                        "qldb-override".to_string().into(),
                    );
                    out.insert(
                        "signingRegion".to_string(),
                        "us-east-override".to_string().into(),
                    );
                    out
                })],
            )
            .build();
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(region);
            props.insert(SigningService::from_static("qldb"));
            props.insert(endpoint);
        };
        let req = AwsAuthStage.apply(req).expect("should succeed");
        assert_eq!(
            req.properties().get(),
            Some(&SigningRegion::from(Region::new("us-east-override")))
        );
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("qldb-override"))
        );
    }

    #[test]
    fn supports_fallback_when_scope_is_unset() {
        let endpoint = Endpoint::builder().url("www.service.com").build();
        let req = http::Request::new(SdkBody::from(""));
        let region = SigningRegion::from_static("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(region.clone());
            props.insert(SigningService::from_static("qldb"));
            props.insert(endpoint);
        };
        let req = AwsAuthStage.apply(req).expect("should succeed");
        assert_eq!(req.properties().get(), Some(&region));
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("qldb"))
        );
    }
}
