/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::Credentials;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
#[non_exhaustive]
pub enum CredentialsError {
    /// No credentials were available for this provider
    CredentialsNotLoaded,

    /// Loading credentials from this provider exceeded the maximum allowed duration
    ProviderTimedOut(Duration),

    /// The provider was given an invalid configuration
    ///
    /// For example:
    /// - syntax error in ~/.aws/config
    /// - assume role profile that forms an infinite loop
    InvalidConfiguration(Box<dyn Error + Send + Sync + 'static>),

    /// The provider experienced an error during credential resolution
    ///
    /// This may include errors like a 503 from STS or a file system error when attempting to
    /// read a configuration file.
    ProviderError(Box<dyn Error + Send + Sync + 'static>),

    /// An unexpected error occured during credential resolution
    ///
    /// If the error is something that can occur during expected usage of a provider, `ProviderError`
    /// should be returned instead. Unhandled is reserved for exceptional cases, for example:
    /// - Returned data not UTF-8
    /// - A provider returns data that is missing required fields
    Unhandled(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for CredentialsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsError::CredentialsNotLoaded => {
                write!(f, "The credential provider was not enabled")
            }
            CredentialsError::ProviderTimedOut(d) => write!(
                f,
                "Credentials provider timed out after {} seconds",
                d.as_secs()
            ),
            CredentialsError::Unhandled(err) => write!(f, "Unexpected credentials error: {}", err),
            CredentialsError::InvalidConfiguration(err) => {
                write!(
                    f,
                    "The credentials provider was not properly configured: {}",
                    err
                )
            }
            CredentialsError::ProviderError(err) => {
                write!(f, "An error occured while loading credentials: {}", err)
            }
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

pub type Result = std::result::Result<Credentials, CredentialsError>;

pub mod future {
    use smithy_async::future::now_or_later::NowOrLater;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
    pub struct ProvideCredentials<'a>(NowOrLater<super::Result, BoxFuture<'a, super::Result>>);

    impl<'a> ProvideCredentials<'a> {
        pub fn new(future: impl Future<Output = super::Result> + Send + 'a) -> Self {
            ProvideCredentials(NowOrLater::new(Box::pin(future)))
        }

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
