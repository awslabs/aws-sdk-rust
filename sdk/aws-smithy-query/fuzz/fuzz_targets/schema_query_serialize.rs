/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![no_main]

use arbitrary::Arbitrary;
use aws_smithy_query::codec::serializer::QueryShapeSerializer;
use aws_smithy_schema::codec::FinishSerializer;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_types::{Blob, DateTime};
use libfuzzer_sys::fuzz_target;

static STR_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::String, "Str", 0);
static INT_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Integer, "Int", 1);
static LIST_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::List, "List", 2);
static MAP_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Map, "Map", 3);
static TS_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Timestamp, "Ts", 4);
static BLOB_MEMBER: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Blob, "Blob", 5);
// A list whose elements are themselves lists — exercises the nested-aggregate
// path where codegen passes `prelude::DOCUMENT` for the inner list.
static NESTED_LIST_MEMBER: Schema =
    Schema::new_member(shape_id!("t", "S"), ShapeType::List, "Nested", 6);
static SCHEMA: Schema = Schema::new_struct(
    shape_id!("t", "S"),
    ShapeType::Structure,
    &[
        &STR_MEMBER,
        &INT_MEMBER,
        &LIST_MEMBER,
        &MAP_MEMBER,
        &TS_MEMBER,
        &BLOB_MEMBER,
        &NESTED_LIST_MEMBER,
    ],
);

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    action: String,
    version: String,
    str_val: String,
    int_val: i32,
    list_vals: Vec<String>,
    map_vals: Vec<(String, String)>,
    ts_secs: i64,
    blob_val: Vec<u8>,
    nested_vals: Vec<Vec<String>>,
}

struct FuzzStruct<'a>(&'a FuzzInput);

impl SerializableStruct for FuzzStruct<'_> {
    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        let input = self.0;
        s.write_string(&STR_MEMBER, &input.str_val)?;
        s.write_integer(&INT_MEMBER, input.int_val)?;
        if !input.list_vals.is_empty() {
            s.write_list(&LIST_MEMBER, &|s| {
                for v in &input.list_vals {
                    s.write_string(&aws_smithy_schema::prelude::STRING, v)?;
                }
                Ok(())
            })?;
        }
        if !input.map_vals.is_empty() {
            s.write_map(&MAP_MEMBER, &|s| {
                for (k, v) in &input.map_vals {
                    s.write_string(&aws_smithy_schema::prelude::STRING, k)?;
                    s.write_string(&aws_smithy_schema::prelude::STRING, v)?;
                }
                Ok(())
            })?;
        }
        // DateTime::from_secs clamps internally; any i64 is a valid input.
        s.write_timestamp(&TS_MEMBER, &DateTime::from_secs(input.ts_secs))?;
        s.write_blob(&BLOB_MEMBER, Blob::new(input.blob_val.clone()).as_ref())?;
        if !input.nested_vals.is_empty() {
            // Outer list whose elements are inner lists — codegen emits the
            // inner `write_list` with `prelude::DOCUMENT`.
            s.write_list(&NESTED_LIST_MEMBER, &|s| {
                for inner in &input.nested_vals {
                    s.write_list(&aws_smithy_schema::prelude::DOCUMENT, &|s| {
                        for v in inner {
                            s.write_string(&aws_smithy_schema::prelude::STRING, v)?;
                        }
                        Ok(())
                    })?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

fuzz_target!(|input: FuzzInput| {
    let mut ser = QueryShapeSerializer::new(&input.action, &input.version);
    let _ = ser.write_struct(&SCHEMA, &FuzzStruct(&input));
    let output = ser.finish();

    // 1. Output must be valid UTF-8 (URL-encoded form data).
    let s = std::str::from_utf8(&output).expect("output must be valid UTF-8");

    // 2. Body is always `Action=..&Version=..` followed by zero or more `&`-joined params.
    assert!(
        s.starts_with("Action="),
        "output must start with Action=, got {s:?}"
    );

    // 3. No raw control characters or spaces may leak into the form body — all
    //    values are percent-encoded, so the wire form is pure ASCII with no
    //    whitespace/control bytes (which would corrupt the query string).
    assert!(
        !s.bytes().any(|b| b == b' ' || b.is_ascii_control()),
        "output must not contain raw spaces or control chars: {s:?}"
    );

    // 4. Every `&`-separated segment is a `key=value` pair with a non-empty key.
    for pair in s.split('&') {
        let key = pair.split('=').next().unwrap_or("");
        assert!(
            !key.is_empty(),
            "empty param name in {pair:?} (full: {s:?})"
        );
    }
});
