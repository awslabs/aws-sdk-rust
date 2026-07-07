/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: crash-freedom for schema-based CBOR deserializer.
//!
//! Feeds arbitrary bytes to every read_* method. The invariant is that
//! no call may panic regardless of input.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::serde::ShapeDeserializer;
use schema_common::*;

fuzz_target!(|data: &[u8]| {
    let codec = default_codec();

    // Simple types — must not panic
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

    // Null
    let _ = codec.create_deserializer(data).is_null();
    let _ = codec.create_deserializer(data).read_null();

    // Aggregates
    let _ = codec
        .create_deserializer(data)
        .read_struct(&STRUCT_SCHEMA, &mut |_member, _deser| Ok(()));
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
});
