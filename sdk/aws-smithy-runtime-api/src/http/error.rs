/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Error types for HTTP requests/responses.

use crate::box_error::BoxError;
use http_02x::header::{InvalidHeaderName, InvalidHeaderValue};
use http_02x::uri::InvalidUri;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::Utf8Error;

#[derive(Debug)]
/// An error occurred constructing an Http Request.
///
/// This is normally due to configuration issues, internal SDK bugs, or other user error.
pub struct HttpError(BoxError);

impl HttpError {
    // TODO(httpRefactor): Add better error internals
    pub(super) fn new<E: Into<Box<dyn Error + Send + Sync + 'static>>>(err: E) -> Self {
        HttpError(err.into())
    }

    #[allow(dead_code)]
    pub(super) fn invalid_extensions() -> Self {
        Self("Extensions were provided during initialization. This prevents the request format from being converted.".into())
    }

    pub(super) fn invalid_header_value(err: InvalidHeaderValue) -> Self {
        Self(err.into())
    }

    pub(super) fn header_was_not_a_string(err: Utf8Error) -> Self {
        Self(err.into())
    }

    pub(super) fn invalid_header_name(err: InvalidHeaderName) -> Self {
        Self(err.into())
    }

    pub(super) fn invalid_uri(err: InvalidUri) -> Self {
        Self(err.into())
    }

    pub(super) fn invalid_status_code() -> Self {
        Self("invalid HTTP status code".into())
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "an error occurred creating an HTTP Request")
    }
}

impl Error for HttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}
