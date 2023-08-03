/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Retry handling and token bucket.
//!
//! This code defines when and how failed requests should be retried. It also defines the behavior
//! used to limit the rate that requests are sent.

use crate::client::interceptors::context::InterceptorContext;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use std::fmt::Debug;
use std::time::Duration;
use tracing::trace;

pub use aws_smithy_types::retry::ErrorKind;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An answer to the question "should I make a request attempt?"
pub enum ShouldAttempt {
    /// Yes, an attempt should be made
    Yes,
    /// No, no attempt should be made
    No,
    /// Yes, an attempt should be made, but only after the given amount of time has passed
    YesAfterDelay(Duration),
}

#[cfg(feature = "test-util")]
impl ShouldAttempt {
    /// Returns the delay duration if this is a `YesAfterDelay` variant.
    pub fn expect_delay(self) -> Duration {
        match self {
            ShouldAttempt::YesAfterDelay(delay) => delay,
            _ => panic!("Expected this to be the `YesAfterDelay` variant but it was the `{self:?}` variant instead"),
        }
    }
}

/// Decider for whether or not to attempt a request, and when.
///
/// The orchestrator consults the retry strategy every time before making a request.
/// This includes the initial request, and any retry attempts thereafter. The
/// orchestrator will retry indefinitely (until success) if the retry strategy
/// always returns `ShouldAttempt::Yes` from `should_attempt_retry`.
pub trait RetryStrategy: Send + Sync + Debug {
    /// Decides if the initial attempt should be made.
    fn should_attempt_initial_request(
        &self,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError>;

    /// Decides if a retry should be done.
    ///
    /// The previous attempt's output or error are provided in the
    /// [`InterceptorContext`] when this is called.
    ///
    /// `ShouldAttempt::YesAfterDelay` can be used to add a backoff time.
    fn should_attempt_retry(
        &self,
        context: &InterceptorContext,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError>;
}

/// A shared retry strategy.
#[derive(Clone, Debug)]
pub struct SharedRetryStrategy(Arc<dyn RetryStrategy>);

impl SharedRetryStrategy {
    /// Creates a new [`SharedRetryStrategy`] from a retry strategy.
    pub fn new(retry_strategy: impl RetryStrategy + 'static) -> Self {
        Self(Arc::new(retry_strategy))
    }
}

impl RetryStrategy for SharedRetryStrategy {
    fn should_attempt_initial_request(
        &self,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        self.0
            .should_attempt_initial_request(runtime_components, cfg)
    }

    fn should_attempt_retry(
        &self,
        context: &InterceptorContext,
        runtime_components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        self.0
            .should_attempt_retry(context, runtime_components, cfg)
    }
}

/// Classification result from [`ClassifyRetry`].
#[non_exhaustive]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum RetryReason {
    /// There was an unexpected error, and this is the kind of error so that it can be properly retried.
    Error(ErrorKind),
    /// The server explicitly told us to back off by this amount of time.
    Explicit(Duration),
}

/// Classifies what kind of retry is needed for a given an [`InterceptorContext`].
pub trait ClassifyRetry: Send + Sync + Debug {
    /// Run this classifier against an error to determine if it should be retried. Returns
    /// `Some(RetryKind)` if the error should be retried; Otherwise returns `None`.
    fn classify_retry(&self, ctx: &InterceptorContext) -> Option<RetryReason>;

    /// The name that this classifier should report for debugging purposes.
    fn name(&self) -> &'static str;
}

/// Classifies an error into a [`RetryReason`].
#[derive(Clone, Debug)]
pub struct RetryClassifiers {
    inner: Vec<Arc<dyn ClassifyRetry>>,
}

impl RetryClassifiers {
    /// Creates a new [`RetryClassifiers`].
    pub fn new() -> Self {
        Self {
            // It's always expected that at least one classifier will be defined,
            // so we eagerly allocate for it.
            inner: Vec::with_capacity(1),
        }
    }

    /// Adds a classifier to this collection.
    pub fn with_classifier(mut self, retry_classifier: impl ClassifyRetry + 'static) -> Self {
        self.inner.push(Arc::new(retry_classifier));
        self
    }

    // TODO(https://github.com/awslabs/smithy-rs/issues/2632) make a map function so users can front-run or second-guess the classifier's decision
    // pub fn map_classifiers(mut self, fun: Fn() -> RetryClassifiers)
}

impl ClassifyRetry for RetryClassifiers {
    fn classify_retry(&self, ctx: &InterceptorContext) -> Option<RetryReason> {
        // return the first non-None result
        self.inner.iter().find_map(|cr| {
            let maybe_reason = cr.classify_retry(ctx);

            match maybe_reason.as_ref() {
                Some(reason) => trace!(
                    "\"{}\" classifier classified error as {:?}",
                    cr.name(),
                    reason
                ),
                None => trace!("\"{}\" classifier ignored the error", cr.name()),
            };

            maybe_reason
        })
    }

    fn name(&self) -> &'static str {
        "Collection of Classifiers"
    }
}

/// A type to track the number of requests sent by the orchestrator for a given operation.
///
/// `RequestAttempts` is added to the `ConfigBag` by the orchestrator,
/// and holds the current attempt number.
#[derive(Debug, Clone, Copy)]
pub struct RequestAttempts {
    attempts: u32,
}

impl RequestAttempts {
    /// Creates a new [`RequestAttempts`] with the given number of attempts.
    pub fn new(attempts: u32) -> Self {
        Self { attempts }
    }

    /// Returns the number of attempts.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }
}

impl From<u32> for RequestAttempts {
    fn from(attempts: u32) -> Self {
        Self::new(attempts)
    }
}

impl From<RequestAttempts> for u32 {
    fn from(value: RequestAttempts) -> Self {
        value.attempts()
    }
}

impl Storable for RequestAttempts {
    type Storer = StoreReplace<Self>;
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::{ClassifyRetry, ErrorKind, RetryReason};
    use crate::client::interceptors::context::InterceptorContext;
    use tracing::trace;

    /// A retry classifier for testing purposes. This classifier always returns
    /// `Some(RetryReason::Error(ErrorKind))` where `ErrorKind` is the value provided when creating
    /// this classifier.
    #[derive(Debug)]
    pub struct AlwaysRetry(pub ErrorKind);

    impl ClassifyRetry for AlwaysRetry {
        fn classify_retry(&self, error: &InterceptorContext) -> Option<RetryReason> {
            trace!("Retrying error {:?} as an {:?}", error, self.0);
            Some(RetryReason::Error(self.0))
        }

        fn name(&self) -> &'static str {
            "Always Retry"
        }
    }
}

use crate::box_error::BoxError;
use crate::client::runtime_components::RuntimeComponents;
use std::sync::Arc;
#[cfg(feature = "test-util")]
pub use test_util::AlwaysRetry;
