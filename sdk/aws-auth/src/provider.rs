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
use std::marker::PhantomData;
use std::pin::Pin;
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
                write!(f, "The provider could not provide credentials or required configuration was not set")
            }
            CredentialsError::ProviderTimedOut(d) => write!(
                f,
                "Credentials provider timed out after {} seconds",
                d.as_secs()
            ),
            CredentialsError::Unhandled(err) => write!(f, "Unexpected credentials error: {}", err),
            CredentialsError::InvalidConfiguration(err) => {
                write!(f, "The credentials provider was not properly: {}", err)
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

pub type CredentialsResult = Result<Credentials, CredentialsError>;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

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
pub struct AsyncProvideCredentialsFn<'c, T, F>
where
    T: Fn() -> F + Send + Sync + 'c,
    F: Future<Output = CredentialsResult> + Send + 'static,
{
    f: T,
    phantom: PhantomData<&'c T>,
}

impl<'c, T, F> AsyncProvideCredentials for AsyncProvideCredentialsFn<'c, T, F>
where
    T: Fn() -> F + Send + Sync + 'c,
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
pub fn async_provide_credentials_fn<'c, T, F>(f: T) -> AsyncProvideCredentialsFn<'c, T, F>
where
    T: Fn() -> F + Send + Sync + 'c,
    F: Future<Output = CredentialsResult> + Send + 'static,
{
    AsyncProvideCredentialsFn {
        f,
        phantom: Default::default(),
    }
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
    use crate::provider::{
        async_provide_credentials_fn, AsyncProvideCredentials, BoxFuture, CredentialsResult,
    };
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

    // Test that the closure passed to `async_provide_credentials_fn` is allowed to borrow things
    #[tokio::test]
    async fn async_provide_credentials_fn_closure_can_borrow() {
        fn check_is_str_ref(_input: &str) {}
        async fn test_async_provider(input: String) -> CredentialsResult {
            Ok(Credentials::from_keys(&input, &input, None))
        }

        let things_to_borrow = vec!["one".to_string(), "two".to_string()];

        let mut providers = Vec::new();
        for thing in &things_to_borrow {
            let provider = async_provide_credentials_fn(move || {
                check_is_str_ref(thing);
                test_async_provider(thing.into())
            });
            providers.push(provider);
        }

        let (two, one) = (providers.pop().unwrap(), providers.pop().unwrap());
        assert_eq!(
            "one",
            one.provide_credentials().await.unwrap().access_key_id()
        );
        assert_eq!(
            "two",
            two.provide_credentials().await.unwrap().access_key_id()
        );
    }
}
