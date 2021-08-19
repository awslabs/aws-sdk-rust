/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::operation;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

type BoxError = Box<dyn Error + Send + Sync>;

/// Successful SDK Result
#[derive(Debug)]
pub struct SdkSuccess<O> {
    pub raw: operation::Response,
    pub parsed: O,
}

/// Failed SDK Result
#[derive(Debug)]
pub enum SdkError<E> {
    /// The request failed during construction. It was not dispatched over the network.
    ConstructionFailure(BoxError),

    /// The request failed during dispatch. An HTTP response was not received. The request MAY
    /// have been sent.
    DispatchFailure(BoxError),

    /// A response was received but it was not parseable according the the protocol (for example
    /// the server hung up while the body was being read)
    ResponseError {
        raw: operation::Response,
        err: BoxError,
    },

    /// An error response was received from the service
    ServiceError { err: E, raw: operation::Response },
}

impl<E> Display for SdkError<E>
where
    E: Error,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SdkError::ConstructionFailure(err) => write!(f, "failed to construct request: {}", err),
            SdkError::DispatchFailure(err) => Display::fmt(&err, f),
            SdkError::ResponseError { err, .. } => Display::fmt(&err, f),
            SdkError::ServiceError { err, .. } => Display::fmt(&err, f),
        }
    }
}

impl<E> Error for SdkError<E>
where
    E: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SdkError::ConstructionFailure(err)
            | SdkError::DispatchFailure(err)
            | SdkError::ResponseError { err, .. } => Some(err.as_ref()),
            SdkError::ServiceError { err, .. } => Some(err),
        }
    }
}
