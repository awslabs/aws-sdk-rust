/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::http::SharedHttpClient;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
use aws_smithy_runtime_api::client::runtime_plugin::{
    Order, SharedRuntimePlugin, StaticRuntimePlugin,
};

/// Interceptor for connection poisoning.
pub mod connection_poisoning;

#[cfg(feature = "test-util")]
pub mod test_util;

/// Default HTTP and TLS connectors that use hyper 0.14.x and rustls.
///
/// This module is named after the hyper version number since we anticipate
/// needing to provide equivalent functionality for hyper 1.x in the future.
#[cfg(feature = "connector-hyper-0-14-x")]
pub mod hyper_014;

/// Runtime plugin that provides a default connector. Intended to be used by the generated code.
pub fn default_http_client_plugin() -> SharedRuntimePlugin {
    let _default: Option<SharedHttpClient> = None;
    #[cfg(feature = "connector-hyper-0-14-x")]
    let _default = hyper_014::default_client();

    let plugin = StaticRuntimePlugin::new()
        .with_order(Order::Defaults)
        .with_runtime_components(
            RuntimeComponentsBuilder::new("default_http_client_plugin").with_http_client(_default),
        );
    SharedRuntimePlugin::new(plugin)
}
