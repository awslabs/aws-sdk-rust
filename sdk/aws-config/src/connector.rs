/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Functionality related to creating new HTTP Connectors

use std::sync::Arc;

use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_client::http_connector::HttpSettings;

// unused when all crate features are disabled
/// Unwrap an [`Option<DynConnector>`](aws_smithy_client::erase::DynConnector), and panic with a helpful error message if it's `None`
pub(crate) fn expect_connector(connector: Option<DynConnector>) -> DynConnector {
    connector.expect("A connector was not available. Either set a custom connector or enable the `rustls` and `native-tls` crate features.")
}

#[cfg(any(feature = "rustls", feature = "native-tls"))]
fn base(
    settings: &HttpSettings,
    sleep: Option<Arc<dyn AsyncSleep>>,
) -> aws_smithy_client::hyper_ext::Builder {
    let mut hyper =
        aws_smithy_client::hyper_ext::Adapter::builder().timeout(&settings.timeout_config);
    if let Some(sleep) = sleep {
        hyper = hyper.sleep_impl(sleep);
    }
    hyper
}

/// Given `HttpSettings` and an `AsyncSleep`, create a `DynConnector` from defaults depending on what cargo features are activated.
#[cfg(feature = "rustls")]
pub fn default_connector(
    settings: &HttpSettings,
    sleep: Option<Arc<dyn AsyncSleep>>,
) -> Option<DynConnector> {
    let hyper = base(settings, sleep).build(aws_smithy_client::conns::https());
    Some(DynConnector::new(hyper))
}

/// Given `HttpSettings` and an `AsyncSleep`, create a `DynConnector` from defaults depending on what cargo features are activated.
#[cfg(all(not(feature = "rustls"), feature = "native-tls"))]
pub fn default_connector(
    settings: &HttpSettings,
    sleep: Option<Arc<dyn AsyncSleep>>,
) -> Option<DynConnector> {
    let hyper = base(settings, sleep).build(aws_smithy_client::conns::native_tls());
    Some(DynConnector::new(hyper))
}

/// Given `HttpSettings` and an `AsyncSleep`, create a `DynConnector` from defaults depending on what cargo features are activated.
#[cfg(not(any(feature = "rustls", feature = "native-tls")))]
pub fn default_connector(
    _settings: &HttpSettings,
    _sleep: Option<Arc<dyn AsyncSleep>>,
) -> Option<DynConnector> {
    None
}
