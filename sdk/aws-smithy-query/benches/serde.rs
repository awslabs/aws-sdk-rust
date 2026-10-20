/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Criterion benchmarks for awsQuery serialization and XML deserialization.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aws_smithy_query::codec::QueryShapeSerializer;
use aws_smithy_schema::codec::{Codec, FinishSerializer};
use aws_smithy_schema::serde::{
    SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer,
};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use aws_smithy_xml::codec::XmlCodec;

// --- Schemas ---

static NAME: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::String, "Name", 0);
static AGE: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::Integer, "Age", 1);
static EMAIL: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::String, "Email", 2);
static TAGS_LIST: Schema = Schema::new_member(shape_id!("test", "S"), ShapeType::List, "Tags", 3);
static TAG_KEY: Schema = Schema::new_member(shape_id!("test", "T"), ShapeType::String, "Key", 0);
static TAG_VAL: Schema = Schema::new_member(shape_id!("test", "T"), ShapeType::String, "Value", 1);
static TAG_STRUCT: Schema = Schema::new_struct(
    shape_id!("test", "T"),
    ShapeType::Structure,
    &[&TAG_KEY, &TAG_VAL],
);
static STRUCT_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "S"),
    ShapeType::Structure,
    &[&NAME, &AGE, &EMAIL, &TAGS_LIST],
);

// --- Helpers ---

struct Tag {
    key: &'static str,
    val: &'static str,
}

impl SerializableStruct for Tag {
    fn schema(&self) -> &Schema<'_> {
        &TAG_STRUCT
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&TAG_KEY, self.key)?;
        s.write_string(&TAG_VAL, self.val)
    }
}

// --- Serialization benchmarks ---

fn serialize_small(c: &mut Criterion) {
    c.bench_function("serialize_small_3_scalars", |b| {
        b.iter(|| {
            let mut ser = QueryShapeSerializer::new("GetUser", "2012-11-05");
            ser.write_string(&NAME, "John Doe").unwrap();
            ser.write_integer(&AGE, 42).unwrap();
            ser.write_string(&EMAIL, "john@example.com").unwrap();
            black_box(ser.finish());
        });
    });
}

fn serialize_medium(c: &mut Criterion) {
    c.bench_function("serialize_medium_3_scalars_10_structs", |b| {
        b.iter(|| {
            let mut ser = QueryShapeSerializer::new("CreateResource", "2012-11-05");
            ser.write_string(&NAME, "my-resource-with-a-longer-name")
                .unwrap();
            ser.write_integer(&AGE, 12345).unwrap();
            ser.write_string(&EMAIL, "user@example.com").unwrap();
            ser.write_list(&TAGS_LIST, &|s| {
                for _ in 0..10 {
                    s.write_struct(
                        &TAG_STRUCT,
                        &Tag {
                            key: "Environment",
                            val: "Production",
                        },
                    )?;
                }
                Ok(())
            })
            .unwrap();
            black_box(ser.finish());
        });
    });
}

fn serialize_large(c: &mut Criterion) {
    c.bench_function("serialize_large_3_scalars_50_structs", |b| {
        b.iter(|| {
            let mut ser = QueryShapeSerializer::new("BatchOperation", "2012-11-05");
            ser.write_string(&NAME, "batch-resource-name-that-is-reasonably-long")
                .unwrap();
            ser.write_integer(&AGE, 999999).unwrap();
            ser.write_string(&EMAIL, "batch-operator@company-domain.example.com")
                .unwrap();
            ser.write_list(&TAGS_LIST, &|s| {
                for i in 0..50 {
                    let _ = i;
                    s.write_struct(
                        &TAG_STRUCT,
                        &Tag {
                            key: "ResourceTag",
                            val: "some-value-that-represents-real-data",
                        },
                    )?;
                }
                Ok(())
            })
            .unwrap();
            black_box(ser.finish());
        });
    });
}

// --- Deserialization benchmarks ---

const SMALL_XML: &[u8] =
    b"<S><Name>John Doe</Name><Age>42</Age><Email>john@example.com</Email></S>";

const MEDIUM_XML: &[u8] = b"<S><Name>my-resource</Name><Age>12345</Age><Email>user@example.com</Email><Tags><member><Key>Env</Key><Value>Prod</Value></member><member><Key>Team</Key><Value>Backend</Value></member><member><Key>Cost</Key><Value>High</Value></member><member><Key>Region</Key><Value>us-east-1</Value></member><member><Key>Owner</Key><Value>alice</Value></member><member><Key>Stack</Key><Value>main</Value></member><member><Key>Version</Key><Value>v2.1</Value></member><member><Key>Tier</Key><Value>standard</Value></member><member><Key>Project</Key><Value>alpha</Value></member><member><Key>Dept</Key><Value>eng</Value></member></Tags></S>";

fn deserialize_small(c: &mut Criterion) {
    c.bench_function("deserialize_small_3_scalars", |b| {
        b.iter(|| {
            let codec = XmlCodec::default();
            let mut deser = codec.create_deserializer(black_box(SMALL_XML));
            deser
                .read_struct(&STRUCT_SCHEMA, &mut |member, d| {
                    match member.member_index().unwrap() {
                        0 => {
                            d.read_string(member)?;
                        }
                        1 => {
                            d.read_integer(member)?;
                        }
                        2 => {
                            d.read_string(member)?;
                        }
                        _ => {}
                    }
                    Ok(())
                })
                .unwrap();
        });
    });
}

fn deserialize_medium(c: &mut Criterion) {
    c.bench_function("deserialize_medium_3_scalars_10_structs", |b| {
        b.iter(|| {
            let codec = XmlCodec::default();
            let mut deser = codec.create_deserializer(black_box(MEDIUM_XML));
            deser
                .read_struct(&STRUCT_SCHEMA, &mut |member, d| {
                    match member.member_index().unwrap() {
                        0 | 2 => {
                            d.read_string(member)?;
                        }
                        1 => {
                            d.read_integer(member)?;
                        }
                        3 => {
                            d.read_list(&TAGS_LIST, &mut |d2| {
                                d2.read_struct(&TAG_STRUCT, &mut |m, d3| {
                                    d3.read_string(m)?;
                                    Ok(())
                                })?;
                                Ok(())
                            })?;
                        }
                        _ => {}
                    }
                    Ok(())
                })
                .unwrap();
        });
    });
}

criterion_group!(
    benches,
    serialize_small,
    serialize_medium,
    serialize_large,
    deserialize_small,
    deserialize_medium,
);
criterion_main!(benches);
