/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shape deserialization interfaces for the Smithy data model.

use super::error::SerdeError;
use crate::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

/// Deserializes Smithy shapes from a serial format.
///
/// This trait provides a format-agnostic API for deserializing the Smithy data model.
/// Implementations read from a serial format and create data objects based on schemas.
///
/// The deserializer uses a consumer pattern for aggregate types (structures, lists, maps)
/// to avoid trait object limitations and enable efficient deserialization without
/// intermediate allocations.
///
/// # Consumer Pattern
///
/// For aggregate types, the deserializer calls a consumer function for each element/member.
/// The consumer receives mutable state and updates it with each deserialized value.
/// This pattern:
/// - Avoids trait object issues with generic methods
/// - Enables zero-cost abstractions (closures can be inlined)
/// - Allows caller to control deserialization order and state management
/// - Matches the SEP's recommendation for compiled typed languages
/// - Uses `&mut dyn ShapeDeserializer` so composite deserializers (e.g., HTTP
///   binding + body) can transparently delegate without the consumer knowing
///   the concrete deserializer type. This enables runtime protocol swapping.
///
/// # Example
///
/// ```ignore
/// // Deserializing a structure
/// let mut builder = MyStructBuilder::default();
/// deserializer.read_struct(
///     &MY_STRUCT_SCHEMA,
///     &mut |member, deser| {
///         match member.member_index() {
///             Some(0) => builder.field1 = Some(deser.read_string(member)?),
///             Some(1) => builder.field2 = Some(deser.read_integer(member)?),
///             _ => {}
///         }
///         Ok(())
///     },
/// )?;
/// let my_struct = builder.build();
/// ```
/// Maximum pre-allocation size for containers, used to prevent denial-of-service
/// from untrusted payloads claiming excessively large sizes.
pub const MAX_CONTAINER_PREALLOC: usize = 10_000;

/// Caps a raw container size at [`MAX_CONTAINER_PREALLOC`].
///
/// Implementations of [`ShapeDeserializer::container_size`] SHOULD use this
/// when returning a size derived from untrusted input (e.g., a CBOR length header).
pub fn capped_container_size(raw: usize) -> usize {
    raw.min(MAX_CONTAINER_PREALLOC)
}

pub trait ShapeDeserializer {
    /// Reads a structure from the deserializer.
    ///
    /// The consumer is called for each member with the member schema and a
    /// `&mut dyn ShapeDeserializer` to read the member value. Using `dyn`
    /// allows composite deserializers (e.g., HTTP binding + body) to
    /// transparently delegate without the consumer knowing the concrete type.
    fn read_struct(
        &mut self,
        schema: &Schema,
        state: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a list from the deserializer.
    ///
    /// The consumer is called for each element with a `&mut dyn ShapeDeserializer`.
    fn read_list(
        &mut self,
        schema: &Schema,
        state: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a map from the deserializer.
    ///
    /// The consumer is called for each entry with the key and a `&mut dyn ShapeDeserializer`.
    fn read_map(
        &mut self,
        schema: &Schema,
        state: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a boolean value.
    fn read_boolean(&mut self, schema: &Schema) -> Result<bool, SerdeError>;

    /// Reads a byte (i8) value.
    fn read_byte(&mut self, schema: &Schema) -> Result<i8, SerdeError>;

    /// Reads a short (i16) value.
    fn read_short(&mut self, schema: &Schema) -> Result<i16, SerdeError>;

    /// Reads an integer (i32) value.
    fn read_integer(&mut self, schema: &Schema) -> Result<i32, SerdeError>;

    /// Reads a long (i64) value.
    fn read_long(&mut self, schema: &Schema) -> Result<i64, SerdeError>;

    /// Reads a float (f32) value.
    fn read_float(&mut self, schema: &Schema) -> Result<f32, SerdeError>;

    /// Reads a double (f64) value.
    fn read_double(&mut self, schema: &Schema) -> Result<f64, SerdeError>;

    /// Reads a big integer value.
    fn read_big_integer(&mut self, schema: &Schema) -> Result<BigInteger, SerdeError>;

    /// Reads a big decimal value.
    fn read_big_decimal(&mut self, schema: &Schema) -> Result<BigDecimal, SerdeError>;

    /// Reads a string value.
    fn read_string(&mut self, schema: &Schema) -> Result<String, SerdeError>;

    /// Reads a blob (byte array) value.
    fn read_blob(&mut self, schema: &Schema) -> Result<Blob, SerdeError>;

    /// Reads a timestamp value.
    fn read_timestamp(&mut self, schema: &Schema) -> Result<DateTime, SerdeError>;

    /// Reads a document value.
    fn read_document(&mut self, schema: &Schema) -> Result<Document, SerdeError>;

    /// Checks if the current value is null.
    ///
    /// This is used for sparse collections where null values are significant.
    fn is_null(&self) -> bool;

    /// Consumes a null value, advancing past it.
    ///
    /// This should be called after `is_null()` returns true to advance the
    /// deserializer past the null token.
    fn read_null(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    /// Returns the size of the current container if known.
    ///
    /// This is an optimization hint that allows pre-allocating collections
    /// with the correct capacity. Returns `None` if the size is unknown or
    /// not applicable.
    ///
    /// Implementations SHOULD cap the returned value at a reasonable maximum
    /// (e.g., 10,000) to prevent denial-of-service from untrusted payloads
    /// that claim excessively large container sizes (e.g., a CBOR header
    /// declaring billions of elements). Use [`capped_container_size`] to apply
    /// a standard cap.
    fn container_size(&self) -> Option<usize>;

    // --- Collection helper methods ---
    //
    // This is a **closed set** of helpers for the most common AWS collection
    // patterns. No additional helpers will be added. New collection patterns
    // should use the generic `read_list`/`read_map` with closures.
    //
    // These exist for two reasons:
    // 1. Code size: each helper replaces ~6-8 lines of closure boilerplate in
    //    generated code, yielding ~43% reduction for collection-heavy models.
    // 2. Performance: codec implementations (e.g., `JsonDeserializer`) override
    //    these to call concrete `read_string`/`read_integer`/etc. methods
    //    directly, eliminating per-element vtable dispatch. This requires the
    //    methods to be on the core trait (not an extension trait) since they
    //    are called through `&mut dyn ShapeDeserializer` in generated code.

    /// Reads a list of strings.
    fn read_string_list(&mut self, schema: &Schema) -> Result<Vec<String>, SerdeError> {
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_string(schema)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of blobs.
    fn read_blob_list(
        &mut self,
        schema: &Schema,
    ) -> Result<Vec<aws_smithy_types::Blob>, SerdeError> {
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_blob(schema)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of integers.
    fn read_integer_list(&mut self, schema: &Schema) -> Result<Vec<i32>, SerdeError> {
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_integer(schema)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of longs.
    fn read_long_list(&mut self, schema: &Schema) -> Result<Vec<i64>, SerdeError> {
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_long(schema)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a map with string values.
    fn read_string_string_map(
        &mut self,
        schema: &Schema,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        let mut out = std::collections::HashMap::new();
        self.read_map(schema, &mut |key, deser| {
            out.insert(key, deser.read_string(schema)?);
            Ok(())
        })?;
        Ok(out)
    }
}
