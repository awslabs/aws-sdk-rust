/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Top-level configuration for timeouts
///
/// # Example
///
/// ```rust
/// # use std::time::Duration;
///
/// # fn main() {
/// use aws_smithy_types::{timeout, tristate::TriState};
///
/// let api_timeouts = timeout::Api::new()
///     .with_call_timeout(TriState::Set(Duration::from_secs(2)))
///     .with_call_attempt_timeout(TriState::Set(Duration::from_secs_f32(0.5)));
/// let timeout_config = timeout::Config::new()
///     .with_api_timeouts(api_timeouts);
///
/// assert_eq!(
///     timeout_config.api.call_timeout(),
///     TriState::Set(Duration::from_secs(2))
/// );
///
/// assert_eq!(
///     timeout_config.api.call_attempt_timeout(),
///     TriState::Set(Duration::from_secs_f32(0.5))
/// );
/// # }
/// ```
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Config {
    /// API timeouts used by Smithy `Client`s
    pub api: super::Api,
    /// HTTP timeouts used by `DynConnector`s
    pub http: super::Http,
    /// TCP timeouts used by lower-level `DynConnector`s
    pub tcp: super::Tcp,
}

impl Config {
    /// Create a new `Config` with no timeouts set
    pub fn new() -> Self {
        Default::default()
    }

    /// Return the API-related timeouts from this config
    pub fn api_timeouts(&self) -> super::Api {
        self.api.clone()
    }

    /// Return the API-related timeouts from this config
    pub fn http_timeouts(&self) -> super::Http {
        self.http.clone()
    }

    /// Return the API-related timeouts from this config
    pub fn tcp_timeouts(&self) -> super::Tcp {
        self.tcp.clone()
    }

    /// Consume an `Config` to create a new one, setting the API-related timeouts
    pub fn with_api_timeouts(mut self, timeouts: super::Api) -> Self {
        self.api = timeouts;
        self
    }

    /// Consume a `Config` to create a new one, setting HTTP-related timeouts
    pub fn with_http_timeouts(mut self, timeouts: super::Http) -> Self {
        self.http = timeouts;
        self
    }

    /// Consume a `Config` to create a new one, setting TCP-related timeouts
    pub fn with_tcp_timeouts(mut self, timeouts: super::Tcp) -> Self {
        self.tcp = timeouts;
        self
    }

    /// Merges two timeout configs together.
    ///
    /// Values from `other` will only be used as a fallback for values
    /// from `self`. Useful for merging configs from different sources together when you want to
    /// handle "precedence" per value instead of at the config level
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// # use aws_smithy_types::timeout;
    /// # use aws_smithy_types::tristate::TriState;
    /// let a = timeout::Config::new().with_api_timeouts(
    ///     timeout::Api::new().with_call_timeout(TriState::Set(Duration::from_secs(2)))
    /// );
    /// let b = timeout::Config::new().with_api_timeouts(
    ///     timeout::Api::new().with_call_attempt_timeout(TriState::Set(Duration::from_secs(3)))
    /// );
    /// let timeout_config = a.take_unset_from(b);
    /// // A's value take precedence over B's value
    /// assert_eq!(timeout_config.api.call_timeout(), TriState::Set(Duration::from_secs(2)));
    /// // A never set a connect timeout so B's value was used
    /// assert_eq!(timeout_config.api.call_attempt_timeout(), TriState::Set(Duration::from_secs(3)));
    /// ```
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            api: self.api.take_unset_from(other.api),
            http: self.http.take_unset_from(other.http),
            tcp: self.tcp.take_unset_from(other.tcp),
        }
    }

    /// Returns true if any of the possible timeouts are se
    pub fn has_timeouts(&self) -> bool {
        self.api.has_timeouts() || self.http.has_timeouts() || self.tcp.has_timeouts()
    }
}
