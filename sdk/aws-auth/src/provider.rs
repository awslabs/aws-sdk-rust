/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! AWS credential providers, generic caching provider implementations, and traits to implement custom providers.
//!
//! Credentials providers acquire AWS credentials from environment variables, files,
//! or calls to AWS services such as STS. Custom credential provider implementations can
//! be provided by implementing [`ProvideCredentials`] for synchronous use-cases, or
//! [`AsyncProvideCredentials`] for async use-cases. Generic credential caching implementations,
//! for example,
//! [`LazyCachingCredentialsProvider`](crate::provider::lazy_caching::LazyCachingCredentialsProvider),
//! are also provided as part of this module.

mod cache;
pub mod env;
pub mod lazy_caching;
mod time;

use crate::Credentials;
use smithy_http::property_bag::PropertyBag;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::future::{self, Future};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
#[non_exhaustive]
pub enum CredentialsError {
    CredentialsNotLoaded,
    ProviderTimedOut(Duration),
    Unhandled(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for CredentialsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CredentialsError::CredentialsNotLoaded => write!(f, "CredentialsNotLoaded"),
            CredentialsError::ProviderTimedOut(d) => write!(
                f,
                "Credentials provider timed out after {} seconds",
                d.as_secs()
            ),
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

pub type CredentialsResult = Result<Credentials, CredentialsError>;
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// An asynchronous credentials provider
///
/// If your use-case is synchronous, you should implement [`ProvideCredentials`] instead. Otherwise,
/// consider using [`async_provide_credentials_fn`] with a closure rather than directly implementing
/// this trait.
pub trait AsyncProvideCredentials: Send + Sync {
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a;
}

pub type CredentialsProvider = Arc<dyn AsyncProvideCredentials>;

/// A [`AsyncProvideCredentials`] implemented by a closure.
///
/// See [`async_provide_credentials_fn`] for more details.
#[derive(Copy, Clone)]
pub struct AsyncProvideCredentialsFn<T: Send + Sync> {
    f: T,
}

impl<T, F> AsyncProvideCredentials for AsyncProvideCredentialsFn<T>
where
    T: Fn() -> F + Send + Sync,
    F: Future<Output = CredentialsResult> + Send + 'static,
{
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a,
    {
        Box::pin((self.f)())
    }
}

/// Returns a new [`AsyncProvideCredentialsFn`] with the given closure. This allows you
/// to create an [`AsyncProvideCredentials`] implementation from an async block that returns
/// a [`CredentialsResult`].
///
/// # Example
///
/// ```
/// use aws_auth::Credentials;
/// use aws_auth::provider::async_provide_credentials_fn;
///
/// async fn load_credentials() -> Credentials {
///     todo!()
/// }
///
/// async_provide_credentials_fn(|| async {
///     // Async process to retrieve credentials goes here
///     let credentials = load_credentials().await;
///     Ok(credentials)
/// });
/// ```
pub fn async_provide_credentials_fn<T, F>(f: T) -> AsyncProvideCredentialsFn<T>
where
    T: Fn() -> F + Send + Sync,
    F: Future<Output = CredentialsResult> + Send + 'static,
{
    AsyncProvideCredentialsFn { f }
}

/// A synchronous credentials provider
///
/// This is offered as a convenience for credential provider implementations that don't
/// need to be async. Otherwise, implement [`AsyncProvideCredentials`].
pub trait ProvideCredentials: Send + Sync {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError>;
}

impl<T> AsyncProvideCredentials for T
where
    T: ProvideCredentials,
{
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a,
    {
        let result = self.provide_credentials();
        Box::pin(future::ready(result))
    }
}

pub fn default_provider() -> impl AsyncProvideCredentials {
    // TODO: this should be a chain based on the CRT
    env::EnvironmentVariableCredentialsProvider::new()
}

impl ProvideCredentials for Credentials {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError> {
        Ok(self.clone())
    }
}

pub fn set_provider(config: &mut PropertyBag, provider: Arc<dyn AsyncProvideCredentials>) {
    config.insert(provider);
}

#[cfg(test)]
mod test {
    use crate::provider::{AsyncProvideCredentials, BoxFuture, CredentialsResult};
    use crate::Credentials;
    use async_trait::async_trait;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn creds_are_send_sync() {
        assert_send_sync::<Credentials>()
    }

    #[async_trait]
    trait AnotherTrait: Send + Sync {
        async fn creds(&self) -> Credentials;
    }

    struct AnotherTraitWrapper<T> {
        inner: T,
    }

    impl<T: AnotherTrait> AsyncProvideCredentials for AnotherTraitWrapper<T> {
        fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
        where
            Self: 'a,
        {
            let inner_fut = self.inner.creds();
            Box::pin(async move { Ok(inner_fut.await) })
        }
    }
}
