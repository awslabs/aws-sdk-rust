/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! This module defines types that describe when to retry given a response.

use std::time::Duration;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    /// A connection-level error.
    ///
    /// A `TransientError` can represent conditions such as socket timeouts, socket connection errors, or TLS negotiation timeouts.
    ///
    /// `TransientError` is not modeled by Smithy and is instead determined through client-specific heuristics and response status codes.
    ///
    /// Typically these should never be applied for non-idempotent request types
    /// since in this scenario, it's impossible to know whether the operation had
    /// a side effect on the server.
    ///
    /// TransientErrors are not currently modeled. They are determined based on specific provider
    /// level errors & response status code.
    TransientError,

    /// An error where the server explicitly told the client to back off, such as a 429 or 503 HTTP error.
    ThrottlingError,

    /// Server error that isn't explicitly throttling but is considered by the client
    /// to be something that should be retried.
    ServerError,

    /// Doesn't count against any budgets. This could be something like a 401 challenge in Http.
    ClientError,
}

pub trait ProvideErrorKind {
    /// Returns the `ErrorKind` when the error is modeled as retryable
    ///
    /// If the error kind cannot be determined (eg. the error is unmodeled at the error kind depends
    /// on an HTTP status code, return `None`.
    fn retryable_error_kind(&self) -> Option<ErrorKind>;

    /// Returns the `code` for this error if one exists
    fn code(&self) -> Option<&str>;
}

/// `RetryKind` describes how a request MAY be retried for a given response
///
/// A `RetryKind` describes how a response MAY be retried; it does not mandate retry behavior.
/// The actual retry behavior is at the sole discretion of the RetryStrategy in place.
/// A RetryStrategy may ignore the suggestion for a number of reasons including but not limited to:
/// - Number of retry attempts exceeded
/// - The required retry delay exceeds the maximum backoff configured by the client
/// - No retry tokens are available due to service health
#[non_exhaustive]
#[derive(Eq, PartialEq, Debug)]
pub enum RetryKind {
    /// Retry the associated request due to a known `ErrorKind`.
    Error(ErrorKind),

    /// An Explicit retry (eg. from `x-amz-retry-after`).
    ///
    /// Note: The specified `Duration` is considered a suggestion and may be replaced or ignored.
    Explicit(Duration),

    /// The response associated with this variant should not be retried.
    NotRetryable,
}
