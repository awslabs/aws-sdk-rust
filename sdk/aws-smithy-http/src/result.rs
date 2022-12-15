/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    missing_debug_implementations,
    missing_docs,
    rustdoc::all,
    unreachable_pub
)]

//! `Result` wrapper types for [success](SdkSuccess) and [failure](SdkError) responses.

use crate::operation;
use aws_smithy_types::retry::ErrorKind;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

type BoxError = Box<dyn Error + Send + Sync>;

/// Successful SDK Result
#[derive(Debug)]
pub struct SdkSuccess<O> {
    /// Raw Response from the service. (e.g. Http Response)
    pub raw: operation::Response,

    /// Parsed response from the service
    pub parsed: O,
}

/// Error context for [`SdkError::ConstructionFailure`]
#[derive(Debug)]
pub struct ConstructionFailure {
    source: BoxError,
}

/// Error context for [`SdkError::TimeoutError`]
#[derive(Debug)]
pub struct TimeoutError {
    source: BoxError,
}

/// Error context for [`SdkError::DispatchFailure`]
#[derive(Debug)]
pub struct DispatchFailure {
    source: ConnectorError,
}

impl DispatchFailure {
    /// Returns true if the error is an IO error
    pub fn is_io(&self) -> bool {
        self.source.is_io()
    }

    /// Returns true if the error is an timeout error
    pub fn is_timeout(&self) -> bool {
        self.source.is_timeout()
    }

    /// Returns true if the error is a user-caused error (e.g., invalid HTTP request)
    pub fn is_user(&self) -> bool {
        self.source.is_user()
    }

    /// Returns the optional error kind associated with an unclassified error
    pub fn is_other(&self) -> Option<ErrorKind> {
        self.source.is_other()
    }
}

/// Error context for [`SdkError::ResponseError`]
#[derive(Debug)]
pub struct ResponseError<R> {
    /// Error encountered while parsing the response
    source: BoxError,
    /// Raw response that was available
    raw: R,
}

impl<R> ResponseError<R> {
    /// Returns a reference to the raw response
    pub fn raw(&self) -> &R {
        &self.raw
    }

    /// Converts this error context into the raw response
    pub fn into_raw(self) -> R {
        self.raw
    }
}

/// Error context for [`SdkError::ServiceError`]
#[derive(Debug)]
pub struct ServiceError<E, R> {
    /// Modeled service error
    source: E,
    /// Raw response from the service
    raw: R,
}

impl<E, R> ServiceError<E, R> {
    /// Returns the underlying error of type `E`
    pub fn err(&self) -> &E {
        &self.source
    }

    /// Converts this error context into the underlying error `E`
    pub fn into_err(self) -> E {
        self.source
    }

    /// Returns a reference to the raw response
    pub fn raw(&self) -> &R {
        &self.raw
    }

    /// Converts this error context into the raw response
    pub fn into_raw(self) -> R {
        self.raw
    }
}

/// Constructs the unhandled variant of a code generated error.
///
/// This trait exists so that [`SdkError::into_service_error`] can be infallible.
pub trait CreateUnhandledError {
    /// Creates an unhandled error variant with the given `source`.
    fn create_unhandled_error(source: Box<dyn Error + Send + Sync + 'static>) -> Self;
}

/// Failed SDK Result
///
/// When logging an error from the SDK, it is recommended that you either wrap the error in
/// [`DisplayErrorContext`](aws_smithy_types::error::display::DisplayErrorContext), use another
/// error reporter library that visits the error's cause/source chain, or call
/// [`Error::source`](std::error::Error::source) for more details about the underlying cause.
#[non_exhaustive]
#[derive(Debug)]
pub enum SdkError<E, R = operation::Response> {
    /// The request failed during construction. It was not dispatched over the network.
    ConstructionFailure(ConstructionFailure),

    /// The request failed due to a timeout. The request MAY have been sent and received.
    TimeoutError(TimeoutError),

    /// The request failed during dispatch. An HTTP response was not received. The request MAY
    /// have been sent.
    DispatchFailure(DispatchFailure),

    /// A response was received but it was not parseable according the the protocol (for example
    /// the server hung up while the body was being read)
    ResponseError(ResponseError<R>),

    /// An error response was received from the service
    ServiceError(ServiceError<E, R>),
}

impl<E, R> SdkError<E, R> {
    /// Construct a `SdkError` for a construction failure
    pub fn construction_failure(source: impl Into<BoxError>) -> Self {
        Self::ConstructionFailure(ConstructionFailure {
            source: source.into(),
        })
    }

    /// Construct a `SdkError` for a timeout
    pub fn timeout_error(source: impl Into<BoxError>) -> Self {
        Self::TimeoutError(TimeoutError {
            source: source.into(),
        })
    }

    /// Construct a `SdkError` for a dispatch failure with a [`ConnectorError`]
    pub fn dispatch_failure(source: ConnectorError) -> Self {
        Self::DispatchFailure(DispatchFailure { source })
    }

    /// Construct a `SdkError` for a response error
    pub fn response_error(source: impl Into<BoxError>, raw: R) -> Self {
        Self::ResponseError(ResponseError {
            source: source.into(),
            raw,
        })
    }

    /// Construct a `SdkError` for a service failure
    pub fn service_error(source: E, raw: R) -> Self {
        Self::ServiceError(ServiceError { source, raw })
    }

    /// Returns the underlying service error `E` if there is one
    ///
    /// If the `SdkError` is not a `ServiceError` (for example, the error is a network timeout),
    /// then it will be converted into an unhandled variant of `E`. This makes it easy to match
    /// on the service's error response while simultaneously bubbling up transient failures.
    /// For example, handling the `NoSuchKey` error for S3's `GetObject` operation may look as
    /// follows:
    ///
    /// ```no_run
    /// # use aws_smithy_http::result::{SdkError, CreateUnhandledError};
    /// # #[derive(Debug)] enum GetObjectErrorKind { NoSuchKey(()), Other(()) }
    /// # #[derive(Debug)] struct GetObjectError { kind: GetObjectErrorKind }
    /// # impl std::fmt::Display for GetObjectError {
    /// #     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { unimplemented!() }
    /// # }
    /// # impl std::error::Error for GetObjectError {}
    /// # impl CreateUnhandledError for GetObjectError {
    /// #     fn create_unhandled_error(_: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self { unimplemented!() }
    /// # }
    /// # fn example() -> Result<(), GetObjectError> {
    /// # let sdk_err = SdkError::service_error(GetObjectError { kind: GetObjectErrorKind::NoSuchKey(()) }, ());
    /// match sdk_err.into_service_error() {
    ///     GetObjectError { kind: GetObjectErrorKind::NoSuchKey(_) } => {
    ///         // handle NoSuchKey
    ///     }
    ///     err @ _ => return Err(err),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_service_error(self) -> E
    where
        E: std::error::Error + Send + Sync + CreateUnhandledError + 'static,
        R: Debug + Send + Sync + 'static,
    {
        match self {
            Self::ServiceError(context) => context.source,
            _ => E::create_unhandled_error(self.into()),
        }
    }

    /// Converts this error into its error source.
    ///
    /// If there is no error source, then `Err(Self)` is returned.
    pub fn into_source(self) -> Result<Box<dyn Error + Send + Sync + 'static>, Self>
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        use SdkError::*;
        match self {
            ConstructionFailure(context) => Ok(context.source),
            TimeoutError(context) => Ok(context.source),
            ResponseError(context) => Ok(context.source),
            DispatchFailure(context) => Ok(context.source.into()),
            ServiceError(context) => Ok(context.source.into()),
        }
    }
}

impl<E, R> Display for SdkError<E, R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::ConstructionFailure(_) => write!(f, "failed to construct request"),
            SdkError::TimeoutError(_) => write!(f, "request has timed out"),
            SdkError::DispatchFailure(_) => write!(f, "dispatch failure"),
            SdkError::ResponseError(_) => write!(f, "response error"),
            SdkError::ServiceError(_) => write!(f, "service error"),
        }
    }
}

impl<E, R> Error for SdkError<E, R>
where
    E: Error + 'static,
    R: Debug,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use SdkError::*;
        match self {
            ConstructionFailure(context) => Some(context.source.as_ref()),
            TimeoutError(context) => Some(context.source.as_ref()),
            ResponseError(context) => Some(context.source.as_ref()),
            DispatchFailure(context) => Some(&context.source),
            ServiceError(context) => Some(&context.source),
        }
    }
}

#[derive(Debug)]
enum ConnectorErrorKind {
    /// A timeout occurred while processing the request
    Timeout,

    /// A user-caused error (e.g., invalid HTTP request)
    User,

    /// Socket/IO error
    Io,

    /// An unclassified Error with an explicit error kind
    Other(Option<ErrorKind>),
}

/// Error from the underlying Connector
///
/// Connector exists to attach a `ConnectorErrorKind` to what would otherwise be an opaque `Box<dyn Error>`
/// that comes off a potentially generic or dynamic connector.
/// The attached `kind` is used to determine what retry behavior should occur (if any) based on the
/// connector error.
#[derive(Debug)]
pub struct ConnectorError {
    kind: ConnectorErrorKind,
    source: BoxError,
}

impl Display for ConnectorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ConnectorErrorKind::Timeout => write!(f, "timeout"),
            ConnectorErrorKind::User => write!(f, "user error"),
            ConnectorErrorKind::Io => write!(f, "io error"),
            ConnectorErrorKind::Other(_) => write!(f, "other"),
        }
    }
}

impl Error for ConnectorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl ConnectorError {
    /// Construct a [`ConnectorError`] from an error caused by a timeout
    ///
    /// Timeout errors are typically retried on a new connection.
    pub fn timeout(source: BoxError) -> Self {
        Self {
            kind: ConnectorErrorKind::Timeout,
            source,
        }
    }

    /// Construct a [`ConnectorError`] from an error caused by the user (e.g. invalid HTTP request)
    pub fn user(source: BoxError) -> Self {
        Self {
            kind: ConnectorErrorKind::User,
            source,
        }
    }

    /// Construct a [`ConnectorError`] from an IO related error (e.g. socket hangup)
    pub fn io(source: BoxError) -> Self {
        Self {
            kind: ConnectorErrorKind::Io,
            source,
        }
    }

    /// Construct a [`ConnectorError`] from an different unclassified error.
    ///
    /// Optionally, an explicit `Kind` may be passed.
    pub fn other(source: BoxError, kind: Option<ErrorKind>) -> Self {
        Self {
            source,
            kind: ConnectorErrorKind::Other(kind),
        }
    }

    /// Returns true if the error is an IO error
    pub fn is_io(&self) -> bool {
        matches!(self.kind, ConnectorErrorKind::Io)
    }

    /// Returns true if the error is an timeout error
    pub fn is_timeout(&self) -> bool {
        matches!(self.kind, ConnectorErrorKind::Timeout)
    }

    /// Returns true if the error is a user-caused error (e.g., invalid HTTP request)
    pub fn is_user(&self) -> bool {
        matches!(self.kind, ConnectorErrorKind::User)
    }

    /// Returns the optional error kind associated with an unclassified error
    pub fn is_other(&self) -> Option<ErrorKind> {
        match &self.kind {
            ConnectorErrorKind::Other(ek) => *ek,
            _ => None,
        }
    }
}
