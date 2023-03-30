/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP Auth Definition

use crate::location::HttpAuthLocation;
use std::cmp::PartialEq;
use std::fmt::Debug;

/// A HTTP-specific authentication scheme that sends an arbitrary
/// auth value in a header or query string parameter.
// As described in the Smithy documentation:
// https://github.com/awslabs/smithy/blob/main/smithy-model/src/main/resources/software/amazon/smithy/model/loader/prelude.smithy
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HttpAuthDefinition {
    /// Defines the location of where the Auth is serialized.
    location: HttpAuthLocation,

    /// Defines the name of the HTTP header or query string parameter
    /// that contains the Auth.
    name: String,

    /// Defines the security scheme to use on the `Authorization` header value.
    /// This can only be set if the "location" property is set to [`HttpAuthLocation::Header`].
    scheme: Option<String>,
}

impl HttpAuthDefinition {
    /// Returns a builder for `HttpAuthDefinition`.
    pub fn builder() -> http_auth_definition::Builder {
        http_auth_definition::Builder::default()
    }

    /// Constructs a new HTTP auth definition in header.
    pub fn header<N, S>(header_name: N, scheme: S) -> Self
    where
        N: Into<String>,
        S: Into<Option<String>>,
    {
        let mut builder = Self::builder()
            .location(HttpAuthLocation::Header)
            .name(header_name);
        let scheme: Option<String> = scheme.into();
        if scheme.is_some() {
            builder.set_scheme(scheme);
        }
        builder.build()
    }

    /// Constructs a new HTTP auth definition following the RFC 2617 for Basic Auth.
    pub fn basic_auth() -> Self {
        Self::builder()
            .location(HttpAuthLocation::Header)
            .name("Authorization".to_owned())
            .scheme("Basic".to_owned())
            .build()
    }

    /// Constructs a new HTTP auth definition following the RFC 2617 for Digest Auth.
    pub fn digest_auth() -> Self {
        Self::builder()
            .location(HttpAuthLocation::Header)
            .name("Authorization".to_owned())
            .scheme("Digest".to_owned())
            .build()
    }

    /// Constructs a new HTTP auth definition following the RFC 6750 for Bearer Auth.
    pub fn bearer_auth() -> Self {
        Self::builder()
            .location(HttpAuthLocation::Header)
            .name("Authorization".to_owned())
            .scheme("Bearer".to_owned())
            .build()
    }

    /// Constructs a new HTTP auth definition in query string.
    pub fn query(name: impl Into<String>) -> Self {
        Self::builder()
            .location(HttpAuthLocation::Query)
            .name(name.into())
            .build()
    }

    /// Returns the HTTP auth location.
    pub fn location(&self) -> HttpAuthLocation {
        self.location
    }

    /// Returns the HTTP auth name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the HTTP auth scheme.
    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }
}

/// Types associated with [`HttpAuthDefinition`].
pub mod http_auth_definition {
    use super::HttpAuthDefinition;
    use crate::{
        definition::HttpAuthLocation,
        error::{AuthError, AuthErrorKind},
    };

    /// A builder for [`HttpAuthDefinition`].
    #[derive(Debug, Default)]
    pub struct Builder {
        location: Option<HttpAuthLocation>,
        name: Option<String>,
        scheme: Option<String>,
    }

    impl Builder {
        /// Sets the HTTP auth location.
        pub fn location(mut self, location: HttpAuthLocation) -> Self {
            self.location = Some(location);
            self
        }

        /// Sets the HTTP auth location.
        pub fn set_location(&mut self, location: Option<HttpAuthLocation>) -> &mut Self {
            self.location = location;
            self
        }

        /// Sets the the HTTP auth name.
        pub fn name(mut self, name: impl Into<String>) -> Self {
            self.name = Some(name.into());
            self
        }

        /// Sets the the HTTP auth name.
        pub fn set_name(&mut self, name: Option<String>) -> &mut Self {
            self.name = name;
            self
        }

        /// Sets the HTTP auth scheme.
        pub fn scheme(mut self, scheme: impl Into<String>) -> Self {
            self.scheme = Some(scheme.into());
            self
        }

        /// Sets the HTTP auth scheme.
        pub fn set_scheme(&mut self, scheme: Option<String>) -> &mut Self {
            self.scheme = scheme;
            self
        }

        /// Constructs a [`HttpAuthDefinition`] from the builder.
        pub fn build(self) -> HttpAuthDefinition {
            if self.scheme.is_some()
                && self
                    .name
                    .as_deref()
                    .map_or("".to_string(), |s| s.to_ascii_lowercase())
                    != "authorization"
            {
                // Stop execution because the Smithy model should not contain such combination.
                // Otherwise, this would cause unexpected behavior in the SDK.
                panic!("{}", AuthError::from(AuthErrorKind::SchemeNotAllowed));
            }
            HttpAuthDefinition {
                location: self.location.unwrap_or_else(|| {
                    panic!(
                        "{}",
                        AuthError::from(AuthErrorKind::MissingRequiredField("location"))
                    )
                }),
                name: self.name.unwrap_or_else(|| {
                    panic!(
                        "{}",
                        AuthError::from(AuthErrorKind::MissingRequiredField("name"))
                    )
                }),
                scheme: self.scheme,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::HttpAuthDefinition;
    use crate::location::HttpAuthLocation;

    #[test]
    fn definition_for_header_without_scheme() {
        let definition = HttpAuthDefinition::header("Header", None);
        assert_eq!(definition.location, HttpAuthLocation::Header);
        assert_eq!(definition.name, "Header");
        assert_eq!(definition.scheme, None);
    }

    #[test]
    fn definition_for_authorization_header_with_scheme() {
        let definition = HttpAuthDefinition::header("authorization", "Scheme".to_owned());
        assert_eq!(definition.location(), HttpAuthLocation::Header);
        assert_eq!(definition.name(), "authorization");
        assert_eq!(definition.scheme(), Some("Scheme"));
    }

    #[test]
    #[should_panic]
    fn definition_fails_with_scheme_not_allowed() {
        let _ = HttpAuthDefinition::header("Invalid".to_owned(), "Scheme".to_owned());
    }

    #[test]
    fn definition_for_basic() {
        let definition = HttpAuthDefinition::basic_auth();
        assert_eq!(
            definition,
            HttpAuthDefinition {
                location: HttpAuthLocation::Header,
                name: "Authorization".to_owned(),
                scheme: Some("Basic".to_owned()),
            }
        );
    }

    #[test]
    fn definition_for_digest() {
        let definition = HttpAuthDefinition::digest_auth();
        assert_eq!(definition.location(), HttpAuthLocation::Header);
        assert_eq!(definition.name(), "Authorization");
        assert_eq!(definition.scheme(), Some("Digest"));
    }

    #[test]
    fn definition_for_bearer_token() {
        let definition = HttpAuthDefinition::bearer_auth();
        assert_eq!(definition.location(), HttpAuthLocation::Header);
        assert_eq!(definition.name(), "Authorization");
        assert_eq!(definition.scheme(), Some("Bearer"));
    }

    #[test]
    fn definition_for_query() {
        let definition = HttpAuthDefinition::query("query_key");
        assert_eq!(definition.location(), HttpAuthLocation::Query);
        assert_eq!(definition.name(), "query_key");
        assert_eq!(definition.scheme(), None);
    }
}
