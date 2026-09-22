/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Runtime plugins that provide defaults for clients.
//!
//! Note: these are the absolute base-level defaults. They may not be the defaults
//! for _your_ client, since many things can change these defaults on the way to
//! code generating and constructing a full client.

use crate::client::http::body::content_length_enforcement::EnforceContentLengthRuntimePlugin;
use crate::client::identity::IdentityCache;
use crate::client::retries::strategy::standard::TokenBucketProvider;
use crate::client::retries::strategy::StandardRetryStrategy;
use crate::client::retries::RetryPartition;
use aws_smithy_async::rt::sleep::default_async_sleep;
use aws_smithy_async::time::SystemTimeSource;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
use aws_smithy_runtime_api::client::http::SharedHttpClient;
use aws_smithy_runtime_api::client::interceptors::SharedInterceptor;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponentsBuilder, SharedConfigValidator,
};
use aws_smithy_runtime_api::client::runtime_plugin::{
    Order, SharedRuntimePlugin, StaticRuntimePlugin,
};
use aws_smithy_runtime_api::client::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use std::borrow::Cow;
use std::time::Duration;

/// Default connect timeout for all clients with BehaviorVersion >= v2026_01_12
pub(crate) const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_millis(3100);

fn default_plugin<CompFn>(name: &'static str, components_fn: CompFn) -> StaticRuntimePlugin
where
    CompFn: FnOnce(RuntimeComponentsBuilder) -> RuntimeComponentsBuilder,
{
    StaticRuntimePlugin::new()
        .with_order(Order::Defaults)
        .with_runtime_components((components_fn)(RuntimeComponentsBuilder::new(name)))
}

fn layer<LayerFn>(name: &'static str, layer_fn: LayerFn) -> FrozenLayer
where
    LayerFn: FnOnce(&mut Layer),
{
    let mut layer = Layer::new(name);
    (layer_fn)(&mut layer);
    layer.freeze()
}

/// Runtime plugin that provides a default connector.
#[deprecated(
    since = "1.8.0",
    note = "This function wasn't intended to be public, and didn't take the behavior major version as an argument, so it couldn't be evolved over time."
)]
pub fn default_http_client_plugin() -> Option<SharedRuntimePlugin> {
    #[expect(deprecated)]
    default_http_client_plugin_v2(BehaviorVersion::v2024_03_28())
}

/// Announce the upcoming default HTTP client change, while the legacy stack is still the default.
///
/// Called only where the legacy client was actually selected, which is exactly the set of
/// configurations that will resolve differently once `rustls` stops being a default feature of
/// generated SDK crates. Callers who have already pinned `legacy-https-client` cannot be
/// distinguished from callers riding the default at this layer — both arrive as `tls-rustls` — so
/// they see it too; the message is written to be actionable either way.
///
/// This is transient: delete it when that default change lands. From then on the fallback warning
/// in `default_http_client_plugin_v2` is what callers see instead.
#[cfg(feature = "connector-hyper-0-14-x")]
fn warn_legacy_client_default_is_changing(behavior_version: BehaviorVersion) {
    let emit = || {
        tracing::warn!(
            behavior_version = ?behavior_version,
            "this build resolves to the legacy hyper 0.14.x / http 0.2.x HTTP client. In the 2.x \
             release, currently expected November 2026, the default becomes the hyper 1.x client: \
             a different TLS implementation, with different connection-pooling and timeout \
             behavior. To keep the legacy client, add `features = [\"legacy-https-client\"]` to \
             your AWS SDK crate now — that spelling is stable across the change, whereas `rustls` \
             will become a synonym for the hyper 1.x client. To move early instead, use \
             `BehaviorVersion::v2026_01_12()` or later. \
             See https://github.com/smithy-lang/smithy-rs/issues/4489",
        );
    };

    // Once per process, so building a client per request does not flood the log. Under `cfg(test)`
    // every call warns, because a process-wide latch would let whichever test ran first consume
    // the only warning and make the others silently vacuous.
    #[cfg(not(test))]
    {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(emit);
    }
    #[cfg(test)]
    emit();
}

/// Runtime plugin that provides a default HTTPS connector.
pub fn default_http_client_plugin_v2(
    behavior_version: BehaviorVersion,
) -> Option<SharedRuntimePlugin> {
    let mut _default: Option<SharedHttpClient> = None;

    #[allow(deprecated)]
    if behavior_version.is_at_least(BehaviorVersion::v2026_01_12()) {
        // the latest https stack takes precedence if the config flag
        // is enabled otherwise try to fall back to the legacy connector
        // if that feature flag is available.
        #[cfg(all(
            feature = "connector-hyper-0-14-x",
            not(feature = "default-https-client")
        ))]
        #[allow(deprecated)]
        {
            _default = crate::client::http::hyper_014::default_client();

            // A legacy-only build reaches the legacy client even on a current behavior version,
            // and will resolve to hyper 1.x once `rustls` stops being a default feature.
            if _default.is_some() {
                warn_legacy_client_default_is_changing(behavior_version);
            }
        }

        // takes precedence over legacy connector if enabled
        #[cfg(feature = "default-https-client")]
        {
            let opts = crate::client::http::DefaultClientOptions::default()
                .with_behavior_version(behavior_version);
            _default = crate::client::http::default_https_client(opts);
        }
    } else {
        // fallback to legacy hyper client for given behavior version
        #[cfg(feature = "connector-hyper-0-14-x")]
        #[allow(deprecated)]
        {
            _default = crate::client::http::hyper_014::default_client();

            // The main population for the upcoming default change: an older behavior version with
            // the legacy stack compiled in, which is what a default build is today.
            if _default.is_some() {
                warn_legacy_client_default_is_changing(behavior_version);
            }
        }

        // Fall back to the latest https stack so that an older behavior version still gets a
        // working HTTP client rather than none at all. The legacy connector comes back empty both
        // when it isn't compiled in and when it is compiled in without a TLS implementation
        // (`hyper_014::default_client` requires `legacy-rustls-ring`), so key off the value rather
        // than off `connector-hyper-0-14-x`.
        //
        // NOTE: this deliberately only runs when the legacy client came back empty, so builds that
        // do have one keep getting it for these behavior versions, exactly as before.
        #[cfg(feature = "default-https-client")]
        if _default.is_none() {
            let opts = crate::client::http::DefaultClientOptions::default()
                .with_behavior_version(behavior_version);
            _default = crate::client::http::default_https_client(opts);

            // Say so rather than substituting a different HTTP stack silently: the behavior
            // version asked for the legacy one, and a caller who pinned it for a hyper 0.14.x
            // quirk needs to know they are not getting it.
            if _default.is_some() {
                tracing::warn!(
                    behavior_version = ?behavior_version,
                    "this behavior version selects the legacy hyper 0.14.x HTTP client, which is \
                     not available in this build, so the default hyper 1.x HTTPS client is being \
                     used instead. Enable the `legacy-https-client` feature on your AWS SDK crate \
                     (or `aws-smithy-runtime/tls-rustls`) to get the legacy stack, or move to \
                     `BehaviorVersion::v2026_01_12()` or later to stop seeing this warning.",
                );
            }
        }
    }

    _default.map(|default| {
        default_plugin("default_http_client_plugin", |components| {
            components.with_http_client(Some(default))
        })
        .into_shared()
    })
}

/// Runtime plugin that provides a default async sleep implementation.
pub fn default_sleep_impl_plugin() -> Option<SharedRuntimePlugin> {
    default_async_sleep().map(|default| {
        default_plugin("default_sleep_impl_plugin", |components| {
            components.with_sleep_impl(Some(default))
        })
        .into_shared()
    })
}

/// Runtime plugin that provides a default time source.
pub fn default_time_source_plugin() -> Option<SharedRuntimePlugin> {
    Some(
        default_plugin("default_time_source_plugin", |components| {
            components.with_time_source(Some(SystemTimeSource::new()))
        })
        .into_shared(),
    )
}

/// Runtime plugin that sets the default retry strategy, config (disabled), and partition.
pub fn default_retry_config_plugin(
    default_partition_name: impl Into<Cow<'static, str>>,
) -> Option<SharedRuntimePlugin> {
    let retry_partition = RetryPartition::new(default_partition_name);
    Some(
        default_plugin("default_retry_config_plugin", |components| {
            components
                .with_retry_strategy(Some(StandardRetryStrategy::new()))
                .with_config_validator(SharedConfigValidator::base_client_config_fn(
                    validate_retry_config,
                ))
                // TODO(retry 2.1 on by default): revert TokenBucketProvider to the old
                // approach: `new()` takes `init: impl FnOnce() -> TokenBucket`, eagerly
                // calls `TOKEN_BUCKET.get_or_init(default_partition.clone(), init)`, stores
                // the result directly (no OnceLock), and the hot path is just `.clone()`.
                .with_interceptor(SharedInterceptor::permanent(TokenBucketProvider::new(
                    retry_partition.clone(),
                )))
        })
        .with_config(layer("default_retry_config", |layer| {
            layer.store_put(RetryConfig::disabled());
            layer.store_put(retry_partition);
        }))
        .into_shared(),
    )
}

/// Runtime plugin that sets the default retry strategy, config, and partition.
///
/// This version respects the behavior version to enable retries by default for newer versions.
/// For AWS SDK clients with BehaviorVersion >= v2026_01_12, retries are enabled by default.
pub fn default_retry_config_plugin_v2(params: &DefaultPluginParams) -> Option<SharedRuntimePlugin> {
    let retry_partition = RetryPartition::new(
        params
            .retry_partition_name
            .as_ref()
            .expect("retry partition name is required")
            .clone(),
    );
    let is_aws_sdk = params.is_aws_sdk;
    let behavior_version = params
        .behavior_version
        .unwrap_or_else(BehaviorVersion::latest);
    Some(
        default_plugin("default_retry_config_plugin", |components| {
            components
                .with_retry_strategy(Some(StandardRetryStrategy::new()))
                .with_config_validator(SharedConfigValidator::base_client_config_fn(
                    validate_retry_config,
                ))
                .with_interceptor(SharedInterceptor::permanent(TokenBucketProvider::new(
                    retry_partition.clone(),
                )))
        })
        .with_config(layer("default_retry_config", |layer| {
            #[allow(deprecated)]
            let retry_config =
                if is_aws_sdk && behavior_version.is_at_least(BehaviorVersion::v2026_01_12()) {
                    RetryConfig::standard()
                } else {
                    RetryConfig::disabled()
                };
            layer.store_put(retry_config);
            layer.store_put(retry_partition);
        }))
        .into_shared(),
    )
}

fn validate_retry_config(
    components: &RuntimeComponentsBuilder,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    if let Some(retry_config) = cfg.load::<RetryConfig>() {
        if retry_config.has_retry() && components.sleep_impl().is_none() {
            Err("An async sleep implementation is required for retry to work. Please provide a `sleep_impl` on \
                 the config, or disable timeouts.".into())
        } else {
            Ok(())
        }
    } else {
        Err(
            "The default retry config was removed, and no other config was put in its place."
                .into(),
        )
    }
}

/// Runtime plugin that sets the default timeout config (no timeouts).
pub fn default_timeout_config_plugin() -> Option<SharedRuntimePlugin> {
    Some(
        default_plugin("default_timeout_config_plugin", |components| {
            components.with_config_validator(SharedConfigValidator::base_client_config_fn(
                validate_timeout_config,
            ))
        })
        .with_config(layer("default_timeout_config", |layer| {
            layer.store_put(TimeoutConfig::disabled());
        }))
        .into_shared(),
    )
}

/// Runtime plugin that sets the default timeout config.
///
/// This version respects the behavior version to enable connection timeout by default for newer versions.
/// For all clients with BehaviorVersion >= v2026_01_12, a 3.1s connection timeout is set.
pub fn default_timeout_config_plugin_v2(
    params: &DefaultPluginParams,
) -> Option<SharedRuntimePlugin> {
    let behavior_version = params
        .behavior_version
        .unwrap_or_else(BehaviorVersion::latest);
    Some(
        default_plugin("default_timeout_config_plugin", |components| {
            components.with_config_validator(SharedConfigValidator::base_client_config_fn(
                validate_timeout_config,
            ))
        })
        .with_config(layer("default_timeout_config", |layer| {
            #[allow(deprecated)]
            let timeout_config = if behavior_version.is_at_least(BehaviorVersion::v2026_01_12()) {
                // All clients with BMV >= v2026_01_12: Set connect_timeout only
                TimeoutConfig::builder()
                    .connect_timeout(DEFAULT_CONNECT_TIMEOUT)
                    .build()
            } else {
                // Old behavior versions: All timeouts disabled
                TimeoutConfig::disabled()
            };
            layer.store_put(timeout_config);
        }))
        .into_shared(),
    )
}

fn validate_timeout_config(
    components: &RuntimeComponentsBuilder,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    if let Some(timeout_config) = cfg.load::<TimeoutConfig>() {
        if timeout_config.has_timeouts() && components.sleep_impl().is_none() {
            Err("An async sleep implementation is required for timeouts to work. Please provide a `sleep_impl` on \
                 the config, or disable timeouts.".into())
        } else {
            Ok(())
        }
    } else {
        Err(
            "The default timeout config was removed, and no other config was put in its place."
                .into(),
        )
    }
}

/// Runtime plugin that registers the default identity cache implementation.
pub fn default_identity_cache_plugin() -> Option<SharedRuntimePlugin> {
    Some(
        default_plugin("default_identity_cache_plugin", |components| {
            components.with_identity_cache(Some(IdentityCache::lazy().build()))
        })
        .into_shared(),
    )
}

/// Runtime plugin that sets the default stalled stream protection config.
///
/// By default, when throughput falls below 1/Bs for more than 5 seconds, the
/// stream is cancelled.
#[deprecated(
    since = "1.2.0",
    note = "This function wasn't intended to be public, and didn't take the behavior major version as an argument, so it couldn't be evolved over time."
)]
pub fn default_stalled_stream_protection_config_plugin() -> Option<SharedRuntimePlugin> {
    #[expect(deprecated)]
    default_stalled_stream_protection_config_plugin_v2(BehaviorVersion::v2023_11_09())
}
fn default_stalled_stream_protection_config_plugin_v2(
    behavior_version: BehaviorVersion,
) -> Option<SharedRuntimePlugin> {
    Some(
        default_plugin(
            "default_stalled_stream_protection_config_plugin",
            |components| {
                components.with_config_validator(SharedConfigValidator::base_client_config_fn(
                    validate_stalled_stream_protection_config,
                ))
            },
        )
        .with_config(layer("default_stalled_stream_protection_config", |layer| {
            let mut config =
                StalledStreamProtectionConfig::enabled().grace_period(Duration::from_secs(5));
            // Before v2024_03_28, upload streams did not have stalled stream protection by default
            #[expect(deprecated)]
            if !behavior_version.is_at_least(BehaviorVersion::v2024_03_28()) {
                config = config.upload_enabled(false);
            }
            layer.store_put(config.build());
        }))
        .into_shared(),
    )
}

fn enforce_content_length_runtime_plugin() -> Option<SharedRuntimePlugin> {
    Some(EnforceContentLengthRuntimePlugin::new().into_shared())
}

fn validate_stalled_stream_protection_config(
    components: &RuntimeComponentsBuilder,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    if let Some(stalled_stream_protection_config) = cfg.load::<StalledStreamProtectionConfig>() {
        if stalled_stream_protection_config.is_enabled() {
            if components.sleep_impl().is_none() {
                return Err(
                    "An async sleep implementation is required for stalled stream protection to work. \
                     Please provide a `sleep_impl` on the config, or disable stalled stream protection.".into());
            }

            if components.time_source().is_none() {
                return Err(
                    "A time source is required for stalled stream protection to work.\
                     Please provide a `time_source` on the config, or disable stalled stream protection.".into());
            }
        }

        Ok(())
    } else {
        Err(
            "The default stalled stream protection config was removed, and no other config was put in its place."
                .into(),
        )
    }
}

/// Arguments for the [`default_plugins`] method.
///
/// This is a struct to enable adding new parameters in the future without breaking the API.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct DefaultPluginParams {
    retry_partition_name: Option<Cow<'static, str>>,
    behavior_version: Option<BehaviorVersion>,
    is_aws_sdk: bool,
}

impl DefaultPluginParams {
    /// Creates a new [`DefaultPluginParams`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the retry partition name.
    pub fn with_retry_partition_name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.retry_partition_name = Some(name.into());
        self
    }

    /// Sets the behavior major version.
    pub fn with_behavior_version(mut self, version: BehaviorVersion) -> Self {
        self.behavior_version = Some(version);
        self
    }

    /// Marks this as an AWS SDK client (enables retries by default for newer behavior versions).
    pub fn with_is_aws_sdk(mut self, is_aws_sdk: bool) -> Self {
        self.is_aws_sdk = is_aws_sdk;
        self
    }
}

/// All default plugins.
pub fn default_plugins(
    params: DefaultPluginParams,
) -> impl IntoIterator<Item = SharedRuntimePlugin> {
    let behavior_version = params
        .behavior_version
        .unwrap_or_else(BehaviorVersion::latest);

    [
        default_http_client_plugin_v2(behavior_version),
        default_identity_cache_plugin(),
        default_retry_config_plugin_v2(&params),
        default_sleep_impl_plugin(),
        default_time_source_plugin(),
        default_timeout_config_plugin_v2(&params),
        enforce_content_length_runtime_plugin(),
        default_stalled_stream_protection_config_plugin_v2(behavior_version),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<SharedRuntimePlugin>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::client::runtime_plugin::{RuntimePlugin, RuntimePlugins};
    #[cfg(any(feature = "default-https-client", feature = "tls-rustls"))]
    use tracing_test::traced_test;

    fn test_plugin_params(version: BehaviorVersion) -> DefaultPluginParams {
        DefaultPluginParams::new()
            .with_behavior_version(version)
            .with_retry_partition_name("dontcare")
            .with_is_aws_sdk(false) // Default to non-AWS SDK for existing tests
    }
    fn config_for(plugins: impl IntoIterator<Item = SharedRuntimePlugin>) -> ConfigBag {
        let mut config = ConfigBag::base();
        let plugins = RuntimePlugins::new().with_client_plugins(plugins);
        plugins.apply_client_configuration(&mut config).unwrap();
        config
    }

    #[test]
    #[expect(deprecated)]
    fn v2024_03_28_stalled_stream_protection_difference() {
        let latest = config_for(default_plugins(test_plugin_params(
            BehaviorVersion::latest(),
        )));
        let v2023 = config_for(default_plugins(test_plugin_params(
            BehaviorVersion::v2023_11_09(),
        )));

        assert!(
            latest
                .load::<StalledStreamProtectionConfig>()
                .unwrap()
                .upload_enabled(),
            "stalled stream protection on uploads MUST be enabled after v2024_03_28"
        );
        assert!(
            !v2023
                .load::<StalledStreamProtectionConfig>()
                .unwrap()
                .upload_enabled(),
            "stalled stream protection on uploads MUST NOT be enabled before v2024_03_28"
        );
    }

    #[test]
    fn test_retry_enabled_for_aws_sdk() {
        let params = DefaultPluginParams::new()
            .with_retry_partition_name("test-partition")
            .with_behavior_version(BehaviorVersion::latest())
            .with_is_aws_sdk(true);
        let plugin = default_retry_config_plugin_v2(&params).expect("plugin should be created");

        let config = plugin.config().expect("config should exist");
        let retry_config = config
            .load::<RetryConfig>()
            .expect("retry config should exist");

        assert_eq!(
            retry_config.max_attempts(),
            3,
            "retries should be enabled with max_attempts=3 for AWS SDK with latest behavior version"
        );
    }

    #[test]
    #[expect(deprecated)]
    fn test_retry_disabled_for_aws_sdk_old_behavior_version() {
        // Any version before v2026_01_12 should have retries disabled
        let params = DefaultPluginParams::new()
            .with_retry_partition_name("test-partition")
            .with_behavior_version(BehaviorVersion::v2024_03_28())
            .with_is_aws_sdk(true);
        let plugin = default_retry_config_plugin_v2(&params).expect("plugin should be created");

        let config = plugin.config().expect("config should exist");
        let retry_config = config
            .load::<RetryConfig>()
            .expect("retry config should exist");

        assert_eq!(
            retry_config.max_attempts(),
            1,
            "retries should be disabled for AWS SDK with behavior version < v2026_01_12"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_retry_enabled_at_cutoff_version() {
        // v2026_01_12 is the cutoff - retries should be enabled from this version onwards
        let params = DefaultPluginParams::new()
            .with_retry_partition_name("test-partition")
            .with_behavior_version(BehaviorVersion::v2026_01_12())
            .with_is_aws_sdk(true);
        let plugin = default_retry_config_plugin_v2(&params).expect("plugin should be created");

        let config = plugin.config().expect("config should exist");
        let retry_config = config
            .load::<RetryConfig>()
            .expect("retry config should exist");

        assert_eq!(
            retry_config.max_attempts(),
            3,
            "retries should be enabled for AWS SDK starting from v2026_01_12"
        );
    }

    #[test]
    fn test_retry_disabled_for_non_aws_sdk() {
        let params = DefaultPluginParams::new()
            .with_retry_partition_name("test-partition")
            .with_behavior_version(BehaviorVersion::latest())
            .with_is_aws_sdk(false);
        let plugin = default_retry_config_plugin_v2(&params).expect("plugin should be created");

        let config = plugin.config().expect("config should exist");
        let retry_config = config
            .load::<RetryConfig>()
            .expect("retry config should exist");

        assert_eq!(
            retry_config.max_attempts(),
            1,
            "retries should be disabled for non-AWS SDK clients"
        );
    }

    #[test]
    #[expect(deprecated)]
    fn test_behavior_version_gates_retry_for_aws_sdk() {
        // This test demonstrates the complete behavior:
        // AWS SDK clients get retries enabled ONLY when BehaviorVersion >= v2026_01_12

        // Test all behavior versions
        let test_cases = vec![
            (BehaviorVersion::v2023_11_09(), 1, "v2023_11_09 (old)"),
            (BehaviorVersion::v2024_03_28(), 1, "v2024_03_28 (old)"),
            (BehaviorVersion::v2025_01_17(), 1, "v2025_01_17 (old)"),
            (BehaviorVersion::v2025_08_07(), 1, "v2025_08_07 (old)"),
            (BehaviorVersion::v2026_01_12(), 3, "v2026_01_12 (cutoff)"),
            (BehaviorVersion::latest(), 3, "latest"),
        ];

        for (version, expected_attempts, version_name) in test_cases {
            let params = DefaultPluginParams::new()
                .with_retry_partition_name("test-partition")
                .with_behavior_version(version)
                .with_is_aws_sdk(true);

            let plugin = default_retry_config_plugin_v2(&params).expect("plugin should be created");
            let config = plugin.config().expect("config should exist");
            let retry_config = config
                .load::<RetryConfig>()
                .expect("retry config should exist");

            assert_eq!(
                retry_config.max_attempts(),
                expected_attempts,
                "AWS SDK with {} should have {} max attempts",
                version_name,
                expected_attempts
            );
        }
    }

    #[test]
    #[expect(deprecated)]
    fn test_complete_default_plugins_integration() {
        // This test simulates the complete flow as it would happen in a real AWS SDK client
        // It verifies that default_plugins() correctly applies retry config based on
        // both is_aws_sdk flag and BehaviorVersion

        // Scenario 1: AWS SDK with latest behavior version -> retries enabled
        let params_aws_latest = DefaultPluginParams::new()
            .with_retry_partition_name("aws-s3")
            .with_behavior_version(BehaviorVersion::latest())
            .with_is_aws_sdk(true);

        let config_aws_latest = config_for(default_plugins(params_aws_latest));
        let retry_aws_latest = config_aws_latest
            .load::<RetryConfig>()
            .expect("retry config should exist");
        assert_eq!(
            retry_aws_latest.max_attempts(),
            3,
            "AWS SDK with latest behavior version should have retries enabled (3 attempts)"
        );

        // Scenario 2: AWS SDK with old behavior version -> retries disabled
        let params_aws_old = DefaultPluginParams::new()
            .with_retry_partition_name("aws-s3")
            .with_behavior_version(BehaviorVersion::v2024_03_28())
            .with_is_aws_sdk(true);

        let config_aws_old = config_for(default_plugins(params_aws_old));
        let retry_aws_old = config_aws_old
            .load::<RetryConfig>()
            .expect("retry config should exist");
        assert_eq!(
            retry_aws_old.max_attempts(),
            1,
            "AWS SDK with old behavior version should have retries disabled (1 attempt)"
        );

        // Scenario 3: Non-AWS SDK (generic Smithy client) -> retries always disabled
        let params_generic = DefaultPluginParams::new()
            .with_retry_partition_name("my-service")
            .with_behavior_version(BehaviorVersion::latest())
            .with_is_aws_sdk(false);

        let config_generic = config_for(default_plugins(params_generic));
        let retry_generic = config_generic
            .load::<RetryConfig>()
            .expect("retry config should exist");
        assert_eq!(
            retry_generic.max_attempts(),
            1,
            "Non-AWS SDK clients should always have retries disabled (1 attempt)"
        );

        // Scenario 4: Verify the cutoff version v2026_01_12 is the exact boundary
        let params_cutoff = DefaultPluginParams::new()
            .with_retry_partition_name("aws-s3")
            .with_behavior_version(BehaviorVersion::v2026_01_12())
            .with_is_aws_sdk(true);

        let config_cutoff = config_for(default_plugins(params_cutoff));
        let retry_cutoff = config_cutoff
            .load::<RetryConfig>()
            .expect("retry config should exist");
        assert_eq!(
            retry_cutoff.max_attempts(),
            3,
            "AWS SDK with v2026_01_12 (the cutoff version) should have retries enabled (3 attempts)"
        );
    }

    /// A behavior version older than `v2026_01_12` must still end up with an HTTP client whenever
    /// the hyper 1.x stack is compiled in, rather than with none at all.
    ///
    /// The configuration that actually exercises the fallback is `connector-hyper-0-14-x` plus
    /// `default-https-client` with no legacy TLS implementation: `hyper_014::default_client()`
    /// returns `None` there, so the fallback is the only thing that can supply a client. Note that
    /// `--all-features` does *not* exercise it, because `tls-rustls` gives the legacy connector a
    /// TLS implementation and it returns `Some`, which would satisfy the assertion below no matter
    /// what the fallback did. `tools/ci-scripts/check-rust-runtimes` runs that combination
    /// explicitly so this test has teeth.
    #[test]
    #[expect(deprecated)]
    fn old_behavior_version_still_gets_an_http_client() {
        let old = default_http_client_plugin_v2(BehaviorVersion::v2024_03_28());
        let latest = default_http_client_plugin_v2(BehaviorVersion::latest());

        // The hyper 1.x stack is available, so both behavior versions get a client: the latest
        // directly, and the older one either from a working legacy connector or from the fallback.
        #[cfg(feature = "default-https-client")]
        {
            assert!(
                old.is_some(),
                "a pre-v2026_01_12 behavior version must fall back to the hyper 1.x client \
                 instead of getting no HTTP client"
            );
            assert!(
                latest.is_some(),
                "the latest behavior version must get the hyper 1.x client"
            );
        }

        // No hyper 1.x stack, so there is nothing to fall back to and the legacy connector is the
        // only possible source. It yields a client only when it also has a TLS implementation.
        #[cfg(all(not(feature = "default-https-client"), feature = "tls-rustls"))]
        assert!(
            old.is_some(),
            "a pre-v2026_01_12 behavior version must still get the legacy client when that is \
             the only stack compiled in"
        );

        // Neither stack is compiled in, so no default client is possible for either version.
        #[cfg(all(
            not(feature = "default-https-client"),
            not(feature = "connector-hyper-0-14-x")
        ))]
        {
            assert!(
                old.is_none(),
                "no HTTP client stack is compiled in, so there is nothing to install"
            );
            assert!(
                latest.is_none(),
                "no HTTP client stack is compiled in, so there is nothing to install"
            );
        }

        let _ = (old, latest);
    }

    /// Falling back must not be silent: a caller who pinned an old behavior version for a
    /// hyper 0.14.x quirk needs to learn that they are on the hyper 1.x client instead.
    ///
    /// Gated to the configurations where the fallback actually runs. With `tls-rustls` the legacy
    /// connector has a TLS implementation and returns `Some`, so no fallback happens and there is
    /// correctly nothing to warn about.
    #[test]
    #[traced_test]
    #[expect(deprecated)]
    #[cfg(all(feature = "default-https-client", not(feature = "tls-rustls")))]
    fn falling_back_to_the_hyper_1x_client_warns() {
        let old = default_http_client_plugin_v2(BehaviorVersion::v2024_03_28());
        assert!(old.is_some(), "the fallback should have supplied a client");
        assert!(
            logs_contain("selects the legacy hyper 0.14.x HTTP client"),
            "falling back to the hyper 1.x client must be logged"
        );
    }

    /// While the legacy stack is still the default, a build that resolves to it must be told the
    /// default is changing — otherwise the change arrives with no notice.
    ///
    /// Gated to a build where the legacy client actually yields something: `hyper_014::default_client`
    /// needs `legacy-rustls-ring`, which `tls-rustls` supplies.
    #[test]
    #[traced_test]
    #[expect(deprecated)]
    #[cfg(feature = "tls-rustls")]
    fn resolving_to_the_legacy_client_warns_that_the_default_is_changing() {
        let old = default_http_client_plugin_v2(BehaviorVersion::v2024_03_28());
        assert!(old.is_some(), "the legacy client should have been selected");
        assert!(
            logs_contain("the default becomes the hyper 1.x client"),
            "resolving to the legacy client must announce the upcoming default change"
        );
        assert!(
            logs_contain("legacy-https-client"),
            "the warning must name the feature that pins the legacy client"
        );
    }

    /// The warning is about *resolving to* the legacy client, so a build without it must stay quiet
    /// rather than warn about a stack it never had.
    #[test]
    #[traced_test]
    #[expect(deprecated)]
    #[cfg(all(
        feature = "default-https-client",
        not(feature = "connector-hyper-0-14-x")
    ))]
    fn a_build_without_the_legacy_client_does_not_warn_about_it() {
        let _ = default_http_client_plugin_v2(BehaviorVersion::v2024_03_28());
        assert!(
            !logs_contain("the default becomes the hyper 1.x client"),
            "a build with no legacy client must not warn about the legacy default changing"
        );
    }

    /// The converse: a behavior version that asks for the current stack has nothing to warn about.
    #[test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    fn the_latest_behavior_version_does_not_warn() {
        let latest = default_http_client_plugin_v2(BehaviorVersion::latest());
        assert!(latest.is_some(), "the latest version should get a client");
        assert!(
            !logs_contain("selects the legacy hyper 0.14.x HTTP client"),
            "the latest behavior version must not warn about the legacy stack"
        );
    }
}
