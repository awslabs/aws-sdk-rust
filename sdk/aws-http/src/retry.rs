/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
//! AWS-specific retry logic

use aws_smithy_http::result::SdkError;
use aws_smithy_http::retry::{ClassifyRetry, DefaultResponseRetryClassifier};
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind, RetryKind};
use std::time::Duration;

const TRANSIENT_ERROR_STATUS_CODES: &[u16] = &[500, 502, 503, 504];
const THROTTLING_ERRORS: &[&str] = &[
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
const TRANSIENT_ERRORS: &[&str] = &["RequestTimeout", "RequestTimeoutException"];

/// Implementation of [`ClassifyRetry`] that classifies AWS error codes.
///
/// In order of priority:
/// 1. The `x-amz-retry-after` header is checked
/// 2. The modeled error retry mode is checked
/// 3. The code is checked against a predetermined list of throttling errors & transient error codes
/// 4. The status code is checked against a predetermined list of status codes
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct AwsResponseRetryClassifier;

impl AwsResponseRetryClassifier {
    /// Create an `AwsResponseRetryClassifier` with the default set of known error & status codes
    pub fn new() -> Self {
        Self
    }
}

impl Default for AwsResponseRetryClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> ClassifyRetry<T, SdkError<E>> for AwsResponseRetryClassifier
where
    E: ProvideErrorKind,
{
    fn classify_retry(&self, result: Result<&T, &SdkError<E>>) -> RetryKind {
        // Run common retry classification logic from aws-smithy-http, and if it yields
        // a `RetryKind`, then return that immediately. Otherwise, continue on to run some
        // AWS SDK specific classification logic.
        let (err, response) = match DefaultResponseRetryClassifier::try_extract_err_response(result)
        {
            Ok(extracted) => extracted,
            Err(retry_kind) => return retry_kind,
        };
        if let Some(retry_after_delay) = response
            .http()
            .headers()
            .get("x-amz-retry-after")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.parse::<u64>().ok())
        {
            return RetryKind::Explicit(Duration::from_millis(retry_after_delay));
        }
        if let Some(kind) = err.retryable_error_kind() {
            return RetryKind::Error(kind);
        };
        if let Some(code) = err.code() {
            if THROTTLING_ERRORS.contains(&code) {
                return RetryKind::Error(ErrorKind::ThrottlingError);
            }
            if TRANSIENT_ERRORS.contains(&code) {
                return RetryKind::Error(ErrorKind::TransientError);
            }
        };
        if TRANSIENT_ERROR_STATUS_CODES.contains(&response.http().status().as_u16()) {
            return RetryKind::Error(ErrorKind::TransientError);
        };
        // TODO(https://github.com/awslabs/smithy-rs/issues/966): IDPCommuncation error needs to be retried
        RetryKind::UnretryableFailure
    }
}

#[cfg(test)]
mod test {
    use crate::retry::AwsResponseRetryClassifier;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::result::{SdkError, SdkSuccess};
    use aws_smithy_http::retry::ClassifyRetry;
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind, RetryKind};
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
        code: &'static str,
    }

    impl ProvideErrorKind for UnmodeledError {
        fn retryable_error_kind(&self) -> Option<ErrorKind> {
            None
        }

        fn code(&self) -> Option<&str> {
            None
        }
    }

    impl ProvideErrorKind for CodedError {
        fn retryable_error_kind(&self) -> Option<ErrorKind> {
            None
        }

        fn code(&self) -> Option<&str> {
            Some(self.code)
        }
    }

    fn make_err<E>(
        err: E,
        raw: http::Response<&'static str>,
    ) -> Result<SdkSuccess<()>, SdkError<E>> {
        Err(SdkError::service_error(
            err,
            operation::Response::new(raw.map(SdkBody::from)),
        ))
    }

    #[test]
    fn not_an_error() {
        let policy = AwsResponseRetryClassifier::new();
        let test_response = http::Response::new("OK");
        assert_eq!(
            policy.classify_retry(make_err(UnmodeledError, test_response).as_ref()),
            RetryKind::UnretryableFailure
        );
    }

    #[test]
    fn classify_by_response_status() {
        let policy = AwsResponseRetryClassifier::new();
        let test_resp = http::Response::builder()
            .status(500)
            .body("error!")
            .unwrap();
        assert_eq!(
            policy.classify_retry(make_err(UnmodeledError, test_resp).as_ref()),
            RetryKind::Error(ErrorKind::TransientError)
        );
    }

    #[test]
    fn classify_by_response_status_not_retryable() {
        let policy = AwsResponseRetryClassifier::new();
        let test_resp = http::Response::builder()
            .status(408)
            .body("error!")
            .unwrap();
        assert_eq!(
            policy.classify_retry(make_err(UnmodeledError, test_resp).as_ref()),
            RetryKind::UnretryableFailure
        );
    }

    #[test]
    fn classify_by_error_code() {
        let test_response = http::Response::new("OK");
        let policy = AwsResponseRetryClassifier::new();

        assert_eq!(
            policy.classify_retry(
                make_err(CodedError { code: "Throttling" }, test_response).as_ref()
            ),
            RetryKind::Error(ErrorKind::ThrottlingError)
        );

        let test_response = http::Response::new("OK");
        assert_eq!(
            policy.classify_retry(
                make_err(
                    CodedError {
                        code: "RequestTimeout"
                    },
                    test_response,
                )
                .as_ref()
            ),
            RetryKind::Error(ErrorKind::TransientError)
        )
    }

    #[test]
    fn classify_generic() {
        let err = aws_smithy_types::Error::builder().code("SlowDown").build();
        let test_response = http::Response::new("OK");
        let policy = AwsResponseRetryClassifier::new();
        assert_eq!(
            policy.classify_retry(make_err(err, test_response).as_ref()),
            RetryKind::Error(ErrorKind::ThrottlingError)
        );
    }

    #[test]
    fn classify_by_error_kind() {
        struct ModeledRetries;
        let test_response = http::Response::new("OK");
        impl ProvideErrorKind for ModeledRetries {
            fn retryable_error_kind(&self) -> Option<ErrorKind> {
                Some(ErrorKind::ClientError)
            }

            fn code(&self) -> Option<&str> {
                // code should not be called when `error_kind` is provided
                unimplemented!()
            }
        }

        let policy = AwsResponseRetryClassifier::new();

        assert_eq!(
            policy.classify_retry(make_err(ModeledRetries, test_response).as_ref()),
            RetryKind::Error(ErrorKind::ClientError)
        );
    }

    #[test]
    fn test_retry_after_header() {
        let policy = AwsResponseRetryClassifier::new();
        let test_response = http::Response::builder()
            .header("x-amz-retry-after", "5000")
            .body("retry later")
            .unwrap();

        assert_eq!(
            policy.classify_retry(make_err(UnmodeledError, test_response).as_ref()),
            RetryKind::Explicit(Duration::from_millis(5000))
        );
    }

    #[test]
    fn classify_response_error() {
        let policy = AwsResponseRetryClassifier::new();
        assert_eq!(
            policy.classify_retry(
                Result::<SdkSuccess<()>, SdkError<UnmodeledError>>::Err(SdkError::response_error(
                    UnmodeledError,
                    operation::Response::new(http::Response::new("OK").map(SdkBody::from)),
                ))
                .as_ref()
            ),
            RetryKind::Error(ErrorKind::TransientError)
        );
    }

    #[test]
    fn test_timeout_error() {
        let policy = AwsResponseRetryClassifier::new();
        let err: Result<(), SdkError<UnmodeledError>> = Err(SdkError::timeout_error("blah"));
        assert_eq!(
            policy.classify_retry(err.as_ref()),
            RetryKind::Error(ErrorKind::TransientError)
        );
    }
}
