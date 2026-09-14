/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! [`DocumentShapeSerializer`] — a [`ShapeSerializer`] implementation that
//! builds up an [`aws_smithy_types::Document`] tree (wrapped in a
//! [`DiscriminatedDocument`] at the root) from a typed Smithy shape.
//!
//! A [`ShapeSerializer`] receives a sequence of `write_*(schema, value)`
//! calls driven either by `value.serialize_members(self)` (for structures)
//! or by a closure passed to `write_list`/`write_map`. The serializer
//! does not see the shape's static type; it only sees the schema and the
//! value at each call site.
//!
//! [`DocumentShapeSerializer`] turns each such call into a [`Document`]
//! node and stitches them into a tree using a small frame stack:
//!
//! - **No frame on the stack** — the next `write_*` call sets the root
//!   document. The common entry point is
//!   [`DiscriminatedDocument::from_struct`](crate::document::DiscriminatedDocumentExt::from_struct),
//!   which immediately calls `write_struct` and produces a structure-
//!   typed root. A bare `write_string` (etc.) at the top level produces
//!   a scalar root, which is also valid (e.g. a payload that's just
//!   `"foo"`).
//! - **Struct frame** — each `write_*(member_schema, value)` insertion
//!   uses `member_schema.member_name()` as the map key.
//! - **List frame** — each `write_*` value is appended to the list.
//! - **Map frame** — calls alternate `key`/`value`. The first call in a
//!   map frame must produce a [`String`] document (the key); the next
//!   call's value is bound to that key.
//!
//! When a frame's body completes, the serializer pops the frame, wraps
//! it in the appropriate [`Document`] variant, and commits the wrapper
//! to the parent frame (or to the root slot if the stack is now empty).

use aws_smithy_types::document::DocumentObject;
use aws_smithy_types::{BigDecimal, BigInteger, DateTime, DiscriminatedDocument, Document, Number};

use crate::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use crate::Schema;

/// Builds a [`DiscriminatedDocument`] from a typed shape via the
/// [`ShapeSerializer`] interface.
///
/// See the module-level documentation for an overview.
///
/// # Discriminator capture
///
/// Top-level discriminator capture (e.g. via
/// [`DiscriminatedDocument::from_struct`](crate::document::DiscriminatedDocumentExt::from_struct)
/// or via direct calls like `ser.write_struct(&MY_SCHEMA, ...)`) goes
/// through the inherent [`DocumentShapeSerializer::write_struct`]
/// method, which captures the schema's shape ID into the serializer's
/// `root_discriminator` slot.
///
/// Nested discriminator capture (a struct member that is itself a
/// struct, called through `value.serialize_members(&mut dyn ShapeSerializer)`)
/// is **not** preserved, because [`Document`] has no per-node
/// discriminator slot — only the outer
/// [`DiscriminatedDocument`] wrapper carries one. Discriminators apply
/// at the wrapper level, not at every nested struct.
///
/// # Example
///
/// Use [`DiscriminatedDocument::from_struct`](crate::document::DiscriminatedDocumentExt::from_struct)
/// for the common case of converting a [`SerializableStruct`] into a
/// [`DiscriminatedDocument`]:
///
/// ```ignore
/// use aws_smithy_schema::document::DiscriminatedDocumentExt;
/// use aws_smithy_types::DiscriminatedDocument;
///
/// let doc = DiscriminatedDocument::from_struct(MyStruct::SCHEMA, &my_struct)?;
/// ```
///
/// Direct construction is useful when round-tripping pieces of a tree
/// outside the standard entry point:
///
/// ```ignore
/// use aws_smithy_schema::document::DocumentShapeSerializer;
/// use aws_smithy_schema::serde::ShapeSerializer;
///
/// let mut ser = DocumentShapeSerializer::new();
/// ser.write_string(&aws_smithy_schema::prelude::STRING, "hello")?;
/// let doc = ser.finish()?;
/// ```
#[derive(Debug, Default)]
pub struct DocumentShapeSerializer {
    /// Stack of in-progress containers. Top is the active write target.
    stack: Vec<Frame>,
    /// Holds the root document once it is committed (i.e. once a write_*
    /// call returns with an empty stack).
    finished: Option<Document>,
    /// FQN of the top-level struct's schema, captured by the inherent
    /// [`DocumentShapeSerializer::write_struct`] method. `None` for
    /// non-struct top-level writes or when the top-level write went
    /// through the trait method.
    root_discriminator: Option<String>,
}

/// An in-progress container on the serializer's frame stack.
#[derive(Debug)]
enum Frame {
    Struct {
        members: DocumentObject,
    },
    List(Vec<Document>),
    Map {
        entries: DocumentObject,
        /// `Some(k)` after a key has been written and we're awaiting the
        /// matching value; `None` when we are at an entry boundary
        /// (next write becomes the next key).
        pending_key: Option<String>,
    },
}

impl DocumentShapeSerializer {
    /// Creates a fresh serializer with an empty frame stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Consumes the serializer and returns the constructed
    /// [`DiscriminatedDocument`].
    ///
    /// The wrapper carries the captured top-level discriminator (the
    /// FQN of the schema passed to the inherent
    /// [`DocumentShapeSerializer::write_struct`] entry point), if any.
    /// Settings are not attached — serialize-side documents have no
    /// protocol context.
    ///
    /// Returns an error if no value has been written yet, if any frame
    /// is still open (caller forgot a closing callback), or if the
    /// serializer was driven into a malformed state.
    pub fn finish(self) -> Result<DiscriminatedDocument, SerdeError> {
        if !self.stack.is_empty() {
            return Err(SerdeError::custom(format!(
                "DocumentShapeSerializer::finish called with {} unfinished container(s) on the stack",
                self.stack.len()
            )));
        }
        let document = self.finished.ok_or_else(|| {
            SerdeError::custom(
                "DocumentShapeSerializer::finish called before any value was written",
            )
        })?;
        let mut wrapper = DiscriminatedDocument::new(document);
        if let Some(d) = self.root_discriminator {
            wrapper = wrapper.with_discriminator(d);
        }
        Ok(wrapper)
    }

    /// Inherent struct write that captures the schema's shape ID as the
    /// serializer's top-level discriminator.
    ///
    /// Method resolution prefers this inherent method over the
    /// [`ShapeSerializer::write_struct`] trait method when the receiver
    /// is a concrete `&mut DocumentShapeSerializer` — so call sites
    /// like
    ///
    /// ```ignore
    /// let mut ser = DocumentShapeSerializer::new();
    /// ser.write_struct(&MY_SCHEMA, &my_value)?;
    /// ```
    ///
    /// hit this method. Indirect calls through `&mut dyn ShapeSerializer`
    /// (e.g., from inside another struct's `serialize_members` callback)
    /// take the trait method instead, which doesn't capture a
    /// discriminator — nested struct discriminators are deliberately
    /// not preserved (see the type-level docs).
    pub fn write_struct(
        &mut self,
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        let is_top_level = self.stack.is_empty() && self.finished.is_none();
        if is_top_level {
            self.root_discriminator = Some(schema.shape_id().as_str().to_string());
        }
        // Capture the top-level discriminator above, then defer the
        // actual struct framing to the trait method so both entry
        // points share one implementation. Nested struct writes reach
        // the trait method directly through `&mut dyn ShapeSerializer`
        // and therefore never capture a discriminator.
        <Self as ShapeSerializer>::write_struct(self, schema, value)
    }

    /// Routes a constructed [`Document`] into the active frame, or
    /// commits it as the root if the stack is empty.
    fn commit_value(&mut self, schema: &Schema<'_>, value: Document) -> Result<(), SerdeError> {
        match self.stack.last_mut() {
            None => {
                if self.finished.is_some() {
                    return Err(SerdeError::custom(
                        "DocumentShapeSerializer received multiple top-level writes",
                    ));
                }
                self.finished = Some(value);
                Ok(())
            }
            Some(Frame::Struct { members }) => {
                let name = schema.member_name().ok_or_else(|| {
                    SerdeError::custom(format!(
                        "writing a struct member requires a member schema with member_name set; got schema for {}",
                        schema.shape_id().as_str()
                    ))
                })?;
                members.insert(name.to_string(), value);
                Ok(())
            }
            Some(Frame::List(items)) => {
                items.push(value);
                Ok(())
            }
            Some(Frame::Map {
                entries,
                pending_key,
            }) => {
                if let Some(key) = pending_key.take() {
                    entries.insert(key, value);
                } else {
                    // First write in an entry must be the key, and
                    // map keys are always strings in the Smithy data
                    // model.
                    let key = match value {
                        Document::String(s) => s,
                        other => {
                            return Err(SerdeError::custom(format!(
                                "map key must be a String document; got {}",
                                shape_kind_name(&other),
                            )))
                        }
                    };
                    *pending_key = Some(key);
                }
                Ok(())
            }
        }
    }
}

/// Human-readable name for a [`Document`] variant, used in error
/// messages for type-mismatch diagnostics.
fn shape_kind_name(d: &Document) -> &'static str {
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
        // The legacy enum is `#[non_exhaustive]`. Future variants are
        // surfaced as a generic kind name for diagnostics.
        _ => "unknown",
    }
}

impl ShapeSerializer for DocumentShapeSerializer {
    fn write_struct(
        &mut self,
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        // Trait dispatch path: nested struct writes from within
        // `serialize_members` callbacks land here. We do NOT capture
        // a discriminator — only the top-level inherent write_struct
        // entry point does that (see the type-level docs).
        self.stack.push(Frame::Struct {
            members: DocumentObject::new(),
        });
        value.serialize_members(self)?;
        let frame = self.stack.pop().expect("frame just pushed");
        let members = match frame {
            Frame::Struct { members } => members,
            _ => {
                return Err(SerdeError::custom(
                    "DocumentShapeSerializer struct frame replaced by a different frame kind during serialization",
                ));
            }
        };
        self.commit_value(schema, Document::Object(members))
    }

    fn write_list(
        &mut self,
        schema: &Schema<'_>,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.stack.push(Frame::List(Vec::new()));
        write_elements(self)?;
        let frame = self.stack.pop().expect("frame just pushed");
        let items = match frame {
            Frame::List(items) => items,
            _ => {
                return Err(SerdeError::custom(
                    "DocumentShapeSerializer list frame replaced by a different frame kind during serialization",
                ));
            }
        };
        self.commit_value(schema, Document::Array(items))
    }

    fn write_map(
        &mut self,
        schema: &Schema<'_>,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.stack.push(Frame::Map {
            entries: DocumentObject::new(),
            pending_key: None,
        });
        write_entries(self)?;
        let frame = self.stack.pop().expect("frame just pushed");
        let (entries, pending_key) = match frame {
            Frame::Map {
                entries,
                pending_key,
            } => (entries, pending_key),
            _ => {
                return Err(SerdeError::custom(
                    "DocumentShapeSerializer map frame replaced by a different frame kind during serialization",
                ));
            }
        };
        if pending_key.is_some() {
            return Err(SerdeError::custom(
                "map ended with an unmatched key (key written without a matching value)",
            ));
        }
        self.commit_value(schema, Document::Object(entries))
    }

    fn write_boolean(&mut self, schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Bool(value))
    }

    fn write_byte(&mut self, schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        // Smithy byte (i8) — encode through Number's signed-int slot.
        self.commit_value(schema, Document::Number(signed_to_number(value as i64)))
    }

    fn write_short(&mut self, schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Number(signed_to_number(value as i64)))
    }

    fn write_integer(&mut self, schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Number(signed_to_number(value as i64)))
    }

    fn write_long(&mut self, schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Number(signed_to_number(value)))
    }

    fn write_float(&mut self, schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Number(Number::Float(value as f64)))
    }

    fn write_double(&mut self, schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Number(Number::Float(value)))
    }

    fn write_big_integer(
        &mut self,
        schema: &Schema<'_>,
        value: &BigInteger,
    ) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::BigInteger(value.clone()))
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema<'_>,
        value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::BigDecimal(value.clone()))
    }

    fn write_string(&mut self, schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::String(value.to_string()))
    }

    fn write_blob(
        &mut self,
        schema: &Schema<'_>,
        value: aws_smithy_types::Blob,
    ) -> Result<(), SerdeError> {
        // `into_inner` consumes the `Blob`, so this no longer copies the payload
        // when the underlying `Bytes` uniquely owns its allocation.
        self.commit_value(schema, Document::Blob(value.into_inner()))
    }

    fn write_timestamp(&mut self, schema: &Schema<'_>, value: &DateTime) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Timestamp(*value))
    }

    fn write_document(&mut self, schema: &Schema<'_>, value: &Document) -> Result<(), SerdeError> {
        // `Document` is fully owned, so this is just a clone.
        self.commit_value(schema, value.clone())
    }

    fn write_null(&mut self, schema: &Schema<'_>) -> Result<(), SerdeError> {
        self.commit_value(schema, Document::Null)
    }
}

/// Routes a signed integer into [`Number::PosInt`] or [`Number::NegInt`]
/// based on sign — [`Number::PosInt`] is the natural slot for non-
/// negative values; [`Number::NegInt`] for negatives.
fn signed_to_number(v: i64) -> Number {
    if v >= 0 {
        Number::PosInt(v as u64)
    } else {
        Number::NegInt(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::{SerdeError, SerializableStruct, ShapeSerializer};
    use crate::{prelude, shape_id, Schema, ShapeId, ShapeType};

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

    /// `struct Person { name: String, age: i32 }`.
    struct Person {
        name: String,
        age: i32,
    }

    impl SerializableStruct for Person {
        fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            ser.write_string(&PERSON_NAME_MEMBER, &self.name)?;
            ser.write_integer(&PERSON_AGE_MEMBER, self.age)?;
            Ok(())
        }
    }

    // -- Top-level scalar -----------------------------------------------

    #[test]
    fn top_level_scalar_string_becomes_root() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_string(&prelude::STRING, "hello").unwrap();
        let doc = ser.finish().unwrap();
        assert_eq!(doc.document().as_string(), Some("hello"));
        assert!(doc.discriminator().is_none());
    }

    #[test]
    fn top_level_scalar_integer_becomes_root() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_integer(&prelude::INTEGER, 42).unwrap();
        let doc = ser.finish().unwrap();
        // 42 is non-negative → encoded as Number::PosInt(42).
        assert!(matches!(
            doc.document().as_number(),
            Some(Number::PosInt(42))
        ));
    }

    #[test]
    fn top_level_null_becomes_root() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_null(&prelude::STRING).unwrap();
        let doc = ser.finish().unwrap();
        assert!(matches!(doc.document(), Document::Null));
    }

    // -- Struct ----------------------------------------------------------

    #[test]
    fn write_struct_produces_object_with_discriminator() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_struct(
            &PERSON_SCHEMA,
            &Person {
                name: "Iago".into(),
                age: 7,
            },
        )
        .unwrap();
        let doc = ser.finish().unwrap();

        assert_eq!(doc.discriminator(), Some("smithy.example#Person"));
        let map = doc.document().as_object().unwrap();
        assert!(matches!(map.get("name"), Some(Document::String(s)) if s == "Iago"));
        assert!(matches!(map.get("age"), Some(Document::Number(_))));
    }

    // -- List ------------------------------------------------------------

    #[test]
    fn write_list_of_strings() {
        let mut ser = DocumentShapeSerializer::new();
        let values = ["a", "b", "c"];
        ser.write_list(&prelude::DOCUMENT, &|inner| {
            for v in &values {
                inner.write_string(&prelude::STRING, v)?;
            }
            Ok(())
        })
        .unwrap();
        let doc = ser.finish().unwrap();
        let items = doc.document().as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], Document::String(s) if s == "a"));
        assert!(matches!(&items[2], Document::String(s) if s == "c"));
    }

    #[test]
    fn write_sparse_list_with_nulls() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_list(&prelude::DOCUMENT, &|inner| {
            inner.write_string(&prelude::STRING, "first")?;
            inner.write_null(&prelude::STRING)?;
            inner.write_string(&prelude::STRING, "third")?;
            Ok(())
        })
        .unwrap();
        let doc = ser.finish().unwrap();
        let items = doc.document().as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert!(matches!(&items[0], Document::String(s) if s == "first"));
        assert!(matches!(&items[1], Document::Null));
        assert!(matches!(&items[2], Document::String(s) if s == "third"));
    }

    // -- Map -------------------------------------------------------------

    #[test]
    fn write_map_of_strings() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_map(&prelude::DOCUMENT, &|inner| {
            inner.write_string(&prelude::STRING, "key1")?;
            inner.write_string(&prelude::STRING, "value1")?;
            inner.write_string(&prelude::STRING, "key2")?;
            inner.write_string(&prelude::STRING, "value2")?;
            Ok(())
        })
        .unwrap();
        let doc = ser.finish().unwrap();
        let map = doc.document().as_object().unwrap();
        assert_eq!(map.len(), 2);
        assert!(matches!(map.get("key1"), Some(Document::String(s)) if s == "value1"));
        assert!(matches!(map.get("key2"), Some(Document::String(s)) if s == "value2"));
    }

    #[test]
    fn write_map_with_non_string_key_errors() {
        let mut ser = DocumentShapeSerializer::new();
        let result = ser.write_map(&prelude::DOCUMENT, &|inner| {
            inner.write_integer(&prelude::INTEGER, 1)?;
            inner.write_string(&prelude::STRING, "value")?;
            Ok(())
        });
        let err = result.expect_err("non-string map key should error");
        let msg = format!("{err}");
        assert!(
            msg.contains("map key must be a String document"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn write_map_with_unmatched_key_errors() {
        let mut ser = DocumentShapeSerializer::new();
        let result = ser.write_map(&prelude::DOCUMENT, &|inner| {
            inner.write_string(&prelude::STRING, "lonely-key")?;
            Ok(())
        });
        let err = result.expect_err("trailing key without value should error");
        let msg = format!("{err}");
        assert!(
            msg.contains("unmatched key"),
            "unexpected error message: {msg}"
        );
    }

    // -- Nested aggregates ----------------------------------------------

    #[test]
    fn nested_struct_in_map_round_trips_data_but_loses_inner_discriminator() {
        // map<String, Person> with a single entry. Nested Person
        // discriminator is NOT preserved by design (see type-level
        // docs): `Document` has no per-node discriminator slot —
        // only the outer `DiscriminatedDocument` wrapper carries
        // one. We still verify the data round-trips.
        let mut ser = DocumentShapeSerializer::new();
        ser.write_map(&prelude::DOCUMENT, &|inner| {
            inner.write_string(&prelude::STRING, "owner")?;
            inner.write_struct(
                &PERSON_SCHEMA,
                &Person {
                    name: "Alex".into(),
                    age: 30,
                },
            )?;
            Ok(())
        })
        .unwrap();
        let doc = ser.finish().unwrap();
        let map = doc.document().as_object().unwrap();
        let nested = map.get("owner").unwrap();
        let nested_members = nested.as_object().unwrap();
        assert!(matches!(
            nested_members.get("name"),
            Some(Document::String(s)) if s == "Alex"
        ));
    }

    // -- write_document round-trips a Document --------------------------

    #[test]
    fn write_document_commits_value_directly() {
        let nested = Document::Object(DocumentObject::from([(
            "foo".to_string(),
            Document::String("bar".to_string()),
        )]));
        let mut ser = DocumentShapeSerializer::new();
        ser.write_document(&prelude::DOCUMENT, &nested).unwrap();
        let doc = ser.finish().unwrap();
        let map = doc.document().as_object().unwrap();
        assert!(matches!(map.get("foo"), Some(Document::String(s)) if s == "bar"));
    }

    // -- Finish error paths ---------------------------------------------

    #[test]
    fn finish_with_no_writes_errors() {
        let ser = DocumentShapeSerializer::new();
        let err = ser.finish().expect_err("empty serializer should error");
        let msg = format!("{err}");
        assert!(
            msg.contains("before any value was written"),
            "unexpected error message: {msg}"
        );
    }

    #[test]
    fn double_top_level_write_errors() {
        let mut ser = DocumentShapeSerializer::new();
        ser.write_string(&prelude::STRING, "first").unwrap();
        let err = ser
            .write_string(&prelude::STRING, "second")
            .expect_err("second top-level write should error");
        let msg = format!("{err}");
        assert!(
            msg.contains("multiple top-level writes"),
            "unexpected error message: {msg}"
        );
    }
}
