/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::retries::classifiers::run_classifiers_on_ctx;
use crate::client::retries::client_rate_limiter::{ClientRateLimiter, RequestReason};
use crate::client::retries::strategy::standard::ReleaseResult::{
    APermitWasReleased, NoPermitWasReleased,
};
use crate::client::retries::token_bucket::TokenBucket;
use crate::client::retries::{ClientRateLimiterPartition, RetryPartition};
use crate::static_partition_map::StaticPartitionMap;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
use aws_smithy_runtime_api::client::retries::classifiers::{RetryAction, RetryReason};
use aws_smithy_runtime_api::client::retries::{RequestAttempts, RetryStrategy, ShouldAttempt};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::retry::{ErrorKind, RetryConfig, RetryMode};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tokio::sync::OwnedSemaphorePermit;
use tracing::debug;

static CLIENT_RATE_LIMITER: StaticPartitionMap<ClientRateLimiterPartition, ClientRateLimiter> =
    StaticPartitionMap::new();

/// Retry strategy with exponential backoff, max attempts, and a token bucket.
#[derive(Debug, Default)]
pub struct StandardRetryStrategy {
    retry_permit: Mutex<Option<OwnedSemaphorePermit>>,
}

impl Storable for StandardRetryStrategy {
    type Storer = StoreReplace<Self>;
}

impl StandardRetryStrategy {
    /// Create a new standard retry strategy with the given config.
    pub fn new() -> Self {
        Default::default()
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

    /// Returns a [`ClientRateLimiter`] if adaptive retry is configured.
    fn adaptive_retry_rate_limiter(
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Option<ClientRateLimiter> {
        let retry_config = cfg.load::<RetryConfig>().expect("retry config is required");
        if retry_config.mode() == RetryMode::Adaptive {
            if let Some(time_source) = runtime_components.time_source() {
                let retry_partition = cfg.load::<RetryPartition>().expect("set in default config");
                let seconds_since_unix_epoch = time_source
                    .now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .expect("the present takes place after the UNIX_EPOCH")
                    .as_secs_f64();
                let client_rate_limiter_partition =
                    ClientRateLimiterPartition::new(retry_partition.clone());
                let client_rate_limiter = CLIENT_RATE_LIMITER
                    .get_or_init(client_rate_limiter_partition, || {
                        ClientRateLimiter::new(seconds_since_unix_epoch)
                    });
                return Some(client_rate_limiter);
            }
        }
        None
    }

    fn calculate_backoff(
        &self,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
        retry_cfg: &RetryConfig,
        retry_reason: &RetryAction,
    ) -> Result<Duration, ShouldAttempt> {
        let request_attempts = cfg
            .load::<RequestAttempts>()
            .expect("at least one request attempt is made before any retry is attempted")
            .attempts();
        let token_bucket = cfg.load::<TokenBucket>();

        match retry_reason {
            RetryAction::RetryIndicated(RetryReason::RetryableError { kind, retry_after }) => {
                update_rate_limiter_if_exists(
                    runtime_components,
                    cfg,
                    *kind == ErrorKind::ThrottlingError,
                );

                if let Some(delay) = *retry_after {
                    let delay = delay.min(retry_cfg.max_backoff());
                    debug!("explicit request from server to delay {delay:?} before retrying");
                    Ok(delay)
                } else if let Some(delay) =
                    check_rate_limiter_for_delay(runtime_components, cfg, *kind)
                {
                    let delay = delay.min(retry_cfg.max_backoff());
                    debug!("rate limiter has requested a {delay:?} delay before retrying");
                    Ok(delay)
                } else {
                    if let Some(tb) = token_bucket {
                        match tb.acquire(kind) {
                            Some(permit) => self.set_retry_permit(permit),
                            None => {
                                debug!("attempt #{request_attempts} failed with {kind:?}; However, no retry permits are available, so no retry will be attempted.");
                                return Err(ShouldAttempt::No);
                            }
                        }
                    }

                    let base = if retry_cfg.use_static_exponential_base() {
                        1.0
                    } else {
                        fastrand::f64()
                    };
                    let backoff = calculate_exponential_backoff(
                        // Generate a random base multiplier to create jitter
                        base,
                        // Get the backoff time multiplier in seconds (with fractional seconds)
                        retry_cfg.initial_backoff().as_secs_f64(),
                        // `self.local.attempts` tracks number of requests made including the initial request
                        // The initial attempt shouldn't count towards backoff calculations so we subtract it
                        request_attempts - 1,
                    );
                    Ok(Duration::from_secs_f64(backoff).min(retry_cfg.max_backoff()))
                }
            }
            RetryAction::RetryForbidden | RetryAction::NoActionIndicated => {
                update_rate_limiter_if_exists(runtime_components, cfg, false);
                debug!(
                    attempts = request_attempts,
                    max_attempts = retry_cfg.max_attempts(),
                    "encountered unretryable error"
                );
                Err(ShouldAttempt::No)
            }
            _ => unreachable!("RetryAction is non-exhaustive"),
        }
    }
}

enum ReleaseResult {
    APermitWasReleased,
    NoPermitWasReleased,
}

impl RetryStrategy for StandardRetryStrategy {
    fn should_attempt_initial_request(
        &self,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        if let Some(crl) = Self::adaptive_retry_rate_limiter(runtime_components, cfg) {
            let seconds_since_unix_epoch = get_seconds_since_unix_epoch(runtime_components);
            if let Err(delay) = crl.acquire_permission_to_send_a_request(
                seconds_since_unix_epoch,
                RequestReason::InitialRequest,
            ) {
                return Ok(ShouldAttempt::YesAfterDelay(delay));
            }
        } else {
            debug!("no client rate limiter configured, so no token is required for the initial request.");
        }

        Ok(ShouldAttempt::Yes)
    }

    fn should_attempt_retry(
        &self,
        ctx: &InterceptorContext,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        let retry_cfg = cfg.load::<RetryConfig>().expect("retry config is required");
        // Look a the result. If it's OK then we're done; No retry required. Otherwise, we need to inspect it
        let output_or_error = ctx.output_or_error().expect(
            "This must never be called without reaching the point where the result exists.",
        );
        let token_bucket = cfg.load::<TokenBucket>();
        if output_or_error.is_ok() {
            debug!("request succeeded, no retry necessary");
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
            update_rate_limiter_if_exists(runtime_components, cfg, false);

            return Ok(ShouldAttempt::No);
        }

        // Check if we're out of attempts
        let request_attempts = cfg
            .load::<RequestAttempts>()
            .expect("at least one request attempt is made before any retry is attempted")
            .attempts();
        if request_attempts >= retry_cfg.max_attempts() {
            update_rate_limiter_if_exists(runtime_components, cfg, false);

            debug!(
                attempts = request_attempts,
                max_attempts = retry_cfg.max_attempts(),
                "not retrying because we are out of attempts"
            );
            return Ok(ShouldAttempt::No);
        }

        // Run the classifier against the context to determine if we should retry
        let retry_classifiers = runtime_components.retry_classifiers();
        let classifier_result = run_classifiers_on_ctx(retry_classifiers, ctx);

        // Calculate the appropriate backoff time.
        let backoff =
            match self.calculate_backoff(runtime_components, cfg, retry_cfg, &classifier_result) {
                Ok(value) => value,
                // In some cases, backoff calculation will decide that we shouldn't retry at all.
                Err(value) => return Ok(value),
            };
        debug!(
            "attempt #{request_attempts} failed with {:?}; retrying after {:?}",
            classifier_result, backoff,
        );

        Ok(ShouldAttempt::YesAfterDelay(backoff))
    }
}

fn update_rate_limiter_if_exists(
    runtime_components: &RuntimeComponents,
    cfg: &ConfigBag,
    is_throttling_error: bool,
) {
    if let Some(crl) = StandardRetryStrategy::adaptive_retry_rate_limiter(runtime_components, cfg) {
        let seconds_since_unix_epoch = get_seconds_since_unix_epoch(runtime_components);
        crl.update_rate_limiter(seconds_since_unix_epoch, is_throttling_error);
    }
}

fn check_rate_limiter_for_delay(
    runtime_components: &RuntimeComponents,
    cfg: &ConfigBag,
    kind: ErrorKind,
) -> Option<Duration> {
    if let Some(crl) = StandardRetryStrategy::adaptive_retry_rate_limiter(runtime_components, cfg) {
        let retry_reason = if kind == ErrorKind::ThrottlingError {
            RequestReason::RetryTimeout
        } else {
            RequestReason::Retry
        };
        if let Err(delay) = crl.acquire_permission_to_send_a_request(
            get_seconds_since_unix_epoch(runtime_components),
            retry_reason,
        ) {
            return Some(delay);
        }
    }

    None
}

fn calculate_exponential_backoff(base: f64, initial_backoff: f64, retry_attempts: u32) -> f64 {
    base * initial_backoff * 2_u32.pow(retry_attempts) as f64
}

fn get_seconds_since_unix_epoch(runtime_components: &RuntimeComponents) -> f64 {
    let request_time = runtime_components
        .time_source()
        .expect("time source required for retries");
    request_time
        .now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
    use aws_smithy_runtime_api::client::retries::classifiers::{
        ClassifyRetry, RetryAction, SharedRetryClassifier,
    };
    use aws_smithy_runtime_api::client::retries::{AlwaysRetry, RetryStrategy};
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use aws_smithy_types::config_bag::Layer;
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
    use std::fmt;
    use std::sync::Mutex;
    use std::time::Duration;

    #[cfg(feature = "test-util")]
    use crate::client::retries::token_bucket::TokenBucket;
    use aws_smithy_runtime_api::client::interceptors::context::{Input, Output};

    #[test]
    fn no_retry_necessary_for_ok_result() {
        let cfg = ConfigBag::of_layers(vec![{
            let mut layer = Layer::new("test");
            layer.store_put(RetryConfig::standard());
            layer
        }]);
        let rc = RuntimeComponentsBuilder::for_tests().build().unwrap();
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        let strategy = StandardRetryStrategy::default();
        ctx.set_output_or_error(Ok(Output::doesnt_matter()));
        let actual = strategy
            .should_attempt_retry(&ctx, &rc, &cfg)
            .expect("method is infallible for this use");
        assert_eq!(ShouldAttempt::No, actual);
    }

    fn set_up_cfg_and_context(
        error_kind: ErrorKind,
        current_request_attempts: u32,
        retry_config: RetryConfig,
    ) -> (InterceptorContext, RuntimeComponents, ConfigBag) {
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::other("doesn't matter")));
        let rc = RuntimeComponentsBuilder::for_tests()
            .with_retry_classifier(SharedRetryClassifier::new(AlwaysRetry(error_kind)))
            .build()
            .unwrap();
        let mut layer = Layer::new("test");
        layer.store_put(RequestAttempts::new(current_request_attempts));
        layer.store_put(retry_config);
        let cfg = ConfigBag::of_layers(vec![layer]);

        (ctx, rc, cfg)
    }

    // Test that error kinds produce the correct "retry after X seconds" output.
    // All error kinds are handled in the same way for the standard strategy.
    fn test_should_retry_error_kind(error_kind: ErrorKind) {
        let (ctx, rc, cfg) = set_up_cfg_and_context(
            error_kind,
            3,
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(4),
        );
        let strategy = StandardRetryStrategy::new();
        let actual = strategy
            .should_attempt_retry(&ctx, &rc, &cfg)
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
        let (ctx, rc, cfg) = set_up_cfg_and_context(
            ErrorKind::TransientError,
            current_attempts,
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(max_attempts),
        );
        let strategy = StandardRetryStrategy::new();
        let actual = strategy
            .should_attempt_retry(&ctx, &rc, &cfg)
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
        retry_actions: Mutex<Vec<RetryAction>>,
    }

    #[cfg(feature = "test-util")]
    impl PresetReasonRetryClassifier {
        fn new(mut retry_reasons: Vec<RetryAction>) -> Self {
            // We'll pop the retry_reasons in reverse order so we reverse the list to fix that.
            retry_reasons.reverse();
            Self {
                retry_actions: Mutex::new(retry_reasons),
            }
        }
    }

    impl ClassifyRetry for PresetReasonRetryClassifier {
        fn classify_retry(&self, ctx: &InterceptorContext) -> RetryAction {
            // Check for a result
            let output_or_error = ctx.output_or_error();
            // Check for an error
            match output_or_error {
                Some(Ok(_)) | None => return RetryAction::NoActionIndicated,
                _ => (),
            };

            let mut retry_actions = self.retry_actions.lock().unwrap();
            if retry_actions.len() == 1 {
                retry_actions.first().unwrap().clone()
            } else {
                retry_actions.pop().unwrap()
            }
        }

        fn name(&self) -> &'static str {
            "Always returns a preset retry reason"
        }
    }

    #[cfg(feature = "test-util")]
    fn setup_test(
        retry_reasons: Vec<RetryAction>,
        retry_config: RetryConfig,
    ) -> (ConfigBag, RuntimeComponents, InterceptorContext) {
        let rc = RuntimeComponentsBuilder::for_tests()
            .with_retry_classifier(SharedRetryClassifier::new(
                PresetReasonRetryClassifier::new(retry_reasons),
            ))
            .build()
            .unwrap();
        let mut layer = Layer::new("test");
        layer.store_put(retry_config);
        let cfg = ConfigBag::of_layers(vec![layer]);
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        // This type doesn't matter b/c the classifier will just return whatever we tell it to.
        ctx.set_output_or_error(Err(OrchestratorError::other("doesn't matter")));

        (cfg, rc, ctx)
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn eventual_success() {
        let (mut cfg, rc, mut ctx) = setup_test(
            vec![RetryAction::server_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(5),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::default());
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        ctx.set_output_or_error(Ok(Output::doesnt_matter()));

        cfg.interceptor_state().store_put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 495);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn no_more_attempts() {
        let (mut cfg, rc, ctx) = setup_test(
            vec![RetryAction::server_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(3),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::default());
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().store_put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 490);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn no_quota() {
        let (mut cfg, rc, ctx) = setup_test(
            vec![RetryAction::server_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(5),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::new(5));
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 0);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 0);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn quota_replenishes_on_success() {
        let (mut cfg, rc, mut ctx) = setup_test(
            vec![
                RetryAction::transient_error(),
                RetryAction::retryable_error_with_explicit_delay(
                    ErrorKind::TransientError,
                    Duration::from_secs(1),
                ),
            ],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(5),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::new(100));
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 90);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 90);

        ctx.set_output_or_error(Ok(Output::doesnt_matter()));

        cfg.interceptor_state().store_put(RequestAttempts::new(3));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);

        assert_eq!(token_bucket.available_permits(), 100);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn quota_replenishes_on_first_try_success() {
        const PERMIT_COUNT: usize = 20;
        let (mut cfg, rc, mut ctx) = setup_test(
            vec![RetryAction::transient_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(u32::MAX),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state()
            .store_put(TokenBucket::new(PERMIT_COUNT));
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        let mut attempt = 1;

        // Drain all available permits with failed attempts
        while token_bucket.available_permits() > 0 {
            // Draining should complete in 2 attempts
            if attempt > 2 {
                panic!("This test should have completed by now (drain)");
            }

            cfg.interceptor_state()
                .store_put(RequestAttempts::new(attempt));
            let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
            assert!(matches!(should_retry, ShouldAttempt::YesAfterDelay(_)));
            attempt += 1;
        }

        // Forget the permit so that we can only refill by "success on first try".
        let permit = strategy.retry_permit.lock().unwrap().take().unwrap();
        permit.forget();

        ctx.set_output_or_error(Ok(Output::doesnt_matter()));

        // Replenish permits until we get back to `PERMIT_COUNT`
        while token_bucket.available_permits() < PERMIT_COUNT {
            if attempt > 23 {
                panic!("This test should have completed by now (fill-up)");
            }

            cfg.interceptor_state()
                .store_put(RequestAttempts::new(attempt));
            let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
            assert_eq!(no_retry, ShouldAttempt::No);
            attempt += 1;
        }

        assert_eq!(attempt, 23);
        assert_eq!(token_bucket.available_permits(), PERMIT_COUNT);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn backoff_timing() {
        let (mut cfg, rc, ctx) = setup_test(
            vec![RetryAction::server_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(5),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::default());
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().store_put(RequestAttempts::new(3));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(4));
        assert_eq!(token_bucket.available_permits(), 485);

        cfg.interceptor_state().store_put(RequestAttempts::new(4));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(8));
        assert_eq!(token_bucket.available_permits(), 480);

        cfg.interceptor_state().store_put(RequestAttempts::new(5));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        assert_eq!(no_retry, ShouldAttempt::No);
        assert_eq!(token_bucket.available_permits(), 480);
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn max_backoff_time() {
        let (mut cfg, rc, ctx) = setup_test(
            vec![RetryAction::server_error()],
            RetryConfig::standard()
                .with_use_static_exponential_base(true)
                .with_max_attempts(5)
                .with_initial_backoff(Duration::from_secs(1))
                .with_max_backoff(Duration::from_secs(3)),
        );
        let strategy = StandardRetryStrategy::new();
        cfg.interceptor_state().store_put(TokenBucket::default());
        let token_bucket = cfg.load::<TokenBucket>().unwrap().clone();

        cfg.interceptor_state().store_put(RequestAttempts::new(1));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(token_bucket.available_permits(), 495);

        cfg.interceptor_state().store_put(RequestAttempts::new(2));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(token_bucket.available_permits(), 490);

        cfg.interceptor_state().store_put(RequestAttempts::new(3));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(token_bucket.available_permits(), 485);

        cfg.interceptor_state().store_put(RequestAttempts::new(4));
        let should_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
        let dur = should_retry.expect_delay();
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(token_bucket.available_permits(), 480);

        cfg.interceptor_state().store_put(RequestAttempts::new(5));
        let no_retry = strategy.should_attempt_retry(&ctx, &rc, &cfg).unwrap();
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
