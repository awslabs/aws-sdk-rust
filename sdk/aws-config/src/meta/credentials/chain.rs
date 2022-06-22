/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use aws_types::credentials::{self, future, CredentialsError, ProvideCredentials};
use tracing::Instrument;

/// Credentials provider that checks a series of inner providers
///
/// Each provider will be evaluated in order:
/// * If a provider returns valid [`Credentials`](aws_types::Credentials) they will be returned immediately.
///   No other credential providers will be used.
/// * Otherwise, if a provider returns
///   [`CredentialsError::CredentialsNotLoaded`](aws_types::credentials::CredentialsError::CredentialsNotLoaded),
///   the next provider will be checked.
/// * Finally, if a provider returns any other error condition, an error will be returned immediately.
///
/// # Examples
///
/// ```no_run
/// # fn example() {
/// use aws_config::meta::credentials::CredentialsProviderChain;
/// use aws_config::environment::credentials::EnvironmentVariableCredentialsProvider;
/// use aws_config::profile::ProfileFileCredentialsProvider;
///
/// let provider = CredentialsProviderChain::first_try("Environment", EnvironmentVariableCredentialsProvider::new())
///     .or_else("Profile", ProfileFileCredentialsProvider::builder().build());
/// # }
/// ```
#[derive(Debug)]
pub struct CredentialsProviderChain {
    providers: Vec<(Cow<'static, str>, Box<dyn ProvideCredentials>)>,
}

impl CredentialsProviderChain {
    /// Create a `CredentialsProviderChain` that begins by evaluating this provider
    pub fn first_try(
        name: impl Into<Cow<'static, str>>,
        provider: impl ProvideCredentials + 'static,
    ) -> Self {
        CredentialsProviderChain {
            providers: vec![(name.into(), Box::new(provider))],
        }
    }

    /// Add a fallback provider to the credentials provider chain
    pub fn or_else(
        mut self,
        name: impl Into<Cow<'static, str>>,
        provider: impl ProvideCredentials + 'static,
    ) -> Self {
        self.providers.push((name.into(), Box::new(provider)));
        self
    }

    /// Add a fallback to the default provider chain
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    pub async fn or_default_provider(self) -> Self {
        self.or_else(
            "DefaultProviderChain",
            crate::default_provider::credentials::default_provider().await,
        )
    }

    /// Creates a credential provider chain that starts with the default provider
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    pub async fn default_provider() -> Self {
        Self::first_try(
            "DefaultProviderChain",
            crate::default_provider::credentials::default_provider().await,
        )
    }

    async fn credentials(&self) -> credentials::Result {
        for (name, provider) in &self.providers {
            let span = tracing::debug_span!("load_credentials", provider = %name);
            match provider.provide_credentials().instrument(span).await {
                Ok(credentials) => {
                    tracing::info!(provider = %name, "loaded credentials");
                    return Ok(credentials);
                }
                Err(CredentialsError::CredentialsNotLoaded { context, .. }) => {
                    tracing::info!(provider = %name, context = %context, "provider in chain did not provide credentials");
                }
                Err(e) => {
                    tracing::warn!(provider = %name, error = %e, "provider failed to provide credentials");
                    return Err(e);
                }
            }
        }
        Err(CredentialsError::not_loaded(
            "no providers in chain provided credentials",
        ))
    }
}

impl ProvideCredentials for CredentialsProviderChain {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'_>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}
