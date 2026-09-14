/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Error types for shape serialization and deserialization.

use std::fmt;

/// Error type for shape serialization and deserialization operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum SerdeError {
    /// The data did not match the expected type described by the schema.
    #[non_exhaustive]
    TypeMismatch {
        /// Description of what was expected vs what was found.
        message: String,
    },
    /// A required structure member was missing during deserialization.
    ///
    /// **Not currently emitted.** The schema-serde deserializers use the
    /// push/consumer pattern and defer required-member enforcement to the
    /// generated builder's `build()`. This variant is reserved for
    /// deserializers that choose to enforce member presence during the read.
    #[non_exhaustive]
    MissingMember {
        /// The name of the missing member.
        member_name: String,
    },
    /// An unknown member was encountered during deserialization.
    #[non_exhaustive]
    UnknownMember {
        /// The name of the unknown member.
        member_name: String,
    },
    /// The input data was malformed or invalid for the format.
    #[non_exhaustive]
    InvalidInput {
        /// Description of the problem.
        message: String,
    },
    /// The operation is not supported by this serializer or deserializer.
    #[non_exhaustive]
    UnsupportedOperation {
        /// Description of what was attempted.
        message: String,
    },
    /// An error occurred while writing output during serialization.
    #[non_exhaustive]
    WriteFailed {
        /// Description of the write failure.
        message: String,
    },
    /// A numeric coercion overflowed the target type's representable
    /// range.
    ///
    /// Emitted by [`Document::as_byte`](aws_smithy_types::Document::as_byte)
    /// (and the other narrow numeric accessors) when the source value
    /// is outside the target's `[min, max]` range.
    #[non_exhaustive]
    NumericCoercionOverflow {
        /// Target type name (e.g. `"byte"`, `"integer"`, `"long"`).
        target: String,
        /// String representation of the overflowing value, included
        /// for diagnostics.
        value: String,
    },
    /// A numeric coercion would lose precision in a context where
    /// precision loss is not acceptable.
    ///
    /// **Not currently emitted by `Document::as_*` accessors.** Per the
    /// SEP "Number coercion" rules, precision loss is silently ignored
    /// on the standard Document accessor path. This variant is
    /// reserved for callers (e.g. strict deserializers, customer code
    /// using a custom accessor) that choose to enforce lossless
    /// coercion.
    #[non_exhaustive]
    NumericCoercionLossy {
        /// Target type name.
        target: String,
        /// String representation of the value that would lose precision.
        value: String,
    },
    /// Failed to decode a blob from its wire-format representation —
    /// for example, invalid base64 in a JSON payload.
    ///
    /// **Not currently emitted.** In-tree codecs surface blob-decode
    /// failures as [`InvalidInput`](Self::InvalidInput). This variant is
    /// reserved for codecs that want to distinguish blob-decode failures as a
    /// separate category.
    #[non_exhaustive]
    BlobDecodeFailed {
        /// Description of the decode failure.
        message: String,
    },
    /// Failed to parse a timestamp from its wire-format representation —
    /// for example, a malformed RFC-3339 string or an out-of-range
    /// epoch-seconds value.
    ///
    /// **Not currently emitted.** In-tree codecs surface timestamp-parse
    /// failures as [`InvalidInput`](Self::InvalidInput). This variant is
    /// reserved for codecs that want to distinguish timestamp-parse failures
    /// as a separate category.
    #[non_exhaustive]
    TimestampParseFailed {
        /// Description of the parse failure.
        message: String,
    },
    /// Catch-all for errors not covered by other variants.
    #[non_exhaustive]
    Custom {
        /// Explanatory message.
        message: String,
    },
}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerdeError::TypeMismatch { message } => write!(f, "type mismatch: {message}"),
            SerdeError::MissingMember { member_name } => {
                write!(f, "missing required member: {member_name}")
            }
            SerdeError::UnknownMember { member_name } => {
                write!(f, "unknown member: {member_name}")
            }
            SerdeError::InvalidInput { message } => write!(f, "invalid input: {message}"),
            SerdeError::UnsupportedOperation { message } => {
                write!(f, "unsupported operation: {message}")
            }
            SerdeError::WriteFailed { message } => write!(f, "write failed: {message}"),
            SerdeError::NumericCoercionOverflow { target, value } => {
                write!(f, "numeric value {value} out of range for {target}")
            }
            SerdeError::NumericCoercionLossy { target, value } => {
                write!(
                    f,
                    "numeric value {value} cannot be coerced to {target} without precision loss"
                )
            }
            SerdeError::BlobDecodeFailed { message } => write!(f, "blob decode failed: {message}"),
            SerdeError::TimestampParseFailed { message } => {
                write!(f, "timestamp parse failed: {message}")
            }
            SerdeError::Custom { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for SerdeError {}

impl SerdeError {
    /// Creates a `TypeMismatch` error describing what was expected
    /// versus what was found.
    pub fn type_mismatch(message: impl Into<String>) -> Self {
        SerdeError::TypeMismatch {
            message: message.into(),
        }
    }

    /// Creates a `MissingMember` error for the given member name.
    pub fn missing_member(member_name: impl Into<String>) -> Self {
        SerdeError::MissingMember {
            member_name: member_name.into(),
        }
    }

    /// Creates an `UnknownMember` error for the given member name.
    pub fn unknown_member(member_name: impl Into<String>) -> Self {
        SerdeError::UnknownMember {
            member_name: member_name.into(),
        }
    }

    /// Creates an `InvalidInput` error with the given message.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        SerdeError::InvalidInput {
            message: message.into(),
        }
    }

    /// Creates an `UnsupportedOperation` error with the given message.
    pub fn unsupported(message: impl Into<String>) -> Self {
        SerdeError::UnsupportedOperation {
            message: message.into(),
        }
    }

    /// Creates a `WriteFailed` error with the given message.
    pub fn write_failed(message: impl Into<String>) -> Self {
        SerdeError::WriteFailed {
            message: message.into(),
        }
    }

    /// Creates a `NumericCoercionOverflow` error for a `value` that is
    /// outside the representable range of `target` (e.g. `"byte"`).
    pub fn numeric_coercion_overflow(target: impl Into<String>, value: impl Into<String>) -> Self {
        SerdeError::NumericCoercionOverflow {
            target: target.into(),
            value: value.into(),
        }
    }

    /// Creates a `NumericCoercionLossy` error for a `value` that cannot
    /// be coerced to `target` without losing precision.
    pub fn numeric_coercion_lossy(target: impl Into<String>, value: impl Into<String>) -> Self {
        SerdeError::NumericCoercionLossy {
            target: target.into(),
            value: value.into(),
        }
    }

    /// Creates a `BlobDecodeFailed` error with the given message.
    pub fn blob_decode_failed(message: impl Into<String>) -> Self {
        SerdeError::BlobDecodeFailed {
            message: message.into(),
        }
    }

    /// Creates a `TimestampParseFailed` error with the given message.
    pub fn timestamp_parse_failed(message: impl Into<String>) -> Self {
        SerdeError::TimestampParseFailed {
            message: message.into(),
        }
    }

    /// Creates a custom error with the given message.
    pub fn custom(message: impl Into<String>) -> Self {
        SerdeError::Custom {
            message: message.into(),
        }
    }
}

/// Lift a [`DocumentError`](aws_smithy_types::DocumentError) coming
/// out of the [`Document`](aws_smithy_types::Document)'s
/// numeric / coercion accessors into the schema crate's broader
/// [`SerdeError`].
///
/// The variant shapes line up one-to-one — `DocumentError` is a
/// strict subset (Document-shaped failures only); `SerdeError` adds
/// shape-serde concerns like missing/unknown members and write
/// failures that are out of scope for the types crate.
impl From<aws_smithy_types::DocumentError> for SerdeError {
    fn from(err: aws_smithy_types::DocumentError) -> Self {
        use aws_smithy_types::DocumentError as DE;
        match err {
            DE::TypeMismatch { message, .. } => SerdeError::type_mismatch(message),
            DE::NumericCoercionOverflow { target, value, .. } => {
                SerdeError::numeric_coercion_overflow(target, value)
            }
            DE::InvalidInput { message, .. } => SerdeError::invalid_input(message),
            DE::UnsupportedOperation { message, .. } => SerdeError::unsupported(message),
            DE::Custom { message, .. } => SerdeError::custom(message),
            // `DocumentError` is `#[non_exhaustive]`. Future variants
            // surface as `Custom` so callers always see a reasonable
            // mapping.
            other => SerdeError::custom(format!("{other}")),
        }
    }
}
