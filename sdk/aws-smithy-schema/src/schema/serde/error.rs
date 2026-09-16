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
    TypeMismatch {
        /// Description of what was expected vs what was found.
        message: String,
    },
    /// A required structure member was missing during deserialization.
    MissingMember {
        /// The name of the missing member.
        member_name: String,
    },
    /// An unknown member was encountered during deserialization.
    UnknownMember {
        /// The name of the unknown member.
        member_name: String,
    },
    /// The input data was malformed or invalid for the format.
    InvalidInput {
        /// Description of the problem.
        message: String,
    },
    /// The operation is not supported by this serializer or deserializer.
    UnsupportedOperation {
        /// Description of what was attempted.
        message: String,
    },
    /// An error occurred while writing output during serialization.
    WriteFailed {
        /// Description of the write failure.
        message: String,
    },
    /// Catch-all for errors not covered by other variants.
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
            SerdeError::Custom { message } => f.write_str(message),
        }
    }
}

impl std::error::Error for SerdeError {}

impl SerdeError {
    /// Creates a custom error with the given message.
    pub fn custom(message: impl Into<String>) -> Self {
        SerdeError::Custom {
            message: message.into(),
        }
    }
}
