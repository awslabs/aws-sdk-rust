/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use smithy_http::middleware::AsyncMapRequest;
use smithy_http::operation::Request;
use std::future::Future;
use std::pin::Pin;

/// Middleware stage that loads credentials from a [CredentialsProvider](aws_types::credentials::ProvideCredentials)
/// and places them in the property bag of the request.
///
/// [CredentialsStage] implements [`AsyncMapRequest`](smithy_http::middleware::AsyncMapRequest), and:
/// 1. Retrieves a `CredentialsProvider` from the property bag.
/// 2. Calls the credential provider's `provide_credentials` and awaits its result.
/// 3. Places returned `Credentials` into the property bad to drive downstream signing middleware.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct CredentialsStage;

impl CredentialsStage {
    pub fn new() -> Self {
        CredentialsStage
    }

    async fn load_creds(mut request: Request) -> Result<Request, CredentialsStageError> {
        let provider = request
            .properties()
            .get::<SharedCredentialsProvider>()
            .cloned();
        let provider = match provider {
            Some(provider) => provider,
            None => {
                tracing::info!("no credentials provider for request");
                return Ok(request);
            }
        };
        match provider.provide_credentials().await {
            Ok(creds) => {
                request.properties_mut().insert(creds);
            }
            // ignore the case where there is no provider wired up
            Err(CredentialsError::CredentialsNotLoaded) => {
                tracing::info!("provider returned CredentialsNotLoaded, ignoring")
            }
            // if we get another error class, there is probably something actually wrong that the user will
            // want to know about
            Err(other) => return Err(CredentialsStageError::CredentialsLoadingError(other)),
        }
        Ok(request)
    }
}

mod error {
    use aws_types::credentials::CredentialsError;
    use std::error::Error as StdError;
    use std::fmt;

    #[derive(Debug)]
    pub enum CredentialsStageError {
        MissingCredentialsProvider,
        CredentialsLoadingError(CredentialsError),
    }

    impl StdError for CredentialsStageError {}

    impl fmt::Display for CredentialsStageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use CredentialsStageError::*;
            match self {
                MissingCredentialsProvider => {
                    write!(f, "No credentials provider in the property bag")
                }
                CredentialsLoadingError(err) => write!(
                    f,
                    "Failed to load credentials from the credentials provider: {}",
                    err
                ),
            }
        }
    }

    impl From<CredentialsError> for CredentialsStageError {
        fn from(err: CredentialsError) -> Self {
            CredentialsStageError::CredentialsLoadingError(err)
        }
    }
}

use aws_types::credentials::{CredentialsError, ProvideCredentials, SharedCredentialsProvider};
pub use error::*;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

impl AsyncMapRequest for CredentialsStage {
    type Error = CredentialsStageError;
    type Future = Pin<Box<dyn Future<Output = Result<Request, Self::Error>> + Send + 'static>>;

    fn apply(&self, request: Request) -> BoxFuture<Result<Request, Self::Error>> {
        Box::pin(Self::load_creds(request))
    }
}

#[cfg(test)]
mod tests {
    use super::CredentialsStage;
    use crate::set_provider;
    use aws_types::credentials::{
        future, CredentialsError, ProvideCredentials, SharedCredentialsProvider,
    };
    use aws_types::Credentials;
    use smithy_http::body::SdkBody;
    use smithy_http::middleware::AsyncMapRequest;
    use smithy_http::operation;

    #[derive(Debug)]
    struct Unhandled;
    impl ProvideCredentials for Unhandled {
        fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            future::ProvideCredentials::ready(Err(CredentialsError::Unhandled("whoops".into())))
        }
    }

    #[derive(Debug)]
    struct NoCreds;
    impl ProvideCredentials for NoCreds {
        fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            future::ProvideCredentials::ready(Err(CredentialsError::CredentialsNotLoaded))
        }
    }

    #[tokio::test]
    async fn no_cred_provider_is_ok() {
        let req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        CredentialsStage::new()
            .apply(req)
            .await
            .expect("no credential provider should not populate credentials");
    }

    #[tokio::test]
    async fn provider_failure_is_failure() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_provider(
            &mut req.properties_mut(),
            SharedCredentialsProvider::new(Unhandled),
        );
        CredentialsStage::new()
            .apply(req)
            .await
            .expect_err("no credential provider should not populate credentials");
    }

    #[tokio::test]
    async fn credentials_not_loaded_is_ok() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_provider(
            &mut req.properties_mut(),
            SharedCredentialsProvider::new(NoCreds),
        );
        CredentialsStage::new()
            .apply(req)
            .await
            .expect("credentials not loaded is OK");
    }

    #[tokio::test]
    async fn async_map_request_apply_populates_credentials() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_provider(
            &mut req.properties_mut(),
            SharedCredentialsProvider::new(Credentials::from_keys("test", "test", None)),
        );
        let req = CredentialsStage::new()
            .apply(req)
            .await
            .expect("credential provider is in the bag; should succeed");
        assert!(
            req.properties().get::<Credentials>().is_some(),
            "it should set credentials on the request config"
        );
    }
}
