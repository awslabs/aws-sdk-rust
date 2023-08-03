/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::http::HttpHeaders;
use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
use aws_smithy_runtime_api::client::retries::{ClassifyRetry, RetryReason};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ErrorKind;
use std::error::Error as StdError;
use std::marker::PhantomData;

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
#[derive(Debug, Default)]
pub struct AwsErrorCodeClassifier<E> {
    _inner: PhantomData<E>,
}

impl<E> AwsErrorCodeClassifier<E> {
    /// Create a new AwsErrorCodeClassifier
    pub fn new() -> Self {
        Self {
            _inner: PhantomData,
        }
    }
}

impl<E> ClassifyRetry for AwsErrorCodeClassifier<E>
where
    E: StdError + ProvideErrorMetadata + Send + Sync + 'static,
{
    fn classify_retry(&self, ctx: &InterceptorContext) -> Option<RetryReason> {
        let error = ctx
            .output_or_error()?
            .err()
            .and_then(OrchestratorError::as_operation_error)?
            .downcast_ref::<E>()?;

        if let Some(error_code) = error.code() {
            if THROTTLING_ERRORS.contains(&error_code) {
                return Some(RetryReason::Error(ErrorKind::ThrottlingError));
            } else if TRANSIENT_ERRORS.contains(&error_code) {
                return Some(RetryReason::Error(ErrorKind::TransientError));
            }
        };

        None
    }

    fn name(&self) -> &'static str {
        "AWS Error Code"
    }
}

/// A retry classifier that checks for `x-amz-retry-after` headers. If one is found, a
/// [`RetryReason::Explicit`] is returned containing the duration to wait before retrying.
#[derive(Debug, Default)]
pub struct AmzRetryAfterHeaderClassifier;

impl AmzRetryAfterHeaderClassifier {
    /// Create a new `AmzRetryAfterHeaderClassifier`.
    pub fn new() -> Self {
        Self
    }
}

impl ClassifyRetry for AmzRetryAfterHeaderClassifier {
    fn classify_retry(&self, ctx: &InterceptorContext) -> Option<RetryReason> {
        ctx.response()
            .and_then(|res| res.http_headers().get("x-amz-retry-after"))
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.parse::<u64>().ok())
            .map(|retry_after_delay| {
                RetryReason::Explicit(std::time::Duration::from_millis(retry_after_delay))
            })
    }

    fn name(&self) -> &'static str {
        "'Retry After' Header"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::interceptors::context::{Error, Input};
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

    impl ProvideErrorKind for UnmodeledError {
        fn retryable_error_kind(&self) -> Option<ErrorKind> {
            None
        }

        fn code(&self) -> Option<&str> {
            None
        }
    }

    #[derive(Debug)]
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

    impl fmt::Display for CodedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Coded Error")
        }
    }

    impl std::error::Error for CodedError {}

    impl ProvideErrorMetadata for CodedError {
        fn meta(&self) -> &ErrorMetadata {
            &self.metadata
        }
    }

    #[test]
    fn classify_by_error_code() {
        let policy = AwsErrorCodeClassifier::<CodedError>::new();
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
            CodedError::new("Throttling"),
        ))));

        assert_eq!(
            policy.classify_retry(&ctx),
            Some(RetryReason::Error(ErrorKind::ThrottlingError))
        );

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
            CodedError::new("RequestTimeout"),
        ))));
        assert_eq!(
            policy.classify_retry(&ctx),
            Some(RetryReason::Error(ErrorKind::TransientError))
        )
    }

    #[test]
    fn classify_generic() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let err = aws_smithy_types::Error::builder().code("SlowDown").build();
        let test_response = http::Response::new("OK").map(SdkBody::from);

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(test_response);
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));

        assert_eq!(
            policy.classify_retry(&ctx),
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
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(res);
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
            UnmodeledError,
        ))));

        assert_eq!(
            policy.classify_retry(&ctx),
            Some(RetryReason::Explicit(Duration::from_millis(5000))),
        );
    }
}
