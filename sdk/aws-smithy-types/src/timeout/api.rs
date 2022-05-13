/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::tristate::TriState;
use std::time::Duration;

/// API timeouts used by Smithy `Client`s
#[non_exhaustive]
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Api {
    /// A limit on the amount of time it takes for the first byte to be sent over an established,
    /// open connection and when the last byte is received from the service for a single attempt.
    /// If you want to set a timeout for an entire request including retry attempts,
    /// use [`Api::call_attempt`] instead.
    call: TriState<Duration>,
    /// A limit on the amount of time it takes for request to complete. A single request may be
    /// comprised of several attempts depending on an app's [`RetryConfig`](crate::retry::RetryConfig). If you want
    /// to control timeouts for a single attempt, use [`Api::call`].
    call_attempt: TriState<Duration>,
}

impl Api {
    /// Create a new API timeout config with no timeouts set
    pub fn new() -> Self {
        Default::default()
    }

    /// Return this config's `call` timeout
    pub fn call_timeout(&self) -> TriState<Duration> {
        self.call.clone()
    }

    /// Mutate this `timeout::Api` config, setting the API call timeout
    pub fn with_call_timeout(mut self, timeout: TriState<Duration>) -> Self {
        self.call = timeout;
        self
    }

    /// Return this config's `call_attempt` timeout
    pub fn call_attempt_timeout(&self) -> TriState<Duration> {
        self.call_attempt.clone()
    }

    /// Mutate this `timeout::Api` config, setting the API call single attempt timeout
    pub fn with_call_attempt_timeout(mut self, timeout: TriState<Duration>) -> Self {
        self.call_attempt = timeout;
        self
    }

    /// Return true if any timeouts are intentionally set or disabled
    pub fn has_timeouts(&self) -> bool {
        !self.is_unset()
    }

    /// Return true if all timeouts are unset
    fn is_unset(&self) -> bool {
        self.call.is_unset() && self.call_attempt.is_unset()
    }

    /// Merges two API timeout configs together.
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            call: self.call.or(other.call),
            call_attempt: self.call_attempt.or(other.call_attempt),
        }
    }
}

impl From<super::Config> for Api {
    fn from(config: super::Config) -> Self {
        config.api
    }
}
