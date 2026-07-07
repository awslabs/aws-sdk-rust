/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: crash-freedom + differential correctness for schema-based JSON deserializer.
//!
//! # Strategy
//!
//! libfuzzer feeds completely arbitrary `&[u8]` — no structure, no guarantees.
//! We throw the same raw bytes at *every* `read_*` method with a fixed schema.
//! Each call creates a fresh deserializer from the same bytes. Most calls will
//! return `Err` (the bytes aren't valid JSON for that type) — that's expected.
//! The invariant is that **no call may panic** regardless of input.
//!
//! Schema matching is irrelevant here: we're not testing that the deserializer
//! produces correct values, only that it doesn't crash. The schemas are fixed
//! constants (e.g., `ALL_TYPES_SCHEMA` for structs, `STRING_LIST_SCHEMA` for
//! lists) and never change between runs.
//!
//! The one exception is the **differential check** at the bottom: when
//! `serde_json` successfully parses the input as valid JSON, we also parse it
//! with `read_document` and compare results. `read_document` doesn't need a
//! matching schema because the Document type is self-describing — it accepts
//! any JSON structure, like `serde_json::Value`.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::serde::ShapeDeserializer;
use schema_common::*;

fuzz_target!(|data: &[u8]| {
    let codec = default_codec();

    // --- Crash-freedom: every read_* method must not panic on arbitrary input ---
    // Each call gets a fresh deserializer so they don't interfere with each other.
    // Errors are discarded — we only care that the code doesn't panic.

    // Simple types
    let _ = codec
        .create_deserializer(data)
        .read_boolean(&aws_smithy_schema::prelude::BOOLEAN);
    let _ = codec
        .create_deserializer(data)
        .read_byte(&aws_smithy_schema::prelude::BYTE);
    let _ = codec
        .create_deserializer(data)
        .read_short(&aws_smithy_schema::prelude::SHORT);
    let _ = codec
        .create_deserializer(data)
        .read_integer(&aws_smithy_schema::prelude::INTEGER);
    let _ = codec
        .create_deserializer(data)
        .read_long(&aws_smithy_schema::prelude::LONG);
    let _ = codec
        .create_deserializer(data)
        .read_float(&aws_smithy_schema::prelude::FLOAT);
    let _ = codec
        .create_deserializer(data)
        .read_double(&aws_smithy_schema::prelude::DOUBLE);
    let _ = codec
        .create_deserializer(data)
        .read_string(&aws_smithy_schema::prelude::STRING);
    let _ = codec
        .create_deserializer(data)
        .read_blob(&aws_smithy_schema::prelude::BLOB);
    let _ = codec
        .create_deserializer(data)
        .read_timestamp(&aws_smithy_schema::prelude::TIMESTAMP);
    let _ = codec
        .create_deserializer(data)
        .read_big_integer(&aws_smithy_schema::prelude::BIG_INTEGER);
    let _ = codec
        .create_deserializer(data)
        .read_big_decimal(&aws_smithy_schema::prelude::BIG_DECIMAL);
    let _ = codec
        .create_deserializer(data)
        .read_document(&aws_smithy_schema::prelude::DOCUMENT);

    // Null
    let _ = codec.create_deserializer(data).is_null();
    let _ = codec.create_deserializer(data).read_null();

    // Aggregates — consumers propagate errors to avoid infinite loops
    // (if the consumer swallows an error, the deserializer position doesn't
    // advance and the loop in read_list/read_map spins forever).
    let _ = codec
        .create_deserializer(data)
        .read_struct(&ALL_TYPES_SCHEMA, &mut |_member, _deser| Ok(()));
    let _ = codec
        .create_deserializer(data)
        .read_list(&STRING_LIST_SCHEMA, &mut |deser| {
            deser.read_string(&aws_smithy_schema::prelude::STRING)?;
            Ok(())
        });
    let _ = codec
        .create_deserializer(data)
        .read_map(&STRING_MAP_SCHEMA, &mut |_key, deser| {
            deser.read_string(&aws_smithy_schema::prelude::STRING)?;
            Ok(())
        });

    // Collection helpers
    let _ = codec
        .create_deserializer(data)
        .read_string_list(&STRING_LIST_SCHEMA);
    let _ = codec
        .create_deserializer(data)
        .read_blob_list(&aws_smithy_schema::prelude::BLOB);
    let _ = codec
        .create_deserializer(data)
        .read_integer_list(&aws_smithy_schema::prelude::INTEGER);
    let _ = codec
        .create_deserializer(data)
        .read_long_list(&aws_smithy_schema::prelude::LONG);
    let _ = codec
        .create_deserializer(data)
        .read_string_string_map(&STRING_MAP_SCHEMA);

    // Struct with @jsonName — exercises the JsonFieldMapper code path
    let _ = codec
        .create_deserializer(data)
        .read_struct(&RENAMED_SCHEMA, &mut |_member, _deser| Ok(()));

    // --- Differential check: compare read_document against serde_json ---
    // read_document is self-describing (like serde_json::Value), so no schema
    // matching is needed. If both parsers accept the input, they must agree on
    // the result. We re-serialize through serde first to normalize whitespace
    // and float representation.
    if let Ok(serde_value) = serde_json::from_slice::<serde_json::Value>(data) {
        let normalized = serde_json::to_string(&serde_value).unwrap();
        let mut deser = codec.create_deserializer(normalized.as_bytes());
        if let Ok(doc) = deser.read_document(&aws_smithy_schema::prelude::DOCUMENT) {
            let converted = document_to_serde_value(&doc);
            assert_eq!(
                serde_value, converted,
                "Differential mismatch!\nInput: {:?}\nserde: {:?}\nschema: {:?}",
                normalized, serde_value, converted
            );
        }
    }
});
