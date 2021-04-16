/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

pub mod provider;

use smithy_http::property_bag::PropertyBag;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use std::time::SystemTime;

/// AWS SDK Credentials
///
/// An opaque struct representing credentials that may be used in an AWS SDK, modeled on
/// the [CRT credentials implementation](https://github.com/awslabs/aws-c-auth/blob/main/source/credentials.c).
///
/// Future design note: It may be desirable to make Credentials cheap to clone because they are cloned frequently.
#[derive(Clone)]
pub struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,

    /// Credential Expiry
    ///
    /// A timepoint at which the credentials should no longer
    /// be used because they have expired. The primary purpose of this value is to allow
    /// credentials to communicate to the caching provider when they need to be refreshed.
    ///
    /// If these credentials never expire, this value will be set to `None`
    expires_after: Option<SystemTime>,

    provider_name: &'static str,
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut creds = f.debug_struct("Credentials");
        creds.field("provider_name", &self.provider_name);
        creds.finish()
    }
}

const STATIC_CREDENTIALS: &str = "Static";
impl Credentials {
    pub fn from_keys(
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
        session_token: Option<String>,
    ) -> Self {
        Credentials {
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            session_token,
            expires_after: None,

            provider_name: STATIC_CREDENTIALS,
        }
    }

    pub fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    pub fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum CredentialsError {
    CredentialsNotLoaded,
    Unhandled(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for CredentialsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsError::CredentialsNotLoaded => write!(f, "CredentialsNotLoaded"),
            CredentialsError::Unhandled(err) => write!(f, "{}", err),
        }
    }
}

impl Error for CredentialsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CredentialsError::Unhandled(e) => Some(e.as_ref() as _),
            _ => None,
        }
    }
}

pub type CredentialsProvider = Arc<dyn ProvideCredentials>;

/// A Credentials Provider
///
/// This interface is intentionally NOT async. Credential providers should provide a separate
/// async method to drive refresh (eg. in a background task).
///
/// Pending future design iteration, an async credentials provider may be introduced.
pub trait ProvideCredentials: Send + Sync {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError>;
}

pub fn default_provider() -> impl ProvideCredentials {
    // TODO: this should be a chain based on the CRT
    provider::EnvironmentVariableCredentialsProvider::new()
}

impl ProvideCredentials for Credentials {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError> {
        Ok(self.clone())
    }
}

pub fn set_provider(config: &mut PropertyBag, provider: Arc<dyn ProvideCredentials>) {
    config.insert(provider);
}
