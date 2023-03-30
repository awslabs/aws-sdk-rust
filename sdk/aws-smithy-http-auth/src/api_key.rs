/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP Auth API Key

use std::cmp::PartialEq;
use std::fmt::Debug;
use std::sync::Arc;
use zeroize::Zeroizing;

/// Authentication configuration to connect to a Smithy Service
#[derive(Clone, Eq, PartialEq)]
pub struct AuthApiKey(Arc<Inner>);

#[derive(Clone, Eq, PartialEq)]
struct Inner {
    api_key: Zeroizing<String>,
}

impl Debug for AuthApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut auth_api_key = f.debug_struct("AuthApiKey");
        auth_api_key.field("api_key", &"** redacted **").finish()
    }
}

impl AuthApiKey {
    /// Constructs a new API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self(Arc::new(Inner {
            api_key: Zeroizing::new(api_key.into()),
        }))
    }

    /// Returns the underlying api key.
    pub fn api_key(&self) -> &str {
        &self.0.api_key
    }
}

impl From<&str> for AuthApiKey {
    fn from(api_key: &str) -> Self {
        Self::from(api_key.to_owned())
    }
}

impl From<String> for AuthApiKey {
    fn from(api_key: String) -> Self {
        Self(Arc::new(Inner {
            api_key: Zeroizing::new(api_key),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::AuthApiKey;

    #[test]
    fn api_key_is_equal() {
        let api_key_a: AuthApiKey = "some-api-key".into();
        let api_key_b = AuthApiKey::new("some-api-key");
        assert_eq!(api_key_a, api_key_b);
    }

    #[test]
    fn api_key_is_different() {
        let api_key_a = AuthApiKey::new("some-api-key");
        let api_key_b: AuthApiKey = String::from("another-api-key").into();
        assert_ne!(api_key_a, api_key_b);
    }
}
