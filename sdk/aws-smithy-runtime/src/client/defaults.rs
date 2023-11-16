/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Runtime plugins that provide defaults for clients.
//!
//! Note: these are the absolute base-level defaults. They may not be the defaults
//! for _your_ client, since many things can change these defaults on the way to
//! code generating and constructing a full client.

use crate::client::identity::IdentityCache;
use crate::client::retries::strategy::StandardRetryStrategy;
use crate::client::retries::RetryPartition;
use aws_smithy_async::rt::sleep::default_async_sleep;
use aws_smithy_async::time::SystemTimeSource;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
use aws_smithy_runtime_api::client::http::SharedHttpClient;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponentsBuilder, SharedConfigValidator,
};
use aws_smithy_runtime_api::client::runtime_plugin::{
    Order, SharedRuntimePlugin, StaticRuntimePlugin,
};
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;
use std::borrow::Cow;

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
pub fn default_http_client_plugin() -> Option<SharedRuntimePlugin> {
    let _default: Option<SharedHttpClient> = None;
    #[cfg(feature = "connector-hyper-0-14-x")]
    let _default = crate::client::http::hyper_014::default_client();

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
    Some(
        default_plugin("default_retry_config_plugin", |components| {
            components
                .with_retry_strategy(Some(StandardRetryStrategy::new()))
                .with_config_validator(SharedConfigValidator::base_client_config_fn(
                    validate_retry_config,
                ))
        })
        .with_config(layer("default_retry_config", |layer| {
            layer.store_put(RetryConfig::disabled());
            layer.store_put(RetryPartition::new(default_partition_name));
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

/// Arguments for the [`default_plugins`] method.
///
/// This is a struct to enable adding new parameters in the future without breaking the API.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct DefaultPluginParams {
    retry_partition_name: Option<Cow<'static, str>>,
    behavior_version: Option<BehaviorVersion>,
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
}

/// All default plugins.
pub fn default_plugins(
    params: DefaultPluginParams,
) -> impl IntoIterator<Item = SharedRuntimePlugin> {
    [
        default_http_client_plugin(),
        default_identity_cache_plugin(),
        default_retry_config_plugin(
            params
                .retry_partition_name
                .expect("retry_partition_name is required"),
        ),
        default_sleep_impl_plugin(),
        default_time_source_plugin(),
        default_timeout_config_plugin(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<SharedRuntimePlugin>>()
}
