/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Retry support for aws-hyper
//!
//! The actual retry policy implementation will likely be replaced
//! with the CRT implementation once the bindings exist. This
//! implementation is intended to be _correct_ but not especially long lasting.
//!
//! Components:
//! - [`Standard`]: Top level manager, intended to be associated with a [`Client`](crate::Client).
//!   Its sole purpose in life is to create a [`RetryHandler`] for individual requests.
//! - [`RetryHandler`]: A request-scoped retry policy, backed by request-local state and shared
//!   state contained within [`Standard`].
//! - [`Config`]: Static configuration (max retries, max backoff etc.)

use crate::{SdkError, SdkSuccess};
use smithy_http::operation;
use smithy_http::operation::Operation;
use smithy_http::retry::ClassifyResponse;
use smithy_types::retry::{ErrorKind, RetryKind};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::Instrument;

/// A policy instantiator.
///
/// Implementors are essentially "policy factories" that can produce a new instance of a retry
/// policy mechanism for each request, which allows both shared global state _and_ per-request
/// local state.
pub trait NewRequestPolicy {
    /// The type of the per-request policy mechanism.
    type Policy;

    /// Create a new policy mechanism instance.
    fn new_request_policy(&self) -> Self::Policy;
}

/// Retry Policy Configuration
///
/// Without specific use cases, users should generally rely on the default values set by `[Config::default]`(Config::default).`
///
/// Currently these fields are private and no setters provided. As needed, this configuration will become user-modifiable in the future..
#[derive(Clone, Debug)]
pub struct Config {
    initial_retry_tokens: usize,
    retry_cost: usize,
    no_retry_increment: usize,
    timeout_retry_cost: usize,
    max_attempts: u32,
    max_backoff: Duration,
    base: fn() -> f64,
}

impl Config {
    /// Override `b` in the exponential backoff computation
    ///
    /// By default, `base` is a randomly generated value between 0 and 1. In tests, it can
    /// be helpful to override this:
    /// ```rust
    /// use smithy_client::retry::Config;
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
        }
    }
}

impl From<smithy_types::retry::RetryConfig> for Config {
    fn from(conf: smithy_types::retry::RetryConfig) -> Self {
        Self::default().with_max_attempts(conf.max_attempts())
    }
}

const MAX_ATTEMPTS: u32 = 3;
const INITIAL_RETRY_TOKENS: usize = 500;
const RETRY_COST: usize = 5;

/// Manage retries for a service
///
/// An implementation of the `standard` AWS retry strategy as specified in the SEP. A `Strategy` is scoped to a client.
/// For an individual request, call [`Standard::new_request_policy()`](Standard::new_request_policy)
///
/// In the future, adding support for the adaptive retry strategy will be added by adding a `TokenBucket` to
/// `CrossRequestRetryState`
/// Its main functionality is via `new_request_policy` which creates a `RetryHandler` to manage the retry for
/// an individual request.
#[derive(Debug)]
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

    fn new_request_policy(&self) -> Self::Policy {
        RetryHandler {
            local: RequestLocalRetryState::new(),
            shared: self.shared_state.clone(),
            config: self.config.clone(),
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
    pub fn new() -> Self {
        Self::default()
    }
}

/* TODO in followup PR:
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
    pub fn new(initial_quota: usize) -> Self {
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
}

#[cfg(test)]
impl RetryHandler {
    fn retry_quota(&self) -> usize {
        *self.shared.quota_available.lock().unwrap()
    }
}

impl RetryHandler {
    /// Determine the correct response given `retry_kind`
    ///
    /// If a retry is specified, this function returns `(next, backoff_duration)`
    /// If no retry is specified, this function returns None
    fn attempt_retry(&self, retry_kind: Result<(), ErrorKind>) -> Option<(Self, Duration)> {
        let quota_used = match retry_kind {
            Ok(_) => {
                self.shared
                    .quota_release(self.local.last_quota_usage, &self.config);
                return None;
            }
            Err(e) => {
                if self.local.attempts == self.config.max_attempts {
                    return None;
                }
                self.shared.quota_acquire(&e, &self.config)?
            }
        };
        /*
        From the retry spec:
            b = random number within the range of: 0 <= b <= 1
            r = 2
            t_i = min(br^i, MAX_BACKOFF);
         */
        let r: i32 = 2;
        let b = (self.config.base)();
        // `self.local.attempts` tracks number of requests made including the initial request
        // The initial attempt shouldn't count towards backoff calculations so we subtract it
        let backoff = b * (r.pow(self.local.attempts - 1) as f64);
        let backoff = Duration::from_secs_f64(backoff).min(self.config.max_backoff);
        let next = RetryHandler {
            local: RequestLocalRetryState {
                attempts: self.local.attempts + 1,
                last_quota_usage: Some(quota_used),
            },
            shared: self.shared.clone(),
            config: self.config.clone(),
        };

        Some((next, backoff))
    }
}

impl<Handler, R, T, E>
    tower::retry::Policy<operation::Operation<Handler, R>, SdkSuccess<T>, SdkError<E>>
    for RetryHandler
where
    Handler: Clone,
    R: ClassifyResponse<SdkSuccess<T>, SdkError<E>>,
{
    type Future = Pin<Box<dyn Future<Output = Self> + Send>>;

    fn retry(
        &self,
        req: &Operation<Handler, R>,
        result: Result<&SdkSuccess<T>, &SdkError<E>>,
    ) -> Option<Self::Future> {
        let policy = req.retry_policy();
        let retry = policy.classify(result);
        let (next, dur) = match retry {
            RetryKind::Explicit(dur) => (self.clone(), dur),
            RetryKind::NotRetryable => return None,
            RetryKind::Error(err) => self.attempt_retry(Err(err))?,
            _ => return None,
        };

        let fut = async move {
            tokio::time::sleep(dur).await;
            next
        }
        .instrument(tracing::info_span!("retry", kind = &debug(retry)));
        Some(check_send_sync(Box::pin(fut)))
    }

    fn clone_request(&self, req: &Operation<Handler, R>) -> Option<Operation<Handler, R>> {
        req.try_clone()
    }
}

fn check_send_sync<T: Send>(t: T) -> T {
    t
}

#[cfg(test)]
mod test {
    use crate::retry::{Config, NewRequestPolicy, RetryHandler, Standard};
    use smithy_types::retry::ErrorKind;
    use std::time::Duration;

    fn assert_send_sync<T: Send + Sync>() {}

    fn test_config() -> Config {
        Config::default().with_base(|| 1_f64)
    }

    #[test]
    fn retry_handler_send_sync() {
        assert_send_sync::<RetryHandler>()
    }

    #[test]
    fn eventual_success() {
        let policy = Standard::new(test_config()).new_request_policy();
        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let no_retry = policy.attempt_retry(Ok(()));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 495);
    }

    #[test]
    fn no_more_attempts() {
        let policy = Standard::new(test_config()).new_request_policy();
        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let no_retry = policy.attempt_retry(Err(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 490);
    }

    #[test]
    fn no_quota() {
        let mut conf = test_config();
        conf.initial_retry_tokens = 5;
        let policy = Standard::new(conf).new_request_policy();
        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 0);
        let no_retry = policy.attempt_retry(Err(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 0);
    }

    #[test]
    fn backoff_timing() {
        let mut conf = test_config();
        conf.max_attempts = 5;
        let policy = Standard::new(conf).new_request_policy();
        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(4));
        assert_eq!(policy.retry_quota(), 485);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(8));
        assert_eq!(policy.retry_quota(), 480);

        let no_retry = policy.attempt_retry(Err(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 480);
    }

    #[test]
    fn max_backoff_time() {
        let mut conf = test_config();
        conf.max_attempts = 5;
        conf.max_backoff = Duration::from_secs(3);
        let policy = Standard::new(conf).new_request_policy();
        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(1));
        assert_eq!(policy.retry_quota(), 495);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(2));
        assert_eq!(policy.retry_quota(), 490);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(policy.retry_quota(), 485);

        let (policy, dur) = policy
            .attempt_retry(Err(ErrorKind::ServerError))
            .expect("should retry");
        assert_eq!(dur, Duration::from_secs(3));
        assert_eq!(policy.retry_quota(), 480);

        let no_retry = policy.attempt_retry(Err(ErrorKind::ServerError));
        assert!(no_retry.is_none());
        assert_eq!(policy.retry_quota(), 480);
    }
}
