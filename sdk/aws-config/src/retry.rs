/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Retry configuration

// Re-export from aws-smithy-types
pub use aws_smithy_types::retry::ErrorKind;
pub use aws_smithy_types::retry::ProvideErrorKind;
pub use aws_smithy_types::retry::RetryConfig;
pub use aws_smithy_types::retry::RetryConfigBuilder;
pub use aws_smithy_types::retry::RetryKind;
pub use aws_smithy_types::retry::RetryMode;

/// Errors for retry configuration
pub mod error {
    use std::borrow::Cow;
    use std::fmt;
    use std::num::ParseIntError;

    // Re-export from aws-smithy-types
    pub use aws_smithy_types::retry::RetryModeParseError;

    #[derive(Debug)]
    pub(crate) enum RetryConfigErrorKind {
        /// The configured retry mode wasn't recognized.
        InvalidRetryMode {
            /// Cause of the error.
            source: RetryModeParseError,
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
    }

    /// Failure to parse retry config from profile file or environment variable.
    #[derive(Debug)]
    pub struct RetryConfigError {
        pub(crate) kind: RetryConfigErrorKind,
    }

    impl fmt::Display for RetryConfigError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            use RetryConfigErrorKind::*;
            match &self.kind {
                InvalidRetryMode { set_by, .. } => {
                    write!(f, "invalid configuration set by {set_by}")
                }
                MaxAttemptsMustNotBeZero { set_by } => {
                    write!(f, "invalid configuration set by {set_by}: It is invalid to set max attempts to 0. Unset it or set it to an integer greater than or equal to one.")
                }
                FailedToParseMaxAttempts { set_by, .. } => {
                    write!(f, "failed to parse max attempts set by {set_by}",)
                }
            }
        }
    }

    impl std::error::Error for RetryConfigError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            use RetryConfigErrorKind::*;
            match &self.kind {
                InvalidRetryMode { source, .. } => Some(source),
                FailedToParseMaxAttempts { source, .. } => Some(source),
                MaxAttemptsMustNotBeZero { .. } => None,
            }
        }
    }

    impl From<RetryConfigErrorKind> for RetryConfigError {
        fn from(kind: RetryConfigErrorKind) -> Self {
            Self { kind }
        }
    }
}
