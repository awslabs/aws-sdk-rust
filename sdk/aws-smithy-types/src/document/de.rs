/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Deserializer implementation that converts a [`Document`] into any `Deserialize` type.

use super::doc_error::DocError;
use super::Document;
use crate::Number;
use serde::de::{
    self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;

/// Convert a [`Document`] into any `T: DeserializeOwned`.
///
/// # Example
///
/// ```
/// # #[cfg(all(aws_sdk_unstable, feature = "serde-serialize", feature = "serde-deserialize"))]
/// # {
/// use serde::{Serialize, Deserialize};
/// use aws_smithy_types::Document;
/// use aws_smithy_types::document::{to_document, from_document};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct MyStruct {
///     name: String,
///     age: u32,
/// }
///
/// let my_struct = MyStruct { name: "Alice".into(), age: 30 };
/// let doc = to_document(&my_struct).unwrap();
/// let roundtripped: MyStruct = from_document(doc).unwrap();
/// assert_eq!(my_struct, roundtripped);
/// # }
/// ```
pub fn from_document<T>(doc: Document) -> Result<T, DocError>
where
    T: serde::de::DeserializeOwned,
{
    T::deserialize(doc)
}

impl<'de> de::Deserializer<'de> for Document {
    type Error = DocError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self {
            Document::Null => visitor.visit_unit(),
            Document::Bool(b) => visitor.visit_bool(b),
            Document::Number(n) => match n {
                Number::PosInt(v) => visitor.visit_u64(v),
                Number::NegInt(v) => visitor.visit_i64(v),
                Number::Float(v) => visitor.visit_f64(v),
            },
            Document::String(s) => visitor.visit_string(s),
            Document::Array(arr) => {
                let mut de = SeqDeserializer {
                    iter: arr.into_iter(),
                };
                let seq = visitor.visit_seq(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(seq)
                } else {
                    Err(DocError::custom(format_args!(
                        "expected end of sequence, got {} remaining elements",
                        de.iter.len()
                    )))
                }
            }
            Document::Object(map) => {
                let mut de = MapDeserializer {
                    iter: map.into_iter(),
                    value: None,
                };
                let map_result = visitor.visit_map(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(map_result)
                } else {
                    Err(DocError::custom(format_args!(
                        "expected end of map, got {} remaining entries",
                        de.iter.len()
                    )))
                }
            }
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self {
            Document::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self {
            Document::String(s) => visitor.visit_enum(EnumDeserializer {
                variant: s,
                value: None,
            }),
            Document::Object(map) => {
                let mut iter = map.into_iter();
                if let Some((variant, value)) = iter.next() {
                    if iter.next().is_some() {
                        return Err(DocError::custom(
                            "expected a single-entry map for enum variant",
                        ));
                    }
                    visitor.visit_enum(EnumDeserializer {
                        variant,
                        value: Some(value),
                    })
                } else {
                    Err(DocError::custom(
                        "expected a non-empty map for enum variant",
                    ))
                }
            }
            _ => Err(DocError::custom(
                "expected a string or map for enum deserialization",
            )),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct seq tuple tuple_struct map struct identifier
        ignored_any
    }
}

/// Deserializer for sequence (array) values.
struct SeqDeserializer {
    iter: std::vec::IntoIter<Document>,
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = DocError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, DocError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(doc) => seed.deserialize(doc).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        let (lower, _upper) = self.iter.size_hint();
        Some(lower)
    }
}

/// Deserializer for map (object) values.
struct MapDeserializer {
    iter: std::collections::hash_map::IntoIter<String, Document>,
    value: Option<Document>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = DocError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, DocError>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(MapKeyDeserializer { key }).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, DocError>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        let (lower, _) = self.iter.size_hint();
        Some(lower)
    }
}

/// Deserializer for map keys.
///
/// Map keys in a [`Document`] are always stored as `String`, but the user's key type
/// may be any `Deserialize` type (e.g., `HashMap<u32, _>`). This deserializer parses
/// the string back into the requested primitive type when needed, mirroring how
/// `serde_json` handles non-string map keys.
struct MapKeyDeserializer {
    key: String,
}

impl<'de> de::Deserializer<'de> for MapKeyDeserializer {
    type Error = DocError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.key)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.as_str() {
            "true" => visitor.visit_bool(true),
            "false" => visitor.visit_bool(false),
            _ => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<i8>() {
            Ok(v) => visitor.visit_i8(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<i16>() {
            Ok(v) => visitor.visit_i16(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<i32>() {
            Ok(v) => visitor.visit_i32(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<i64>() {
            Ok(v) => visitor.visit_i64(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<u8>() {
            Ok(v) => visitor.visit_u8(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<u16>() {
            Ok(v) => visitor.visit_u16(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<u32>() {
            Ok(v) => visitor.visit_u32(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<u64>() {
            Ok(v) => visitor.visit_u64(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<f32>() {
            Ok(v) => visitor.visit_f32(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.key.parse::<f64>() {
            Ok(v) => visitor.visit_f64(v),
            Err(_) => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        let mut chars = self.key.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => visitor.visit_char(c),
            _ => Err(de::Error::invalid_value(
                Unexpected::Str(&self.key),
                &visitor,
            )),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        Document::String(self.key).deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        str string bytes byte_buf unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

/// Deserializer for enum values.
struct EnumDeserializer {
    variant: String,
    value: Option<Document>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = DocError;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), DocError>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(Document::String(self.variant))?;
        Ok((variant, VariantDeserializer { value: self.value }))
    }
}

/// Deserializer for enum variant values.
struct VariantDeserializer {
    value: Option<Document>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = DocError;

    fn unit_variant(self) -> Result<(), DocError> {
        match self.value {
            None => Ok(()),
            Some(Document::Null) => Ok(()),
            Some(_) => Err(DocError::custom("expected unit variant, found value")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, DocError>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(DocError::custom(
                "expected newtype variant, found unit variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Document::Array(arr)) => {
                let mut de = SeqDeserializer {
                    iter: arr.into_iter(),
                };
                visitor.visit_seq(&mut de)
            }
            Some(_) => Err(DocError::custom("expected array for tuple variant")),
            None => Err(DocError::custom(
                "expected tuple variant, found unit variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, DocError>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Document::Object(map)) => {
                let mut de = MapDeserializer {
                    iter: map.into_iter(),
                    value: None,
                };
                visitor.visit_map(&mut de)
            }
            Some(_) => Err(DocError::custom("expected object for struct variant")),
            None => Err(DocError::custom(
                "expected struct variant, found unit variant",
            )),
        }
    }
}
