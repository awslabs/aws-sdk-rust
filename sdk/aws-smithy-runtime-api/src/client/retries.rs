/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::interceptors::InterceptorContext;
use crate::client::orchestrator::{BoxError, RetryStrategy};
use crate::config_bag::ConfigBag;
use aws_smithy_http::body::SdkBody;

pub mod rate_limiting;

#[derive(Debug, Clone)]
pub struct NeverRetryStrategy {}

impl NeverRetryStrategy {
    pub fn new() -> Self {
        Self {}
    }
}

impl RetryStrategy for NeverRetryStrategy {
    fn should_attempt_initial_request(&self, _cfg: &ConfigBag) -> Result<(), BoxError> {
        Ok(())
    }

    fn should_attempt_retry(
        &self,
        _context: &InterceptorContext<http::Request<SdkBody>, http::Response<SdkBody>>,
        _cfg: &ConfigBag,
    ) -> Result<bool, BoxError> {
        Ok(false)
    }
}
