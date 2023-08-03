/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Runtime plugin type definitions.
//!
//! Runtime plugins are used to extend the runtime with custom behavior.
//! This can include:
//! - Registering interceptors
//! - Registering auth schemes
//! - Adding entries to the [`ConfigBag`](aws_smithy_types::config_bag::ConfigBag) for orchestration
//! - Setting runtime components
//!
//! Runtime plugins are divided into service/operation "levels", with service runtime plugins
//! executing before operation runtime plugins. Runtime plugins configured in a service
//! config will always be at the service level, while runtime plugins added during
//! operation customization will be at the operation level. Custom runtime plugins will
//! always run after the default runtime plugins within their level.

use crate::box_error::BoxError;
use crate::client::runtime_components::{
    RuntimeComponentsBuilder, EMPTY_RUNTIME_COMPONENTS_BUILDER,
};
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer};
use std::borrow::Cow;
use std::fmt::Debug;
use std::sync::Arc;

/// Runtime plugin trait
///
/// A `RuntimePlugin` is the unit of configuration for augmenting the SDK with new behavior.
///
/// Runtime plugins can register interceptors, set runtime components, and modify configuration.
pub trait RuntimePlugin: Debug + Send + Sync {
    /// Optionally returns additional config that should be added to the [`ConfigBag`](aws_smithy_types::config_bag::ConfigBag).
    ///
    /// As a best practice, a frozen layer should be stored on the runtime plugin instance as
    /// a member, and then cloned upon return since that clone is cheap. Constructing a new
    /// [`Layer`](aws_smithy_types::config_bag::Layer) and freezing it will require a lot of allocations.
    fn config(&self) -> Option<FrozenLayer> {
        None
    }

    /// Returns a [`RuntimeComponentsBuilder`](RuntimeComponentsBuilder) to incorporate into the final runtime components.
    ///
    /// The order of runtime plugins determines which runtime components "win". Components set by later runtime plugins will
    /// override those set by earlier runtime plugins.
    ///
    /// If no runtime component changes are desired, just return an empty builder.
    ///
    /// This method returns a [`Cow`] for flexibility. Some implementers may want to store the components builder
    /// as a member and return a reference to it, while others may need to create the builder every call. If possible,
    /// returning a reference is preferred for performance.
    fn runtime_components(&self) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&EMPTY_RUNTIME_COMPONENTS_BUILDER)
    }
}

/// Shared runtime plugin
///
/// Allows for multiple places to share ownership of one runtime plugin.
#[derive(Debug, Clone)]
pub struct SharedRuntimePlugin(Arc<dyn RuntimePlugin>);

impl SharedRuntimePlugin {
    /// Returns a new [`SharedRuntimePlugin`].
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

/// Runtime plugin that simply returns the config and components given at construction time.
#[derive(Default, Debug)]
pub struct StaticRuntimePlugin {
    config: Option<FrozenLayer>,
    runtime_components: Option<RuntimeComponentsBuilder>,
}

impl StaticRuntimePlugin {
    /// Returns a new [`StaticRuntimePlugin`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Changes the config.
    pub fn with_config(mut self, config: FrozenLayer) -> Self {
        self.config = Some(config);
        self
    }

    /// Changes the runtime components.
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

/// Used internally in the orchestrator implementation and in the generated code. Not intended to be used elsewhere.
#[doc(hidden)]
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
