/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::result::SdkError;
use aws_smithy_runtime_api::client::retries::RetryReason;
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};
use std::borrow::Cow;

/// A retry classifier for checking if an error is modeled as retryable.
#[derive(Debug)]
pub struct ModeledAsRetryableClassifier;

impl ModeledAsRetryableClassifier {
    /// Check if an error is modeled as retryable, returning a [`RetryReason::Error`] if it is.
    pub fn classify_error<E: ProvideErrorKind, R>(
        &self,
        error: &SdkError<E, R>,
    ) -> Option<RetryReason> {
        match error {
            SdkError::ServiceError(inner) => {
                inner.err().retryable_error_kind().map(RetryReason::Error)
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct SmithyErrorClassifier;

impl SmithyErrorClassifier {
    pub fn classify_error<E, R>(&self, result: &SdkError<E, R>) -> Option<RetryReason> {
        match result {
            SdkError::TimeoutError(_err) => Some(RetryReason::Error(ErrorKind::TransientError)),
            SdkError::ResponseError { .. } => Some(RetryReason::Error(ErrorKind::TransientError)),
            SdkError::DispatchFailure(err) if (err.is_timeout() || err.is_io()) => {
                Some(RetryReason::Error(ErrorKind::TransientError))
            }
            SdkError::DispatchFailure(err) => err.is_other().map(RetryReason::Error),
            _ => None,
        }
    }
}

const TRANSIENT_ERROR_STATUS_CODES: &[u16] = &[500, 502, 503, 504];

/// A retry classifier that will treat HTTP response with those status codes as retryable.
/// The `Default` version will retry 500, 502, 503, and 504 errors.
#[derive(Debug)]
pub struct HttpStatusCodeClassifier {
    retryable_status_codes: Cow<'static, [u16]>,
}

impl HttpStatusCodeClassifier {
    /// Given a `Vec<u16>` where the `u16`s represent status codes, create a retry classifier that will
    /// treat HTTP response with those status codes as retryable. The `Default` version will retry
    /// 500, 502, 503, and 504 errors.
    pub fn new_from_codes(retryable_status_codes: impl Into<Cow<'static, [u16]>>) -> Self {
        Self {
            retryable_status_codes: retryable_status_codes.into(),
        }
    }

    /// Classify an HTTP response based on its status code.
    pub fn classify_error<E>(&self, error: &SdkError<E>) -> Option<RetryReason> {
        error
            .raw_response()
            .map(|res| res.http().status().as_u16())
            .map(|status| self.retryable_status_codes.contains(&status))
            .unwrap_or_default()
            .then_some(RetryReason::Error(ErrorKind::TransientError))
    }
}

impl Default for HttpStatusCodeClassifier {
    fn default() -> Self {
        Self::new_from_codes(TRANSIENT_ERROR_STATUS_CODES.to_owned())
    }
}

// Generic smithy clients would have something like this:
// pub fn default_retry_classifiers() -> RetryClassifiers {
//     RetryClassifiers::new()
//         .with_classifier(SmithyErrorClassifier::new())
//         .with_classifier(ModeledAsRetryableClassifier::new())
//         .with_classifier(HttpStatusCodeClassifier::new())
// }
// This ordering is different than the default AWS ordering because the old generic client classifer
// was the same.

#[cfg(test)]
mod test {
    use std::fmt;

    use crate::client::retries::classifier::{
        HttpStatusCodeClassifier, ModeledAsRetryableClassifier,
    };
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation;
    use aws_smithy_http::result::SdkError;
    use aws_smithy_runtime_api::client::retries::RetryReason;
    use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind};

    use super::SmithyErrorClassifier;

    #[derive(Debug)]
    struct UnmodeledError;

    impl fmt::Display for UnmodeledError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "UnmodeledError")
        }
    }

    impl std::error::Error for UnmodeledError {}

    #[test]
    fn classify_by_response_status() {
        let policy = HttpStatusCodeClassifier::default();
        let res = http::Response::builder()
            .status(500)
            .body("error!")
            .unwrap()
            .map(SdkBody::from);
        let res = operation::Response::new(res);
        let err = SdkError::service_error(UnmodeledError, res);
        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::TransientError))
        );
    }

    #[test]
    fn classify_by_response_status_not_retryable() {
        let policy = HttpStatusCodeClassifier::default();
        let res = http::Response::builder()
            .status(408)
            .body("error!")
            .unwrap()
            .map(SdkBody::from);
        let res = operation::Response::new(res);
        let err = SdkError::service_error(UnmodeledError, res);

        assert_eq!(policy.classify_error(&err), None);
    }

    #[test]
    fn classify_by_error_kind() {
        struct ModeledRetries;

        impl ProvideErrorKind for ModeledRetries {
            fn retryable_error_kind(&self) -> Option<ErrorKind> {
                Some(ErrorKind::ClientError)
            }

            fn code(&self) -> Option<&str> {
                // code should not be called when `error_kind` is provided
                unimplemented!()
            }
        }

        let policy = ModeledAsRetryableClassifier;
        let res = http::Response::new("OK");
        let err = SdkError::service_error(ModeledRetries, res);

        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::ClientError)),
        );
    }

    #[test]
    fn classify_response_error() {
        let policy = SmithyErrorClassifier;
        let test_response = http::Response::new("OK").map(SdkBody::from);
        let err: SdkError<UnmodeledError> =
            SdkError::response_error(UnmodeledError, operation::Response::new(test_response));
        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::TransientError)),
        );
    }

    #[test]
    fn test_timeout_error() {
        let policy = SmithyErrorClassifier;
        let err: SdkError<UnmodeledError, ()> = SdkError::timeout_error("blah");
        assert_eq!(
            policy.classify_error(&err),
            Some(RetryReason::Error(ErrorKind::TransientError)),
        );
    }
}
