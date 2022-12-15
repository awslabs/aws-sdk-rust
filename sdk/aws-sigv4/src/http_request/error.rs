/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use http::header::{InvalidHeaderName, InvalidHeaderValue};
use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

#[derive(Debug)]
enum SigningErrorKind {
    FailedToCreateCanonicalRequest { source: CanonicalRequestError },
}

/// Error signing request
#[derive(Debug)]
pub struct SigningError {
    kind: SigningErrorKind,
}

impl fmt::Display for SigningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            SigningErrorKind::FailedToCreateCanonicalRequest { .. } => {
                write!(f, "failed to create canonical request")
            }
        }
    }
}

impl Error for SigningError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            SigningErrorKind::FailedToCreateCanonicalRequest { source } => Some(source),
        }
    }
}

impl From<CanonicalRequestError> for SigningError {
    fn from(source: CanonicalRequestError) -> Self {
        Self {
            kind: SigningErrorKind::FailedToCreateCanonicalRequest { source },
        }
    }
}

#[derive(Debug)]
enum CanonicalRequestErrorKind {
    InvalidHeaderName { source: InvalidHeaderName },
    InvalidHeaderValue { source: InvalidHeaderValue },
    InvalidUtf8InHeaderValue { source: Utf8Error },
}

#[derive(Debug)]
pub(crate) struct CanonicalRequestError {
    kind: CanonicalRequestErrorKind,
}

impl fmt::Display for CanonicalRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CanonicalRequestErrorKind::*;
        match self.kind {
            InvalidHeaderName { .. } => write!(f, "invalid header name"),
            InvalidHeaderValue { .. } => write!(f, "invalid header value"),
            InvalidUtf8InHeaderValue { .. } => write!(f, "invalid UTF-8 in header value"),
        }
    }
}

impl Error for CanonicalRequestError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use CanonicalRequestErrorKind::*;
        match &self.kind {
            InvalidHeaderName { source } => Some(source),
            InvalidHeaderValue { source } => Some(source),
            InvalidUtf8InHeaderValue { source } => Some(source),
        }
    }
}

impl CanonicalRequestError {
    pub(crate) fn invalid_utf8_in_header_value(source: Utf8Error) -> Self {
        Self {
            kind: CanonicalRequestErrorKind::InvalidUtf8InHeaderValue { source },
        }
    }
}

impl From<InvalidHeaderName> for CanonicalRequestError {
    fn from(source: InvalidHeaderName) -> Self {
        Self {
            kind: CanonicalRequestErrorKind::InvalidHeaderName { source },
        }
    }
}

impl From<InvalidHeaderValue> for CanonicalRequestError {
    fn from(source: InvalidHeaderValue) -> Self {
        Self {
            kind: CanonicalRequestErrorKind::InvalidHeaderValue { source },
        }
    }
}
