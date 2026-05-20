/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Enumeration of Smithy shape types.
///
/// This represents the core shape types from the Smithy specification,
/// including simple types, aggregate types, and the special member type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ShapeType {
    // Simple types
    /// Boolean type
    Boolean,
    /// 8-bit signed integer
    Byte,
    /// 16-bit signed integer
    Short,
    /// 32-bit signed integer
    Integer,
    /// 64-bit signed integer
    Long,
    /// 32-bit floating point
    Float,
    /// 64-bit floating point
    Double,
    /// Arbitrary precision integer
    BigInteger,
    /// Arbitrary precision decimal
    BigDecimal,
    /// UTF-8 string
    String,
    /// Binary data
    Blob,
    /// Timestamp
    Timestamp,
    /// Document type
    Document,

    // Aggregate types
    /// List type
    List,
    /// Map type
    Map,
    /// Structure type
    Structure,
    /// Union type
    Union,

    // Member
    /// Member shape
    Member,
}

impl ShapeType {
    /// Returns true if this is a simple type.
    #[inline]
    pub fn is_simple(&self) -> bool {
        matches!(
            self,
            Self::Boolean
                | Self::Byte
                | Self::Short
                | Self::Integer
                | Self::Long
                | Self::Float
                | Self::Double
                | Self::BigInteger
                | Self::BigDecimal
                | Self::String
                | Self::Blob
                | Self::Timestamp
                | Self::Document
        )
    }

    /// Returns true if this is an aggregate type.
    #[inline]
    pub fn is_aggregate(&self) -> bool {
        matches!(self, Self::List | Self::Map | Self::Structure | Self::Union)
    }

    /// Returns true if this is a member type.
    #[inline]
    pub fn is_member(&self) -> bool {
        matches!(self, Self::Member)
    }
}
