/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::provider::cache::Cache;
use crate::provider::time::TimeSource;
use crate::provider::{AsyncProvideCredentials, BoxFuture, CredentialsResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{trace_span, Instrument};

const DEFAULT_REFRESH_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_CREDENTIAL_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const DEFAULT_BUFFER_TIME: Duration = Duration::from_secs(10);

// TODO: Implement async runtime-agnostic timeouts
// TODO: Add catch_unwind() to handle panics
// TODO: Update doc comment below once catch_unwind and timeouts are implemented
// TODO: Update warning not to use this in the STS example once it's prod ready

/// `LazyCachingCredentialsProvider` implements [`AsyncProvideCredentials`] by caching
/// credentials that it loads by calling a user-provided [`AsyncProvideCredentials`] implementation.
///
/// For example, you can provide an [`AsyncProvideCredentials`] implementation that calls
/// AWS STS's AssumeRole operation to get temporary credentials, and `LazyCachingCredentialsProvider`
/// will cache those credentials until they expire.
///
/// # Note
///
/// This is __NOT__ production ready yet. Timeouts and panic safety have not been implemented yet.
pub struct LazyCachingCredentialsProvider {
    time: Box<dyn TimeSource>,
    cache: Cache,
    refresh: Arc<dyn AsyncProvideCredentials>,
    _refresh_timeout: Duration,
    default_credential_expiration: Duration,
}

impl LazyCachingCredentialsProvider {
    fn new(
        time: impl TimeSource,
        refresh: Arc<dyn AsyncProvideCredentials>,
        refresh_timeout: Duration,
        default_credential_expiration: Duration,
        buffer_time: Duration,
    ) -> Self {
        LazyCachingCredentialsProvider {
            time: Box::new(time),
            cache: Cache::new(buffer_time),
            refresh,
            _refresh_timeout: refresh_timeout,
            default_credential_expiration,
        }
    }

    /// Returns a new `Builder` that can be used to construct the `LazyCachingCredentialsProvider`.
    pub fn builder() -> builder::Builder {
        builder::Builder::new()
    }
}

impl AsyncProvideCredentials for LazyCachingCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a,
    {
        let now = self.time.now();
        let refresh = self.refresh.clone();
        let cache = self.cache.clone();
        let default_credential_expiration = self.default_credential_expiration;

        Box::pin(async move {
            // Attempt to get cached credentials, or clear the cache if they're expired
            if let Some(credentials) = cache.yield_or_clear_if_expired(now).await {
                Ok(credentials)
            } else {
                // If we didn't get credentials from the cache, then we need to try and refresh.
                // There may be other threads also refreshing simultaneously, but this is OK
                // since the futures are not eagerly executed, and the cache will only run one
                // of them.
                let span = trace_span!("lazy_refresh_credentials");
                let future = refresh.provide_credentials();
                cache
                    .get_or_load(|| {
                        async move {
                            let mut credentials = future.await?;
                            // If the credentials don't have an expiration time, then create a default one
                            if credentials.expiry().is_none() {
                                *credentials.expiry_mut() =
                                    Some(now + default_credential_expiration);
                            }
                            Ok(credentials)
                        }
                        // Only instrument the the actual refreshing future so that no span
                        // is opened if the cache decides not to execute it.
                        .instrument(span)
                    })
                    .await
            }
        })
    }
}

pub mod builder {
    use crate::provider::lazy_caching::{
        LazyCachingCredentialsProvider, DEFAULT_BUFFER_TIME, DEFAULT_CREDENTIAL_EXPIRATION,
        DEFAULT_REFRESH_TIMEOUT,
    };
    use crate::provider::time::SystemTimeSource;
    use crate::provider::AsyncProvideCredentials;
    use std::sync::Arc;
    use std::time::Duration;

    /// Builder for constructing a [`LazyCachingCredentialsProvider`].
    ///
    /// # Example
    ///
    /// ```
    /// use aws_auth::Credentials;
    /// use aws_auth::provider::async_provide_credentials_fn;
    /// use aws_auth::provider::lazy_caching::LazyCachingCredentialsProvider;
    /// use std::sync::Arc;
    /// use std::time::Duration;
    ///
    /// let provider = LazyCachingCredentialsProvider::builder()
    ///     .refresh(async_provide_credentials_fn(|| async {
    ///         // An async process to retrieve credentials would go here:
    ///         Ok(Credentials::from_keys("example", "example", None))
    ///     }))
    ///     .build();
    /// ```
    #[derive(Default)]
    pub struct Builder {
        refresh: Option<Arc<dyn AsyncProvideCredentials>>,
        refresh_timeout: Option<Duration>,
        buffer_time: Option<Duration>,
        default_credential_expiration: Option<Duration>,
    }

    impl Builder {
        pub fn new() -> Self {
            Default::default()
        }

        /// An implementation of [`AsyncProvideCredentials`] that will be used to refresh
        /// the cached credentials once they're expired.
        pub fn refresh(mut self, refresh: impl AsyncProvideCredentials + 'static) -> Self {
            self.refresh = Some(Arc::new(refresh));
            self
        }

        /// (Optional) Timeout for the given [`AsyncProvideCredentials`] implementation.
        /// Defaults to 5 seconds.
        pub fn refresh_timeout(mut self, timeout: Duration) -> Self {
            self.refresh_timeout = Some(timeout);
            unimplemented!("refresh_timeout hasn't been implemented yet")
        }

        /// (Optional) Amount of time before the actual credential expiration time
        /// where credentials are considered expired. For example, if credentials are expiring
        /// in 15 minutes, and the buffer time is 10 seconds, then any requests made after
        /// 14 minutes and 50 seconds will load new credentials. Defaults to 10 seconds.
        pub fn buffer_time(mut self, buffer_time: Duration) -> Self {
            self.buffer_time = Some(buffer_time);
            self
        }

        /// (Optional) Default expiration time to set on credentials if they don't
        /// have an expiration time. This is only used if the given [`AsyncProvideCredentials`]
        /// returns [`Credentials`](crate::Credentials) that don't have their `expiry` set.
        /// This must be at least 15 minutes.
        pub fn default_credential_expiration(mut self, duration: Duration) -> Self {
            self.default_credential_expiration = Some(duration);
            self
        }

        /// Creates the [`LazyCachingCredentialsProvider`].
        pub fn build(self) -> LazyCachingCredentialsProvider {
            let default_credential_expiration = self
                .default_credential_expiration
                .unwrap_or(DEFAULT_CREDENTIAL_EXPIRATION);
            assert!(
                default_credential_expiration >= DEFAULT_CREDENTIAL_EXPIRATION,
                "default_credential_expiration must be at least 15 minutes"
            );
            LazyCachingCredentialsProvider::new(
                SystemTimeSource,
                self.refresh.expect("refresh provider is required"),
                self.refresh_timeout.unwrap_or(DEFAULT_REFRESH_TIMEOUT),
                self.buffer_time.unwrap_or(DEFAULT_BUFFER_TIME),
                default_credential_expiration,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::provider::lazy_caching::{
        LazyCachingCredentialsProvider, TimeSource, DEFAULT_BUFFER_TIME,
        DEFAULT_CREDENTIAL_EXPIRATION, DEFAULT_REFRESH_TIMEOUT,
    };
    use crate::provider::{
        async_provide_credentials_fn, AsyncProvideCredentials, CredentialsError, CredentialsResult,
    };
    use crate::Credentials;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime};
    use tracing::info;

    #[derive(Clone)]
    struct TestTime {
        time: Arc<Mutex<SystemTime>>,
    }

    impl TestTime {
        fn new(time: SystemTime) -> Self {
            TestTime {
                time: Arc::new(Mutex::new(time)),
            }
        }

        fn set(inner: &Arc<Mutex<SystemTime>>, time: SystemTime) {
            *inner.lock().unwrap() = time;
        }
    }

    impl TimeSource for TestTime {
        fn now(&self) -> SystemTime {
            *self.time.lock().unwrap()
        }
    }

    fn test_provider<T: TimeSource>(
        time: T,
        refresh_list: Vec<CredentialsResult>,
    ) -> LazyCachingCredentialsProvider {
        let refresh_list = Arc::new(Mutex::new(refresh_list));
        LazyCachingCredentialsProvider::new(
            time,
            Arc::new(async_provide_credentials_fn(move || {
                let list = refresh_list.clone();
                async move {
                    let next = list.lock().unwrap().remove(0);
                    info!("refreshing the credentials to {:?}", next);
                    next
                }
            })),
            DEFAULT_REFRESH_TIMEOUT,
            DEFAULT_CREDENTIAL_EXPIRATION,
            DEFAULT_BUFFER_TIME,
        )
    }

    fn epoch_secs(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn credentials(expired_secs: u64) -> Credentials {
        Credentials::new("test", "test", None, Some(epoch_secs(expired_secs)), "test")
    }

    async fn expect_creds(expired_secs: u64, provider: &LazyCachingCredentialsProvider) {
        let creds = provider
            .provide_credentials()
            .await
            .expect("expected credentials");
        assert_eq!(Some(epoch_secs(expired_secs)), creds.expiry());
    }

    #[test_env_log::test(tokio::test)]
    async fn initial_populate_credentials() {
        let time = TestTime::new(epoch_secs(100));
        let refresh = Arc::new(async_provide_credentials_fn(|| async {
            info!("refreshing the credentials");
            Ok(credentials(1000))
        }));
        let provider = LazyCachingCredentialsProvider::new(
            time,
            refresh,
            DEFAULT_REFRESH_TIMEOUT,
            DEFAULT_CREDENTIAL_EXPIRATION,
            DEFAULT_BUFFER_TIME,
        );
        assert_eq!(
            epoch_secs(1000),
            provider
                .provide_credentials()
                .await
                .unwrap()
                .expiry()
                .unwrap()
        );
    }

    #[test_env_log::test(tokio::test)]
    async fn refresh_expired_credentials() {
        let time = TestTime::new(epoch_secs(100));
        let time_inner = time.time.clone();
        let provider = test_provider(
            time,
            vec![
                Ok(credentials(1000)),
                Ok(credentials(2000)),
                Ok(credentials(3000)),
            ],
        );

        expect_creds(1000, &provider).await;
        expect_creds(1000, &provider).await;
        TestTime::set(&time_inner, epoch_secs(1500));
        expect_creds(2000, &provider).await;
        expect_creds(2000, &provider).await;
        TestTime::set(&time_inner, epoch_secs(2500));
        expect_creds(3000, &provider).await;
        expect_creds(3000, &provider).await;
    }

    #[test_env_log::test(tokio::test)]
    async fn refresh_failed_error() {
        let time = TestTime::new(epoch_secs(100));
        let time_inner = time.time.clone();
        let provider = test_provider(
            time,
            vec![
                Ok(credentials(1000)),
                Err(CredentialsError::CredentialsNotLoaded),
            ],
        );

        expect_creds(1000, &provider).await;
        TestTime::set(&time_inner, epoch_secs(1500));
        assert!(provider.provide_credentials().await.is_err());
    }

    #[test_env_log::test]
    fn refresh_retrieve_contention() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(16)
            .build()
            .unwrap();

        let time = TestTime::new(epoch_secs(0));
        let time_inner = time.time.clone();
        let provider = Arc::new(test_provider(
            time,
            vec![
                Ok(credentials(500)),
                Ok(credentials(1500)),
                Ok(credentials(2500)),
                Ok(credentials(3500)),
                Ok(credentials(4500)),
            ],
        ));

        for i in 0..4 {
            let mut tasks = Vec::new();
            for j in 0..50 {
                let provider = provider.clone();
                let time_inner = time_inner.clone();
                tasks.push(rt.spawn(async move {
                    let now = epoch_secs(i * 1000 + (4 * j));
                    TestTime::set(&time_inner, now);

                    let creds = provider.provide_credentials().await.unwrap();
                    assert!(
                        creds.expiry().unwrap() >= now,
                        "{:?} >= {:?}",
                        creds.expiry(),
                        now
                    );
                }));
            }
            for task in tasks {
                rt.block_on(task).unwrap();
            }
        }
    }
}
