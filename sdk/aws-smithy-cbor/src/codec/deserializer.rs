/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! CBOR deserializer implementation.

use aws_smithy_schema::serde::{capped_container_size, SerdeError, ShapeDeserializer};
use aws_smithy_schema::{Schema, ShapeType};
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

use crate::data::Type;

/// CBOR deserializer that implements the ShapeDeserializer trait.
///
/// Wraps the existing `Decoder` which handles both definite and
/// indefinite-length strings/blobs and includes millisecond-precision
/// timestamp truncation.
pub struct CborDeserializer<'a> {
    decoder: crate::Decoder<'a>,
    input_len: usize,
    depth: u32,
    max_depth: u32,
    enforce_strictness: bool,
}

impl<'a> CborDeserializer<'a> {
    pub(crate) fn new(input: &'a [u8], max_depth: u32) -> Self {
        Self {
            decoder: crate::Decoder::new(input),
            input_len: input.len(),
            depth: 0,
            max_depth,
            enforce_strictness: false,
        }
    }

    pub(crate) fn with_strictness(mut self, value: bool) -> Self {
        self.enforce_strictness = value;
        self
    }

    fn leave_container(&mut self) -> Result<(), SerdeError> {
        self.depth -= 1;
        if self.enforce_strictness && self.depth == 0 && self.decoder.position() != self.input_len {
            return Err(SerdeError::invalid_input("trailing bytes after CBOR value"));
        }
        Ok(())
    }

    fn check_depth(&mut self) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.max_depth {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        Ok(())
    }

    /// Returns true if the current CBOR item is a break code (end of indefinite container).
    fn is_break(&self) -> bool {
        matches!(self.decoder.datatype(), Ok(Type::Break))
    }

    /// Skips the break code at the end of an indefinite-length container.
    fn consume_break(&mut self) -> Result<(), SerdeError> {
        self.decoder.skip().map_err(deser_err)
    }

    /// Reads a list of items using the provided element reader, handling
    /// both definite and indefinite-length arrays.
    fn read_list_items<T>(
        &mut self,
        mut read_element: impl FnMut(&mut crate::Decoder<'_>) -> Result<T, SerdeError>,
    ) -> Result<Vec<T>, SerdeError> {
        self.check_depth()?;
        let len = self.decoder.list().map_err(deser_err)?;
        let is_indefinite = len.is_none();
        let count = len.unwrap_or(0) as usize;
        let mut out = Vec::with_capacity(capped_container_size(count));
        let mut i = 0;
        loop {
            if !is_indefinite && i >= count {
                break;
            }
            if is_indefinite && self.is_break() {
                self.consume_break()?;
                break;
            }
            out.push(read_element(&mut self.decoder)?);
            i += 1;
        }
        self.leave_container()?;
        Ok(out)
    }
}

impl ShapeDeserializer for CborDeserializer<'_> {
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Empty input (e.g., empty HTTP response body) is treated as an empty struct
        if self.decoder.position() >= self.input_len {
            if self.enforce_strictness && self.depth != 0 {
                return Err(SerdeError::invalid_input("truncated CBOR structure"));
            }
            return Ok(());
        }
        self.check_depth()?;
        let len = self.decoder.map().map_err(deser_err)?;
        let is_indefinite = len.is_none();
        let count = len.unwrap_or(0) as usize;

        let is_union = schema.shape_type() == ShapeType::Union;
        let mut i = 0;
        loop {
            if !is_indefinite && i >= count {
                break;
            }
            if is_indefinite && self.is_break() {
                self.consume_break()?;
                break;
            }
            let key = self.decoder.str().map_err(deser_err)?;
            if self.enforce_strictness && !is_union && self.is_null() {
                // A null structure member means the member is absent; the consumer
                // never sees it, so builder defaults stay untouched. Unions are
                // excluded: a null variant value must reach the consumer to be
                // rejected there.
                self.read_null()?;
            } else if let Some(member_schema) = schema.member_schema(&key) {
                consumer(member_schema, self)?;
            } else if &*key == "__type" || self.is_null() {
                // A protocol discriminator is never a member, and a `null` value is an
                // absent member. Neither is reported.
                self.decoder.skip().map_err(deser_err)?;
            } else {
                // Report the unknown key so a union consumer can act on it: the prelude
                // `DOCUMENT` schema has no member index, so generated code lands in its
                // fallthrough arm. The consumer may read the value with a typed read; if
                // it does not, the position is unchanged and the value is skipped here.
                let start = self.decoder.position();
                consumer(&aws_smithy_schema::prelude::DOCUMENT, self)?;
                if self.decoder.position() == start {
                    self.decoder.skip().map_err(deser_err)?;
                }
            }
            i += 1;
        }
        self.leave_container()?;
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.check_depth()?;
        let len = self.decoder.list().map_err(deser_err)?;
        let is_indefinite = len.is_none();
        let count = len.unwrap_or(0) as usize;

        let mut i = 0;
        loop {
            if !is_indefinite && i >= count {
                break;
            }
            if is_indefinite && self.is_break() {
                self.consume_break()?;
                break;
            }
            consumer(self)?;
            i += 1;
        }
        self.leave_container()?;
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.check_depth()?;
        let len = self.decoder.map().map_err(deser_err)?;
        let is_indefinite = len.is_none();
        let count = len.unwrap_or(0) as usize;

        let mut i = 0;
        loop {
            if !is_indefinite && i >= count {
                break;
            }
            if is_indefinite && self.is_break() {
                self.consume_break()?;
                break;
            }
            let key = self.decoder.str().map_err(deser_err)?.into_owned();
            consumer(key, self)?;
            i += 1;
        }
        self.leave_container()?;
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.decoder.boolean().map_err(deser_err)
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.decoder.byte().map_err(deser_err)
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.decoder.short().map_err(deser_err)
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.decoder.integer().map_err(deser_err)
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.decoder.long().map_err(deser_err)
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.decoder.float().map_err(deser_err)
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.decoder.double().map_err(deser_err)
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        Err(SerdeError::unsupported(
            "CBOR big integer not yet supported (smithy-rs#4611)",
        ))
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        Err(SerdeError::unsupported(
            "CBOR big decimal not yet supported (smithy-rs#4611)",
        ))
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.decoder
            .str()
            .map(|cow| cow.into_owned())
            .map_err(deser_err)
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        self.decoder.blob().map_err(deser_err)
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        self.decoder.timestamp().map_err(deser_err)
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Err(SerdeError::unsupported(
            "document types are not supported by rpcv2Cbor protocol",
        ))
    }

    fn is_null(&self) -> bool {
        matches!(self.decoder.datatype(), Ok(Type::Null))
    }

    fn read_null(&mut self) -> Result<(), SerdeError> {
        self.decoder.null().map_err(deser_err)
    }

    fn container_size(&self) -> Option<usize> {
        let mut peek = self.decoder.clone();
        match peek.datatype().ok()? {
            Type::Array | Type::ArrayIndef => {
                peek.list().ok()?.map(|n| capped_container_size(n as usize))
            }
            Type::Map | Type::MapIndef => {
                peek.map().ok()?.map(|n| capped_container_size(n as usize))
            }
            _ => None,
        }
    }

    fn read_string_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        self.read_list_items(|dec| dec.str().map(|c| c.into_owned()).map_err(deser_err))
    }

    fn read_blob_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<Blob>, SerdeError> {
        self.read_list_items(|dec| dec.blob().map_err(deser_err))
    }

    fn read_integer_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        self.read_list_items(|dec| dec.integer().map_err(deser_err))
    }

    fn read_long_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        self.read_list_items(|dec| dec.long().map_err(deser_err))
    }

    fn read_string_string_map(
        &mut self,
        _schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        self.check_depth()?;
        let len = self.decoder.map().map_err(deser_err)?;
        let is_indefinite = len.is_none();
        let count = len.unwrap_or(0) as usize;
        let mut out = std::collections::HashMap::with_capacity(capped_container_size(count));
        let mut i = 0;
        loop {
            if !is_indefinite && i >= count {
                break;
            }
            if is_indefinite && self.is_break() {
                self.consume_break()?;
                break;
            }
            let key = self
                .decoder
                .str()
                .map(|c| c.into_owned())
                .map_err(deser_err)?;
            let val = self
                .decoder
                .str()
                .map(|c| c.into_owned())
                .map_err(deser_err)?;
            out.insert(key, val);
            i += 1;
        }
        self.leave_container()?;
        Ok(out)
    }
}

fn deser_err(e: crate::decode::DeserializeError) -> SerdeError {
    SerdeError::invalid_input(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::codec::Codec;
    use aws_smithy_schema::prelude::*;
    use aws_smithy_schema::serde::{SerializableStruct, ShapeSerializer};
    use aws_smithy_schema::{shape_id, ShapeType};

    use crate::codec::{CborCodec, CborCodecSettings};

    /// Helper: serialize with CborSerializer, then deserialize with CborDeserializer.
    fn make_deser(f: impl FnOnce(&mut crate::codec::CborSerializer)) -> Vec<u8> {
        use aws_smithy_schema::codec::FinishSerializer;
        let codec = CborCodec::default();
        let mut ser = codec.create_serializer();
        f(&mut ser);
        ser.finish()
    }

    #[test]
    fn strict_null_struct_members_are_skipped_as_absent() {
        static X: Schema =
            Schema::new_member(shape_id!("test", "S", "x"), ShapeType::Integer, "x", 0);
        static S: Schema = Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&X]);
        for strict in [false, true] {
            let codec = CborCodec::new(CborCodecSettings::default().enforce_strictness(strict));
            // Strict: the codec consumes the null itself, the consumer never runs
            // and defaults stay untouched. Lenient: the consumer sees the null and
            // fails here by requiring an integer.
            let mut visited = false;
            let result = codec.create_deserializer(b"\xa1\x61x\xf6").read_struct(
                &S,
                &mut |member, deser| {
                    visited = true;
                    deser.read_integer(member).map(|_| ())
                },
            );
            assert_eq!(result.is_ok(), strict);
            assert_eq!(visited, !strict);
        }
    }

    #[test]
    fn strict_top_level_containers() {
        for input in [
            vec![0xa0],
            vec![0xbf, 0xff],
            vec![0xa1, 0x61, b'x', 0x81, 0xa0],
            vec![0xbf, 0x61, b'x', 0x9f, 0xbf, 0xff, 0xff, 0xff],
        ] {
            for strict in [false, true] {
                let mut de = CborDeserializer::new(&input, 128).with_strictness(strict);
                de.read_struct(&STRING, &mut |_, _| Ok(())).unwrap();
                let mut trailing = input.clone();
                trailing.push(0);
                let mut de = CborDeserializer::new(&trailing, 128).with_strictness(strict);
                assert_eq!(de.read_struct(&STRING, &mut |_, _| Ok(())).is_err(), strict);
            }
        }
    }

    #[test]
    fn test_read_boolean() {
        let bytes = make_deser(|s| s.write_boolean(&BOOLEAN, true).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert!(de.read_boolean(&BOOLEAN).unwrap());
    }

    #[test]
    fn test_read_byte() {
        let bytes = make_deser(|s| s.write_byte(&BYTE, -42).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_byte(&BYTE).unwrap(), -42);
    }

    #[test]
    fn test_read_short() {
        let bytes = make_deser(|s| s.write_short(&SHORT, 1234).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_short(&SHORT).unwrap(), 1234);
    }

    #[test]
    fn test_read_integer() {
        let bytes = make_deser(|s| s.write_integer(&INTEGER, -99999).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_integer(&INTEGER).unwrap(), -99999);
    }

    #[test]
    fn test_read_long() {
        let bytes = make_deser(|s| s.write_long(&LONG, i64::MIN).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_long(&LONG).unwrap(), i64::MIN);
    }

    #[test]
    fn test_read_float_nan() {
        let bytes = make_deser(|s| s.write_float(&FLOAT, f32::NAN).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert!(de.read_float(&FLOAT).unwrap().is_nan());
    }

    #[test]
    fn test_read_double_neg_infinity() {
        let bytes = make_deser(|s| s.write_double(&DOUBLE, f64::NEG_INFINITY).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_double(&DOUBLE).unwrap(), f64::NEG_INFINITY);
    }

    #[test]
    fn test_read_string() {
        let bytes = make_deser(|s| s.write_string(&STRING, "hello world").unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_string(&STRING).unwrap(), "hello world");
    }

    #[test]
    fn test_read_blob() {
        let blob = Blob::new(b"binary");
        let bytes = make_deser(|s| s.write_blob(&BLOB, blob.clone()).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_blob(&BLOB).unwrap(), blob);
    }

    #[test]
    fn test_read_timestamp() {
        let ts = DateTime::from_secs_f64(1700000000.123);
        let bytes = make_deser(|s| s.write_timestamp(&TIMESTAMP, &ts).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        let decoded = de.read_timestamp(&TIMESTAMP).unwrap();
        // Millisecond truncation
        assert_eq!(decoded.subsec_nanos() % 1_000_000, 0);
    }

    #[test]
    fn test_is_null() {
        let bytes = make_deser(|s| s.write_null(&STRING).unwrap());
        let de = CborDeserializer::new(&bytes, 128);
        assert!(de.is_null());
    }

    #[test]
    fn test_read_null() {
        let bytes = make_deser(|s| s.write_null(&STRING).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        de.read_null().unwrap();
    }

    #[test]
    fn test_read_struct() {
        static NAME: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::String, "name", 0);
        static AGE: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::Integer, "age", 1);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&NAME, &AGE]);

        struct TestStruct;
        impl SerializableStruct for TestStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME, "Bob")?;
                s.write_integer(&AGE, 25)?;
                Ok(())
            }
        }

        let bytes = make_deser(|s| s.write_struct(&SCHEMA, &TestStruct).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut name = String::new();
        let mut age = 0i32;
        de.read_struct(&SCHEMA, &mut |member, d| {
            match member.member_name() {
                Some("name") => name = d.read_string(member)?,
                Some("age") => age = d.read_integer(member)?,
                _ => {}
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(name, "Bob");
        assert_eq!(age, 25);
    }

    #[test]
    fn test_read_struct_unknown_members_skipped() {
        // Build CBOR with an extra field not in the schema
        let mut enc = crate::Encoder::new(Vec::new());
        enc.begin_map()
            .str("name")
            .str("Alice")
            .str("unknown_field")
            .integer(999)
            .str("age")
            .integer(30)
            .end();
        let bytes = enc.into_writer();

        static NAME: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::String, "name", 0);
        static AGE: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::Integer, "age", 1);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&NAME, &AGE]);

        let mut de = CborDeserializer::new(&bytes, 128);
        let mut name = String::new();
        let mut age = 0i32;
        de.read_struct(&SCHEMA, &mut |member, d| {
            match member.member_name() {
                Some("name") => name = d.read_string(member)?,
                Some("age") => age = d.read_integer(member)?,
                _ => {}
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(name, "Alice");
        assert_eq!(age, 30);
    }

    #[test]
    fn test_read_struct_definite_length_map() {
        // Services may send definite-length maps
        let mut enc = crate::Encoder::new(Vec::new());
        enc.map(1).str("name").str("Charlie");
        let bytes = enc.into_writer();

        static NAME: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::String, "name", 0);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&NAME]);

        let mut de = CborDeserializer::new(&bytes, 128);
        let mut name = String::new();
        de.read_struct(&SCHEMA, &mut |member, d| {
            if member.member_name() == Some("name") {
                name = d.read_string(member)?;
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(name, "Charlie");
    }

    #[test]
    fn test_read_list() {
        let bytes = make_deser(|s| {
            let schema = Schema::new(shape_id!("test", "L"), ShapeType::List);
            s.write_list(&schema, &|s| {
                s.write_integer(&INTEGER, 10)?;
                s.write_integer(&INTEGER, 20)?;
                Ok(())
            })
            .unwrap()
        });
        let list_schema = Schema::new(shape_id!("test", "L"), ShapeType::List);
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut items = Vec::new();
        de.read_list(&list_schema, &mut |d| {
            items.push(d.read_integer(&INTEGER)?);
            Ok(())
        })
        .unwrap();
        assert_eq!(items, vec![10, 20]);
    }

    #[test]
    fn test_read_map() {
        let bytes = make_deser(|s| {
            let schema = Schema::new(shape_id!("test", "M"), ShapeType::Map);
            s.write_map(&schema, &|s| {
                s.write_string(&STRING, "k1")?;
                s.write_string(&STRING, "v1")?;
                s.write_string(&STRING, "k2")?;
                s.write_string(&STRING, "v2")?;
                Ok(())
            })
            .unwrap()
        });
        let map_schema = Schema::new(shape_id!("test", "M"), ShapeType::Map);
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut map = std::collections::HashMap::new();
        de.read_map(&map_schema, &mut |key, d| {
            map.insert(key, d.read_string(&STRING)?);
            Ok(())
        })
        .unwrap();
        assert_eq!(map.get("k1").unwrap(), "v1");
        assert_eq!(map.get("k2").unwrap(), "v2");
    }

    #[test]
    fn test_empty_input_treated_as_empty_struct() {
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[]);
        let mut de = CborDeserializer::new(&[], 128);
        de.read_struct(&SCHEMA, &mut |_, _| Ok(())).unwrap();
    }

    #[test]
    fn test_container_size_definite() {
        let mut enc = crate::Encoder::new(Vec::new());
        enc.array(5);
        let bytes = enc.into_writer();
        let de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.container_size(), Some(5));
    }

    #[test]
    fn test_container_size_indefinite_returns_none() {
        let mut enc = crate::Encoder::new(Vec::new());
        enc.begin_array();
        let bytes = enc.into_writer();
        let de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.container_size(), None);
    }

    #[test]
    fn test_depth_limit_rejects_deeply_nested() {
        // Build deeply nested maps: {"a": {"a": {"a": ...}}}
        let mut enc = crate::Encoder::new(Vec::new());
        for _ in 0..200 {
            enc.begin_map().str("a");
        }
        enc.begin_map().end(); // innermost empty map
        for _ in 0..200 {
            enc.end();
        }
        let bytes = enc.into_writer();

        static MEMBER: Schema =
            Schema::new_member(shape_id!("test", "R"), ShapeType::Structure, "a", 0);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "R"), ShapeType::Structure, &[&MEMBER]);

        let mut de = CborDeserializer::new(&bytes, 128);
        fn recursive_consumer(
            _member: &Schema<'_>,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            static MEMBER: Schema =
                Schema::new_member(shape_id!("test", "R"), ShapeType::Structure, "a", 0);
            static SCHEMA: Schema =
                Schema::new_struct(shape_id!("test", "R"), ShapeType::Structure, &[&MEMBER]);
            deser.read_struct(&SCHEMA, &mut recursive_consumer)
        }
        let result = de.read_struct(&SCHEMA, &mut recursive_consumer);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth"));
    }

    #[test]
    fn test_depth_limit_accepts_under_limit() {
        // 10 levels of nesting — well under 128
        let mut enc = crate::Encoder::new(Vec::new());
        for _ in 0..10 {
            enc.begin_map().str("a");
        }
        enc.begin_map().end();
        for _ in 0..10 {
            enc.end();
        }
        let bytes = enc.into_writer();

        static MEMBER: Schema =
            Schema::new_member(shape_id!("test", "R"), ShapeType::Structure, "a", 0);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "R"), ShapeType::Structure, &[&MEMBER]);

        let mut de = CborDeserializer::new(&bytes, 128);
        fn recursive_consumer(
            _member: &Schema<'_>,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            static MEMBER: Schema =
                Schema::new_member(shape_id!("test", "R"), ShapeType::Structure, "a", 0);
            static SCHEMA: Schema =
                Schema::new_struct(shape_id!("test", "R"), ShapeType::Structure, &[&MEMBER]);
            deser.read_struct(&SCHEMA, &mut recursive_consumer)
        }
        de.read_struct(&SCHEMA, &mut recursive_consumer).unwrap();
    }

    #[test]
    fn test_read_struct_with_null_optional_members() {
        // Struct with null value for an optional member — null should be skipped
        let mut enc = crate::Encoder::new(Vec::new());
        enc.begin_map()
            .str("name")
            .str("Alice")
            .str("age")
            .null()
            .end();
        let bytes = enc.into_writer();

        static NAME: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::String, "name", 0);
        static AGE: Schema =
            Schema::new_member(shape_id!("test", "S"), ShapeType::Integer, "age", 1);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&NAME, &AGE]);

        let mut de = CborDeserializer::new(&bytes, 128);
        let mut name: Option<String> = None;
        let mut age: Option<i32> = None;
        de.read_struct(&SCHEMA, &mut |member, d| {
            match member.member_name() {
                Some("name") => {
                    if !d.is_null() {
                        name = Some(d.read_string(member)?);
                    } else {
                        d.read_null()?;
                    }
                }
                Some("age") => {
                    if !d.is_null() {
                        age = Some(d.read_integer(member)?);
                    } else {
                        d.read_null()?;
                    }
                }
                _ => {}
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(name, Some("Alice".to_string()));
        assert_eq!(age, None);
    }

    #[test]
    fn test_read_sparse_list_with_nulls() {
        // @sparse list: [1, null, 3] — null should not be skipped
        let mut enc = crate::Encoder::new(Vec::new());
        enc.begin_array().integer(1).null().integer(3).end();
        let bytes = enc.into_writer();

        let list_schema = Schema::new(shape_id!("test", "L"), ShapeType::List);
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut items: Vec<Option<i32>> = Vec::new();
        de.read_list(&list_schema, &mut |d| {
            if d.is_null() {
                d.read_null()?;
                items.push(None);
            } else {
                items.push(Some(d.read_integer(&INTEGER)?));
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(items, vec![Some(1), None, Some(3)]);
    }

    #[test]
    fn test_read_sparse_map_with_nulls() {
        // @sparse map: {"a": "hello", "b": null} — null should not be skipped
        let mut enc = crate::Encoder::new(Vec::new());
        enc.begin_map().str("a").str("hello").str("b").null().end();
        let bytes = enc.into_writer();

        let map_schema = Schema::new(shape_id!("test", "M"), ShapeType::Map);
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut map: std::collections::HashMap<String, Option<String>> =
            std::collections::HashMap::new();
        de.read_map(&map_schema, &mut |key, d| {
            if d.is_null() {
                d.read_null()?;
                map.insert(key, None);
            } else {
                map.insert(key, Some(d.read_string(&STRING)?));
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(map.get("a"), Some(&Some("hello".to_string())));
        assert_eq!(map.get("b"), Some(&None));
    }

    // --- Union deserialization: union-in-union, empty / unknown-only union ---
    //
    // Mirrors the generated union deserializer: read_struct over a
    // ShapeType::Union schema, dispatch on member_index(), and ok_or_else when
    // no variant was set. A union-valued member recurses into the inner union's
    // own deserialize (its own self-contained read_struct). Because the CBOR
    // read_struct reads its own map header and terminator, the recursion is
    // safe and an empty / unknown-only union produces a clean error, never a
    // panic or a corrupted stream. Both definite- and indefinite-length maps
    // are exercised.

    static U_INNER_LAMBDA: Schema = Schema::new_member(
        shape_id!("test", "InnerUnion"),
        ShapeType::String,
        "lambda",
        0,
    );
    static U_INNER_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "InnerUnion"),
        ShapeType::Union,
        &[&U_INNER_LAMBDA],
    );
    static U_OUTER_MCP: Schema =
        Schema::new_member(shape_id!("test", "OuterUnion"), ShapeType::Union, "mcp", 0);
    static U_OUTER_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "OuterUnion"),
        ShapeType::Union,
        &[&U_OUTER_MCP],
    );

    #[derive(Debug, PartialEq)]
    enum InnerUnion {
        Lambda(String),
        Unknown,
    }
    #[derive(Debug, PartialEq)]
    enum OuterUnion {
        Mcp(InnerUnion),
        Unknown,
    }

    fn deser_inner_union(deser: &mut dyn ShapeDeserializer) -> Result<InnerUnion, SerdeError> {
        let mut result: Option<InnerUnion> = None;
        deser.read_struct(&U_INNER_SCHEMA, &mut |member, d| {
            result = Some(match member.member_index() {
                Some(0) => InnerUnion::Lambda(d.read_string(member)?),
                _ => InnerUnion::Unknown,
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    fn deser_outer_union(deser: &mut dyn ShapeDeserializer) -> Result<OuterUnion, SerdeError> {
        let mut result: Option<OuterUnion> = None;
        deser.read_struct(&U_OUTER_SCHEMA, &mut |member, d| {
            result = Some(match member.member_index() {
                // union-valued member recurses into the inner union's own deserialize
                Some(0) => OuterUnion::Mcp(deser_inner_union(d)?),
                _ => OuterUnion::Unknown,
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    #[test]
    fn union_in_union_definite_map() {
        // outer {mcp: inner {lambda: "arn"}} as definite-length maps.
        let mut e = crate::Encoder::new(Vec::new());
        e.map(1)
            .str("mcp")
            .map(1)
            .str("lambda")
            .str("arn:aws:lambda:fn");
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(
            deser_outer_union(&mut de).expect("union-in-union must deserialize"),
            OuterUnion::Mcp(InnerUnion::Lambda("arn:aws:lambda:fn".to_string()))
        );
    }

    #[test]
    fn union_in_union_indefinite_map() {
        let mut e = crate::Encoder::new(Vec::new());
        e.begin_map()
            .str("mcp")
            .begin_map()
            .str("lambda")
            .str("arn:aws:lambda:fn")
            .end()
            .end();
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(
            deser_outer_union(&mut de).expect("union-in-union (indefinite) must deserialize"),
            OuterUnion::Mcp(InnerUnion::Lambda("arn:aws:lambda:fn".to_string()))
        );
    }

    static U_HOLDER_CHOICE: Schema =
        Schema::new_member(shape_id!("test", "Holder"), ShapeType::Union, "choice", 0);
    static U_HOLDER_TRAILING: Schema = Schema::new_member(
        shape_id!("test", "Holder"),
        ShapeType::String,
        "trailing",
        1,
    );
    static U_HOLDER_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "Holder"),
        ShapeType::Structure,
        &[&U_HOLDER_CHOICE, &U_HOLDER_TRAILING],
    );

    #[test]
    fn nested_union_leaves_decoder_positioned_for_trailing_sibling() {
        // {choice: {mcp: {lambda: "x"}}, trailing: "ok"} — the trailing sibling
        // must still parse after the nested union is read.
        let mut e = crate::Encoder::new(Vec::new());
        e.map(2)
            .str("choice")
            .map(1)
            .str("mcp")
            .map(1)
            .str("lambda")
            .str("x")
            .str("trailing")
            .str("ok");
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut choice: Option<OuterUnion> = None;
        let mut trailing: Option<String> = None;
        de.read_struct(&U_HOLDER_SCHEMA, &mut |member, d| {
            match member.member_index() {
                Some(0) => choice = Some(deser_outer_union(d)?),
                Some(1) => trailing = Some(d.read_string(member)?),
                _ => {}
            }
            Ok(())
        })
        .expect("holder with a nested union must parse");
        assert_eq!(
            choice,
            Some(OuterUnion::Mcp(InnerUnion::Lambda("x".to_string())))
        );
        assert_eq!(
            trailing,
            Some("ok".to_string()),
            "trailing sibling must survive: the nested union read must leave the decoder positioned correctly"
        );
    }

    #[test]
    fn empty_union_definite_map_yields_clean_error() {
        let mut e = crate::Encoder::new(Vec::new());
        e.map(0);
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        let err = deser_inner_union(&mut de).unwrap_err();
        assert!(
            err.to_string().contains("expected a union variant"),
            "empty definite union map must be a clean error, got {err:?}"
        );
    }

    #[test]
    fn empty_union_indefinite_map_yields_clean_error() {
        let mut e = crate::Encoder::new(Vec::new());
        e.begin_map().end();
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        let err = deser_inner_union(&mut de).unwrap_err();
        assert!(
            err.to_string().contains("expected a union variant"),
            "empty indefinite union map must be a clean error, got {err:?}"
        );
    }

    #[test]
    fn union_with_only_unknown_member_is_reported_to_the_consumer() {
        // A union map whose sole key matches no member is reported to the
        // consumer as an unknown member; this consumer maps it to `Unknown`
        // (as a client would) and the value is skipped for it.
        let mut e = crate::Encoder::new(Vec::new());
        e.map(1).str("zzz").str("ignored");
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(deser_inner_union(&mut de).unwrap(), InnerUnion::Unknown);
    }

    #[test]
    fn list_of_union_in_union() {
        let mut e = crate::Encoder::new(Vec::new());
        e.array(2)
            .map(1)
            .str("mcp")
            .map(1)
            .str("lambda")
            .str("a")
            .map(1)
            .str("mcp")
            .map(1)
            .str("lambda")
            .str("b");
        let bytes = e.into_writer();
        let list_schema = Schema::new(shape_id!("test", "L"), ShapeType::List);
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut out = Vec::new();
        de.read_list(&list_schema, &mut |el| {
            out.push(deser_outer_union(el)?);
            Ok(())
        })
        .expect("list of union-in-union must deserialize");
        assert_eq!(
            out,
            vec![
                OuterUnion::Mcp(InnerUnion::Lambda("a".to_string())),
                OuterUnion::Mcp(InnerUnion::Lambda("b".to_string())),
            ]
        );
    }

    #[test]
    fn null_valued_union_member_yields_clean_error_not_panic() {
        // Unlike JSON (whose read_struct skips explicit-null members before
        // dispatching), CBOR read_struct passes the null value straight to the
        // consumer. A realistic CBOR union carries exactly one non-null member,
        // so a null-valued member is malformed; the generated deserializer
        // reads it with the variant's typed read_* and gets a clean type error.
        // The important property for bug parity is that this is a clean error,
        // never a panic or a corrupted stream.
        let mut e = crate::Encoder::new(Vec::new());
        e.map(1).str("lambda").null();
        let bytes = e.into_writer();
        let mut de = CborDeserializer::new(&bytes, 128);
        assert!(
            deser_inner_union(&mut de).is_err(),
            "a null-valued union member must be a clean error, not a panic"
        );
    }

    // --- Required value-type members are serialized even when equal to the
    // zero/default (bool false). Mirrors the generated non-optional branch
    // `{ let val = &self.x; ser.write_boolean(..) }` (unconditional, never a
    // skip-if-default). Shapes the ELB
    // `LoadBalancerAttributes { ConnectionDraining { Enabled = false } }` case:
    // the nested structure member is present and `enabled=false` is on the wire.

    static B_CD_ENABLED: Schema = Schema::new_member(
        shape_id!("test", "ConnectionDraining$enabled"),
        ShapeType::Boolean,
        "enabled",
        0,
    );
    static B_CD_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "ConnectionDraining"),
        ShapeType::Structure,
        &[&B_CD_ENABLED],
    );
    static B_LBA_CD: Schema = Schema::new_member(
        shape_id!("test", "LoadBalancerAttributes$connectionDraining"),
        ShapeType::Structure,
        "connectionDraining",
        0,
    );
    static B_LBA_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "LoadBalancerAttributes"),
        ShapeType::Structure,
        &[&B_LBA_CD],
    );

    struct BConnectionDraining {
        enabled: bool,
    }
    impl SerializableStruct for BConnectionDraining {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            let val = &self.enabled;
            s.write_boolean(&B_CD_ENABLED, *val)
        }
    }
    struct BLoadBalancerAttributes {
        connection_draining: BConnectionDraining,
    }
    impl SerializableStruct for BLoadBalancerAttributes {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_struct(&B_LBA_CD, &self.connection_draining)
        }
    }

    #[test]
    fn required_value_type_false_bool_is_serialized() {
        let bytes = make_deser(|s| {
            s.write_struct(
                &B_LBA_SCHEMA,
                &BLoadBalancerAttributes {
                    connection_draining: BConnectionDraining { enabled: false },
                },
            )
            .unwrap()
        });
        // Read back: `enabled=false` (Some) proves the member was written to the
        // wire; if it had been dropped, the inner consumer would never fire.
        let mut de = CborDeserializer::new(&bytes, 128);
        let mut cd_seen = false;
        let mut enabled: Option<bool> = None;
        de.read_struct(&B_LBA_SCHEMA, &mut |m, d| {
            if m.member_name() == Some("connectionDraining") {
                cd_seen = true;
                d.read_struct(&B_CD_SCHEMA, &mut |im, id| {
                    if im.member_name() == Some("enabled") {
                        enabled = Some(id.read_boolean(im)?);
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
        .unwrap();
        assert!(cd_seen, "outer connectionDraining member must be present");
        assert_eq!(
            enabled,
            Some(false),
            "required value-type enabled=false must be serialized, not dropped"
        );
    }
}

/// `read_struct` reports keys that name no member: the consumer is called with the
/// prelude `DOCUMENT` schema (no member index) positioned at the value. See the JSON
/// codec's `unknown_member_tests` for the full contract; these cover the CBOR specifics
/// (definite and indefinite maps, `null` values, `__type`).
#[cfg(test)]
mod unknown_member_tests {
    use super::*;
    use aws_smithy_schema::{shape_id, ShapeType};

    static A: Schema<'static> =
        Schema::new_member(shape_id!("test", "S"), ShapeType::String, "a", 0);
    static S: Schema<'static> =
        Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&A]);

    /// CBOR has no document type, so a consumer that wants an unknown value reads it with
    /// a typed read. This one reads it as a string when asked.
    fn read_s(bytes: &[u8], read_unknown_value: bool) -> (Option<String>, Vec<Option<String>>) {
        let mut a = None;
        let mut unknown = Vec::new();
        CborDeserializer::new(bytes, 128)
            .read_struct(&S, &mut |member, d| {
                match member.member_index() {
                    Some(0) => a = Some(d.read_string(member)?),
                    _ => {
                        assert_eq!(member.shape_type(), ShapeType::Document);
                        unknown.push(if read_unknown_value {
                            Some(d.read_string(member)?)
                        } else {
                            None
                        });
                    }
                }
                Ok(())
            })
            .unwrap();
        (a, unknown)
    }

    #[test]
    fn unknown_key_is_reported_and_the_consumer_can_read_the_value() {
        let mut e = crate::Encoder::new(Vec::new());
        e.map(2).str("zzz").str("value").str("a").str("x");
        let (a, unknown) = read_s(&e.into_writer(), true);
        assert_eq!(a.as_deref(), Some("x"));
        assert_eq!(unknown, [Some("value".to_string())]);
    }

    #[test]
    fn unknown_key_value_is_skipped_when_not_read_in_definite_and_indefinite_maps() {
        let mut e = crate::Encoder::new(Vec::new());
        e.map(3)
            .str("zzz")
            .map(1)
            .str("deep")
            .array(2)
            .integer(1)
            .integer(2)
            .str("a")
            .str("x")
            .str("yyy")
            .str("tail");
        let (a, unknown) = read_s(&e.into_writer(), false);
        assert_eq!(a.as_deref(), Some("x"));
        assert_eq!(unknown.len(), 2);

        let mut e = crate::Encoder::new(Vec::new());
        e.begin_map()
            .str("zzz")
            .array(2)
            .integer(1)
            .integer(2)
            .str("a")
            .str("x")
            .end();
        let (a, unknown) = read_s(&e.into_writer(), false);
        assert_eq!(a.as_deref(), Some("x"));
        assert_eq!(unknown.len(), 1);
    }

    #[test]
    fn type_discriminator_and_null_values_are_not_reported() {
        let mut e = crate::Encoder::new(Vec::new());
        e.map(3)
            .str("__type")
            .str("ns#Foo")
            .str("zzz")
            .null()
            .str("a")
            .str("x");
        let (a, unknown) = read_s(&e.into_writer(), false);
        assert_eq!(a.as_deref(), Some("x"));
        assert!(unknown.is_empty());
    }
}
