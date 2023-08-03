/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Client request orchestration.
//!
//! The orchestrator handles the full request/response lifecycle including:
//! - Request serialization
//! - Endpoint resolution
//! - Identity resolution
//! - Signing
//! - Request transmission with retry and timeouts
//! - Response deserialization
//!
//! There are several hook points in the orchestration where [interceptors](crate::client::interceptors)
//! can read and modify the input, request, response, or output/error.

use crate::box_error::BoxError;
use crate::client::interceptors::context::phase::Phase;
use crate::client::interceptors::context::Error;
use crate::client::interceptors::InterceptorError;
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::{ConnectorError, SdkError};
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use bytes::Bytes;
use std::fmt::Debug;
use std::future::Future as StdFuture;
use std::pin::Pin;

/// Type alias for the HTTP request type that the orchestrator uses.
pub type HttpRequest = http::Request<SdkBody>;

/// Type alias for the HTTP response type that the orchestrator uses.
pub type HttpResponse = http::Response<SdkBody>;

/// Type alias for boxed futures that are returned from several traits since async trait functions are not stable yet (as of 2023-07-21).
///
/// See [the Rust blog](https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html) for
/// more information on async functions in traits.
pub type BoxFuture<T> = Pin<Box<dyn StdFuture<Output = Result<T, BoxError>> + Send>>;

/// Type alias for futures that are returned from several traits since async trait functions are not stable yet (as of 2023-07-21).
///
/// See [the Rust blog](https://blog.rust-lang.org/inside-rust/2023/05/03/stabilizing-async-fn-in-trait.html) for
/// more information on async functions in traits.
pub type Future<T> = NowOrLater<Result<T, BoxError>, BoxFuture<T>>;

/// Informs the orchestrator on whether or not the request body needs to be loaded into memory before transmit.
///
/// This enum gets placed into the `ConfigBag` to change the orchestrator behavior.
/// Immediately after serialization (before the `read_after_serialization` interceptor hook),
/// if it was set to `Requested` in the config bag, it will be replaced back into the config bag as
/// `Loaded` with the request body contents for use in later interceptors.
///
/// This all happens before the attempt loop, so the loaded request body will remain available
/// for interceptors that run in any subsequent retry attempts.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum LoadedRequestBody {
    /// Don't attempt to load the request body into memory.
    NotNeeded,
    /// Attempt to load the request body into memory.
    Requested,
    /// The request body is already loaded.
    Loaded(Bytes),
}

impl Storable for LoadedRequestBody {
    type Storer = StoreReplace<Self>;
}

#[derive(Debug)]
enum ErrorKind<E> {
    /// An error occurred within an interceptor.
    Interceptor { source: InterceptorError },
    /// An error returned by a service.
    Operation { err: E },
    /// An error that occurs when a request times out.
    Timeout { source: BoxError },
    /// An error that occurs when request dispatch fails.
    Connector { source: ConnectorError },
    /// An error that occurs when a response can't be deserialized.
    Response { source: BoxError },
    /// A general orchestrator error.
    Other { source: BoxError },
}

/// Errors that can occur while running the orchestrator.
#[derive(Debug)]
pub struct OrchestratorError<E> {
    kind: ErrorKind<E>,
}

impl<E> OrchestratorError<E> {
    /// Create a new `OrchestratorError` from the given source.
    pub fn other(source: impl Into<Box<dyn std::error::Error + Send + Sync + 'static>>) -> Self {
        Self {
            kind: ErrorKind::Other {
                source: source.into(),
            },
        }
    }

    /// Create an operation error.
    pub fn operation(err: E) -> Self {
        Self {
            kind: ErrorKind::Operation { err },
        }
    }

    /// True if the underlying error is an operation error.
    pub fn is_operation_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Operation { .. })
    }

    /// Return this orchestrator error as an operation error if possible.
    pub fn as_operation_error(&self) -> Option<&E> {
        match &self.kind {
            ErrorKind::Operation { err } => Some(err),
            _ => None,
        }
    }

    /// Create an interceptor error with the given source.
    pub fn interceptor(source: InterceptorError) -> Self {
        Self {
            kind: ErrorKind::Interceptor { source },
        }
    }

    /// True if the underlying error is an interceptor error.
    pub fn is_interceptor_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Interceptor { .. })
    }

    /// Create a timeout error with the given source.
    pub fn timeout(source: BoxError) -> Self {
        Self {
            kind: ErrorKind::Timeout { source },
        }
    }

    /// True if the underlying error is a timeout error.
    pub fn is_timeout_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Timeout { .. })
    }

    /// Create a response error with the given source.
    pub fn response(source: BoxError) -> Self {
        Self {
            kind: ErrorKind::Response { source },
        }
    }

    /// True if the underlying error is a response error.
    pub fn is_response_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Response { .. })
    }

    /// Create a connector error with the given source.
    pub fn connector(source: ConnectorError) -> Self {
        Self {
            kind: ErrorKind::Connector { source },
        }
    }

    /// True if the underlying error is a [`ConnectorError`].
    pub fn is_connector_error(&self) -> bool {
        matches!(self.kind, ErrorKind::Connector { .. })
    }

    /// Return this orchestrator error as a connector error if possible.
    pub fn as_connector_error(&self) -> Option<&ConnectorError> {
        match &self.kind {
            ErrorKind::Connector { source } => Some(source),
            _ => None,
        }
    }

    /// Convert the `OrchestratorError` into an [`SdkError`].
    pub(crate) fn into_sdk_error(
        self,
        phase: &Phase,
        response: Option<HttpResponse>,
    ) -> SdkError<E, HttpResponse> {
        match self.kind {
            ErrorKind::Interceptor { source } => {
                use Phase::*;
                match phase {
                    BeforeSerialization | Serialization => SdkError::construction_failure(source),
                    BeforeTransmit | Transmit => match response {
                        Some(response) => SdkError::response_error(source, response),
                        None => {
                            SdkError::dispatch_failure(ConnectorError::other(source.into(), None))
                        }
                    },
                    BeforeDeserialization | Deserialization | AfterDeserialization => {
                        SdkError::response_error(source, response.expect("phase has a response"))
                    }
                }
            }
            ErrorKind::Operation { err } => {
                debug_assert!(phase.is_after_deserialization(), "operation errors are a result of successfully receiving and parsing a response from the server. Therefore, we must be in the 'After Deserialization' phase.");
                SdkError::service_error(err, response.expect("phase has a response"))
            }
            ErrorKind::Connector { source } => SdkError::dispatch_failure(source),
            ErrorKind::Timeout { source } => SdkError::timeout_error(source),
            ErrorKind::Response { source } => SdkError::response_error(source, response.unwrap()),
            ErrorKind::Other { source } => {
                use Phase::*;
                match phase {
                    BeforeSerialization | Serialization => SdkError::construction_failure(source),
                    BeforeTransmit | Transmit => convert_dispatch_error(source, response),
                    BeforeDeserialization | Deserialization | AfterDeserialization => {
                        SdkError::response_error(source, response.expect("phase has a response"))
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

impl From<Error> for OrchestratorError<Error> {
    fn from(err: Error) -> Self {
        Self::operation(err)
    }
}
