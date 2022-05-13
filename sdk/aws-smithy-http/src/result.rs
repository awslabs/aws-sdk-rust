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

/// Failed SDK Result
#[derive(Debug)]
pub enum SdkError<E, R = operation::Response> {
    /// The request failed during construction. It was not dispatched over the network.
    ConstructionFailure(BoxError),

    /// The request failed due to a timeout. The request MAY have been sent and received.
    TimeoutError(BoxError),

    /// The request failed during dispatch. An HTTP response was not received. The request MAY
    /// have been sent.
    DispatchFailure(ConnectorError),

    /// A response was received but it was not parseable according the the protocol (for example
    /// the server hung up while the body was being read)
    ResponseError {
        /// Error encountered while parsing the response
        err: BoxError,
        /// Raw response that was available
        raw: R,
    },

    /// An error response was received from the service
    ServiceError {
        /// Modeled service error
        err: E,
        /// Raw response from the service
        raw: R,
    },
}

/// Error from the underlying Connector
///
/// Connector exists to attach a `ConnectorErrorKind` to what would otherwise be an opaque `Box<dyn Error>`
/// that comes off a potentially generic or dynamic connector.
/// The attached `kind` is used to determine what retry behavior should occur (if any) based on the
/// connector error.
#[derive(Debug)]
pub struct ConnectorError {
    err: BoxError,
    kind: ConnectorErrorKind,
}

impl Display for ConnectorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.err)
    }
}

impl Error for ConnectorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.err.as_ref())
    }
}

impl ConnectorError {
    /// Construct a [`ConnectorError`] from an error caused by a timeout
    ///
    /// Timeout errors are typically retried on a new connection.
    pub fn timeout(err: BoxError) -> Self {
        Self {
            err,
            kind: ConnectorErrorKind::Timeout,
        }
    }

    /// Construct a [`ConnectorError`] from an error caused by the user (e.g. invalid HTTP request)
    pub fn user(err: BoxError) -> Self {
        Self {
            err,
            kind: ConnectorErrorKind::User,
        }
    }

    /// Construct a [`ConnectorError`] from an IO related error (e.g. socket hangup)
    pub fn io(err: BoxError) -> Self {
        Self {
            err,
            kind: ConnectorErrorKind::Io,
        }
    }

    /// Construct a [`ConnectorError`] from an different unclassified error.
    ///
    /// Optionally, an explicit `Kind` may be passed.
    pub fn other(err: BoxError, kind: Option<ErrorKind>) -> Self {
        Self {
            err,
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

    /// Returns true if the error is a user error
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

#[derive(Debug)]
enum ConnectorErrorKind {
    /// A timeout occurred while processing the request
    Timeout,

    /// A user-caused error (e.g. invalid HTTP request)
    User,

    /// Socket/IO error
    Io,

    /// An unclassified Error with an explicit error kind
    Other(Option<ErrorKind>),
}

impl Display for ConnectorErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConnectorErrorKind::Timeout => write!(f, "timeout"),
            ConnectorErrorKind::User => write!(f, "user error"),
            ConnectorErrorKind::Io => write!(f, "io error"),
            ConnectorErrorKind::Other(Some(kind)) => write!(f, "{:?}", kind),
            ConnectorErrorKind::Other(None) => write!(f, "other"),
        }
    }
}

impl<E, R> Display for SdkError<E, R>
where
    E: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::ConstructionFailure(err) => write!(f, "failed to construct request: {}", err),
            SdkError::TimeoutError(err) => write!(f, "request has timed out: {}", err),
            SdkError::DispatchFailure(err) => Display::fmt(&err, f),
            SdkError::ResponseError { err, .. } => Display::fmt(&err, f),
            SdkError::ServiceError { err, .. } => Display::fmt(&err, f),
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
            ConstructionFailure(err) | TimeoutError(err) | ResponseError { err, .. } => {
                Some(err.as_ref())
            }
            DispatchFailure(err) => Some(err),
            ServiceError { err, .. } => Some(err),
        }
    }
}
