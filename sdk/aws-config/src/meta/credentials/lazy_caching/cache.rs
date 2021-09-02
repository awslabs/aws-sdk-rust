/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::credentials::CredentialsError;
use aws_types::{credentials, Credentials};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{OnceCell, RwLock};

#[derive(Clone, Debug)]
pub(super) struct Cache {
    /// Amount of time before the actual credential expiration time
    /// where credentials are considered expired.
    buffer_time: Duration,
    value: Arc<RwLock<OnceCell<(Credentials, SystemTime)>>>,
}

impl Cache {
    pub fn new(buffer_time: Duration) -> Cache {
        Cache {
            buffer_time,
            value: Arc::new(RwLock::new(OnceCell::new())),
        }
    }

    #[cfg(test)]
    async fn get(&self) -> Option<Credentials> {
        self.value
            .read()
            .await
            .get()
            .cloned()
            .map(|(creds, _expiry)| creds)
    }

    /// Attempts to refresh the cached credentials with the given async future.
    /// If multiple threads attempt to refresh at the same time, one of them will win,
    /// and the others will await that thread's result rather than multiple refreshes occurring.
    /// The function given to acquire a credentials future, `f`, will not be called
    /// if another thread is chosen to load the credentials.
    pub async fn get_or_load<F, Fut>(&self, f: F) -> credentials::Result
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<(Credentials, SystemTime), CredentialsError>>,
    {
        let lock = self.value.read().await;
        let future = lock.get_or_try_init(f);
        future
            .await
            .map(|(credentials, _expiry)| credentials.clone())
    }

    /// If the credentials are expired, clears the cache. Otherwise, yields the current credentials value.
    pub async fn yield_or_clear_if_expired(&self, now: SystemTime) -> Option<Credentials> {
        // Short-circuit if the credential is not expired
        if let Some((credentials, expiry)) = self.value.read().await.get() {
            if !expired(*expiry, self.buffer_time, now) {
                return Some(credentials.clone());
            }
        }

        // Acquire a write lock to clear the cache, but then once the lock is acquired,
        // check again that the credential is not already cleared. If it has been cleared,
        // then another thread is refreshing the cache by the time the write lock was acquired.
        let mut lock = self.value.write().await;
        if let Some((_credentials, expiration)) = lock.get() {
            // Also check that we're clearing the expired credentials and not credentials
            // that have been refreshed by another thread.
            if expired(*expiration, self.buffer_time, now) {
                *lock = OnceCell::new();
            }
        }
        None
    }
}

fn expired(expiration: SystemTime, buffer_time: Duration, now: SystemTime) -> bool {
    now >= (expiration - buffer_time)
}

#[cfg(test)]
mod tests {
    use super::{expired, Cache};
    use aws_types::credentials::CredentialsError;
    use aws_types::Credentials;
    use std::time::{Duration, SystemTime};
    use tracing_test::traced_test;

    fn credentials(expired_secs: u64) -> Result<(Credentials, SystemTime), CredentialsError> {
        let expiry = epoch_secs(expired_secs);
        let creds = Credentials::new("test", "test", None, Some(expiry), "test");
        Ok((creds, expiry))
    }

    fn epoch_secs(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    #[test]
    fn expired_check() {
        let ts = epoch_secs(100);
        assert!(expired(ts, Duration::from_secs(10), epoch_secs(1000)));
        assert!(expired(ts, Duration::from_secs(10), epoch_secs(90)));
        assert!(!expired(ts, Duration::from_secs(10), epoch_secs(10)));
    }

    #[traced_test]
    #[tokio::test]
    async fn cache_clears_if_expired_only() {
        let cache = Cache::new(Duration::from_secs(10));
        assert!(cache
            .yield_or_clear_if_expired(epoch_secs(100))
            .await
            .is_none());

        cache
            .get_or_load(|| async { credentials(100) })
            .await
            .unwrap();
        assert_eq!(Some(epoch_secs(100)), cache.get().await.unwrap().expiry());

        // It should not clear the credentials if they're not expired
        assert_eq!(
            Some(epoch_secs(100)),
            cache
                .yield_or_clear_if_expired(epoch_secs(10))
                .await
                .unwrap()
                .expiry()
        );

        // It should clear the credentials if they're expired
        assert!(cache
            .yield_or_clear_if_expired(epoch_secs(500))
            .await
            .is_none());
        assert!(cache.get().await.is_none());
    }
}
