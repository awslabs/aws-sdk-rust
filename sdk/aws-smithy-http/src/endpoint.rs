/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;
use std::str::FromStr;

use http::uri::{Authority, Uri};

use crate::operation::BuildError;

/// API Endpoint
///
/// This implements an API endpoint as specified in the
/// [Smithy Endpoint Specification](https://awslabs.github.io/smithy/1.0/spec/core/endpoint-traits.html)
#[derive(Clone, Debug)]
pub struct Endpoint {
    uri: http::Uri,

    /// If true, endpointPrefix does ignored when setting the endpoint on a request
    immutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EndpointPrefix(String);
impl EndpointPrefix {
    pub fn new(prefix: impl Into<String>) -> Result<Self, BuildError> {
        let prefix = prefix.into();
        match Authority::from_str(&prefix) {
            Ok(_) => Ok(EndpointPrefix(prefix)),
            Err(err) => Err(BuildError::InvalidUri {
                uri: prefix,
                err,
                message: "invalid prefix".into(),
            }),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum InvalidEndpoint {
    EndpointMustHaveAuthority,
}

impl Endpoint {
    /// Create a new endpoint from a URI
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. If your endpoint does not support this,
    /// (for example, when communicating with localhost), use [`Endpoint::immutable`].
    pub fn mutable(uri: Uri) -> Self {
        Endpoint {
            uri,
            immutable: false,
        }
    }

    /// Create a new immutable endpoint from a URI
    ///
    /// ```rust
    /// # use aws_smithy_http::endpoint::Endpoint;
    /// use http::Uri;
    /// let endpoint = Endpoint::immutable(Uri::from_static("http://localhost:8000"));
    /// ```
    ///
    /// Certain services will augment the endpoint with additional metadata. For example,
    /// S3 can prefix the host with the bucket name. This constructor creates an endpoint which will
    /// ignore those mutations. If you want an endpoint which will obey mutation requests, use
    /// [`Endpoint::mutable`] instead.
    pub fn immutable(uri: Uri) -> Self {
        Endpoint {
            uri,
            immutable: true,
        }
    }

    /// Sets the endpoint on `uri`, potentially applying the specified `prefix` in the process.
    pub fn set_endpoint(&self, uri: &mut http::Uri, prefix: Option<&EndpointPrefix>) {
        let prefix = prefix.map(|p| p.0.as_str()).unwrap_or("");
        let authority = self
            .uri
            .authority()
            .as_ref()
            .map(|auth| auth.as_str())
            .unwrap_or("");
        let authority = if !self.immutable && !prefix.is_empty() {
            Authority::from_str(&format!("{}{}", prefix, authority)).expect("parts must be valid")
        } else {
            Authority::from_str(authority).expect("authority is valid")
        };
        let scheme = *self.uri.scheme().as_ref().expect("scheme must be provided");
        let new_uri = Uri::builder()
            .authority(authority)
            .scheme(scheme.clone())
            .path_and_query(Self::merge_paths(&self.uri, uri).as_ref())
            .build()
            .expect("valid uri");
        *uri = new_uri;
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
}

#[cfg(test)]
mod test {
    use http::Uri;

    use crate::endpoint::{Endpoint, EndpointPrefix};

    #[test]
    fn prefix_endpoint() {
        let ep = Endpoint::mutable(Uri::from_static("https://us-east-1.dynamo.amazonaws.com"));
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        );
        assert_eq!(
            uri,
            Uri::from_static("https://subregion.us-east-1.dynamo.amazonaws.com/list_tables?k=v")
        );
    }

    #[test]
    fn prefix_endpoint_custom_port() {
        let ep = Endpoint::mutable(Uri::from_static(
            "https://us-east-1.dynamo.amazonaws.com:6443",
        ));
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        );
        assert_eq!(
            uri,
            Uri::from_static(
                "https://subregion.us-east-1.dynamo.amazonaws.com:6443/list_tables?k=v"
            )
        );
    }

    #[test]
    fn prefix_immutable_endpoint() {
        let ep = Endpoint::immutable(Uri::from_static("https://us-east-1.dynamo.amazonaws.com"));
        let mut uri = Uri::from_static("/list_tables?k=v");
        ep.set_endpoint(
            &mut uri,
            Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
        );
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
            let ep = Endpoint::immutable(Uri::from_static(uri));
            let mut uri = Uri::from_static("/list_tables?k=v");
            ep.set_endpoint(
                &mut uri,
                Some(&EndpointPrefix::new("subregion.").expect("valid prefix")),
            );
            assert_eq!(
                uri,
                Uri::from_static("https://us-east-1.dynamo.amazonaws.com/private/list_tables?k=v")
            );
        }
    }

    #[test]
    fn set_endpoint_empty_path() {
        let ep = Endpoint::immutable(Uri::from_static("http://localhost:8000"));
        let mut uri = Uri::from_static("/");
        ep.set_endpoint(&mut uri, None);
        assert_eq!(uri, Uri::from_static("http://localhost:8000/"))
    }
}
