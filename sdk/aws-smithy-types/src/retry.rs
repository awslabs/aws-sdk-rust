/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! This module defines types that describe when to retry given a response.

use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Duration;

/// Type of error that occurred when making a request.
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

/// Trait that provides an `ErrorKind` and an error code.
pub trait ProvideErrorKind {
    /// Returns the `ErrorKind` when the error is modeled as retryable
    ///
    /// If the error kind cannot be determined (e.g. the error is unmodeled at the error kind depends
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

    /// An Explicit retry (e.g. from `x-amz-retry-after`).
    ///
    /// Note: The specified `Duration` is considered a suggestion and may be replaced or ignored.
    Explicit(Duration),

    /// The response was a failure that should _not_ be retried.
    UnretryableFailure,

    /// The response was successful, so no retry is necessary.
    Unnecessary,
}

/// Specifies how failed requests should be retried.
#[non_exhaustive]
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum RetryMode {
    /// The standard set of retry rules across AWS SDKs. This mode includes a standard set of errors
    /// that are retried, and support for retry quotas. The default maximum number of attempts
    /// with this mode is three, unless otherwise explicitly configured with [`RetryConfig`].
    Standard,

    /// An experimental retry mode that includes the functionality of standard mode but includes
    /// automatic client-side throttling. Because this mode is experimental, it might change
    /// behavior in the future.
    Adaptive,
}

const VALID_RETRY_MODES: &[RetryMode] = &[RetryMode::Standard];

/// Failure to parse a `RetryMode` from string.
#[derive(Debug)]
pub struct RetryModeParseErr(String);

impl Display for RetryModeParseErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error parsing string '{}' as RetryMode, valid options are: {:#?}",
            self.0, VALID_RETRY_MODES
        )
    }
}

impl std::error::Error for RetryModeParseErr {}

impl FromStr for RetryMode {
    type Err = RetryModeParseErr;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let string = string.trim();
        // eq_ignore_ascii_case is OK here because the only strings we need to check for are ASCII
        if string.eq_ignore_ascii_case("standard") {
            Ok(RetryMode::Standard)
        // TODO(https://github.com/awslabs/aws-sdk-rust/issues/247): adaptive retries
        // } else if string.eq_ignore_ascii_case("adaptive") {
        //     Ok(RetryMode::Adaptive)
        } else {
            Err(RetryModeParseErr(string.to_owned()))
        }
    }
}

/// Builder for [`RetryConfig`].
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RetryConfigBuilder {
    mode: Option<RetryMode>,
    max_attempts: Option<u32>,
}

impl RetryConfigBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the retry mode.
    pub fn set_mode(&mut self, retry_mode: Option<RetryMode>) -> &mut Self {
        self.mode = retry_mode;
        self
    }

    /// Sets the max attempts. This value must be greater than zero.
    pub fn set_max_attempts(&mut self, max_attempts: Option<u32>) -> &mut Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Sets the retry mode.
    pub fn mode(mut self, mode: RetryMode) -> Self {
        self.set_mode(Some(mode));
        self
    }

    /// Sets the max attempts. This value must be greater than zero.
    pub fn max_attempts(mut self, max_attempts: u32) -> Self {
        self.set_max_attempts(Some(max_attempts));
        self
    }

    /// Merge two builders together. Values from `other` will only be used as a fallback for values
    /// from `self` Useful for merging configs from different sources together when you want to
    /// handle "precedence" per value instead of at the config level
    ///
    /// # Example
    ///
    /// ```rust
    /// # use aws_smithy_types::retry::{RetryMode, RetryConfigBuilder};
    /// let a = RetryConfigBuilder::new().max_attempts(1);
    /// let b = RetryConfigBuilder::new().max_attempts(5).mode(RetryMode::Adaptive);
    /// let retry_config = a.take_unset_from(b).build();
    /// // A's value take precedence over B's value
    /// assert_eq!(retry_config.max_attempts(), 1);
    /// // A never set a retry mode so B's value was used
    /// assert_eq!(retry_config.mode(), RetryMode::Adaptive);
    /// ```
    pub fn take_unset_from(self, other: Self) -> Self {
        Self {
            mode: self.mode.or(other.mode),
            max_attempts: self.max_attempts.or(other.max_attempts),
        }
    }

    /// Builds a `RetryConfig`.
    pub fn build(self) -> RetryConfig {
        RetryConfig {
            mode: self.mode.unwrap_or(RetryMode::Standard),
            max_attempts: self.max_attempts.unwrap_or(3),
        }
    }
}

/// Retry configuration for requests.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub struct RetryConfig {
    mode: RetryMode,
    max_attempts: u32,
}

impl RetryConfig {
    /// Creates a default `RetryConfig` with `RetryMode::Standard` and max attempts of three.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a `RetryConfig` that has retries disabled.
    pub fn disabled() -> Self {
        Self::default().with_max_attempts(1)
    }

    /// Changes the retry mode.
    pub fn with_retry_mode(mut self, retry_mode: RetryMode) -> Self {
        self.mode = retry_mode;
        self
    }

    /// Changes the max attempts. This value must be greater than zero.
    pub fn with_max_attempts(mut self, max_attempts: u32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    /// Returns the retry mode.
    pub fn mode(&self) -> RetryMode {
        self.mode
    }

    /// Returns the max attempts.
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            mode: RetryMode::Standard,
            max_attempts: 3,
        }
    }
}

/// Failure to parse retry config from profile file or environment variable.
#[non_exhaustive]
#[derive(Debug)]
pub enum RetryConfigErr {
    /// The configured retry mode wasn't recognized.
    InvalidRetryMode {
        /// Cause of the error.
        source: RetryModeParseErr,
        /// Where the invalid retry mode value originated from.
        set_by: Cow<'static, str>,
    },
    /// Max attempts must be greater than zero.
    MaxAttemptsMustNotBeZero {
        /// Where the invalid max attempts value originated from.
        set_by: Cow<'static, str>,
    },
    /// The max attempts value couldn't be parsed to an integer.
    FailedToParseMaxAttempts {
        /// Cause of the error.
        source: ParseIntError,
        /// Where the invalid max attempts value originated from.
        set_by: Cow<'static, str>,
    },
    /// The adaptive retry mode hasn't been implemented yet.
    AdaptiveModeIsNotSupported {
        /// Where the invalid retry mode value originated from.
        set_by: Cow<'static, str>,
    },
}

impl Display for RetryConfigErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use RetryConfigErr::*;
        match self {
            InvalidRetryMode { set_by, source } => {
                write!(f, "invalid configuration set by {}: {}", set_by, source)
            }
            MaxAttemptsMustNotBeZero { set_by } => {
                write!(f, "invalid configuration set by {}: It is invalid to set max attempts to 0. Unset it or set it to an integer greater than or equal to one.", set_by)
            }
            FailedToParseMaxAttempts { set_by, source } => {
                write!(
                    f,
                    "failed to parse max attempts set by {}: {}",
                    set_by, source
                )
            }
            AdaptiveModeIsNotSupported { set_by } => {
                write!(f, "invalid configuration set by {}: Setting retry mode to 'adaptive' is not yet supported. Unset it or set it to 'standard' mode.", set_by)
            }
        }
    }
}

impl std::error::Error for RetryConfigErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use RetryConfigErr::*;
        match self {
            InvalidRetryMode { source, .. } => Some(source),
            FailedToParseMaxAttempts { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::retry::{RetryConfigBuilder, RetryMode};
    use std::str::FromStr;

    #[test]
    fn retry_config_builder_merge_with_favors_self_values_over_other_values() {
        let self_builder = RetryConfigBuilder::new()
            .max_attempts(1)
            .mode(RetryMode::Adaptive);
        let other_builder = RetryConfigBuilder::new()
            .max_attempts(5)
            .mode(RetryMode::Standard);
        let retry_config = self_builder.take_unset_from(other_builder).build();

        assert_eq!(retry_config.max_attempts, 1);
        assert_eq!(retry_config.mode, RetryMode::Adaptive);
    }

    #[test]
    fn retry_mode_from_str_parses_valid_strings_regardless_of_casing() {
        assert_eq!(
            RetryMode::from_str("standard").ok(),
            Some(RetryMode::Standard)
        );
        assert_eq!(
            RetryMode::from_str("STANDARD").ok(),
            Some(RetryMode::Standard)
        );
        assert_eq!(
            RetryMode::from_str("StAnDaRd").ok(),
            Some(RetryMode::Standard)
        );
        // assert_eq!(
        //     RetryMode::from_str("adaptive").ok(),
        //     Some(RetryMode::Adaptive)
        // );
        // assert_eq!(
        //     RetryMode::from_str("ADAPTIVE").ok(),
        //     Some(RetryMode::Adaptive)
        // );
        // assert_eq!(
        //     RetryMode::from_str("aDaPtIvE").ok(),
        //     Some(RetryMode::Adaptive)
        // );
    }

    #[test]
    fn retry_mode_from_str_ignores_whitespace_before_and_after() {
        assert_eq!(
            RetryMode::from_str("  standard ").ok(),
            Some(RetryMode::Standard)
        );
        assert_eq!(
            RetryMode::from_str("   STANDARD  ").ok(),
            Some(RetryMode::Standard)
        );
        assert_eq!(
            RetryMode::from_str("  StAnDaRd   ").ok(),
            Some(RetryMode::Standard)
        );
        // assert_eq!(
        //     RetryMode::from_str("  adaptive  ").ok(),
        //     Some(RetryMode::Adaptive)
        // );
        // assert_eq!(
        //     RetryMode::from_str("   ADAPTIVE ").ok(),
        //     Some(RetryMode::Adaptive)
        // );
        // assert_eq!(
        //     RetryMode::from_str("  aDaPtIvE    ").ok(),
        //     Some(RetryMode::Adaptive)
        // );
    }

    #[test]
    fn retry_mode_from_str_wont_parse_invalid_strings() {
        assert_eq!(RetryMode::from_str("std").ok(), None);
        assert_eq!(RetryMode::from_str("aws").ok(), None);
        assert_eq!(RetryMode::from_str("s t a n d a r d").ok(), None);
        assert_eq!(RetryMode::from_str("a d a p t i v e").ok(), None);
    }
}
