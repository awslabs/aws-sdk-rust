/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::retries::strategy::standard::ReleaseResult::{
    APermitWasReleased, NoPermitWasReleased,
};
use crate::client::runtime_plugin::standard_token_bucket::StandardTokenBucket;
use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors};
use aws_smithy_runtime_api::client::request_attempts::RequestAttempts;
use aws_smithy_runtime_api::client::retries::{
    ClassifyRetry, RetryReason, RetryStrategy, ShouldAttempt,
};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::retry::RetryConfig;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::OwnedSemaphorePermit;

// The initial attempt, plus three retries.
const DEFAULT_MAX_ATTEMPTS: usize = 4;

#[derive(Debug)]
pub struct StandardRetryStrategy {
    // Retry settings
    base: fn() -> f64,
    initial_backoff: Duration,
    max_attempts: usize,
    max_backoff: Duration,
    retry_permit: Mutex<Option<OwnedSemaphorePermit>>,
}

impl StandardRetryStrategy {
    pub fn new(retry_config: &RetryConfig) -> Self {
        // TODO(enableNewSmithyRuntimeLaunch) add support for `retry_config.reconnect_mode()` here or in the orchestrator flow.
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

    pub fn with_max_backoff(mut self, max_backoff: Duration) -> Self {
        self.max_backoff = max_backoff;
        self
    }

    fn release_retry_permit(&self) -> ReleaseResult {
        let mut retry_permit = self.retry_permit.lock().unwrap();
        match retry_permit.take() {
            Some(p) => {
                drop(p);
                APermitWasReleased
            }
            None => NoPermitWasReleased,
        }
    }

    fn set_retry_permit(&self, new_retry_permit: OwnedSemaphorePermit) {
        let mut old_retry_permit = self.retry_permit.lock().unwrap();
        if let Some(p) = old_retry_permit.replace(new_retry_permit) {
            // Whenever we set a new retry permit and it replaces the old one, we need to "forget"
            // the old permit, removing it from the bucket forever.
            p.forget()
        }
    }
}

enum ReleaseResult {
    APermitWasReleased,
    NoPermitWasReleased,
}

impl Default for StandardRetryStrategy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
            max_backoff: Duration::from_secs(20),
            // by default, use a random base for exponential backoff
            base: fastrand::f64,
            initial_backoff: Duration::from_secs(1),
            retry_permit: Mutex::new(None),
        }
    }
}

impl RetryStrategy for StandardRetryStrategy {
    fn should_attempt_initial_request(&self, _cfg: &ConfigBag) -> Result<ShouldAttempt, BoxError> {
        // The standard token bucket is only ever considered for retry requests.
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
        let token_bucket = cfg.get::<StandardTokenBucket>();
        if output_or_error.is_ok() {
            tracing::debug!("request succeeded, no retry necessary");
            if let Some(tb) = token_bucket {
                // If this retry strategy is holding any permits, release them back to the bucket.
                if let NoPermitWasReleased = self.release_retry_permit() {
                    // In the event that there was no retry permit to release, we generate new
                    // permits from nothing. We do this to make up for permits we had to "forget".
                    // Otherwise, repeated retries would empty the bucket and nothing could fill it
                    // back up again.
                    tb.regenerate_a_token();
                }
            }

            return Ok(ShouldAttempt::No);
        }

        // Check if we're out of attempts
        let request_attempts = cfg
            .get::<RequestAttempts>()
            .expect("at least one request attempt is made before any retry is attempted")
            .attempts();
        if request_attempts >= self.max_attempts {
            tracing::trace!(
                attempts = request_attempts,
                max_attempts = self.max_attempts,
                "not retrying because we are out of attempts"
            );
            return Ok(ShouldAttempt::No);
        }

        // Run the classifiers against the context to determine if we should retry
        let retry_classifiers = cfg.retry_classifiers();
        let retry_reason = retry_classifiers.classify_retry(ctx);

        // Calculate the appropriate backoff time.
        let backoff = match retry_reason {
            Some(RetryReason::Explicit(dur)) => dur,
            Some(RetryReason::Error(kind)) => {
                // If a token bucket was set, and the RetryReason IS NOT explicit, attempt to acquire a retry permit.
                if let Some(tb) = token_bucket {
                    match tb.acquire(&kind) {
                        Some(permit) => self.set_retry_permit(permit),
                        None => {
                            tracing::debug!(
                        "attempt #{request_attempts} failed with {kind:?}; However, no retry permits are available, so no retry will be attempted.",
                    );
                            return Ok(ShouldAttempt::No);
                        }
                    }
                };

                let backoff = calculate_exponential_backoff(
                    // Generate a random base multiplier to create jitter
                    (self.base)(),
                    // Get the backoff time multiplier in seconds (with fractional seconds)
                    self.initial_backoff.as_secs_f64(),
                    // `self.local.attempts` tracks number of requests made including the initial request
                    // The initial attempt shouldn't count towards backoff calculations so we subtract it
                    (request_attempts - 1) as u32,
                );
                Duration::from_secs_f64(backoff).min(self.max_backoff)
            }
            Some(_) => unreachable!("RetryReason is non-exhaustive"),
            None => {
                tracing::trace!(
                    attempts = request_attempts,
                    max_attempts = self.max_attempts,
                    "encountered unretryable error"
                );
                return Ok(ShouldAttempt::No);
            }
        };

        tracing::debug!(
            "attempt #{request_attempts} failed with {:?}; retrying after {:?}",
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
    use super::{calculate_exponential_backoff, ShouldAttempt, StandardRetryStrategy};
    use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
    use aws_smithy_runtime_api::client::orchestrator::{ConfigBagAccessors, OrchestratorError};
    use aws_smithy_runtime_api::client::request_attempts::RequestAttempts;
    use aws_smithy_runtime_api::client::retries::{
        AlwaysRetry, ClassifyRetry, RetryClassifiers, RetryReason, RetryStrategy,
    };
    use aws_smithy_types::config_bag::{ConfigBag, Layer};
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
    use aws_smithy_types::type_erasure::TypeErasedBox;
    use std::fmt;
    use std::sync::Mutex;
    use std::time::Duration;

    #[cfg(feature = "test-util")]
    use crate::client::runtime_plugin::standard_token_bucket::StandardTokenBucket;

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
        let mut layer = Layer::new("test");
        layer.set_retry_classifiers(
            RetryClassifiers::new().with_classifier(AlwaysRetry(error_kind)),
        );
        layer.put(RequestAttempts::new(current_request_attempts));
        let cfg = ConfigBag::of_layers(vec![layer]);

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

    #[derive(Debug)]
    struct ServerError;
    impl fmt::Display for ServerError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "OperationError")
        }
    }

    impl std::error::Error for ServerError {}

    impl ProvideErrorKind for ServerError {
        fn retryable_error_kind(&self) -> Option<ErrorKind> {
            Some(ErrorKind::ServerError)
        }

        fn code(&self) -> Option<&str> {
            None
        }
    }

    #[derive(Debug)]
    struct PresetReasonRetryClassifier {
        retry_reasons: Mutex<Vec<RetryReason>>,
    }

    #[cfg(feature = "test-util")]
    impl PresetReasonRetryClassifier {
        fn new(mut retry_reasons: Vec<RetryReason>) -> Self {
            // We'll pop the retry_reasons in reverse order so we reverse the list to fix that.
            retry_reasons.reverse();
            Self {
                retry_reasons: Mutex::new(retry_reasons),
            }
        }
    }

    impl ClassifyRetry for PresetReasonRetryClassifier {
        fn classify_retry(&self, ctx: &InterceptorContext) -> Option<RetryReason> {
            if ctx.output_or_error().map(|it| it.is_ok()).unwrap_or(false) {
                return None;
            }

            let mut retry_reasons = self.retry_reasons.lock().unwrap();
            if retry_reasons.len() == 1 {
                Some(retry_reasons.first().unwrap().clone())
            } else {
                retry_reasons.pop()
            }
        }

        fn name(&self) -> &'static str {
            "Always returns a preset retry reason"
        }
    }

    #[cfg(feature = "test-util")]
    fn setup_test(retry_reasons: Vec<RetryReason>) -> (ConfigBag, InterceptorContext) {
        let mut cfg = ConfigBag::base();
        cfg.interceptor_state().set_retry_classifiers(
            RetryClassifiers::new()
                .with_classifier(PresetReasonRetryClassifier::new(retry_reasons)),
        );
        let mut ctx = InterceptorContext::new(TypeErasedBox::doesnt_matter());
        // This type doesn't matter b/c the classifier will just return whatever we tell it to.
        ctx.set_output_or_error(Err(OrchestratorError::other("doesn't matter")));

        (cfg, ctx)
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn eventual_success() {
        let (mut cfg, mut ctx) = setup_test(vec![RetryReason::Error(ErrorKind::ServerError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(5);
        cfg.interceptor_state().put(StandardTokenBucket::default());
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        ctx.set_output_or_error(Ok(TypeErasedBox::doesnt_matter()));

        cfg.interceptor_state().put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 495);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn no_more_attempts() {
        let (mut cfg, ctx) = setup_test(vec![RetryReason::Error(ErrorKind::ServerError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(3);
        cfg.interceptor_state().put(StandardTokenBucket::default());
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 490);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn no_quota() {
        let (mut cfg, ctx) = setup_test(vec![RetryReason::Error(ErrorKind::ServerError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(5);
        cfg.interceptor_state().put(StandardTokenBucket::new(5));
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 0);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 0);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn quota_replenishes_on_success() {
        let (mut cfg, mut ctx) = setup_test(vec![
            RetryReason::Error(ErrorKind::TransientError),
            RetryReason::Explicit(Duration::from_secs(1)),
        ]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(5);
        cfg.interceptor_state().put(StandardTokenBucket::new(100));
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 90);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 90);

        ctx.set_output_or_error(Ok(TypeErasedBox::doesnt_matter()));

        cfg.interceptor_state().put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);

        assert_eq!(token_bucket.available_permits(), 100);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn quota_replenishes_on_first_try_success() {
        const PERMIT_COUNT: usize = 20;
        let (mut cfg, mut ctx) = setup_test(vec![RetryReason::Error(ErrorKind::TransientError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(usize::MAX);
        cfg.interceptor_state()
            .put(StandardTokenBucket::new(PERMIT_COUNT));
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        let mut attempt = 1;

        // Drain all available permits with failed attempts
        while token_bucket.available_permits() > 0 {
            // Draining should complete in 2 attempts
            if attempt > 2 {
                panic!("This test should have completed by now (drain)");
            }

            cfg.interceptor_state().put(RequestAttempts::new(attempt));
            let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
            assert!(matches!(should_retry, ShouldAttempt::YesAfterDelay(_)));
            attempt += 1;
        }

        // Forget the permit so that we can only refill by "success on first try".
        let permit = strategy.retry_permit.lock().unwrap().take().unwrap();
        permit.forget();

        ctx.set_output_or_error(Ok(TypeErasedBox::doesnt_matter()));

        // Replenish permits until we get back to `PERMIT_COUNT`
        while token_bucket.available_permits() < PERMIT_COUNT {
            if attempt > 23 {
                panic!("This test should have completed by now (fillup)");
            }

            cfg.interceptor_state().put(RequestAttempts::new(attempt));
            let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
            assert_eq!(no_retry, ShouldAttempt::No);
            attempt += 1;
        }

        assert_eq!(attempt, 23);
        assert_eq!(token_bucket.available_permits(), PERMIT_COUNT);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn backoff_timing() {
        let (mut cfg, ctx) = setup_test(vec![RetryReason::Error(ErrorKind::ServerError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(5);
        cfg.interceptor_state().put(StandardTokenBucket::default());
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().put(RequestAttempts::new(3));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(4));
        assert_eq!(token_bucket.available_permits(), 485);

        cfg.interceptor_state().put(RequestAttempts::new(4));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(8));
        assert_eq!(token_bucket.available_permits(), 480);

        cfg.interceptor_state().put(RequestAttempts::new(5));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 480);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn max_backoff_time() {
        let (mut cfg, ctx) = setup_test(vec![RetryReason::Error(ErrorKind::ServerError)]);
        let strategy = StandardRetryStrategy::default()
            .with_base(|| 1.0)
            .with_max_attempts(5)
            .with_initial_backoff(Duration::from_secs(1))
            .with_max_backoff(Duration::from_secs(3));
        cfg.interceptor_state().put(StandardTokenBucket::default());
        let token_bucket = cfg.get::<StandardTokenBucket>().unwrap().clone();

        cfg.interceptor_state().put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().put(RequestAttempts::new(3));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(token_bucket.available_permits(), 485);

        cfg.interceptor_state().put(RequestAttempts::new(4));
        let should_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(token_bucket.available_permits(), 480);

        cfg.interceptor_state().put(RequestAttempts::new(5));
        let no_retry = strategy.should_attempt_retry(&ctx, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 480);
    }

    #[test]
    fn calculate_exponential_backoff_where_initial_backoff_is_one() {
        let initial_backoff = 1.0;

        for (attempt, expected_backoff) in [initial_backoff, 2.0, 4.0].into_iter().enumerate() {
            let actual_backoff =
                calculate_exponential_backoff(1.0, initial_backoff, attempt as u32);
            assert_eq!(expected_backoff, actual_backoff);
        }
    }

    #[test]
    fn calculate_exponential_backoff_where_initial_backoff_is_greater_than_one() {
        let initial_backoff = 3.0;

        for (attempt, expected_backoff) in [initial_backoff, 6.0, 12.0].into_iter().enumerate() {
            let actual_backoff =
                calculate_exponential_backoff(1.0, initial_backoff, attempt as u32);
            assert_eq!(expected_backoff, actual_backoff);
        }
    }

    #[test]
    fn calculate_exponential_backoff_where_initial_backoff_is_less_than_one() {
        let initial_backoff = 0.03;

        for (attempt, expected_backoff) in [initial_backoff, 0.06, 0.12].into_iter().enumerate() {
            let actual_backoff =
                calculate_exponential_backoff(1.0, initial_backoff, attempt as u32);
            assert_eq!(expected_backoff, actual_backoff);
        }
    }
}
