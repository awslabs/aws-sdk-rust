/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::Credentials;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

/// Error returned when credentials failed to load.
#[derive(Debug)]
#[non_exhaustive]
pub enum CredentialsError {
    /// No credentials were available for this provider
    #[non_exhaustive]
    CredentialsNotLoaded {
        /// Underlying cause of the error.
        context: Box<dyn Error + Send + Sync + 'static>,
    },

    /// Loading credentials from this provider exceeded the maximum allowed duration
    #[non_exhaustive]
    ProviderTimedOut(Duration),

    /// The provider was given an invalid configuration
    ///
    /// For example:
    /// - syntax error in ~/.aws/config
    /// - assume role profile that forms an infinite loop
    #[non_exhaustive]
    InvalidConfiguration {
        /// Underlying cause of the error.
        cause: Box<dyn Error + Send + Sync + 'static>,
    },

    /// The provider experienced an error during credential resolution
    ///
    /// This may include errors like a 503 from STS or a file system error when attempting to
    /// read a configuration file.
    #[non_exhaustive]
    ProviderError {
        /// Underlying cause of the error.
        cause: Box<dyn Error + Send + Sync + 'static>,
    },

    /// An unexpected error occurred during credential resolution
    ///
    /// If the error is something that can occur during expected usage of a provider, `ProviderError`
    /// should be returned instead. Unhandled is reserved for exceptional cases, for example:
    /// - Returned data not UTF-8
    /// - A provider returns data that is missing required fields
    #[non_exhaustive]
    Unhandled {
        /// Underlying cause of the error.
        cause: Box<dyn Error + Send + Sync + 'static>,
    },
}

impl CredentialsError {
    /// The credentials provider did not provide credentials
    ///
    /// This error indicates the credentials provider was not enable or no configuration was set.
    /// This contrasts with [`invalid_configuration`](CredentialsError::InvalidConfiguration), indicating
    /// that the provider was configured in some way, but certain settings were invalid.
    pub fn not_loaded(context: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        CredentialsError::CredentialsNotLoaded {
            context: context.into(),
        }
    }

    /// An unexpected error occurred loading credentials from this provider
    ///
    /// Unhandled errors should not occur during normal operation and should be reserved for exceptional
    /// cases, such as a JSON API returning an output that was not parseable as JSON.
    pub fn unhandled(cause: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self::Unhandled {
            cause: cause.into(),
        }
    }

    /// The credentials provider returned an error
    ///
    /// Provider errors may occur during normal use of a credentials provider, e.g. a 503 when
    /// retrieving credentials from IMDS.
    pub fn provider_error(cause: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self::ProviderError {
            cause: cause.into(),
        }
    }

    /// The provided configuration for a provider was invalid
    pub fn invalid_configuration(cause: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self::InvalidConfiguration {
            cause: cause.into(),
        }
    }

    /// The credentials provider did not provide credentials within an allotted duration
    pub fn provider_timed_out(context: Duration) -> Self {
        Self::ProviderTimedOut(context)
    }
}

impl Display for CredentialsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsError::CredentialsNotLoaded { context } => {
                write!(f, "The credential provider was not enabled: {}", context)
            }
            CredentialsError::ProviderTimedOut(d) => write!(
                f,
                "Credentials provider timed out after {} seconds",
                d.as_secs()
            ),
            CredentialsError::Unhandled { cause } => {
                write!(f, "Unexpected credentials error: {}", cause)
            }
            CredentialsError::InvalidConfiguration { cause } => {
                write!(
                    f,
                    "The credentials provider was not properly configured: {}",
                    cause
                )
            }
            CredentialsError::ProviderError { cause } => {
                write!(f, "An error occurred while loading credentials: {}", cause)
            }
        }
    }
}

impl Error for CredentialsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CredentialsError::Unhandled { cause }
            | CredentialsError::ProviderError { cause }
            | CredentialsError::InvalidConfiguration { cause } => Some(cause.as_ref() as _),
            CredentialsError::CredentialsNotLoaded { context } => Some(context.as_ref() as _),
            _ => None,
        }
    }
}

/// Result type for credential providers.
pub type Result = std::result::Result<Credentials, CredentialsError>;

/// Convenience `ProvideCredentials` struct that implements the `ProvideCredentials` trait.
pub mod future {
    use aws_smithy_async::future::now_or_later::NowOrLater;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// Future new-type that the `ProvideCredentials` trait must return.
    #[derive(Debug)]
    pub struct ProvideCredentials<'a>(NowOrLater<super::Result, BoxFuture<'a, super::Result>>);

    impl<'a> ProvideCredentials<'a> {
        /// Creates a `ProvideCredentials` struct from a future.
        pub fn new(future: impl Future<Output = super::Result> + Send + 'a) -> Self {
            ProvideCredentials(NowOrLater::new(Box::pin(future)))
        }

        /// Creates a `ProvideCredentials` struct from a resolved credentials value.
        pub fn ready(credentials: super::Result) -> Self {
            ProvideCredentials(NowOrLater::ready(credentials))
        }
    }

    impl Future for ProvideCredentials<'_> {
        type Output = super::Result;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            Pin::new(&mut self.0).poll(cx)
        }
    }
}

/// Asynchronous Credentials Provider
pub trait ProvideCredentials: Send + Sync + Debug {
    /// Returns a future that provides credentials.
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a;
}

impl ProvideCredentials for Credentials {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::ready(Ok(self.clone()))
    }
}

impl ProvideCredentials for Arc<dyn ProvideCredentials> {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        self.as_ref().provide_credentials()
    }
}

/// Credentials Provider wrapper that may be shared
///
/// Newtype wrapper around ProvideCredentials that implements Clone using an internal
/// Arc.
#[derive(Clone, Debug)]
pub struct SharedCredentialsProvider(Arc<dyn ProvideCredentials>);

impl SharedCredentialsProvider {
    /// Create a new SharedCredentials provider from `ProvideCredentials`
    ///
    /// The given provider will be wrapped in an internal `Arc`. If your
    /// provider is already in an `Arc`, use `SharedCredentialsProvider::from(provider)` instead.
    pub fn new(provider: impl ProvideCredentials + 'static) -> Self {
        Self(Arc::new(provider))
    }
}

impl AsRef<dyn ProvideCredentials> for SharedCredentialsProvider {
    fn as_ref(&self) -> &(dyn ProvideCredentials + 'static) {
        self.0.as_ref()
    }
}

impl From<Arc<dyn ProvideCredentials>> for SharedCredentialsProvider {
    fn from(provider: Arc<dyn ProvideCredentials>) -> Self {
        SharedCredentialsProvider(provider)
    }
}

impl ProvideCredentials for SharedCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        self.0.provide_credentials()
    }
}
