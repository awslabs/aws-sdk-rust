/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use http::uri::{Authority, InvalidUri, Uri};
use std::str::FromStr;

/// API Endpoint
///
/// This implements an API endpoint as specified in the
/// [Smithy Endpoint Specification](https://awslabs.github.io/smithy/1.0/spec/core/endpoint-traits.html)
#[derive(Clone)]
pub struct Endpoint {
    uri: http::Uri,

    /// If true, endpointPrefix does ignored when setting the endpoint on a request
    immutable: bool,
}

pub struct EndpointPrefix(String);
impl EndpointPrefix {
    pub fn new(prefix: impl Into<String>) -> Result<Self, InvalidUri> {
        let prefix = prefix.into();
        let _ = Authority::from_str(&prefix)?;
        Ok(EndpointPrefix(prefix))
    }
}

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum InvalidEndpoint {
    EndpointMustHaveAuthority,
}

// user gives: Endpoint Provider. Give endpoint. Is this endpoint extensible?
// where endpoint discovery starts from.
// endpoints can mutate a request, potentially with a

impl Endpoint {
    /// Create a new endpoint from a URI
    ///
    /// Certain protocols will attempt to prefix additional information onto an endpoint. If you
    /// wish to ignore these prefixes (for example, when communicating with localhost), set `immutable` to `true`.
    pub fn mutable(uri: Uri) -> Self {
        Endpoint {
            uri,
            immutable: false,
        }
    }

    /// Create a new immutable endpoint from a URI
    ///
    /// ```rust
    /// # use smithy_http::endpoint::Endpoint;
    /// use http::Uri;
    /// let endpoint = Endpoint::immutable(Uri::from_static("http://localhost:8000"));
    /// ```
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
            .path_and_query(uri.path_and_query().unwrap().clone())
            .build()
            .expect("valid uri");
        *uri = new_uri;
    }
}

#[cfg(test)]
mod test {
    use crate::endpoint::{Endpoint, EndpointPrefix};
    use http::Uri;

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
    fn set_endpoint_empty_path() {
        let ep = Endpoint::immutable(Uri::from_static("http://localhost:8000"));
        let mut uri = Uri::from_static("/");
        ep.set_endpoint(&mut uri, None);
        assert_eq!(uri, Uri::from_static("http://localhost:8000/"))
    }
}
