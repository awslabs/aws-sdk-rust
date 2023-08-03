/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Time source abstraction to support WASM and testing
use std::fmt::Debug;
use std::time::SystemTime;

/// Trait with a `now()` function returning the current time
pub trait TimeSource: Debug + Send + Sync {
    /// Returns the current time
    fn now(&self) -> SystemTime;
}

/// Time source that delegates to [`SystemTime::now`]
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct SystemTimeSource;

impl SystemTimeSource {
    /// Creates a new SystemTimeSource
    pub fn new() -> Self {
        SystemTimeSource
    }
}

impl TimeSource for SystemTimeSource {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
