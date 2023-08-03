/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Default connectors based on what TLS features are active. Also contains HTTP-related abstractions
//! that enable passing HTTP connectors around.

use crate::erase::DynConnector;
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_smithy_types::timeout::TimeoutConfig;
use std::time::Duration;
use std::{fmt::Debug, sync::Arc};

/// Type alias for a Connector factory function.
pub type MakeConnectorFn =
    dyn Fn(&ConnectorSettings, Option<SharedAsyncSleep>) -> Option<DynConnector> + Send + Sync;

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

impl Storable for HttpConnector {
    type Storer = StoreReplace<HttpConnector>;
}

impl HttpConnector {
    /// If `HttpConnector` is `Prebuilt`, return a clone of that connector.
    /// If `HttpConnector` is `ConnectorFn`, generate a new connector from settings and return it.
    pub fn connector(
        &self,
        settings: &ConnectorSettings,
        sleep: Option<SharedAsyncSleep>,
    ) -> Option<DynConnector> {
        match self {
            HttpConnector::Prebuilt(conn) => conn.clone(),
            HttpConnector::ConnectorFn(func) => func(settings, sleep),
        }
    }
}

/// Builder for [`ConnectorSettings`].
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct ConnectorSettingsBuilder {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl ConnectorSettingsBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    /// Sets the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn set_connect_timeout(&mut self, connect_timeout: Option<Duration>) -> &mut Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.read_timeout = Some(read_timeout);
        self
    }

    /// Sets the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn set_read_timeout(&mut self, read_timeout: Option<Duration>) -> &mut Self {
        self.read_timeout = read_timeout;
        self
    }

    /// Builds the [`ConnectorSettings`].
    pub fn build(self) -> ConnectorSettings {
        ConnectorSettings {
            connect_timeout: self.connect_timeout,
            read_timeout: self.read_timeout,
        }
    }
}

/// Settings for HTTP Connectors
#[non_exhaustive]
#[derive(Clone, Default, Debug)]
pub struct ConnectorSettings {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl ConnectorSettings {
    /// Returns a builder for `ConnectorSettings`.
    pub fn builder() -> ConnectorSettingsBuilder {
        Default::default()
    }

    /// Returns the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// Returns the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(&self) -> Option<Duration> {
        self.read_timeout
    }

    // This function may be removed/refactored in the future if other non-timeout
    // properties are added to the `ConnectorSettings` struct.
    #[doc(hidden)]
    pub fn from_timeout_config(timeout_config: &TimeoutConfig) -> Self {
        Self {
            connect_timeout: timeout_config.connect_timeout(),
            read_timeout: timeout_config.read_timeout(),
        }
    }
}
