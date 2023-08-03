/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! This module defines types that describe timeouts that can be applied to various stages of the
//! Smithy networking stack.

use crate::config_bag::{Storable, StoreReplace};
use std::time::Duration;

/// Builder for [`TimeoutConfig`].
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct TimeoutConfigBuilder {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    operation_timeout: Option<Duration>,
    operation_attempt_timeout: Option<Duration>,
}

impl TimeoutConfigBuilder {
    /// Creates a new builder with no timeouts set.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the connect timeout.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    /// Sets the connect timeout.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn set_connect_timeout(&mut self, connect_timeout: Option<Duration>) -> &mut Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets the read timeout.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.read_timeout = Some(read_timeout);
        self
    }

    /// Sets the read timeout.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn set_read_timeout(&mut self, read_timeout: Option<Duration>) -> &mut Self {
        self.read_timeout = read_timeout;
        self
    }

    /// Sets the operation timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// The operation timeout is a limit on the total amount of time it takes for an operation to be
    /// fully serviced, including the time for all retries that may have been attempted for it.
    ///
    /// If you want to set a timeout on individual retry attempts, then see [`Self::operation_attempt_timeout`]
    /// or [`Self::set_operation_attempt_timeout`].
    pub fn operation_timeout(mut self, operation_timeout: Duration) -> Self {
        self.operation_timeout = Some(operation_timeout);
        self
    }

    /// Sets the operation timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// The operation timeout is a limit on the total amount of time it takes for an operation to be
    /// fully serviced, including the time for all retries that may have been attempted for it.
    ///
    /// If you want to set a timeout on individual retry attempts, then see [`Self::operation_attempt_timeout`]
    /// or [`Self::set_operation_attempt_timeout`].
    pub fn set_operation_timeout(&mut self, operation_timeout: Option<Duration>) -> &mut Self {
        self.operation_timeout = operation_timeout;
        self
    }

    /// Sets the operation attempt timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// When retries are enabled, then this setting makes it possible to set a timeout for individual
    /// retry attempts (including the initial attempt) for an operation.
    ///
    /// If you want to set a timeout on the total time for an entire request including all of its retries,
    /// then see [`Self::operation_timeout`] /// or [`Self::set_operation_timeout`].
    pub fn operation_attempt_timeout(mut self, operation_attempt_timeout: Duration) -> Self {
        self.operation_attempt_timeout = Some(operation_attempt_timeout);
        self
    }

    /// Sets the operation attempt timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// When retries are enabled, then this setting makes it possible to set a timeout for individual
    /// retry attempts (including the initial attempt) for an operation.
    ///
    /// If you want to set a timeout on individual retry attempts, then see [`Self::operation_attempt_timeout`]
    /// or [`Self::set_operation_attempt_timeout`].
    pub fn set_operation_attempt_timeout(
        &mut self,
        operation_attempt_timeout: Option<Duration>,
    ) -> &mut Self {
        self.operation_attempt_timeout = operation_attempt_timeout;
        self
    }

    /// Merges two timeout config builders together.
    ///
    /// Values from `other` will only be used as a fallback for values
    /// from `self`. Useful for merging configs from different sources together when you want to
    /// handle "precedence" per value instead of at the config level
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// # use aws_smithy_types::timeout::TimeoutConfig;
    /// let a = TimeoutConfig::builder()
    ///     .connect_timeout(Duration::from_secs(3));
    /// let b = TimeoutConfig::builder()
    ///     .connect_timeout(Duration::from_secs(5))
    ///     .operation_timeout(Duration::from_secs(3));
    /// let timeout_config = a.take_unset_from(b).build();
    ///
    /// // A's value take precedence over B's value
    /// assert_eq!(timeout_config.connect_timeout(), Some(Duration::from_secs(3)));
    /// // A never set an operation timeout so B's value is used
    /// assert_eq!(timeout_config.operation_timeout(), Some(Duration::from_secs(3)));
    /// ```
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            connect_timeout: self.connect_timeout.or(other.connect_timeout),
            read_timeout: self.read_timeout.or(other.read_timeout),
            operation_timeout: self.operation_timeout.or(other.operation_timeout),
            operation_attempt_timeout: self
                .operation_attempt_timeout
                .or(other.operation_attempt_timeout),
        }
    }

    /// Builds a `TimeoutConfig`.
    pub fn build(self) -> TimeoutConfig {
        TimeoutConfig {
            connect_timeout: self.connect_timeout,
            read_timeout: self.read_timeout,
            operation_timeout: self.operation_timeout,
            operation_attempt_timeout: self.operation_attempt_timeout,
        }
    }
}

impl From<TimeoutConfig> for TimeoutConfigBuilder {
    fn from(timeout_config: TimeoutConfig) -> Self {
        TimeoutConfigBuilder {
            connect_timeout: timeout_config.connect_timeout,
            read_timeout: timeout_config.read_timeout,
            operation_timeout: timeout_config.operation_timeout,
            operation_attempt_timeout: timeout_config.operation_attempt_timeout,
        }
    }
}

/// Top-level configuration for timeouts
///
/// # Example
///
/// ```rust
/// # use std::time::Duration;
///
/// # fn main() {
/// use aws_smithy_types::timeout::TimeoutConfig;
///
/// let timeout_config = TimeoutConfig::builder()
///     .operation_timeout(Duration::from_secs(30))
///     .operation_attempt_timeout(Duration::from_secs(10))
///     .connect_timeout(Duration::from_secs(3))
///     .build();
///
/// assert_eq!(
///     timeout_config.operation_timeout(),
///     Some(Duration::from_secs(30))
/// );
/// assert_eq!(
///     timeout_config.operation_attempt_timeout(),
///     Some(Duration::from_secs(10))
/// );
/// assert_eq!(
///     timeout_config.connect_timeout(),
///     Some(Duration::from_secs(3))
/// );
/// # }
/// ```
#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
pub struct TimeoutConfig {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    operation_timeout: Option<Duration>,
    operation_attempt_timeout: Option<Duration>,
}

impl Storable for TimeoutConfig {
    type Storer = StoreReplace<TimeoutConfig>;
}

impl TimeoutConfig {
    /// Returns a builder to create a `TimeoutConfig`.
    pub fn builder() -> TimeoutConfigBuilder {
        TimeoutConfigBuilder::new()
    }

    /// Returns a builder equivalent of this `TimeoutConfig`.
    pub fn to_builder(&self) -> TimeoutConfigBuilder {
        TimeoutConfigBuilder::from(self.clone())
    }

    /// Converts this `TimeoutConfig` into a builder.
    pub fn into_builder(self) -> TimeoutConfigBuilder {
        TimeoutConfigBuilder::from(self)
    }

    /// Returns a timeout config with all timeouts disabled.
    pub fn disabled() -> TimeoutConfig {
        TimeoutConfig {
            connect_timeout: None,
            read_timeout: None,
            operation_timeout: None,
            operation_attempt_timeout: None,
        }
    }

    /// Returns this config's connect timeout.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// Returns this config's read timeout.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(&self) -> Option<Duration> {
        self.read_timeout
    }

    /// Returns this config's operation timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// The operation timeout is a limit on the total amount of time it takes for an operation to be
    /// fully serviced, including the time for all retries that may have been attempted for it.
    pub fn operation_timeout(&self) -> Option<Duration> {
        self.operation_timeout
    }

    /// Returns this config's operation attempt timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// When retries are enabled, then this setting makes it possible to set a timeout for individual
    /// retry attempts (including the initial attempt) for an operation.
    pub fn operation_attempt_timeout(&self) -> Option<Duration> {
        self.operation_attempt_timeout
    }

    /// Returns true if any of the possible timeouts are set.
    pub fn has_timeouts(&self) -> bool {
        self.connect_timeout.is_some()
            || self.operation_timeout.is_some()
            || self.operation_attempt_timeout.is_some()
    }
}

/// Configuration subset of [`TimeoutConfig`] for operation timeouts
#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
pub struct OperationTimeoutConfig {
    operation_timeout: Option<Duration>,
    operation_attempt_timeout: Option<Duration>,
}

impl OperationTimeoutConfig {
    /// Returns this config's operation timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// The operation timeout is a limit on the total amount of time it takes for an operation to be
    /// fully serviced, including the time for all retries that may have been attempted for it.
    pub fn operation_timeout(&self) -> Option<Duration> {
        self.operation_timeout
    }

    /// Returns this config's operation attempt timeout.
    ///
    /// An operation represents the full request/response lifecycle of a call to a service.
    /// When retries are enabled, then this setting makes it possible to set a timeout for individual
    /// retry attempts (including the initial attempt) for an operation.
    pub fn operation_attempt_timeout(&self) -> Option<Duration> {
        self.operation_attempt_timeout
    }

    /// Returns true if any of the possible timeouts are set.
    pub fn has_timeouts(&self) -> bool {
        self.operation_timeout.is_some() || self.operation_attempt_timeout.is_some()
    }
}

impl From<&TimeoutConfig> for OperationTimeoutConfig {
    fn from(cfg: &TimeoutConfig) -> Self {
        OperationTimeoutConfig {
            operation_timeout: cfg.operation_timeout,
            operation_attempt_timeout: cfg.operation_attempt_timeout,
        }
    }
}

impl From<TimeoutConfig> for OperationTimeoutConfig {
    fn from(cfg: TimeoutConfig) -> Self {
        OperationTimeoutConfig::from(&cfg)
    }
}
