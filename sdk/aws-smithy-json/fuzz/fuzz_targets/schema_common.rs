/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared schemas, types, and helpers for schema-based JSON fuzz targets.

#![allow(dead_code)]

use arbitrary::Arbitrary;
use aws_smithy_json::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_schema::serde::{ShapeDeserializer, ShapeSerializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};

// ---------------------------------------------------------------------------
// Static schemas covering all shape types
// ---------------------------------------------------------------------------

static MEMBER_BOOL: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "boolField"),
    ShapeType::Boolean,
    "boolField",
    0,
);
static MEMBER_INT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "intField"),
    ShapeType::Integer,
    "intField",
    1,
);
static MEMBER_STR: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "strField"),
    ShapeType::String,
    "strField",
    2,
);
static MEMBER_FLOAT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "floatField"),
    ShapeType::Float,
    "floatField",
    3,
);
static MEMBER_DOUBLE: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "doubleField"),
    ShapeType::Double,
    "doubleField",
    4,
);
static MEMBER_LONG: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "longField"),
    ShapeType::Long,
    "longField",
    5,
);
static MEMBER_BYTE: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "byteField"),
    ShapeType::Byte,
    "byteField",
    6,
);
static MEMBER_SHORT: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "shortField"),
    ShapeType::Short,
    "shortField",
    7,
);
static MEMBER_BLOB: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "blobField"),
    ShapeType::Blob,
    "blobField",
    8,
);
static MEMBER_DOC: Schema = Schema::new_member(
    shape_id!("test", "AllTypes", "documentField"),
    ShapeType::Document,
    "documentField",
    9,
);

pub static ALL_TYPES_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "AllTypes"),
    ShapeType::Structure,
    &[
        &MEMBER_BOOL,
        &MEMBER_INT,
        &MEMBER_STR,
        &MEMBER_FLOAT,
        &MEMBER_DOUBLE,
        &MEMBER_LONG,
        &MEMBER_BYTE,
        &MEMBER_SHORT,
        &MEMBER_BLOB,
        &MEMBER_DOC,
    ],
);

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

pub static STRING_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "StringMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::STRING,
);

pub static INTEGER_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "IntegerMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::INTEGER,
);

pub static LONG_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "LongMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::LONG,
);

// Schema with @jsonName for field mapping coverage
static MEMBER_RENAMED: Schema = Schema::new_member(
    shape_id!("test", "Renamed", "originalName"),
    ShapeType::String,
    "originalName",
    0,
)
.with_json_name("RenamedField");

pub static RENAMED_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "Renamed"),
    ShapeType::Structure,
    &[&MEMBER_RENAMED],
);

// ---------------------------------------------------------------------------
// FuzzValue — structured representation of the Smithy data model
// ---------------------------------------------------------------------------

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
    /// Each list/map variant uses elements matching its schema, so roundtrip
    /// deserialization always calls the correct `read_*` method.
    StringList(Vec<String>),
    IntegerList(Vec<i32>),
    LongList(Vec<i64>),
    BlobList(Vec<Vec<u8>>),
    StringStringMap(Vec<(String, String)>),
    StringIntegerMap(Vec<(String, i32)>),
    StringLongMap(Vec<(String, i64)>),
    Null,
}

// ---------------------------------------------------------------------------
// Codec helpers
// ---------------------------------------------------------------------------

pub fn default_codec() -> JsonCodec {
    JsonCodec::new(JsonCodecSettings::default())
}

pub fn no_json_name_codec() -> JsonCodec {
    JsonCodec::new(JsonCodecSettings::builder().use_json_name(false).build())
}

// ---------------------------------------------------------------------------
// Serialize FuzzValue → JsonSerializer
// ---------------------------------------------------------------------------

pub fn serialize_fuzz_value(value: &FuzzValue, ser: &mut dyn ShapeSerializer) {
    use aws_smithy_schema::prelude::*;
    match value {
        FuzzValue::Boolean(v) => {
            let _ = ser.write_boolean(&BOOLEAN, *v);
        }
        FuzzValue::Byte(v) => {
            let _ = ser.write_byte(&BYTE, *v);
        }
        FuzzValue::Short(v) => {
            let _ = ser.write_short(&SHORT, *v);
        }
        FuzzValue::Integer(v) => {
            let _ = ser.write_integer(&INTEGER, *v);
        }
        FuzzValue::Long(v) => {
            let _ = ser.write_long(&LONG, *v);
        }
        FuzzValue::Float(v) => {
            let _ = ser.write_float(&FLOAT, *v);
        }
        FuzzValue::Double(v) => {
            let _ = ser.write_double(&DOUBLE, *v);
        }
        FuzzValue::String(v) => {
            let _ = ser.write_string(&STRING, v);
        }
        FuzzValue::Blob(v) => {
            let _ = ser.write_blob(&BLOB, &aws_smithy_types::Blob::new(v.clone()));
        }
        FuzzValue::StringList(items) => {
            let _ = ser.write_list(&STRING_LIST_SCHEMA, &|ser| {
                for item in items {
                    ser.write_string(&STRING, item)?;
                }
                Ok(())
            });
        }
        FuzzValue::IntegerList(items) => {
            let _ = ser.write_list(&INTEGER_LIST_SCHEMA, &|ser| {
                for item in items {
                    ser.write_integer(&INTEGER, *item)?;
                }
                Ok(())
            });
        }
        FuzzValue::LongList(items) => {
            let _ = ser.write_list(&LONG_LIST_SCHEMA, &|ser| {
                for item in items {
                    ser.write_long(&LONG, *item)?;
                }
                Ok(())
            });
        }
        FuzzValue::BlobList(items) => {
            let _ = ser.write_list(&BLOB_LIST_SCHEMA, &|ser| {
                for item in items {
                    ser.write_blob(&BLOB, &aws_smithy_types::Blob::new(item.clone()))?;
                }
                Ok(())
            });
        }
        FuzzValue::StringStringMap(entries) => {
            let _ = ser.write_map(&STRING_MAP_SCHEMA, &|ser| {
                for (k, v) in entries {
                    ser.write_string(&STRING, k)?;
                    ser.write_string(&STRING, v)?;
                }
                Ok(())
            });
        }
        FuzzValue::StringIntegerMap(entries) => {
            let _ = ser.write_map(&INTEGER_MAP_SCHEMA, &|ser| {
                for (k, v) in entries {
                    ser.write_string(&STRING, k)?;
                    ser.write_integer(&INTEGER, *v)?;
                }
                Ok(())
            });
        }
        FuzzValue::StringLongMap(entries) => {
            let _ = ser.write_map(&LONG_MAP_SCHEMA, &|ser| {
                for (k, v) in entries {
                    ser.write_string(&STRING, k)?;
                    ser.write_long(&LONG, *v)?;
                }
                Ok(())
            });
        }
        FuzzValue::Null => {
            let _ = ser.write_null(&aws_smithy_schema::prelude::STRING);
        }
    }
}

// ---------------------------------------------------------------------------
// Deserialize JsonDeserializer → FuzzValue
// ---------------------------------------------------------------------------

pub fn deserialize_fuzz_value(
    deser: &mut dyn ShapeDeserializer,
    original: &FuzzValue,
) -> Result<FuzzValue, aws_smithy_schema::serde::SerdeError> {
    use aws_smithy_schema::prelude::*;
    match original {
        FuzzValue::Boolean(_) => Ok(FuzzValue::Boolean(deser.read_boolean(&BOOLEAN)?)),
        FuzzValue::Byte(_) => Ok(FuzzValue::Byte(deser.read_byte(&BYTE)?)),
        FuzzValue::Short(_) => Ok(FuzzValue::Short(deser.read_short(&SHORT)?)),
        FuzzValue::Integer(_) => Ok(FuzzValue::Integer(deser.read_integer(&INTEGER)?)),
        FuzzValue::Long(_) => Ok(FuzzValue::Long(deser.read_long(&LONG)?)),
        FuzzValue::Float(_) => Ok(FuzzValue::Float(deser.read_float(&FLOAT)?)),
        FuzzValue::Double(_) => Ok(FuzzValue::Double(deser.read_double(&DOUBLE)?)),
        FuzzValue::String(_) => Ok(FuzzValue::String(deser.read_string(&STRING)?)),
        FuzzValue::Blob(_) => {
            let blob = deser.read_blob(&BLOB)?;
            Ok(FuzzValue::Blob(blob.into_inner()))
        }
        FuzzValue::StringList(_) => {
            let mut out = Vec::new();
            deser.read_list(&STRING_LIST_SCHEMA, &mut |d| {
                out.push(d.read_string(&STRING)?);
                Ok(())
            })?;
            Ok(FuzzValue::StringList(out))
        }
        FuzzValue::IntegerList(_) => {
            let mut out = Vec::new();
            deser.read_list(&INTEGER_LIST_SCHEMA, &mut |d| {
                out.push(d.read_integer(&INTEGER)?);
                Ok(())
            })?;
            Ok(FuzzValue::IntegerList(out))
        }
        FuzzValue::LongList(_) => {
            let mut out = Vec::new();
            deser.read_list(&LONG_LIST_SCHEMA, &mut |d| {
                out.push(d.read_long(&LONG)?);
                Ok(())
            })?;
            Ok(FuzzValue::LongList(out))
        }
        FuzzValue::BlobList(_) => {
            let mut out = Vec::new();
            deser.read_list(&BLOB_LIST_SCHEMA, &mut |d| {
                out.push(d.read_blob(&BLOB)?.into_inner());
                Ok(())
            })?;
            Ok(FuzzValue::BlobList(out))
        }
        FuzzValue::StringStringMap(_) => {
            let mut out = Vec::new();
            deser.read_map(&STRING_MAP_SCHEMA, &mut |key, d| {
                out.push((key, d.read_string(&STRING)?));
                Ok(())
            })?;
            Ok(FuzzValue::StringStringMap(out))
        }
        FuzzValue::StringIntegerMap(_) => {
            let mut out = Vec::new();
            deser.read_map(&INTEGER_MAP_SCHEMA, &mut |key, d| {
                out.push((key, d.read_integer(&INTEGER)?));
                Ok(())
            })?;
            Ok(FuzzValue::StringIntegerMap(out))
        }
        FuzzValue::StringLongMap(_) => {
            let mut out = Vec::new();
            deser.read_map(&LONG_MAP_SCHEMA, &mut |key, d| {
                out.push((key, d.read_long(&LONG)?));
                Ok(())
            })?;
            Ok(FuzzValue::StringLongMap(out))
        }
        FuzzValue::Null => {
            if deser.is_null() {
                deser.read_null()?;
            }
            Ok(FuzzValue::Null)
        }
    }
}

// ---------------------------------------------------------------------------
// FuzzValue equality with float NaN handling
// ---------------------------------------------------------------------------

pub fn fuzz_values_equal(a: &FuzzValue, b: &FuzzValue) -> bool {
    match (a, b) {
        (FuzzValue::Float(a), FuzzValue::Float(b)) => float_eq(*a as f64, *b as f64),
        (FuzzValue::Double(a), FuzzValue::Double(b)) => float_eq(*a, *b),
        (FuzzValue::Boolean(a), FuzzValue::Boolean(b)) => a == b,
        (FuzzValue::Byte(a), FuzzValue::Byte(b)) => a == b,
        (FuzzValue::Short(a), FuzzValue::Short(b)) => a == b,
        (FuzzValue::Integer(a), FuzzValue::Integer(b)) => a == b,
        (FuzzValue::Long(a), FuzzValue::Long(b)) => a == b,
        (FuzzValue::String(a), FuzzValue::String(b)) => a == b,
        (FuzzValue::Blob(a), FuzzValue::Blob(b)) => a == b,
        (FuzzValue::Null, FuzzValue::Null) => true,
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

fn float_eq(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else {
        a == b
    }
}

// ---------------------------------------------------------------------------
// Document ↔ serde_json::Value conversion for differential checks
// ---------------------------------------------------------------------------

pub fn document_to_serde_value(doc: &aws_smithy_types::Document) -> serde_json::Value {
    match doc {
        aws_smithy_types::Document::Null => serde_json::Value::Null,
        aws_smithy_types::Document::Bool(b) => serde_json::Value::Bool(*b),
        aws_smithy_types::Document::Number(n) => match *n {
            aws_smithy_types::Number::PosInt(v) => {
                serde_json::Value::Number(serde_json::Number::from(v))
            }
            aws_smithy_types::Number::NegInt(v) => {
                serde_json::Value::Number(serde_json::Number::from(v))
            }
            aws_smithy_types::Number::Float(v) => serde_json::Number::from_f64(v)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
        },
        aws_smithy_types::Document::String(s) => serde_json::Value::String(s.clone()),
        aws_smithy_types::Document::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(document_to_serde_value).collect())
        }
        aws_smithy_types::Document::Object(map) => {
            let m: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), document_to_serde_value(v)))
                .collect();
            serde_json::Value::Object(m)
        }
    }
}

/// Returns true if the FuzzValue contains non-finite floats (NaN, Infinity)
/// which serialize as strings and can't roundtrip through numeric deserialization.
pub fn contains_non_finite_float(value: &FuzzValue) -> bool {
    match value {
        FuzzValue::Float(f) => !f.is_finite(),
        FuzzValue::Double(f) => !f.is_finite(),
        _ => false,
    }
}
