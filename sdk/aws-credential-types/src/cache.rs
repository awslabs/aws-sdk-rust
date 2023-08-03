/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types and traits for enabling caching

mod expiring_cache;
mod lazy_caching;
mod no_caching;

pub use expiring_cache::ExpiringCache;
pub use lazy_caching::Builder as LazyBuilder;
use no_caching::NoCredentialsCache;

use crate::provider::{future, SharedCredentialsProvider};
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::sync::Arc;

/// Asynchronous Cached Credentials Provider
pub trait ProvideCachedCredentials: Send + Sync + std::fmt::Debug {
    /// Returns a future that provides cached credentials.
    fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a;
}

/// Credentials cache wrapper that may be shared
///
/// Newtype wrapper around `ProvideCachedCredentials` that implements `Clone` using an internal
/// `Arc`.
#[derive(Clone, Debug)]
pub struct SharedCredentialsCache(Arc<dyn ProvideCachedCredentials>);

impl SharedCredentialsCache {
    /// Create a new `SharedCredentialsCache` from `ProvideCachedCredentials`
    ///
    /// The given `cache` will be wrapped in an internal `Arc`. If your
    /// cache is already in an `Arc`, use `SharedCredentialsCache::from(cache)` instead.
    pub fn new(provider: impl ProvideCachedCredentials + 'static) -> Self {
        Self(Arc::new(provider))
    }
}

impl AsRef<dyn ProvideCachedCredentials> for SharedCredentialsCache {
    fn as_ref(&self) -> &(dyn ProvideCachedCredentials + 'static) {
        self.0.as_ref()
    }
}

impl From<Arc<dyn ProvideCachedCredentials>> for SharedCredentialsCache {
    fn from(cache: Arc<dyn ProvideCachedCredentials>) -> Self {
        SharedCredentialsCache(cache)
    }
}

impl ProvideCachedCredentials for SharedCredentialsCache {
    fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        self.0.provide_cached_credentials()
    }
}

impl Storable for SharedCredentialsCache {
    type Storer = StoreReplace<SharedCredentialsCache>;
}

#[derive(Clone, Debug)]
pub(crate) enum Inner {
    Lazy(lazy_caching::Builder),
    NoCaching,
}

/// `CredentialsCache` allows for configuring and creating a credentials cache.
///
/// # Examples
///
/// ```no_run
/// use aws_credential_types::Credentials;
/// use aws_credential_types::cache::CredentialsCache;
/// use aws_credential_types::credential_fn::provide_credentials_fn;
/// use aws_credential_types::provider::SharedCredentialsProvider;
///
/// let credentials_cache = CredentialsCache::lazy_builder()
///     .into_credentials_cache()
///     .create_cache(SharedCredentialsProvider::new(provide_credentials_fn(|| async {
///         // An async process to retrieve credentials would go here:
///         Ok(Credentials::new(
///             "example",
///             "example",
///             None,
///             None,
///             "my_provider_name"
///         ))
///     })));
/// ```
#[derive(Clone, Debug)]
pub struct CredentialsCache {
    pub(crate) inner: Inner,
}

impl CredentialsCache {
    /// Creates a [`CredentialsCache`] from the default [`LazyBuilder`].
    pub fn lazy() -> Self {
        Self::lazy_builder().into_credentials_cache()
    }

    /// Returns the default [`LazyBuilder`].
    pub fn lazy_builder() -> LazyBuilder {
        lazy_caching::Builder::new()
    }

    /// Creates a [`CredentialsCache`] that offers no caching ability.
    pub fn no_caching() -> Self {
        Self {
            inner: Inner::NoCaching,
        }
    }

    /// Creates a [`SharedCredentialsCache`] wrapping a concrete caching implementation.
    pub fn create_cache(self, provider: SharedCredentialsProvider) -> SharedCredentialsCache {
        match self.inner {
            Inner::Lazy(builder) => SharedCredentialsCache::new(builder.build(provider)),
            Inner::NoCaching => SharedCredentialsCache::new(NoCredentialsCache::new(provider)),
        }
    }
}

impl Storable for CredentialsCache {
    type Storer = StoreReplace<CredentialsCache>;
}
