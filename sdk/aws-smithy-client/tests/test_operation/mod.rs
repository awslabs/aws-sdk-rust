/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::operation;
use aws_smithy_http::response::ParseHttpResponse;
use aws_smithy_http::result::SdkError;
use aws_smithy_http::retry::ClassifyRetry;
use aws_smithy_types::retry::{ErrorKind, ProvideErrorKind, RetryKind};
use bytes::Bytes;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::str;

#[derive(Clone)]
pub(super) struct TestOperationParser;

#[derive(Debug)]
pub(super) struct OperationError(ErrorKind);

impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for OperationError {}

impl ProvideErrorKind for OperationError {
    fn retryable_error_kind(&self) -> Option<ErrorKind> {
        Some(self.0)
    }

    fn code(&self) -> Option<&str> {
        None
    }
}

impl ParseHttpResponse for TestOperationParser {
    type Output = Result<String, OperationError>;

    fn parse_unloaded(&self, response: &mut operation::Response) -> Option<Self::Output> {
        tracing::debug!("got response: {:?}", response);
        match response.http().status() {
            s if s.is_success() => None,
            s if s.is_client_error() => Some(Err(OperationError(ErrorKind::ServerError))),
            s if s.is_server_error() => Some(Err(OperationError(ErrorKind::TransientError))),
            _ => panic!("unexpected status: {}", response.http().status()),
        }
    }

    fn parse_loaded(&self, response: &http::Response<Bytes>) -> Self::Output {
        Ok(str::from_utf8(response.body().as_ref())
            .unwrap()
            .to_string())
    }
}

#[derive(Clone)]
pub(super) struct TestRetryClassifier;

impl<T, E> ClassifyRetry<T, SdkError<E>> for TestRetryClassifier
where
    E: ProvideErrorKind + Debug,
    T: Debug,
{
    fn classify_retry(&self, err: Result<&T, &SdkError<E>>) -> RetryKind {
        tracing::info!("got response: {:?}", err);
        let kind = match err {
            Err(SdkError::ServiceError(context)) => context.err().retryable_error_kind(),
            Err(SdkError::DispatchFailure(err)) if err.is_timeout() => {
                Some(ErrorKind::TransientError)
            }
            Err(SdkError::TimeoutError(_)) => Some(ErrorKind::TransientError),
            Ok(_) => return RetryKind::Unnecessary,
            _ => panic!("test handler only handles modeled errors got: {:?}", err),
        };
        match kind {
            Some(kind) => RetryKind::Error(kind),
            None => RetryKind::UnretryableFailure,
        }
    }
}
