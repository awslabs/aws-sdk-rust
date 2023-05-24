/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::http::HttpHeaders;
use aws_smithy_http::result::SdkError;
use aws_smithy_runtime_api::client::retries::RetryReason;
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ErrorKind;

/// AWS error codes that represent throttling errors.
pub const THROTTLING_ERRORS: &[&str] = &[
    "Throttling",
    "ThrottlingException",
    "ThrottledException",
    "RequestThrottledException",
    "TooManyRequestsException",
    "ProvisionedThroughputExceededException",
    "TransactionInProgressException",
    "RequestLimitExceeded",
    "BandwidthLimitExceeded",
    "LimitExceededException",
    "RequestThrottled",
    "SlowDown",
    "PriorRequestNotComplete",
    "EC2ThrottledException",
];

/// AWS error codes that represent transient errors.
pub const TRANSIENT_ERRORS: &[&str] = &["RequestTimeout", "RequestTimeoutException"];

/// A retry classifier for determining if the response sent by an AWS service requires a retry.
#[derive(Debug)]
pub struct AwsErrorCodeClassifier;

impl AwsErrorCodeClassifier {
    /// Classify an error code to check if represents a retryable error. The codes of retryable
    /// errors are defined [here](THROTTLING_ERRORS) and [here](TRANSIENT_ERRORS).
    pub fn classify_error<E: ProvideErrorMetadata, R>(
        &self,
        error: &SdkError<E, R>,
    ) -> Option<RetryReason> {
        if let Some(error_code) = error.code() {
            if THROTTLING_ERRORS.contains(&error_code) {
                return Some(RetryReason::Error(ErrorKind::ThrottlingError));
            } else if TRANSIENT_ERRORS.contains(&error_code) {
                return Some(RetryReason::Error(ErrorKind::TransientError));
            }
        };

        None
    }
}

/// A retry classifier that checks for `x-amz-retry-after` headers. If one is found, a
/// [`RetryReason::Explicit`] is returned containing the duration to wait before retrying.
#[derive(Debug)]
pub struct AmzRetryAfterHeaderClassifier;

impl AmzRetryAfterHeaderClassifier {
    /// Classify an AWS responses error code to determine how (and if) it should be retried.
    pub fn classify_error<E>(&self, error: &SdkError<E>) -> Option<RetryReason> {
        error
            .raw_response()
            .and_then(|res| res.http_headers().get("x-amz-retry-after"))
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.parse::<u64>().ok())
            .map(|retry_after_delay| {
                RetryReason::Explicit(std::time::Duration::from_millis(retry_after_delay))
            })
    }
}

#[cfg(test)]
mod test {
    use super::{AmzRetryAfterHeaderClassifier, AwsErrorCodeClassifier};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::result::SdkError;
    use aws_smithy_runtime_api::client::retries::RetryReason;
    use aws_smithy_types::error::metadata::ProvideErrorMetadata;
    use aws_smithy_types::error::ErrorMetadata;
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
    use std::fmt;
    use std::time::Duration;

    #[derive(Debug)]
    struct UnmodeledError;

    impl fmt::Display for UnmodeledError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "UnmodeledError")
        }
    }

    impl std::error::Error for UnmodeledError {}

    struct CodedError {
        metadata: ErrorMetadata,
    }

    impl CodedError {
        fn new(code: &'static str) -> Self {
            Self {
                metadata: ErrorMetadata::builder().code(code).build(),
            }
        }
    }

    impl ProvideErrorKind for UnmodeledError {
        fn retryable_error_kind(&self) -> Option<ErrorKind> {
            None
        }

        fn code(&self) -> Option<&str> {
            None
        }
    }

    impl ProvideErrorMetadata for CodedError {
        fn meta(&self) -> &ErrorMetadata {
            &self.metadata
        }
    }

    #[test]
    fn classify_by_error_code() {
        let policy = AwsErrorCodeClassifier;
        let res = http::Response::new("OK");
        let err = SdkError::service_error(CodedError::new("Throttling"), res);

        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::ThrottlingError))
        );

        let res = http::Response::new("OK");
        let err = SdkError::service_error(CodedError::new("RequestTimeout"), res);
        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::TransientError))
        )
    }

    #[test]
    fn classify_generic() {
        let policy = AwsErrorCodeClassifier;
        let res = http::Response::new("OK");
        let err = aws_smithy_types::Error::builder().code("SlowDown").build();
        let err = SdkError::service_error(err, res);
        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::ThrottlingError))
        );
    }

    #[test]
    fn test_retry_after_header() {
        let policy = AmzRetryAfterHeaderClassifier;
        let res = http::Response::builder()
            .header("x-amz-retry-after", "5000")
            .body("retry later")
            .unwrap()
            .map(SdkBody::from);
        let res = operation::Response::new(res);
        let err = SdkError::service_error(UnmodeledError, res);

        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Explicit(Duration::from_millis(5000))),
        );
    }
}
