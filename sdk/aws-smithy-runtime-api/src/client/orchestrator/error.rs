/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::BoxError;
use crate::client::interceptors::context::phase::Phase;
use crate::client::interceptors::InterceptorError;
use crate::client::orchestrator::HttpResponse;
use aws_smithy_http::result::{ConnectorError, SdkError};
use aws_smithy_types::type_erasure::TypeErasedError;
use std::fmt::Debug;

#[derive(Debug)]
#[non_exhaustive]
pub enum OrchestratorError<E> {
    /// An error occurred within an interceptor.
    Interceptor { err: InterceptorError },
    /// An error returned by a service.
    Operation { err: E },
    /// An error that occurs when a request times out.
    Timeout { err: BoxError },
    /// An error that occurs when request dispatch fails.
    Connector { err: ConnectorError },
    /// An error that occurs when a response can't be deserialized.
    Response { err: BoxError },
    /// A general orchestrator error.
    Other { err: BoxError },
}

impl<E: Debug> OrchestratorError<E> {
    /// Create a new `OrchestratorError` from a [`BoxError`].
    pub fn other(err: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>) -> Self {
        let err = err.into();
        Self::Other { err }
    }

    /// Create a new `OrchestratorError` from an error received from a service.
    pub fn operation(err: E) -> Self {
        Self::Operation { err }
    }

    /// Create a new `OrchestratorError::Interceptor` from an [`InterceptorError`].
    pub fn interceptor(err: InterceptorError) -> Self {
        Self::Interceptor { err }
    }

    /// Create a new `OrchestratorError::Timeout` from a [`BoxError`].
    pub fn timeout(err: BoxError) -> Self {
        Self::Timeout { err }
    }

    /// Create a new `OrchestratorError::Response` from a [`BoxError`].
    pub fn response(err: BoxError) -> Self {
        Self::Response { err }
    }

    /// Create a new `OrchestratorError::Connector` from a [`ConnectorError`].
    pub fn connector(err: ConnectorError) -> Self {
        Self::Connector { err }
    }

    /// Convert the `OrchestratorError` into `Some` operation specific error if it is one. Otherwise,
    /// return `None`.
    pub fn as_operation_error(&self) -> Option<&E> {
        match self {
            Self::Operation { err } => Some(err),
            _ => None,
        }
    }

    /// Convert the `OrchestratorError` into an [`SdkError`].
    pub(crate) fn into_sdk_error(
        self,
        phase: &Phase,
        response: Option<HttpResponse>,
    ) -> SdkError<E, HttpResponse> {
        match self {
            Self::Interceptor { err } => {
                use Phase::*;
                match phase {
                    BeforeSerialization | Serialization => SdkError::construction_failure(err),
                    BeforeTransmit | Transmit => match response {
                        Some(response) => SdkError::response_error(err, response),
                        None => SdkError::dispatch_failure(ConnectorError::other(err.into(), None)),
                    },
                    BeforeDeserialization | Deserialization | AfterDeserialization => {
                        SdkError::response_error(err, response.expect("phase has a response"))
                    }
                }
            }
            Self::Operation { err } => {
                debug_assert!(phase.is_after_deserialization(), "operation errors are a result of successfully receiving and parsing a response from the server. Therefore, we must be in the 'After Deserialization' phase.");
                SdkError::service_error(err, response.expect("phase has a response"))
            }
            Self::Connector { err } => SdkError::dispatch_failure(err),
            Self::Timeout { err } => SdkError::timeout_error(err),
            Self::Response { err } => SdkError::response_error(err, response.unwrap()),
            Self::Other { err } => {
                use Phase::*;
                match phase {
                    BeforeSerialization | Serialization => SdkError::construction_failure(err),
                    BeforeTransmit | Transmit => convert_dispatch_error(err, response),
                    BeforeDeserialization | Deserialization | AfterDeserialization => {
                        SdkError::response_error(err, response.expect("phase has a response"))
                    }
                }
            }
        }
    }
}

fn convert_dispatch_error<O>(
    err: BoxError,
    response: Option<HttpResponse>,
) -> SdkError<O, HttpResponse> {
    let err = match err.downcast::<ConnectorError>() {
        Ok(connector_error) => {
            return SdkError::dispatch_failure(*connector_error);
        }
        Err(e) => e,
    };
    match response {
        Some(response) => SdkError::response_error(err, response),
        None => SdkError::dispatch_failure(ConnectorError::other(err, None)),
    }
}

impl<E> From<InterceptorError> for OrchestratorError<E>
where
    E: Debug + std::error::Error + 'static,
{
    fn from(err: InterceptorError) -> Self {
        Self::interceptor(err)
    }
}

impl From<TypeErasedError> for OrchestratorError<TypeErasedError> {
    fn from(err: TypeErasedError) -> Self {
        Self::operation(err)
    }
}

impl<E> From<aws_smithy_http::byte_stream::error::Error> for OrchestratorError<E>
where
    E: Debug + std::error::Error + 'static,
{
    fn from(err: aws_smithy_http::byte_stream::error::Error) -> Self {
        Self::other(err)
    }
}
