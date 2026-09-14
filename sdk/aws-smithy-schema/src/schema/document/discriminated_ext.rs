/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Extension methods on [`DiscriminatedDocument`] that need access to
//! [`Schema`], [`SerializableStruct`], and the schema-side
//! [`ShapeSerializer`](crate::serde::ShapeSerializer) / [`ShapeDeserializer`] machinery — types that
//! live in `aws-smithy-schema` and therefore cannot be referenced from
//! the inherent-impl in `aws-smithy-types` where
//! [`DiscriminatedDocument`] is defined.
//!
//! Without this trait the SEP-mandated `Document.of(struct)` and
//! `Document.asShape(deserialize)` entry points would not be reachable
//! on `DiscriminatedDocument` because the dependency direction
//! (`aws-smithy-schema → aws-smithy-types`) and Rust's orphan rule
//! together prevent adding inherent methods to `DiscriminatedDocument`
//! from this crate.
//!
//! Users import this trait to use the methods:
//!
//! ```ignore
//! use aws_smithy_schema::document::DiscriminatedDocumentExt;
//!
//! let doc = DiscriminatedDocument::from_struct(MyShape::SCHEMA, &my_value)?;
//! let restored: MyShape = doc.as_shape(|deser| MyShape::deserialize(deser))?;
//! ```
//!
//! # Implementation
//!
//! The implementation routes through the schema-side
//! [`DocumentShapeSerializer`] /
//! [`DocumentShapeDeserializer`],
//! which operate on [`aws_smithy_types::Document`] directly.
//! Discriminator capture happens on the way out (the schema's shape
//! ID is recorded as the resulting wrapper's [`discriminator()`]
//! field); on `as_shape`, the deserializer reads from
//! `self.document()` directly.
//!
//! [`discriminator()`]: aws_smithy_types::DiscriminatedDocument::discriminator

use aws_smithy_types::{DiscriminatedDocument, Number};

use super::{DocumentShapeDeserializer, DocumentShapeSerializer};
use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer};
use crate::{Schema, ShapeType};

/// Schema-aware operations on [`DiscriminatedDocument`].
///
/// Implemented for [`DiscriminatedDocument`] in `aws-smithy-schema`
/// because the methods need [`Schema`], [`SerializableStruct`], and
/// the [`ShapeDeserializer`] trait — types that live in this crate.
/// See the trait's documentation for the rationale.
pub trait DiscriminatedDocumentExt {
    /// Constructs a [`DiscriminatedDocument`] from any
    /// [`SerializableStruct`] by driving it through the
    /// [`DocumentShapeSerializer`] pipeline.
    ///
    /// This is the SEP's `Document.of(struct)` entry point. The
    /// resulting wrapper carries `schema.shape_id().as_str()` as its
    /// discriminator, so a downstream type registry can find the right
    /// schema for the reverse conversion.
    ///
    /// Settings are not attached — serialize-side documents have no
    /// protocol context.
    ///
    /// # Errors
    ///
    /// Returns [`SerdeError`] if the struct's `serialize_members`
    /// callback fails (rare — typically only on internal invariant
    /// violations or invalid map-key writes).
    fn from_struct(
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<DiscriminatedDocument, SerdeError>;

    /// Reifies this [`DiscriminatedDocument`] as a typed shape by
    /// driving the given `deserialize` callback through a
    /// [`DocumentShapeDeserializer`].
    ///
    /// This is the SEP's `Document.asShape(...)` entry point. The
    /// callback is typically the generated `<Type>::deserialize`
    /// function on a shape's data carrier or builder.
    ///
    /// # Errors
    ///
    /// Returns whatever error the `deserialize` callback returns —
    /// typically a [`SerdeError::TypeMismatch`] when the document's
    /// shape doesn't match what the callback expects.
    fn as_shape<T, F>(&self, deserialize: F) -> Result<T, SerdeError>
    where
        F: FnOnce(&mut dyn ShapeDeserializer) -> Result<T, SerdeError>;

    /// Returns the [`ShapeType`] this document would be reported as if
    /// converted to a typed shape.
    ///
    /// For numeric values, follows the SEP "Reporting `Document`
    /// ambiguous shape types" guidance — picks the narrowest
    /// unambiguous container from the precedence
    /// `Integer → Long → BigInteger → Double → BigDecimal`. `Byte`,
    /// `IntEnum`, `Short`, and `Float` are skipped.
    ///
    /// `Null` is reported as [`ShapeType::Document`] since `Null`
    /// itself is not a Smithy type variant.
    fn shape_type(&self) -> ShapeType;
}

impl DiscriminatedDocumentExt for DiscriminatedDocument {
    fn from_struct(
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<DiscriminatedDocument, SerdeError> {
        // The schema-side serializer produces a `DiscriminatedDocument`
        // directly. The inherent `DocumentShapeSerializer::write_struct`
        // captures the schema's shape ID into the resulting wrapper's
        // discriminator slot; settings are not attached.
        let mut ser = DocumentShapeSerializer::default();
        ser.write_struct(schema, value)?;
        ser.finish()
    }

    fn as_shape<T, F>(&self, deserialize: F) -> Result<T, SerdeError>
    where
        F: FnOnce(&mut dyn ShapeDeserializer) -> Result<T, SerdeError>,
    {
        // The schema-side deserializer reads directly from the inner
        // `Document`. Settings on the wrapper are attached to the
        // deserializer so format-aware coercion (e.g. a JSON-wire
        // base64 string read as a blob, an epoch-seconds number read
        // as a timestamp) succeeds when the data tree carries the
        // wire-format representation rather than a native variant.
        let mut deser =
            DocumentShapeDeserializer::new_with_settings(self.document(), self.settings().cloned());
        deserialize(&mut deser)
    }

    fn shape_type(&self) -> ShapeType {
        use aws_smithy_types::Document as Doc;
        // The number-disambiguation logic delegates to the private
        // helper that drives `Document::shape_type`. Reads through the
        // `document()` accessor for consistency with `as_shape`.
        match self.document() {
            Doc::Null => ShapeType::Document,
            Doc::Bool(_) => ShapeType::Boolean,
            Doc::Number(n) => number_shape_type(n),
            Doc::Blob(_) => ShapeType::Blob,
            Doc::Timestamp(_) => ShapeType::Timestamp,
            Doc::BigInteger(_) => ShapeType::BigInteger,
            Doc::BigDecimal(_) => ShapeType::BigDecimal,
            Doc::String(_) => ShapeType::String,
            Doc::Array(_) => ShapeType::List,
            Doc::Object(_) => ShapeType::Map,
            // Future variants on the `#[non_exhaustive]` enum fall
            // through to a generic Document type.
            _ => ShapeType::Document,
        }
    }
}

/// SEP "Reporting `Document` ambiguous shape types" — for numeric
/// values stored in a [`Number`], picks the narrowest unambiguous
/// container from the precedence order `Integer → Long → BigInteger →
/// Double → BigDecimal`. `Byte`, `IntEnum`, `Short`, and `Float` are
/// skipped per the SEP.
///
/// Private helper for [`DiscriminatedDocumentExt::shape_type`] on
/// numeric documents.
fn number_shape_type(n: &Number) -> ShapeType {
    match n {
        Number::PosInt(v) => {
            if *v <= i32::MAX as u64 {
                ShapeType::Integer
            } else if *v <= i64::MAX as u64 {
                ShapeType::Long
            } else {
                ShapeType::BigInteger
            }
        }
        Number::NegInt(v) => {
            if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                ShapeType::Integer
            } else {
                // i64 fits in Long but not BigInteger; Number::NegInt is
                // bounded by i64 so BigInteger is unreachable here.
                ShapeType::Long
            }
        }
        Number::Float(v) => {
            // Per the SEP, integer-valued floats should be reported as
            // the narrowest integer container that fits without
            // precision loss. f64 represents integers up to 2^53
            // exactly; beyond that the value is already lossy as f64,
            // so `Double` is the correct report.
            if v.is_finite() && v.fract() == 0.0 {
                if (i32::MIN as f64..=i32::MAX as f64).contains(v) {
                    ShapeType::Integer
                } else if (i64::MIN as f64..=i64::MAX as f64).contains(v) {
                    ShapeType::Long
                } else {
                    ShapeType::Double
                }
            } else {
                ShapeType::Double
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Tests for [`DiscriminatedDocumentExt`].
    //!
    //! Coverage:
    //! - Round-trip a struct using base scalar variants
    //!   (`Person { name: String, age: i32 }`).
    //! - Discriminator capture on `from_struct`.
    //! - `as_shape` reverses the round-trip.
    //! - Shape-type reporting for the extended variants
    //!   (Blob/Timestamp/BigInteger/BigDecimal).
    //! - End-to-end round-trip through `from_struct` for structs whose
    //!   members use the extended variants.
    //! - End-to-end `as_shape` on a top-level extended-variant document.
    //!
    //! All tests use `Schema<'static>` because the schema-crate
    //! prelude and codegen-emitted schemas are `'static`. The trait
    //! method signatures accept any lifetime via `&Schema<'_>`.
    use aws_smithy_types::DateTime;

    use super::*;
    use crate::serde::{SerdeError, ShapeSerializer};
    use crate::{prelude, shape_id, Schema, ShapeId, ShapeType};

    // -- Test schemas + types ------------------------------------------

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

    // -- from_struct happy path ----------------------------------------

    #[test]
    fn from_struct_attaches_fqn_discriminator() {
        let p = Person {
            name: Some("Iago".into()),
            age: Some(7),
        };
        let doc = DiscriminatedDocument::from_struct(&PERSON_SCHEMA, &p).unwrap();
        assert_eq!(doc.discriminator(), Some("smithy.example#Person"));
        assert!(doc.settings().is_none());
        // The inner document is a map with the two members.
        let map = doc.document().as_object().unwrap();
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("name"));
        assert!(map.contains_key("age"));
    }

    #[test]
    fn from_struct_with_only_optional_members_set() {
        let p = Person {
            name: Some("Sam".into()),
            age: None,
        };
        let doc = DiscriminatedDocument::from_struct(&PERSON_SCHEMA, &p).unwrap();
        let map = doc.document().as_object().unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("name"));
    }

    // -- as_shape happy path -------------------------------------------

    #[test]
    fn as_shape_reverses_from_struct() {
        let original = Person {
            name: Some("Alex".into()),
            age: Some(30),
        };
        let doc = DiscriminatedDocument::from_struct(&PERSON_SCHEMA, &original).unwrap();
        let restored: Person = doc.as_shape(|deser| deserialize_person(deser)).unwrap();
        assert_eq!(restored, original);
    }

    #[test]
    fn as_shape_works_on_directly_constructed_document() {
        // Construct a DiscriminatedDocument directly from a map of
        // base scalar variants (no schema-side intermediate). Confirms
        // as_shape works on documents that didn't come from
        // `from_struct`.
        use aws_smithy_types::document::DocumentObject;
        let mut map = DocumentObject::new();
        map.insert(
            "name".to_string(),
            aws_smithy_types::Document::String("Joe".into()),
        );
        map.insert(
            "age".to_string(),
            aws_smithy_types::Document::Number(aws_smithy_types::Number::PosInt(42)),
        );
        let doc = DiscriminatedDocument::new(aws_smithy_types::Document::Object(map));
        let person: Person = doc.as_shape(|deser| deserialize_person(deser)).unwrap();
        assert_eq!(
            person,
            Person {
                name: Some("Joe".into()),
                age: Some(42),
            }
        );
    }

    // -- shape_type reporting ------------------------------------------

    #[test]
    fn shape_type_reports_each_base_variant() {
        use aws_smithy_types::{Document, Number};
        let cases = [
            (Document::Null, ShapeType::Document),
            (Document::Bool(true), ShapeType::Boolean),
            (Document::String("x".into()), ShapeType::String),
            (Document::Number(Number::PosInt(0)), ShapeType::Integer),
            (
                Document::Number(Number::PosInt(u64::MAX)),
                ShapeType::BigInteger,
            ),
            (Document::Array(vec![]), ShapeType::List),
            (Document::Object(Default::default()), ShapeType::Map),
        ];
        for (doc, expected) in cases {
            let wrapped = DiscriminatedDocument::new(doc);
            assert_eq!(wrapped.shape_type(), expected);
        }
    }

    #[test]
    fn shape_type_reports_each_extended_variant() {
        // The extended variants (Blob, Timestamp, BigInteger,
        // BigDecimal) are matched directly by the impl.
        use aws_smithy_types::{BigDecimal, BigInteger, Document};
        use std::str::FromStr;
        let cases: [(Document, ShapeType); 4] = [
            (Document::Blob(vec![1, 2, 3]), ShapeType::Blob),
            (
                Document::Timestamp(DateTime::from_secs(0)),
                ShapeType::Timestamp,
            ),
            (
                Document::BigInteger(BigInteger::from_str("1").unwrap()),
                ShapeType::BigInteger,
            ),
            (
                Document::BigDecimal(BigDecimal::from_str("1.0").unwrap()),
                ShapeType::BigDecimal,
            ),
        ];
        for (doc, expected) in cases {
            let wrapped = DiscriminatedDocument::new(doc);
            assert_eq!(wrapped.shape_type(), expected);
        }
    }

    // -- Extended variants round-trip end-to-end ------------------------

    #[test]
    fn from_struct_with_blob_member_round_trips() {
        const BLOBBY_ID: ShapeId<'static> = shape_id!("smithy.example", "Blobby");
        const BLOBBY_DATA_ID: ShapeId<'static> = shape_id!("smithy.example", "Blobby", "data");
        static BLOBBY_DATA_MEMBER: Schema<'static> =
            Schema::new_member(BLOBBY_DATA_ID, ShapeType::Blob, "data", 0);
        static BLOBBY_SCHEMA: Schema<'static> =
            Schema::new_struct(BLOBBY_ID, ShapeType::Structure, &[&BLOBBY_DATA_MEMBER]);

        struct Blobby {
            data: Vec<u8>,
        }
        impl SerializableStruct for Blobby {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_blob(
                    &BLOBBY_DATA_MEMBER,
                    aws_smithy_types::Blob::new(self.data.clone()),
                )
            }
        }

        let doc = DiscriminatedDocument::from_struct(
            &BLOBBY_SCHEMA,
            &Blobby {
                data: b"raw".to_vec(),
            },
        )
        .expect("from_struct should succeed for blob members");
        assert_eq!(doc.discriminator(), Some("smithy.example#Blobby"));
        let map = doc.document().as_object().unwrap();
        match map.get("data").unwrap() {
            aws_smithy_types::Document::Blob(b) => assert_eq!(b.as_slice(), b"raw"),
            other => panic!("expected Blob in 'data' field, got {other:?}"),
        }
    }

    #[test]
    fn as_shape_on_top_level_blob_document() {
        let doc = DiscriminatedDocument::new(aws_smithy_types::Document::Blob(b"x".to_vec()));
        let blob = doc
            .as_shape(|deser| deser.read_blob(&prelude::BLOB))
            .expect("as_shape on a Blob document should succeed");
        assert_eq!(blob.as_ref(), b"x");
    }

    // -- Discriminator preservation across construction paths ----------

    #[test]
    fn from_struct_then_as_shape_preserves_data_but_ignores_discriminator() {
        // The discriminator captured during from_struct is not
        // re-read during as_shape's deserialize callback (the callback
        // receives the document data, not the discriminator). This
        // matches the SEP behavior: callers wanting discriminator-
        // dispatched deserialization use a TypeRegistry, not as_shape.
        let original = Person {
            name: Some("Lee".into()),
            age: Some(25),
        };
        let doc = DiscriminatedDocument::from_struct(&PERSON_SCHEMA, &original).unwrap();
        assert_eq!(doc.discriminator(), Some("smithy.example#Person"));
        let restored: Person = doc.as_shape(|d| deserialize_person(d)).unwrap();
        assert_eq!(restored, original);
    }
}
