/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Retry support
//!
//! Components:
//! - [`Standard`]: Top level manager, intended to be associated with a [`Client`](crate::Client).
//!   Its sole purpose in life is to create a [`RetryHandler`] for individual requests.
//! - [`RetryHandler`]: A request-scoped retry policy, backed by request-local state and shared
//!   state contained within [`Standard`].
//! - [`Config`]: Static configuration (max attempts, max backoff etc.)

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tracing::Instrument;

use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep};

use aws_smithy_http::operation::Operation;
use aws_smithy_http::retry::ClassifyRetry;
use aws_smithy_types::retry::{ErrorKind, RetryKind};

use crate::{SdkError, SdkSuccess};

/// A policy instantiator.
///
/// Implementors are essentially "policy factories" that can produce a new instance of a retry
/// policy mechanism for each request, which allows both shared global state _and_ per-request
/// local state.
pub trait NewRequestPolicy
where
    Self::Policy: Send + Sync,
{
    /// The type of the per-request policy mechanism.
    type Policy;

    /// Create a new policy mechanism instance.
    fn new_request_policy(&self, sleep_impl: Option<SharedAsyncSleep>) -> Self::Policy;
}

/// Retry Policy Configuration
///
/// Without specific use cases, users should generally rely on the default values set
/// by [`Config::default`](Config::default).
///
/// Currently these fields are private and no setters provided. As needed, this configuration
/// will become user-modifiable in the future.
#[derive(Clone, Debug)]
pub struct Config {
    initial_retry_tokens: usize,
    retry_cost: usize,
    no_retry_increment: usize,
    timeout_retry_cost: usize,
    max_attempts: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
    base: fn() -> f64,
}

impl Config {
    /// Override `b` in the exponential backoff computation
    ///
    /// By default, `base` is a randomly generated value between 0 and 1. In tests, it can
    /// be helpful to override this:
    /// ```no_run
    /// use aws_smithy_client::retry::Config;
    /// let conf = Config::default().with_base(||1_f64);
    /// ```
    pub fn with_base(mut self, base: fn() -> f64) -> Self {
        self.base = base;
        self
    }

    /// Override the maximum number of attempts
    ///
    /// `max_attempts` must be set to a value of at least `1` (indicating that retries are disabled).
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Override the default backoff multiplier of 1 second.
    ///
    /// ## Example
    ///
    /// For a request that gets retried 3 times, when initial_backoff is 1 second:
    /// - the first retry will occur after 0 to 1 seconds
    /// - the second retry will occur after 0 to 2 seconds
    /// - the third retry will occur after 0 to 4 seconds
    ///
    /// For a request that gets retried 3 times, when initial_backoff is 30 milliseconds:
    /// - the first retry will occur after 0 to 30 milliseconds
    /// - the second retry will occur after 0 to 60 milliseconds
    /// - the third retry will occur after 0 to 120 milliseconds
    pub fn with_initial_backoff(mut self, initial_backoff: Duration) -> Self {
        self.initial_backoff = initial_backoff;
        self
    }

    /// Returns true if retry is enabled with this config
    pub fn has_retry(&self) -> bool {
        self.max_attempts > 1
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            initial_retry_tokens: INITIAL_RETRY_TOKENS,
            retry_cost: RETRY_COST,
            no_retry_increment: 1,
            timeout_retry_cost: 10,
            max_attempts: MAX_ATTEMPTS,
            max_backoff: Duration::from_secs(20),
            // by default, use a random base for exponential backoff
            base: fastrand::f64,
            initial_backoff: Duration::from_secs(1),
        }
    }
}

impl From<aws_smithy_types::retry::RetryConfig> for Config {
    fn from(conf: aws_smithy_types::retry::RetryConfig) -> Self {
        Self::default()
            .with_max_attempts(conf.max_attempts())
            .with_initial_backoff(conf.initial_backoff())
    }
}

const MAX_ATTEMPTS: u32 = 3;
const INITIAL_RETRY_TOKENS: usize = 500;
const RETRY_COST: usize = 5;

/// Manage retries for a service
///
/// An implementation of the `standard` AWS retry strategy. A `Strategy` is scoped to a client.
/// For an individual request, call [`Standard::new_request_policy()`](Standard::new_request_policy)
#[derive(Debug, Clone)]
pub struct Standard {
    config: Config,
    shared_state: CrossRequestRetryState,
}

impl Standard {
    /// Construct a new standard retry policy from the given policy configuration.
    pub fn new(config: Config) -> Self {
        Self {
            shared_state: CrossRequestRetryState::new(config.initial_retry_tokens),
            config,
        }
    }

    /// Set the configuration for this retry policy.
    pub fn with_config(&mut self, config: Config) -> &mut Self {
        self.config = config;
        self
    }
}

impl NewRequestPolicy for Standard {
    type Policy = RetryHandler;

    fn new_request_policy(&self, sleep_impl: Option<SharedAsyncSleep>) -> Self::Policy {
        RetryHandler {
            local: RequestLocalRetryState::new(),
            shared: self.shared_state.clone(),
            config: self.config.clone(),
            sleep_impl,
        }
    }
}

impl Default for Standard {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

#[derive(Clone, Debug)]
struct RequestLocalRetryState {
    attempts: u32,
    last_quota_usage: Option<usize>,
}

impl Default for RequestLocalRetryState {
    fn default() -> Self {
        Self {
            // Starts at one to account for the initial request that failed and warranted a retry
            attempts: 1,
            last_quota_usage: None,
        }
    }
}

impl RequestLocalRetryState {
    fn new() -> Self {
        Self::default()
    }
}

/* TODO(retries)
/// RetryPartition represents a scope for cross request retry state
///
/// For example, a retry partition could be the id of a service. This would give each service a separate retry budget.
struct RetryPartition(Cow<'static, str>); */

/// Shared state between multiple requests to the same client.
#[derive(Clone, Debug)]
struct CrossRequestRetryState {
    quota_available: Arc<Mutex<usize>>,
}

// clippy is upset that we didn't use AtomicUsize here, but doing so makes the code
// significantly more complicated for negligible benefit.
#[allow(clippy::mutex_atomic)]
impl CrossRequestRetryState {
    fn new(initial_quota: usize) -> Self {
        Self {
            quota_available: Arc::new(Mutex::new(initial_quota)),
        }
    }

    fn quota_release(&self, value: Option<usize>, config: &Config) {
        let mut quota = self.quota_available.lock().unwrap();
        *quota += value.unwrap_or(config.no_retry_increment);
    }

    /// Attempt to acquire retry quota for `ErrorKind`
    ///
    /// If quota is available, the amount of quota consumed is returned
    /// If no quota is available, `None` is returned.
    fn quota_acquire(&self, err: &ErrorKind, config: &Config) -> Option<usize> {
        let mut quota = self.quota_available.lock().unwrap();
        let retry_cost = if err == &ErrorKind::TransientError {
            config.timeout_retry_cost
        } else {
            config.retry_cost
        };
        if retry_cost > *quota {
            None
        } else {
            *quota -= retry_cost;
            Some(retry_cost)
        }
    }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// RetryHandler
///
/// Implement retries for an individual request.
/// It is intended to be used as a [Tower Retry Policy](tower::retry::Policy) for use in tower-based
/// middleware stacks.
#[derive(Clone, Debug)]
pub struct RetryHandler {
    local: RequestLocalRetryState,
    shared: CrossRequestRetryState,
    config: Config,
    sleep_impl: Option<SharedAsyncSleep>,
}

#[cfg(test)]
impl RetryHandler {
    fn retry_quota(&self) -> usize {
        *self.shared.quota_available.lock().unwrap()
    }
}

/// For a request that gets retried 3 times, when base is 1 and initial_backoff is 2 seconds:
/// - the first retry will occur after 0 to 2 seconds
/// - the second retry will occur after 0 to 4 seconds
/// - the third retry will occur after 0 to 8 seconds
///
/// For a request that gets retried 3 times, when base is 1 and initial_backoff is 30 milliseconds:
/// - the first retry will occur after 0 to 30 milliseconds
/// - the second retry will occur after 0 to 60 milliseconds
/// - the third retry will occur after 0 to 120 milliseconds
fn calculate_exponential_backoff(base: f64, initial_backoff: f64, retry_attempts: u32) -> f64 {
    base * initial_backoff * 2_u32.pow(retry_attempts) as f64
}

impl RetryHandler {
    /// Determine the correct response given `retry_kind`
    ///
    /// If a retry is specified, this function returns `(next, backoff_duration)`
    /// If no retry is specified, this function returns None
    fn should_retry_error(&self, error_kind: &ErrorKind) -> Option<(Self, Duration)> {
        let quota_used = {
            if self.local.attempts == self.config.max_attempts {
                tracing::trace!(
                    attempts = self.local.attempts,
                    max_attempts = self.config.max_attempts,
                    "not retrying becuase we are out of attempts"
                );
                return None;
            }
            match self.shared.quota_acquire(error_kind, &self.config) {
                Some(quota) => quota,
                None => {
                    tracing::trace!(state = ?self.shared, "not retrying because no quota is available");
                    return None;
                }
            }
        };
        let backoff = calculate_exponential_backoff(
            // Generate a random base multiplier to create jitter
            (self.config.base)(),
            // Get the backoff time multiplier in seconds (with fractional seconds)
            self.config.initial_backoff.as_secs_f64(),
            // `self.local.attempts` tracks number of requests made including the initial request
            // The initial attempt shouldn't count towards backoff calculations so we subtract it
            self.local.attempts - 1,
        );
        let backoff = Duration::from_secs_f64(backoff).min(self.config.max_backoff);
        let next = RetryHandler {
            local: RequestLocalRetryState {
                attempts: self.local.attempts + 1,
                last_quota_usage: Some(quota_used),
            },
            shared: self.shared.clone(),
            config: self.config.clone(),
            sleep_impl: self.sleep_impl.clone(),
        };

        Some((next, backoff))
    }

    fn should_retry(&self, retry_kind: &RetryKind) -> Option<(Self, Duration)> {
        match retry_kind {
            RetryKind::Explicit(dur) => Some((self.clone(), *dur)),
            RetryKind::UnretryableFailure => None,
            RetryKind::Unnecessary => {
                self.shared
                    .quota_release(self.local.last_quota_usage, &self.config);
                None
            }
            RetryKind::Error(err) => self.should_retry_error(err),
            _ => None,
        }
    }

    fn retry_for(&self, retry_kind: RetryKind) -> Option<BoxFuture<Self>> {
        let retry = self.should_retry(&retry_kind);
        tracing::trace!(retry=?retry, retry_kind = ?retry_kind, "retry action");
        let (next, dur) = retry?;

        let sleep = match &self.sleep_impl {
            Some(sleep) => sleep,
            None => {
                if retry_kind != RetryKind::UnretryableFailure {
                    tracing::debug!("cannot retry because no sleep implementation exists");
                }
                return None;
            }
        };

        tracing::debug!(
            "attempt {} failed with {:?}; retrying after {:?}",
            self.local.attempts,
            retry_kind,
            dur
        );
        let sleep_future = sleep.sleep(dur);
        let fut = async move {
            sleep_future.await;
            next
        }
        .instrument(tracing::debug_span!("retry", kind = &debug(retry_kind)));
        Some(check_send(Box::pin(fut)))
    }
}

impl<Handler, R, T, E> tower::retry::Policy<Operation<Handler, R>, SdkSuccess<T>, SdkError<E>>
    for RetryHandler
where
    Handler: Clone,
    R: ClassifyRetry<SdkSuccess<T>, SdkError<E>>,
{
    type Future = BoxFuture<Self>;

    fn retry(
        &self,
        req: &Operation<Handler, R>,
        result: Result<&SdkSuccess<T>, &SdkError<E>>,
    ) -> Option<Self::Future> {
        let classifier = req.retry_classifier();
        let retry_kind = classifier.classify_retry(result);
        tracing::trace!(retry_kind = ?retry_kind, "retry classification");
        self.retry_for(retry_kind)
    }

    fn clone_request(&self, req: &Operation<Handler, R>) -> Option<Operation<Handler, R>> {
        req.try_clone()
    }
}

fn check_send<T: Send>(t: T) -> T {
    t
}

#[cfg(test)]
mod test {
    use super::{calculate_exponential_backoff, Config, NewRequestPolicy, RetryHandler, Standard};

    use aws_smithy_types::retry::{ErrorKind, RetryKind};

    use std::time::Duration;

    fn test_config() -> Config {
        Config::default().with_base(|| 1_f64)
    }

    #[test]
    fn retry_handler_send_sync() {
        fn must_be_send_sync<T: Send + Sync>() {}

        must_be_send_sync::<RetryHandler>()
    }

    #[test]
    fn eventual_success() {
        let policy = Standard::new(test_config()).new_request_policy(None);
        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let no_retry = policy.should_retry(&RetryKind::Unnecessary);
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 495);
    }

    #[test]
    fn no_more_attempts() {
        let policy = Standard::new(test_config()).new_request_policy(None);
        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let no_retry = policy.should_retry(&RetryKind::Error(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 490);
    }

    #[test]
    fn no_quota() {
        let mut conf = test_config();
        conf.initial_retry_tokens = 5;
        let policy = Standard::new(conf).new_request_policy(None);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 0);

        let no_retry = policy.should_retry(&RetryKind::Error(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 0);
    }

    #[test]
    fn quota_replenishes_on_success() {
        let mut conf = test_config();
        conf.initial_retry_tokens = 100;
        let policy = Standard::new(conf).new_request_policy(None);
        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::TransientError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 90);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Explicit(Duration::from_secs(1)))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(
            policy.retry_quota(),
            90,
            "explicit retry should not subtract from quota"
        );

        assert!(
            policy.should_retry(&RetryKind::Unnecessary).is_none(),
            "it should not retry success"
        );
        let available = policy.shared.quota_available.lock().unwrap();
        assert_eq!(100, *available, "successful request should replenish quota");
    }

    #[test]
    fn backoff_timing() {
        let mut conf = test_config();
        conf.max_attempts = 5;
        let policy = Standard::new(conf).new_request_policy(None);
        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(4));
        assert_eq!(policy.retry_quota(), 485);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(8));
        assert_eq!(policy.retry_quota(), 480);

        let no_retry = policy.should_retry(&RetryKind::Error(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 480);
    }

    #[test]
    fn max_backoff_time() {
        let mut conf = test_config();
        conf.max_attempts = 5;
        conf.initial_backoff = Duration::from_secs(1);
        conf.max_backoff = Duration::from_secs(3);
        let policy = Standard::new(conf).new_request_policy(None);
        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(policy.retry_quota(), 485);

        let (policy, dur) = policy
            .should_retry(&RetryKind::Error(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(policy.retry_quota(), 480);

        let no_retry = policy.should_retry(&RetryKind::Error(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 480);
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
