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
/// Tracks the kind of CBOR container currently being written.
///
/// In RPC v2 CBOR a structure is a map keyed by member name, but list elements
/// are positional and map values follow a key that is written separately. A
/// member schema carries a `member_name` (e.g. a list's `member` or a map's
/// `value` member), so without tracking the surrounding container that name
/// would be wrongly emitted as a key when the member appears as a list element
/// or map value. Member-name keys are therefore only emitted in [`Container::Struct`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Container {
    Struct,
    List,
    Map,
}

pub struct CborSerializer {
    encoder: crate::Encoder,
    container_stack: Vec<Container>,
}

impl CborSerializer {
    pub(crate) fn new() -> Self {
        Self {
            encoder: crate::Encoder::new(Vec::new()),
            container_stack: Vec::new(),
        }
    }

    /// Writes the member name as a CBOR text string key, but only for structure
    /// members. List elements and map values are not keyed by member name even
    /// though their schemas carry one (see [`Container`]).
    #[inline]
    fn write_member_key(&mut self, schema: &Schema<'_>) {
        if self.container_stack.last() == Some(&Container::Struct) {
            if let Some(name) = schema.member_name() {
                self.encoder.str(name);
            }
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
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_map();
        self.container_stack.push(Container::Struct);
        value.serialize_members(self)?;
        self.container_stack.pop();
        self.encoder.end();
        Ok(())
    }

    fn write_list(
        &mut self,
        schema: &Schema<'_>,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_array();
        self.container_stack.push(Container::List);
        write_elements(self)?;
        self.container_stack.pop();
        self.encoder.end();
        Ok(())
    }

    fn write_map(
        &mut self,
        schema: &Schema<'_>,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.begin_map();
        self.container_stack.push(Container::Map);
        write_entries(self)?;
        self.container_stack.pop();
        self.encoder.end();
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.boolean(value);
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.byte(value);
        Ok(())
    }

    fn write_short(&mut self, schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.short(value);
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.integer(value);
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.long(value);
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.float(value);
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.double(value);
        Ok(())
    }

    fn write_big_integer(
        &mut self,
        _schema: &Schema<'_>,
        _value: &BigInteger,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "CBOR big integer not yet supported (smithy-rs#4611)",
        ))
    }

    fn write_big_decimal(
        &mut self,
        _schema: &Schema<'_>,
        _value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "CBOR big decimal not yet supported (smithy-rs#4611)",
        ))
    }

    fn write_string(&mut self, schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.str(value);
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema<'_>, value: Blob) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.blob_bytes(value.as_ref());
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema<'_>, value: &DateTime) -> Result<(), SerdeError> {
        self.write_member_key(schema);
        self.encoder.timestamp(value);
        Ok(())
    }

    fn write_document(
        &mut self,
        _schema: &Schema<'_>,
        _value: &Document,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "document types are not supported by rpcv2Cbor protocol",
        ))
    }

    fn write_null(&mut self, schema: &Schema<'_>) -> Result<(), SerdeError> {
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
    use aws_smithy_types::Blob;

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
        assert!(dec.boolean().unwrap());
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
        let bytes = round_trip(|s| s.write_blob(&BLOB, blob.clone()).unwrap());
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

    // --- Tagged-union request serialization over CBOR.
    //
    // A tagged union serializes as a single-entry CBOR map keyed by the active
    // variant name (`{variantName: value}`), nested inside the surrounding
    // indefinite-length map of the request. This mirrors a `CreateConnection`-
    // style request whose input carries tagged-union members. The union's
    // `serialize_members` writes exactly the active variant (mirroring the
    // generated `renderSerializableUnion`). Verified at the wire-byte level
    // (exact bytes via the Encoder) and by full serialize -> deserialize
    // round-trip, including a scalar variant, a struct variant, two union
    // members in one input, and indefinite-length map acceptance.
    mod tagged_union_serialization {
        use crate::codec::{CborCodec, CborDeserializer, CborSerializer};
        use aws_smithy_schema::codec::{Codec, FinishSerializer};
        use aws_smithy_schema::serde::{
            SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer,
        };
        use aws_smithy_schema::{shape_id, Schema, ShapeType};

        fn serialize(f: impl FnOnce(&mut CborSerializer)) -> Vec<u8> {
            let codec = CborCodec::default();
            let mut ser = codec.create_serializer();
            f(&mut ser);
            ser.finish()
        }

        // Union `Endpoint { arn: String, config: EndpointConfig }`.
        static EP_ARN: Schema =
            Schema::new_member(shape_id!("test", "Endpoint"), ShapeType::String, "arn", 0);
        static EP_CONFIG: Schema = Schema::new_member(
            shape_id!("test", "Endpoint"),
            ShapeType::Structure,
            "config",
            1,
        );
        static ENDPOINT_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "Endpoint"),
            ShapeType::Union,
            &[&EP_ARN, &EP_CONFIG],
        );

        // Struct `EndpointConfig { name: String }`.
        static EC_NAME: Schema = Schema::new_member(
            shape_id!("test", "EndpointConfig"),
            ShapeType::String,
            "name",
            0,
        );
        static ENDPOINT_CONFIG_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "EndpointConfig"),
            ShapeType::Structure,
            &[&EC_NAME],
        );

        // Input `CreateConnectionInput { attachPoint: Endpoint, remoteAccount: Endpoint }`.
        static CC_ATTACH: Schema = Schema::new_member(
            shape_id!("test", "CreateConnectionInput"),
            ShapeType::Union,
            "attachPoint",
            0,
        );
        static CC_REMOTE: Schema = Schema::new_member(
            shape_id!("test", "CreateConnectionInput"),
            ShapeType::Union,
            "remoteAccount",
            1,
        );
        static CREATE_CONNECTION_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "CreateConnectionInput"),
            ShapeType::Structure,
            &[&CC_ATTACH, &CC_REMOTE],
        );

        #[derive(Debug, Clone, PartialEq)]
        struct EndpointConfig {
            name: String,
        }
        impl SerializableStruct for EndpointConfig {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&EC_NAME, &self.name)
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        enum Endpoint {
            Arn(String),
            Config(EndpointConfig),
        }
        impl SerializableStruct for Endpoint {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                // Exactly the active variant is written (mirrors renderSerializableUnion).
                match self {
                    Endpoint::Arn(v) => s.write_string(&EP_ARN, v),
                    Endpoint::Config(v) => s.write_struct(&EP_CONFIG, v),
                }
            }
        }

        #[derive(Debug, Clone, PartialEq)]
        struct CreateConnectionInput {
            attach_point: Endpoint,
            remote_account: Endpoint,
        }
        impl SerializableStruct for CreateConnectionInput {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_struct(&CC_ATTACH, &self.attach_point)?;
                s.write_struct(&CC_REMOTE, &self.remote_account)
            }
        }

        // deserialize mirrors renderDeserializeUnion / struct deserialize.
        fn deser_endpoint_config(
            d: &mut dyn ShapeDeserializer,
        ) -> Result<EndpointConfig, SerdeError> {
            let mut name = String::new();
            d.read_struct(&ENDPOINT_CONFIG_SCHEMA, &mut |m, de| {
                if m.member_name() == Some("name") {
                    name = de.read_string(m)?;
                }
                Ok(())
            })?;
            Ok(EndpointConfig { name })
        }
        fn deser_endpoint(d: &mut dyn ShapeDeserializer) -> Result<Endpoint, SerdeError> {
            let mut out: Option<Endpoint> = None;
            d.read_struct(&ENDPOINT_SCHEMA, &mut |m, de| {
                out = Some(match m.member_index() {
                    Some(0) => Endpoint::Arn(de.read_string(m)?),
                    Some(1) => Endpoint::Config(deser_endpoint_config(de)?),
                    _ => return Err(SerdeError::custom("unknown variant")),
                });
                Ok(())
            })?;
            out.ok_or_else(|| SerdeError::custom("expected a union variant"))
        }
        fn deser_create_connection(
            d: &mut dyn ShapeDeserializer,
        ) -> Result<CreateConnectionInput, SerdeError> {
            let mut ap: Option<Endpoint> = None;
            let mut ra: Option<Endpoint> = None;
            d.read_struct(&CREATE_CONNECTION_SCHEMA, &mut |m, de| {
                match m.member_index() {
                    Some(0) => ap = Some(deser_endpoint(de)?),
                    Some(1) => ra = Some(deser_endpoint(de)?),
                    _ => {}
                }
                Ok(())
            })?;
            Ok(CreateConnectionInput {
                attach_point: ap.ok_or_else(|| SerdeError::custom("missing attachPoint"))?,
                remote_account: ra.ok_or_else(|| SerdeError::custom("missing remoteAccount"))?,
            })
        }

        #[test]
        fn scalar_variant_wire_bytes() {
            let bytes = serialize(|s| {
                s.write_struct(&ENDPOINT_SCHEMA, &Endpoint::Arn("dxcon-abc".to_string()))
                    .unwrap()
            });
            // A single-entry indefinite-length map keyed by the variant name.
            let mut expected = crate::Encoder::new(Vec::new());
            expected.begin_map().str("arn").str("dxcon-abc").end();
            assert_eq!(
                bytes,
                expected.into_writer(),
                "tagged union must be a single-entry map keyed by the active variant name"
            );
            assert_eq!(bytes[0], 0xBF, "must use an indefinite-length map (0xBF)");
            assert_eq!(
                *bytes.last().unwrap(),
                0xFF,
                "must be terminated by a break (0xFF)"
            );
        }

        #[test]
        fn struct_variant_wire_bytes() {
            let bytes = serialize(|s| {
                s.write_struct(
                    &ENDPOINT_SCHEMA,
                    &Endpoint::Config(EndpointConfig {
                        name: "eth0".to_string(),
                    }),
                )
                .unwrap()
            });
            let mut expected = crate::Encoder::new(Vec::new());
            expected
                .begin_map()
                .str("config")
                .begin_map()
                .str("name")
                .str("eth0")
                .end()
                .end();
            assert_eq!(bytes, expected.into_writer());
        }

        #[test]
        fn two_union_members_in_one_input_wire_bytes() {
            let input = CreateConnectionInput {
                attach_point: Endpoint::Arn("ap".to_string()),
                remote_account: Endpoint::Config(EndpointConfig {
                    name: "ra".to_string(),
                }),
            };
            let bytes = serialize(|s| s.write_struct(&CREATE_CONNECTION_SCHEMA, &input).unwrap());
            let mut expected = crate::Encoder::new(Vec::new());
            expected
                .begin_map()
                .str("attachPoint")
                .begin_map()
                .str("arn")
                .str("ap")
                .end()
                .str("remoteAccount")
                .begin_map()
                .str("config")
                .begin_map()
                .str("name")
                .str("ra")
                .end()
                .end()
                .end();
            assert_eq!(bytes, expected.into_writer());
        }

        #[test]
        fn tagged_unions_round_trip() {
            let input = CreateConnectionInput {
                attach_point: Endpoint::Arn("dxcon-1".to_string()),
                remote_account: Endpoint::Config(EndpointConfig {
                    name: "primary".to_string(),
                }),
            };
            let bytes = serialize(|s| s.write_struct(&CREATE_CONNECTION_SCHEMA, &input).unwrap());
            // 128 = codec default max_depth.
            let mut de = CborDeserializer::new(&bytes, 128);
            let decoded = deser_create_connection(&mut de).unwrap();
            assert_eq!(
                decoded, input,
                "serialize -> deserialize must round-trip the tagged unions"
            );
        }
    }
}
