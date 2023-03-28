/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP Auth Location

use std::cmp::PartialEq;
use std::fmt::Debug;

use crate::error::{AuthError, AuthErrorKind};

/// Enum for describing where the HTTP Auth can be placed.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum HttpAuthLocation {
    /// In the HTTP header.
    #[default]
    Header,
    /// In the query string of the URL.
    Query,
}

impl HttpAuthLocation {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Query => "query",
        }
    }
}

impl TryFrom<&str> for HttpAuthLocation {
    type Error = AuthError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "header" => Ok(Self::Header),
            "query" => Ok(Self::Query),
            _ => Err(AuthError::from(AuthErrorKind::InvalidLocation)),
        }
    }
}

impl TryFrom<String> for HttpAuthLocation {
    type Error = AuthError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl AsRef<str> for HttpAuthLocation {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for HttpAuthLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::HttpAuthLocation;
    use crate::error::{AuthError, AuthErrorKind};

    #[test]
    fn fails_if_location_is_invalid() {
        let actual = HttpAuthLocation::try_from("invalid").unwrap_err();
        let expected = AuthError::from(AuthErrorKind::InvalidLocation);
        assert_eq!(actual, expected);
    }
}
