/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functionality related to creating new HTTP Connectors

use aws_smithy_client::erase::DynConnector;

/// Unwrap an [`Option<DynConnector>`](aws_smithy_client::erase::DynConnector), and panic with a helpful error message if it's `None`
pub(crate) fn expect_connector(for_what: &str, connector: Option<DynConnector>) -> DynConnector {
    if let Some(conn) = connector {
        conn
    } else {
        panic!("{for_what} require(s) a HTTP connector, but none was available. Enable the `rustls` crate feature or set a connector to fix this.")
    }
}

#[cfg(feature = "client-hyper")]
pub use aws_smithy_client::conns::default_connector;

#[cfg(all(feature = "native-tls", not(feature = "allow-compilation")))]
compile_error!("Feature native-tls has been removed. For upgrade instructions, see: https://awslabs.github.io/smithy-rs/design/transport/connector.html");

/// Given `ConnectorSettings` and a [`SharedAsyncSleep`](aws_smithy_async::rt::sleep::SharedAsyncSleep), create a `DynConnector` from defaults depending on what cargo features are activated.
#[cfg(not(feature = "client-hyper"))]
pub fn default_connector(
    _settings: &aws_smithy_client::http_connector::ConnectorSettings,
    _sleep: Option<aws_smithy_async::rt::sleep::SharedAsyncSleep>,
) -> Option<DynConnector> {
    None
}
