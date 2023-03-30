/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Lazy, credentials cache implementation

use std::sync::Arc;
use std::time::{Duration, Instant};

use aws_smithy_async::future::timeout::Timeout;
use aws_smithy_async::rt::sleep::AsyncSleep;
use tracing::{debug, info, info_span, Instrument};

use crate::cache::{ExpiringCache, ProvideCachedCredentials};
use crate::provider::SharedCredentialsProvider;
use crate::provider::{error::CredentialsError, future, ProvideCredentials};
use crate::time_source::TimeSource;

const DEFAULT_LOAD_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_CREDENTIAL_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const DEFAULT_BUFFER_TIME: Duration = Duration::from_secs(10);
const DEFAULT_BUFFER_TIME_JITTER_FRACTION: fn() -> f64 = fastrand::f64;

#[derive(Debug)]
pub(crate) struct LazyCredentialsCache {
    time: TimeSource,
    sleeper: Arc<dyn AsyncSleep>,
    cache: ExpiringCache<Credentials, CredentialsError>,
    provider: SharedCredentialsProvider,
    load_timeout: Duration,
    buffer_time: Duration,
    buffer_time_jitter_fraction: fn() -> f64,
    default_credential_expiration: Duration,
}

impl LazyCredentialsCache {
    fn new(
        time: TimeSource,
        sleeper: Arc<dyn AsyncSleep>,
        provider: SharedCredentialsProvider,
        load_timeout: Duration,
        buffer_time: Duration,
        buffer_time_jitter_fraction: fn() -> f64,
        default_credential_expiration: Duration,
    ) -> Self {
        Self {
            time,
            sleeper,
            cache: ExpiringCache::new(buffer_time),
            provider,
            load_timeout,
            buffer_time,
            buffer_time_jitter_fraction,
            default_credential_expiration,
        }
    }
}

impl ProvideCachedCredentials for LazyCredentialsCache {
    fn provide_cached_credentials<'a>(&'a self) -> future::ProvideCredentials<'_>
    where
        Self: 'a,
    {
        let now = self.time.now();
        let provider = self.provider.clone();
        let timeout_future = self.sleeper.sleep(self.load_timeout);
        let load_timeout = self.load_timeout;
        let cache = self.cache.clone();
        let default_credential_expiration = self.default_credential_expiration;

        future::ProvideCredentials::new(async move {
            // Attempt to get cached credentials, or clear the cache if they're expired
            if let Some(credentials) = cache.yield_or_clear_if_expired(now).await {
                debug!("loaded credentials from cache");
                Ok(credentials)
            } else {
                // If we didn't get credentials from the cache, then we need to try and load.
                // There may be other threads also loading simultaneously, but this is OK
                // since the futures are not eagerly executed, and the cache will only run one
                // of them.
                let future = Timeout::new(provider.provide_credentials(), timeout_future);
                let start_time = Instant::now();
                let result = cache
                    .get_or_load(|| {
                        let span = info_span!("lazy_load_credentials");
                        let provider = provider.clone();
                        async move {
                            let credentials = match future.await {
                                Ok(creds) => creds?,
                                Err(_err) => match provider.fallback_on_interrupt() {
                                    Some(creds) => creds,
                                    None => {
                                        return Err(CredentialsError::provider_timed_out(
                                            load_timeout,
                                        ))
                                    }
                                },
                            };
                            // If the credentials don't have an expiration time, then create a default one
                            let expiry = credentials
                                .expiry()
                                .unwrap_or(now + default_credential_expiration);

                            let jitter = self
                                .buffer_time
                                .mul_f64((self.buffer_time_jitter_fraction)());

                            // Logging for cache miss should be emitted here as opposed to after the call to
                            // `cache.get_or_load` above. In the case of multiple threads concurrently executing
                            // `cache.get_or_load`, logging inside `cache.get_or_load` ensures that it is emitted
                            // only once for the first thread that succeeds in populating a cache value.
                            info!(
                                "credentials cache miss occurred; added new AWS credentials (took {:?})",
                                start_time.elapsed()
                            );

                            Ok((credentials, expiry + jitter))
                        }
                        // Only instrument the the actual load future so that no span
                        // is opened if the cache decides not to execute it.
                        .instrument(span)
                    })
                    .await;
                debug!("loaded credentials");
                result
            }
        })
    }
}

use crate::Credentials;
pub use builder::Builder;

mod builder {
    use std::sync::Arc;
    use std::time::Duration;

    use crate::cache::{CredentialsCache, Inner};
    use crate::provider::SharedCredentialsProvider;
    use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};

    use super::TimeSource;
    use super::{
        LazyCredentialsCache, DEFAULT_BUFFER_TIME, DEFAULT_BUFFER_TIME_JITTER_FRACTION,
        DEFAULT_CREDENTIAL_EXPIRATION, DEFAULT_LOAD_TIMEOUT,
    };

    /// Builder for constructing a `LazyCredentialsCache`.
    ///
    /// `LazyCredentialsCache` implements [`ProvideCachedCredentials`](crate::cache::ProvideCachedCredentials) by caching
    /// credentials that it loads by calling a user-provided [`ProvideCredentials`](crate::provider::ProvideCredentials) implementation.
    ///
    /// For example, you can provide a [`ProvideCredentials`](crate::provider::ProvideCredentials) implementation that calls
    /// AWS STS's AssumeRole operation to get temporary credentials, and `LazyCredentialsCache`
    /// will cache those credentials until they expire.
    ///
    /// Callers outside of this crate cannot call `build` directly. They can instead call
    /// `into_credentials_cache` to obtain a [`CredentialsCache`]. Its `create_cache` then calls
    /// `build` to create a `LazyCredentialsCache`.
    #[derive(Clone, Debug, Default)]
    pub struct Builder {
        sleep: Option<Arc<dyn AsyncSleep>>,
        time_source: Option<TimeSource>,
        load_timeout: Option<Duration>,
        buffer_time: Option<Duration>,
        buffer_time_jitter_fraction: Option<fn() -> f64>,
        default_credential_expiration: Option<Duration>,
    }

    impl Builder {
        /// Creates a new builder
        pub fn new() -> Self {
            Default::default()
        }

        /// Implementation of [`AsyncSleep`] to use for timeouts.
        ///
        /// This enables use of the `LazyCredentialsCache` with other async runtimes.
        /// If using Tokio as the async runtime, this should be set to an instance of
        /// [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep).
        pub fn sleep(mut self, sleep: Arc<dyn AsyncSleep>) -> Self {
            self.set_sleep(Some(sleep));
            self
        }

        /// Implementation of [`AsyncSleep`] to use for timeouts.
        ///
        /// This enables use of the `LazyCredentialsCache` with other async runtimes.
        /// If using Tokio as the async runtime, this should be set to an instance of
        /// [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep).
        pub fn set_sleep(&mut self, sleep: Option<Arc<dyn AsyncSleep>>) -> &mut Self {
            self.sleep = sleep;
            self
        }

        #[doc(hidden)] // because they only exist for tests
        pub fn time_source(mut self, time_source: TimeSource) -> Self {
            self.set_time_source(Some(time_source));
            self
        }

        #[doc(hidden)] // because they only exist for tests
        pub fn set_time_source(&mut self, time_source: Option<TimeSource>) -> &mut Self {
            self.time_source = time_source;
            self
        }

        /// Timeout for the given [`ProvideCredentials`](crate::provider::ProvideCredentials) implementation.
        ///
        /// Defaults to 5 seconds.
        pub fn load_timeout(mut self, timeout: Duration) -> Self {
            self.set_load_timeout(Some(timeout));
            self
        }

        /// Timeout for the given [`ProvideCredentials`](crate::provider::ProvideCredentials) implementation.
        ///
        /// Defaults to 5 seconds.
        pub fn set_load_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
            self.load_timeout = timeout;
            self
        }

        /// Amount of time before the actual credential expiration time
        /// where credentials are considered expired.
        ///
        /// For example, if credentials are expiring in 15 minutes, and the buffer time is 10 seconds,
        /// then any requests made after 14 minutes and 50 seconds will load new credentials.
        ///
        /// Defaults to 10 seconds.
        pub fn buffer_time(mut self, buffer_time: Duration) -> Self {
            self.set_buffer_time(Some(buffer_time));
            self
        }

        /// Amount of time before the actual credential expiration time
        /// where credentials are considered expired.
        ///
        /// For example, if credentials are expiring in 15 minutes, and the buffer time is 10 seconds,
        /// then any requests made after 14 minutes and 50 seconds will load new credentials.
        ///
        /// Defaults to 10 seconds.
        pub fn set_buffer_time(&mut self, buffer_time: Option<Duration>) -> &mut Self {
            self.buffer_time = buffer_time;
            self
        }

        /// A random percentage by which buffer time is jittered for randomization.
        ///
        /// For example, if credentials are expiring in 15 minutes, the buffer time is 10 seconds,
        /// and buffer time jitter fraction is 0.2, then buffer time is adjusted to 8 seconds.
        /// Therefore, any requests made after 14 minutes and 52 seconds will load new credentials.
        ///
        /// Defaults to a randomly generated value between 0.0 and 1.0. This setter is for testing only.
        #[cfg(feature = "test-util")]
        pub fn buffer_time_jitter_fraction(
            mut self,
            buffer_time_jitter_fraction: fn() -> f64,
        ) -> Self {
            self.set_buffer_time_jitter_fraction(Some(buffer_time_jitter_fraction));
            self
        }

        /// A random percentage by which buffer time is jittered for randomization.
        ///
        /// For example, if credentials are expiring in 15 minutes, the buffer time is 10 seconds,
        /// and buffer time jitter fraction is 0.2, then buffer time is adjusted to 8 seconds.
        /// Therefore, any requests made after 14 minutes and 52 seconds will load new credentials.
        ///
        /// Defaults to a randomly generated value between 0.0 and 1.0. This setter is for testing only.
        #[cfg(feature = "test-util")]
        pub fn set_buffer_time_jitter_fraction(
            &mut self,
            buffer_time_jitter_fraction: Option<fn() -> f64>,
        ) -> &mut Self {
            self.buffer_time_jitter_fraction = buffer_time_jitter_fraction;
            self
        }

        /// Default expiration time to set on credentials if they don't have an expiration time.
        ///
        /// This is only used if the given [`ProvideCredentials`](crate::provider::ProvideCredentials) returns
        /// [`Credentials`](crate::Credentials) that don't have their `expiry` set.
        /// This must be at least 15 minutes.
        ///
        /// Defaults to 15 minutes.
        pub fn default_credential_expiration(mut self, duration: Duration) -> Self {
            self.set_default_credential_expiration(Some(duration));
            self
        }

        /// Default expiration time to set on credentials if they don't have an expiration time.
        ///
        /// This is only used if the given [`ProvideCredentials`](crate::provider::ProvideCredentials) returns
        /// [`Credentials`](crate::Credentials) that don't have their `expiry` set.
        /// This must be at least 15 minutes.
        ///
        /// Defaults to 15 minutes.
        pub fn set_default_credential_expiration(
            &mut self,
            duration: Option<Duration>,
        ) -> &mut Self {
            self.default_credential_expiration = duration;
            self
        }

        /// Converts [`Builder`] into [`CredentialsCache`].
        pub fn into_credentials_cache(self) -> CredentialsCache {
            CredentialsCache {
                inner: Inner::Lazy(self),
            }
        }

        /// Creates the [`LazyCredentialsCache`] with the passed-in `provider`.
        ///
        /// # Panics
        /// This will panic if no `sleep` implementation is given and if no default crate features
        /// are used. By default, the [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep)
        /// implementation will be set automatically.
        pub(crate) fn build(self, provider: SharedCredentialsProvider) -> LazyCredentialsCache {
            let default_credential_expiration = self
                .default_credential_expiration
                .unwrap_or(DEFAULT_CREDENTIAL_EXPIRATION);
            assert!(
                default_credential_expiration >= DEFAULT_CREDENTIAL_EXPIRATION,
                "default_credential_expiration must be at least 15 minutes"
            );
            LazyCredentialsCache::new(
                self.time_source.unwrap_or_default(),
                self.sleep.unwrap_or_else(|| {
                    default_async_sleep().expect("no default sleep implementation available")
                }),
                provider,
                self.load_timeout.unwrap_or(DEFAULT_LOAD_TIMEOUT),
                self.buffer_time.unwrap_or(DEFAULT_BUFFER_TIME),
                self.buffer_time_jitter_fraction
                    .unwrap_or(DEFAULT_BUFFER_TIME_JITTER_FRACTION),
                default_credential_expiration,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use aws_smithy_async::rt::sleep::TokioSleep;
    use tracing::info;
    use tracing_test::traced_test;

    use crate::provider::SharedCredentialsProvider;
    use crate::{
        cache::ProvideCachedCredentials, credential_fn::provide_credentials_fn,
        provider::error::CredentialsError, time_source::TestingTimeSource, Credentials,
    };

    use super::{
        LazyCredentialsCache, TimeSource, DEFAULT_BUFFER_TIME, DEFAULT_CREDENTIAL_EXPIRATION,
        DEFAULT_LOAD_TIMEOUT,
    };

    const BUFFER_TIME_NO_JITTER: fn() -> f64 = || 0_f64;

    fn test_provider(
        time: TimeSource,
        buffer_time_jitter_fraction: fn() -> f64,
        load_list: Vec<crate::provider::Result>,
    ) -> LazyCredentialsCache {
        let load_list = Arc::new(Mutex::new(load_list));
        LazyCredentialsCache::new(
            time,
            Arc::new(TokioSleep::new()),
            SharedCredentialsProvider::new(provide_credentials_fn(move || {
                let list = load_list.clone();
                async move {
                    let next = list.lock().unwrap().remove(0);
                    info!("refreshing the credentials to {:?}", next);
                    next
                }
            })),
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            buffer_time_jitter_fraction,
            DEFAULT_CREDENTIAL_EXPIRATION,
        )
    }

    fn epoch_secs(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn credentials(expired_secs: u64) -> Credentials {
        Credentials::new("test", "test", None, Some(epoch_secs(expired_secs)), "test")
    }

    async fn expect_creds(expired_secs: u64, provider: &LazyCredentialsCache) {
        let creds = provider
            .provide_cached_credentials()
            .await
            .expect("expected credentials");
        assert_eq!(Some(epoch_secs(expired_secs)), creds.expiry());
    }

    #[traced_test]
    #[tokio::test]
    async fn initial_populate_credentials() {
        let time = TestingTimeSource::new(UNIX_EPOCH);
        let provider = SharedCredentialsProvider::new(provide_credentials_fn(|| async {
            info!("refreshing the credentials");
            Ok(credentials(1000))
        }));
        let credentials_cache = LazyCredentialsCache::new(
            TimeSource::testing(&time),
            Arc::new(TokioSleep::new()),
            provider,
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_CREDENTIAL_EXPIRATION,
        );
        assert_eq!(
            epoch_secs(1000),
            credentials_cache
                .provide_cached_credentials()
                .await
                .unwrap()
                .expiry()
                .unwrap()
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn reload_expired_credentials() {
        let mut time = TestingTimeSource::new(epoch_secs(100));
        let credentials_cache = test_provider(
            TimeSource::testing(&time),
            BUFFER_TIME_NO_JITTER,
            vec![
                Ok(credentials(1000)),
                Ok(credentials(2000)),
                Ok(credentials(3000)),
            ],
        );

        expect_creds(1000, &credentials_cache).await;
        expect_creds(1000, &credentials_cache).await;
        time.set_time(epoch_secs(1500));
        expect_creds(2000, &credentials_cache).await;
        expect_creds(2000, &credentials_cache).await;
        time.set_time(epoch_secs(2500));
        expect_creds(3000, &credentials_cache).await;
        expect_creds(3000, &credentials_cache).await;
    }

    #[traced_test]
    #[tokio::test]
    async fn load_failed_error() {
        let mut time = TestingTimeSource::new(epoch_secs(100));
        let credentials_cache = test_provider(
            TimeSource::testing(&time),
            BUFFER_TIME_NO_JITTER,
            vec![
                Ok(credentials(1000)),
                Err(CredentialsError::not_loaded("failed")),
            ],
        );

        expect_creds(1000, &credentials_cache).await;
        time.set_time(epoch_secs(1500));
        assert!(credentials_cache
            .provide_cached_credentials()
            .await
            .is_err());
    }

    #[traced_test]
    #[test]
    fn load_contention() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .worker_threads(16)
            .build()
            .unwrap();

        let time = TestingTimeSource::new(epoch_secs(0));
        let credentials_cache = Arc::new(test_provider(
            TimeSource::testing(&time),
            BUFFER_TIME_NO_JITTER,
            vec![
                Ok(credentials(500)),
                Ok(credentials(1500)),
                Ok(credentials(2500)),
                Ok(credentials(3500)),
                Ok(credentials(4500)),
            ],
        ));

        let locked_time = Arc::new(Mutex::new(time));

        for i in 0..4 {
            let mut tasks = Vec::new();
            for j in 0..50 {
                let credentials_cache = credentials_cache.clone();
                let time = locked_time.clone();
                tasks.push(rt.spawn(async move {
                    let now = epoch_secs(i * 1000 + (4 * j));
                    time.lock().unwrap().set_time(now);

                    let creds = credentials_cache
                        .provide_cached_credentials()
                        .await
                        .unwrap();
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

    #[tokio::test]
    #[traced_test]
    async fn load_timeout() {
        let time = TestingTimeSource::new(epoch_secs(100));
        let credentials_cache = LazyCredentialsCache::new(
            TimeSource::testing(&time),
            Arc::new(TokioSleep::new()),
            SharedCredentialsProvider::new(provide_credentials_fn(|| async {
                aws_smithy_async::future::never::Never::new().await;
                Ok(credentials(1000))
            })),
            Duration::from_millis(5),
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_CREDENTIAL_EXPIRATION,
        );

        assert!(matches!(
            credentials_cache.provide_cached_credentials().await,
            Err(CredentialsError::ProviderTimedOut { .. })
        ));
    }

    #[tokio::test]
    async fn buffer_time_jitter() {
        let mut time = TestingTimeSource::new(epoch_secs(100));
        let buffer_time_jitter_fraction = || 0.5_f64;
        let credentials_cache = test_provider(
            TimeSource::testing(&time),
            buffer_time_jitter_fraction,
            vec![Ok(credentials(1000)), Ok(credentials(2000))],
        );

        expect_creds(1000, &credentials_cache).await;
        let buffer_time_with_jitter =
            (DEFAULT_BUFFER_TIME.as_secs_f64() * buffer_time_jitter_fraction()) as u64;
        assert_eq!(buffer_time_with_jitter, 5);
        // Advance time to the point where the first credentials are about to expire (but haven't).
        let almost_expired_secs = 1000 - buffer_time_with_jitter - 1;
        time.set_time(epoch_secs(almost_expired_secs));
        // We should still use the first credentials.
        expect_creds(1000, &credentials_cache).await;
        // Now let the first credentials expire.
        let expired_secs = almost_expired_secs + 1;
        time.set_time(epoch_secs(expired_secs));
        // Now that the first credentials have been expired, the second credentials will be retrieved.
        expect_creds(2000, &credentials_cache).await;
    }
}
