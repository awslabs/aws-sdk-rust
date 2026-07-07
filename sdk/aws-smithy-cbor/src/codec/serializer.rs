/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! CBOR serializer implementation.

use aws_smithy_schema::codec::FinishSerializer;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

/// CBOR serializer that implements the ShapeSerializer trait.
///
/// Wraps the existing optimized `Encoder` which uses `minicbor` with
/// infallible writes to `Vec<u8>`.
pub struct CborSerializer {
    encoder: crate::Encoder,
}

impl CborSerializer {
    pub(crate) fn new() -> Self {
        Self {
            encoder: crate::Encoder::new(Vec::new()),
        }
    }

    /// Writes the member name as a CBOR text string key if this schema is a struct member.
    #[inline]
    fn write_member_key(&mut self, schema: &Schema) {
        if let Some(name) = schema.member_name() {
            self.encoder.str(name);
        }
    }
}

impl FinishSerializer for CborSerializer {
    fn finish(self) -> Vec<u8> {
        self.encoder.into_writer()
    }
}

impl ShapeSerializer for CborSerializer {
    fn write_struct(
        &mut self,
        schema: &Schema,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_map();
        value.serialize_members(self)?;
        self.encoder.end();
        Ok(())
    }

    fn write_list(
        &mut self,
        schema: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_array();
        write_elements(self)?;
        self.encoder.end();
        Ok(())
    }

    fn write_map(
        &mut self,
        schema: &Schema,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_map();
        write_entries(self)?;
        self.encoder.end();
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.boolean(value);
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.byte(value);
        Ok(())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.short(value);
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.integer(value);
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.long(value);
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.float(value);
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.double(value);
        Ok(())
    }

    fn write_big_integer(
        &mut self,
        _schema: &Schema,
        _value: &BigInteger,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "CBOR big integer not yet supported (smithy-rs#4611)".into(),
        })
    }

    fn write_big_decimal(
        &mut self,
        _schema: &Schema,
        _value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "CBOR big decimal not yet supported (smithy-rs#4611)".into(),
        })
    }

    fn write_string(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.str(value);
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: &Blob) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.blob(value);
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &DateTime) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.timestamp(value);
        Ok(())
    }

    fn write_document(&mut self, _schema: &Schema, _value: &Document) -> Result<(), SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "document types are not supported by rpcv2Cbor protocol".into(),
        })
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.null();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::codec::{Codec, FinishSerializer};
    use aws_smithy_schema::prelude::*;
    use aws_smithy_schema::serde::ShapeSerializer;
    use aws_smithy_schema::{shape_id, ShapeType};

    use crate::codec::CborCodec;

    fn round_trip(f: impl FnOnce(&mut CborSerializer)) -> Vec<u8> {
        let codec = CborCodec::default();
        let mut ser = codec.create_serializer();
        f(&mut ser);
        ser.finish()
    }

    #[test]
    fn test_write_boolean() {
        let bytes = round_trip(|s| s.write_boolean(&BOOLEAN, true).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.boolean().unwrap(), true);
    }

    #[test]
    fn test_write_integer() {
        let bytes = round_trip(|s| s.write_integer(&INTEGER, 42).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.integer().unwrap(), 42);
    }

    #[test]
    fn test_write_long() {
        let bytes = round_trip(|s| s.write_long(&LONG, i64::MAX).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.long().unwrap(), i64::MAX);
    }

    #[test]
    fn test_write_float_nan() {
        let bytes = round_trip(|s| s.write_float(&FLOAT, f32::NAN).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert!(dec.float().unwrap().is_nan());
    }

    #[test]
    fn test_write_double_infinity() {
        let bytes = round_trip(|s| s.write_double(&DOUBLE, f64::INFINITY).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.double().unwrap(), f64::INFINITY);
    }

    #[test]
    fn test_write_string() {
        let bytes = round_trip(|s| s.write_string(&STRING, "hello").unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.str().unwrap().as_ref(), "hello");
    }

    #[test]
    fn test_write_blob() {
        let blob = Blob::new(b"binary data");
        let bytes = round_trip(|s| s.write_blob(&BLOB, &blob).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        assert_eq!(dec.blob().unwrap(), blob);
    }

    #[test]
    fn test_write_timestamp() {
        let ts = DateTime::from_secs_f64(1700000000.5);
        let bytes = round_trip(|s| s.write_timestamp(&TIMESTAMP, &ts).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        let decoded = dec.timestamp().unwrap();
        // Timestamp truncates to millisecond precision
        assert_eq!(decoded.as_secs_f64(), 1700000000.5);
    }

    #[test]
    fn test_write_null() {
        let bytes = round_trip(|s| s.write_null(&STRING).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        dec.null().unwrap();
    }

    #[test]
    fn test_write_list() {
        let list_schema = Schema::new(shape_id!("test", "List"), ShapeType::List);
        let bytes = round_trip(|s| {
            s.write_list(&list_schema, &|s| {
                s.write_integer(&INTEGER, 1)?;
                s.write_integer(&INTEGER, 2)?;
                s.write_integer(&INTEGER, 3)?;
                Ok(())
            })
            .unwrap()
        });
        let mut dec = crate::Decoder::new(&bytes);
        // Indefinite-length array
        let len = dec.list().unwrap();
        assert!(len.is_none()); // indefinite
        assert_eq!(dec.integer().unwrap(), 1);
        assert_eq!(dec.integer().unwrap(), 2);
        assert_eq!(dec.integer().unwrap(), 3);
    }

    #[test]
    fn test_write_struct() {
        static NAME_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "Struct"), ShapeType::String, "name", 0);
        static AGE_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "Struct"), ShapeType::Integer, "age", 1);
        static STRUCT_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "Struct"),
            ShapeType::Structure,
            &[&NAME_MEMBER, &AGE_MEMBER],
        );

        struct TestStruct;
        impl SerializableStruct for TestStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME_MEMBER, "Alice")?;
                s.write_integer(&AGE_MEMBER, 30)?;
                Ok(())
            }
        }

        let bytes = round_trip(|s| s.write_struct(&STRUCT_SCHEMA, &TestStruct).unwrap());
        let mut dec = crate::Decoder::new(&bytes);
        let len = dec.map().unwrap();
        assert!(len.is_none()); // indefinite-length map
        assert_eq!(dec.str().unwrap().as_ref(), "name");
        assert_eq!(dec.str().unwrap().as_ref(), "Alice");
        assert_eq!(dec.str().unwrap().as_ref(), "age");
        assert_eq!(dec.integer().unwrap(), 30);
    }

    #[test]
    fn test_write_map() {
        let map_schema = Schema::new(shape_id!("test", "Map"), ShapeType::Map);
        let bytes = round_trip(|s| {
            s.write_map(&map_schema, &|s| {
                s.write_string(&STRING, "key1")?;
                s.write_string(&STRING, "val1")?;
                Ok(())
            })
            .unwrap()
        });
        let mut dec = crate::Decoder::new(&bytes);
        let len = dec.map().unwrap();
        assert!(len.is_none());
        assert_eq!(dec.str().unwrap().as_ref(), "key1");
        assert_eq!(dec.str().unwrap().as_ref(), "val1");
    }
}
