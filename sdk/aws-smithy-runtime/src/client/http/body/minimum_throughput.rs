/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A body-wrapping type that ensures data is being streamed faster than some lower limit.
//!
//! If data is being streamed too slowly, this body type will emit an error next time it's polled.

use aws_smithy_async::rt::sleep::Sleep;
use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep};
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::shared::IntoShared;
use std::fmt;
use std::time::Duration;
use throughput::{Throughput, ThroughputLogs};

/// An implementation of v0.4 `http_body::Body` for `MinimumThroughputBody` and related code.
pub mod http_body_0_4_x;

mod throughput;

pin_project_lite::pin_project! {
    /// A body-wrapping type that ensures data is being streamed faster than some lower limit.
    ///
    /// If data is being streamed too slowly, this body type will emit an error next time it's polled.
    pub struct MinimumThroughputBody<B> {
        async_sleep: SharedAsyncSleep,
        time_source: SharedTimeSource,
        minimum_throughput: Throughput,
        throughput_logs: ThroughputLogs,
        #[pin]
        sleep_fut: Option<Sleep>,
        #[pin]
        inner: B,
    }
}

const SIZE_OF_ONE_LOG: usize = std::mem::size_of::<(std::time::SystemTime, u64)>(); // 24 bytes per log
const NUMBER_OF_LOGS_IN_ONE_KB: f64 = 1024.0 / SIZE_OF_ONE_LOG as f64;

impl<B> MinimumThroughputBody<B> {
    /// Create a new minimum throughput body.
    pub fn new(
        time_source: impl TimeSource + 'static,
        async_sleep: impl AsyncSleep + 'static,
        body: B,
        (bytes_read, per_time_elapsed): (u64, Duration),
    ) -> Self {
        let minimum_throughput = Throughput::new(bytes_read as f64, per_time_elapsed);
        Self {
            throughput_logs: ThroughputLogs::new(
                // Never keep more than 10KB of logs in memory. This currently
                // equates to 426 logs.
                (NUMBER_OF_LOGS_IN_ONE_KB * 10.0) as usize,
                minimum_throughput.per_time_elapsed(),
            ),
            async_sleep: async_sleep.into_shared(),
            time_source: time_source.into_shared(),
            minimum_throughput,
            inner: body,
            sleep_fut: None,
        }
    }
}

#[derive(Debug)]
enum Error {
    ThroughputBelowMinimum {
        expected: Throughput,
        actual: Throughput,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThroughputBelowMinimum { expected, actual } => {
                write!(
                    f,
                    "minimum throughput was specified at {expected}, but throughput of {actual} was observed",
                )
            }
        }
    }
}

impl std::error::Error for Error {}

// Tests are implemented per HTTP body type.
