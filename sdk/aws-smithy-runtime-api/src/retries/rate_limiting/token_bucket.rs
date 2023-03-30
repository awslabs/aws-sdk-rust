/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A token bucket intended for use with the standard smithy client retry policy.

use super::error::RateLimitingError;
use super::token;
use super::Token;
use aws_smithy_types::retry::{ErrorKind, RetryKind};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::sync::TryAcquireError;

/// The default number of tokens to start with
const STANDARD_INITIAL_RETRY_TOKENS: usize = 500;
/// The amount of tokens to remove from the bucket when a timeout error occurs
const STANDARD_TIMEOUT_ERROR_RETRY_COST: u32 = 10;
/// The amount of tokens to remove from the bucket when a throttling error occurs
const STANDARD_RETRYABLE_ERROR_RETRY_COST: u32 = 5;

/// This trait is implemented by types that act as token buckets. Token buckets are used to regulate
/// the amount of requests sent by clients. Different token buckets may apply different strategies
/// to manage the number of tokens in a bucket.
///
/// related: [`Token`], [`RateLimitingError`]
pub trait TokenBucket {
    /// The type of tokens this bucket dispenses.
    type Token: Token;

    /// Attempt to acquire a token from the bucket. This will fail if the bucket has no more tokens.
    fn try_acquire(
        &self,
        previous_response_kind: Option<RetryKind>,
    ) -> Result<Self::Token, RateLimitingError>;

    /// Get the number of available tokens in the bucket.
    fn available(&self) -> usize;

    /// Refill the bucket with the given number of tokens.
    fn refill(&self, tokens: usize);
}

/// A token bucket implementation that uses a `tokio::sync::Semaphore` to track the number of tokens.
///
/// - Whenever a request succeeds on the first try, `<success_on_first_try_refill_amount>` token(s)
///     are added back to the bucket.
/// - When a request fails with a timeout error, `<timeout_error_cost>` token(s)
///     are removed from the bucket.
/// - When a request fails with a retryable error, `<retryable_error_cost>` token(s)
///     are removed from the bucket.
///
/// The number of tokens in the bucket will always be >= `0` and <= `<max_tokens>`.
#[derive(Clone, Debug)]
pub struct Standard {
    inner: Arc<Semaphore>,
    max_tokens: usize,
    timeout_error_cost: u32,
    retryable_error_cost: u32,
}

impl Standard {
    /// Create a new `TokenBucket` using builder methods.
    pub fn builder() -> Builder {
        Builder::default()
    }
}

/// A builder for `TokenBucket`s.
#[derive(Default, Debug)]
pub struct Builder {
    starting_tokens: Option<usize>,
    max_tokens: Option<usize>,
    timeout_error_cost: Option<u32>,
    retryable_error_cost: Option<u32>,
}

impl Builder {
    /// The number of tokens the bucket will start with. Defaults to 500.
    pub fn starting_tokens(mut self, starting_tokens: usize) -> Self {
        self.starting_tokens = Some(starting_tokens);
        self
    }

    /// The maximum number of tokens that the bucket can hold.
    /// Defaults to the value of `starting_tokens`.
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// How many tokens to remove from the bucket when a request fails due to a timeout error.
    /// Defaults to 10.
    pub fn timeout_error_cost(mut self, timeout_error_cost: u32) -> Self {
        self.timeout_error_cost = Some(timeout_error_cost);
        self
    }

    /// How many tokens to remove from the bucket when a request fails due to a retryable error that
    /// isn't timeout-related. Defaults to 5.
    pub fn retryable_error_cost(mut self, retryable_error_cost: u32) -> Self {
        self.retryable_error_cost = Some(retryable_error_cost);
        self
    }

    /// Build this builder. Unset fields will be set to their default values.
    pub fn build(self) -> Standard {
        let starting_tokens = self
            .starting_tokens
            .unwrap_or(STANDARD_INITIAL_RETRY_TOKENS);
        let max_tokens = self.max_tokens.unwrap_or(starting_tokens);
        let timeout_error_cost = self
            .timeout_error_cost
            .unwrap_or(STANDARD_TIMEOUT_ERROR_RETRY_COST);
        let retryable_error_cost = self
            .retryable_error_cost
            .unwrap_or(STANDARD_RETRYABLE_ERROR_RETRY_COST);

        Standard {
            inner: Arc::new(Semaphore::new(starting_tokens)),
            max_tokens,
            timeout_error_cost,
            retryable_error_cost,
        }
    }
}

impl TokenBucket for Standard {
    type Token = token::Standard;

    fn try_acquire(
        &self,
        previous_response_kind: Option<RetryKind>,
    ) -> Result<Self::Token, RateLimitingError> {
        let number_of_tokens_to_acquire = match previous_response_kind {
            None => {
                // Return an empty token because the quota layer lifecycle expects a for each
                // request even though the standard token bucket only requires tokens for retry
                // attempts.
                return Ok(token::Standard::empty());
            }

            Some(retry_kind) => match retry_kind {
                RetryKind::Unnecessary => {
                    unreachable!("BUG: asked for a token to retry a successful request")
                }
                RetryKind::UnretryableFailure => {
                    unreachable!("BUG: asked for a token to retry an un-retryable request")
                }
                RetryKind::Explicit(_) => self.retryable_error_cost,
                RetryKind::Error(error_kind) => match error_kind {
                    ErrorKind::ThrottlingError | ErrorKind::TransientError => {
                        self.timeout_error_cost
                    }
                    ErrorKind::ServerError => self.retryable_error_cost,
                    ErrorKind::ClientError => unreachable!(
                        "BUG: asked for a token to retry a request that failed due to user error"
                    ),
                    _ => unreachable!(
                        "A new variant '{:?}' was added to ErrorKind, please handle it",
                        error_kind
                    ),
                },
                _ => unreachable!(
                    "A new variant '{:?}' was added to RetryKind, please handle it",
                    retry_kind
                ),
            },
        };

        match self
            .inner
            .clone()
            .try_acquire_many_owned(number_of_tokens_to_acquire)
        {
            Ok(permit) => Ok(token::Standard::new(permit)),
            Err(TryAcquireError::NoPermits) => Err(RateLimitingError::no_tokens()),
            Err(other) => Err(RateLimitingError::bug(other.to_string())),
        }
    }

    fn available(&self) -> usize {
        self.inner.available_permits()
    }

    fn refill(&self, tokens: usize) {
        // Ensure the bucket doesn't overflow by limiting the amount of tokens to add, if necessary.
        let amount_to_add = (self.available() + tokens).min(self.max_tokens) - self.available();
        if amount_to_add > 0 {
            self.inner.add_permits(amount_to_add)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Token, TokenBucket};
    use super::{
        STANDARD_INITIAL_RETRY_TOKENS, STANDARD_RETRYABLE_ERROR_RETRY_COST,
        STANDARD_TIMEOUT_ERROR_RETRY_COST,
    };
    use aws_smithy_types::retry::{ErrorKind, RetryKind};

    #[test]
    fn bucket_works() {
        let bucket = super::Standard::builder().build();
        assert_eq!(bucket.available(), STANDARD_INITIAL_RETRY_TOKENS);

        let token = bucket
            .try_acquire(Some(RetryKind::Error(ErrorKind::ServerError)))
            .unwrap();
        assert_eq!(
            bucket.available(),
            STANDARD_INITIAL_RETRY_TOKENS - STANDARD_RETRYABLE_ERROR_RETRY_COST as usize
        );
        Box::new(token).release();

        let token = bucket
            .try_acquire(Some(RetryKind::Error(ErrorKind::TransientError)))
            .unwrap();
        assert_eq!(
            bucket.available(),
            STANDARD_INITIAL_RETRY_TOKENS - STANDARD_TIMEOUT_ERROR_RETRY_COST as usize
        );
        Box::new(token).forget();
        assert_eq!(
            bucket.available(),
            STANDARD_INITIAL_RETRY_TOKENS - STANDARD_TIMEOUT_ERROR_RETRY_COST as usize
        );

        bucket.refill(STANDARD_TIMEOUT_ERROR_RETRY_COST as usize);
        assert_eq!(bucket.available(), STANDARD_INITIAL_RETRY_TOKENS);
    }
}
