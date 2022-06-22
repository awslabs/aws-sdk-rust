/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
//! AWS-specific retry logic

use aws_smithy_http::result::SdkError;
use aws_smithy_http::retry::ClassifyResponse;
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind, RetryKind};
use std::time::Duration;

/// A retry policy that models AWS error codes as outlined in the SEP
///
/// In order of priority:
/// 1. The `x-amz-retry-after` header is checked
/// 2. The modeled error retry mode is checked
/// 3. The code is checked against a predetermined list of throttling errors & transient error codes
/// 4. The status code is checked against a predetermined list of status codes
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct AwsErrorRetryPolicy;

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

impl AwsErrorRetryPolicy {
    /// Create an `AwsErrorRetryPolicy` with the default set of known error & status codes
    pub fn new() -> Self {
        AwsErrorRetryPolicy
    }
}

impl Default for AwsErrorRetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, E> ClassifyResponse<T, SdkError<E>> for AwsErrorRetryPolicy
where
    E: ProvideErrorKind,
{
    fn classify(&self, err: Result<&T, &SdkError<E>>) -> RetryKind {
        let (err, response) = match err {
            Ok(_) => return RetryKind::Unnecessary,
            Err(SdkError::ServiceError { err, raw }) => (err, raw),
            Err(SdkError::TimeoutError(_err)) => {
                return RetryKind::Error(ErrorKind::TransientError)
            }

            Err(SdkError::DispatchFailure(err)) => {
                return if err.is_timeout() || err.is_io() {
                    RetryKind::Error(ErrorKind::TransientError)
                } else if let Some(ek) = err.is_other() {
                    RetryKind::Error(ek)
                } else {
                    RetryKind::UnretryableFailure
                };
            }
            Err(_) => return RetryKind::UnretryableFailure,
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
    use crate::retry::AwsErrorRetryPolicy;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::result::{SdkError, SdkSuccess};
    use aws_smithy_http::retry::ClassifyResponse;
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind, RetryKind};
    use std::time::Duration;

    struct UnmodeledError;

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
        Err(SdkError::ServiceError {
            err,
            raw: operation::Response::new(raw.map(|b| SdkBody::from(b))),
        })
    }

    #[test]
    fn not_an_error() {
        let policy = AwsErrorRetryPolicy::new();
        let test_response = http::Response::new("OK");
        assert_eq!(
            policy.classify(make_err(UnmodeledError, test_response).as_ref()),
            RetryKind::UnretryableFailure
        );
    }

    #[test]
    fn classify_by_response_status() {
        let policy = AwsErrorRetryPolicy::new();
        let test_resp = http::Response::builder()
            .status(500)
            .body("error!")
            .unwrap();
        assert_eq!(
            policy.classify(make_err(UnmodeledError, test_resp).as_ref()),
            RetryKind::Error(ErrorKind::TransientError)
        );
    }

    #[test]
    fn classify_by_response_status_not_retryable() {
        let policy = AwsErrorRetryPolicy::new();
        let test_resp = http::Response::builder()
            .status(408)
            .body("error!")
            .unwrap();
        assert_eq!(
            policy.classify(make_err(UnmodeledError, test_resp).as_ref()),
            RetryKind::UnretryableFailure
        );
    }

    #[test]
    fn classify_by_error_code() {
        let test_response = http::Response::new("OK");
        let policy = AwsErrorRetryPolicy::new();

        assert_eq!(
            policy.classify(make_err(CodedError { code: "Throttling" }, test_response).as_ref()),
            RetryKind::Error(ErrorKind::ThrottlingError)
        );

        let test_response = http::Response::new("OK");
        assert_eq!(
            policy.classify(
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
        let policy = AwsErrorRetryPolicy::new();
        assert_eq!(
            policy.classify(make_err(err, test_response).as_ref()),
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

        let policy = AwsErrorRetryPolicy::new();

        assert_eq!(
            policy.classify(make_err(ModeledRetries, test_response).as_ref()),
            RetryKind::Error(ErrorKind::ClientError)
        );
    }

    #[test]
    fn test_retry_after_header() {
        let policy = AwsErrorRetryPolicy::new();
        let test_response = http::Response::builder()
            .header("x-amz-retry-after", "5000")
            .body("retry later")
            .unwrap();

        assert_eq!(
            policy.classify(make_err(UnmodeledError, test_response).as_ref()),
            RetryKind::Explicit(Duration::from_millis(5000))
        );
    }

    #[test]
    fn test_timeout_error() {
        let policy = AwsErrorRetryPolicy::new();
        let err: Result<(), SdkError<UnmodeledError>> = Err(SdkError::TimeoutError("blah".into()));
        assert_eq!(
            policy.classify(err.as_ref()),
            RetryKind::Error(ErrorKind::TransientError)
        );
    }
}
