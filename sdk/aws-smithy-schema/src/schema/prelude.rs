/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Prelude schemas for built-in Smithy types.
//!
//! This module provides const schemas for Smithy's prelude types,
//! which are the fundamental types available in all Smithy models.

use crate::{shape_id, Schema, ShapeType};

/// Schema for `smithy.api#String`
pub static STRING: Schema = Schema::new(shape_id!("smithy.api", "String"), ShapeType::String);

/// Schema for `smithy.api#Boolean`
pub static BOOLEAN: Schema = Schema::new(shape_id!("smithy.api", "Boolean"), ShapeType::Boolean);

/// Schema for `smithy.api#Byte`
pub static BYTE: Schema = Schema::new(shape_id!("smithy.api", "Byte"), ShapeType::Byte);

/// Schema for `smithy.api#Short`
pub static SHORT: Schema = Schema::new(shape_id!("smithy.api", "Short"), ShapeType::Short);

/// Schema for `smithy.api#Integer`
pub static INTEGER: Schema = Schema::new(shape_id!("smithy.api", "Integer"), ShapeType::Integer);

/// Schema for `smithy.api#Long`
pub static LONG: Schema = Schema::new(shape_id!("smithy.api", "Long"), ShapeType::Long);

/// Schema for `smithy.api#Float`
pub static FLOAT: Schema = Schema::new(shape_id!("smithy.api", "Float"), ShapeType::Float);

/// Schema for `smithy.api#Double`
pub static DOUBLE: Schema = Schema::new(shape_id!("smithy.api", "Double"), ShapeType::Double);

/// Schema for `smithy.api#BigInteger`
pub static BIG_INTEGER: Schema =
    Schema::new(shape_id!("smithy.api", "BigInteger"), ShapeType::BigInteger);

/// Schema for `smithy.api#BigDecimal`
pub static BIG_DECIMAL: Schema =
    Schema::new(shape_id!("smithy.api", "BigDecimal"), ShapeType::BigDecimal);

/// Schema for `smithy.api#Blob`
pub static BLOB: Schema = Schema::new(shape_id!("smithy.api", "Blob"), ShapeType::Blob);

/// Schema for `smithy.api#Timestamp`
pub static TIMESTAMP: Schema =
    Schema::new(shape_id!("smithy.api", "Timestamp"), ShapeType::Timestamp);

/// Schema for `smithy.api#Document`
pub static DOCUMENT: Schema = Schema::new(shape_id!("smithy.api", "Document"), ShapeType::Document);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        assert_eq!(STRING.shape_id().as_str(), "smithy.api#String");
        assert_eq!(STRING.shape_type(), ShapeType::String);
        assert!(STRING.is_string());
        assert!(STRING.traits().is_none());
    }

    #[test]
    fn test_boolean_schema() {
        assert_eq!(BOOLEAN.shape_id().as_str(), "smithy.api#Boolean");
        assert_eq!(BOOLEAN.shape_type(), ShapeType::Boolean);
        assert!(BOOLEAN.traits().is_none());
    }

    #[test]
    fn test_numeric_schemas() {
        assert_eq!(BYTE.shape_type(), ShapeType::Byte);
        assert_eq!(SHORT.shape_type(), ShapeType::Short);
        assert_eq!(INTEGER.shape_type(), ShapeType::Integer);
        assert_eq!(LONG.shape_type(), ShapeType::Long);
        assert_eq!(FLOAT.shape_type(), ShapeType::Float);
        assert_eq!(DOUBLE.shape_type(), ShapeType::Double);
        assert_eq!(BIG_INTEGER.shape_type(), ShapeType::BigInteger);
        assert_eq!(BIG_DECIMAL.shape_type(), ShapeType::BigDecimal);
    }

    #[test]
    fn test_blob_schema() {
        assert_eq!(BLOB.shape_id().as_str(), "smithy.api#Blob");
        assert_eq!(BLOB.shape_type(), ShapeType::Blob);
        assert!(BLOB.is_blob());
    }

    #[test]
    fn test_timestamp_schema() {
        assert_eq!(TIMESTAMP.shape_id().as_str(), "smithy.api#Timestamp");
        assert_eq!(TIMESTAMP.shape_type(), ShapeType::Timestamp);
    }

    #[test]
    fn test_document_schema() {
        assert_eq!(DOCUMENT.shape_id().as_str(), "smithy.api#Document");
        assert_eq!(DOCUMENT.shape_type(), ShapeType::Document);
    }

    #[test]
    fn test_all_prelude_types_are_simple() {
        assert!(STRING.shape_type().is_simple());
        assert!(BOOLEAN.shape_type().is_simple());
        assert!(BYTE.shape_type().is_simple());
        assert!(SHORT.shape_type().is_simple());
        assert!(INTEGER.shape_type().is_simple());
        assert!(LONG.shape_type().is_simple());
        assert!(FLOAT.shape_type().is_simple());
        assert!(DOUBLE.shape_type().is_simple());
        assert!(BIG_INTEGER.shape_type().is_simple());
        assert!(BIG_DECIMAL.shape_type().is_simple());
        assert!(BLOB.shape_type().is_simple());
        assert!(TIMESTAMP.shape_type().is_simple());
        assert!(DOCUMENT.shape_type().is_simple());
    }
}
