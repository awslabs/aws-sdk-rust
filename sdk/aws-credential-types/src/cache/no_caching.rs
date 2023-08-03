/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Credentials cache that offers no caching ability

use crate::cache::ProvideCachedCredentials;
use crate::provider::SharedCredentialsProvider;
use crate::provider::{future, ProvideCredentials};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct NoCredentialsCache {
    provider: SharedCredentialsProvider,
}

impl NoCredentialsCache {
    pub(crate) fn new(provider: SharedCredentialsProvider) -> Self {
        Self { provider }
    }
}

impl ProvideCachedCredentials for NoCredentialsCache {
    fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'_>
    where
        Self: 'a,
    {
        debug!("Delegating `provide_cached_credentials` to `provide_credentials` on the provider");
        self.provider.provide_credentials()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential_fn::provide_credentials_fn;
    use crate::Credentials;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};

    fn test_provider(load_list: Vec<crate::provider::Result>) -> NoCredentialsCache {
        let load_list = Arc::new(Mutex::new(load_list));
        NoCredentialsCache::new(SharedCredentialsProvider::new(provide_credentials_fn(
            move || {
                let list = load_list.clone();
                async move {
                    let next = list.lock().unwrap().remove(0);
                    next
                }
            },
        )))
    }

    fn epoch_secs(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn credentials(expired_secs: u64) -> Credentials {
        Credentials::new("test", "test", None, Some(epoch_secs(expired_secs)), "test")
    }

    async fn expect_creds(expired_secs: u64, provider: &NoCredentialsCache) {
        let creds = provider
            .provide_cached_credentials()
            .await
            .expect("expected credentials");
        assert_eq!(Some(epoch_secs(expired_secs)), creds.expiry());
    }

    #[tokio::test]
    async fn no_caching() {
        let credentials_cache = test_provider(vec![
            Ok(credentials(1000)),
            Ok(credentials(2000)),
            Ok(credentials(3000)),
        ]);

        expect_creds(1000, &credentials_cache).await;
        expect_creds(2000, &credentials_cache).await;
        expect_creds(3000, &credentials_cache).await;
    }
}
