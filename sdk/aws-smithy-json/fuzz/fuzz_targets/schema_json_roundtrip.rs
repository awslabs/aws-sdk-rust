/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: roundtrip correctness for schema-based JSON serde.
//!
//! # Strategy
//!
//! libfuzzer generates raw `&[u8]`, but the `Arbitrary` derive on `FuzzValue`
//! interprets those bytes as structured Smithy-typed values (booleans, integers,
//! strings, etc.). The fuzzer learns which byte patterns produce interesting
//! variants and explores them.
//!
//! Schema matching is guaranteed because we control both sides:
//! `serialize_fuzz_value` picks the write method based on the variant (e.g.,
//! `FuzzValue::Integer` → `write_integer(&INTEGER, v)`), and
//! `deserialize_fuzz_value` uses the *original variant* to pick the matching
//! read method (e.g., `FuzzValue::Integer(_)` → `read_integer(&INTEGER)`).
//! The schema always matches the data because the same variant drives both.
//!
//! List/Map/Null variants are skipped because `FuzzSimpleValue` elements can
//! be any type (boolean, integer, etc.) but the list/map schemas are
//! string-only. Deserializing a serialized `[false]` through `STRING_LIST_SCHEMA`
//! would correctly fail with a type mismatch — that's not a bug, just a
//! limitation of the test setup.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use aws_smithy_json::codec::JsonCodec;
use aws_smithy_schema::codec::Codec;
use schema_common::*;

fuzz_target!(|value: FuzzValue| {
    // Test both codec configurations to exercise different JsonFieldMapper paths.
    roundtrip(&value, default_codec()); // use_json_name: true  (restJson1)
    roundtrip(&value, no_json_name_codec()); // use_json_name: false (awsJson1_0/1_1)
});

fn roundtrip(value: &FuzzValue, codec: JsonCodec) {
    // Serialize: variant determines which write_* method and schema are used.
    let mut ser = codec.create_serializer();
    serialize_fuzz_value(value, &mut ser);
    let bytes = ser.finish();

    // Deserialize: the original variant determines which read_* method to call,
    // guaranteeing the schema matches what was serialized.
    let mut deser = codec.create_deserializer(&bytes);
    match deserialize_fuzz_value(&mut deser, value) {
        Ok(deserialized) => {
            assert!(
                fuzz_values_equal(value, &deserialized),
                "Roundtrip mismatch!\nOriginal: {:?}\nSerialized: {:?}\nDeserialized: {:?}",
                value,
                String::from_utf8_lossy(&bytes),
                deserialized,
            );
        }
        Err(e) => {
            panic!(
                "Deserialization failed!\nOriginal: {:?}\nSerialized: {:?}\nError: {:?}",
                value,
                String::from_utf8_lossy(&bytes),
                e,
            );
        }
    }
}
