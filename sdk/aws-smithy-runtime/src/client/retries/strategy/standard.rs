/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors};
use aws_smithy_runtime_api::client::request_attempts::RequestAttempts;
use aws_smithy_runtime_api::client::retries::{
    ClassifyRetry, RetryReason, RetryStrategy, ShouldAttempt,
};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::retry::RetryConfig;
use std::time::Duration;

const DEFAULT_MAX_ATTEMPTS: usize = 4;

#[derive(Debug)]
pub struct StandardRetryStrategy {
    max_attempts: usize,
    initial_backoff: Duration,
    max_backoff: Duration,
    base: fn() -> f64,
}

impl StandardRetryStrategy {
    pub fn new(retry_config: &RetryConfig) -> Self {
        // TODO(enableNewSmithyRuntime) add support for `retry_config.reconnect_mode()` here or in the orchestrator flow.
        Self::default()
            .with_max_attempts(retry_config.max_attempts() as usize)
            .with_initial_backoff(retry_config.initial_backoff())
    }

    pub fn with_base(mut self, base: fn() -> f64) -> Self {
        self.base = base;
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn with_initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }
}

impl Default for StandardRetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            max_backoff: Duration::from_secs(20),
            // by default, use a random base for exponential backoff
            base: fastrand::f64,
            initial_backoff: Duration::from_secs(1),
        }
    }
}

impl RetryStrategy for StandardRetryStrategy {
    // TODO(token-bucket) add support for optional cross-request token bucket
    fn should_attempt_initial_request(&self, _cfg: &ConfigBag) -> Result<ShouldAttempt, BoxError> {
        Ok(ShouldAttempt::Yes)
    }

    fn should_attempt_retry(
        &self,
        ctx: &InterceptorContext,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        // Look a the result. If it's OK then we're done; No retry required. Otherwise, we need to inspect it
        let output_or_error = ctx.output_or_error().expect(
            "This must never be called without reaching the point where the result exists.",
        );
        if output_or_error.is_ok() {
            tracing::debug!("request succeeded, no retry necessary");
            return Ok(ShouldAttempt::No);
        }

        // Check if we're out of attempts
        let request_attempts: &RequestAttempts = cfg
            .get()
            .expect("at least one request attempt is made before any retry is attempted");
        if request_attempts.attempts() >= self.max_attempts {
            tracing::trace!(
                attempts = request_attempts.attempts(),
                max_attempts = self.max_attempts,
                "not retrying because we are out of attempts"
            );
            return Ok(ShouldAttempt::No);
        }

        // Run the classifiers against the context to determine if we should retry
        let retry_classifiers = cfg.retry_classifiers();
        let retry_reason = retry_classifiers.classify_retry(ctx);
        let backoff = match retry_reason {
            Some(RetryReason::Explicit(dur)) => dur,
            Some(RetryReason::Error(_)) => {
                let backoff = calculate_exponential_backoff(
                    // Generate a random base multiplier to create jitter
                    (self.base)(),
                    // Get the backoff time multiplier in seconds (with fractional seconds)
                    self.initial_backoff.as_secs_f64(),
                    // `self.local.attempts` tracks number of requests made including the initial request
                    // The initial attempt shouldn't count towards backoff calculations so we subtract it
                    (request_attempts.attempts() - 1) as u32,
                );
                Duration::from_secs_f64(backoff).min(self.max_backoff)
            }
            Some(_) => {
                unreachable!("RetryReason is non-exhaustive. Therefore, we need to cover this unreachable case.")
            }
            None => {
                tracing::trace!(
                    attempts = request_attempts.attempts(),
                    max_attempts = self.max_attempts,
                    "encountered unretryable error"
                );
                return Ok(ShouldAttempt::No);
            }
        };

        tracing::debug!(
            "attempt {} failed with {:?}; retrying after {:?}",
            request_attempts.attempts(),
            retry_reason.expect("the match statement above ensures this is not None"),
            backoff
        );

        Ok(ShouldAttempt::YesAfterDelay(backoff))
    }
}

fn calculate_exponential_backoff(base: f64, initial_backoff: f64, retry_attempts: u32) -> f64 {
    base * initial_backoff * 2_u32.pow(retry_attempts) as f64
}

#[cfg(test)]
mod tests {
    use super::{ShouldAttempt, StandardRetryStrategy};
    use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
    use aws_smithy_runtime_api::client::orchestrator::{ConfigBagAccessors, OrchestratorError};
    use aws_smithy_runtime_api::client::request_attempts::RequestAttempts;
    use aws_smithy_runtime_api::client::retries::{AlwaysRetry, RetryClassifiers, RetryStrategy};
    use aws_smithy_types::config_bag::ConfigBag;
    use aws_smithy_types::retry::ErrorKind;
    use aws_smithy_types::type_erasure::TypeErasedBox;
    use std::time::Duration;

    #[test]
    fn no_retry_necessary_for_ok_result() {
        let cfg = ConfigBag::base();
        let mut ctx = InterceptorContext::new(TypeErasedBox::doesnt_matter());
        let strategy = StandardRetryStrategy::default();
        ctx.set_output_or_error(Ok(TypeErasedBox::doesnt_matter()));
        let actual = strategy
            .should_attempt_retry(&ctx, &cfg)
            .expect("method is infallible for this use");
        assert_eq!(ShouldAttempt::No, actual);
    }

    fn set_up_cfg_and_context(
        error_kind: ErrorKind,
        current_request_attempts: usize,
    ) -> (InterceptorContext, ConfigBag) {
        let mut ctx = InterceptorContext::new(TypeErasedBox::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::other("doesn't matter")));
        let mut cfg = ConfigBag::base();
        cfg.set_retry_classifiers(RetryClassifiers::new().with_classifier(AlwaysRetry(error_kind)));
        cfg.put(RequestAttempts::new(current_request_attempts));

        (ctx, cfg)
    }

    // Test that error kinds produce the correct "retry after X seconds" output.
    // All error kinds are handled in the same way for the standard strategy.
    fn test_should_retry_error_kind(error_kind: ErrorKind) {
        let (ctx, cfg) = set_up_cfg_and_context(error_kind, 3);
        let strategy = StandardRetryStrategy::default().with_base(|| 1.0);
        let actual = strategy
            .should_attempt_retry(&ctx, &cfg)
            .expect("method is infallible for this use");
        assert_eq!(ShouldAttempt::YesAfterDelay(Duration::from_secs(4)), actual);
    }

    #[test]
    fn should_retry_transient_error_result_after_2s() {
        test_should_retry_error_kind(ErrorKind::TransientError);
    }

    #[test]
    fn should_retry_client_error_result_after_2s() {
        test_should_retry_error_kind(ErrorKind::ClientError);
    }

    #[test]
    fn should_retry_server_error_result_after_2s() {
        test_should_retry_error_kind(ErrorKind::ServerError);
    }

    #[test]
    fn should_retry_throttling_error_result_after_2s() {
        test_should_retry_error_kind(ErrorKind::ThrottlingError);
    }

    #[test]
    fn dont_retry_when_out_of_attempts() {
        let current_attempts = 4;
        let max_attempts = current_attempts;
        let (ctx, cfg) = set_up_cfg_and_context(ErrorKind::TransientError, current_attempts);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(max_attempts);
        let actual = strategy
            .should_attempt_retry(&ctx, &cfg)
            .expect("method is infallible for this use");
        assert_eq!(ShouldAttempt::No, actual);
    }
}
