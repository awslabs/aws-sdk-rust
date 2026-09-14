/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shape serialization interfaces for the Smithy data model.

use super::error::SerdeError;
use crate::Schema;
use aws_smithy_types::Document;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime};

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
        schema: &Schema<'_>,
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
        schema: &Schema<'_>,
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
        schema: &Schema<'_>,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Writes a boolean value.
    fn write_boolean(&mut self, schema: &Schema<'_>, value: bool) -> Result<(), SerdeError>;

    /// Writes a byte (i8) value.
    fn write_byte(&mut self, schema: &Schema<'_>, value: i8) -> Result<(), SerdeError>;

    /// Writes a short (i16) value.
    fn write_short(&mut self, schema: &Schema<'_>, value: i16) -> Result<(), SerdeError>;

    /// Writes an integer (i32) value.
    fn write_integer(&mut self, schema: &Schema<'_>, value: i32) -> Result<(), SerdeError>;

    /// Writes a long (i64) value.
    fn write_long(&mut self, schema: &Schema<'_>, value: i64) -> Result<(), SerdeError>;

    /// Writes a float (f32) value.
    fn write_float(&mut self, schema: &Schema<'_>, value: f32) -> Result<(), SerdeError>;

    /// Writes a double (f64) value.
    fn write_double(&mut self, schema: &Schema<'_>, value: f64) -> Result<(), SerdeError>;

    /// Writes a big integer value.
    fn write_big_integer(
        &mut self,
        schema: &Schema<'_>,
        value: &BigInteger,
    ) -> Result<(), SerdeError>;

    /// Writes a big decimal value.
    fn write_big_decimal(
        &mut self,
        schema: &Schema<'_>,
        value: &BigDecimal,
    ) -> Result<(), SerdeError>;

    /// Writes a string value.
    fn write_string(&mut self, schema: &Schema<'_>, value: &str) -> Result<(), SerdeError>;

    /// Writes a blob (byte array) value.
    ///
    /// Takes an owned [`Blob`] rather than `&[u8]` so that an implementor which
    /// needs to *retain* the bytes past this call can do so without copying
    /// them. [`Blob`] wraps `bytes::Bytes`, so moving the parameter — or cloning
    /// it — is a refcount operation rather than a payload copy.
    ///
    /// This is load-bearing for the HTTP `@httpPayload` binding, which stores
    /// the payload on the serializer and hands it to the request body after
    /// serialization finishes. With a borrowed parameter there is no lifetime
    /// relationship between `value` and `self`, so retaining it was only
    /// possible by either copying the payload or unsoundly asserting a lifetime.
    ///
    /// Callers that hold something other than a `Blob` convert cheaply:
    /// `Blob::from_maybe_shared(bytes)` reuses an existing `Bytes` allocation,
    /// and `Blob::new(vec)` takes ownership of a `Vec<u8>`. Generated client code
    /// clones the `Blob` field of its data carrier, which is a refcount bump.
    ///
    /// Note the deliberate asymmetry with [`Self::write_string`] and
    /// [`Self::write_document`], which stay borrowed. `Blob` is `Bytes`-backed,
    /// so an owned parameter costs two atomics; `String` and `Document` have no
    /// shared representation, so an owned parameter there would force a real
    /// copy on every caller including the majority that never retain the value.
    /// Do not "fix" the inconsistency.
    fn write_blob(&mut self, schema: &Schema<'_>, value: Blob) -> Result<(), SerdeError>;

    /// Writes a timestamp value.
    fn write_timestamp(&mut self, schema: &Schema<'_>, value: &DateTime) -> Result<(), SerdeError>;

    /// Writes a document value.
    ///
    /// `value` is the [`aws_smithy_types::Document`] (fully
    /// owned, no lifetime). Implementors clone the value into their
    /// output representation.
    fn write_document(&mut self, schema: &Schema<'_>, value: &Document) -> Result<(), SerdeError>;

    /// Writes a null value (for sparse collections).
    fn write_null(&mut self, schema: &Schema<'_>) -> Result<(), SerdeError>;

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
    fn write_string_list(
        &mut self,
        schema: &Schema<'_>,
        values: &[String],
    ) -> Result<(), SerdeError> {
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
        schema: &Schema<'_>,
        values: &[aws_smithy_types::Blob],
    ) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                // Refcount bump, not a payload copy: `Blob` wraps `bytes::Bytes`.
                ser.write_blob(&crate::prelude::BLOB, item.clone())?;
            }
            Ok(())
        })
    }

    /// Writes a list of integers.
    fn write_integer_list(
        &mut self,
        schema: &Schema<'_>,
        values: &[i32],
    ) -> Result<(), SerdeError> {
        self.write_list(schema, &|ser| {
            for item in values {
                ser.write_integer(&crate::prelude::INTEGER, *item)?;
            }
            Ok(())
        })
    }

    /// Writes a list of longs.
    fn write_long_list(&mut self, schema: &Schema<'_>, values: &[i64]) -> Result<(), SerdeError> {
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
        schema: &Schema<'_>,
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
