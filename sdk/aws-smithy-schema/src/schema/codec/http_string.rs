/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! String codec for HTTP bindings (headers, query params, URI labels).
//!
//! A value written outside a list is emitted verbatim, and a value read outside a list is the
//! whole input, as for a scalar HTTP header. List elements follow the header-list rules the
//! generated (non-schema) code has always used: they are joined with `", "`, quoted and escaped
//! with [`quote_header_value`] when needed, and read back with the RFC 7230 tokenizer from
//! [`header_parse`], which trims surrounding whitespace and unescapes quoted strings. An empty
//! element is written as `""` so that it survives the round trip.

use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
use crate::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime};

use aws_smithy_runtime_api::http::header_parse::{self, quote_header_value};
use aws_smithy_types::Document;
use std::borrow::Cow;

/// Serializer for converting Smithy types to strings (for HTTP headers, query params, labels).
#[derive(Debug)]
pub struct HttpStringSerializer {
    output: String,
    /// True while `write_list` is running its element writer.
    in_list: bool,
    /// True once a value has been written at the current level.
    wrote_value: bool,
}

impl HttpStringSerializer {
    /// Creates a new HTTP string serializer.
    pub fn new() -> Self {
        Self {
            output: String::new(),
            in_list: false,
            wrote_value: false,
        }
    }

    /// Writes one textual value, as a list element when inside a list.
    fn push_value(&mut self, value: &str) -> Result<(), SerdeError> {
        if self.in_list {
            if self.wrote_value {
                self.output.push_str(", ");
            }
            if value.is_empty() {
                self.output.push_str("\"\"");
            } else {
                self.output.push_str(&quote_header_value(value));
            }
        } else if self.wrote_value {
            return Err(SerdeError::write_failed(
                "only one value can be written outside a list",
            ));
        } else {
            self.output.push_str(value);
        }
        self.wrote_value = true;
        Ok(())
    }

    /// Finalizes the serialization and returns the output string.
    pub fn finish(self) -> String {
        self.output
    }
}

impl super::FinishSerializer for HttpStringSerializer {
    fn finish(self) -> Vec<u8> {
        self.output.into_bytes()
    }
}

impl Default for HttpStringSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeSerializer for HttpStringSerializer {
    fn write_struct(
        &mut self,
        _schema: &Schema<'_>,
        _value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "structures cannot be serialized to strings",
        ))
    }

    fn write_list(
        &mut self,
        _schema: &Schema<'_>,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        if self.in_list {
            return Err(SerdeError::unsupported(
                "nested lists cannot be serialized to strings",
            ));
        }
        if self.wrote_value {
            return Err(SerdeError::write_failed(
                "only one value can be written outside a list",
            ));
        }
        self.in_list = true;
        let result = write_elements(self);
        self.in_list = false;
        // The list is the single top-level value, even when it has no elements.
        self.wrote_value = true;
        result
    }

    fn write_map(
        &mut self,
        _schema: &Schema<'_>,
        _write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "maps cannot be serialized to strings",
        ))
    }

    fn write_boolean(&mut self, _schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        self.push_value(if value { "true" } else { "false" })
    }

    fn write_byte(&mut self, _schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        self.push_value(&value.to_string())
    }

    fn write_short(&mut self, _schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        self.push_value(&value.to_string())
    }

    fn write_integer(&mut self, _schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        self.push_value(&value.to_string())
    }

    fn write_long(&mut self, _schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        self.push_value(&value.to_string())
    }

    fn write_float(&mut self, _schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        if value.is_nan() {
            self.push_value("NaN")
        } else if value.is_infinite() {
            self.push_value(if value.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            })
        } else {
            self.push_value(&value.to_string())
        }
    }

    fn write_double(&mut self, _schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        if value.is_nan() {
            self.push_value("NaN")
        } else if value.is_infinite() {
            self.push_value(if value.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            })
        } else {
            self.push_value(&value.to_string())
        }
    }

    fn write_big_integer(
        &mut self,
        _schema: &Schema<'_>,
        value: &BigInteger,
    ) -> Result<(), SerdeError> {
        self.push_value(value.as_ref())
    }

    fn write_big_decimal(
        &mut self,
        _schema: &Schema<'_>,
        value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        self.push_value(value.as_ref())
    }

    fn write_string(&mut self, _schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        self.push_value(value)
    }

    fn write_blob(&mut self, _schema: &Schema<'_>, value: Blob) -> Result<(), SerdeError> {
        // Blobs are base64-encoded for string serialization
        self.push_value(&aws_smithy_types::base64::encode(value.as_ref()))
    }

    fn write_timestamp(
        &mut self,
        _schema: &Schema<'_>,
        value: &DateTime,
    ) -> Result<(), SerdeError> {
        // Default to HTTP date format for string serialization
        // TODO(schema): Check schema for timestampFormat trait
        let formatted = value
            .fmt(aws_smithy_types::date_time::Format::HttpDate)
            .map_err(|e| SerdeError::write_failed(format!("failed to format timestamp: {e}")))?;
        // An HTTP date contains a comma, so inside a list it is quoted.
        self.push_value(&formatted)
    }

    fn write_document(
        &mut self,
        _schema: &Schema<'_>,
        _value: &Document,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "documents cannot be serialized to strings",
        ))
    }

    fn write_null(&mut self, _schema: &Schema<'_>) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "null cannot be serialized to strings",
        ))
    }
}

/// Deserializer for parsing Smithy types from strings.
#[derive(Debug)]
pub struct HttpStringDeserializer<'a> {
    input: Cow<'a, str>,
    /// Whether the single top-level value has been read.
    consumed: bool,
    /// The remaining elements while `read_list` is running its consumer.
    list: Option<std::vec::IntoIter<String>>,
}

impl<'a> HttpStringDeserializer<'a> {
    /// Creates a new HTTP string deserializer from the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input: Cow::Borrowed(input),
            consumed: false,
            list: None,
        }
    }

    /// Splits the input into list elements with the RFC 7230 header-list rules.
    fn list_elements(&self) -> Result<Vec<String>, SerdeError> {
        if self.input.is_empty() {
            return Ok(Vec::new());
        }
        header_parse::read_many_from_str_bytes::<String>(std::iter::once(self.input.as_bytes()))
            .map_err(|e| SerdeError::invalid_input(format!("invalid list: {e}")))
    }

    /// The next value to read: the next element inside a list, else the whole input.
    fn next_value(&mut self, what: &str) -> Result<Cow<'_, str>, SerdeError> {
        let value = match &mut self.list {
            Some(elements) => elements.next().map(Cow::Owned),
            None if self.consumed => None,
            None => {
                self.consumed = true;
                Some(Cow::Borrowed(self.input.as_ref()))
            }
        };
        value.ok_or_else(|| SerdeError::invalid_input(format!("expected {what} value")))
    }
}

impl<'a> ShapeDeserializer for HttpStringDeserializer<'a> {
    fn read_struct(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(
            &Schema<'_>,
            &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "structures cannot be deserialized from strings",
        ))
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Invoke the consumer once per element. Each call drives a single element read
        // (e.g. `read_string`), which takes the next element via `next_value`. An empty
        // input is an empty list.
        if self.list.is_some() {
            return Err(SerdeError::unsupported(
                "nested lists cannot be deserialized from strings",
            ));
        }
        let elements = self.list_elements()?;
        self.consumed = true;
        let count = elements.len();
        self.list = Some(elements.into_iter());
        let mut result = Ok(());
        for _ in 0..count {
            result = consumer(self);
            if result.is_err() {
                break;
            }
        }
        self.list = None;
        result
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Err(SerdeError::unsupported(
            "maps cannot be deserialized from strings",
        ))
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        let value = self.next_value("boolean")?;
        let value = value.as_ref();
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid boolean: {value}")))
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        let value = self.next_value("byte")?;
        let value = value.as_ref();
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid byte: {value}")))
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        let value = self.next_value("short")?;
        let value = value.as_ref();
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid short: {value}")))
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        let value = self.next_value("integer")?;
        let value = value.as_ref();
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid integer: {value}")))
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        let value = self.next_value("long")?;
        let value = value.as_ref();
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid long: {value}")))
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        let value = self.next_value("float")?;
        let value = value.as_ref();
        match value {
            "NaN" => Ok(f32::NAN),
            "Infinity" => Ok(f32::INFINITY),
            "-Infinity" => Ok(f32::NEG_INFINITY),
            _ => value
                .parse()
                .map_err(|_| SerdeError::invalid_input(format!("invalid float: {value}"))),
        }
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        let value = self.next_value("double")?;
        let value = value.as_ref();
        match value {
            "NaN" => Ok(f64::NAN),
            "Infinity" => Ok(f64::INFINITY),
            "-Infinity" => Ok(f64::NEG_INFINITY),
            _ => value
                .parse()
                .map_err(|_| SerdeError::invalid_input(format!("invalid double: {value}"))),
        }
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        let value = self.next_value("big integer")?;
        let value = value.as_ref();
        use std::str::FromStr;
        BigInteger::from_str(value)
            .map_err(|_| SerdeError::invalid_input(format!("invalid big integer: {value}")))
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        let value = self.next_value("big decimal")?;
        let value = value.as_ref();
        use std::str::FromStr;
        BigDecimal::from_str(value)
            .map_err(|_| SerdeError::invalid_input(format!("invalid big decimal: {value}")))
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.next_value("string").map(Cow::into_owned)
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        let value = self.next_value("blob")?;
        let value = value.as_ref();
        let decoded = aws_smithy_types::base64::decode(value)
            .map_err(|e| SerdeError::invalid_input(format!("invalid base64: {e}")))?;
        Ok(Blob::new(decoded))
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        let value = self.next_value("timestamp")?;
        let value = value.as_ref();
        // Try HTTP date format first, then fall back to other formats
        // TODO(schema): Check schema for timestampFormat trait
        DateTime::from_str(value, aws_smithy_types::date_time::Format::HttpDate)
            .or_else(|_| DateTime::from_str(value, aws_smithy_types::date_time::Format::DateTime))
            .map_err(|e| SerdeError::invalid_input(format!("invalid timestamp: {e}")))
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Err(SerdeError::unsupported(
            "documents cannot be deserialized from strings",
        ))
    }

    fn is_null(&self) -> bool {
        // List elements cannot be null; a top-level value is null when the input is empty.
        self.list.is_none() && self.input.is_empty()
    }

    /// Nothing to advance outside a list. This deserializer holds a single already-extracted
    /// value and cannot read structures at all, so it is never the parent of a member
    /// consumer; declining the value simply means not reading it. Inside a list, skipping
    /// consumes one element so the next read sees the following one.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        if let Some(elements) = &mut self.list {
            elements.next();
        }
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        match &self.list {
            Some(elements) => Some(elements.len()),
            None => self.list_elements().ok().map(|elements| elements.len()),
        }
    }
}

/// HTTP string codec for serializing/deserializing to/from strings.
#[derive(Debug)]
pub struct HttpStringCodec;

impl crate::codec::Codec for HttpStringCodec {
    type Serializer = HttpStringSerializer;
    type Deserializer<'a> = HttpStringDeserializer<'a>;

    fn create_serializer(&self) -> Self::Serializer {
        HttpStringSerializer::new()
    }

    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
        let input_str = std::str::from_utf8(input).unwrap_or("");
        HttpStringDeserializer::new(input_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn test_serialize_boolean() {
        let mut ser = HttpStringSerializer::new();
        ser.write_boolean(&BOOLEAN, true).unwrap();
        assert_eq!(ser.finish(), "true");

        let mut ser = HttpStringSerializer::new();
        ser.write_boolean(&BOOLEAN, false).unwrap();
        assert_eq!(ser.finish(), "false");
    }

    #[test]
    fn test_serialize_integers() {
        let mut ser = HttpStringSerializer::new();
        ser.write_byte(&BYTE, 42).unwrap();
        assert_eq!(ser.finish(), "42");

        let mut ser = HttpStringSerializer::new();
        ser.write_integer(&INTEGER, -123).unwrap();
        assert_eq!(ser.finish(), "-123");

        let mut ser = HttpStringSerializer::new();
        ser.write_long(&LONG, 9876543210).unwrap();
        assert_eq!(ser.finish(), "9876543210");
    }

    #[test]
    fn test_serialize_floats() {
        let mut ser = HttpStringSerializer::new();
        ser.write_float(&FLOAT, 3.15).unwrap();
        assert_eq!(ser.finish(), "3.15");

        let mut ser = HttpStringSerializer::new();
        ser.write_float(&FLOAT, f32::NAN).unwrap();
        assert_eq!(ser.finish(), "NaN");

        let mut ser = HttpStringSerializer::new();
        ser.write_float(&FLOAT, f32::INFINITY).unwrap();
        assert_eq!(ser.finish(), "Infinity");
    }

    #[test]
    fn test_serialize_string() {
        let mut ser = HttpStringSerializer::new();
        ser.write_string(&STRING, "hello world").unwrap();
        assert_eq!(ser.finish(), "hello world");
    }

    #[test]
    fn test_serialize_list() {
        let mut ser = HttpStringSerializer::new();
        ser.write_list(&STRING, &|s: &mut dyn ShapeSerializer| {
            s.write_string(&STRING, "a")?;
            s.write_string(&STRING, "b")?;
            s.write_string(&STRING, "c")?;
            Ok(())
        })
        .unwrap();
        assert_eq!(ser.finish(), "a, b, c");
    }

    #[test]
    fn test_serialize_blob() {
        let mut ser = HttpStringSerializer::new();
        let blob = Blob::new(vec![1, 2, 3, 4]);
        ser.write_blob(&BLOB, blob).unwrap();
        // Base64 encoding of [1, 2, 3, 4]
        assert_eq!(ser.finish(), "AQIDBA==");
    }

    #[test]
    fn test_deserialize_boolean() {
        let mut deser = HttpStringDeserializer::new("true");
        assert!(deser.read_boolean(&BOOLEAN).unwrap());

        let mut deser = HttpStringDeserializer::new("false");
        assert!(!(deser.read_boolean(&BOOLEAN).unwrap()));
    }

    #[test]
    fn test_deserialize_integers() {
        let mut deser = HttpStringDeserializer::new("42");
        assert_eq!(deser.read_byte(&BYTE).unwrap(), 42);

        let mut deser = HttpStringDeserializer::new("-123");
        assert_eq!(deser.read_integer(&INTEGER).unwrap(), -123);

        let mut deser = HttpStringDeserializer::new("9876543210");
        assert_eq!(deser.read_long(&LONG).unwrap(), 9876543210);
    }

    #[test]
    fn test_deserialize_floats() {
        let mut deser = HttpStringDeserializer::new("3.15");
        assert!((deser.read_float(&FLOAT).unwrap() - 3.15).abs() < 0.01);

        let mut deser = HttpStringDeserializer::new("NaN");
        assert!(deser.read_float(&FLOAT).unwrap().is_nan());

        let mut deser = HttpStringDeserializer::new("Infinity");
        assert_eq!(deser.read_float(&FLOAT).unwrap(), f32::INFINITY);
    }

    #[test]
    fn test_deserialize_string() {
        let mut deser = HttpStringDeserializer::new("hello world");
        assert_eq!(deser.read_string(&STRING).unwrap(), "hello world");
    }

    #[test]
    fn scalar_read_is_the_whole_value() {
        // Outside a list there is no comma splitting, as for a scalar HTTP header.
        let mut deser = HttpStringDeserializer::new("a, b,c");
        assert_eq!(deser.read_string(&STRING).unwrap(), "a, b,c");
        assert!(
            deser.read_string(&STRING).is_err(),
            "the single value was already read"
        );
    }

    #[test]
    fn only_one_value_can_be_written_outside_a_list() {
        let mut ser = HttpStringSerializer::new();
        ser.write_string(&STRING, "a").unwrap();
        assert!(ser.write_string(&STRING, "b").is_err());
    }

    fn string_list() -> Schema<'static> {
        Schema::new_list(crate::shape_id!("ns", "StringList"), &STRING)
    }

    fn write_strings(items: &[&str]) -> String {
        let mut ser = HttpStringSerializer::new();
        ser.write_list(&string_list(), &|e| {
            for it in items {
                e.write_string(&STRING, it)?;
            }
            Ok(())
        })
        .unwrap();
        ser.finish()
    }

    fn read_strings(wire: &str) -> Result<Vec<String>, SerdeError> {
        let mut de = HttpStringDeserializer::new(wire);
        let mut out = Vec::new();
        de.read_list(&string_list(), &mut |x| {
            out.push(x.read_string(&STRING)?);
            Ok(())
        })?;
        Ok(out)
    }

    #[test]
    fn string_list_round_trips() {
        // From review of PR #4871, extended with whitespace, quotes, parentheses and a
        // single empty element.
        let cases: [&[&str]; 11] = [
            &["a", "b", "c"],
            &["", "b"],
            &["a", ""],
            &["a,b"],
            &["a, b"],
            &[" a"],
            &["a "],
            &["\"q\""],
            &["(x)"],
            &[""],
            &[],
        ];
        for case in cases {
            let wire = write_strings(case);
            let res = read_strings(&wire);
            assert!(res.is_ok(), "read_list failed on wire {wire:?}: {res:?}");
            assert_eq!(
                res.unwrap(),
                case,
                "round trip changed the list; wire was {wire:?}"
            );
        }
    }

    #[test]
    fn list_wire_format_matches_header_lists() {
        assert_eq!(write_strings(&["a", "b", "c"]), "a, b, c");
        assert_eq!(write_strings(&["a,b", ""]), "\"a,b\", \"\"");
        // Lists written by other header writers read back with the same rules.
        assert_eq!(
            read_strings("a,b , \"c,d\"").unwrap(),
            vec!["a", "b", "c,d"]
        );
        assert!(read_strings("\"unterminated").is_err());
    }

    #[test]
    fn timestamp_list_round_trips() {
        // An HTTP date contains a comma, so it must be quoted inside a list.
        let list = Schema::new_list(crate::shape_id!("ns", "TsList"), &TIMESTAMP);
        let times = [DateTime::from_secs(0), DateTime::from_secs(1515531081)];
        let mut ser = HttpStringSerializer::new();
        ser.write_list(&list, &|e| {
            for t in &times {
                e.write_timestamp(&TIMESTAMP, t)?;
            }
            Ok(())
        })
        .unwrap();
        let wire = ser.finish();
        let mut de = HttpStringDeserializer::new(&wire);
        let mut out = Vec::new();
        de.read_list(&list, &mut |x| {
            out.push(x.read_timestamp(&TIMESTAMP)?);
            Ok(())
        })
        .unwrap();
        assert_eq!(out, times, "wire was {wire:?}");
    }

    #[test]
    fn integer_list_round_trips() {
        let list = Schema::new_list(crate::shape_id!("ns", "IntList"), &INTEGER);
        let mut ser = HttpStringSerializer::new();
        ser.write_list(&list, &|e| {
            e.write_integer(&INTEGER, 1)?;
            e.write_integer(&INTEGER, -2)
        })
        .unwrap();
        let wire = ser.finish();
        assert_eq!(wire, "1, -2");
        let mut de = HttpStringDeserializer::new(&wire);
        let mut out = Vec::new();
        de.read_list(&list, &mut |x| {
            out.push(x.read_integer(&INTEGER)?);
            Ok(())
        })
        .unwrap();
        assert_eq!(out, [1, -2]);
    }

    #[test]
    fn skip_value_inside_a_list_consumes_one_element() {
        let mut de = HttpStringDeserializer::new("a, b, c");
        let mut out = Vec::new();
        let mut first = true;
        de.read_list(&string_list(), &mut |x| {
            if std::mem::take(&mut first) {
                x.skip_value()
            } else {
                out.push(x.read_string(&STRING)?);
                Ok(())
            }
        })
        .unwrap();
        assert_eq!(out, ["b", "c"]);
    }

    #[test]
    fn test_read_list_drives_consumer_per_element() {
        let mut deser = HttpStringDeserializer::new("a,b,c");
        let mut collected = Vec::new();
        deser
            .read_list(&STRING, &mut |d: &mut dyn ShapeDeserializer| {
                collected.push(d.read_string(&STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_read_list_empty_input_is_empty_list() {
        let mut deser = HttpStringDeserializer::new("");
        let mut count = 0;
        deser
            .read_list(&STRING, &mut |_d: &mut dyn ShapeDeserializer| {
                count += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_deserialize_blob() {
        let mut deser = HttpStringDeserializer::new("AQIDBA==");
        let blob = deser.read_blob(&BLOB).unwrap();
        assert_eq!(blob.as_ref(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_container_size() {
        let deser = HttpStringDeserializer::new("a,b,c");
        assert_eq!(deser.container_size(), Some(3));

        let deser = HttpStringDeserializer::new("single");
        assert_eq!(deser.container_size(), Some(1));
    }

    #[test]
    fn test_is_null() {
        let deser = HttpStringDeserializer::new("");
        assert!(deser.is_null());

        let deser = HttpStringDeserializer::new("value");
        assert!(!deser.is_null());
    }

    #[test]
    fn test_codec_trait() {
        use crate::codec::Codec;

        let codec = HttpStringCodec;

        // Test serialization through codec
        let mut ser = codec.create_serializer();
        ser.write_string(&STRING, "test").unwrap();
        let output = ser.finish();
        assert_eq!(output, "test");

        // Test deserialization through codec
        let input = b"hello";
        let mut deser = codec.create_deserializer(input);
        let result = deser.read_string(&STRING).unwrap();
        assert_eq!(result, "hello");
    }
}

/// Tests for the [`ShapeDeserializer::skip_value`] contract.
#[cfg(test)]
mod skip_value_contract {
    use super::*;
    use crate::prelude::*;
    use crate::serde::ShapeDeserializer;

    #[test]
    fn skip_value_is_a_no_op_and_leaves_the_value_readable() {
        // This deserializer holds a single already-extracted value and cannot read
        // structures, so it is never the parent of a member consumer. Skipping means
        // "don't read it", with nothing to advance — and notably must not fall through to
        // the allocating trait default, whose `read_document` this type rejects.
        let mut deser = HttpStringDeserializer::new("hello");
        let dynamic: &mut dyn ShapeDeserializer = &mut deser;
        dynamic.skip_value().expect("skip must succeed");
        assert_eq!(deser.read_string(&STRING).unwrap(), "hello");
    }

    #[test]
    fn skip_value_succeeds_where_the_default_would_fail() {
        // Guards the override: `read_document` is unsupported here, so if the override
        // were removed this would start returning an error.
        let mut deser = HttpStringDeserializer::new("value");
        assert!(
            deser.read_document(&DOCUMENT).is_err(),
            "precondition: this deserializer cannot produce documents"
        );
        let mut deser = HttpStringDeserializer::new("value");
        deser
            .skip_value()
            .expect("override must not use read_document");
    }
}
