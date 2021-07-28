/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::time::SystemTime;

/// Allows us to abstract time for tests.
pub(super) trait TimeSource: Send + Sync + 'static {
    fn now(&self) -> SystemTime;
}

#[derive(Copy, Clone)]
pub(super) struct SystemTimeSource;

impl TimeSource for SystemTimeSource {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}
