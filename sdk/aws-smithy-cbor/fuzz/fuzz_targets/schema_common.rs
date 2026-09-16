/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared schemas, types, and helpers for schema-based CBOR fuzz targets.

#![allow(dead_code)]

use arbitrary::Arbitrary;
use aws_smithy_cbor::codec::CborCodec;
use aws_smithy_schema::serde::{ShapeDeserializer, ShapeSerializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};

pub static STRING_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "StringList"),
    &aws_smithy_schema::prelude::STRING,
);

pub static INTEGER_LIST_SCHEMA: Schema = Schema::new_list(
    shape_id!("test", "IntList"),
    &aws_smithy_schema::prelude::INTEGER,
);

pub static STRING_MAP_SCHEMA: Schema = Schema::new_map(
    shape_id!("test", "StringMap"),
    &aws_smithy_schema::prelude::STRING,
    &aws_smithy_schema::prelude::STRING,
);

static MEMBER_BOOL: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::Boolean, "b", 0);
static MEMBER_INT: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::Integer, "i", 1);
static MEMBER_STR: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::String, "s", 2);
static MEMBER_BLOB: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::Blob, "bl", 3);

pub static STRUCT_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "S"),
    ShapeType::Structure,
    &[&MEMBER_BOOL, &MEMBER_INT, &MEMBER_STR, &MEMBER_BLOB],
);

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
    Null,
}

pub fn default_codec() -> CborCodec {
    CborCodec::default()
}

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
            let _ = ser.write_blob(&BLOB, v);
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
        FuzzValue::Null => {
            let _ = ser.write_null(&STRING);
        }
    }
}

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
        FuzzValue::Blob(_) => Ok(FuzzValue::Blob(deser.read_blob(&BLOB)?.into_inner())),
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
        FuzzValue::Null => {
            if deser.is_null() {
                deser.read_null()?;
            }
            Ok(FuzzValue::Null)
        }
    }
}

pub fn fuzz_values_equal(a: &FuzzValue, b: &FuzzValue) -> bool {
    match (a, b) {
        (FuzzValue::Float(a), FuzzValue::Float(b)) => (a.is_nan() && b.is_nan()) || a == b,
        (FuzzValue::Double(a), FuzzValue::Double(b)) => (a.is_nan() && b.is_nan()) || a == b,
        (FuzzValue::Boolean(a), FuzzValue::Boolean(b)) => a == b,
        (FuzzValue::Byte(a), FuzzValue::Byte(b)) => a == b,
        (FuzzValue::Short(a), FuzzValue::Short(b)) => a == b,
        (FuzzValue::Integer(a), FuzzValue::Integer(b)) => a == b,
        (FuzzValue::Long(a), FuzzValue::Long(b)) => a == b,
        (FuzzValue::String(a), FuzzValue::String(b)) => a == b,
        (FuzzValue::Blob(a), FuzzValue::Blob(b)) => a == b,
        (FuzzValue::StringList(a), FuzzValue::StringList(b)) => a == b,
        (FuzzValue::IntegerList(a), FuzzValue::IntegerList(b)) => a == b,
        (FuzzValue::Null, FuzzValue::Null) => true,
        _ => false,
    }
}
