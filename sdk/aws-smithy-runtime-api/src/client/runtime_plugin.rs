/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::config_bag::ConfigBag;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub trait RuntimePlugin {
    fn configure(&self, cfg: &mut ConfigBag) -> Result<(), BoxError>;
}

impl RuntimePlugin for Box<dyn RuntimePlugin> {
    fn configure(&self, cfg: &mut ConfigBag) -> Result<(), BoxError> {
        self.as_ref().configure(cfg)
    }
}

#[derive(Default)]
pub struct RuntimePlugins {
    client_plugins: Vec<Box<dyn RuntimePlugin>>,
    operation_plugins: Vec<Box<dyn RuntimePlugin>>,
}

impl RuntimePlugins {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_client_plugin(mut self, plugin: impl RuntimePlugin + 'static) -> Self {
        self.client_plugins.push(Box::new(plugin));
        self
    }

    pub fn with_operation_plugin(mut self, plugin: impl RuntimePlugin + 'static) -> Self {
        self.operation_plugins.push(Box::new(plugin));
        self
    }

    pub fn apply_client_configuration(&self, cfg: &mut ConfigBag) -> Result<(), BoxError> {
        for plugin in self.client_plugins.iter() {
            plugin.configure(cfg)?;
        }

        Ok(())
    }

    pub fn apply_operation_configuration(&self, cfg: &mut ConfigBag) -> Result<(), BoxError> {
        for plugin in self.operation_plugins.iter() {
            plugin.configure(cfg)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{BoxError, RuntimePlugin, RuntimePlugins};
    use crate::config_bag::ConfigBag;

    struct SomeStruct;

    impl RuntimePlugin for SomeStruct {
        fn configure(&self, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
            todo!()
        }
    }

    #[test]
    fn can_add_runtime_plugin_implementors_to_runtime_plugins() {
        RuntimePlugins::new().with_client_plugin(SomeStruct);
    }
}
