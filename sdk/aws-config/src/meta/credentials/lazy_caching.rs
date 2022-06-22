/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Lazy, caching, credentials provider implementation

use std::sync::Arc;
use std::time::Duration;

use aws_smithy_async::future::timeout::Timeout;
use aws_smithy_async::rt::sleep::AsyncSleep;
use tracing::{trace_span, Instrument};

use aws_types::credentials::{future, CredentialsError, ProvideCredentials};
use aws_types::os_shim_internal::TimeSource;

use crate::cache::ExpiringCache;

const DEFAULT_LOAD_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_CREDENTIAL_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const DEFAULT_BUFFER_TIME: Duration = Duration::from_secs(10);

/// `LazyCachingCredentialsProvider` implements [`ProvideCredentials`] by caching
/// credentials that it loads by calling a user-provided [`ProvideCredentials`] implementation.
///
/// For example, you can provide an [`ProvideCredentials`] implementation that calls
/// AWS STS's AssumeRole operation to get temporary credentials, and `LazyCachingCredentialsProvider`
/// will cache those credentials until they expire.
#[derive(Debug)]
pub struct LazyCachingCredentialsProvider {
    time: TimeSource,
    sleeper: Arc<dyn AsyncSleep>,
    cache: ExpiringCache<Credentials, CredentialsError>,
    loader: Arc<dyn ProvideCredentials>,
    load_timeout: Duration,
    default_credential_expiration: Duration,
}

impl LazyCachingCredentialsProvider {
    fn new(
        time: TimeSource,
        sleeper: Arc<dyn AsyncSleep>,
        loader: Arc<dyn ProvideCredentials>,
        load_timeout: Duration,
        default_credential_expiration: Duration,
        buffer_time: Duration,
    ) -> Self {
        LazyCachingCredentialsProvider {
            time,
            sleeper,
            cache: ExpiringCache::new(buffer_time),
            loader,
            load_timeout,
            default_credential_expiration,
        }
    }

    /// Returns a new `Builder` that can be used to construct the `LazyCachingCredentialsProvider`.
    pub fn builder() -> builder::Builder {
        builder::Builder::new()
    }
}

impl ProvideCredentials for LazyCachingCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'_>
    where
        Self: 'a,
    {
        let now = self.time.now();
        let loader = self.loader.clone();
        let timeout_future = self.sleeper.sleep(self.load_timeout);
        let load_timeout = self.load_timeout;
        let cache = self.cache.clone();
        let default_credential_expiration = self.default_credential_expiration;

        future::ProvideCredentials::new(async move {
            // Attempt to get cached credentials, or clear the cache if they're expired
            if let Some(credentials) = cache.yield_or_clear_if_expired(now).await {
                tracing::debug!("loaded credentials from cache");
                Ok(credentials)
            } else {
                // If we didn't get credentials from the cache, then we need to try and load.
                // There may be other threads also loading simultaneously, but this is OK
                // since the futures are not eagerly executed, and the cache will only run one
                // of them.
                let span = trace_span!("lazy_load_credentials");
                let future = Timeout::new(loader.provide_credentials(), timeout_future);
                cache
                    .get_or_load(|| {
                        async move {
                            let credentials = future.await.map_err(|_err| {
                                CredentialsError::provider_timed_out(load_timeout)
                            })??;
                            // If the credentials don't have an expiration time, then create a default one
                            let expiry = credentials
                                .expiry()
                                .unwrap_or(now + default_credential_expiration);
                            Ok((credentials, expiry))
                        }
                        // Only instrument the the actual load future so that no span
                        // is opened if the cache decides not to execute it.
                        .instrument(span)
                    })
                    .await
            }
        })
    }
}

use aws_types::Credentials;
pub use builder::Builder;

mod builder {
    use std::sync::Arc;
    use std::time::Duration;

    use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
    use aws_types::credentials::ProvideCredentials;

    use super::{
        LazyCachingCredentialsProvider, DEFAULT_BUFFER_TIME, DEFAULT_CREDENTIAL_EXPIRATION,
        DEFAULT_LOAD_TIMEOUT,
    };
    use crate::provider_config::ProviderConfig;
    use aws_types::os_shim_internal::TimeSource;

    /// Builder for constructing a [`LazyCachingCredentialsProvider`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use aws_types::Credentials;
    /// use aws_config::meta::credentials::provide_credentials_fn;
    /// use aws_config::meta::credentials::LazyCachingCredentialsProvider;
    ///
    /// let provider = LazyCachingCredentialsProvider::builder()
    ///     .load(provide_credentials_fn(|| async {
    ///         // An async process to retrieve credentials would go here:
    ///         Ok(Credentials::new("example", "example", None, None, "my_provider_name"))
    ///     }))
    ///     .build();
    /// ```
    #[derive(Debug, Default)]
    pub struct Builder {
        sleep: Option<Arc<dyn AsyncSleep>>,
        time_source: Option<TimeSource>,
        load: Option<Arc<dyn ProvideCredentials>>,
        load_timeout: Option<Duration>,
        buffer_time: Option<Duration>,
        default_credential_expiration: Option<Duration>,
    }

    impl Builder {
        /// Creates a new builder
        pub fn new() -> Self {
            Default::default()
        }

        /// Override configuration for the [Builder]
        pub fn configure(mut self, config: &ProviderConfig) -> Self {
            self.sleep = config.sleep();
            self.time_source = Some(config.time_source());
            self
        }

        /// An implementation of [`ProvideCredentials`] that will be used to load
        /// the cached credentials once they're expired.
        pub fn load(mut self, loader: impl ProvideCredentials + 'static) -> Self {
            self.set_load(Some(loader));
            self
        }

        /// An implementation of [`ProvideCredentials`] that will be used to load
        /// the cached credentials once they're expired.
        pub fn set_load(&mut self, loader: Option<impl ProvideCredentials + 'static>) -> &mut Self {
            self.load = loader.map(|l| Arc::new(l) as Arc<dyn ProvideCredentials>);
            self
        }

        /// Implementation of [`AsyncSleep`] to use for timeouts.
        ///
        /// This enables use of the `LazyCachingCredentialsProvider` with other async runtimes.
        /// If using Tokio as the async runtime, this should be set to an instance of
        /// [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep).
        pub fn sleep(mut self, sleep: impl AsyncSleep + 'static) -> Self {
            self.set_sleep(Some(sleep));
            self
        }

        /// Implementation of [`AsyncSleep`] to use for timeouts.
        ///
        /// This enables use of the `LazyCachingCredentialsProvider` with other async runtimes.
        /// If using Tokio as the async runtime, this should be set to an instance of
        /// [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep).
        pub fn set_sleep(&mut self, sleep: Option<impl AsyncSleep + 'static>) -> &mut Self {
            self.sleep = sleep.map(|s| Arc::new(s) as Arc<dyn AsyncSleep>);
            self
        }

        /// Timeout for the given [`ProvideCredentials`] implementation.
        ///
        /// Defaults to 5 seconds.
        pub fn load_timeout(mut self, timeout: Duration) -> Self {
            self.set_load_timeout(Some(timeout));
            self
        }

        /// Timeout for the given [`ProvideCredentials`] implementation.
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

        /// Default expiration time to set on credentials if they don't have an expiration time.
        ///
        /// This is only used if the given [`ProvideCredentials`] returns
        /// [`Credentials`](aws_types::Credentials) that don't have their `expiry` set.
        /// This must be at least 15 minutes.
        ///
        /// Defaults to 15 minutes.
        pub fn default_credential_expiration(mut self, duration: Duration) -> Self {
            self.set_default_credential_expiration(Some(duration));
            self
        }

        /// Default expiration time to set on credentials if they don't have an expiration time.
        ///
        /// This is only used if the given [`ProvideCredentials`] returns
        /// [`Credentials`](aws_types::Credentials) that don't have their `expiry` set.
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

        /// Creates the [`LazyCachingCredentialsProvider`].
        ///
        /// # Panics
        /// This will panic if no `sleep` implementation is given and if no default crate features
        /// are used. By default, the [`TokioSleep`](aws_smithy_async::rt::sleep::TokioSleep)
        /// implementation will be set automatically.
        pub fn build(self) -> LazyCachingCredentialsProvider {
            let default_credential_expiration = self
                .default_credential_expiration
                .unwrap_or(DEFAULT_CREDENTIAL_EXPIRATION);
            assert!(
                default_credential_expiration >= DEFAULT_CREDENTIAL_EXPIRATION,
                "default_credential_expiration must be at least 15 minutes"
            );
            LazyCachingCredentialsProvider::new(
                self.time_source.unwrap_or_default(),
                self.sleep.unwrap_or_else(|| {
                    default_async_sleep().expect("no default sleep implementation available")
                }),
                self.load.expect("load implementation is required"),
                self.load_timeout.unwrap_or(DEFAULT_LOAD_TIMEOUT),
                default_credential_expiration,
                self.buffer_time.unwrap_or(DEFAULT_BUFFER_TIME),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_types::credentials::{self, CredentialsError, ProvideCredentials};
    use aws_types::Credentials;
    use tracing::info;
    use tracing_test::traced_test;

    use crate::meta::credentials::credential_fn::provide_credentials_fn;

    use super::{
        LazyCachingCredentialsProvider, TimeSource, DEFAULT_BUFFER_TIME,
        DEFAULT_CREDENTIAL_EXPIRATION, DEFAULT_LOAD_TIMEOUT,
    };
    use aws_types::os_shim_internal::ManualTimeSource;

    fn test_provider(
        time: TimeSource,
        load_list: Vec<credentials::Result>,
    ) -> LazyCachingCredentialsProvider {
        let load_list = Arc::new(Mutex::new(load_list));
        LazyCachingCredentialsProvider::new(
            time,
            Arc::new(TokioSleep::new()),
            Arc::new(provide_credentials_fn(move || {
                let list = load_list.clone();
                async move {
                    let next = list.lock().unwrap().remove(0);
                    info!("refreshing the credentials to {:?}", next);
                    next
                }
            })),
            DEFAULT_LOAD_TIMEOUT,
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

    #[traced_test]
    #[tokio::test]
    async fn initial_populate_credentials() {
        let time = ManualTimeSource::new(UNIX_EPOCH);
        let loader = Arc::new(provide_credentials_fn(|| async {
            info!("refreshing the credentials");
            Ok(credentials(1000))
        }));
        let provider = LazyCachingCredentialsProvider::new(
            TimeSource::manual(&time),
            Arc::new(TokioSleep::new()),
            loader,
            DEFAULT_LOAD_TIMEOUT,
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

    #[traced_test]
    #[tokio::test]
    async fn reload_expired_credentials() {
        let mut time = ManualTimeSource::new(epoch_secs(100));
        let provider = test_provider(
            TimeSource::manual(&time),
            vec![
                Ok(credentials(1000)),
                Ok(credentials(2000)),
                Ok(credentials(3000)),
            ],
        );

        expect_creds(1000, &provider).await;
        expect_creds(1000, &provider).await;
        time.set_time(epoch_secs(1500));
        expect_creds(2000, &provider).await;
        expect_creds(2000, &provider).await;
        time.set_time(epoch_secs(2500));
        expect_creds(3000, &provider).await;
        expect_creds(3000, &provider).await;
    }

    #[traced_test]
    #[tokio::test]
    async fn load_failed_error() {
        let mut time = ManualTimeSource::new(epoch_secs(100));
        let provider = test_provider(
            TimeSource::manual(&time),
            vec![
                Ok(credentials(1000)),
                Err(CredentialsError::not_loaded("failed")),
            ],
        );

        expect_creds(1000, &provider).await;
        time.set_time(epoch_secs(1500));
        assert!(provider.provide_credentials().await.is_err());
    }

    #[traced_test]
    #[test]
    fn load_contention() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .worker_threads(16)
            .build()
            .unwrap();

        let time = ManualTimeSource::new(epoch_secs(0));
        let provider = Arc::new(test_provider(
            TimeSource::manual(&time),
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
                let provider = provider.clone();
                let time = locked_time.clone();
                tasks.push(rt.spawn(async move {
                    let now = epoch_secs(i * 1000 + (4 * j));
                    time.lock().unwrap().set_time(now);

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

    #[tokio::test]
    #[traced_test]
    async fn load_timeout() {
        let time = ManualTimeSource::new(epoch_secs(100));
        let provider = LazyCachingCredentialsProvider::new(
            TimeSource::manual(&time),
            Arc::new(TokioSleep::new()),
            Arc::new(provide_credentials_fn(|| async {
                aws_smithy_async::future::never::Never::new().await;
                Ok(credentials(1000))
            })),
            Duration::from_millis(5),
            DEFAULT_CREDENTIAL_EXPIRATION,
            DEFAULT_BUFFER_TIME,
        );

        assert!(matches!(
            provider.provide_credentials().await,
            Err(CredentialsError::ProviderTimedOut { .. })
        ));
    }
}
