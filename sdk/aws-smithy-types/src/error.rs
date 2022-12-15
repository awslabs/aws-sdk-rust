/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Generic errors for Smithy codegen

use crate::retry::{ErrorKind, ProvideErrorKind};
use std::collections::HashMap;
use std::fmt;

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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Debug)]
pub(super) enum TryFromNumberErrorKind {
    /// Used when the conversion from an integer type into a smaller integer type would be lossy.
    OutsideIntegerRange(std::num::TryFromIntError),
    /// Used when the conversion from an `u64` into a floating point type would be lossy.
    U64ToFloatLossyConversion(u64),
    /// Used when the conversion from an `i64` into a floating point type would be lossy.
    I64ToFloatLossyConversion(i64),
    /// Used when attempting to convert an `f64` into an `f32`.
    F64ToF32LossyConversion(f64),
    /// Used when attempting to convert a decimal, infinite, or `NaN` floating point type into an
    /// integer type.
    FloatToIntegerLossyConversion(f64),
    /// Used when attempting to convert a negative [`Number`](crate::Number) into an unsigned integer type.
    NegativeToUnsignedLossyConversion(i64),
}

/// The error type returned when conversion into an integer type or floating point type is lossy.
#[derive(Debug)]
pub struct TryFromNumberError {
    pub(super) kind: TryFromNumberErrorKind,
}

impl fmt::Display for TryFromNumberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use TryFromNumberErrorKind::*;
        match self.kind {
            OutsideIntegerRange(_) => write!(f, "integer too large"),
            FloatToIntegerLossyConversion(v) => write!(
                f,
                "cannot convert floating point number {v} into an integer"
            ),
            NegativeToUnsignedLossyConversion(v) => write!(
                f,
                "cannot convert negative integer {v} into an unsigned integer type"
            ),
            U64ToFloatLossyConversion(v) => {
                write!(
                    f,
                    "cannot convert {v}u64 into a floating point type without precision loss"
                )
            }
            I64ToFloatLossyConversion(v) => {
                write!(
                    f,
                    "cannot convert {v}i64 into a floating point type without precision loss"
                )
            }
            F64ToF32LossyConversion(v) => {
                write!(f, "will not attempt to convert {v}f64 into a f32")
            }
        }
    }
}

impl std::error::Error for TryFromNumberError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use TryFromNumberErrorKind::*;
        match &self.kind {
            OutsideIntegerRange(err) => Some(err as _),
            FloatToIntegerLossyConversion(_)
            | NegativeToUnsignedLossyConversion(_)
            | U64ToFloatLossyConversion(_)
            | I64ToFloatLossyConversion(_)
            | F64ToF32LossyConversion(_) => None,
        }
    }
}

impl From<std::num::TryFromIntError> for TryFromNumberError {
    fn from(value: std::num::TryFromIntError) -> Self {
        Self {
            kind: TryFromNumberErrorKind::OutsideIntegerRange(value),
        }
    }
}

impl From<TryFromNumberErrorKind> for TryFromNumberError {
    fn from(kind: TryFromNumberErrorKind) -> Self {
        Self { kind }
    }
}
