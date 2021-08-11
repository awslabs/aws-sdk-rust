/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::provider::CredentialsProvider;
use smithy_http::middleware::AsyncMapRequest;
use smithy_http::operation::Request;
use std::future::Future;
use std::pin::Pin;

/// Middleware stage that requests credentials from a [CredentialsProvider] and places them in
/// the property bag of the request.
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
}

mod error {
    use crate::provider::CredentialsError;
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

pub use error::*;

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

impl AsyncMapRequest for CredentialsStage {
    type Error = CredentialsStageError;
    type Future = Pin<Box<dyn Future<Output = Result<Request, Self::Error>> + Send + 'static>>;

    fn apply(&self, mut request: Request) -> BoxFuture<Result<Request, Self::Error>> {
        Box::pin(async move {
            let provider = {
                let properties = request.properties();
                let credential_provider = properties
                    .get::<CredentialsProvider>()
                    .ok_or(CredentialsStageError::MissingCredentialsProvider)?;
                // we need to enable releasing the config lock so that we don't hold the config
                // lock across an await point
                credential_provider.clone()
            };
            let cred_future = { provider.provide_credentials() };
            let credentials = cred_future.await?;
            request.properties_mut().insert(credentials);
            Ok(request)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CredentialsStage;
    use crate::provider::set_provider;
    use crate::Credentials;
    use smithy_http::body::SdkBody;
    use smithy_http::middleware::AsyncMapRequest;
    use smithy_http::operation;
    use std::sync::Arc;

    #[tokio::test]
    async fn async_map_request_apply_requires_credential_provider() {
        let req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        CredentialsStage::new()
            .apply(req)
            .await
            .expect_err("should fail if there's no credential provider in the bag");
    }

    #[tokio::test]
    async fn async_map_request_apply_populates_credentials() {
        let mut req = operation::Request::new(http::Request::new(SdkBody::from("some body")));
        set_provider(
            &mut req.properties_mut(),
            Arc::new(Credentials::from_keys("test", "test", None)),
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
