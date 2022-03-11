/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Default connectors based on what TLS features are active. Also contains HTTP-related abstractions
//! that enable passing HTTP connectors around.

use crate::erase::DynConnector;
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_types::timeout;
use std::{fmt::Debug, sync::Arc};

/// Type alias for a Connector factory function.
pub type MakeConnectorFn =
    dyn Fn(&HttpSettings, Option<Arc<dyn AsyncSleep>>) -> Option<DynConnector> + Send + Sync;

/// Enum for describing the two "kinds" of HTTP Connectors in smithy-rs.
#[derive(Clone)]
pub enum HttpConnector {
    /// A `DynConnector` to be used for all requests.
    Prebuilt(Option<DynConnector>),
    /// A factory function that will be used to create new `DynConnector`s whenever one is needed.
    ConnectorFn(Arc<MakeConnectorFn>),
}

impl Debug for HttpConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prebuilt(Some(connector)) => {
                write!(f, "Prebuilt({:?})", connector)
            }
            Self::Prebuilt(None) => {
                write!(f, "Prebuilt(None)")
            }
            Self::ConnectorFn(_) => {
                write!(f, "ConnectorFn(<function pointer>)")
            }
        }
    }
}

impl HttpConnector {
    /// If `HttpConnector` is `Prebuilt`, return a clone of that connector.
    /// If `HttpConnector` is `ConnectorFn`, generate a new connector from settings and return it.
    pub fn connector(
        &self,
        settings: &HttpSettings,
        sleep: Option<Arc<dyn AsyncSleep>>,
    ) -> Option<DynConnector> {
        match self {
            HttpConnector::Prebuilt(conn) => conn.clone(),
            HttpConnector::ConnectorFn(func) => func(settings, sleep),
        }
    }
}

/// HttpSettings for HTTP Connectors
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct HttpSettings {
    /// Timeout configuration used when making HTTP connections
    pub http_timeout_config: timeout::Http,
    /// Timeout configuration used when creating TCP connections
    pub tcp_timeout_config: timeout::Tcp,
}

impl HttpSettings {
    /// Set the HTTP timeouts to be used when making HTTP connections
    pub fn with_http_timeout_config(mut self, http_timeout_config: timeout::Http) -> Self {
        self.http_timeout_config = http_timeout_config;
        self
    }

    /// Set the TCP timeouts to be used when creating TCP connections
    pub fn with_tcp_timeout_config(mut self, tcp_timeout_config: timeout::Tcp) -> Self {
        self.tcp_timeout_config = tcp_timeout_config;
        self
    }
}
