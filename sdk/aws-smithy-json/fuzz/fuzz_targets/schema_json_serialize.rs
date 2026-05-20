/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: serializer output validity for schema-based JSON serde.
//!
//! # Strategy
//!
//! Like the roundtrip target, libfuzzer generates raw bytes that `Arbitrary`
//! interprets as structured `FuzzValue` inputs. `serialize_fuzz_value` picks
//! the write method and schema based on the variant, so the schema always
//! matches the data.
//!
//! We don't deserialize — we only check that `serde_json` can parse the
//! serializer's output. This catches the serializer producing malformed JSON
//! (missing commas, broken string escaping, unclosed brackets, incorrect
//! map key/value alternation, etc.). `serde_json` accepts any valid JSON
//! structure, so no schema matching is needed on the output side.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use aws_smithy_json::codec::JsonCodec;
use aws_smithy_schema::codec::Codec;
use schema_common::*;

fuzz_target!(|value: FuzzValue| {
    // Test both codec configurations to exercise different serializer paths.
    check_valid_json(&value, default_codec()); // use_json_name: true
    check_valid_json(&value, no_json_name_codec()); // use_json_name: false
});

fn check_valid_json(value: &FuzzValue, codec: JsonCodec) {
    // Serialize: variant determines which write_* method and schema are used.
    let mut ser = codec.create_serializer();
    serialize_fuzz_value(value, &mut ser);
    let bytes = ser.finish();

    // Validate: the output must be parseable as JSON by serde_json.
    if serde_json::from_slice::<serde_json::Value>(&bytes).is_err() {
        panic!(
            "JsonSerializer produced invalid JSON!\nInput: {:?}\nOutput: {:?}",
            value,
            String::from_utf8_lossy(&bytes),
        );
    }
}
