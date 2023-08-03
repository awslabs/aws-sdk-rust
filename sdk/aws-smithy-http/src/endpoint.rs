/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Code for resolving an endpoint (URI) that a request should be sent to

use crate::endpoint::error::InvalidEndpointError;
use crate::operation::error::BuildError;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use http::uri::{Authority, Uri};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::result::Result as StdResult;
use std::str::FromStr;
use std::sync::Arc;

pub mod error;
pub mod middleware;

pub use error::ResolveEndpointError;

/// An endpoint-resolution-specific Result. Contains either an [`Endpoint`](aws_smithy_types::endpoint::Endpoint) or a [`ResolveEndpointError`].
pub type Result = std::result::Result<aws_smithy_types::endpoint::Endpoint, ResolveEndpointError>;

/// Implementors of this trait can resolve an endpoint that will be applied to a request.
pub trait ResolveEndpoint<Params>: Send + Sync {
    /// Given some endpoint parameters, resolve an endpoint or return an error when resolution is
    /// impossible.
    fn resolve_endpoint(&self, params: &Params) -> Result;
}

impl<T> ResolveEndpoint<T> for &'static str {
    fn resolve_endpoint(&self, _params: &T) -> Result {
        Ok(aws_smithy_types::endpoint::Endpoint::builder()
            .url(*self)
            .build())
    }
}

/// Endpoint Resolver wrapper that may be shared
#[derive(Clone)]
pub struct SharedEndpointResolver<T>(Arc<dyn ResolveEndpoint<T>>);

impl<T> Debug for SharedEndpointResolver<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedEndpointResolver").finish()
    }
}

impl<T> SharedEndpointResolver<T> {
    /// Create a new `SharedEndpointResolver` from `ResolveEndpoint`
    pub fn new(resolve_endpoint: impl ResolveEndpoint<T> + 'static) -> Self {
        Self(Arc::new(resolve_endpoint))
    }
}

impl<T> AsRef<dyn ResolveEndpoint<T>> for SharedEndpointResolver<T> {
    fn as_ref(&self) -> &(dyn ResolveEndpoint<T> + 'static) {
        self.0.as_ref()
    }
}

impl<T> From<Arc<dyn ResolveEndpoint<T>>> for SharedEndpointResolver<T> {
    fn from(resolve_endpoint: Arc<dyn ResolveEndpoint<T>>) -> Self {
        SharedEndpointResolver(resolve_endpoint)
    }
}

impl<T: 'static> Storable for SharedEndpointResolver<T> {
    type Storer = StoreReplace<SharedEndpointResolver<T>>;
}

impl<T> ResolveEndpoint<T> for SharedEndpointResolver<T> {
    fn resolve_endpoint(&self, params: &T) -> Result {
        self.0.resolve_endpoint(params)
    }
}

/// API Endpoint
///
/// This implements an API endpoint as specified in the
/// [Smithy Endpoint Specification](https://awslabs.github.io/smithy/1.0/spec/core/endpoint-traits.html)
#[derive(Clone, Debug)]
#[deprecated(note = "Use `.endpoint_url(...)` directly instead")]
pub struct Endpoint {
    uri: http::Uri,

    /// If true, endpointPrefix does ignored when setting the endpoint on a request
    immutable: bool,
}

#[allow(deprecated)]
/// This allows customers that use `Endpoint` to override the endpoint to continue to do so
impl<T> ResolveEndpoint<T> for Endpoint {
    fn resolve_endpoint(&self, _params: &T) -> Result {
        Ok(aws_smithy_types::endpoint::Endpoint::builder()
            .url(self.uri.to_string())
            .build())
    }
}

/// A special type that adds support for services that have special URL-prefixing rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointPrefix(String);
impl EndpointPrefix {
    /// Create a new endpoint prefix from an `impl Into<String>`. If the prefix argument is invalid,
    /// a [`BuildError`] will be returned.
    pub fn new(prefix: impl Into<String>) -> StdResult<Self, BuildError> {
        let prefix = prefix.into();
        match Authority::from_str(&prefix) {
            Ok(_) => Ok(EndpointPrefix(prefix)),
            Err(err) => Err(BuildError::invalid_uri(
                prefix,
                "invalid prefix".into(),
                err,
            )),
        }
    }

    /// Get the `str` representation of this `EndpointPrefix`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Storable for EndpointPrefix {
    type Storer = StoreReplace<Self>;
}

/// Apply `endpoint` to `uri`
///
/// This method mutates `uri` by setting the `endpoint` on it
pub fn apply_endpoint(
    uri: &mut Uri,
    endpoint: &Uri,
    prefix: Option<&EndpointPrefix>,
) -> StdResult<(), InvalidEndpointError> {
    let prefix = prefix.map(|p| p.0.as_str()).unwrap_or("");
    let authority = endpoint
        .authority()
        .as_ref()
        .map(|auth| auth.as_str())
        .unwrap_or("");
    let authority = if !prefix.is_empty() {
        Authority::from_str(&format!("{}{}", prefix, authority))
    } else {
        Authority::from_str(authority)
    }
    .map_err(InvalidEndpointError::failed_to_construct_authority)?;
    let scheme = *endpoint
        .scheme()
        .as_ref()
        .ok_or_else(InvalidEndpointError::endpoint_must_have_scheme)?;
    let new_uri = Uri::builder()
        .authority(authority)
        .scheme(scheme.clone())
        .path_and_query(merge_paths(endpoint, uri).as_ref())
        .build()
        .map_err(InvalidEndpointError::failed_to_construct_uri)?;
    *uri = new_uri;
    Ok(())
}

#[allow(deprecated)]
impl Endpoint {
    /// Create a new endpoint from a URI
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. If your endpoint does not support this,
    /// (for example, when communicating with localhost), use [`Endpoint::immutable`].
    pub fn mutable_uri(uri: Uri) -> StdResult<Self, InvalidEndpointError> {
        Ok(Endpoint {
            uri: Self::validate_endpoint(uri)?,
            immutable: false,
        })
    }

    /// Create a new endpoint from a URI string
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. If your endpoint does not support this,
    /// (for example, when communicating with localhost), use [`Endpoint::immutable`].
    pub fn mutable(uri: impl AsRef<str>) -> StdResult<Self, InvalidEndpointError> {
        Self::mutable_uri(
            Uri::try_from(uri.as_ref()).map_err(InvalidEndpointError::failed_to_construct_uri)?,
        )
    }

    /// Returns the URI of this endpoint
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Create a new immutable endpoint from a URI
    ///
    /// ```rust
    /// # use aws_smithy_http::endpoint::Endpoint;
    /// use http::Uri;
    /// let uri = Uri::from_static("http://localhost:8000");
    /// let endpoint = Endpoint::immutable_uri(uri);
    /// ```
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. This constructor creates an endpoint which will
    /// ignore those mutations. If you want an endpoint which will obey mutation requests, use
    /// [`Endpoint::mutable`] instead.
    pub fn immutable_uri(uri: Uri) -> StdResult<Self, InvalidEndpointError> {
        Ok(Endpoint {
            uri: Self::validate_endpoint(uri)?,
            immutable: true,
        })
    }

    /// Create a new immutable endpoint from a URI string
    ///
    /// ```rust
    /// # use aws_smithy_http::endpoint::Endpoint;
    /// let endpoint = Endpoint::immutable("http://localhost:8000");
    /// ```
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. This constructor creates an endpoint which will
    /// ignore those mutations. If you want an endpoint which will obey mutation requests, use
    /// [`Endpoint::mutable`] instead.
    pub fn immutable(uri: impl AsRef<str>) -> StdResult<Self, InvalidEndpointError> {
        Self::immutable_uri(
            Uri::try_from(uri.as_ref()).map_err(InvalidEndpointError::failed_to_construct_uri)?,
        )
    }

    /// Sets the endpoint on `uri`, potentially applying the specified `prefix` in the process.
    pub fn set_endpoint(
        &self,
        uri: &mut http::Uri,
        prefix: Option<&EndpointPrefix>,
    ) -> StdResult<(), InvalidEndpointError> {
        let prefix = match self.immutable {
            true => None,
            false => prefix,
        };
        apply_endpoint(uri, &self.uri, prefix)
    }

    fn validate_endpoint(endpoint: Uri) -> StdResult<Uri, InvalidEndpointError> {
        if endpoint.scheme().is_none() {
            Err(InvalidEndpointError::endpoint_must_have_scheme())
        } else {
            Ok(endpoint)
        }
    }
}

fn merge_paths<'a>(endpoint: &'a Uri, uri: &'a Uri) -> Cow<'a, str> {
    if let Some(query) = endpoint.path_and_query().and_then(|pq| pq.query()) {
        tracing::warn!(query = %query, "query specified in endpoint will be ignored during endpoint resolution");
    }
    let endpoint_path = endpoint.path();
    let uri_path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
    if endpoint_path.is_empty() {
        Cow::Borrowed(uri_path_and_query)
    } else {
        let ep_no_slash = endpoint_path.strip_suffix('/').unwrap_or(endpoint_path);
        let uri_path_no_slash = uri_path_and_query
            .strip_prefix('/')
            .unwrap_or(uri_path_and_query);
        Cow::Owned(format!("{}/{}", ep_no_slash, uri_path_no_slash))
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use crate::endpoint::error::{InvalidEndpointError, InvalidEndpointErrorKind};
    use crate::endpoint::{Endpoint, EndpointPrefix};
    use http::Uri;

    #[test]
    fn prefix_endpoint() {
        let ep = Endpoint::mutable("https://us-east-1.dynamo.amazonaws.com").unwrap();
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        )
        .unwrap();
        assert_eq!(
            uri,
            Uri::from_static("https://subregion.us-east-1.dynamo.amazonaws.com/list_tables?k=v")
        );
    }

    #[test]
    fn prefix_endpoint_custom_port() {
        let ep = Endpoint::mutable("https://us-east-1.dynamo.amazonaws.com:6443").unwrap();
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        )
        .unwrap();
        assert_eq!(
            uri,
            Uri::from_static(
                "https://subregion.us-east-1.dynamo.amazonaws.com:6443/list_tables?k=v"
            )
        );
    }

    #[test]
    fn prefix_immutable_endpoint() {
        let ep = Endpoint::immutable("https://us-east-1.dynamo.amazonaws.com").unwrap();
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        )
        .unwrap();
        assert_eq!(
            uri,
            Uri::from_static("https://us-east-1.dynamo.amazonaws.com/list_tables?k=v")
        );
    }

    #[test]
    fn endpoint_with_path() {
        for uri in &[
            // check that trailing slashes are properly normalized
            "https://us-east-1.dynamo.amazonaws.com/private",
            "https://us-east-1.dynamo.amazonaws.com/private/",
        ] {
            let ep = Endpoint::immutable(uri).unwrap();
            let mut uri = Uri::from_static("/list_tables?k=v");
            ep.set_endpoint(
                &mut uri,
                Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
            )
            .unwrap();
            assert_eq!(
                uri,
                Uri::from_static("https://us-east-1.dynamo.amazonaws.com/private/list_tables?k=v")
            );
        }
    }

    #[test]
    fn set_endpoint_empty_path() {
        let ep = Endpoint::immutable("http://localhost:8000").unwrap();
        let mut uri = Uri::from_static("/");
        ep.set_endpoint(&mut uri, None).unwrap();
        assert_eq!(uri, Uri::from_static("http://localhost:8000/"))
    }

    #[test]
    fn endpoint_construction_missing_scheme() {
        assert!(matches!(
            Endpoint::mutable("localhost:8000"),
            Err(InvalidEndpointError {
                kind: InvalidEndpointErrorKind::EndpointMustHaveScheme
            })
        ));
        assert!(matches!(
            Endpoint::immutable("localhost:8000"),
            Err(InvalidEndpointError {
                kind: InvalidEndpointErrorKind::EndpointMustHaveScheme
            })
        ));
    }
}
