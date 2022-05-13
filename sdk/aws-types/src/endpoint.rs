/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! AWS SDK endpoint support.

use crate::region::{Region, SigningRegion};
use crate::SigningService;
use aws_smithy_http::endpoint::{Endpoint, EndpointPrefix};
use std::error::Error;
use std::fmt::Debug;

/// Endpoint to connect to an AWS Service
///
/// An `AwsEndpoint` captures all necessary information needed to connect to an AWS service, including:
/// - The URI of the endpoint (needed to actually send the request)
/// - The name of the service (needed downstream for signing)
/// - The signing region (which may differ from the actual region)
#[derive(Clone, Debug)]
pub struct AwsEndpoint {
    endpoint: Endpoint,
    credential_scope: CredentialScope,
}

impl AwsEndpoint {
    /// Constructs a new AWS endpoint.
    pub fn new(endpoint: Endpoint, credential_scope: CredentialScope) -> AwsEndpoint {
        AwsEndpoint {
            endpoint,
            credential_scope,
        }
    }

    /// Returns the underlying endpoint.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Returns the credential scope.
    pub fn credential_scope(&self) -> &CredentialScope {
        &self.credential_scope
    }

    /// Sets the endpoint on a given `uri` based on this endpoint
    pub fn set_endpoint(&self, uri: &mut http::Uri, endpoint_prefix: Option<&EndpointPrefix>) {
        self.endpoint.set_endpoint(uri, endpoint_prefix);
    }
}

/// A boxed error.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;

/// Resolve the AWS Endpoint for a given region
///
/// To provide a static endpoint, [`Endpoint`](aws_smithy_http::endpoint::Endpoint) implements this trait.
/// Example usage:
/// ```rust
/// # mod dynamodb {
/// # use aws_types::endpoint::ResolveAwsEndpoint;
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
/// use aws_smithy_http::endpoint::Endpoint;
/// use http::Uri;
/// let config = dynamodb::Config::builder()
///     .endpoint(
///         Endpoint::immutable(Uri::from_static("http://localhost:8080"))
///     );
/// ```
/// Each AWS service generates their own implementation of `ResolveAwsEndpoint`.
pub trait ResolveAwsEndpoint: Send + Sync + Debug {
    /// Resolves the AWS endpoint for a given region.
    // TODO(https://github.com/awslabs/smithy-rs/issues/866): Create `ResolveEndpointError`
    fn resolve_endpoint(&self, region: &Region) -> Result<AwsEndpoint, BoxError>;
}

/// The scope for AWS credentials.
#[derive(Clone, Default, Debug)]
pub struct CredentialScope {
    region: Option<SigningRegion>,
    service: Option<SigningService>,
}

impl CredentialScope {
    /// Creates a builder for [`CredentialScope`].
    pub fn builder() -> credential_scope::Builder {
        credential_scope::Builder::default()
    }
}

/// Types associated with [`CredentialScope`].
pub mod credential_scope {
    use crate::endpoint::CredentialScope;
    use crate::region::SigningRegion;
    use crate::SigningService;

    /// A builder for [`CredentialScope`].
    #[derive(Debug, Default)]
    pub struct Builder {
        region: Option<SigningRegion>,
        service: Option<SigningService>,
    }

    impl Builder {
        /// Sets the signing region.
        pub fn region(mut self, region: impl Into<SigningRegion>) -> Self {
            self.region = Some(region.into());
            self
        }

        /// Sets the signing service.
        pub fn service(mut self, service: impl Into<SigningService>) -> Self {
            self.service = Some(service.into());
            self
        }

        /// Constructs a [`CredentialScope`] from the builder.
        pub fn build(self) -> CredentialScope {
            CredentialScope {
                region: self.region,
                service: self.service,
            }
        }
    }
}

impl CredentialScope {
    /// Returns the signing region.
    pub fn region(&self) -> Option<&SigningRegion> {
        self.region.as_ref()
    }

    /// Returns the signing service.
    pub fn service(&self) -> Option<&SigningService> {
        self.service.as_ref()
    }

    /// Uses the values from `other` to fill in unconfigured parameters on this
    /// credential scope object.
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

#[cfg(test)]
mod test {
    use crate::endpoint::CredentialScope;
    use crate::region::SigningRegion;
    use crate::SigningService;

    #[test]
    fn create_credentials_scope_from_strs() {
        let scope = CredentialScope::builder()
            .service("s3")
            .region("us-east-1")
            .build();
        assert_eq!(scope.service(), Some(&SigningService::from_static("s3")));
        assert_eq!(
            scope.region(),
            Some(&SigningRegion::from_static("us-east-1"))
        );
    }
}
