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
use crate::client::interceptors::InterceptorError;
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::{ConnectorError, SdkError};
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use aws_smithy_types::type_erasure::TypeErasedError;
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

// TODO(enableNewSmithyRuntimeLaunch): Make OrchestratorError adhere to the errors RFC
/// Errors that can occur while running the orchestrator.
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
