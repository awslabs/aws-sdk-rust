/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! CBOR deserializer implementation.

use aws_smithy_schema::serde::{capped_container_size, SerdeError, ShapeDeserializer};
use aws_smithy_schema::Schema;
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
}

impl<'a> CborDeserializer<'a> {
    pub(crate) fn new(input: &'a [u8], max_depth: u32) -> Self {
        Self {
            decoder: crate::Decoder::new(input),
            input_len: input.len(),
            depth: 0,
            max_depth,
        }
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
        self.depth -= 1;
        Ok(out)
    }
}

impl ShapeDeserializer for CborDeserializer<'_> {
    fn read_struct(
        &mut self,
        schema: &Schema,
        consumer: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Empty input (e.g., empty HTTP response body) is treated as an empty struct
        if self.decoder.position() >= self.input_len {
            return Ok(());
        }
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
            let key = self.decoder.str().map_err(deser_err)?;
            if let Some(member_schema) = schema.member_schema(&key) {
                consumer(member_schema, self)?;
            } else {
                self.decoder.skip().map_err(deser_err)?;
            }
            i += 1;
        }
        self.depth -= 1;
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema,
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
        self.depth -= 1;
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema,
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
        self.depth -= 1;
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema) -> Result<bool, SerdeError> {
        self.decoder.boolean().map_err(deser_err)
    }

    fn read_byte(&mut self, _schema: &Schema) -> Result<i8, SerdeError> {
        self.decoder.byte().map_err(deser_err)
    }

    fn read_short(&mut self, _schema: &Schema) -> Result<i16, SerdeError> {
        self.decoder.short().map_err(deser_err)
    }

    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, SerdeError> {
        self.decoder.integer().map_err(deser_err)
    }

    fn read_long(&mut self, _schema: &Schema) -> Result<i64, SerdeError> {
        self.decoder.long().map_err(deser_err)
    }

    fn read_float(&mut self, _schema: &Schema) -> Result<f32, SerdeError> {
        self.decoder.float().map_err(deser_err)
    }

    fn read_double(&mut self, _schema: &Schema) -> Result<f64, SerdeError> {
        self.decoder.double().map_err(deser_err)
    }

    fn read_big_integer(&mut self, _schema: &Schema) -> Result<BigInteger, SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "CBOR big integer not yet supported (smithy-rs#4611)".into(),
        })
    }

    fn read_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "CBOR big decimal not yet supported (smithy-rs#4611)".into(),
        })
    }

    fn read_string(&mut self, _schema: &Schema) -> Result<String, SerdeError> {
        self.decoder
            .str()
            .map(|cow| cow.into_owned())
            .map_err(deser_err)
    }

    fn read_blob(&mut self, _schema: &Schema) -> Result<Blob, SerdeError> {
        self.decoder.blob().map_err(deser_err)
    }

    fn read_timestamp(&mut self, _schema: &Schema) -> Result<DateTime, SerdeError> {
        self.decoder.timestamp().map_err(deser_err)
    }

    fn read_document(&mut self, _schema: &Schema) -> Result<Document, SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "document types are not supported by rpcv2Cbor protocol".into(),
        })
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

    fn read_string_list(&mut self, _schema: &Schema) -> Result<Vec<String>, SerdeError> {
        self.read_list_items(|dec| dec.str().map(|c| c.into_owned()).map_err(deser_err))
    }

    fn read_blob_list(&mut self, _schema: &Schema) -> Result<Vec<Blob>, SerdeError> {
        self.read_list_items(|dec| dec.blob().map_err(deser_err))
    }

    fn read_integer_list(&mut self, _schema: &Schema) -> Result<Vec<i32>, SerdeError> {
        self.read_list_items(|dec| dec.integer().map_err(deser_err))
    }

    fn read_long_list(&mut self, _schema: &Schema) -> Result<Vec<i64>, SerdeError> {
        self.read_list_items(|dec| dec.long().map_err(deser_err))
    }

    fn read_string_string_map(
        &mut self,
        _schema: &Schema,
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
        self.depth -= 1;
        Ok(out)
    }
}

fn deser_err(e: crate::decode::DeserializeError) -> SerdeError {
    SerdeError::InvalidInput {
        message: e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::codec::Codec;
    use aws_smithy_schema::prelude::*;
    use aws_smithy_schema::serde::{SerializableStruct, ShapeSerializer};
    use aws_smithy_schema::{shape_id, ShapeType};

    use crate::codec::CborCodec;

    /// Helper: serialize with CborSerializer, then deserialize with CborDeserializer.
    fn make_deser(f: impl FnOnce(&mut crate::codec::CborSerializer)) -> Vec<u8> {
        use aws_smithy_schema::codec::FinishSerializer;
        let codec = CborCodec::default();
        let mut ser = codec.create_serializer();
        f(&mut ser);
        ser.finish()
    }

    #[test]
    fn test_read_boolean() {
        let bytes = make_deser(|s| s.write_boolean(&BOOLEAN, true).unwrap());
        let mut de = CborDeserializer::new(&bytes, 128);
        assert_eq!(de.read_boolean(&BOOLEAN).unwrap(), true);
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
        let bytes = make_deser(|s| s.write_blob(&BLOB, &blob).unwrap());
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
            _member: &Schema,
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
            _member: &Schema,
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
}
