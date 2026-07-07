/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: roundtrip correctness for schema-based CBOR serde.
//!
//! Generates structured FuzzValues via Arbitrary, serializes them with
//! CborSerializer, then deserializes and checks equality.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use aws_smithy_cbor::codec::CborCodec;
use aws_smithy_schema::codec::Codec;
use schema_common::*;

fuzz_target!(|value: FuzzValue| {
    let codec = default_codec();
    roundtrip(&value, &codec);
});

fn roundtrip(value: &FuzzValue, codec: &CborCodec) {
    use aws_smithy_schema::codec::FinishSerializer;

    let mut ser = codec.create_serializer();
    serialize_fuzz_value(value, &mut ser);
    let bytes = ser.finish();

    let mut deser = codec.create_deserializer(&bytes);
    match deserialize_fuzz_value(&mut deser, value) {
        Ok(deserialized) => {
            assert!(
                fuzz_values_equal(value, &deserialized),
                "Roundtrip mismatch!\nOriginal: {:?}\nDeserialized: {:?}",
                value,
                deserialized,
            );
        }
        Err(e) => {
            panic!(
                "Deserialization failed!\nOriginal: {:?}\nError: {:?}",
                value, e,
            );
        }
    }
}
