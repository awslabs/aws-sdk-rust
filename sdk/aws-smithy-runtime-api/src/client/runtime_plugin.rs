/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::runtime_components::{
    RuntimeComponentsBuilder, EMPTY_RUNTIME_COMPONENTS_BUILDER,
};
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer};
use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

/// RuntimePlugin Trait
///
/// A RuntimePlugin is the unit of configuration for augmenting the SDK with new behavior.
///
/// Runtime plugins can register interceptors, set runtime components, and modify configuration.
pub trait RuntimePlugin: Debug + Send + Sync {
    fn config(&self) -> Option<FrozenLayer> {
        None
    }

    fn runtime_components(&self) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&EMPTY_RUNTIME_COMPONENTS_BUILDER)
    }
}

#[derive(Debug, Clone)]
pub struct SharedRuntimePlugin(Arc<dyn RuntimePlugin>);

impl SharedRuntimePlugin {
    pub fn new(plugin: impl RuntimePlugin + 'static) -> Self {
        Self(Arc::new(plugin))
    }
}

impl RuntimePlugin for SharedRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        self.0.config()
    }

    fn runtime_components(&self) -> Cow<'_, RuntimeComponentsBuilder> {
        self.0.runtime_components()
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
    ) -> Result<RuntimeComponentsBuilder, BoxError> {
        tracing::trace!("applying client runtime plugins");
        let mut builder = RuntimeComponentsBuilder::new("apply_client_configuration");
        for plugin in self.client_plugins.iter() {
            if let Some(layer) = plugin.config() {
                cfg.push_shared_layer(layer);
            }
            builder = builder.merge_from(&plugin.runtime_components());
        }
        Ok(builder)
    }

    pub fn apply_operation_configuration(
        &self,
        cfg: &mut ConfigBag,
    ) -> Result<RuntimeComponentsBuilder, BoxError> {
        tracing::trace!("applying operation runtime plugins");
        let mut builder = RuntimeComponentsBuilder::new("apply_operation_configuration");
        for plugin in self.operation_plugins.iter() {
            if let Some(layer) = plugin.config() {
                cfg.push_shared_layer(layer);
            }
            builder = builder.merge_from(&plugin.runtime_components());
        }
        Ok(builder)
    }
}

#[derive(Default, Debug)]
pub struct StaticRuntimePlugin {
    config: Option<FrozenLayer>,
    runtime_components: Option<RuntimeComponentsBuilder>,
}

impl StaticRuntimePlugin {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_config(mut self, config: FrozenLayer) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_runtime_components(mut self, runtime_components: RuntimeComponentsBuilder) -> Self {
        self.runtime_components = Some(runtime_components);
        self
    }
}

impl RuntimePlugin for StaticRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        self.config.clone()
    }

    fn runtime_components(&self) -> Cow<'_, RuntimeComponentsBuilder> {
        self.runtime_components
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_else(|| RuntimePlugin::runtime_components(self))
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
