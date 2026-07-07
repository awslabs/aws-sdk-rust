/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shape serialization interfaces for the Smithy data model.

use super::error::SerdeError;
use crate::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

/// Serializes Smithy shapes to a target format.
///
/// This trait provides a format-agnostic API for serializing the Smithy data model.
/// Implementations serialize each data type to the corresponding encoding in their
/// serial format (e.g., Smithy integers and floats to JSON numbers).
///
/// The serializer accepts a schema along with the value to provide additional
/// information about how to serialize the value (e.g., timestamp format, JSON name).
///
/// This trait is object-safe so that generated `SerializableStruct` implementations
/// can use `&mut dyn ShapeSerializer`, producing one compiled `serialize_members()`
/// per shape regardless of how many codecs exist (`shapes + codecs` rather than
/// `shapes * codecs` in binary size).
///
/// # Example
///
/// ```ignore
/// let mut serializer = JsonSerializer::new();
/// serializer.write_string(&STRING_SCHEMA, "hello")?;
/// ```
pub trait ShapeSerializer {
    /// Writes a structure to the serializer.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema of the structure being serialized
    /// * `value` - The structure to serialize
    fn write_struct(
        &mut self,
        schema: &Schema,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError>;

    /// Writes a list to the serializer.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema of the list being serialized
    /// * `write_elements` - Callback that writes the list elements
    fn write_list(
        &mut self,
        schema: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Writes a map to the serializer.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema of the map being serialized
    /// * `write_entries` - Callback that writes the map entries
    fn write_map(
        &mut self,
        schema: &Schema,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Writes a boolean value.
    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), SerdeError>;

    /// Writes a byte (i8) value.
    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), SerdeError>;

    /// Writes a short (i16) value.
    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), SerdeError>;

    /// Writes an integer (i32) value.
    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), SerdeError>;

    /// Writes a long (i64) value.
    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), SerdeError>;

    /// Writes a float (f32) value.
    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), SerdeError>;

    /// Writes a double (f64) value.
    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), SerdeError>;

    /// Writes a big integer value.
    fn write_big_integer(&mut self, schema: &Schema, value: &BigInteger) -> Result<(), SerdeError>;

    /// Writes a big decimal value.
    fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal) -> Result<(), SerdeError>;

    /// Writes a string value.
    fn write_string(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError>;

    /// Writes a blob (byte array) value.
    fn write_blob(&mut self, schema: &Schema, value: &Blob) -> Result<(), SerdeError>;

    /// Writes a timestamp value.
    fn write_timestamp(&mut self, schema: &Schema, value: &DateTime) -> Result<(), SerdeError>;

    /// Writes a document value.
    fn write_document(&mut self, schema: &Schema, value: &Document) -> Result<(), SerdeError>;

    /// Writes a null value (for sparse collections).
    fn write_null(&mut self, schema: &Schema) -> Result<(), SerdeError>;

    // --- Collection helper methods ---
    //
    // This is a **closed set** of helpers for the most common AWS collection
    // patterns. No additional helpers will be added. New collection patterns
    // should use the generic `write_list`/`write_map` with closures.
    //
    // These exist for two reasons:
    // 1. Code size: each helper replaces ~6-8 lines of closure boilerplate in
    //    generated code, yielding ~43% reduction for collection-heavy models.
    // 2. Performance: the corresponding `ShapeDeserializer` helpers are
    //    overridden by codec implementations (e.g., `JsonDeserializer`) to
    //    avoid per-element vtable dispatch. Keeping them on the core trait
    //    (rather than an extension trait) is required because they are called
    //    through `&mut dyn ShapeSerializer`/`&mut dyn ShapeDeserializer` in
    //    generated `serialize_members`/`deserialize` methods.

    /// Writes a list of strings.
    fn write_string_list(&mut self, schema: &Schema, values: &[String]) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                ser.write_string(&crate::prelude::STRING, item)?;
            }
            Ok(())
        })
    }

    /// Writes a list of blobs.
    fn write_blob_list(
        &mut self,
        schema: &Schema,
        values: &[aws_smithy_types::Blob],
    ) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                ser.write_blob(&crate::prelude::BLOB, item)?;
            }
            Ok(())
        })
    }

    /// Writes a list of integers.
    fn write_integer_list(&mut self, schema: &Schema, values: &[i32]) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                ser.write_integer(&crate::prelude::INTEGER, *item)?;
            }
            Ok(())
        })
    }

    /// Writes a list of longs.
    fn write_long_list(&mut self, schema: &Schema, values: &[i64]) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                ser.write_long(&crate::prelude::LONG, *item)?;
            }
            Ok(())
        })
    }

    /// Writes a map with string keys and string values.
    fn write_string_string_map(
        &mut self,
        schema: &Schema,
        values: &std::collections::HashMap<String, String>,
    ) -> Result<(), SerdeError> {
        self.write_map(schema, &|ser| {
            for (key, value) in values {
                ser.write_string(&crate::prelude::STRING, key)?;
                ser.write_string(&crate::prelude::STRING, value)?;
            }
            Ok(())
        })
    }
}

/// Trait for structures that can be serialized via a schema.
///
/// Implemented by generated structure types. Because `ShapeSerializer` is object-safe,
/// each struct gets one compiled `serialize_members()` that works with any serializer
/// through dynamic dispatch.
///
/// # Example
///
/// ```ignore
/// impl SerializableStruct for MyStruct {
///     fn serialize_members(&self, serializer: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
///         serializer.write_string(&NAME_SCHEMA, &self.name)?;
///         serializer.write_integer(&AGE_SCHEMA, self.age)?;
///         Ok(())
///     }
/// }
/// ```
pub trait SerializableStruct {
    /// Serializes this structure's members using the provided serializer.
    fn serialize_members(&self, serializer: &mut dyn ShapeSerializer) -> Result<(), SerdeError>;
}

impl<T: SerializableStruct + ?Sized> SerializableStruct for Box<T> {
    fn serialize_members(&self, serializer: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        (**self).serialize_members(serializer)
    }
}
