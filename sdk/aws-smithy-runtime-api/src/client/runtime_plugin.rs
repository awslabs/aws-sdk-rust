/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::interceptors::InterceptorRegistrar;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer};
use std::fmt::Debug;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxRuntimePlugin = Box<dyn RuntimePlugin + Send + Sync>;

/// RuntimePlugin Trait
///
/// A RuntimePlugin is the unit of configuration for augmenting the SDK with new behavior
///
/// Runtime plugins can set configuration and register interceptors.
pub trait RuntimePlugin: Debug {
    fn config(&self) -> Option<FrozenLayer> {
        None
    }

    fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
        let _ = interceptors;
    }
}

impl RuntimePlugin for BoxRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        self.as_ref().config()
    }

    fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
        self.as_ref().interceptors(interceptors)
    }
}

#[derive(Default)]
pub struct RuntimePlugins {
    client_plugins: Vec<BoxRuntimePlugin>,
    operation_plugins: Vec<BoxRuntimePlugin>,
}

impl RuntimePlugins {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_client_plugin(
        mut self,
        plugin: impl RuntimePlugin + Send + Sync + 'static,
    ) -> Self {
        self.client_plugins.push(Box::new(plugin));
        self
    }

    pub fn with_operation_plugin(
        mut self,
        plugin: impl RuntimePlugin + Send + Sync + 'static,
    ) -> Self {
        self.operation_plugins.push(Box::new(plugin));
        self
    }

    pub fn apply_client_configuration(
        &self,
        cfg: &mut ConfigBag,
        interceptors: &mut InterceptorRegistrar,
    ) -> Result<(), BoxError> {
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
}
