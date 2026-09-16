/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Serializer implementation that converts any `Serialize` type into a [`Document`].

use super::doc_error::DocError;
use super::Document;
use crate::Number;
use serde::ser::{self, Impossible, Serialize};
use std::collections::HashMap;
use std::fmt::Display;

/// Convert any `T: Serialize` into a [`Document`].
///
/// # Example
///
/// ```
/// # #[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
/// # {
/// use serde::Serialize;
/// use aws_smithy_types::Document;
/// use aws_smithy_types::document::to_document;
///
/// #[derive(Serialize)]
/// struct MyStruct {
///     name: String,
///     age: u32,
/// }
///
/// let my_struct = MyStruct { name: "Alice".into(), age: 30 };
/// let doc = to_document(&my_struct).unwrap();
/// # }
/// ```
pub fn to_document<T>(value: &T) -> Result<Document, DocError>
where
    T: ?Sized + Serialize,
{
    value.serialize(DocSerializer)
}

/// A serializer whose output is a `Document`.
struct DocSerializer;

impl ser::Serializer for DocSerializer {
    type Ok = Document;
    type Error = DocError;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Document, DocError> {
        Ok(Document::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Document, DocError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Document, DocError> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Document, DocError> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Document, DocError> {
        if value >= 0 {
            Ok(Document::Number(Number::PosInt(value as u64)))
        } else {
            Ok(Document::Number(Number::NegInt(value)))
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Document, DocError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Document, DocError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Document, DocError> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Document, DocError> {
        Ok(Document::Number(Number::PosInt(value)))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Document, DocError> {
        self.serialize_f64(value as f64)
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<Document, DocError> {
        Ok(Document::Number(Number::Float(value)))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Document, DocError> {
        let mut s = String::new();
        s.push(value);
        Ok(Document::String(s))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Document, DocError> {
        Ok(Document::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Document, DocError> {
        let vec = value
            .iter()
            .map(|&b| Document::Number(Number::PosInt(b as u64)))
            .collect();
        Ok(Document::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<Document, DocError> {
        Ok(Document::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Document, DocError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Document, DocError> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Document, DocError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Document, DocError>
    where
        T: ?Sized + Serialize,
    {
        let mut map = HashMap::new();
        map.insert(String::from(variant), to_document(value)?);
        Ok(Document::Object(map))
    }

    #[inline]
    fn serialize_none(self) -> Result<Document, DocError> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Document, DocError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, DocError> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, DocError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, DocError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, DocError> {
        Ok(SerializeTupleVariant {
            name: String::from(variant),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, DocError> {
        Ok(SerializeMap {
            map: HashMap::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, DocError> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, DocError> {
        Ok(SerializeStructVariant {
            name: String::from(variant),
            map: HashMap::new(),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Document, DocError>
    where
        T: ?Sized + Display,
    {
        Ok(Document::String(value.to_string()))
    }
}

struct SerializeVec {
    vec: Vec<Document>,
}

struct SerializeTupleVariant {
    name: String,
    vec: Vec<Document>,
}

struct SerializeMap {
    map: HashMap<String, Document>,
    next_key: Option<String>,
}

struct SerializeStructVariant {
    name: String,
    map: HashMap<String, Document>,
}

impl ser::SerializeSeq for SerializeVec {
    type Ok = Document;
    type Error = DocError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_document(value)?);
        Ok(())
    }

    fn end(self) -> Result<Document, DocError> {
        Ok(Document::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeVec {
    type Ok = Document;
    type Error = DocError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Document, DocError> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleStruct for SerializeVec {
    type Ok = Document;
    type Error = DocError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Document, DocError> {
        ser::SerializeSeq::end(self)
    }
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Document;
    type Error = DocError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(to_document(value)?);
        Ok(())
    }

    fn end(self) -> Result<Document, DocError> {
        let mut object = HashMap::new();
        object.insert(self.name, Document::Array(self.vec));
        Ok(Document::Object(object))
    }
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Document;
    type Error = DocError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .expect("serialize_value called before serialize_key");
        self.map.insert(key, to_document(value)?);
        Ok(())
    }

    fn end(self) -> Result<Document, DocError> {
        Ok(Document::Object(self.map))
    }
}

impl ser::SerializeStruct for SerializeMap {
    type Ok = Document;
    type Error = DocError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Document, DocError> {
        ser::SerializeMap::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Document;
    type Error = DocError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), DocError>
    where
        T: ?Sized + Serialize,
    {
        self.map.insert(String::from(key), to_document(value)?);
        Ok(())
    }

    fn end(self) -> Result<Document, DocError> {
        let mut object = HashMap::new();
        object.insert(self.name, Document::Object(self.map));
        Ok(Document::Object(object))
    }
}

/// Serializer for map keys — only string-like keys are supported.
struct MapKeySerializer;

fn key_must_be_a_string() -> DocError {
    DocError::key_must_be_a_string()
}

impl ser::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = DocError;

    type SerializeSeq = Impossible<String, DocError>;
    type SerializeTuple = Impossible<String, DocError>;
    type SerializeTupleStruct = Impossible<String, DocError>;
    type SerializeTupleVariant = Impossible<String, DocError>;
    type SerializeMap = Impossible<String, DocError>;
    type SerializeStruct = Impossible<String, DocError>;
    type SerializeStructVariant = Impossible<String, DocError>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String, DocError> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String, DocError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, value: bool) -> Result<String, DocError> {
        Ok(if value { "true" } else { "false" }.to_owned())
    }

    fn serialize_i8(self, value: i8) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i16(self, value: i16) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i32(self, value: i32) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_i64(self, value: i64) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u8(self, value: u8) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u16(self, value: u16) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u32(self, value: u32) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_u64(self, value: u64) -> Result<String, DocError> {
        Ok(itoa::Buffer::new().format(value).to_owned())
    }

    fn serialize_f32(self, value: f32) -> Result<String, DocError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(DocError::float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String, DocError> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(DocError::float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String, DocError> {
        let mut s = String::new();
        s.push(value);
        Ok(s)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String, DocError> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String, DocError>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String, DocError>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, DocError> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, DocError> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String, DocError>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}
