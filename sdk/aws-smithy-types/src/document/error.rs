/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Error type for [`Document`](super::Document)-shaped coercion
//! operations.
//!
//! `DocumentError` is returned by the format-aware coercion accessors on
//! [`DiscriminatedDocument`](super::DiscriminatedDocument)
//! ([`as_blob`](super::DiscriminatedDocument::as_blob),
//! [`as_timestamp`](super::DiscriminatedDocument::as_timestamp)) and by
//! the [`DocumentSettings`](super::DocumentSettings) coercion hooks they
//! dispatch through.
//!
//! [`Document`](super::Document) itself has no fallible accessors: it is
//! exactly the released type, whose variant-testing accessors
//! (`Document::as_string`, `Document::as_object`, etc.) return
//! `Option<_>` rather than `Result<_, DocumentError>` — there is no
//! error condition for "this isn't the variant you asked for" beyond
//! the absent value itself. Schema-driven numeric and
//! arbitrary-precision coercion lives in `aws-smithy-schema`'s
//! `DocumentShapeDeserializer` and reports through its `SerdeError`.
//!
//! `DocumentError` is intentionally narrower than the schema crate's
//! `aws_smithy_schema::serde::SerdeError`. `DocumentError` covers
//! Document-shaped failures only (type-mismatch on a coercion, numeric
//! overflow, malformed numeric text). The schema crate's `SerdeError`
//! covers the broader set of shape-serde concerns (missing members,
//! unknown members, write failures, etc.) and lifts `DocumentError`
//! via `From` where their concerns overlap.

use std::fmt;

/// Error returned by [`Document`](super::Document)-shaped coercion
/// operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum DocumentError {
    /// The document's variant didn't match the type the caller asked
    /// for, and no coercion is defined between the actual and the
    /// requested type.
    ///
    /// Example: calling
    /// [`DiscriminatedDocument::as_blob`](super::DiscriminatedDocument::as_blob)
    /// on a `Document::Number(_)`.
    #[non_exhaustive]
    TypeMismatch {
        /// Description of what was expected vs. what was found.
        message: String,
    },
    /// A numeric coercion overflowed the target type's representable
    /// range.
    ///
    /// Emitted when a source value is outside the target's
    /// `[min, max]` range.
    #[non_exhaustive]
    NumericCoercionOverflow {
        /// Target type name (e.g. `"byte"`, `"integer"`, `"long"`).
        target: String,
        /// String representation of the overflowing value, included
        /// for diagnostics.
        value: String,
    },
    /// The document's value couldn't be parsed into the requested
    /// representation. Distinct from `TypeMismatch`: the variants
    /// match, but the underlying string is malformed.
    ///
    /// Example: a [`Document::String`](super::Document::String) whose
    /// text isn't valid base64 when coercing to a blob.
    #[non_exhaustive]
    InvalidInput {
        /// Description of the problem.
        message: String,
    },
    /// Catch-all for errors not covered by other variants.
    #[non_exhaustive]
    Custom {
        /// Explanatory message.
        message: String,
    },
    /// The operation is not supported on this document. Used by
    /// [`DiscriminatedDocument`](super::DiscriminatedDocument)'s
    /// format-aware coercion accessors, and by
    /// [`DocumentSettings`](super::DocumentSettings) trait default
    /// methods for a coercion a particular protocol doesn't support.
    #[non_exhaustive]
    UnsupportedOperation {
        /// Description of which operation isn't supported and why.
        message: String,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentError::TypeMismatch { message } => write!(f, "type mismatch: {message}"),
            DocumentError::NumericCoercionOverflow { target, value } => {
                write!(f, "numeric value {value} out of range for {target}")
            }
            DocumentError::InvalidInput { message } => write!(f, "invalid input: {message}"),
            DocumentError::Custom { message } => f.write_str(message),
            DocumentError::UnsupportedOperation { message } => {
                write!(f, "unsupported operation: {message}")
            }
        }
    }
}

impl std::error::Error for DocumentError {}

impl DocumentError {
    /// Creates a `TypeMismatch` error with the given message describing
    /// what was expected versus what was found.
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        DocumentError::TypeMismatch {
            message: message.into(),
        }
    }

    /// Creates a `NumericCoercionOverflow` error for a `value` that is
    /// outside the representable range of `target` (e.g. `"byte"`).
    pub fn numeric_coercion_overflow(target: impl Into<String>, value: impl Into<String>) -> Self {
        DocumentError::NumericCoercionOverflow {
            target: target.into(),
            value: value.into(),
        }
    }

    /// Creates an `InvalidInput` error with the given message.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        DocumentError::InvalidInput {
            message: message.into(),
        }
    }

    /// Creates a custom error with the given message.
    pub fn custom(message: impl Into<String>) -> Self {
        DocumentError::Custom {
            message: message.into(),
        }
    }

    /// Creates an `UnsupportedOperation` error with the given message.
    pub fn unsupported(message: impl Into<String>) -> Self {
        DocumentError::UnsupportedOperation {
            message: message.into(),
        }
    }
}
