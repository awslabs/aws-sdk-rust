/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Criterion benchmarks for serializing awsQuery collections.
//!
//! Every collection element's parameter name is built from names declared on the
//! collection's member schemas (`@xmlName`, member names, or the `member`/`key`/
//! `value` defaults), so these cases measure the per-collection and per-element
//! cost of resolving and carrying those names. Schemas are `static`s with nested
//! member schemas, as generated code emits them.

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use aws_smithy_query::codec::QueryShapeSerializer;
use aws_smithy_schema::codec::FinishSerializer;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::{prelude, shape_id, Schema, ShapeType};

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

// --- SQS `DeleteMessageBatch`-style list of 20 strings ---

member!(ID_MEMBER, "Ids", String, "member", 0);
static IDS: Schema<'static> =
    Schema::new_member(shape_id!("bench", "In", "Ids"), ShapeType::List, "Ids", 0)
        .with_list_member(&ID_MEMBER);

// --- SNS/IAM-style list of 50 `Tag` structures ---

member!(TAG_KEY, "Tag", String, "Key", 0);
member!(TAG_VALUE, "Tag", String, "Value", 1);
static TAG: Schema<'static> = Schema::new_struct(
    shape_id!("bench", "Tag"),
    ShapeType::Structure,
    &[&TAG_KEY, &TAG_VALUE],
);
member!(TAG_MEMBER, "Tags", Structure, "member", 0);
static TAGS: Schema<'static> =
    Schema::new_member(shape_id!("bench", "In", "Tags"), ShapeType::List, "Tags", 1)
        .with_list_member(&TAG_MEMBER);

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

// --- SNS `Attributes`-style map of 10 string entries with renamed key/value ---

static ATTR_KEY: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Attributes", "key"),
    ShapeType::String,
    "key",
    0,
)
.with_xml_name("Name");
member!(ATTR_VALUE, "Attributes", String, "value", 1);
static ATTRIBUTES: Schema<'static> = Schema::new_member(
    shape_id!("bench", "In", "Attributes"),
    ShapeType::Map,
    "Attributes",
    2,
)
.with_map_members(&ATTR_KEY, &ATTR_VALUE);

// --- A flattened list of 20 strings ---

static FLAT_IDS: Schema<'static> =
    Schema::new_member(shape_id!("bench", "In", "Id"), ShapeType::List, "Id", 3)
        .with_xml_flattened()
        .with_list_member(&ID_MEMBER);

// --- A list of 5 lists of 5 strings (nested names recovered one level down) ---

member!(INNER_MEMBER, "Inner", String, "item", 0);
static INNER: Schema<'static> = Schema::new_member(
    shape_id!("bench", "Outer", "member"),
    ShapeType::List,
    "member",
    0,
)
.with_list_member(&INNER_MEMBER);
static NESTED: Schema<'static> = Schema::new_member(
    shape_id!("bench", "In", "Nested"),
    ShapeType::List,
    "Nested",
    4,
)
.with_list_member(&INNER);

#[inline(never)]
fn serialize(write: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>) -> usize {
    let mut serializer = QueryShapeSerializer::new("Action", "2010-05-08");
    write(&mut serializer).unwrap();
    serializer.finish().len()
}

fn list_20() -> usize {
    serialize(&|s| {
        s.write_list(&IDS, &|s| {
            for _ in 0..20 {
                s.write_string(&prelude::STRING, "a1b2c3d4-e5f6-7890-abcd-ef0123456789")?;
            }
            Ok(())
        })
    })
}

fn flattened_list_20() -> usize {
    serialize(&|s| {
        s.write_list(&FLAT_IDS, &|s| {
            for _ in 0..20 {
                s.write_string(&prelude::STRING, "a1b2c3d4-e5f6-7890-abcd-ef0123456789")?;
            }
            Ok(())
        })
    })
}

fn list_50_tags() -> usize {
    serialize(&|s| {
        s.write_list(&TAGS, &|s| {
            for _ in 0..50 {
                s.write_struct(&prelude::DOCUMENT, &Tag)?;
            }
            Ok(())
        })
    })
}

fn map_10() -> usize {
    serialize(&|s| {
        s.write_map(&ATTRIBUTES, &|s| {
            for _ in 0..10 {
                s.write_string(&prelude::STRING, "DisplayName")?;
                s.write_string(&prelude::STRING, "Notifications for the build pipeline")?;
            }
            Ok(())
        })
    })
}

fn nested_list_5x5() -> usize {
    serialize(&|s| {
        s.write_list(&NESTED, &|s| {
            for _ in 0..5 {
                s.write_list(&prelude::DOCUMENT, &|s| {
                    for _ in 0..5 {
                        s.write_string(&prelude::STRING, "value")?;
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
    })
}

/// A benchmarked case: its name and the function that serializes it.
type Case = (&'static str, fn() -> usize);

/// Every benchmarked case.
pub fn cases() -> Vec<Case> {
    vec![
        ("list_20", list_20),
        ("flattened_list_20", flattened_list_20),
        ("list_50_tags", list_50_tags),
        ("map_10", map_10),
        ("nested_list_5x5", nested_list_5x5),
    ]
}

fn bench(c: &mut Criterion) {
    for (name, case) in cases() {
        c.bench_function(&format!("serialize/{name}"), |b| {
            b.iter(|| black_box(case()))
        });
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
