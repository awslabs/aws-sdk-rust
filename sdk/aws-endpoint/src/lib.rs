/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#[doc(hidden)]
pub mod partition;

#[doc(hidden)]
pub use partition::Partition;
#[doc(hidden)]
pub use partition::PartitionResolver;

use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use http::HeaderValue;

use aws_types::region::{Region, SigningRegion};
use aws_types::SigningService;
use http::header::HOST;
use smithy_http::endpoint::{Endpoint, EndpointPrefix};
use smithy_http::middleware::MapRequest;
use smithy_http::operation::Request;
use smithy_http::property_bag::PropertyBag;
use std::convert::TryFrom;

/// Endpoint to connect to an AWS Service
///
/// An `AwsEndpoint` captures all necessary information needed to connect to an AWS service, including:
/// - The URI of the endpoint (needed to actually send the request)
/// - The name of the service (needed downstream for signing)
/// - The signing region (which may differ from the actual region)
#[derive(Clone)]
pub struct AwsEndpoint {
    endpoint: Endpoint,
    credential_scope: CredentialScope,
}

impl AwsEndpoint {
    pub fn set_endpoint(&self, mut uri: &mut http::Uri, endpoint_prefix: Option<&EndpointPrefix>) {
        self.endpoint.set_endpoint(&mut uri, endpoint_prefix);
    }
}

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Resolve the AWS Endpoint for a given region
///
/// To provide a static endpoint, [`Endpoint`](smithy_http::endpoint::Endpoint) implements this trait.
/// Example usage:
/// ```rust
/// # mod dynamodb {
/// # use aws_endpoint::ResolveAwsEndpoint;
/// # pub struct ConfigBuilder;
/// # impl ConfigBuilder {
/// #     pub fn endpoint(&mut self, resolver: impl ResolveAwsEndpoint + 'static) {
/// #         // ...
/// #     }
/// # }
/// # pub struct Config;
/// # impl Config {
/// #     pub fn builder() -> ConfigBuilder {
/// #         ConfigBuilder
/// #     }
/// # }
/// # }
/// use smithy_http::endpoint::Endpoint;
/// use http::Uri;
/// let config = dynamodb::Config::builder()
///     .endpoint(
///         Endpoint::immutable(Uri::from_static("http://localhost:8080"))
///     );
/// ```
/// In the future, each AWS service will generate their own implementation of `ResolveAwsEndpoint`. This implementation
/// may use endpoint discovery. The list of supported regions for a given service
/// will be codegenerated from `endpoints.json`.
pub trait ResolveAwsEndpoint: Send + Sync {
    // TODO: consider if we want modeled error variants here
    fn resolve_endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError>;
}

#[derive(Clone, Default, Debug)]
pub struct CredentialScope {
    region: Option<SigningRegion>,
    service: Option<SigningService>,
}

impl CredentialScope {
    pub fn builder() -> credential_scope::Builder {
        credential_scope::Builder::default()
    }
}

pub mod credential_scope {
    use crate::CredentialScope;
    use aws_types::region::SigningRegion;
    use aws_types::SigningService;

    #[derive(Debug, Default)]
    pub struct Builder {
        region: Option<SigningRegion>,
        service: Option<SigningService>,
    }

    impl Builder {
        pub fn region(mut self, region: &'static str) -> Self {
            self.region = Some(SigningRegion::from_static(region));
            self
        }

        pub fn service(mut self, service: &'static str) -> Self {
            self.service = Some(SigningService::from_static(service));
            self
        }

        pub fn build(self) -> CredentialScope {
            CredentialScope {
                region: self.region,
                service: self.service,
            }
        }
    }
}

impl CredentialScope {
    pub fn merge(&self, other: &CredentialScope) -> CredentialScope {
        CredentialScope {
            region: self.region.clone().or_else(|| other.region.clone()),
            service: self.service.clone().or_else(|| other.service.clone()),
        }
    }
}

/// An `Endpoint` can be its own resolver to support static endpoints
impl ResolveAwsEndpoint for Endpoint {
    fn resolve_endpoint(&self, _region: &Region) -> Result<AwsEndpoint, BoxError> {
        Ok(AwsEndpoint {
            endpoint: self.clone(),
            credential_scope: Default::default(),
        })
    }
}

type AwsEndpointResolver = Arc<dyn ResolveAwsEndpoint>;
pub fn get_endpoint_resolver(config: &PropertyBag) -> Option<&AwsEndpointResolver> {
    config.get()
}

pub fn set_endpoint_resolver(config: &mut PropertyBag, provider: AwsEndpointResolver) {
    config.insert(provider);
}

/// Middleware Stage to Add an Endpoint to a Request
///
/// AwsEndpointStage implements [`MapRequest`](smithy_http::middleware::MapRequest). It will:
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
        request.augment(|mut http_req, config| {
            let provider =
                get_endpoint_resolver(config).ok_or(AwsEndpointStageError::NoEndpointResolver)?;
            let region = config
                .get::<Region>()
                .ok_or(AwsEndpointStageError::NoRegion)?;
            let endpoint = provider
                .resolve_endpoint(region)
                .map_err(AwsEndpointStageError::EndpointResolutionError)?;
            let signing_region = endpoint
                .credential_scope
                .region
                .unwrap_or_else(|| region.clone().into());
            config.insert::<SigningRegion>(signing_region);
            if let Some(signing_service) = endpoint.credential_scope.service {
                config.insert::<SigningService>(signing_service);
            }
            endpoint
                .endpoint
                .set_endpoint(http_req.uri_mut(), config.get::<EndpointPrefix>());
            // host is only None if authority is not. `set_endpoint` guarantees that authority is not None
            let host = http_req
                .uri()
                .host()
                .expect("authority is guaranteed to be non-empty after `set_endpoint`");
            let host = HeaderValue::try_from(host)
                .expect("authority must only contain valid header characters");
            http_req.headers_mut().insert(HOST, host);
            Ok(http_req)
        })
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use http::Uri;

    use aws_types::region::{Region, SigningRegion};
    use aws_types::SigningService;
    use smithy_http::body::SdkBody;
    use smithy_http::middleware::MapRequest;
    use smithy_http::operation;

    use crate::partition::endpoint::{Metadata, Protocol, SignatureVersion};
    use crate::{set_endpoint_resolver, AwsEndpointStage, CredentialScope};
    use http::header::HOST;

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
            let mut conf = req.config_mut();
            conf.insert(region.clone());
            conf.insert(SigningService::from_static("kinesis"));
            set_endpoint_resolver(&mut conf, provider);
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
        assert_eq!(
            req.config().get(),
            Some(&SigningRegion::from(region.clone()))
        );
        assert_eq!(
            req.config().get(),
            Some(&SigningService::from_static("kinesis"))
        );

        let (req, _conf) = req.into_parts();
        assert_eq!(
            req.uri(),
            &Uri::from_static("https://kinesis.us-east-1.amazonaws.com")
        );
        assert_eq!(
            req.headers().get(HOST).expect("host header must be set"),
            "kinesis.us-east-1.amazonaws.com"
        );
    }

    #[test]
    fn sets_service_override_when_set() {
        let provider = Arc::new(Metadata {
            uri_template: "www.service.com",
            protocol: Protocol::Http,
            credential_scope: CredentialScope::builder()
                .service("qldb-override")
                .region("us-east-override")
                .build(),
            signature_versions: SignatureVersion::V4,
        });
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut conf = req.config_mut();
            conf.insert(region.clone());
            conf.insert(SigningService::from_static("kinesis"));
            set_endpoint_resolver(&mut conf, provider);
        };
        let req = AwsEndpointStage.apply(req).expect("should succeed");
        assert_eq!(
            req.config().get(),
            Some(&SigningRegion::from(Region::new("us-east-override")))
        );
        assert_eq!(
            req.config().get(),
            Some(&SigningService::from_static("qldb-override"))
        );
    }
}
