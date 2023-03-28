/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_credential_types::cache::{ProvideCachedCredentials, SharedCredentialsCache};
use aws_credential_types::provider::error::CredentialsError;
use aws_smithy_http::middleware::AsyncMapRequest;
use aws_smithy_http::operation::Request;
use aws_smithy_http::property_bag::PropertyBag;
use std::future::Future;
use std::pin::Pin;

/// Sets the credentials cache in the given property bag.
pub fn set_credentials_cache(bag: &mut PropertyBag, cache: SharedCredentialsCache) {
    bag.insert(cache);
}

/// Middleware stage that loads credentials from a [SharedCredentialsCache](aws_credential_types::cache::SharedCredentialsCache)
/// and places them in the property bag of the request.
///
/// [CredentialsStage] implements [`AsyncMapRequest`](aws_smithy_http::middleware::AsyncMapRequest), and:
/// 1. Retrieves a `SharedCredentialsCache` from the property bag.
/// 2. Calls the credential cache's `provide_cached_credentials` and awaits its result.
/// 3. Places returned `Credentials` into the property bag to drive downstream signing middleware.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct CredentialsStage;

impl CredentialsStage {
    /// Creates a new credentials stage.
    pub fn new() -> Self {
        CredentialsStage
    }

    async fn load_creds(mut request: Request) -> Result<Request, CredentialsStageError> {
        let credentials_cache = request
            .properties()
            .get::<SharedCredentialsCache>()
            .cloned();
        let credentials_cache = match credentials_cache {
            Some(credentials_cache) => credentials_cache,
            None => {
                tracing::info!("no credentials cache for request");
                return Ok(request);
            }
        };
        match credentials_cache.provide_cached_credentials().await {
            Ok(creds) => {
                request.properties_mut().insert(creds);
            }
            // ignore the case where there is no credentials cache wired up
            Err(CredentialsError::CredentialsNotLoaded { .. }) => {
                tracing::info!("credentials cache returned CredentialsNotLoaded, ignoring")
            }
            // if we get another error class, there is probably something actually wrong that the user will
            // want to know about
            Err(other) => return Err(other.into()),
        }
        Ok(request)
    }
}

mod error {
    use aws_credential_types::provider::error::CredentialsError;
    use std::error::Error as StdError;
    use std::fmt;

    /// Failures that can occur in the credentials middleware.
    #[derive(Debug)]
    pub struct CredentialsStageError {
        source: CredentialsError,
    }

    impl StdError for CredentialsStageError {
        fn source(&self) -> Option<&(dyn StdError + 'static)> {
            Some(&self.source as _)
        }
    }

    impl fmt::Display for CredentialsStageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "failed to load credentials from the credentials cache")
        }
    }

    impl From<CredentialsError> for CredentialsStageError {
        fn from(source: CredentialsError) -> Self {
            CredentialsStageError { source }
        }
    }
}

pub use error::*;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

impl AsyncMapRequest for CredentialsStage {
    type Error = CredentialsStageError;
    type Future = Pin<Box<dyn Future<Output = Result<Request, Self::Error>> + Send + 'static>>;

    fn name(&self) -> &'static str {
        "retrieve_credentials"
    }

    fn apply(&self, request: Request) -> BoxFuture<Result<Request, Self::Error>> {
        Box::pin(Self::load_creds(request))
    }
}

#[cfg(test)]
mod tests {
    use super::set_credentials_cache;
    use super::CredentialsStage;
    use aws_credential_types::cache::{
        CredentialsCache, ProvideCachedCredentials, SharedCredentialsCache,
    };
    use aws_credential_types::credential_fn::provide_credentials_fn;
    use aws_credential_types::provider::SharedCredentialsProvider;
    use aws_credential_types::provider::{error::CredentialsError, future};
    use aws_credential_types::Credentials;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::middleware::AsyncMapRequest;
    use aws_smithy_http::operation;

    #[derive(Debug)]
    struct Unhandled;
    impl ProvideCachedCredentials for Unhandled {
        fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            future::ProvideCredentials::ready(Err(CredentialsError::unhandled("whoops")))
        }
    }

    #[derive(Debug)]
    struct NoCreds;
    impl ProvideCachedCredentials for NoCreds {
        fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            future::ProvideCredentials::ready(Err(CredentialsError::not_loaded("no creds")))
        }
    }

    #[tokio::test]
    async fn no_credential_cache_is_ok() {
        let req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        CredentialsStage::new()
            .apply(req)
            .await
            .expect("no credentials cache should not populate credentials");
    }

    #[tokio::test]
    async fn credentials_cache_failure_is_failure() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_credentials_cache(
            &mut req.properties_mut(),
            SharedCredentialsCache::new(Unhandled),
        );
        CredentialsStage::new()
            .apply(req)
            .await
            .expect_err("no credentials cache should not populate credentials");
    }

    #[tokio::test]
    async fn credentials_not_loaded_is_ok() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_credentials_cache(
            &mut req.properties_mut(),
            SharedCredentialsCache::new(NoCreds),
        );
        CredentialsStage::new()
            .apply(req)
            .await
            .expect("credentials not loaded is OK");
    }

    #[tokio::test]
    async fn async_map_request_apply_populates_credentials() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        let credentials_cache = CredentialsCache::lazy_builder()
            .into_credentials_cache()
            .create_cache(SharedCredentialsProvider::new(provide_credentials_fn(
                || async { Ok(Credentials::for_tests()) },
            )));
        set_credentials_cache(&mut req.properties_mut(), credentials_cache);
        let req = CredentialsStage::new()
            .apply(req)
            .await
            .expect("credentials cache is in the bag; should succeed");
        assert!(
            req.properties().get::<Credentials>().is_some(),
            "it should set credentials on the request config"
        );
    }
}
