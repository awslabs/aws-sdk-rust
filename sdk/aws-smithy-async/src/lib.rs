/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    missing_debug_implementations,
    missing_docs,
    rustdoc::all,
    rust_2018_idioms
)]

//! Future utilities and runtime-agnostic abstractions for smithy-rs.
//!
//! Async runtime specific code is abstracted behind async traits, and implementations are
//! provided via feature flag. For now, only Tokio runtime implementations are provided.

pub mod future;
pub mod rt;

/// Given an `Instant` and a `Duration`, assert time elapsed since `Instant` is equal to `Duration`.
/// This macro allows for a 5ms margin of error.
///
/// # Example
///
/// ```rust,ignore
/// let now = std::time::Instant::now();
/// let _ = some_function_that_always_takes_five_seconds_to_run().await;
/// assert_elapsed!(now, std::time::Duration::from_secs(5));
/// ```
#[macro_export]
macro_rules! assert_elapsed {
    ($start:expr, $dur:expr) => {{
        let elapsed = $start.elapsed();
        // type ascription improves compiler error when wrong type is passed
        let lower: std::time::Duration = $dur;

        // Handles ms rounding
        assert!(
            elapsed >= lower && elapsed <= lower + std::time::Duration::from_millis(5),
            "actual = {:?}, expected = {:?}",
            elapsed,
            lower
        );
    }};
}
