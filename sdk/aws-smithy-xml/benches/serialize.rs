/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Criterion benchmarks for schema-based XML serialization.
//!
//! Schemas are `static`s written the way generated code emits them: list and
//! map members carry their member schemas, and list items, map keys and map
//! values are written with prelude schemas.

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use aws_smithy_schema::codec::{Codec, FinishSerializer};
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::{prelude, shape_id, Schema, ShapeType};
use aws_smithy_xml::codec::{XmlCodec, XmlCodecSettings};

macro_rules! member {
    ($static_name:ident, $shape:literal, $ty:ident, $name:literal, $index:literal) => {
        static $static_name: Schema<'static> = Schema::new_member(
            shape_id!("bench", $shape, $name),
            ShapeType::$ty,
            $name,
            $index,
        );
    };
}

// --- 6 scalar members ---

member!(P90_DOCUMENT_NAME, "P90", String, "DocumentName", 0);
member!(P90_INSTANCE_ID, "P90", String, "InstanceId", 1);
member!(P90_TIMEOUT_SECONDS, "P90", Integer, "TimeoutSeconds", 2);
member!(P90_COMMENT, "P90", String, "Comment", 3);
member!(P90_MAX_CONCURRENCY, "P90", String, "MaxConcurrency", 4);
member!(P90_WITH_DECRYPTION, "P90", Boolean, "WithDecryption", 5);
static P90: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "P90"),
    ShapeType::Structure,
    &[
        &P90_DOCUMENT_NAME,
        &P90_INSTANCE_ID,
        &P90_TIMEOUT_SECONDS,
        &P90_COMMENT,
        &P90_MAX_CONCURRENCY,
        &P90_WITH_DECRYPTION,
    ],
);

struct P90Value;

impl SerializableStruct for P90Value {
    fn schema(&self) -> &Schema<'_> {
        &P90
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&P90_DOCUMENT_NAME, "AWS-RunShellScript")?;
        s.write_string(&P90_INSTANCE_ID, "i-0123456789abcdef0")?;
        s.write_integer(&P90_TIMEOUT_SECONDS, 600)?;
        s.write_string(&P90_COMMENT, "nightly patch run")?;
        s.write_string(&P90_MAX_CONCURRENCY, "50")?;
        s.write_boolean(&P90_WITH_DECRYPTION, true)
    }
}

// --- A wrapped list of 20 strings and a flattened one ---

member!(KEYS_MEMBER, "KeyList", String, "member", 0);
static KEYS: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Keys", "Keys"),
    ShapeType::List,
    "Keys",
    0,
)
.with_list_member(&KEYS_MEMBER);
static FLAT_KEYS: Schema<'static> =
    Schema::new_member(shape_id!("bench", "Keys", "Key"), ShapeType::List, "Key", 0)
        .with_xml_flattened()
        .with_list_member(&KEYS_MEMBER);
static KEY_LIST: Schema<'static> =
    Schema::new_struct(shape_id!("bench", "Keys"), ShapeType::Structure, &[&KEYS]);
static FLAT_KEY_LIST: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Keys"),
    ShapeType::Structure,
    &[&FLAT_KEYS],
);

struct KeyList(&'static Schema<'static>, &'static Schema<'static>);

impl SerializableStruct for KeyList {
    fn schema(&self) -> &Schema<'_> {
        self.0
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_list(self.1, &|s| {
            for _ in 0..20 {
                s.write_string(&prelude::STRING, "photos/2026/10/07/IMG_0001.jpg")?;
            }
            Ok(())
        })
    }
}

// --- S3 `PutBucketTagging`: a list of 50 `Tag` structures ---

member!(TAG_KEY, "Tag", String, "Key", 0);
member!(TAG_VALUE, "Tag", String, "Value", 1);
static TAG: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Tag"),
    ShapeType::Structure,
    &[&TAG_KEY, &TAG_VALUE],
);

struct Tag;

impl SerializableStruct for Tag {
    fn schema(&self) -> &Schema<'_> {
        &TAG
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_string(&TAG_KEY, "Environment")?;
        s.write_string(&TAG_VALUE, "Production")
    }
}

static TAG_SET_MEMBER: Schema<'static> = Schema::new_member(
    shape_id!("bench", "TagSet", "member"),
    ShapeType::Structure,
    "member",
    0,
)
.with_xml_name("Tag");
static TAG_SET: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Tagging", "TagSet"),
    ShapeType::List,
    "TagSet",
    0,
)
.with_list_member(&TAG_SET_MEMBER);
static TAGGING: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Tagging"),
    ShapeType::Structure,
    &[&TAG_SET],
);

struct Tagging;

impl SerializableStruct for Tagging {
    fn schema(&self) -> &Schema<'_> {
        &TAGGING
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_list(&TAG_SET, &|s| {
            for _ in 0..50 {
                s.write_struct(&TAG, &Tag)?;
            }
            Ok(())
        })
    }
}

// --- A map of 10 string entries ---

member!(ATTRIBUTES_KEY, "Attributes", String, "key", 0);
member!(ATTRIBUTES_VALUE, "Attributes", String, "value", 1);
static ATTRIBUTES: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Attributed", "Attributes"),
    ShapeType::Map,
    "Attributes",
    0,
)
.with_map_members(&ATTRIBUTES_KEY, &ATTRIBUTES_VALUE);
static ATTRIBUTED: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Attributed"),
    ShapeType::Structure,
    &[&ATTRIBUTES],
);

struct Attributed;

impl SerializableStruct for Attributed {
    fn schema(&self) -> &Schema<'_> {
        &ATTRIBUTED
    }

    fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        s.write_map(&ATTRIBUTES, &|s| {
            for (key, value) in [
                ("DelaySeconds", "0"),
                ("MaximumMessageSize", "262144"),
                ("MessageRetentionPeriod", "345600"),
                ("ReceiveMessageWaitTimeSeconds", "20"),
                ("VisibilityTimeout", "30"),
                ("KmsMasterKeyId", "alias/aws/sqs"),
                ("KmsDataKeyReusePeriodSeconds", "300"),
                ("FifoQueue", "false"),
                ("ContentBasedDeduplication", "false"),
                ("SqsManagedSseEnabled", "true"),
            ] {
                s.write_string(&prelude::STRING, key)?;
                s.write_string(&prelude::STRING, value)?;
            }
            Ok(())
        })
    }
}

// --- Benchmarks ---

fn bench_case(
    c: &mut Criterion,
    name: &str,
    codec: &XmlCodec,
    schema: &'static Schema<'static>,
    value: &dyn SerializableStruct,
) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let mut ser = codec.create_serializer();
            ser.write_struct(black_box(schema), black_box(value))
                .unwrap();
            black_box(ser.finish())
        })
    });
}

fn serialize(c: &mut Criterion) {
    let codec = XmlCodec::new(XmlCodecSettings::default());
    bench_case(c, "serialize/restXml/struct_6", &codec, &P90, &P90Value);
    bench_case(
        c,
        "serialize/restXml/list_20_strings",
        &codec,
        &KEY_LIST,
        &KeyList(&KEY_LIST, &KEYS),
    );
    bench_case(
        c,
        "serialize/restXml/flattened_list_20_strings",
        &codec,
        &FLAT_KEY_LIST,
        &KeyList(&FLAT_KEY_LIST, &FLAT_KEYS),
    );
    bench_case(
        c,
        "serialize/restXml/list_50_tags",
        &codec,
        &TAGGING,
        &Tagging,
    );
    bench_case(
        c,
        "serialize/restXml/map_10",
        &codec,
        &ATTRIBUTED,
        &Attributed,
    );
}

criterion_group!(benches, serialize);
criterion_main!(benches);
