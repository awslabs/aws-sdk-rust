/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared schemas, types, and helpers for schema-based XML fuzz targets.
//!
//! XML differs from JSON in three important ways for fuzzing:
//!
//! 1. **No naked top-level scalars / collections.** XML documents need a
//!    single root element. Where JSON's harness can serialize
//!    `FuzzValue::Integer(5)` directly to `5`, XML needs the value wrapped
//!    in a one-member struct: `<W><intField>5</intField></W>`. Every
//!    variant of [`FuzzValue`] therefore has a paired single-member
//!    "wrapper" schema this module provides.
//!
//! 2. **No `Document`.** REST XML doesn't support the Smithy document type;
//!    [`XmlSerializer::write_document`] / [`XmlDeserializer::read_document`]
//!    return `SerdeError`. There's no `FuzzValue::Document` variant and
//!    no differential check against an external XML-as-document parser
//!    (the XML data model is structural, so unlike `serde_json::Value` it
//!    has no clean schema-free representation to diff against).
//!
//! 3. **XML-specific traits.** `@xmlAttribute`, `@xmlFlattened`,
//!    `@xmlName`, `@xmlNamespace` each take a different code path in the
//!    serializer and deserializer. The deserialize target exercises these
//!    by including hand-authored schemas that activate each path.

#![allow(dead_code)]

use arbitrary::Arbitrary;
use aws_smithy_schema::serde::{ShapeDeserializer, ShapeSerializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_xml::codec::XmlCodec;

// ---------------------------------------------------------------------------
// Member schemas
// ---------------------------------------------------------------------------
//
// Each `FuzzValue` variant maps to one of these "wrapper" structs whose sole
// member targets the right type. Members carry positional indexes per the SEP
// recommendation; codegen-driven deserialization dispatches on them.

static MEMBER_BOOL: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$boolField"),
    ShapeType::Boolean,
    "boolField",
    0,
);
static MEMBER_BYTE: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$byteField"),
    ShapeType::Byte,
    "byteField",
    0,
);
static MEMBER_SHORT: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$shortField"),
    ShapeType::Short,
    "shortField",
    0,
);
static MEMBER_INT: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$intField"),
    ShapeType::Integer,
    "intField",
    0,
);
static MEMBER_LONG: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$longField"),
    ShapeType::Long,
    "longField",
    0,
);
static MEMBER_FLOAT: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$floatField"),
    ShapeType::Float,
    "floatField",
    0,
);
static MEMBER_DOUBLE: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$doubleField"),
    ShapeType::Double,
    "doubleField",
    0,
);
static MEMBER_STR: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$strField"),
    ShapeType::String,
    "strField",
    0,
);
static MEMBER_BLOB: Schema = Schema::new_member(
    shape_id!("test", "Wrapper$blobField"),
    ShapeType::Blob,
    "blobField",
    0,
);

// ---------------------------------------------------------------------------
// Wrapper schemas — one per scalar variant
// ---------------------------------------------------------------------------

pub static WRAPPER_BOOL_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperBool"),
    ShapeType::Structure,
    &[&MEMBER_BOOL],
);
pub static WRAPPER_BYTE_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperByte"),
    ShapeType::Structure,
    &[&MEMBER_BYTE],
);
pub static WRAPPER_SHORT_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperShort"),
    ShapeType::Structure,
    &[&MEMBER_SHORT],
);
pub static WRAPPER_INT_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperInt"),
    ShapeType::Structure,
    &[&MEMBER_INT],
);
pub static WRAPPER_LONG_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperLong"),
    ShapeType::Structure,
    &[&MEMBER_LONG],
);
pub static WRAPPER_FLOAT_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperFloat"),
    ShapeType::Structure,
    &[&MEMBER_FLOAT],
);
pub static WRAPPER_DOUBLE_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperDouble"),
    ShapeType::Structure,
    &[&MEMBER_DOUBLE],
);
pub static WRAPPER_STRING_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperString"),
    ShapeType::Structure,
    &[&MEMBER_STR],
);
pub static WRAPPER_BLOB_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperBlob"),
    ShapeType::Structure,
    &[&MEMBER_BLOB],
);

// ---------------------------------------------------------------------------
// Collection schemas + their wrappers
// ---------------------------------------------------------------------------
//
// Two layers: the inner *List/*Map schemas are used by read_string_list and
// friends and as the targets for inner member schemas. The wrapper structs
// hold ONE list/map member each so the value can serve as a complete XML
// document during round-trip.

pub static STRING_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "StringList"),
    &aws_smithy_schema::prelude::STRING,
);
pub static INTEGER_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "IntegerList"),
    &aws_smithy_schema::prelude::INTEGER,
);
pub static LONG_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "LongList"),
    &aws_smithy_schema::prelude::LONG,
);
pub static BLOB_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "BlobList"),
    &aws_smithy_schema::prelude::BLOB,
);
pub static STRING_STRING_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "StringStringMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::STRING,
);
pub static STRING_INTEGER_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "StringIntegerMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::INTEGER,
);
pub static STRING_LONG_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "StringLongMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::LONG,
);

static MEMBER_STRING_LIST: Schema = Schema::new_member(
    shape_id!("test", "WrapperStringList$items"),
    ShapeType::List,
    "items",
    0,
);
static MEMBER_INTEGER_LIST: Schema = Schema::new_member(
    shape_id!("test", "WrapperIntegerList$items"),
    ShapeType::List,
    "items",
    0,
);
static MEMBER_LONG_LIST: Schema = Schema::new_member(
    shape_id!("test", "WrapperLongList$items"),
    ShapeType::List,
    "items",
    0,
);
static MEMBER_BLOB_LIST: Schema = Schema::new_member(
    shape_id!("test", "WrapperBlobList$items"),
    ShapeType::List,
    "items",
    0,
);
static MEMBER_STRING_STRING_MAP: Schema = Schema::new_member(
    shape_id!("test", "WrapperStringStringMap$entries"),
    ShapeType::Map,
    "entries",
    0,
);
static MEMBER_STRING_INTEGER_MAP: Schema = Schema::new_member(
    shape_id!("test", "WrapperStringIntegerMap$entries"),
    ShapeType::Map,
    "entries",
    0,
);
static MEMBER_STRING_LONG_MAP: Schema = Schema::new_member(
    shape_id!("test", "WrapperStringLongMap$entries"),
    ShapeType::Map,
    "entries",
    0,
);

pub static WRAPPER_STRING_LIST_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperStringList"),
    ShapeType::Structure,
    &[&MEMBER_STRING_LIST],
);
pub static WRAPPER_INTEGER_LIST_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperIntegerList"),
    ShapeType::Structure,
    &[&MEMBER_INTEGER_LIST],
);
pub static WRAPPER_LONG_LIST_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperLongList"),
    ShapeType::Structure,
    &[&MEMBER_LONG_LIST],
);
pub static WRAPPER_BLOB_LIST_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperBlobList"),
    ShapeType::Structure,
    &[&MEMBER_BLOB_LIST],
);
pub static WRAPPER_STRING_STRING_MAP_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperStringStringMap"),
    ShapeType::Structure,
    &[&MEMBER_STRING_STRING_MAP],
);
pub static WRAPPER_STRING_INTEGER_MAP_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperStringIntegerMap"),
    ShapeType::Structure,
    &[&MEMBER_STRING_INTEGER_MAP],
);
pub static WRAPPER_STRING_LONG_MAP_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "WrapperStringLongMap"),
    ShapeType::Structure,
    &[&MEMBER_STRING_LONG_MAP],
);

// ---------------------------------------------------------------------------
// All-types schema for blanket struct dispatch in the deserialize fuzz target
// ---------------------------------------------------------------------------
//
// Every shape type as a child element of one struct. Used by the deserialize
// target so that arbitrary input can attempt to populate any of these member
// names. Member indices distinct so the consumer (which we leave empty)
// could distinguish them.

static AT_BOOL: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$boolField"),
    ShapeType::Boolean,
    "boolField",
    0,
);
static AT_BYTE: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$byteField"),
    ShapeType::Byte,
    "byteField",
    1,
);
static AT_SHORT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$shortField"),
    ShapeType::Short,
    "shortField",
    2,
);
static AT_INT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$intField"),
    ShapeType::Integer,
    "intField",
    3,
);
static AT_LONG: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$longField"),
    ShapeType::Long,
    "longField",
    4,
);
static AT_FLOAT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$floatField"),
    ShapeType::Float,
    "floatField",
    5,
);
static AT_DOUBLE: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$doubleField"),
    ShapeType::Double,
    "doubleField",
    6,
);
static AT_STR: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$strField"),
    ShapeType::String,
    "strField",
    7,
);
static AT_BLOB: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$blobField"),
    ShapeType::Blob,
    "blobField",
    8,
);
static AT_TS: Schema = Schema::new_member(
    shape_id!("test", "AllTypes$timestampField"),
    ShapeType::Timestamp,
    "timestampField",
    9,
);

pub static ALL_TYPES_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "AllTypes"),
    ShapeType::Structure,
    &[
        &AT_BOOL, &AT_BYTE, &AT_SHORT, &AT_INT, &AT_LONG, &AT_FLOAT, &AT_DOUBLE, &AT_STR, &AT_BLOB,
        &AT_TS,
    ],
);

// ---------------------------------------------------------------------------
// Attribute schema — exercises the deferred-start-tag-flush state machine
// ---------------------------------------------------------------------------
//
// Two `@xmlAttribute` members followed by two body members. Order matters in
// generated code: codegen emits attribute writes before non-attribute writes
// to keep the start tag open while attributes accumulate.

static ATTR_ID: Schema =
    Schema::new_member(shape_id!("test", "Attr$id"), ShapeType::String, "id", 0)
        .with_xml_attribute();
static ATTR_TYPE: Schema = Schema::new_member(
    shape_id!("test", "Attr$type"),
    ShapeType::Integer,
    "type",
    1,
)
.with_xml_attribute();
static ATTR_NAME: Schema =
    Schema::new_member(shape_id!("test", "Attr$name"), ShapeType::String, "name", 2);
static ATTR_AGE: Schema =
    Schema::new_member(shape_id!("test", "Attr$age"), ShapeType::Integer, "age", 3);

pub static ATTR_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "Attr"),
    ShapeType::Structure,
    &[&ATTR_ID, &ATTR_TYPE, &ATTR_NAME, &ATTR_AGE],
);

// ---------------------------------------------------------------------------
// xmlName / xmlNamespace schemas
// ---------------------------------------------------------------------------

static RENAMED_MEMBER: Schema = Schema::new_member(
    shape_id!("test", "Renamed$originalName"),
    ShapeType::String,
    "originalName",
    0,
)
.with_xml_name("RenamedField");

pub static RENAMED_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "Renamed"),
    ShapeType::Structure,
    &[&RENAMED_MEMBER],
)
.with_xml_name("RenamedRoot");

static NS_MEMBER: Schema = Schema::new_member(
    shape_id!("test", "Namespaced$content"),
    ShapeType::String,
    "content",
    0,
);

pub static NAMESPACED_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "Namespaced"),
    ShapeType::Structure,
    &[&NS_MEMBER],
)
.with_xml_namespace("urn:test:fuzz", None);

// ---------------------------------------------------------------------------
// Flattened collection schemas — exercise the flattened-aggregate accumulator
// in the deserializer and the wrapper-elision branch in the serializer.
// ---------------------------------------------------------------------------

static FLAT_LIST_MEMBER: Schema = Schema::new_member(
    shape_id!("test", "FlatList$items"),
    ShapeType::List,
    "items",
    0,
)
.with_xml_flattened();

pub static FLAT_LIST_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "FlatList"),
    ShapeType::Structure,
    &[&FLAT_LIST_MEMBER],
);

static FLAT_MAP_MEMBER: Schema = Schema::new_member(
    shape_id!("test", "FlatMap$entries"),
    ShapeType::Map,
    "entries",
    0,
)
.with_xml_flattened();

pub static FLAT_MAP_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "FlatMap"),
    ShapeType::Structure,
    &[&FLAT_MAP_MEMBER],
);

// ---------------------------------------------------------------------------
// FuzzValue — structured representation of the Smithy data model
// ---------------------------------------------------------------------------
//
// One variant per (de)serializable Smithy primitive plus the four collection
// variants the JSON harness covers. Each variant maps to exactly one
// wrapper schema in `wrapper_schema_for`.
//
// No `Document` variant: REST XML rejects documents. No `Timestamp` variant:
// `arbitrary` doesn't have a derive for `DateTime` and timestamp formatting
// is already exercised by the `AllTypes` schema in the deserialize target.

#[derive(Debug, Clone, Arbitrary)]
pub enum FuzzValue {
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Blob(Vec<u8>),
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    LongList(Vec<i64>),
    BlobList(Vec<Vec<u8>>),
    StringStringMap(Vec<(String, String)>),
    StringIntegerMap(Vec<(String, i32)>),
    StringLongMap(Vec<(String, i64)>),
}

/// Returns the wrapper schema this value will be serialized inside.
///
/// Used by both [`serialize_value`] and [`deserialize_value`] so that the
/// schema used to read back the value matches the one used to write it.
pub fn wrapper_schema_for(value: &FuzzValue) -> &'static Schema {
    match value {
        FuzzValue::Boolean(_) => &WRAPPER_BOOL_SCHEMA,
        FuzzValue::Byte(_) => &WRAPPER_BYTE_SCHEMA,
        FuzzValue::Short(_) => &WRAPPER_SHORT_SCHEMA,
        FuzzValue::Integer(_) => &WRAPPER_INT_SCHEMA,
        FuzzValue::Long(_) => &WRAPPER_LONG_SCHEMA,
        FuzzValue::Float(_) => &WRAPPER_FLOAT_SCHEMA,
        FuzzValue::Double(_) => &WRAPPER_DOUBLE_SCHEMA,
        FuzzValue::String(_) => &WRAPPER_STRING_SCHEMA,
        FuzzValue::Blob(_) => &WRAPPER_BLOB_SCHEMA,
        FuzzValue::StringList(_) => &WRAPPER_STRING_LIST_SCHEMA,
        FuzzValue::IntegerList(_) => &WRAPPER_INTEGER_LIST_SCHEMA,
        FuzzValue::LongList(_) => &WRAPPER_LONG_LIST_SCHEMA,
        FuzzValue::BlobList(_) => &WRAPPER_BLOB_LIST_SCHEMA,
        FuzzValue::StringStringMap(_) => &WRAPPER_STRING_STRING_MAP_SCHEMA,
        FuzzValue::StringIntegerMap(_) => &WRAPPER_STRING_INTEGER_MAP_SCHEMA,
        FuzzValue::StringLongMap(_) => &WRAPPER_STRING_LONG_MAP_SCHEMA,
    }
}

// ---------------------------------------------------------------------------
// Codec helpers
// ---------------------------------------------------------------------------

pub fn default_codec() -> XmlCodec {
    XmlCodec::default()
}

// ---------------------------------------------------------------------------
// SerializableStruct adapter for the wrapper struct shape
// ---------------------------------------------------------------------------
//
// `XmlSerializer::write_struct` takes `&dyn SerializableStruct`. We can't
// implement that directly on `FuzzValue` because the trait signature is
// `serialize_members(&self, &mut dyn ShapeSerializer)` and we want to pick
// the serialization path based on the variant. This adapter does that.

struct FuzzValueAsStruct<'a>(&'a FuzzValue);

impl aws_smithy_schema::serde::SerializableStruct for FuzzValueAsStruct<'_> {
    fn serialize_members(
        &self,
        ser: &mut dyn ShapeSerializer,
    ) -> Result<(), aws_smithy_schema::serde::SerdeError> {
        write_member(self.0, ser)
    }
}

/// Writes the single wrapper-struct member for `value`.
///
/// Called from [`FuzzValueAsStruct::serialize_members`] which is in turn
/// called from inside [`XmlSerializer::write_struct`]. The member schema
/// matches the wrapper schema chosen by [`wrapper_schema_for`].
fn write_member(
    value: &FuzzValue,
    ser: &mut dyn ShapeSerializer,
) -> Result<(), aws_smithy_schema::serde::SerdeError> {
    use aws_smithy_schema::prelude::*;
    match value {
        FuzzValue::Boolean(v) => ser.write_boolean(&MEMBER_BOOL, *v),
        FuzzValue::Byte(v) => ser.write_byte(&MEMBER_BYTE, *v),
        FuzzValue::Short(v) => ser.write_short(&MEMBER_SHORT, *v),
        FuzzValue::Integer(v) => ser.write_integer(&MEMBER_INT, *v),
        FuzzValue::Long(v) => ser.write_long(&MEMBER_LONG, *v),
        FuzzValue::Float(v) => ser.write_float(&MEMBER_FLOAT, *v),
        FuzzValue::Double(v) => ser.write_double(&MEMBER_DOUBLE, *v),
        FuzzValue::String(v) => ser.write_string(&MEMBER_STR, v),
        FuzzValue::Blob(v) => ser.write_blob(&MEMBER_BLOB, v),
        FuzzValue::StringList(items) => ser.write_list(&MEMBER_STRING_LIST, &|ser| {
            for item in items {
                ser.write_string(&STRING, item)?;
            }
            Ok(())
        }),
        FuzzValue::IntegerList(items) => ser.write_list(&MEMBER_INTEGER_LIST, &|ser| {
            for item in items {
                ser.write_integer(&INTEGER, *item)?;
            }
            Ok(())
        }),
        FuzzValue::LongList(items) => ser.write_list(&MEMBER_LONG_LIST, &|ser| {
            for item in items {
                ser.write_long(&LONG, *item)?;
            }
            Ok(())
        }),
        FuzzValue::BlobList(items) => ser.write_list(&MEMBER_BLOB_LIST, &|ser| {
            for item in items {
                ser.write_blob(&BLOB, item)?;
            }
            Ok(())
        }),
        FuzzValue::StringStringMap(entries) => ser.write_map(&MEMBER_STRING_STRING_MAP, &|ser| {
            for (k, v) in entries {
                ser.write_string(&STRING, k)?;
                ser.write_string(&STRING, v)?;
            }
            Ok(())
        }),
        FuzzValue::StringIntegerMap(entries) => ser.write_map(&MEMBER_STRING_INTEGER_MAP, &|ser| {
            for (k, v) in entries {
                ser.write_string(&STRING, k)?;
                ser.write_integer(&INTEGER, *v)?;
            }
            Ok(())
        }),
        FuzzValue::StringLongMap(entries) => ser.write_map(&MEMBER_STRING_LONG_MAP, &|ser| {
            for (k, v) in entries {
                ser.write_string(&STRING, k)?;
                ser.write_long(&LONG, *v)?;
            }
            Ok(())
        }),
    }
}

/// Serializes `value` wrapped in its single-member struct via the codec,
/// returning the serialized XML bytes.
pub fn serialize_value(
    value: &FuzzValue,
    codec: &XmlCodec,
) -> Result<Vec<u8>, aws_smithy_schema::serde::SerdeError> {
    use aws_smithy_schema::codec::{Codec, FinishSerializer};
    let mut ser = codec.create_serializer();
    ser.write_struct(wrapper_schema_for(value), &FuzzValueAsStruct(value))?;
    Ok(ser.finish())
}

/// Deserializes XML bytes into a `FuzzValue` using `original`'s variant to
/// pick the matching read path.
///
/// The returned variant is structurally identical to `original`'s variant
/// (e.g. `FuzzValue::Integer(_)` → `FuzzValue::Integer(_)`); only the inner
/// payload reflects what was actually deserialized.
pub fn deserialize_value(
    bytes: &[u8],
    original: &FuzzValue,
    codec: &XmlCodec,
) -> Result<FuzzValue, aws_smithy_schema::serde::SerdeError> {
    use aws_smithy_schema::codec::Codec;
    use aws_smithy_schema::prelude::*;

    let mut deser = codec.create_deserializer(bytes);
    let schema = wrapper_schema_for(original);
    let mut out: Option<FuzzValue> = None;

    deser.read_struct(schema, &mut |member, d| {
        match original {
            FuzzValue::Boolean(_) => out = Some(FuzzValue::Boolean(d.read_boolean(member)?)),
            FuzzValue::Byte(_) => out = Some(FuzzValue::Byte(d.read_byte(member)?)),
            FuzzValue::Short(_) => out = Some(FuzzValue::Short(d.read_short(member)?)),
            FuzzValue::Integer(_) => out = Some(FuzzValue::Integer(d.read_integer(member)?)),
            FuzzValue::Long(_) => out = Some(FuzzValue::Long(d.read_long(member)?)),
            FuzzValue::Float(_) => out = Some(FuzzValue::Float(d.read_float(member)?)),
            FuzzValue::Double(_) => out = Some(FuzzValue::Double(d.read_double(member)?)),
            FuzzValue::String(_) => out = Some(FuzzValue::String(d.read_string(member)?)),
            FuzzValue::Blob(_) => out = Some(FuzzValue::Blob(d.read_blob(member)?.into_inner())),
            FuzzValue::StringList(_) => {
                let mut items = Vec::new();
                d.read_list(member, &mut |d| {
                    items.push(d.read_string(&STRING)?);
                    Ok(())
                })?;
                out = Some(FuzzValue::StringList(items));
            }
            FuzzValue::IntegerList(_) => {
                let mut items = Vec::new();
                d.read_list(member, &mut |d| {
                    items.push(d.read_integer(&INTEGER)?);
                    Ok(())
                })?;
                out = Some(FuzzValue::IntegerList(items));
            }
            FuzzValue::LongList(_) => {
                let mut items = Vec::new();
                d.read_list(member, &mut |d| {
                    items.push(d.read_long(&LONG)?);
                    Ok(())
                })?;
                out = Some(FuzzValue::LongList(items));
            }
            FuzzValue::BlobList(_) => {
                let mut items = Vec::new();
                d.read_list(member, &mut |d| {
                    items.push(d.read_blob(&BLOB)?.into_inner());
                    Ok(())
                })?;
                out = Some(FuzzValue::BlobList(items));
            }
            FuzzValue::StringStringMap(_) => {
                let mut entries = Vec::new();
                d.read_map(member, &mut |k, d| {
                    entries.push((k, d.read_string(&STRING)?));
                    Ok(())
                })?;
                out = Some(FuzzValue::StringStringMap(entries));
            }
            FuzzValue::StringIntegerMap(_) => {
                let mut entries = Vec::new();
                d.read_map(member, &mut |k, d| {
                    entries.push((k, d.read_integer(&INTEGER)?));
                    Ok(())
                })?;
                out = Some(FuzzValue::StringIntegerMap(entries));
            }
            FuzzValue::StringLongMap(_) => {
                let mut entries = Vec::new();
                d.read_map(member, &mut |k, d| {
                    entries.push((k, d.read_long(&LONG)?));
                    Ok(())
                })?;
                out = Some(FuzzValue::StringLongMap(entries));
            }
        }
        Ok(())
    })?;

    out.ok_or_else(|| {
        aws_smithy_schema::serde::SerdeError::custom(
            "deserializer did not invoke the struct member consumer",
        )
    })
}

// ---------------------------------------------------------------------------
// Equality with NaN handling
// ---------------------------------------------------------------------------

pub fn fuzz_values_equal(a: &FuzzValue, b: &FuzzValue) -> bool {
    match (a, b) {
        (FuzzValue::Float(a), FuzzValue::Float(b)) => float32_eq(*a, *b),
        (FuzzValue::Double(a), FuzzValue::Double(b)) => float64_eq(*a, *b),
        (FuzzValue::Boolean(a), FuzzValue::Boolean(b)) => a == b,
        (FuzzValue::Byte(a), FuzzValue::Byte(b)) => a == b,
        (FuzzValue::Short(a), FuzzValue::Short(b)) => a == b,
        (FuzzValue::Integer(a), FuzzValue::Integer(b)) => a == b,
        (FuzzValue::Long(a), FuzzValue::Long(b)) => a == b,
        (FuzzValue::String(a), FuzzValue::String(b)) => a == b,
        (FuzzValue::Blob(a), FuzzValue::Blob(b)) => a == b,
        (FuzzValue::StringList(a), FuzzValue::StringList(b)) => a == b,
        (FuzzValue::IntegerList(a), FuzzValue::IntegerList(b)) => a == b,
        (FuzzValue::LongList(a), FuzzValue::LongList(b)) => a == b,
        (FuzzValue::BlobList(a), FuzzValue::BlobList(b)) => a == b,
        (FuzzValue::StringStringMap(a), FuzzValue::StringStringMap(b)) => a == b,
        (FuzzValue::StringIntegerMap(a), FuzzValue::StringIntegerMap(b)) => a == b,
        (FuzzValue::StringLongMap(a), FuzzValue::StringLongMap(b)) => a == b,
        _ => false,
    }
}

fn float32_eq(a: f32, b: f32) -> bool {
    (a.is_nan() && b.is_nan()) || a == b
}

fn float64_eq(a: f64, b: f64) -> bool {
    (a.is_nan() && b.is_nan()) || a == b
}

/// Returns `true` if `value` contains a non-finite float (NaN, +/-Infinity).
///
/// Smithy XML serializes these as the literal strings `NaN` / `Infinity` /
/// `-Infinity`, but the deserializer only accepts numeric forms in
/// scalar-read methods. Round-trip targets skip such values to avoid false
/// failures.
pub fn contains_non_finite_float(value: &FuzzValue) -> bool {
    match value {
        FuzzValue::Float(f) => !f.is_finite(),
        FuzzValue::Double(f) => !f.is_finite(),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Strings that XML 1.0 rejects in element content
// ---------------------------------------------------------------------------
//
// XML 1.0 allows only Tab (0x09), LF (0x0A), CR (0x0D), and any byte 0x20
// or above, except for surrogates and a few non-character code points. The
// `xmlparser` reference parser rejects anything else. Since `arbitrary`
// happily generates `String`s containing NUL and other control bytes, we
// need a pre-flight check before round-trip / serialize-validity targets.

pub fn contains_xml10_invalid_char(value: &FuzzValue) -> bool {
    fn check_str(s: &str) -> bool {
        s.chars().any(|c| {
            // XML 1.0 Char production: #x9 | #xA | #xD | [#x20-#xD7FF] |
            // [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            let v = c as u32;
            !(v == 0x9
                || v == 0xA
                || v == 0xD
                || (0x20..=0xD7FF).contains(&v)
                || (0xE000..=0xFFFD).contains(&v)
                || (0x10000..=0x10FFFF).contains(&v))
        })
    }
    match value {
        FuzzValue::String(s) => check_str(s),
        FuzzValue::StringList(items) => items.iter().any(|s| check_str(s)),
        FuzzValue::StringStringMap(entries) => {
            entries.iter().any(|(k, v)| check_str(k) || check_str(v))
        }
        FuzzValue::StringIntegerMap(entries) => entries.iter().any(|(k, _)| check_str(k)),
        FuzzValue::StringLongMap(entries) => entries.iter().any(|(k, _)| check_str(k)),
        _ => false,
    }
}

/// Returns `true` if `value` carries a string usable as an XML element /
/// attribute / map-key name. The XML map-key wire form is `<key>NAME</key>`
/// where NAME is the literal contents of the JSON-like key. Empty keys are
/// not valid XML element content for this purpose, but they are valid
/// strings; round-trip targets that would emit them as element text need
/// to skip them. (No empty-key restriction here — JSON's harness skips
/// empty map keys for the same reason; we mirror that rule via a small
/// helper on the round-trip target.)
pub fn map_key_invalid_for_roundtrip(s: &str) -> bool {
    // Empty string can serialize as `<key></key>` which is well-formed but
    // some XML toolchains collapse it; round-trip is safer to skip.
    s.is_empty()
}
