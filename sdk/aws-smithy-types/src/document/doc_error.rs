/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Error types for [`Document`](super::Document) serialization and deserialization.

use std::fmt::{self, Debug, Display};

/// An error that occurred during [`Document`](super::Document) serialization
/// or deserialization.
pub struct DocError {
    err: Box<ErrorCode>,
}

enum ErrorCode {
    /// A custom error message produced by serde.
    Message(String),
    /// A map key was not a type that can be represented as a string.
    KeyMustBeAString,
    /// A float map key was NaN or infinite.
    FloatKeyMustBeFinite,
}

impl DocError {
    pub(crate) fn custom(msg: impl Display) -> Self {
        DocError {
            err: Box::new(ErrorCode::Message(msg.to_string())),
        }
    }

    pub(crate) fn key_must_be_a_string() -> Self {
        DocError {
            err: Box::new(ErrorCode::KeyMustBeAString),
        }
    }

    pub(crate) fn float_key_must_be_finite() -> Self {
        DocError {
            err: Box::new(ErrorCode::FloatKeyMustBeFinite),
        }
    }
}

impl Display for DocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.err {
            ErrorCode::Message(msg) => f.write_str(msg),
            ErrorCode::KeyMustBeAString => f.write_str("key must be a string"),
            ErrorCode::FloatKeyMustBeFinite => {
                f.write_str("float key must be finite (got NaN or +/-inf)")
            }
        }
    }
}

impl Debug for DocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DocError({:?})", self.to_string())
    }
}

impl std::error::Error for DocError {}

impl serde::ser::Error for DocError {
    fn custom<T: Display>(msg: T) -> Self {
        DocError::custom(msg)
    }
}

impl serde::de::Error for DocError {
    fn custom<T: Display>(msg: T) -> Self {
        DocError::custom(msg)
    }
}
