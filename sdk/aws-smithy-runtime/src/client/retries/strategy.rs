/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod never;
pub(crate) mod standard;

pub use never::NeverRetryStrategy;
pub use standard::StandardRetryStrategy;

#[cfg(feature = "test-util")]
mod fixed_delay;
#[cfg(feature = "test-util")]
pub use fixed_delay::FixedDelayRetryStrategy;
