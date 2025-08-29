/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_smithy_types::retry::ErrorKind;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::trace;

const DEFAULT_CAPACITY: usize = 500;
const RETRY_COST: u32 = 5;
const RETRY_TIMEOUT_COST: u32 = RETRY_COST * 2;
const PERMIT_REGENERATION_AMOUNT: usize = 1;

/// Token bucket used for standard and adaptive retry.
#[derive(Clone, Debug)]
pub struct TokenBucket {
    semaphore: Arc<Semaphore>,
    max_permits: usize,
    timeout_retry_cost: u32,
    retry_cost: u32,
}

impl Storable for TokenBucket {
    type Storer = StoreReplace<Self>;
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(DEFAULT_CAPACITY)),
            max_permits: DEFAULT_CAPACITY,
            timeout_retry_cost: RETRY_TIMEOUT_COST,
            retry_cost: RETRY_COST,
        }
    }
}

impl TokenBucket {
    /// Creates a new `TokenBucket` with the given initial quota.
    pub fn new(initial_quota: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(initial_quota)),
            max_permits: initial_quota,
            ..Default::default()
        }
    }

    /// A token bucket with unlimited capacity that allows retries at no cost.
    pub fn unlimited() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(Semaphore::MAX_PERMITS)),
            max_permits: Semaphore::MAX_PERMITS,
            timeout_retry_cost: 0,
            retry_cost: 0,
        }
    }

    /// Creates a builder for constructing a `TokenBucket`.
    pub fn builder() -> TokenBucketBuilder {
        TokenBucketBuilder::default()
    }

    pub(crate) fn acquire(&self, err: &ErrorKind) -> Option<OwnedSemaphorePermit> {
        let retry_cost = if err == &ErrorKind::TransientError {
            self.timeout_retry_cost
        } else {
            self.retry_cost
        };

        self.semaphore
            .clone()
            .try_acquire_many_owned(retry_cost)
            .ok()
    }

    pub(crate) fn regenerate_a_token(&self) {
        if self.semaphore.available_permits() < self.max_permits {
            trace!("adding {PERMIT_REGENERATION_AMOUNT} back into the bucket");
            self.semaphore.add_permits(PERMIT_REGENERATION_AMOUNT)
        }
    }

    #[cfg(all(test, any(feature = "test-util", feature = "legacy-test-util")))]
    pub(crate) fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

/// Builder for constructing a `TokenBucket`.
#[derive(Clone, Debug, Default)]
pub struct TokenBucketBuilder {
    capacity: Option<usize>,
    retry_cost: Option<u32>,
    timeout_retry_cost: Option<u32>,
}

impl TokenBucketBuilder {
    /// Creates a new `TokenBucketBuilder` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum bucket capacity for the builder.
    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Sets the specified retry cost for the builder.
    pub fn retry_cost(mut self, retry_cost: u32) -> Self {
        self.retry_cost = Some(retry_cost);
        self
    }

    /// Sets the specified timeout retry cost for the builder.
    pub fn timeout_retry_cost(mut self, timeout_retry_cost: u32) -> Self {
        self.timeout_retry_cost = Some(timeout_retry_cost);
        self
    }

    /// Builds a `TokenBucket`.
    pub fn build(self) -> TokenBucket {
        TokenBucket {
            semaphore: Arc::new(Semaphore::new(self.capacity.unwrap_or(DEFAULT_CAPACITY))),
            max_permits: self.capacity.unwrap_or(DEFAULT_CAPACITY),
            retry_cost: self.retry_cost.unwrap_or(RETRY_COST),
            timeout_retry_cost: self.timeout_retry_cost.unwrap_or(RETRY_TIMEOUT_COST),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlimited_token_bucket() {
        let bucket = TokenBucket::unlimited();

        // Should always acquire permits regardless of error type
        assert!(bucket.acquire(&ErrorKind::ThrottlingError).is_some());
        assert!(bucket.acquire(&ErrorKind::TransientError).is_some());

        // Should have maximum capacity
        assert_eq!(bucket.max_permits, Semaphore::MAX_PERMITS);

        // Should have zero retry costs
        assert_eq!(bucket.retry_cost, 0);
        assert_eq!(bucket.timeout_retry_cost, 0);

        // The loop count is arbitrary; should obtain permits without limit
        let mut permits = Vec::new();
        for _ in 0..100 {
            let permit = bucket.acquire(&ErrorKind::ThrottlingError);
            assert!(permit.is_some());
            permits.push(permit);
            // Available permits should stay constant
            assert_eq!(
                tokio::sync::Semaphore::MAX_PERMITS,
                bucket.semaphore.available_permits()
            );
        }
    }

    #[test]
    fn test_bounded_permits_exhaustion() {
        let bucket = TokenBucket::new(10);
        let mut permits = Vec::new();

        for _ in 0..100 {
            let permit = bucket.acquire(&ErrorKind::ThrottlingError);
            if let Some(p) = permit {
                permits.push(p);
            } else {
                break;
            }
        }

        assert_eq!(permits.len(), 2); // 10 capacity / 5 retry cost = 2 permits

        // Verify next acquisition fails
        assert!(bucket.acquire(&ErrorKind::ThrottlingError).is_none());
    }
}
