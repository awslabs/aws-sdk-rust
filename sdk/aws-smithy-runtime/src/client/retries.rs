/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Smithy retry classifiers.
pub mod classifier;

/// Smithy retry strategies.
pub mod strategy;

mod client_rate_limiter;
mod token_bucket;

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::fmt;

pub use client_rate_limiter::{ClientRateLimiter, ClientRateLimiterRuntimePlugin};
pub use token_bucket::{TokenBucket, TokenBucketRuntimePlugin};

#[doc(hidden)]
pub use client_rate_limiter::ClientRateLimiterPartition;
#[doc(hidden)]
pub use token_bucket::TokenBucketPartition;

#[doc(hidden)]
#[non_exhaustive]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RetryPartition {
    inner: &'static str,
}

impl RetryPartition {
    pub fn new(name: &'static str) -> Self {
        Self { inner: name }
    }
}

impl fmt::Display for RetryPartition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl Storable for RetryPartition {
    type Storer = StoreReplace<RetryPartition>;
}
