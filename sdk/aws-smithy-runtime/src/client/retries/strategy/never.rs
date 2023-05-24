/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::BoxError;
use aws_smithy_runtime_api::client::retries::{RetryStrategy, ShouldAttempt};
use aws_smithy_runtime_api::config_bag::ConfigBag;

#[derive(Debug, Clone, Default)]
pub struct NeverRetryStrategy {}

impl NeverRetryStrategy {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RetryStrategy for NeverRetryStrategy {
    fn should_attempt_initial_request(&self, _cfg: &ConfigBag) -> Result<ShouldAttempt, BoxError> {
        Ok(ShouldAttempt::Yes)
    }

    fn should_attempt_retry(
        &self,
        _context: &InterceptorContext,
        _cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        Ok(ShouldAttempt::No)
    }
}
