/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Timeout Configuration
//!
//! While timeout configuration is unstable, this module is in aws-smithy-client.
//!
//! As timeout and HTTP configuration stabilizes, this will move to aws-types and become a part of
//! HttpSettings.
use std::time::Duration;

/// Timeout Configuration
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Settings {
    connect_timeout: Option<Duration>,
    http_read_timeout: Option<Duration>,
}

impl Settings {
    /// Create a new timeout configuration
    ///
    /// The current default (subject to change) is no timeouts
    pub fn new() -> Self {
        Self {
            connect_timeout: None,
            http_read_timeout: None,
        }
    }

    /// The configured TCP-connect timeout
    pub fn connect(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// The configured HTTP-read timeout
    pub fn read(&self) -> Option<Duration> {
        self.http_read_timeout
    }

    /// Sets the connect timeout
    pub fn with_connect_timeout(self, connect_timeout: Duration) -> Self {
        Self {
            connect_timeout: Some(connect_timeout),
            ..self
        }
    }

    /// Sets the read timeout
    pub fn with_read_timeout(self, read_timeout: Duration) -> Self {
        Self {
            http_read_timeout: Some(read_timeout),
            ..self
        }
    }
}
