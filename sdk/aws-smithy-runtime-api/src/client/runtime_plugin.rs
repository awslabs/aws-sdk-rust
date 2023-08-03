/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::interceptors::InterceptorRegistrar;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer};
use std::fmt::Debug;
use std::sync::Arc;

/// RuntimePlugin Trait
///
/// A RuntimePlugin is the unit of configuration for augmenting the SDK with new behavior
///
/// Runtime plugins can set configuration and register interceptors.
pub trait RuntimePlugin: Debug + Send + Sync {
    fn config(&self) -> Option<FrozenLayer> {
        None
    }

    fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
        let _ = interceptors;
    }
}

#[derive(Debug, Clone)]
struct SharedRuntimePlugin(Arc<dyn RuntimePlugin>);

impl SharedRuntimePlugin {
    fn new(plugin: impl RuntimePlugin + 'static) -> Self {
        Self(Arc::new(plugin))
    }
}

impl RuntimePlugin for SharedRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        self.0.config()
    }

    fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
        self.0.interceptors(interceptors)
    }
}

#[derive(Default, Clone, Debug)]
pub struct RuntimePlugins {
    client_plugins: Vec<SharedRuntimePlugin>,
    operation_plugins: Vec<SharedRuntimePlugin>,
}

impl RuntimePlugins {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_client_plugin(mut self, plugin: impl RuntimePlugin + 'static) -> Self {
        self.client_plugins.push(SharedRuntimePlugin::new(plugin));
        self
    }

    pub fn with_operation_plugin(mut self, plugin: impl RuntimePlugin + 'static) -> Self {
        self.operation_plugins
            .push(SharedRuntimePlugin::new(plugin));
        self
    }

    pub fn apply_client_configuration(
        &self,
        cfg: &mut ConfigBag,
        interceptors: &mut InterceptorRegistrar,
    ) -> Result<(), BoxError> {
        tracing::trace!("applying client runtime plugins");
        for plugin in self.client_plugins.iter() {
            if let Some(layer) = plugin.config() {
                cfg.push_shared_layer(layer);
            }
            plugin.interceptors(interceptors);
        }

        Ok(())
    }

    pub fn apply_operation_configuration(
        &self,
        cfg: &mut ConfigBag,
        interceptors: &mut InterceptorRegistrar,
    ) -> Result<(), BoxError> {
        tracing::trace!("applying operation runtime plugins");
        for plugin in self.operation_plugins.iter() {
            if let Some(layer) = plugin.config() {
                cfg.push_shared_layer(layer);
            }
            plugin.interceptors(interceptors);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{RuntimePlugin, RuntimePlugins};

    #[derive(Debug)]
    struct SomeStruct;

    impl RuntimePlugin for SomeStruct {}

    #[test]
    fn can_add_runtime_plugin_implementors_to_runtime_plugins() {
        RuntimePlugins::new().with_client_plugin(SomeStruct);
    }

    #[test]
    fn runtime_plugins_are_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RuntimePlugins>();
    }
}
