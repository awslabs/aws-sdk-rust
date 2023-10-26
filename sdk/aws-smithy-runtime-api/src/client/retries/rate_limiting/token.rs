/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types and traits related to token buckets. Token buckets are used to limit the amount of
//! requests a client sends in order to avoid getting throttled. Token buckets can also act as a
//! form of concurrency control if a token is required to send a new request (as opposed to retry
//! requests only).

use tokio::sync::OwnedSemaphorePermit;

/// A trait implemented by types that represent a token dispensed from a [`TokenBucket`](super::TokenBucket).
pub trait Token {
    /// Release this token back to the bucket. This should be called if the related request succeeds.
    fn release(self);

    /// Forget this token, forever banishing it to the shadow realm, from whence no tokens return.
    /// This should be called if the related request fails.
    fn forget(self);
}

/// The token type of [`Standard`].
#[derive(Debug)]
pub struct Standard {
    permit: Option<OwnedSemaphorePermit>,
}

impl Standard {
    pub(crate) fn new(permit: OwnedSemaphorePermit) -> Self {
        Self {
            permit: Some(permit),
        }
    }

    // Return an "empty" token for times when you need to return a token but there's no "cost"
    // associated with an action.
    pub(crate) fn empty() -> Self {
        Self { permit: None }
    }
}

impl Token for Standard {
    fn release(self) {
        drop(self.permit)
    }

    fn forget(self) {
        if let Some(permit) = self.permit {
            permit.forget()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Standard as Token;
    use crate::client::retries::rate_limiting::token_bucket::Standard as TokenBucket;

    #[test]
    fn token_bucket_trait_is_dyn_safe() {
        let _tb: Box<dyn crate::client::retries::rate_limiting::TokenBucket<Token = Token>> =
            Box::new(TokenBucket::builder().build());
    }
}
