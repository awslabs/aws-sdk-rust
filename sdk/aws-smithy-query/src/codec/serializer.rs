/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_schema::codec::FinishSerializer;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, DateTime, Document};
use std::fmt::Write;
use urlencoding::encode;

/// A collection path segment, kept `Copy` so it can be formatted directly into
/// the output/prefix without an intermediate `String` allocation.
/// - `Index(i)` — a list element: renders as `i`.
/// - `Entry(i, name)` — a map key/value: renders as `i.name`.
#[derive(Clone, Copy)]
enum Segment {
    Index(usize),
    Entry(usize, &'static str),
}

impl std::fmt::Display for Segment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Segment::Index(i) => write!(f, "{i}"),
            Segment::Entry(i, name) => write!(f, "{i}.{name}"),
        }
    }
}

enum CollectionContext {
    List {
        index: usize,
        /// Schema of this list's element member. When an element is itself a
        /// nested aggregate, codegen invokes `write_list`/`write_map` with a
        /// member-less placeholder (`prelude::DOCUMENT`); the child recovers its
        /// own `@xmlName`/member chain from this stashed schema. `None` when the
        /// element is a scalar.
        member_schema: Option<&'static Schema>,
    },
    Map {
        index: usize,
        expecting_key: bool,
        key_name: &'static str,
        value_name: &'static str,
        /// Schema of this map's value member — used by a nested inner aggregate
        /// value the same way as `List::member_schema`. `None` for scalar values.
        value_schema: Option<&'static Schema>,
    },
}

/// Serializes a request shape to the awsQuery `application/x-www-form-urlencoded` body.
///
/// Recurses through nested structs/lists/maps without a depth bound; this is safe
/// because the input is the client's own request shape, whose depth is fixed by the
/// model at codegen time (not attacker-controlled).
pub struct QueryShapeSerializer {
    output: String,
    prefix: String,
    prefix_lengths: Vec<usize>,
    context_stack: Vec<CollectionContext>,
}

impl QueryShapeSerializer {
    pub fn new(action: &str, version: &str) -> Self {
        let mut output = String::with_capacity(256);
        // Writing to a String is infallible.
        let _ = write!(
            output,
            "Action={}&Version={}",
            encode(action),
            encode(version)
        );
        Self {
            output,
            prefix: String::with_capacity(64),
            prefix_lengths: Vec::with_capacity(8),
            context_stack: Vec::with_capacity(4),
        }
    }

    fn wire_name<'a>(&self, schema: &'a Schema) -> &'a str {
        schema
            .xml_name()
            .map(|t| t.value())
            .or(schema.member_name())
            .unwrap_or("")
    }

    /// Resolves the wire element name for a nested collection member schema,
    /// mirroring the AWS REST XML serializer's resolution order:
    /// `@xmlName` on the member, then the member's smithy name, then `default`
    /// (e.g. `"member"`/`"key"`/`"value"`). The member-name info lives in the
    /// nested member schemas emitted by codegen's `emitAggregateMemberChain`
    /// (list member, map key, map value), so no protocol-specific schema
    /// fields are required.
    fn collection_member_name(member: Option<&Schema>, default: &'static str) -> &'static str {
        member
            .and_then(|m| m.xml_name().map(|n| n.value()))
            .or_else(|| member.and_then(|m| m.member_name()))
            .unwrap_or(default)
    }

    /// Resolves the schema to use for a nested aggregate's own element-name
    /// resolution. Codegen emits nested list/map values with a member-less
    /// placeholder (`prelude::DOCUMENT`), so when `schema` carries no member
    /// info we substitute the real inner schema the enclosing collection stashed
    /// (`List::member_schema` for a list element, `Map::value_schema` for a map
    /// value). Mirrors the XML serializer's `effective_schema`. Falls back to
    /// `schema` when there's nothing to inherit.
    fn effective_collection_schema<'a>(&self, schema: &'a Schema) -> &'a Schema {
        if schema.member().is_some() || schema.key().is_some() {
            return schema;
        }
        let inherited = match self.context_stack.last() {
            Some(CollectionContext::List { member_schema, .. }) => *member_schema,
            Some(CollectionContext::Map {
                expecting_key: false,
                value_schema,
                ..
            }) => *value_schema,
            _ => None,
        };
        inherited.unwrap_or(schema)
    }

    fn push_prefix(&mut self, segment: &str) {
        let prev_len = self.prefix.len();
        if !self.prefix.is_empty() {
            self.prefix.push('.');
        }
        self.prefix.push_str(segment);
        self.prefix_lengths.push(prev_len);
    }

    /// Like [`Self::push_prefix`] but formats a [`Segment`] directly onto the
    /// prefix, avoiding an intermediate `String`.
    fn push_prefix_segment(&mut self, segment: Segment) {
        let prev_len = self.prefix.len();
        if !self.prefix.is_empty() {
            self.prefix.push('.');
        }
        // Writing to a String is infallible.
        let _ = write!(self.prefix, "{segment}");
        self.prefix_lengths.push(prev_len);
    }

    fn pop_prefix(&mut self) {
        let prev_len = self.prefix_lengths.pop().expect("prefix stack underflow");
        self.prefix.truncate(prev_len);
    }

    /// Returns the path segment for the current collection element and advances the
    /// cursor: a 1-based index for lists, `<index>.<key|value_name>` for maps. Returns
    /// `None` when not inside a collection.
    ///
    /// Returns a `Copy` [`Segment`] rather than an owned `String` so callers can
    /// format it directly into `output`/`prefix` without a per-element allocation.
    fn next_collection_segment(&mut self) -> Option<Segment> {
        let ctx = self.context_stack.last_mut()?;
        Some(match ctx {
            CollectionContext::List { index, .. } => {
                let seg = Segment::Index(*index);
                *index += 1;
                seg
            }
            CollectionContext::Map {
                index,
                expecting_key,
                key_name,
                value_name,
                ..
            } => {
                if *expecting_key {
                    let seg = Segment::Entry(*index, key_name);
                    *expecting_key = false;
                    seg
                } else {
                    let seg = Segment::Entry(*index, value_name);
                    *expecting_key = true;
                    *index += 1;
                    seg
                }
            }
        })
    }

    /// Appends `&<param>=<value>` to the output, where `<param>` is `prefix` joined
    /// with either the collection segment (for an anonymous element) or the scalar's
    /// own wire name.
    fn write_scalar(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError> {
        let segment = if schema.member_name().is_none() {
            self.next_collection_segment()
        } else {
            None
        };
        self.output.push('&');
        // Writing to a String is infallible.
        match segment {
            Some(seg) => {
                let _ = write!(self.output, "{}.{}", self.prefix, seg);
            }
            None => {
                let name = self.wire_name(schema);
                // A named scalar written outside a collection must resolve to a
                // non-empty parameter name; an empty name would emit `&=value`
                // (or `&prefix.=value`), a malformed query param. In practice
                // codegen always supplies a member/xmlName here — this guards
                // against a future caller passing a member-less schema.
                debug_assert!(
                    !name.is_empty(),
                    "awsQuery scalar has no wire name (would emit an empty param name)"
                );
                if self.prefix.is_empty() {
                    self.output.push_str(name);
                } else {
                    let _ = write!(self.output, "{}.{}", self.prefix, name);
                }
            }
        }
        self.output.push('=');
        self.output.push_str(&encode(value));
        Ok(())
    }
}

impl FinishSerializer for QueryShapeSerializer {
    fn finish(self) -> Vec<u8> {
        self.output.into_bytes()
    }
}

impl ShapeSerializer for QueryShapeSerializer {
    fn write_struct(
        &mut self,
        schema: &Schema,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        let is_member = schema.member_name().is_some();
        let pushed_index = if is_member {
            self.push_prefix(self.wire_name(schema));
            false
        } else if let Some(seg) = self.next_collection_segment() {
            self.push_prefix_segment(seg);
            true
        } else {
            false
        };
        value.serialize_members(self)?;
        if is_member || pushed_index {
            self.pop_prefix();
        }
        Ok(())
    }

    fn write_list(
        &mut self,
        schema: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Resolve the inner schema BEFORE `next_collection_segment` mutates the
        // enclosing collection's cursor (a nested list arrives with a member-less
        // `prelude::DOCUMENT`; recover its real `@xmlName`/member chain here).
        let effective = self.effective_collection_schema(schema);
        let flat = schema.xml_flattened();
        let is_member = schema.member_name().is_some();
        let pushed_index = if is_member {
            self.push_prefix(self.wire_name(schema));
            false
        } else if let Some(seg) = self.next_collection_segment() {
            self.push_prefix_segment(seg);
            true
        } else {
            false
        };
        if !flat {
            let member_name = Self::collection_member_name(effective.member(), "member");
            self.push_prefix(member_name);
        }
        self.context_stack.push(CollectionContext::List {
            index: 1,
            member_schema: effective.member_static(),
        });
        let output_len_before = self.output.len();
        write_elements(self)?;
        let wrote_elements = self.output.len() > output_len_before;
        self.context_stack.pop();
        if !flat {
            self.pop_prefix();
        }
        if !wrote_elements && is_member {
            self.output.push('&');
            self.output.push_str(&self.prefix);
            self.output.push('=');
        }
        if is_member || pushed_index {
            self.pop_prefix();
        }
        Ok(())
    }

    fn write_map(
        &mut self,
        schema: &Schema,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Resolve the inner schema BEFORE `next_collection_segment` mutates the
        // enclosing collection's cursor (a nested map arrives with a member-less
        // `prelude::DOCUMENT`; recover its real key/value chain here).
        let effective = self.effective_collection_schema(schema);
        let flat = schema.xml_flattened();
        let is_member = schema.member_name().is_some();
        let pushed_index = if is_member {
            self.push_prefix(self.wire_name(schema));
            false
        } else if let Some(seg) = self.next_collection_segment() {
            self.push_prefix_segment(seg);
            true
        } else {
            false
        };
        if !flat {
            self.push_prefix("entry");
        }
        let key_name = Self::collection_member_name(effective.key(), "key");
        let value_name = Self::collection_member_name(effective.member(), "value");
        self.context_stack.push(CollectionContext::Map {
            index: 1,
            expecting_key: true,
            key_name,
            value_name,
            value_schema: effective.member_static(),
        });
        write_entries(self)?;
        self.context_stack.pop();
        if !flat {
            self.pop_prefix();
        }
        if is_member || pushed_index {
            self.pop_prefix();
        }
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError> {
        self.write_scalar(schema, value)
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), SerdeError> {
        self.write_scalar(schema, if value { "true" } else { "false" })
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), SerdeError> {
        self.write_scalar(schema, &value.to_string())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), SerdeError> {
        self.write_scalar(schema, &value.to_string())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), SerdeError> {
        self.write_scalar(schema, &value.to_string())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), SerdeError> {
        self.write_scalar(schema, &value.to_string())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), SerdeError> {
        let s = if value.is_nan() {
            "NaN".to_string()
        } else if value == f32::INFINITY {
            "Infinity".to_string()
        } else if value == f32::NEG_INFINITY {
            "-Infinity".to_string()
        } else {
            aws_smithy_types::primitive::Encoder::from(value)
                .encode()
                .to_owned()
        };
        self.write_scalar(schema, &s)
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), SerdeError> {
        let s = if value.is_nan() {
            "NaN".to_string()
        } else if value == f64::INFINITY {
            "Infinity".to_string()
        } else if value == f64::NEG_INFINITY {
            "-Infinity".to_string()
        } else {
            aws_smithy_types::primitive::Encoder::from(value)
                .encode()
                .to_owned()
        };
        self.write_scalar(schema, &s)
    }

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInteger) -> Result<(), SerdeError> {
        self.write_scalar(schema, value.as_ref())
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal) -> Result<(), SerdeError> {
        self.write_scalar(schema, value.as_ref())
    }

    fn write_blob(&mut self, schema: &Schema, value: &[u8]) -> Result<(), SerdeError> {
        self.write_scalar(schema, &aws_smithy_types::base64::encode(value))
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &DateTime) -> Result<(), SerdeError> {
        let format = if let Some(ts_trait) = schema.timestamp_format() {
            match ts_trait.format() {
                aws_smithy_schema::traits::TimestampFormat::EpochSeconds => {
                    aws_smithy_types::date_time::Format::EpochSeconds
                }
                aws_smithy_schema::traits::TimestampFormat::HttpDate => {
                    aws_smithy_types::date_time::Format::HttpDate
                }
                aws_smithy_schema::traits::TimestampFormat::DateTime => {
                    aws_smithy_types::date_time::Format::DateTime
                }
            }
        } else {
            aws_smithy_types::date_time::Format::DateTime
        };
        let formatted = value
            .fmt(format)
            .map_err(|e| SerdeError::custom(format!("timestamp format error: {e}")))?;
        self.write_scalar(schema, &formatted)
    }

    fn write_document(&mut self, _: &Schema, _: &Document) -> Result<(), SerdeError> {
        Err(SerdeError::UnsupportedOperation {
            message: "documents not supported in awsQuery".into(),
        })
    }

    fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
        // awsQuery has no null representation: nulls are omitted. The collection
        // index is not advanced here (only emitted elements consume an index), so a
        // `@sparse` null is dropped rather than reserving a positional slot.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::{shape_id, ShapeType};

    static NAME_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "Input"), ShapeType::String, "Name", 0);
    static AGE_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "Input"), ShapeType::Integer, "Age", 1);
    static INPUT_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "Input"),
        ShapeType::Structure,
        &[&NAME_MEMBER, &AGE_MEMBER],
    );

    struct SimpleInput;
    impl SerializableStruct for SimpleInput {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&NAME_MEMBER, "Alice")?;
            s.write_integer(&AGE_MEMBER, 30)
        }
    }

    #[test]
    fn simple_struct() {
        let mut ser = QueryShapeSerializer::new("GetUser", "2012-11-05");
        ser.write_struct(&INPUT_SCHEMA, &SimpleInput).unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(
            output,
            "Action=GetUser&Version=2012-11-05&Name=Alice&Age=30"
        );
    }

    #[test]
    fn boolean_values() {
        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Boolean, "Verbose", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_boolean(&M, true).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Verbose=true"
        );
    }

    #[test]
    fn string_encoding() {
        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Message", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_string(&M, "hello world&foo=bar").unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Message=hello%20world%26foo%3Dbar"
        );
    }

    #[test]
    fn nested_struct() {
        static INNER_FIELD: Schema =
            Schema::new_member(shape_id!("test", "Inner"), ShapeType::String, "Value", 0);
        static OUTER_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "Outer"),
            ShapeType::Structure,
            "Config",
            0,
        );

        struct Inner;
        impl SerializableStruct for Inner {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&INNER_FIELD, "hello")
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_struct(&OUTER_MEMBER, &Inner).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Config.Value=hello"
        );
    }

    #[test]
    fn list_non_flat() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "foo")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "bar")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Items.member.1=foo&Items.member.2=bar"
        );
    }

    #[test]
    fn list_flat() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0)
            .with_xml_flattened();
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "A")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "B")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Items.1=A&Items.2=B"
        );
    }

    #[test]
    fn list_of_integers() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Ids", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_integer(&aws_smithy_schema::prelude::INTEGER, 10)?;
            s.write_integer(&aws_smithy_schema::prelude::INTEGER, 20)?;
            s.write_integer(&aws_smithy_schema::prelude::INTEGER, 30)
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Ids.member.1=10&Ids.member.2=20&Ids.member.3=30"
        );
    }

    #[test]
    fn map_non_flat() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "Tags", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "color")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "red")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "size")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "large")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0\
             &Tags.entry.1.key=color&Tags.entry.1.value=red\
             &Tags.entry.2.key=size&Tags.entry.2.value=large"
        );
    }

    #[test]
    fn list_non_flat_uses_renamed_member_name() {
        // @xmlName on the list's inner member schema (attached via
        // with_list_member, exactly as codegen's emitAggregateMemberChain
        // does) drives the repeated element name — no protocol-specific
        // schema field required. Mirrors the AWS REST XML serializer.
        static ITEM: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("Item");
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0)
            .with_list_member(&ITEM);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "foo")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "bar")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Items.Item.1=foo&Items.Item.2=bar"
        );
    }

    #[test]
    fn map_non_flat_uses_renamed_key_value_names() {
        // @xmlName on the map's key/value member schemas (attached via
        // with_map_members) drives the entry key/value element names.
        static KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0)
                .with_xml_name("Attribute");
        static VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1)
                .with_xml_name("Setting");
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "Tags", 0)
            .with_map_members(&KEY, &VALUE);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "color")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "red")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Tags.entry.1.Attribute=color&Tags.entry.1.Setting=red"
        );
    }

    #[test]
    fn map_flat() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "Tags", 0)
            .with_xml_flattened();
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "k1")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "v1")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Tags.1.key=k1&Tags.1.value=v1"
        );
    }
}

#[cfg(test)]
mod cross_validation {
    use super::*;
    use crate::QueryWriter;
    use aws_smithy_schema::{shape_id, ShapeType};

    #[test]
    fn matches_query_writer_simple_params() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "SomeAction", "1.0");
        writer.prefix("Name").string("Alice");
        writer
            .prefix("Age")
            .number(aws_smithy_types::Number::PosInt(30));
        writer.finish();

        static NAME: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Name", 0);
        static AGE: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Integer, "Age", 1);
        static SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "I"), ShapeType::Structure, &[&NAME, &AGE]);
        struct Input;
        impl SerializableStruct for Input {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME, "Alice")?;
                s.write_integer(&AGE, 30)
            }
        }

        let mut ser = QueryShapeSerializer::new("SomeAction", "1.0");
        ser.write_struct(&SCHEMA, &Input).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn matches_query_writer_list() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        let mut list = writer.prefix("ListArg").start_list(false, None);
        list.entry().string("foo");
        list.entry().string("bar");
        list.entry().string("baz");
        list.finish();
        writer.finish();

        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "ListArg", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "foo")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "bar")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "baz")
        })
        .unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn matches_query_writer_flat_list() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        let mut list = writer.prefix("FlatList").start_list(true, None);
        list.entry().string("A");
        list.entry().string("B");
        list.finish();
        writer.finish();

        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "FlatList", 0)
                .with_xml_flattened();
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "A")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "B")
        })
        .unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn matches_query_writer_map() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        let mut map = writer.prefix("MapArg").start_map(false, "key", "value");
        map.entry("bar").string("Bar");
        map.entry("foo").string("Foo");
        map.finish();
        writer.finish();

        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "MapArg", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "bar")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "Bar")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "foo")?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "Foo")
        })
        .unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn matches_query_writer_nested_prefix() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("first").prefix("second").string("val");
        writer.finish();

        static SECOND: Schema =
            Schema::new_member(shape_id!("test", "Inner"), ShapeType::String, "second", 0);
        static FIRST: Schema =
            Schema::new_member(shape_id!("test", "Outer"), ShapeType::Structure, "first", 0);

        struct Inner;
        impl SerializableStruct for Inner {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&SECOND, "val")
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_struct(&FIRST, &Inner).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn nested_renamed_inner_list_member_matches_legacy() {
        // Regression: a `map<string, list<string>>` whose inner list member has
        // `@xmlName("item")`. Codegen invokes the inner `write_list` with a
        // member-less `prelude::DOCUMENT`, so the rename must be recovered from
        // the outer map's value-member schema (threaded via `value_schema`).
        // Previously this fell back to "member"; now it must honor "item" and
        // match the legacy `QueryWriter`.
        static INNER_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("item");
        // The map's value member is a list carrying the renamed inner member.
        static VALUE_LIST: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::List, "value", 1)
                .with_list_member(&INNER_MEMBER);
        static KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static OUTER_MAP: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "MapOfLists", 0)
                .with_map_members(&KEY, &VALUE_LIST);

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&OUTER_MAP, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "bar")?;
            // codegen emits `write_list(&prelude::DOCUMENT, ..)` for the inner list
            s.write_list(&aws_smithy_schema::prelude::DOCUMENT, &|inner| {
                inner.write_string(&aws_smithy_schema::prelude::STRING, "C")?;
                inner.write_string(&aws_smithy_schema::prelude::STRING, "D")
            })
        })
        .unwrap();
        let schema_out = String::from_utf8(ser.finish()).unwrap();

        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        let mut map = writer.prefix("MapOfLists").start_map(false, "key", "value");
        {
            let mut list = map.entry("bar").start_list(false, Some("item"));
            list.entry().string("C");
            list.entry().string("D");
            list.finish();
        }
        map.finish();
        writer.finish();

        assert_eq!(schema_out, expected);
        assert_eq!(
            schema_out,
            "Action=Op&Version=1.0&MapOfLists.entry.1.key=bar\
             &MapOfLists.entry.1.value.item.1=C\
             &MapOfLists.entry.1.value.item.2=D"
        );
    }

    #[test]
    fn nested_renamed_inner_map_members_matches_legacy() {
        // Same fix for a `map<string, map<string, string>>` whose INNER map has
        // `@xmlName` on its key/value members. The inner `write_map` gets
        // `prelude::DOCUMENT`; the key/value renames come from the outer map's
        // value-member schema.
        static INNER_KEY: Schema =
            Schema::new_member(shape_id!("test", "IM$key"), ShapeType::String, "key", 0)
                .with_xml_name("K");
        static INNER_VALUE: Schema =
            Schema::new_member(shape_id!("test", "IM$value"), ShapeType::String, "value", 1)
                .with_xml_name("V");
        // Outer map value member is itself a map with renamed key/value.
        static VALUE_MAP: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::Map, "value", 1)
                .with_map_members(&INNER_KEY, &INNER_VALUE);
        static OUTER_KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static OUTER_MAP: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "MapOfMaps", 0)
                .with_map_members(&OUTER_KEY, &VALUE_MAP);

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&OUTER_MAP, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "outer")?;
            s.write_map(&aws_smithy_schema::prelude::DOCUMENT, &|inner| {
                inner.write_string(&aws_smithy_schema::prelude::STRING, "ik")?;
                inner.write_string(&aws_smithy_schema::prelude::STRING, "iv")
            })
        })
        .unwrap();
        let schema_out = String::from_utf8(ser.finish()).unwrap();

        assert_eq!(
            schema_out,
            "Action=Op&Version=1.0&MapOfMaps.entry.1.key=outer\
             &MapOfMaps.entry.1.value.entry.1.K=ik\
             &MapOfMaps.entry.1.value.entry.1.V=iv"
        );
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;
    use crate::QueryWriter;
    use aws_smithy_schema::{shape_id, ShapeType};

    #[test]
    fn empty_string_value() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("Empty").string("");
        writer.finish();

        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Empty", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_string(&M, "").unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn empty_non_flat_list_emits_bare_param() {
        // An empty list serializes to just `<prefix>=`; cross-checked against the legacy writer.
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("myList").start_list(false, None).finish();
        writer.finish();

        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "myList", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|_| Ok(())).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
        assert_eq!(expected, "Action=Op&Version=1.0&myList=");
    }

    #[test]
    fn empty_flat_list_emits_bare_param() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("myList").start_list(true, None).finish();
        writer.finish();

        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "myList", 0)
            .with_xml_flattened();
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|_| Ok(())).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
        assert_eq!(expected, "Action=Op&Version=1.0&myList=");
    }

    #[test]
    fn empty_list_does_not_alias_single_empty_string_element() {
        // Guards the `wrote_elements` length-delta heuristic in `write_list`: an empty
        // list (`myList=`) must not produce the same output as a one-empty-string-element
        // list (`myList.member.1=`).
        static EMPTY: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "myList", 0);
        let mut empty_ser = QueryShapeSerializer::new("Op", "1.0");
        empty_ser.write_list(&EMPTY, &|_| Ok(())).unwrap();
        let empty_out = String::from_utf8(empty_ser.finish()).unwrap();

        static ONE: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "myList", 0);
        let mut one_ser = QueryShapeSerializer::new("Op", "1.0");
        one_ser
            .write_list(&ONE, &|s| {
                s.write_string(&aws_smithy_schema::prelude::STRING, "")
            })
            .unwrap();
        let one_out = String::from_utf8(one_ser.finish()).unwrap();

        assert_eq!(empty_out, "Action=Op&Version=1.0&myList=");
        assert_eq!(one_out, "Action=Op&Version=1.0&myList.member.1=");
        assert_ne!(empty_out, one_out);
    }

    #[test]
    fn sparse_null_list_element_is_dropped_and_index_reflects_emitted_only() {
        // A null element is dropped without consuming an index, so the next real
        // element takes `.2` rather than `.3`.
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&M, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "a")?;
            s.write_null(&aws_smithy_schema::prelude::STRING)?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "b")
        })
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Items.member.1=a&Items.member.2=b"
        );
    }

    #[test]
    fn special_characters() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("Val").string("a=b&c<d>e\"f");
        writer.finish();

        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Val", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_string(&M, "a=b&c<d>e\"f").unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn unicode_value() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("Name").string("日本語");
        writer.finish();

        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Name", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_string(&M, "日本語").unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn timestamp_epoch_seconds() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer
            .prefix("Time")
            .date_time(
                &aws_smithy_types::DateTime::from_secs(1700000000),
                aws_smithy_types::date_time::Format::DateTime,
            )
            .unwrap();
        writer.finish();

        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Timestamp, "Time", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_timestamp(&M, &aws_smithy_types::DateTime::from_secs(1700000000))
            .unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn deeply_nested_struct() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("A").prefix("B").prefix("C").string("deep");
        writer.finish();

        static C_FIELD: Schema = Schema::new_member(shape_id!("t", "C"), ShapeType::String, "C", 0);
        static B_MEMBER: Schema =
            Schema::new_member(shape_id!("t", "B"), ShapeType::Structure, "B", 0);
        static A_MEMBER: Schema =
            Schema::new_member(shape_id!("t", "A"), ShapeType::Structure, "A", 0);

        struct CStruct;
        impl SerializableStruct for CStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&C_FIELD, "deep")
            }
        }
        struct BStruct;
        impl SerializableStruct for BStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_struct(&B_MEMBER, &CStruct)
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_struct(&A_MEMBER, &BStruct).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn list_inside_struct() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        let mut outer = writer.prefix("Outer");
        let mut list = outer.prefix("Items").start_list(false, None);
        list.entry().string("x");
        list.entry().string("y");
        list.finish();
        writer.finish();

        static ITEMS: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::List, "Items", 0);
        static OUTER: Schema =
            Schema::new_member(shape_id!("t", "T"), ShapeType::Structure, "Outer", 0);

        struct Inner;
        impl SerializableStruct for Inner {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_list(&ITEMS, &|s| {
                    s.write_string(&aws_smithy_schema::prelude::STRING, "x")?;
                    s.write_string(&aws_smithy_schema::prelude::STRING, "y")
                })
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_struct(&OUTER, &Inner).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn multiple_fields_with_list_and_scalar() {
        let mut expected = String::new();
        let mut writer = QueryWriter::new(&mut expected, "Op", "1.0");
        writer.prefix("Name").string("Alice");
        let mut list = writer.prefix("Tags").start_list(false, None);
        list.entry().string("admin");
        list.entry().string("user");
        list.finish();
        writer
            .prefix("Age")
            .number(aws_smithy_types::Number::PosInt(30));
        writer.finish();

        static NAME: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::String, "Name", 0);
        static TAGS: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::List, "Tags", 1);
        static AGE: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Integer, "Age", 2);
        static SCHEMA: Schema = Schema::new_struct(
            shape_id!("t", "S"),
            ShapeType::Structure,
            &[&NAME, &TAGS, &AGE],
        );

        struct Input;
        impl SerializableStruct for Input {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME, "Alice")?;
                s.write_list(&TAGS, &|s| {
                    s.write_string(&aws_smithy_schema::prelude::STRING, "admin")?;
                    s.write_string(&aws_smithy_schema::prelude::STRING, "user")
                })?;
                s.write_integer(&AGE, 30)
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_struct(&SCHEMA, &Input).unwrap();
        assert_eq!(String::from_utf8(ser.finish()).unwrap(), expected);
    }

    #[test]
    fn list_of_struct_emits_per_element_index() {
        static NAME: Schema =
            Schema::new_member(shape_id!("test", "Item"), ShapeType::String, "Name", 0);
        static ITEM_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "Item"), ShapeType::Structure, &[&NAME]);
        static ITEMS_LIST: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0);

        struct Item(&'static str);
        impl SerializableStruct for Item {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME, self.0)
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&ITEMS_LIST, &|s| {
            s.write_struct(&ITEM_SCHEMA, &Item("X1"))?;
            s.write_struct(&ITEM_SCHEMA, &Item("X2"))
        })
        .unwrap();

        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Items.member.1.Name=X1&Items.member.2.Name=X2",
        );
    }

    #[test]
    fn list_of_list() {
        static OUTER: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Outer", 0);
        static INNER_LIST: Schema = Schema::new_list(
            shape_id!("test", "InnerList"),
            &aws_smithy_schema::prelude::STRING,
        );

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&OUTER, &|s_outer| {
            s_outer.write_list(&INNER_LIST, &|s_inner| {
                s_inner.write_string(&aws_smithy_schema::prelude::STRING, "a")?;
                s_inner.write_string(&aws_smithy_schema::prelude::STRING, "b")
            })?;
            s_outer.write_list(&INNER_LIST, &|s_inner| {
                s_inner.write_string(&aws_smithy_schema::prelude::STRING, "c")
            })
        })
        .unwrap();

        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0\
             &Outer.member.1.member.1=a\
             &Outer.member.1.member.2=b\
             &Outer.member.2.member.1=c",
        );
    }

    #[test]
    fn map_of_struct_emits_per_entry_index() {
        static NAME: Schema =
            Schema::new_member(shape_id!("test", "Val"), ShapeType::String, "Name", 0);
        static VAL_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "Val"), ShapeType::Structure, &[&NAME]);
        static MAP_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Map, "Things", 0);

        struct Val(&'static str);
        impl SerializableStruct for Val {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&NAME, self.0)
            }
        }

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_map(&MAP_MEMBER, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "k1")?;
            s.write_struct(&VAL_SCHEMA, &Val("v1"))?;
            s.write_string(&aws_smithy_schema::prelude::STRING, "k2")?;
            s.write_struct(&VAL_SCHEMA, &Val("v2"))
        })
        .unwrap();

        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0\
             &Things.entry.1.key=k1&Things.entry.1.value.Name=v1\
             &Things.entry.2.key=k2&Things.entry.2.value.Name=v2",
        );
    }

    #[test]
    fn map_nested_in_list() {
        static MAP_SCHEMA: Schema = Schema::new_map(
            shape_id!("test", "M"),
            &aws_smithy_schema::prelude::STRING,
            &aws_smithy_schema::prelude::STRING,
        );
        static OUTER: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::List, "Items", 0);

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_list(&OUTER, &|s| {
            s.write_map(&MAP_SCHEMA, &|m| {
                m.write_string(&aws_smithy_schema::prelude::STRING, "k1")?;
                m.write_string(&aws_smithy_schema::prelude::STRING, "v1")
            })?;
            s.write_map(&MAP_SCHEMA, &|m| {
                m.write_string(&aws_smithy_schema::prelude::STRING, "k2")?;
                m.write_string(&aws_smithy_schema::prelude::STRING, "v2")
            })
        })
        .unwrap();

        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0\
             &Items.member.1.entry.1.key=k1&Items.member.1.entry.1.value=v1\
             &Items.member.2.entry.1.key=k2&Items.member.2.entry.1.value=v2",
        );
    }

    #[test]
    fn float_whole_number_keeps_decimal() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Float, "Val", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_double(&M, 5.0).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=5.0"
        );
    }

    #[test]
    fn float_special_values() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Float, "Val", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_float(&M, f32::INFINITY).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=Infinity"
        );

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_float(&M, f32::NEG_INFINITY).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=-Infinity"
        );

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_float(&M, f32::NAN).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=NaN"
        );
    }

    #[test]
    fn double_special_values() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Double, "Val", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_double(&M, f64::INFINITY).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=Infinity"
        );

        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_double(&M, f64::NEG_INFINITY).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Val=-Infinity"
        );
    }

    #[test]
    fn blob_base64_encoded() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::Blob, "Data", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_blob(&M, b"hello").unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&Data=aGVsbG8%3D"
        );
    }

    #[test]
    fn document_returns_error() {
        static M: Schema =
            Schema::new_member(shape_id!("test", "I"), ShapeType::Document, "Doc", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        let result = ser.write_document(&M, &Document::Null);
        assert!(result.is_err());
    }

    #[test]
    fn write_null_is_noop() {
        static M: Schema = Schema::new_member(shape_id!("test", "I"), ShapeType::String, "Val", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_null(&M).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0"
        );
    }

    #[test]
    fn numeric_types() {
        static B: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Byte, "B", 0);
        static S: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Short, "S", 0);
        static L: Schema = Schema::new_member(shape_id!("t", "S"), ShapeType::Long, "L", 0);
        let mut ser = QueryShapeSerializer::new("Op", "1.0");
        ser.write_byte(&B, -1).unwrap();
        ser.write_short(&S, 32000).unwrap();
        ser.write_long(&L, 9999999999).unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            "Action=Op&Version=1.0&B=-1&S=32000&L=9999999999"
        );
    }
}
