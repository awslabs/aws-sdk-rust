/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;

use http::{HeaderValue, Uri};

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
    signing_service: Option<SigningService>,
    signing_region: Option<SigningRegion>,
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
    fn endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError>;
}

/// Default AWS Endpoint Implementation
///
/// This is used as a temporary stub. Prior to GA, this will be replaced with specifically generated endpoint
/// resolvers for each service that model the endpoints for each service correctly. Some services differ
/// from the standard endpoint pattern.
pub struct DefaultAwsEndpointResolver {
    service: &'static str,
}

impl DefaultAwsEndpointResolver {
    pub fn for_service(service: &'static str) -> Self {
        Self { service }
    }
}

/// An `Endpoint` can be its own resolver to support static endpoints
impl ResolveAwsEndpoint for Endpoint {
    fn endpoint(&self, _region: &Region) -> Result<AwsEndpoint, BoxError> {
        Ok(AwsEndpoint {
            endpoint: self.clone(),
            signing_service: None,
            signing_region: None,
        })
    }
}

impl ResolveAwsEndpoint for DefaultAwsEndpointResolver {
    fn endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError> {
        let uri = Uri::from_str(&format!(
            "https://{}.{}.amazonaws.com",
            self.service,
            region.as_ref(),
        ))?;
        Ok(AwsEndpoint {
            endpoint: Endpoint::mutable(uri),
            signing_region: Some(region.clone().into()),
            signing_service: Some(SigningService::from_static(self.service)),
        })
    }
}

type AwsEndpointResolver = Arc<dyn ResolveAwsEndpoint>;
fn get_endpoint_resolver(config: &PropertyBag) -> Option<&AwsEndpointResolver> {
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
#[derive(Clone)]
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
                .endpoint(region)
                .map_err(AwsEndpointStageError::EndpointResolutionError)?;
            let signing_region = endpoint
                .signing_region
                .unwrap_or_else(|| region.clone().into());
            config.insert::<SigningRegion>(signing_region);
            if let Some(signing_service) = endpoint.signing_service {
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

    use crate::{set_endpoint_resolver, AwsEndpointStage, DefaultAwsEndpointResolver};
    use http::header::HOST;

    #[test]
    fn default_endpoint_updates_request() {
        let provider = Arc::new(DefaultAwsEndpointResolver::for_service("kinesis"));
        let req = http::Request::new(SdkBody::from(""));
        let region = Region::new("us-east-1");
        let mut req = operation::Request::new(req);
        {
            let mut conf = req.config_mut();
            conf.insert(region.clone());
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
}
