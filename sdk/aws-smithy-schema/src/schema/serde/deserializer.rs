/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shape deserialization interfaces for the Smithy data model.

use super::error::SerdeError;
use crate::Schema;
use aws_smithy_types::Document;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime};

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

/// Reads values of the Smithy data model from a serialized source, guided by a
/// [`Schema`].
///
/// This is the deserialization counterpart to
/// [`ShapeSerializer`](crate::serde::ShapeSerializer): codecs implement it so a
/// shape can be deserialized from any format without shape-specific code.
pub trait ShapeDeserializer {
    /// Reads a structure from the deserializer.
    ///
    /// The consumer is called for each member with the member schema and a
    /// `&mut dyn ShapeDeserializer` to read the member value. Using `dyn`
    /// allows composite deserializers (e.g., HTTP binding + body) to
    /// transparently delegate without the consumer knowing the concrete type.
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        state: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a list from the deserializer.
    ///
    /// The consumer is called for each element with a `&mut dyn ShapeDeserializer`.
    fn read_list(
        &mut self,
        schema: &Schema<'_>,
        state: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a map from the deserializer.
    ///
    /// The consumer is called for each entry with the key and a `&mut dyn ShapeDeserializer`.
    fn read_map(
        &mut self,
        schema: &Schema<'_>,
        state: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError>;

    /// Reads a boolean value.
    fn read_boolean(&mut self, schema: &Schema<'_>) -> Result<bool, SerdeError>;

    /// Reads a byte (i8) value.
    fn read_byte(&mut self, schema: &Schema<'_>) -> Result<i8, SerdeError>;

    /// Reads a short (i16) value.
    fn read_short(&mut self, schema: &Schema<'_>) -> Result<i16, SerdeError>;

    /// Reads an integer (i32) value.
    fn read_integer(&mut self, schema: &Schema<'_>) -> Result<i32, SerdeError>;

    /// Reads a long (i64) value.
    fn read_long(&mut self, schema: &Schema<'_>) -> Result<i64, SerdeError>;

    /// Reads a float (f32) value.
    fn read_float(&mut self, schema: &Schema<'_>) -> Result<f32, SerdeError>;

    /// Reads a double (f64) value.
    fn read_double(&mut self, schema: &Schema<'_>) -> Result<f64, SerdeError>;

    /// Reads a big integer value.
    fn read_big_integer(&mut self, schema: &Schema<'_>) -> Result<BigInteger, SerdeError>;

    /// Reads a big decimal value.
    fn read_big_decimal(&mut self, schema: &Schema<'_>) -> Result<BigDecimal, SerdeError>;

    /// Reads a string value.
    fn read_string(&mut self, schema: &Schema<'_>) -> Result<String, SerdeError>;

    /// Reads a blob (byte array) value.
    fn read_blob(&mut self, schema: &Schema<'_>) -> Result<Blob, SerdeError>;

    /// Reads a timestamp value.
    fn read_timestamp(&mut self, schema: &Schema<'_>) -> Result<DateTime, SerdeError>;

    /// Reads a document value.
    ///
    /// Returns the [`aws_smithy_types::Document`] (fully owned,
    /// no lifetime). Implementations construct the value from their
    /// underlying source representation.
    fn read_document(&mut self, schema: &Schema<'_>) -> Result<Document, SerdeError>;

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

    /// Consumes the current value without interpreting it.
    ///
    /// A consumer calls this when a value is present but must not be used. The
    /// motivating case is a protocol body that carries a field for a member whose
    /// authoritative value comes from elsewhere in the message — for example an HTTP
    /// response where the member is bound to a header or the status code. The
    /// transport value wins, so the body copy is consumed and discarded rather than
    /// being allowed to overwrite it.
    ///
    /// # Implementor contract
    ///
    /// The requirement is that **after this returns, the parent aggregate read can
    /// continue correctly**. What that takes depends on how the implementation hands
    /// values to a consumer, and the two cases are opposites:
    ///
    /// - **Cursor-based** formats (JSON, CBOR) position a single shared cursor at the
    ///   value and rely on the consumer to move it. These MUST advance past the
    ///   complete value, including all nested content. Failing to advance leaves the
    ///   cursor mid-value and corrupts the rest of the parse.
    /// - **Pre-isolated** formats (XML, document trees, and single-value adapters)
    ///   give the consumer a value the parent has already delimited, and the parent's
    ///   own iteration advances independently. These MUST NOT attempt a second
    ///   advance, and so return `Ok(())` without doing anything.
    ///
    /// The default is a correctness fallback for third-party codecs that have not
    /// considered this method: it reads and drops a [`Document`], which advances a
    /// cursor-based codec correctly at the cost of an allocation. Implementations
    /// SHOULD override it. A codec that cannot produce documents will surface
    /// [`SerdeError::unsupported`] from the default, which is why the built-in
    /// deserializers all override it and never allocate here.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        let _ = self.read_document(&crate::prelude::DOCUMENT)?;
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
    fn read_string_list(&mut self, schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        // Element reads receive the member (list element / map value) schema,
        // not the container schema, so element-level traits (e.g.
        // `@mediaType`, `@timestampFormat`) reach the codec — matching the
        // serializer side. The prelude scalar is the fallback for a schema
        // built without a member set.
        let element = schema.member().unwrap_or(&crate::prelude::STRING);
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_string(element)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of blobs.
    fn read_blob_list(
        &mut self,
        schema: &Schema<'_>,
    ) -> Result<Vec<aws_smithy_types::Blob>, SerdeError> {
        let element = schema.member().unwrap_or(&crate::prelude::BLOB);
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_blob(element)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of integers.
    fn read_integer_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        let element = schema.member().unwrap_or(&crate::prelude::INTEGER);
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_integer(element)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a list of longs.
    fn read_long_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        let element = schema.member().unwrap_or(&crate::prelude::LONG);
        let mut out = Vec::new();
        self.read_list(schema, &mut |deser| {
            out.push(deser.read_long(element)?);
            Ok(())
        })?;
        Ok(out)
    }

    /// Reads a map with string values.
    fn read_string_string_map(
        &mut self,
        schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        // `member()` returns the map *value* schema; the key is produced by
        // `read_map` itself. Fall back to the prelude scalar when unset.
        let value = schema.member().unwrap_or(&crate::prelude::STRING);
        let mut out = std::collections::HashMap::new();
        self.read_map(schema, &mut |key, deser| {
            out.insert(key, deser.read_string(value)?);
            Ok(())
        })?;
        Ok(out)
    }
}

/// Tests for the defaulted [`ShapeDeserializer::skip_value`].
///
/// The default exists so that a third-party codec written before this method compiles
/// unchanged and still behaves correctly. These lock in what "correctly" means: a codec
/// that can produce documents gets a working (if allocating) skip, and one that cannot
/// gets a clear error rather than silent cursor corruption.
#[cfg(test)]
mod default_skip_value {
    use super::*;
    use crate::{shape_id, ShapeType};

    /// A minimal third-party-style deserializer that does **not** override `skip_value`.
    /// `read_document` is the only method that does anything, mirroring a codec whose
    /// document support is its generic value reader.
    struct DocumentCapable {
        read_documents: usize,
    }

    impl ShapeDeserializer for DocumentCapable {
        fn read_struct(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_list(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
            Err(SerdeError::unsupported("big integer"))
        }
        fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
            Err(SerdeError::unsupported("big decimal"))
        }
        fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
            Ok(String::new())
        }
        fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
            Ok(Blob::new(Vec::new()))
        }
        fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
            Ok(DateTime::from_secs(0))
        }
        fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
            self.read_documents += 1;
            Ok(Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    /// Same, but cannot produce documents — the case the doc comment warns about.
    struct NotDocumentCapable(DocumentCapable);

    impl ShapeDeserializer for NotDocumentCapable {
        fn read_struct(
            &mut self,
            schema: &Schema<'_>,
            consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            self.0.read_struct(schema, consumer)
        }
        fn read_list(
            &mut self,
            schema: &Schema<'_>,
            consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            self.0.read_list(schema, consumer)
        }
        fn read_map(
            &mut self,
            schema: &Schema<'_>,
            consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            self.0.read_map(schema, consumer)
        }
        fn read_boolean(&mut self, s: &Schema<'_>) -> Result<bool, SerdeError> {
            self.0.read_boolean(s)
        }
        fn read_byte(&mut self, s: &Schema<'_>) -> Result<i8, SerdeError> {
            self.0.read_byte(s)
        }
        fn read_short(&mut self, s: &Schema<'_>) -> Result<i16, SerdeError> {
            self.0.read_short(s)
        }
        fn read_integer(&mut self, s: &Schema<'_>) -> Result<i32, SerdeError> {
            self.0.read_integer(s)
        }
        fn read_long(&mut self, s: &Schema<'_>) -> Result<i64, SerdeError> {
            self.0.read_long(s)
        }
        fn read_float(&mut self, s: &Schema<'_>) -> Result<f32, SerdeError> {
            self.0.read_float(s)
        }
        fn read_double(&mut self, s: &Schema<'_>) -> Result<f64, SerdeError> {
            self.0.read_double(s)
        }
        fn read_big_integer(&mut self, s: &Schema<'_>) -> Result<BigInteger, SerdeError> {
            self.0.read_big_integer(s)
        }
        fn read_big_decimal(&mut self, s: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
            self.0.read_big_decimal(s)
        }
        fn read_string(&mut self, s: &Schema<'_>) -> Result<String, SerdeError> {
            self.0.read_string(s)
        }
        fn read_blob(&mut self, s: &Schema<'_>) -> Result<Blob, SerdeError> {
            self.0.read_blob(s)
        }
        fn read_timestamp(&mut self, s: &Schema<'_>) -> Result<DateTime, SerdeError> {
            self.0.read_timestamp(s)
        }
        fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
            Err(SerdeError::unsupported(
                "this codec has no document support",
            ))
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    #[test]
    fn default_consumes_the_value_through_read_document() {
        let mut deser = DocumentCapable { read_documents: 0 };
        deser.skip_value().expect("default should succeed");
        assert_eq!(
            deser.read_documents, 1,
            "the default is specified to consume the value via `read_document`"
        );
    }

    #[test]
    fn default_passes_the_prelude_document_schema() {
        // Any codec keying behavior off the schema must receive the prelude document
        // schema, not a member schema it would try to interpret.
        struct CaptureSchema(Option<(String, String)>);
        impl ShapeDeserializer for CaptureSchema {
            fn read_struct(
                &mut self,
                _s: &Schema<'_>,
                _c: &mut dyn FnMut(
                    &Schema<'_>,
                    &mut dyn ShapeDeserializer,
                ) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                Ok(())
            }
            fn read_list(
                &mut self,
                _s: &Schema<'_>,
                _c: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                Ok(())
            }
            fn read_map(
                &mut self,
                _s: &Schema<'_>,
                _c: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                Ok(())
            }
            fn read_boolean(&mut self, _s: &Schema<'_>) -> Result<bool, SerdeError> {
                Ok(false)
            }
            fn read_byte(&mut self, _s: &Schema<'_>) -> Result<i8, SerdeError> {
                Ok(0)
            }
            fn read_short(&mut self, _s: &Schema<'_>) -> Result<i16, SerdeError> {
                Ok(0)
            }
            fn read_integer(&mut self, _s: &Schema<'_>) -> Result<i32, SerdeError> {
                Ok(0)
            }
            fn read_long(&mut self, _s: &Schema<'_>) -> Result<i64, SerdeError> {
                Ok(0)
            }
            fn read_float(&mut self, _s: &Schema<'_>) -> Result<f32, SerdeError> {
                Ok(0.0)
            }
            fn read_double(&mut self, _s: &Schema<'_>) -> Result<f64, SerdeError> {
                Ok(0.0)
            }
            fn read_big_integer(&mut self, _s: &Schema<'_>) -> Result<BigInteger, SerdeError> {
                Err(SerdeError::unsupported("x"))
            }
            fn read_big_decimal(&mut self, _s: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
                Err(SerdeError::unsupported("x"))
            }
            fn read_string(&mut self, _s: &Schema<'_>) -> Result<String, SerdeError> {
                Ok(String::new())
            }
            fn read_blob(&mut self, _s: &Schema<'_>) -> Result<Blob, SerdeError> {
                Ok(Blob::new(Vec::new()))
            }
            fn read_timestamp(&mut self, _s: &Schema<'_>) -> Result<DateTime, SerdeError> {
                Ok(DateTime::from_secs(0))
            }
            fn read_document(&mut self, schema: &Schema<'_>) -> Result<Document, SerdeError> {
                let id = schema.shape_id();
                self.0 = Some((id.namespace().to_owned(), id.shape_name().to_owned()));
                Ok(Document::Null)
            }
            fn is_null(&self) -> bool {
                false
            }
            fn container_size(&self) -> Option<usize> {
                None
            }
        }

        let mut deser = CaptureSchema(None);
        deser.skip_value().unwrap();
        let want = crate::prelude::DOCUMENT.shape_id();
        assert_eq!(
            deser.0,
            Some((want.namespace().to_owned(), want.shape_name().to_owned())),
            "the default must pass the prelude document schema"
        );
    }

    #[test]
    fn default_surfaces_unsupported_for_a_codec_without_documents() {
        // Reporting an error is the intended outcome: it is strictly better than
        // pretending the value was consumed and corrupting the rest of the parse.
        let mut deser = NotDocumentCapable(DocumentCapable { read_documents: 0 });
        let err = deser.skip_value().expect_err("should not silently succeed");
        assert!(
            err.to_string().contains("document support"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn skip_value_is_object_safe() {
        // The composite calls this through `&mut dyn ShapeDeserializer`, so the method
        // must remain dispatchable on a trait object.
        static S: Schema<'static> =
            Schema::new_member(shape_id!("test", "S"), ShapeType::String, "v", 0);
        let _ = &S;
        let mut concrete = DocumentCapable { read_documents: 0 };
        let dynamic: &mut dyn ShapeDeserializer = &mut concrete;
        dynamic.skip_value().unwrap();
        assert_eq!(concrete.read_documents, 1);
    }
}
