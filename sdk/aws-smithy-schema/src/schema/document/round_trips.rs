/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Round-trip tests through [`Document`] for every Smithy data-model type.
//!
//! Each test follows the same pattern:
//!
//! 1. Construct a typed value.
//! 2. Serialize it to a [`Document`] via [`Document::from_struct`].
//! 3. Deserialize the [`Document`] back to the same typed value via
//!    [`Document::as_shape`].
//! 4. Assert structural equality between the original and the restored
//!    value.
//!
//! Coverage:
//! - All simple types: byte, short, integer, long, float, double,
//!   bigInteger, bigDecimal, boolean, string, blob, timestamp.
//! - Aggregates: dense list, sparse list (with intermixed nulls),
//!   nested list, typed map, list of structures, map of structures.
//! - Recursive structures (a tree shape with self-referential members).
//! - Discriminator preservation across the round-trip.
//!
//! The unit tests scattered through [`super::serializer`] and
//! [`super::deserializer`] cover individual write / read paths and their
//! error cases. This module exercises the *combination* — the same
//! lossless contract every codec is supposed to honour.

use std::collections::HashMap;
use std::str::FromStr;

use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, DiscriminatedDocument};

use super::{DiscriminatedDocumentExt, DocumentShapeDeserializer, DocumentShapeSerializer};
use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
use crate::{prelude, shape_id, Schema, ShapeId, ShapeType};

// -- AllTypes: every Smithy simple type + a typed list and map --------

const ALL_TYPES_ID: ShapeId<'static> = shape_id!("smithy.example", "AllTypes");

// Every member is optional so the same struct can drive both
// "everything populated" and "everything absent" tests.
static M_BYTE: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_byte"),
    ShapeType::Byte,
    "a_byte",
    0,
);
static M_SHORT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_short"),
    ShapeType::Short,
    "a_short",
    1,
);
static M_INTEGER: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "an_integer"),
    ShapeType::Integer,
    "an_integer",
    2,
);
static M_LONG: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_long"),
    ShapeType::Long,
    "a_long",
    3,
);
static M_FLOAT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_float"),
    ShapeType::Float,
    "a_float",
    4,
);
static M_DOUBLE: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_double"),
    ShapeType::Double,
    "a_double",
    5,
);
static M_BIG_INTEGER: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_big_integer"),
    ShapeType::BigInteger,
    "a_big_integer",
    6,
);
static M_BIG_DECIMAL: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_big_decimal"),
    ShapeType::BigDecimal,
    "a_big_decimal",
    7,
);
static M_BOOLEAN: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_boolean"),
    ShapeType::Boolean,
    "a_boolean",
    8,
);
static M_STRING: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_string"),
    ShapeType::String,
    "a_string",
    9,
);
static M_BLOB: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_blob"),
    ShapeType::Blob,
    "a_blob",
    10,
);
static M_TIMESTAMP: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_timestamp"),
    ShapeType::Timestamp,
    "a_timestamp",
    11,
);
static M_LIST: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_list"),
    ShapeType::List,
    "a_list",
    12,
);
static M_MAP: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AllTypes", "a_map"),
    ShapeType::Map,
    "a_map",
    13,
);

static ALL_TYPES_SCHEMA: Schema<'static> = Schema::new_struct(
    ALL_TYPES_ID,
    ShapeType::Structure,
    &[
        &M_BYTE,
        &M_SHORT,
        &M_INTEGER,
        &M_LONG,
        &M_FLOAT,
        &M_DOUBLE,
        &M_BIG_INTEGER,
        &M_BIG_DECIMAL,
        &M_BOOLEAN,
        &M_STRING,
        &M_BLOB,
        &M_TIMESTAMP,
        &M_LIST,
        &M_MAP,
    ],
);

#[derive(Debug, Default, PartialEq)]
struct AllTypes {
    a_byte: Option<i8>,
    a_short: Option<i16>,
    an_integer: Option<i32>,
    a_long: Option<i64>,
    a_float: Option<f32>,
    a_double: Option<f64>,
    a_big_integer: Option<BigInteger>,
    a_big_decimal: Option<BigDecimal>,
    a_boolean: Option<bool>,
    a_string: Option<String>,
    a_blob: Option<Blob>,
    a_timestamp: Option<DateTime>,
    a_list: Option<Vec<String>>,
    a_map: Option<HashMap<String, String>>,
}

impl SerializableStruct for AllTypes {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        if let Some(v) = self.a_byte {
            ser.write_byte(&M_BYTE, v)?;
        }
        if let Some(v) = self.a_short {
            ser.write_short(&M_SHORT, v)?;
        }
        if let Some(v) = self.an_integer {
            ser.write_integer(&M_INTEGER, v)?;
        }
        if let Some(v) = self.a_long {
            ser.write_long(&M_LONG, v)?;
        }
        if let Some(v) = self.a_float {
            ser.write_float(&M_FLOAT, v)?;
        }
        if let Some(v) = self.a_double {
            ser.write_double(&M_DOUBLE, v)?;
        }
        if let Some(v) = &self.a_big_integer {
            ser.write_big_integer(&M_BIG_INTEGER, v)?;
        }
        if let Some(v) = &self.a_big_decimal {
            ser.write_big_decimal(&M_BIG_DECIMAL, v)?;
        }
        if let Some(v) = self.a_boolean {
            ser.write_boolean(&M_BOOLEAN, v)?;
        }
        if let Some(v) = &self.a_string {
            ser.write_string(&M_STRING, v)?;
        }
        if let Some(v) = &self.a_blob {
            ser.write_blob(&M_BLOB, v.clone())?;
        }
        if let Some(v) = self.a_timestamp {
            ser.write_timestamp(&M_TIMESTAMP, &v)?;
        }
        if let Some(items) = &self.a_list {
            ser.write_list(&M_LIST, &|inner| {
                for s in items {
                    inner.write_string(&prelude::STRING, s)?;
                }
                Ok(())
            })?;
        }
        if let Some(entries) = &self.a_map {
            ser.write_map(&M_MAP, &|inner| {
                for (k, v) in entries {
                    inner.write_string(&prelude::STRING, k)?;
                    inner.write_string(&prelude::STRING, v)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

fn deserialize_all_types(deser: &mut dyn ShapeDeserializer) -> Result<AllTypes, SerdeError> {
    let mut out = AllTypes::default();
    deser.read_struct(&ALL_TYPES_SCHEMA, &mut |member, sub| {
        match member.member_index() {
            Some(0) => out.a_byte = Some(sub.read_byte(member)?),
            Some(1) => out.a_short = Some(sub.read_short(member)?),
            Some(2) => out.an_integer = Some(sub.read_integer(member)?),
            Some(3) => out.a_long = Some(sub.read_long(member)?),
            Some(4) => out.a_float = Some(sub.read_float(member)?),
            Some(5) => out.a_double = Some(sub.read_double(member)?),
            Some(6) => out.a_big_integer = Some(sub.read_big_integer(member)?),
            Some(7) => out.a_big_decimal = Some(sub.read_big_decimal(member)?),
            Some(8) => out.a_boolean = Some(sub.read_boolean(member)?),
            Some(9) => out.a_string = Some(sub.read_string(member)?),
            Some(10) => out.a_blob = Some(sub.read_blob(member)?),
            Some(11) => out.a_timestamp = Some(sub.read_timestamp(member)?),
            Some(12) => {
                let mut items = Vec::new();
                sub.read_list(member, &mut |item| {
                    items.push(item.read_string(&prelude::STRING)?);
                    Ok(())
                })?;
                out.a_list = Some(items);
            }
            Some(13) => {
                let mut entries = HashMap::new();
                sub.read_map(member, &mut |key, value| {
                    entries.insert(key, value.read_string(&prelude::STRING)?);
                    Ok(())
                })?;
                out.a_map = Some(entries);
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn round_trip_every_simple_type_and_simple_aggregates() {
    let original = AllTypes {
        a_byte: Some(-12),
        a_short: Some(1234),
        an_integer: Some(-987_654),
        a_long: Some(9_876_543_210),
        a_float: Some(3.5),
        a_double: Some(2.123_456_789),
        a_big_integer: Some("123456789012345678901234567890".parse().unwrap()),
        a_big_decimal: Some("123456789012345678901234567890.5".parse().unwrap()),
        a_boolean: Some(true),
        a_string: Some("hello".into()),
        a_blob: Some(Blob::new(vec![0x01, 0x02, 0x03, 0xff])),
        a_timestamp: Some(DateTime::from_secs(1_700_000_000)),
        a_list: Some(vec!["a".into(), "b".into(), "c".into()]),
        a_map: Some(HashMap::from([
            ("k1".into(), "v1".into()),
            ("k2".into(), "v2".into()),
        ])),
    };

    let doc = DiscriminatedDocument::from_struct(&ALL_TYPES_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_all_types).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn round_trip_with_all_members_absent_yields_empty_struct() {
    let original = AllTypes::default();
    let doc = DiscriminatedDocument::from_struct(&ALL_TYPES_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_all_types).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn round_trip_preserves_struct_discriminator() {
    let original = AllTypes {
        a_string: Some("anchor".into()),
        ..AllTypes::default()
    };
    let doc = DiscriminatedDocument::from_struct(&ALL_TYPES_SCHEMA, &original).unwrap();
    assert_eq!(doc.discriminator(), Some("smithy.example#AllTypes"));
    // The discriminator does not interfere with deserialization.
    let restored = doc.as_shape(deserialize_all_types).unwrap();
    assert_eq!(restored, original);
}

// -- Sparse list of strings -----------------------------------------------

const SPARSE_LIST_HOLDER_ID: ShapeId<'static> = shape_id!("smithy.example", "SparseListHolder");
static M_SPARSE_LIST: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "SparseListHolder", "values"),
    ShapeType::List,
    "values",
    0,
);
static SPARSE_LIST_HOLDER_SCHEMA: Schema<'static> = Schema::new_struct(
    SPARSE_LIST_HOLDER_ID,
    ShapeType::Structure,
    &[&M_SPARSE_LIST],
);

#[derive(Debug, Default, PartialEq)]
struct SparseListHolder {
    values: Vec<Option<String>>,
}

impl SerializableStruct for SparseListHolder {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_list(&M_SPARSE_LIST, &|inner| {
            for v in &self.values {
                match v {
                    Some(s) => inner.write_string(&prelude::STRING, s)?,
                    None => inner.write_null(&prelude::STRING)?,
                }
            }
            Ok(())
        })
    }
}

fn deserialize_sparse_list_holder(
    deser: &mut dyn ShapeDeserializer,
) -> Result<SparseListHolder, SerdeError> {
    let mut out = SparseListHolder::default();
    deser.read_struct(&SPARSE_LIST_HOLDER_SCHEMA, &mut |member, sub| {
        if member.member_index() == Some(0) {
            sub.read_list(member, &mut |item| {
                if item.is_null() {
                    out.values.push(None);
                } else {
                    out.values.push(Some(item.read_string(&prelude::STRING)?));
                }
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn round_trip_sparse_list_preserves_null_positions() {
    let original = SparseListHolder {
        values: vec![
            Some("first".into()),
            None,
            Some("third".into()),
            None,
            None,
            Some("sixth".into()),
        ],
    };
    let doc = DiscriminatedDocument::from_struct(&SPARSE_LIST_HOLDER_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_sparse_list_holder).unwrap();
    assert_eq!(restored, original);
}

// -- Nested aggregates: list of struct, map of struct ---------------------

// `struct Item { id: String, count: i32 }` used as element / value type.

const ITEM_ID: ShapeId<'static> = shape_id!("smithy.example", "Item");
static M_ITEM_ID: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Item", "id"),
    ShapeType::String,
    "id",
    0,
);
static M_ITEM_COUNT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Item", "count"),
    ShapeType::Integer,
    "count",
    1,
);
static ITEM_SCHEMA: Schema<'static> =
    Schema::new_struct(ITEM_ID, ShapeType::Structure, &[&M_ITEM_ID, &M_ITEM_COUNT]);

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone)]
struct Item {
    id: String,
    count: i32,
}

impl SerializableStruct for Item {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_string(&M_ITEM_ID, &self.id)?;
        ser.write_integer(&M_ITEM_COUNT, self.count)?;
        Ok(())
    }
}

fn deserialize_item(deser: &mut dyn ShapeDeserializer) -> Result<Item, SerdeError> {
    let mut out = Item::default();
    deser.read_struct(&ITEM_SCHEMA, &mut |member, sub| {
        match member.member_index() {
            Some(0) => out.id = sub.read_string(member)?,
            Some(1) => out.count = sub.read_integer(member)?,
            _ => {}
        }
        Ok(())
    })?;
    Ok(out)
}

const ITEM_BAG_ID: ShapeId<'static> = shape_id!("smithy.example", "ItemBag");
static M_ITEM_LIST: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "ItemBag", "items"),
    ShapeType::List,
    "items",
    0,
);
static M_ITEM_MAP: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "ItemBag", "named_items"),
    ShapeType::Map,
    "named_items",
    1,
);
static ITEM_BAG_SCHEMA: Schema<'static> = Schema::new_struct(
    ITEM_BAG_ID,
    ShapeType::Structure,
    &[&M_ITEM_LIST, &M_ITEM_MAP],
);

#[derive(Debug, Default, PartialEq)]
struct ItemBag {
    items: Vec<Item>,
    named_items: HashMap<String, Item>,
}

impl SerializableStruct for ItemBag {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_list(&M_ITEM_LIST, &|inner| {
            for item in &self.items {
                inner.write_struct(&ITEM_SCHEMA, item)?;
            }
            Ok(())
        })?;
        ser.write_map(&M_ITEM_MAP, &|inner| {
            for (k, v) in &self.named_items {
                inner.write_string(&prelude::STRING, k)?;
                inner.write_struct(&ITEM_SCHEMA, v)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

fn deserialize_item_bag(deser: &mut dyn ShapeDeserializer) -> Result<ItemBag, SerdeError> {
    let mut out = ItemBag::default();
    deser.read_struct(&ITEM_BAG_SCHEMA, &mut |member, sub| {
        match member.member_index() {
            Some(0) => {
                sub.read_list(member, &mut |item_de| {
                    out.items.push(deserialize_item(item_de)?);
                    Ok(())
                })?;
            }
            Some(1) => {
                sub.read_map(member, &mut |key, value_de| {
                    out.named_items.insert(key, deserialize_item(value_de)?);
                    Ok(())
                })?;
            }
            _ => {}
        }
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn round_trip_list_of_structs_and_map_of_structs() {
    let original = ItemBag {
        items: vec![
            Item {
                id: "alpha".into(),
                count: 1,
            },
            Item {
                id: "beta".into(),
                count: 22,
            },
            Item {
                id: "gamma".into(),
                count: 333,
            },
        ],
        named_items: HashMap::from([
            (
                "primary".to_string(),
                Item {
                    id: "p".into(),
                    count: 9,
                },
            ),
            (
                "secondary".to_string(),
                Item {
                    id: "s".into(),
                    count: 81,
                },
            ),
        ]),
    };
    let doc = DiscriminatedDocument::from_struct(&ITEM_BAG_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_item_bag).unwrap();
    assert_eq!(restored, original);
}

// -- Recursive structure --------------------------------------------------

// `struct Tree { value: i32, left: Option<Box<Tree>>, right: Option<Box<Tree>> }`

const TREE_ID: ShapeId<'static> = shape_id!("smithy.example", "Tree");
static M_TREE_VALUE: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Tree", "value"),
    ShapeType::Integer,
    "value",
    0,
);
static M_TREE_LEFT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Tree", "left"),
    ShapeType::Structure,
    "left",
    1,
);
static M_TREE_RIGHT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Tree", "right"),
    ShapeType::Structure,
    "right",
    2,
);
static TREE_SCHEMA: Schema<'static> = Schema::new_struct(
    TREE_ID,
    ShapeType::Structure,
    &[&M_TREE_VALUE, &M_TREE_LEFT, &M_TREE_RIGHT],
);

#[derive(Debug, Default, PartialEq)]
struct Tree {
    value: i32,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

impl SerializableStruct for Tree {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_integer(&M_TREE_VALUE, self.value)?;
        if let Some(left) = &self.left {
            // For nested struct members, codegen passes the *member*
            // schema (not the target's root schema) so commit_value can
            // resolve the member name from the parent struct frame.
            // See SchemaGenerator.kt's write_struct emission for member
            // contexts.
            ser.write_struct(&M_TREE_LEFT, left.as_ref())?;
        }
        if let Some(right) = &self.right {
            ser.write_struct(&M_TREE_RIGHT, right.as_ref())?;
        }
        Ok(())
    }
}

fn deserialize_tree(deser: &mut dyn ShapeDeserializer) -> Result<Tree, SerdeError> {
    let mut out = Tree::default();
    deser.read_struct(&TREE_SCHEMA, &mut |member, sub| {
        match member.member_index() {
            Some(0) => out.value = sub.read_integer(member)?,
            Some(1) => out.left = Some(Box::new(deserialize_tree(sub)?)),
            Some(2) => out.right = Some(Box::new(deserialize_tree(sub)?)),
            _ => {}
        }
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn round_trip_recursive_tree_structure() {
    // Tree:
    //         1
    //        / \
    //       2   3
    //      /     \
    //     4       5
    let original = Tree {
        value: 1,
        left: Some(Box::new(Tree {
            value: 2,
            left: Some(Box::new(Tree {
                value: 4,
                left: None,
                right: None,
            })),
            right: None,
        })),
        right: Some(Box::new(Tree {
            value: 3,
            left: None,
            right: Some(Box::new(Tree {
                value: 5,
                left: None,
                right: None,
            })),
        })),
    };
    let doc = DiscriminatedDocument::from_struct(&TREE_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_tree).unwrap();
    assert_eq!(restored, original);
}

// -- Nested list-of-list --------------------------------------------------

const MATRIX_HOLDER_ID: ShapeId<'static> = shape_id!("smithy.example", "MatrixHolder");
static M_MATRIX: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "MatrixHolder", "matrix"),
    ShapeType::List,
    "matrix",
    0,
);
static MATRIX_HOLDER_SCHEMA: Schema<'static> =
    Schema::new_struct(MATRIX_HOLDER_ID, ShapeType::Structure, &[&M_MATRIX]);

#[derive(Debug, Default, PartialEq)]
struct MatrixHolder {
    matrix: Vec<Vec<i32>>,
}

impl SerializableStruct for MatrixHolder {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_list(&M_MATRIX, &|outer| {
            for row in &self.matrix {
                outer.write_list(&prelude::DOCUMENT, &|inner| {
                    for v in row {
                        inner.write_integer(&prelude::INTEGER, *v)?;
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })
    }
}

fn deserialize_matrix_holder(
    deser: &mut dyn ShapeDeserializer,
) -> Result<MatrixHolder, SerdeError> {
    let mut out = MatrixHolder::default();
    deser.read_struct(&MATRIX_HOLDER_SCHEMA, &mut |member, sub| {
        if member.member_index() == Some(0) {
            sub.read_list(member, &mut |row_de| {
                let mut row = Vec::new();
                row_de.read_list(&prelude::DOCUMENT, &mut |item| {
                    row.push(item.read_integer(&prelude::INTEGER)?);
                    Ok(())
                })?;
                out.matrix.push(row);
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(out)
}

#[test]
fn round_trip_nested_list_of_lists() {
    let original = MatrixHolder {
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
    };
    let doc = DiscriminatedDocument::from_struct(&MATRIX_HOLDER_SCHEMA, &original).unwrap();
    let restored = doc.as_shape(deserialize_matrix_holder).unwrap();
    assert_eq!(restored, original);
}

// -- Direct (non-helper) round-trip exercising both adapters explicitly ---

#[test]
fn round_trip_via_explicit_serializer_and_deserializer() {
    // Same shape as the AllTypes test, but uses the
    // DocumentShapeSerializer / DocumentShapeDeserializer directly to
    // make sure the helper entry points (`from_struct`, `as_shape`)
    // are not the only path that works.
    let original = AllTypes {
        a_string: Some("explicit".into()),
        an_integer: Some(123),
        a_big_decimal: Some(BigDecimal::from_str("0.0001").unwrap()),
        ..AllTypes::default()
    };

    let mut ser = DocumentShapeSerializer::new();
    ser.write_struct(&ALL_TYPES_SCHEMA, &original).unwrap();
    let doc = ser.finish().unwrap();

    let mut deser = DocumentShapeDeserializer::new(doc.document());
    let restored = deserialize_all_types(&mut deser).unwrap();

    assert_eq!(restored, original);
}
