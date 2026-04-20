/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::expiring_cache::ExpiringCache;
use aws_smithy_async::future::timeout::Timeout;
use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep};
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::identity::{
    Identity, IdentityCachePartition, IdentityFuture, ResolveCachedIdentity, ResolveIdentity,
    SharedIdentityCache, SharedIdentityResolver,
};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::DateTime;
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;
use std::time::Duration;
use tracing::Instrument;

const DEFAULT_LOAD_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const DEFAULT_BUFFER_TIME: Duration = Duration::from_secs(10);
const DEFAULT_BUFFER_TIME_JITTER_FRACTION: fn() -> f64 = || fastrand::f64() * 0.5;
const DEFAULT_MAX_PARTITIONS: usize = 64;

/// Builder for lazy identity caching.
#[derive(Default, Debug)]
pub struct LazyCacheBuilder {
    time_source: Option<SharedTimeSource>,
    sleep_impl: Option<SharedAsyncSleep>,
    load_timeout: Option<Duration>,
    buffer_time: Option<Duration>,
    buffer_time_jitter_fraction: Option<fn() -> f64>,
    default_expiration: Option<Duration>,
    max_partitions: Option<usize>,
}

impl LazyCacheBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Set the time source for this cache.
    pub fn time_source(mut self, time_source: impl TimeSource + 'static) -> Self {
        self.set_time_source(time_source.into_shared());
        self
    }
    /// Set the time source for this cache.
    pub fn set_time_source(&mut self, time_source: SharedTimeSource) -> &mut Self {
        self.time_source = Some(time_source.into_shared());
        self
    }

    /// Set the async sleep implementation for this cache.
    pub fn sleep_impl(mut self, sleep_impl: impl AsyncSleep + 'static) -> Self {
        self.set_sleep_impl(sleep_impl.into_shared());
        self
    }
    /// Set the async sleep implementation for this cache.
    pub fn set_sleep_impl(&mut self, sleep_impl: SharedAsyncSleep) -> &mut Self {
        self.sleep_impl = Some(sleep_impl);
        self
    }

    /// Timeout for identity resolution.
    ///
    /// Defaults to 5 seconds.
    pub fn load_timeout(mut self, timeout: Duration) -> Self {
        self.set_load_timeout(Some(timeout));
        self
    }

    /// Timeout for identity resolution.
    ///
    /// Defaults to 5 seconds.
    pub fn set_load_timeout(&mut self, timeout: Option<Duration>) -> &mut Self {
        self.load_timeout = timeout;
        self
    }

    /// Amount of time before the actual identity expiration time where the identity is considered expired.
    ///
    /// For example, if the identity are expiring in 15 minutes, and the buffer time is 10 seconds,
    /// then any requests made after 14 minutes and 50 seconds will load a new identity.
    ///
    /// Note: random jitter value between [0.0, 0.5] is multiplied to this buffer time.
    ///
    /// Defaults to 10 seconds.
    pub fn buffer_time(mut self, buffer_time: Duration) -> Self {
        self.set_buffer_time(Some(buffer_time));
        self
    }

    /// Amount of time before the actual identity expiration time where the identity is considered expired.
    ///
    /// For example, if the identity are expiring in 15 minutes, and the buffer time is 10 seconds,
    /// then any requests made after 14 minutes and 50 seconds will load a new identity.
    ///
    /// Note: random jitter value between [0.0, 0.5] is multiplied to this buffer time.
    ///
    /// Defaults to 10 seconds.
    pub fn set_buffer_time(&mut self, buffer_time: Option<Duration>) -> &mut Self {
        self.buffer_time = buffer_time;
        self
    }

    /// A random percentage by which buffer time is jittered for randomization.
    ///
    /// For example, if the identity is expiring in 15 minutes, the buffer time is 10 seconds,
    /// and buffer time jitter fraction is 0.2, then buffer time is adjusted to 8 seconds.
    /// Therefore, any requests made after 14 minutes and 52 seconds will load a new identity.
    ///
    /// Defaults to a randomly generated value between 0.0 and 0.5. This setter is for testing only.
    #[allow(unused)]
    #[cfg(test)]
    fn buffer_time_jitter_fraction(mut self, buffer_time_jitter_fraction: fn() -> f64) -> Self {
        self.set_buffer_time_jitter_fraction(Some(buffer_time_jitter_fraction));
        self
    }

    /// A random percentage by which buffer time is jittered for randomization.
    ///
    /// For example, if the identity is expiring in 15 minutes, the buffer time is 10 seconds,
    /// and buffer time jitter fraction is 0.2, then buffer time is adjusted to 8 seconds.
    /// Therefore, any requests made after 14 minutes and 52 seconds will load a new identity.
    ///
    /// Defaults to a randomly generated value between 0.0 and 0.5. This setter is for testing only.
    #[allow(unused)]
    #[cfg(test)]
    fn set_buffer_time_jitter_fraction(
        &mut self,
        buffer_time_jitter_fraction: Option<fn() -> f64>,
    ) -> &mut Self {
        self.buffer_time_jitter_fraction = buffer_time_jitter_fraction;
        self
    }

    /// Default expiration time to set on an identity if it doesn't have an expiration time.
    ///
    /// This is only used if the resolved identity doesn't have an expiration time set.
    /// This must be at least 15 minutes.
    ///
    /// Defaults to 15 minutes.
    pub fn default_expiration(mut self, duration: Duration) -> Self {
        self.set_default_expiration(Some(duration));
        self
    }

    /// Default expiration time to set on an identity if it doesn't have an expiration time.
    ///
    /// This is only used if the resolved identity doesn't have an expiration time set.
    /// This must be at least 15 minutes.
    ///
    /// Defaults to 15 minutes.
    pub fn set_default_expiration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.default_expiration = duration;
        self
    }

    /// Maximum number of identity cache partitions before eviction occurs.
    ///
    /// A normally functioning application should not have more than 5-10
    /// credential providers active at any given time. This limit acts as
    /// a safety net against memory leaks.
    ///
    /// Defaults to 64.
    ///
    /// # Panics
    ///
    /// Panics if `max` is 0.
    pub fn max_partitions(mut self, max: usize) -> Self {
        self.set_max_partitions(Some(max));
        self
    }

    /// Maximum number of identity cache partitions before eviction occurs.
    ///
    /// A normally functioning application should not have more than 5-10
    /// credential providers active at any given time. This limit acts as
    /// a safety net against memory leaks.
    ///
    /// Defaults to 64.
    ///
    /// # Panics
    ///
    /// Panics if `max` is `Some(0)`.
    pub fn set_max_partitions(&mut self, max: Option<usize>) -> &mut Self {
        if let Some(0) = max {
            panic!("max_partitions must be greater than 0");
        }
        self.max_partitions = max;
        self
    }

    /// Builds a [`SharedIdentityCache`] from this builder.
    ///
    /// # Panics
    ///
    /// This builder will panic if required fields are not given, or if given values are not valid.
    pub fn build(self) -> SharedIdentityCache {
        let default_expiration = self.default_expiration.unwrap_or(DEFAULT_EXPIRATION);
        assert!(
            default_expiration >= DEFAULT_EXPIRATION,
            "default_expiration must be at least 15 minutes"
        );
        LazyCache::new(
            self.load_timeout.unwrap_or(DEFAULT_LOAD_TIMEOUT),
            self.buffer_time.unwrap_or(DEFAULT_BUFFER_TIME),
            self.buffer_time_jitter_fraction
                .unwrap_or(DEFAULT_BUFFER_TIME_JITTER_FRACTION),
            default_expiration,
            self.max_partitions.unwrap_or(DEFAULT_MAX_PARTITIONS),
        )
        .into_shared()
    }
}

#[derive(Debug)]
struct CachePartitions {
    partitions: RwLock<HashMap<IdentityCachePartition, ExpiringCache<Identity, BoxError>>>,
    buffer_time: Duration,
    max_partitions: usize,
}

impl CachePartitions {
    fn new(buffer_time: Duration, max_partitions: usize) -> Self {
        Self {
            partitions: RwLock::new(HashMap::new()),
            buffer_time,
            max_partitions,
        }
    }

    fn partition(&self, key: IdentityCachePartition) -> ExpiringCache<Identity, BoxError> {
        // Fast path: read lock for cache hits
        if let Some(partition) = self.partitions.read().unwrap().get(&key).cloned() {
            return partition;
        }
        // Slow path: write lock for cache misses
        let mut partitions = self.partitions.write().unwrap();
        // Another thread may have inserted while we waited for the write lock
        if let Some(partition) = partitions.get(&key).cloned() {
            return partition;
        }
        // Evict an arbitrary entry if at capacity. Eviction order doesn't matter
        // because a normally functioning application should not have more than
        // 5-10 credential providers active at any given time, well under the cap.
        if partitions.len() >= self.max_partitions {
            if let Some(&evict_key) = partitions.keys().next() {
                partitions.remove(&evict_key);
            }
        }
        let partition = ExpiringCache::new(self.buffer_time);
        partitions.insert(key, partition.clone());
        tracing::debug!(
            partition_count = partitions.len(),
            "identity cache partition created"
        );
        partition
    }
}

#[derive(Debug)]
struct LazyCache {
    partitions: CachePartitions,
    load_timeout: Duration,
    buffer_time: Duration,
    buffer_time_jitter_fraction: fn() -> f64,
    default_expiration: Duration,
}

impl LazyCache {
    fn new(
        load_timeout: Duration,
        buffer_time: Duration,
        buffer_time_jitter_fraction: fn() -> f64,
        default_expiration: Duration,
        max_partitions: usize,
    ) -> Self {
        Self {
            partitions: CachePartitions::new(buffer_time, max_partitions),
            load_timeout,
            buffer_time,
            buffer_time_jitter_fraction,
            default_expiration,
        }
    }
}

macro_rules! required_err {
    ($thing:literal, $how:literal) => {
        BoxError::from(concat!(
            "Lazy identity caching requires ",
            $thing,
            " to be configured. ",
            $how,
            " If this isn't possible, then disable identity caching by calling ",
            "the `identity_cache` method on config with `IdentityCache::no_cache()`",
        ))
    };
}
macro_rules! validate_components {
    ($components:ident) => {
        let _ = $components.time_source().ok_or_else(|| {
            required_err!(
                "a time source",
                "Set a time source using the `time_source` method on config."
            )
        })?;
        let _ = $components.sleep_impl().ok_or_else(|| {
            required_err!(
                "an async sleep implementation",
                "Set a sleep impl using the `sleep_impl` method on config."
            )
        })?;
    };
}

impl ResolveCachedIdentity for LazyCache {
    fn validate_base_client_config(
        &self,
        runtime_components: &aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder,
        _cfg: &ConfigBag,
    ) -> Result<(), BoxError> {
        validate_components!(runtime_components);
        Ok(())
    }

    fn validate_final_config(
        &self,
        runtime_components: &RuntimeComponents,
        _cfg: &ConfigBag,
    ) -> Result<(), BoxError> {
        validate_components!(runtime_components);
        Ok(())
    }

    fn resolve_cached_identity<'a>(
        &'a self,
        resolver: SharedIdentityResolver,
        runtime_components: &'a RuntimeComponents,
        config_bag: &'a ConfigBag,
    ) -> IdentityFuture<'a> {
        let (time_source, sleep_impl) = (
            runtime_components.time_source().expect("validated"),
            runtime_components.sleep_impl().expect("validated"),
        );

        let now = time_source.now();
        let timeout_future = sleep_impl.sleep(self.load_timeout);
        let load_timeout = self.load_timeout;
        let partition = resolver.cache_partition();
        let cache = self.partitions.partition(partition);
        let default_expiration = self.default_expiration;

        IdentityFuture::new(async move {
            // Attempt to get cached identity, or clear the cache if they're expired
            if let Some(identity) = cache.yield_or_clear_if_expired(now).await {
                tracing::debug!(
                    buffer_time=?self.buffer_time,
                    cached_expiration=?identity.expiration(),
                    now=?now,
                    "loaded identity from cache"
                );
                Ok(identity)
            } else {
                // If we didn't get identity from the cache, then we need to try and load.
                // There may be other threads also loading simultaneously, but this is OK
                // since the futures are not eagerly executed, and the cache will only run one
                // of them.
                let start_time = time_source.now();
                let result = cache
                    .get_or_load(|| {
                        let span = tracing::debug_span!("lazy_load_identity");
                        async move {
                            let fut = Timeout::new(
                                resolver.resolve_identity(runtime_components, config_bag),
                                timeout_future,
                            );
                            let identity = match fut.await {
                                Ok(result) => result?,
                                Err(_err) => match resolver.fallback_on_interrupt() {
                                    Some(identity) => identity,
                                    None => {
                                        return Err(BoxError::from(TimedOutError(load_timeout)))
                                    }
                                },
                            };
                            // If the identity don't have an expiration time, then create a default one
                            let expiration =
                                identity.expiration().unwrap_or(now + default_expiration);

                            let jitter = self
                                .buffer_time
                                .mul_f64((self.buffer_time_jitter_fraction)());

                            // Logging for cache miss should be emitted here as opposed to after the call to
                            // `cache.get_or_load` above. In the case of multiple threads concurrently executing
                            // `cache.get_or_load`, logging inside `cache.get_or_load` ensures that it is emitted
                            // only once for the first thread that succeeds in populating a cache value.
                            let printable = DateTime::from(expiration);
                            tracing::debug!(
                                new_expiration=%printable,
                                valid_for=?expiration.duration_since(time_source.now()).unwrap_or_default(),
                                partition=?partition,
                                "identity cache miss occurred; added new identity (took {:?})",
                                time_source.now().duration_since(start_time).unwrap_or_default()
                            );

                            Ok((identity, expiration + jitter))
                        }
                        // Only instrument the the actual load future so that no span
                        // is opened if the cache decides not to execute it.
                        .instrument(span)
                    })
                    .await;
                tracing::debug!("loaded identity");
                result
            }
        })
    }
}

#[derive(Debug)]
struct TimedOutError(Duration);

impl std::error::Error for TimedOutError {}

impl fmt::Display for TimedOutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "identity resolver timed out after {:?}", self.0)
    }
}

#[cfg(all(test, feature = "client", feature = "http-auth"))]
mod tests {
    use super::*;
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_async::test_util::{instant_time_and_sleep, ManualTimeSource};
    use aws_smithy_async::time::TimeSource;
    use aws_smithy_runtime_api::client::identity::http::Token;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tracing::info;

    const BUFFER_TIME_NO_JITTER: fn() -> f64 = || 0_f64;

    struct ResolverFn<F>(F);
    impl<F> fmt::Debug for ResolverFn<F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("ResolverFn")
        }
    }
    impl<F> ResolveIdentity for ResolverFn<F>
    where
        F: Fn() -> IdentityFuture<'static> + Send + Sync,
    {
        fn resolve_identity<'a>(
            &'a self,
            _: &'a RuntimeComponents,
            _config_bag: &'a ConfigBag,
        ) -> IdentityFuture<'a> {
            (self.0)()
        }
    }

    fn resolver_fn<F>(f: F) -> SharedIdentityResolver
    where
        F: Fn() -> IdentityFuture<'static> + Send + Sync + 'static,
    {
        SharedIdentityResolver::new(ResolverFn(f))
    }

    fn test_cache(
        buffer_time_jitter_fraction: fn() -> f64,
        load_list: Vec<Result<Identity, BoxError>>,
    ) -> (LazyCache, SharedIdentityResolver) {
        #[derive(Debug)]
        struct Resolver(Mutex<Vec<Result<Identity, BoxError>>>);
        impl ResolveIdentity for Resolver {
            fn resolve_identity<'a>(
                &'a self,
                _: &'a RuntimeComponents,
                _config_bag: &'a ConfigBag,
            ) -> IdentityFuture<'a> {
                let mut list = self.0.lock().unwrap();
                if list.len() > 0 {
                    let next = list.remove(0);
                    info!("refreshing the identity to {:?}", next);
                    IdentityFuture::ready(next)
                } else {
                    drop(list);
                    panic!("no more identities")
                }
            }
        }

        let identity_resolver = SharedIdentityResolver::new(Resolver(Mutex::new(load_list)));
        let cache = LazyCache::new(
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            buffer_time_jitter_fraction,
            DEFAULT_EXPIRATION,
            DEFAULT_MAX_PARTITIONS,
        );
        (cache, identity_resolver)
    }

    fn epoch_secs(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    fn test_identity(expired_secs: u64) -> Identity {
        let expiration = Some(epoch_secs(expired_secs));
        Identity::new(Token::new("test", expiration), expiration)
    }

    async fn expect_identity(
        expired_secs: u64,
        cache: &LazyCache,
        components: &RuntimeComponents,
        resolver: SharedIdentityResolver,
    ) {
        let config_bag = ConfigBag::base();
        let identity = cache
            .resolve_cached_identity(resolver, components, &config_bag)
            .await
            .expect("expected identity");
        assert_eq!(Some(epoch_secs(expired_secs)), identity.expiration());
    }

    #[tokio::test]
    async fn initial_populate_test_identity() {
        let time = ManualTimeSource::new(UNIX_EPOCH);
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let config_bag = ConfigBag::base();
        let resolver = SharedIdentityResolver::new(resolver_fn(|| {
            info!("refreshing the test_identity");
            IdentityFuture::ready(Ok(test_identity(1000)))
        }));
        let cache = LazyCache::new(
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_EXPIRATION,
            DEFAULT_MAX_PARTITIONS,
        );
        assert_eq!(
            epoch_secs(1000),
            cache
                .resolve_cached_identity(resolver, &components, &config_bag)
                .await
                .unwrap()
                .expiration()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn reload_expired_test_identity() {
        let time = ManualTimeSource::new(epoch_secs(100));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let (cache, resolver) = test_cache(
            BUFFER_TIME_NO_JITTER,
            vec![
                Ok(test_identity(1000)),
                Ok(test_identity(2000)),
                Ok(test_identity(3000)),
            ],
        );

        expect_identity(1000, &cache, &components, resolver.clone()).await;
        expect_identity(1000, &cache, &components, resolver.clone()).await;
        time.set_time(epoch_secs(1500));
        expect_identity(2000, &cache, &components, resolver.clone()).await;
        expect_identity(2000, &cache, &components, resolver.clone()).await;
        time.set_time(epoch_secs(2500));
        expect_identity(3000, &cache, &components, resolver.clone()).await;
        expect_identity(3000, &cache, &components, resolver.clone()).await;
    }

    #[tokio::test]
    async fn load_failed_error() {
        let config_bag = ConfigBag::base();
        let time = ManualTimeSource::new(epoch_secs(100));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let (cache, resolver) = test_cache(
            BUFFER_TIME_NO_JITTER,
            vec![Ok(test_identity(1000)), Err("failed".into())],
        );

        expect_identity(1000, &cache, &components, resolver.clone()).await;
        time.set_time(epoch_secs(1500));
        assert!(cache
            .resolve_cached_identity(resolver.clone(), &components, &config_bag)
            .await
            .is_err());
    }

    #[test]
    fn load_contention() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .worker_threads(16)
            .build()
            .unwrap();

        let time = ManualTimeSource::new(epoch_secs(0));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let (cache, resolver) = test_cache(
            BUFFER_TIME_NO_JITTER,
            vec![
                Ok(test_identity(500)),
                Ok(test_identity(1500)),
                Ok(test_identity(2500)),
                Ok(test_identity(3500)),
                Ok(test_identity(4500)),
            ],
        );
        let cache: SharedIdentityCache = cache.into_shared();

        // test_identity are available up until 4500 seconds after the unix epoch
        // 4*50 = 200 tasks are launched => we can advance time 4500/20 => 225 seconds per advance
        for _ in 0..4 {
            let mut tasks = Vec::new();
            for _ in 0..50 {
                let resolver = resolver.clone();
                let cache = cache.clone();
                let time = time.clone();
                let components = components.clone();
                tasks.push(rt.spawn(async move {
                    let now = time.advance(Duration::from_secs(22));

                    let config_bag = ConfigBag::base();
                    let identity = cache
                        .resolve_cached_identity(resolver, &components, &config_bag)
                        .await
                        .unwrap();
                    assert!(
                        identity.expiration().unwrap() >= now,
                        "{:?} >= {:?}",
                        identity.expiration(),
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
    async fn load_timeout() {
        let config_bag = ConfigBag::base();
        let (time, sleep) = instant_time_and_sleep(epoch_secs(100));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(sleep))
            .build()
            .unwrap();
        let resolver = SharedIdentityResolver::new(resolver_fn(|| {
            IdentityFuture::new(async {
                aws_smithy_async::future::never::Never::new().await;
                Ok(test_identity(1000))
            })
        }));
        let cache = LazyCache::new(
            Duration::from_secs(5),
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_EXPIRATION,
            DEFAULT_MAX_PARTITIONS,
        );

        let err: BoxError = cache
            .resolve_cached_identity(resolver, &components, &config_bag)
            .await
            .expect_err("it should return an error");
        let downcasted = err.downcast_ref::<TimedOutError>();
        assert!(
            downcasted.is_some(),
            "expected a BoxError of TimedOutError, but was {err:?}"
        );
        assert_eq!(time.now(), epoch_secs(105));
    }

    #[tokio::test]
    async fn buffer_time_jitter() {
        let time = ManualTimeSource::new(epoch_secs(100));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let buffer_time_jitter_fraction = || 0.5_f64;
        let (cache, resolver) = test_cache(
            buffer_time_jitter_fraction,
            vec![Ok(test_identity(1000)), Ok(test_identity(2000))],
        );

        expect_identity(1000, &cache, &components, resolver.clone()).await;
        let buffer_time_with_jitter =
            (DEFAULT_BUFFER_TIME.as_secs_f64() * buffer_time_jitter_fraction()) as u64;
        assert_eq!(buffer_time_with_jitter, 5);
        // Advance time to the point where the first test_identity are about to expire (but haven't).
        let almost_expired_secs = 1000 - buffer_time_with_jitter - 1;
        time.set_time(epoch_secs(almost_expired_secs));
        // We should still use the first test_identity.
        expect_identity(1000, &cache, &components, resolver.clone()).await;
        // Now let the first test_identity expire.
        let expired_secs = almost_expired_secs + 1;
        time.set_time(epoch_secs(expired_secs));
        // Now that the first test_identity have been expired, the second test_identity will be retrieved.
        expect_identity(2000, &cache, &components, resolver.clone()).await;
    }

    #[tokio::test]
    async fn cache_partitioning() {
        let time = ManualTimeSource::new(epoch_secs(0));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        let (cache, _) = test_cache(BUFFER_TIME_NO_JITTER, Vec::new());

        #[allow(clippy::disallowed_methods)]
        let far_future = SystemTime::now() + Duration::from_secs(10_000);

        // Resolver A and B both return an identical identity type with different tokens with an expiration
        // time that should NOT be hit within this test. They each have their own partition key.
        let resolver_a_calls = Arc::new(AtomicUsize::new(0));
        let resolver_b_calls = Arc::new(AtomicUsize::new(0));
        let resolver_a = resolver_fn({
            let calls = resolver_a_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("A", Some(far_future)),
                    Some(far_future),
                )))
            }
        });
        let resolver_b = resolver_fn({
            let calls = resolver_b_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("B", Some(far_future)),
                    Some(far_future),
                )))
            }
        });
        assert_ne!(
            resolver_a.cache_partition(),
            resolver_b.cache_partition(),
            "pre-condition: they should have different partition keys"
        );

        let config_bag = ConfigBag::base();

        // Loading the identity twice with resolver A should result in a single call
        // to the underlying identity resolver since the result gets cached.
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));

        // Now, loading an identity from B will use a separate cache partition
        // and return a different result.
        let identity = cache
            .resolve_cached_identity(resolver_b.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("B", identity.data::<Token>().unwrap().token());
        let identity = cache
            .resolve_cached_identity(resolver_b.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("B", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));
        assert_eq!(1, resolver_b_calls.load(Ordering::Relaxed));

        // Finally, loading with resolver A again should return the original cached A value
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));
        assert_eq!(1, resolver_b_calls.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn eviction_when_at_capacity() {
        let time = ManualTimeSource::new(epoch_secs(0));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        // Create a cache with max_partitions=2
        let cache = LazyCache::new(
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_EXPIRATION,
            2,
        );

        #[allow(clippy::disallowed_methods)]
        let far_future = SystemTime::now() + Duration::from_secs(10_000);

        let resolver_a_calls = Arc::new(AtomicUsize::new(0));
        let resolver_b_calls = Arc::new(AtomicUsize::new(0));
        let resolver_c_calls = Arc::new(AtomicUsize::new(0));

        let resolver_a = resolver_fn({
            let calls = resolver_a_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("A", Some(far_future)),
                    Some(far_future),
                )))
            }
        });
        let resolver_b = resolver_fn({
            let calls = resolver_b_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("B", Some(far_future)),
                    Some(far_future),
                )))
            }
        });
        let resolver_c = resolver_fn({
            let calls = resolver_c_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("C", Some(far_future)),
                    Some(far_future),
                )))
            }
        });

        let config_bag = ConfigBag::base();

        // Fill the cache with A and B
        cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        cache
            .resolve_cached_identity(resolver_b.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));
        assert_eq!(1, resolver_b_calls.load(Ordering::Relaxed));

        // Adding C should evict one of A or B (arbitrary eviction order)
        cache
            .resolve_cached_identity(resolver_c.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!(1, resolver_c_calls.load(Ordering::Relaxed));

        // Resolve all three again — at least one of A or B must be re-resolved because
        // the cache only holds 2 partitions. Depending on HashMap iteration order, re-inserting
        // the evicted entry may cascade-evict the other, leading to 4 or 5 total calls.
        cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        cache
            .resolve_cached_identity(resolver_b.clone(), &components, &config_bag)
            .await
            .unwrap();
        let total_calls = resolver_a_calls.load(Ordering::Relaxed)
            + resolver_b_calls.load(Ordering::Relaxed)
            + resolver_c_calls.load(Ordering::Relaxed);
        // Initial: 3 calls (A, B, C). At least one of A or B was evicted and re-resolved (+1).
        // If re-inserting the evicted entry cascade-evicts the other, both need re-resolution (+2).
        assert!(
            (4..=5).contains(&total_calls),
            "expected 4 or 5 total calls (3 initial + 1 or 2 re-resolutions), got {total_calls}"
        );
    }

    #[tokio::test]
    async fn single_partition_cache() {
        let time = ManualTimeSource::new(epoch_secs(0));
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .with_sleep_impl(Some(TokioSleep::new()))
            .build()
            .unwrap();
        // Mimics the operation-scoped cache used for config overrides
        let cache = LazyCache::new(
            DEFAULT_LOAD_TIMEOUT,
            DEFAULT_BUFFER_TIME,
            BUFFER_TIME_NO_JITTER,
            DEFAULT_EXPIRATION,
            1,
        );

        #[allow(clippy::disallowed_methods)]
        let far_future = SystemTime::now() + Duration::from_secs(10_000);

        let resolver_a_calls = Arc::new(AtomicUsize::new(0));
        let resolver_b_calls = Arc::new(AtomicUsize::new(0));

        let resolver_a = resolver_fn({
            let calls = resolver_a_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("A", Some(far_future)),
                    Some(far_future),
                )))
            }
        });
        let resolver_b = resolver_fn({
            let calls = resolver_b_calls.clone();
            move || {
                calls.fetch_add(1, Ordering::Relaxed);
                IdentityFuture::ready(Ok(Identity::new(
                    Token::new("B", Some(far_future)),
                    Some(far_future),
                )))
            }
        });

        let config_bag = ConfigBag::base();

        // First call resolves A
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));

        // Second call with same resolver is cached
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_a_calls.load(Ordering::Relaxed));

        // Resolving B evicts A (only 1 partition)
        let identity = cache
            .resolve_cached_identity(resolver_b.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("B", identity.data::<Token>().unwrap().token());
        assert_eq!(1, resolver_b_calls.load(Ordering::Relaxed));

        // A must be re-resolved
        let identity = cache
            .resolve_cached_identity(resolver_a.clone(), &components, &config_bag)
            .await
            .unwrap();
        assert_eq!("A", identity.data::<Token>().unwrap().token());
        assert_eq!(2, resolver_a_calls.load(Ordering::Relaxed));
    }

    #[test]
    #[should_panic(expected = "max_partitions must be greater than 0")]
    fn max_partitions_zero_panics() {
        LazyCacheBuilder::new().max_partitions(0);
    }
}
