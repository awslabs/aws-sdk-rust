/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Generic errors for Smithy codegen

use crate::retry::{ErrorKind, ProvideErrorKind};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod display;

/// Generic Error type
///
/// For many services, Errors are modeled. However, many services only partially model errors or don't
/// model errors at all. In these cases, the SDK will return this generic error type to expose the
/// `code`, `message` and `request_id`.
#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct Error {
    code: Option<String>,
    message: Option<String>,
    request_id: Option<String>,
    extras: HashMap<&'static str, String>,
}

/// Builder for [`Error`].
#[derive(Debug, Default)]
pub struct Builder {
    inner: Error,
}

impl Builder {
    /// Sets the error message.
    pub fn message(&mut self, message: impl Into<String>) -> &mut Self {
        self.inner.message = Some(message.into());
        self
    }

    /// Sets the error code.
    pub fn code(&mut self, code: impl Into<String>) -> &mut Self {
        self.inner.code = Some(code.into());
        self
    }

    /// Sets the request ID the error happened for.
    pub fn request_id(&mut self, request_id: impl Into<String>) -> &mut Self {
        self.inner.request_id = Some(request_id.into());
        self
    }

    /// Set a custom field on the error metadata
    ///
    /// Typically, these will be accessed with an extension trait:
    /// ```rust
    /// use aws_smithy_types::Error;
    /// const HOST_ID: &str = "host_id";
    /// trait S3ErrorExt {
    ///     fn extended_request_id(&self) -> Option<&str>;
    /// }
    ///
    /// impl S3ErrorExt for Error {
    ///     fn extended_request_id(&self) -> Option<&str> {
    ///         self.extra(HOST_ID)
    ///     }
    /// }
    ///
    /// fn main() {
    ///     // Extension trait must be brought into scope
    ///     use S3ErrorExt;
    ///     let sdk_response: Result<(), Error> = Err(Error::builder().custom(HOST_ID, "x-1234").build());
    ///     if let Err(err) = sdk_response {
    ///         println!("request id: {:?}, extended request id: {:?}", err.request_id(), err.extended_request_id());
    ///     }
    /// }
    /// ```
    pub fn custom(&mut self, key: &'static str, value: impl Into<String>) -> &mut Self {
        self.inner.extras.insert(key, value.into());
        self
    }

    /// Creates the error.
    pub fn build(&mut self) -> Error {
        std::mem::take(&mut self.inner)
    }
}

impl Error {
    /// Returns the error code.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }
    /// Returns the error message.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    /// Returns the request ID the error occurred for, if it's available.
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
    /// Returns additional information about the error if it's present.
    pub fn extra(&self, key: &'static str) -> Option<&str> {
        self.extras.get(key).map(|k| k.as_str())
    }

    /// Creates an `Error` builder.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Converts an `Error` into a builder.
    pub fn into_builder(self) -> Builder {
        Builder { inner: self }
    }
}

impl ProvideErrorKind for Error {
    fn retryable_error_kind(&self) -> Option<ErrorKind> {
        None
    }

    fn code(&self) -> Option<&str> {
        Error::code(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_struct("Error");
        if let Some(code) = &self.code {
            fmt.field("code", code);
        }
        if let Some(message) = &self.message {
            fmt.field("message", message);
        }
        if let Some(req_id) = &self.request_id {
            fmt.field("request_id", req_id);
        }
        for (k, v) in &self.extras {
            fmt.field(k, &v);
        }
        fmt.finish()
    }
}

impl std::error::Error for Error {}
