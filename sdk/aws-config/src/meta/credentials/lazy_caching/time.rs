/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::fmt::Debug;
use std::time::SystemTime;

/// Wall Clock Time Source
///
/// By default, `SystemTime::now()` is used, however, this trait allows
/// tests to provide their own time source.
pub(super) trait TimeSource: Send + Sync + Debug + 'static {
    fn now(&self) -> SystemTime;
}

/// Load time from `SystemTime::now()`
#[derive(Copy, Clone, Debug)]
pub(super) struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
