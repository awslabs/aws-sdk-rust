/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! [`DocumentShapeDeserializer`] — a [`ShapeDeserializer`]
//! implementation that walks an [`aws_smithy_types::Document`] tree.
//!
//! Generated `deserialize` methods on Smithy shapes call the
//! [`ShapeDeserializer`] interface to drive structure / list / map
//! consumer dispatch and read scalar leaves. Pointing such generated
//! code at this deserializer reifies a [`Document`] tree as the
//! corresponding typed shape — the inverse of [`DocumentShapeSerializer`].
//!
//! The deserializer holds a borrow of the source [`Document`] so reads
//! are zero-copy where possible (e.g. [`String`] reads still clone the
//! payload, but list/map navigation does not allocate).
//!
//! # Member-name resolution
//!
//! For struct reads, member dispatch uses the document's keys (not the
//! schema's member list) so that documents created from a typed shape
//! by [`DocumentShapeSerializer`] round-trip — those use Smithy member
//! names. Wire-name resolution for `@jsonName` / `@xmlName`-style
//! renames is the responsibility of the protocol's deserializer
//! (`JsonDeserializer`, etc.) — not this generic Document walker. The
//! `JsonFieldMapper` machinery in `aws-smithy-json` performs that
//! mapping during the codec stage; this deserializer simply matches
//! Smithy member names against document keys.
//!
//! Members present in the schema but absent from the document are not
//! reported: generated builders default unset optional members to
//! `None` and required-member enforcement is the builder's
//! responsibility, not the deserializer's.
//!
//! [`DocumentShapeSerializer`]: super::DocumentShapeSerializer

use std::sync::Arc;

use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document, DocumentSettings};

use crate::serde::{capped_container_size, SerdeError, ShapeDeserializer};
use crate::Schema;

/// Walks a [`Document`] tree via the [`ShapeDeserializer`] interface.
///
/// See the module-level documentation for an overview.
///
/// # Example
///
/// Use [`DiscriminatedDocument::as_shape`](crate::document::DiscriminatedDocumentExt::as_shape)
/// for the common case of consuming a [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument)
/// via a generated `deserialize` method:
///
/// ```ignore
/// use aws_smithy_schema::document::DiscriminatedDocumentExt;
///
/// let person: Person = doc.as_shape(|deser| Person::deserialize(deser))?;
/// ```
///
/// Direct construction is useful when consuming a sub-document outside
/// the standard entry point:
///
/// ```ignore
/// use aws_smithy_schema::document::DocumentShapeDeserializer;
/// use aws_smithy_schema::serde::ShapeDeserializer;
///
/// let mut deser = DocumentShapeDeserializer::new(&doc);
/// let s = deser.read_string(&aws_smithy_schema::prelude::STRING)?;
/// ```
#[derive(Debug)]
pub struct DocumentShapeDeserializer<'a> {
    /// The document this deserializer is currently positioned at.
    cursor: &'a Document,
    /// Optional codec settings used for format-aware coercion when
    /// the cursor doesn't match a native variant. For example: a
    /// document parsed from JSON wire bytes encodes blobs as strings
    /// (base64) and may encode timestamps as strings or numbers; with
    /// settings attached, [`Self::read_blob`] / [`Self::read_timestamp`]
    /// fall back through [`DocumentSettings::coerce_string_to_blob`] /
    /// [`DocumentSettings::coerce_string_to_timestamp`] /
    /// [`DocumentSettings::coerce_number_to_timestamp`] when the
    /// native variant isn't present.
    ///
    /// `None` for serialize-side documents (e.g. those produced by
    /// [`super::DocumentShapeSerializer`]) that always carry native
    /// variants — no coercion required.
    settings: Option<Arc<dyn DocumentSettings>>,
}

impl<'a> DocumentShapeDeserializer<'a> {
    /// Creates a deserializer positioned at the given document.
    ///
    /// The lifetime parameter is the borrow lifetime of `document` —
    /// the cursor is just a `&Document`. [`Document`] itself has no
    /// lifetime parameter (it is fully owned), so this `'a` is purely
    /// the per-call borrow lifetime.
    ///
    /// No format-aware coercion is performed; this deserializer is
    /// variant-only. For coercion of wire-encoded blobs and timestamps
    /// (e.g. JSON's base64-string blobs, epoch-seconds-number
    /// timestamps), use [`Self::new_with_settings`].
    pub fn new(document: &'a Document) -> Self {
        Self {
            cursor: document,
            settings: None,
        }
    }

    /// Creates a deserializer positioned at the given document with
    /// codec settings attached.
    ///
    /// With settings present, [`Self::read_blob`] and
    /// [`Self::read_timestamp`] consult the settings to coerce
    /// non-native variants — JSON wire bytes encode blobs as
    /// base64 strings and may encode timestamps as strings or
    /// numbers, so a document parsed from JSON via
    /// [`crate::codec::Codec::create_deserializer`] needs settings
    /// for those round-trips to succeed.
    pub fn new_with_settings(
        document: &'a Document,
        settings: Option<Arc<dyn DocumentSettings>>,
    ) -> Self {
        Self {
            cursor: document,
            settings,
        }
    }
}

/// Builds a `TypeMismatch` error message for a read that expected one
/// kind of value and found another.
fn type_mismatch(expected: &str, found: &Document) -> SerdeError {
    SerdeError::type_mismatch(format!(
        "expected {expected} document, got {}",
        kind_name(found)
    ))
}

/// Human-readable name for a [`Document`] variant, used in error
/// messages for type-mismatch diagnostics.
fn kind_name(d: &Document) -> &'static str {
    match d {
        Document::Null => "null",
        Document::Bool(_) => "boolean",
        Document::Number(_) => "number",
        Document::String(_) => "string",
        Document::Blob(_) => "blob",
        Document::Timestamp(_) => "timestamp",
        Document::BigInteger(_) => "bigInteger",
        Document::BigDecimal(_) => "bigDecimal",
        Document::Array(_) => "list",
        Document::Object(_) => "map",
        // Document is `#[non_exhaustive]`. Future variants surface as
        // a generic kind name so error messages remain informative.
        _ => "unknown",
    }
}

/// Resolves a wire-level map key to a member of `schema`, matching
/// against [`Schema::member_name`] directly.
///
/// Wire-name resolution for `@jsonName` / `@xmlName`-style renames is
/// the responsibility of the protocol's deserializer (which has access
/// to the codec settings); this generic Document walker matches Smithy
/// member names only. Documents produced by [`DocumentShapeSerializer`](super::DocumentShapeSerializer)
/// always use Smithy member names, so the round-trip is exact.
fn resolve_member<'s>(schema: &'s Schema<'s>, wire_name: &str) -> Option<&'s Schema<'s>> {
    let idx = schema
        .members()
        .iter()
        .position(|m| m.member_name() == Some(wire_name))?;
    schema.member_schema_by_index(idx)
}

impl<'a> ShapeDeserializer for DocumentShapeDeserializer<'a> {
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let map = self
            .cursor
            .as_object()
            .ok_or_else(|| type_mismatch("struct (map)", self.cursor))?;
        for (key, value) in map {
            let Some(member_schema) = resolve_member(schema, key) else {
                // Unknown member — silently ignore. Matches the
                // tolerant "ignore unknown fields" behavior of the
                // JSON deserializer.
                continue;
            };
            let mut sub = Self::new_with_settings(value, self.settings.clone());
            consumer(member_schema, &mut sub)?;
        }
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let items = self
            .cursor
            .as_array()
            .ok_or_else(|| type_mismatch("list", self.cursor))?;
        for item in items {
            let mut sub = Self::new_with_settings(item, self.settings.clone());
            consumer(&mut sub)?;
        }
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let entries = self
            .cursor
            .as_object()
            .ok_or_else(|| type_mismatch("map", self.cursor))?;
        for (key, value) in entries {
            let mut sub = Self::new_with_settings(value, self.settings.clone());
            consumer(key.clone(), &mut sub)?;
        }
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.cursor
            .as_bool()
            .ok_or_else(|| type_mismatch("boolean", self.cursor))
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        Ok(self.cursor.as_byte()?)
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        Ok(self.cursor.as_short()?)
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        Ok(self.cursor.as_integer()?)
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        Ok(self.cursor.as_long()?)
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        Ok(self.cursor.as_float()?)
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        Ok(self.cursor.as_double()?)
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        // `coerce_big_integer` widens numeric variants into BigInteger
        // and accepts a native `BigInteger` directly. Mirrors the
        // schema-side type's previous behavior.
        Ok(self.cursor.coerce_big_integer()?)
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        Ok(self.cursor.coerce_big_decimal()?)
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        match self.cursor {
            Document::String(s) => Ok(s.clone()),
            other => Err(type_mismatch("string", other)),
        }
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        // Native variant — return directly.
        if let Document::Blob(b) = self.cursor {
            return Ok(Blob::new(b.clone()));
        }
        // Settings-driven coercion: e.g. JSON encodes blobs as
        // base64-encoded strings on the wire, and the codec's
        // [`DocumentSettings::coerce_string_to_blob`] decodes them.
        if let (Document::String(s), Some(settings)) = (self.cursor, &self.settings) {
            return settings
                .coerce_string_to_blob(s)
                .map(Blob::new)
                .map_err(SerdeError::from);
        }
        Err(type_mismatch("blob", self.cursor))
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        // Native variant — return directly.
        if let Document::Timestamp(ts) = self.cursor {
            return Ok(*ts);
        }
        // Settings-driven coercion: JSON typically encodes timestamps
        // as numbers (epoch-seconds) or strings (date-time / http-date),
        // depending on the codec's default format. The codec's
        // [`DocumentSettings`] supplies the format; the right
        // [`DocumentSettings::coerce_*_to_timestamp`] is dispatched
        // by source variant.
        if let Some(settings) = &self.settings {
            match self.cursor {
                Document::String(s) => {
                    return settings
                        .coerce_string_to_timestamp(s)
                        .map_err(SerdeError::from);
                }
                Document::Number(n) => {
                    return settings
                        .coerce_number_to_timestamp(n)
                        .map_err(SerdeError::from);
                }
                _ => {}
            }
        }
        Err(type_mismatch("timestamp", self.cursor))
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Ok(self.cursor.clone())
    }

    fn is_null(&self) -> bool {
        matches!(self.cursor, Document::Null)
    }

    fn container_size(&self) -> Option<usize> {
        let raw = match self.cursor {
            Document::Array(items) => items.len(),
            Document::Object(entries) => entries.len(),
            _ => return None,
        };
        Some(capped_container_size(raw))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::document::DocumentShapeSerializer;
    use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
    use crate::{prelude, shape_id, Schema, ShapeId, ShapeType};

    use aws_smithy_types::document::DocumentObject;
    use aws_smithy_types::Number;

    // -- Test schemas ----------------------------------------------------

    const PERSON_ID: ShapeId<'static> = shape_id!("smithy.example", "Person");
    const PERSON_NAME_ID: ShapeId<'static> = shape_id!("smithy.example", "Person", "name");
    const PERSON_AGE_ID: ShapeId<'static> = shape_id!("smithy.example", "Person", "age");

    static PERSON_NAME_MEMBER: Schema<'static> =
        Schema::new_member(PERSON_NAME_ID, ShapeType::String, "name", 0);
    static PERSON_AGE_MEMBER: Schema<'static> =
        Schema::new_member(PERSON_AGE_ID, ShapeType::Integer, "age", 1);
    static PERSON_SCHEMA: Schema<'static> = Schema::new_struct(
        PERSON_ID,
        ShapeType::Structure,
        &[&PERSON_NAME_MEMBER, &PERSON_AGE_MEMBER],
    );

    /// Test struct + builder pair to drive `read_struct` consumer
    /// dispatch the same way generated code does.
    #[derive(Debug, Default, PartialEq)]
    struct Person {
        name: Option<String>,
        age: Option<i32>,
    }

    impl SerializableStruct for Person {
        fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            if let Some(n) = &self.name {
                ser.write_string(&PERSON_NAME_MEMBER, n)?;
            }
            if let Some(a) = self.age {
                ser.write_integer(&PERSON_AGE_MEMBER, a)?;
            }
            Ok(())
        }
    }

    fn deserialize_person(deser: &mut dyn ShapeDeserializer) -> Result<Person, SerdeError> {
        let mut out = Person::default();
        deser.read_struct(&PERSON_SCHEMA, &mut |member, sub| {
            match member.member_index() {
                Some(0) => out.name = Some(sub.read_string(member)?),
                Some(1) => out.age = Some(sub.read_integer(member)?),
                _ => {}
            }
            Ok(())
        })?;
        Ok(out)
    }

    // -- Scalars ---------------------------------------------------------

    #[test]
    fn read_string_returns_value() {
        let doc = Document::String("hello".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.read_string(&prelude::STRING).unwrap(), "hello");
    }

    #[test]
    fn read_string_on_non_string_errors() {
        let doc = Document::Number(Number::PosInt(1));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_string(&prelude::STRING).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    #[test]
    fn read_integer_with_coercion() {
        // Integer narrowing follows SEP rules (precision loss ignored).
        let doc = Document::Number(Number::PosInt(42));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.read_integer(&prelude::INTEGER).unwrap(), 42);
    }

    #[test]
    fn read_integer_overflow_errors() {
        let doc = Document::Number(Number::PosInt(u64::MAX));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_integer(&prelude::INTEGER).unwrap_err();
        assert!(matches!(err, SerdeError::NumericCoercionOverflow { .. }));
    }

    #[test]
    fn read_boolean_returns_value() {
        let doc = Document::Bool(true);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.read_boolean(&prelude::BOOLEAN).unwrap());
    }

    #[test]
    fn read_blob_returns_value_for_native_blob() {
        let doc = Document::Blob(vec![1, 2, 3]);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let blob = deser.read_blob(&prelude::BLOB).unwrap();
        assert_eq!(blob.as_ref(), &[1u8, 2, 3]);
    }

    #[test]
    fn read_blob_on_string_errors_without_settings() {
        // The deserializer no longer consults DocumentSettings for
        // base64-coercion — that lives on DiscriminatedDocument now.
        let doc = Document::String("YWJjZA==".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_blob(&prelude::BLOB).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    #[test]
    fn is_null_for_null_document() {
        let doc = Document::Null;
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.is_null());
    }

    #[test]
    fn is_null_false_for_non_null() {
        let doc = Document::String("not-null".to_string());
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(!deser.is_null());
    }

    // -- Aggregates ------------------------------------------------------

    #[test]
    fn read_list_iterates_elements() {
        let doc = Document::Array(vec![
            Document::String("a".to_string()),
            Document::String("b".to_string()),
            Document::String("c".to_string()),
        ]);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut collected = Vec::new();
        deser
            .read_list(&prelude::DOCUMENT, &mut |sub| {
                collected.push(sub.read_string(&prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected, ["a", "b", "c"]);
    }

    #[test]
    fn read_map_iterates_entries() {
        let doc = Document::Object(DocumentObject::from([
            ("k1".to_string(), Document::String("v1".to_string())),
            ("k2".to_string(), Document::String("v2".to_string())),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut collected = HashMap::new();
        deser
            .read_map(&prelude::DOCUMENT, &mut |key, sub| {
                collected.insert(key, sub.read_string(&prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected.get("k1").map(String::as_str), Some("v1"));
        assert_eq!(collected.get("k2").map(String::as_str), Some("v2"));
    }

    #[test]
    fn container_size_on_list() {
        let doc = Document::Array(vec![Document::Null; 5]);
        let deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.container_size(), Some(5));
    }

    #[test]
    fn container_size_on_scalar_is_none() {
        let doc = Document::String("foo".to_string());
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.container_size().is_none());
    }

    // -- Struct round-trip ----------------------------------------------

    #[test]
    fn read_struct_with_consumer_dispatch() {
        let doc = Document::Object(DocumentObject::from([
            ("name".to_string(), Document::String("Alex".to_string())),
            ("age".to_string(), Document::Number(Number::PosInt(30))),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(
            person,
            Person {
                name: Some("Alex".into()),
                age: Some(30),
            }
        );
    }

    #[test]
    fn read_struct_with_missing_optional_member() {
        // Document only has `name`; `age` is missing.
        let doc = Document::Object(DocumentObject::from([(
            "name".to_string(),
            Document::String("Sam".to_string()),
        )]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(
            person,
            Person {
                name: Some("Sam".into()),
                age: None,
            }
        );
    }

    #[test]
    fn read_struct_ignores_unknown_members() {
        let doc = Document::Object(DocumentObject::from([
            ("name".to_string(), Document::String("Joe".to_string())),
            (
                "unknown_field".to_string(),
                Document::String("ignored".to_string()),
            ),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(person.name.as_deref(), Some("Joe"));
    }

    #[test]
    fn read_struct_on_non_map_errors() {
        let doc = Document::String("not-a-struct".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deserialize_person(&mut deser).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    // -- Round-trip with the serializer ---------------------------------

    #[test]
    fn round_trip_through_document() {
        let original = Person {
            name: Some("Iago".into()),
            age: Some(7),
        };
        // serialize → DiscriminatedDocument
        let mut ser = DocumentShapeSerializer::new();
        ser.write_struct(&PERSON_SCHEMA, &original).unwrap();
        let doc = ser.finish().unwrap();
        // deserialize the inner Document → typed
        let mut deser = DocumentShapeDeserializer::new(doc.document());
        let restored = deserialize_person(&mut deser).unwrap();
        assert_eq!(restored, original);
        // discriminator is preserved at the wrapper level
        assert_eq!(doc.discriminator(), Some("smithy.example#Person"));
    }
}
