/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::SystemTime;
use zeroize::Zeroizing;

/// AWS SDK Credentials
///
/// An opaque struct representing credentials that may be used in an AWS SDK, modeled on
/// the [CRT credentials implementation](https://github.com/awslabs/aws-c-auth/blob/main/source/credentials.c).
///
/// When `Credentials` is dropped, its contents are zeroed in memory. Credentials uses an interior Arc to ensure
/// that even when cloned, credentials don't exist in multiple memory locations.
#[derive(Clone, Eq, PartialEq)]
pub struct Credentials(Arc<Inner>);

#[derive(Clone, Eq, PartialEq)]
struct Inner {
    access_key_id: Zeroizing<String>,
    secret_access_key: Zeroizing<String>,
    session_token: Zeroizing<Option<String>>,

    /// Credential Expiry
    ///
    /// A SystemTime at which the credentials should no longer be used because they have expired.
    /// The primary purpose of this value is to allow credentials to communicate to the caching
    /// provider when they need to be refreshed.
    ///
    /// If these credentials never expire, this value will be set to `None`
    expires_after: Option<SystemTime>,

    provider_name: &'static str,
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut creds = f.debug_struct("Credentials");
        creds
            .field("provider_name", &self.0.provider_name)
            .field("access_key_id", &self.0.access_key_id.as_str())
            .field("secret_access_key", &"** redacted **");
        if let Some(expiry) = self.expiry() {
            // TODO: format the expiry nicely
            creds.field("expires_after", &expiry);
        }
        creds.finish()
    }
}

const STATIC_CREDENTIALS: &str = "Static";

impl Credentials {
    pub fn new(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: Option<String>,
        expires_after: Option<SystemTime>,
        provider_name: &'static str,
    ) -> Self {
        Credentials(Arc::new(Inner {
            access_key_id: Zeroizing::new(access_key_id.into()),
            secret_access_key: Zeroizing::new(secret_access_key.into()),
            session_token: Zeroizing::new(session_token),
            expires_after,
            provider_name,
        }))
    }

    pub fn from_keys(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: Option<String>,
    ) -> Self {
        Self::new(
            access_key_id,
            secret_access_key,
            session_token,
            None,
            STATIC_CREDENTIALS,
        )
    }

    pub fn access_key_id(&self) -> &str {
        &self.0.access_key_id
    }

    pub fn secret_access_key(&self) -> &str {
        &self.0.secret_access_key
    }

    pub fn expiry(&self) -> Option<SystemTime> {
        self.0.expires_after
    }

    pub fn expiry_mut(&mut self) -> &mut Option<SystemTime> {
        &mut Arc::make_mut(&mut self.0).expires_after
    }

    pub fn session_token(&self) -> Option<&str> {
        self.0.session_token.as_deref()
    }
}
