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

use aws_smithy_http::endpoint::EndpointPrefix;
use aws_smithy_http::middleware::MapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_http::property_bag::PropertyBag;
use aws_types::region::{Region, SigningRegion};
use aws_types::SigningService;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

pub use aws_types::endpoint::{AwsEndpoint, BoxError, CredentialScope, ResolveAwsEndpoint};

type AwsEndpointResolver = Arc<dyn ResolveAwsEndpoint>;
pub fn get_endpoint_resolver(properties: &PropertyBag) -> Option<&AwsEndpointResolver> {
    properties.get()
}

pub fn set_endpoint_resolver(properties: &mut PropertyBag, provider: AwsEndpointResolver) {
    properties.insert(provider);
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
pub enum AwsEndpointStageError {
    NoEndpointResolver,
    NoRegion,
    EndpointResolutionError(BoxError),
}

impl Display for AwsEndpointStageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Error for AwsEndpointStageError {}

impl MapRequest for AwsEndpointStage {
    type Error = AwsEndpointStageError;

    fn apply(&self, request: Request) -> Result<Request, Self::Error> {
        request.augment(|mut http_req, props| {
            let provider =
                get_endpoint_resolver(props).ok_or(AwsEndpointStageError::NoEndpointResolver)?;
            let region = props
                .get::<Region>()
                .ok_or(AwsEndpointStageError::NoRegion)?;
            let endpoint = provider
                .resolve_endpoint(region)
                .map_err(AwsEndpointStageError::EndpointResolutionError)?;
            tracing::debug!(endpoint = ?endpoint, base_region = ?region, "resolved endpoint");
            let signing_region = endpoint
                .credential_scope()
                .region()
                .cloned()
                .unwrap_or_else(|| region.clone().into());
            props.insert::<SigningRegion>(signing_region);
            if let Some(signing_service) = endpoint.credential_scope().service() {
                props.insert::<SigningService>(signing_service.clone());
            }
            endpoint.set_endpoint(http_req.uri_mut(), props.get::<EndpointPrefix>());
            Ok(http_req)
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use http::header::HOST;
    use http::Uri;

    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::MapRequest;
    use aws_smithy_http::operation;
    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;

    use crate::partition::endpoint::{Metadata, Protocol, SignatureVersion};
    use crate::{set_endpoint_resolver, AwsEndpointStage, CredentialScope};

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
            set_endpoint_resolver(&mut props, provider);
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
        assert_eq!(req.properties().get(), Some(&SigningRegion::from(region)));
        assert_eq!(
            req.properties().get(),
            Some(&SigningService::from_static("kinesis"))
        );

        let (req, _conf) = req.into_parts();
        assert_eq!(
            req.uri(),
            &Uri::from_static("https://kinesis.us-east-1.amazonaws.com")
        );
        assert!(req.headers().get(HOST).is_none());
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
            props.insert(region);
            props.insert(SigningService::from_static("kinesis"));
            set_endpoint_resolver(&mut props, provider);
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
}
