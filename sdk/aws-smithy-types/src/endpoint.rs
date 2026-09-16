/*
 *  Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *  SPDX-License-Identifier: Apache-2.0
 */
//! Smithy Endpoint Types

use crate::config_bag::{Storable, StoreReplace};
use crate::Document;
use std::borrow::Cow;
use std::collections::HashMap;

type MaybeStatic = Cow<'static, str>;

/// An authentication scheme configuration for an endpoint.
///
/// This is a lightweight alternative to storing auth schemes as
/// `Document::Object(HashMap<String, Document>)` in endpoint properties.
/// Properties are stored in a flat `Vec` and looked up via linear scan,
/// which is faster than HashMap for the typical 3-4 entries.
#[derive(Debug, Clone, PartialEq)]
pub struct EndpointAuthScheme {
    name: MaybeStatic,
    properties: Vec<(MaybeStatic, Document)>,
}

impl EndpointAuthScheme {
    /// Creates a new `EndpointAuthScheme` with pre-allocated capacity for properties.
    pub fn with_capacity(name: impl Into<MaybeStatic>, capacity: usize) -> Self {
        Self {
            name: name.into(),
            properties: Vec::with_capacity(capacity),
        }
    }

    /// Returns the auth scheme name (e.g., "sigv4", "sigv4a").
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a property to this auth scheme. Chainable.
    pub fn put(mut self, key: impl Into<MaybeStatic>, value: impl Into<Document>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }

    /// Gets a property value by name (linear scan).
    pub fn get(&self, key: &str) -> Option<&Document> {
        self.properties
            .iter()
            .find(|(k, _)| k.as_ref() == key)
            .map(|(_, v)| v)
    }

    /// Converts this auth scheme into a `Document` for backward compatibility.
    pub fn as_document(&self) -> Document {
        let mut map = HashMap::with_capacity(self.properties.len() + 1);
        map.insert("name".to_string(), Document::String(self.name.to_string()));
        for (k, v) in &self.properties {
            map.insert(k.to_string(), v.clone());
        }
        Document::Object(map)
    }
}

/* ANCHOR: endpoint */
/// Smithy Endpoint Type
///
/// Generally, this type should not be used from user code
#[derive(Debug, Clone, PartialEq)]
pub struct Endpoint {
    url: MaybeStatic,
    headers: HashMap<MaybeStatic, Vec<MaybeStatic>>,
    properties: HashMap<MaybeStatic, Document>,
    auth_schemes: Vec<EndpointAuthScheme>,
}

/* ANCHOR_END: endpoint */

#[allow(unused)]
impl Endpoint {
    /// Returns the URL of this endpoint
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the headers associated with this endpoint
    pub fn headers(&self) -> impl Iterator<Item = (&str, impl Iterator<Item = &str>)> {
        self.headers
            .iter()
            .map(|(k, v)| (k.as_ref(), v.iter().map(|v| v.as_ref())))
    }

    /// Returns the properties associated with this endpoint
    pub fn properties(&self) -> &HashMap<Cow<'static, str>, Document> {
        &self.properties
    }

    /// Returns the typed auth schemes associated with this endpoint
    pub fn auth_schemes(&self) -> &[EndpointAuthScheme] {
        &self.auth_schemes
    }

    /// Converts this endpoint back into a [`Builder`]
    pub fn into_builder(self) -> Builder {
        Builder { endpoint: self }
    }

    /// A builder for [`Endpoint`]
    pub fn builder() -> Builder {
        Builder::new()
    }
}

impl Storable for Endpoint {
    type Storer = StoreReplace<Self>;
}

#[derive(Debug, Clone)]
/// Builder for [`Endpoint`]
pub struct Builder {
    endpoint: Endpoint,
}

#[allow(unused)]
impl Builder {
    pub(crate) fn new() -> Self {
        Self {
            endpoint: Endpoint {
                url: Default::default(),
                headers: HashMap::new(),
                properties: HashMap::new(),
                auth_schemes: Vec::new(),
            },
        }
    }

    /// Set the URL of the Endpoint
    ///
    /// # Examples
    /// ```rust
    /// use aws_smithy_types::endpoint::Endpoint;
    /// let endpoint = Endpoint::builder().url("https://www.example.com").build();
    /// ```
    pub fn url(mut self, url: impl Into<MaybeStatic>) -> Self {
        self.endpoint.url = url.into();
        self
    }

    /// Adds a header to the endpoint
    ///
    /// If there is already a header for this key, this header will be appended to that key
    ///
    /// # Examples
    /// ```rust
    /// use aws_smithy_types::endpoint::Endpoint;
    /// let endpoint = Endpoint::builder().url("https://www.example.com").header("x-my-header", "hello").build();
    /// ```
    pub fn header(mut self, name: impl Into<MaybeStatic>, value: impl Into<MaybeStatic>) -> Self {
        self.endpoint
            .headers
            .entry(name.into())
            .or_default()
            .push(value.into());
        self
    }

    /// Adds a property to the endpoint
    ///
    /// If there is already a property for this key, the existing property will be overwritten
    ///
    /// # Examples
    /// ```rust
    /// use aws_smithy_types::endpoint::Endpoint;
    /// let endpoint = Endpoint::builder()
    ///   .url("https://www.example.com")
    ///   .property("x-my-header", true)
    ///   .build();
    /// ```
    pub fn property(mut self, key: impl Into<MaybeStatic>, value: impl Into<Document>) -> Self {
        self.endpoint.properties.insert(key.into(), value.into());
        self
    }

    /// Adds a typed auth scheme to the endpoint
    pub fn auth_scheme(mut self, auth_scheme: EndpointAuthScheme) -> Self {
        self.endpoint.auth_schemes.push(auth_scheme);
        self
    }

    /// Constructs an [`Endpoint`] from this builder
    ///
    /// # Panics
    /// Panics if URL is unset or empty
    pub fn build(self) -> Endpoint {
        assert_ne!(self.endpoint.url(), "", "URL was unset");
        self.endpoint
    }
}

#[cfg(test)]
mod test {
    use crate::endpoint::Endpoint;
    use crate::Document;
    use std::borrow::Cow;
    use std::collections::HashMap;

    #[test]
    fn endpoint_builder() {
        let endpoint = Endpoint::builder()
            .url("https://www.amazon.com")
            .header("x-amz-test", "header-value")
            .property("custom", Document::Bool(true))
            .build();
        assert_eq!(endpoint.url, Cow::Borrowed("https://www.amazon.com"));
        assert_eq!(
            endpoint.headers,
            HashMap::from([(
                Cow::Borrowed("x-amz-test"),
                vec![Cow::Borrowed("header-value")]
            )])
        );
        assert_eq!(
            endpoint.properties,
            HashMap::from([(Cow::Borrowed("custom"), Document::Bool(true))])
        );

        assert_eq!(endpoint.url(), "https://www.amazon.com");
        assert_eq!(
            endpoint
                .headers()
                .map(|(k, v)| (k, v.collect::<Vec<_>>()))
                .collect::<Vec<_>>(),
            vec![("x-amz-test", vec!["header-value"])]
        );
    }

    #[test]
    fn borrowed_values() {
        fn foo(a: &str) {
            // borrowed values without a static lifetime need to be converted into owned values
            let endpoint = Endpoint::builder().url(a.to_string()).build();
            assert_eq!(endpoint.url(), a);
        }

        foo("asdf");
    }
}
