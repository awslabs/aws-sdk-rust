/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[doc(hidden)]
pub mod partition;

#[doc(hidden)]
pub use partition::Partition;
#[doc(hidden)]
pub use partition::PartitionResolver;
use std::collections::HashMap;

use aws_smithy_http::endpoint::error::ResolveEndpointError;
use aws_smithy_http::endpoint::{apply_endpoint, EndpointPrefix, ResolveEndpoint};
use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_types::endpoint::Endpoint as SmithyEndpoint;
use aws_smithy_types::Document;
use aws_types::region::{Region, SigningRegion};
use aws_types::SigningService;
use http::header::HeaderName;
use http::{HeaderValue, Uri};
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

pub use aws_types::endpoint::{AwsEndpoint, BoxError, CredentialScope, ResolveAwsEndpoint};

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

impl ResolveEndpoint<Params> for EndpointShim {
    fn resolve_endpoint(&self, params: &Params) -> Result<SmithyEndpoint, ResolveEndpointError> {
        let aws_endpoint = self
            .0
            .resolve_endpoint(
                params
                    .region
                    .as_ref()
                    .ok_or_else(|| ResolveEndpointError::message("no region in params"))?,
            )
            .map_err(|err| {
                ResolveEndpointError::message("failure resolving endpoint").with_source(err)
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

/// Middleware Stage to Add an Endpoint to a Request
///
/// AwsEndpointStage implements [`MapRequest`](aws_smithy_http::middleware::MapRequest). It will:
/// 1. Load an endpoint provider from the property bag.
/// 2. Load an endpoint given the [`Region`](aws_types::region::Region) in the property bag.
/// 3. Apply the endpoint to the URI in the request
/// 4. Set the `SigningRegion` and `SigningService` in the property bag to drive downstream
/// signing middleware.
#[derive(Clone, Debug)]
pub struct AwsEndpointStage;

#[derive(Debug)]
enum AwsEndpointStageErrorKind {
    NoEndpointResolver,
    EndpointResolutionError(BoxError),
}

#[derive(Debug)]
pub struct AwsEndpointStageError {
    kind: AwsEndpointStageErrorKind,
}

impl fmt::Display for AwsEndpointStageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AwsEndpointStageErrorKind::*;
        match &self.kind {
            NoEndpointResolver => write!(f, "endpoint resolution failed: no endpoint resolver"),
            EndpointResolutionError(_) => write!(f, "endpoint resolution failed"),
        }
    }
}

impl Error for AwsEndpointStageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use AwsEndpointStageErrorKind::*;
        match &self.kind {
            EndpointResolutionError(source) => Some(source.as_ref() as _),
            NoEndpointResolver => None,
        }
    }
}

impl From<AwsEndpointStageErrorKind> for AwsEndpointStageError {
    fn from(kind: AwsEndpointStageErrorKind) -> Self {
        Self { kind }
    }
}

impl MapRequest for AwsEndpointStage {
    type Error = AwsEndpointStageError;

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|mut http_req, props| {
            let endpoint_result = props
                .get_mut::<aws_smithy_http::endpoint::Result>()
                .ok_or(AwsEndpointStageErrorKind::NoEndpointResolver)?;
            let endpoint = match endpoint_result {
                // downgrade the mut ref to a shared ref
                Ok(_endpoint) => props.get::<aws_smithy_http::endpoint::Result>()
                    .expect("unreachable (prevalidated that the endpoint is in the bag)")
                    .as_ref()
                    .expect("unreachable (prevalidated that this is OK)"),
                Err(e) => {
                    // We need to own the error to return it, so take it and leave a stub error in
                    // its place
                    return Err(AwsEndpointStageErrorKind::EndpointResolutionError(std::mem::replace(
                        e,
                        ResolveEndpointError::message("the original error was directly returned")
                    ).into()).into());
                }
            };
            let (uri, signing_scope_override, signing_service_override) = smithy_to_aws(endpoint)
                .map_err(|err| AwsEndpointStageErrorKind::EndpointResolutionError(err))?;
            tracing::debug!(endpoint = ?endpoint, base_region = ?signing_scope_override, "resolved endpoint");
            apply_endpoint(http_req.uri_mut(), &uri, props.get::<EndpointPrefix>())
                .map_err(|err| AwsEndpointStageErrorKind::EndpointResolutionError(err.into()))?;
            for (header_name, header_values) in endpoint.headers() {
                http_req.headers_mut().remove(header_name);
                for value in header_values {
                    http_req.headers_mut().insert(
                        HeaderName::from_str(header_name)
                            .map_err(|err| AwsEndpointStageErrorKind::EndpointResolutionError(err.into()))?,
                        HeaderValue::from_str(value)
                            .map_err(|err| AwsEndpointStageErrorKind::EndpointResolutionError(err.into()))?,
                    );
                }
            }

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

type EndpointMetadata = (Uri, Option<SigningRegion>, Option<SigningService>);

fn smithy_to_aws(value: &SmithyEndpoint) -> Result<EndpointMetadata, Box<dyn Error + Send + Sync>> {
    let uri: Uri = value.url().parse()?;
    // look for v4 as an auth scheme
    let auth_schemes = match value
        .properties()
        .get("authSchemes")
        .ok_or("no auth schemes in metadata")?
    {
        Document::Array(schemes) => schemes,
        _other => return Err("expected an array for authSchemes".into()),
    };
    let v4 = auth_schemes
        .iter()
        .flat_map(|doc| match doc {
            Document::Object(map)
                if map.get("name") == Some(&Document::String("sigv4".to_string())) =>
            {
                Some(map)
            }
            _ => None,
        })
        .next()
        .ok_or("could not find v4 as an acceptable auth scheme")?;

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
    Ok((uri, signing_scope, signing_service))
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use http::header::HOST;
    use http::Uri;

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::endpoint::ResolveEndpoint;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use aws_types::endpoint::CredentialScope;
    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;

    use crate::partition::endpoint::{Metadata, Protocol, SignatureVersion};
    use crate::{AwsEndpointStage, EndpointShim, Params};

    #[test]
    fn default_endpoint_updates_request() {
        let provider = Arc::new(Metadata {
            uri_template: "kinesis.{region}.amazonaws.com",
            protocol: Protocol::Https,
            credential_scope: Default::default(),
            signature_versions: SignatureVersion::V4,
        });
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(region.clone());
            props.insert(SigningService::from_static("kinesis"));
            props.insert(
                EndpointShim::from_arc(provider)
                    .resolve_endpoint(&Params::new(Some(region.clone()))),
            );
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
        assert_eq!(req.properties().get(), Some(&SigningRegion::from(region)));
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("kinesis"))
        );

        let (req, conf) = req.into_parts();
        assert_eq!(
            req.uri(),
            &Uri::from_static("https://kinesis.us-east-1.amazonaws.com")
        );
        assert!(req.headers().get(HOST).is_none());
        assert!(
            conf.acquire()
                .get::<aws_smithy_http::endpoint::Result>()
                .is_some(),
            "Endpoint middleware MUST leave the result in the bag"
        );
    }

    #[test]
    fn sets_service_override_when_set() {
        let provider = Arc::new(Metadata {
            uri_template: "www.service.com",
            protocol: Protocol::Http,
            credential_scope: CredentialScope::builder()
                .service(SigningService::from_static("qldb-override"))
                .region(SigningRegion::from_static("us-east-override"))
                .build(),
            signature_versions: SignatureVersion::V4,
        });
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(region.clone());
            props.insert(SigningService::from_static("qldb"));
            props.insert(
                EndpointShim::from_arc(provider).resolve_endpoint(&Params::new(Some(region))),
            );
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
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
        let provider = Arc::new(Metadata {
            uri_template: "www.service.com",
            protocol: Protocol::Http,
            credential_scope: CredentialScope::builder().build(),
            signature_versions: SignatureVersion::V4,
        });
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut props = req.properties_mut();
            props.insert(region.clone());
            props.insert(SigningService::from_static("qldb"));
            props.insert(
                EndpointShim::from_arc(provider).resolve_endpoint(&Params::new(Some(region))),
            );
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
        assert_eq!(
            req.properties().get(),
            Some(&SigningRegion::from(Region::new("us-east-1")))
        );
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("qldb"))
        );
    }
}
