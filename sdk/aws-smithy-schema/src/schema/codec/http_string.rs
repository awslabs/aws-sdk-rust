/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! String codec for HTTP bindings (headers, query params, URI labels).

use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
use crate::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime};

use aws_smithy_types::Document;

/// Serializer for converting Smithy types to strings (for HTTP headers, query params, labels).
#[derive(Debug)]
pub struct HttpStringSerializer {
    output: String,
}

impl HttpStringSerializer {
    /// Creates a new HTTP string serializer.
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
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
        // Lists are serialized as comma-separated values
        write_elements(self)
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
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    fn write_byte(&mut self, _schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(&value.to_string());
        Ok(())
    }

    fn write_short(&mut self, _schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(&value.to_string());
        Ok(())
    }

    fn write_integer(&mut self, _schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(&value.to_string());
        Ok(())
    }

    fn write_long(&mut self, _schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(&value.to_string());
        Ok(())
    }

    fn write_float(&mut self, _schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        if value.is_nan() {
            self.output.push_str("NaN");
        } else if value.is_infinite() {
            self.output.push_str(if value.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            });
        } else {
            self.output.push_str(&value.to_string());
        }
        Ok(())
    }

    fn write_double(&mut self, _schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        if value.is_nan() {
            self.output.push_str("NaN");
        } else if value.is_infinite() {
            self.output.push_str(if value.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            });
        } else {
            self.output.push_str(&value.to_string());
        }
        Ok(())
    }

    fn write_big_integer(
        &mut self,
        _schema: &Schema<'_>,
        value: &BigInteger,
    ) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(value.as_ref());
        Ok(())
    }

    fn write_big_decimal(
        &mut self,
        _schema: &Schema<'_>,
        value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(value.as_ref());
        Ok(())
    }

    fn write_string(&mut self, _schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        self.output.push_str(value);
        Ok(())
    }

    fn write_blob(&mut self, _schema: &Schema<'_>, value: Blob) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        // Blobs are base64-encoded for string serialization
        self.output
            .push_str(&aws_smithy_types::base64::encode(value.as_ref()));
        Ok(())
    }

    fn write_timestamp(
        &mut self,
        _schema: &Schema<'_>,
        value: &DateTime,
    ) -> Result<(), SerdeError> {
        if !self.output.is_empty() {
            self.output.push(',');
        }
        // Default to HTTP date format for string serialization
        // TODO(schema): Check schema for timestampFormat trait
        let formatted = value
            .fmt(aws_smithy_types::date_time::Format::HttpDate)
            .map_err(|e| SerdeError::write_failed(format!("failed to format timestamp: {e}")))?;
        self.output.push_str(&formatted);
        Ok(())
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
    input: std::borrow::Cow<'a, str>,
    position: usize,
}

impl<'a> HttpStringDeserializer<'a> {
    /// Creates a new HTTP string deserializer from the given input.
    pub fn new(input: &'a str) -> Self {
        Self {
            input: std::borrow::Cow::Borrowed(input),
            position: 0,
        }
    }

    fn next_value(&mut self) -> Option<&str> {
        if self.position >= self.input.len() {
            return None;
        }

        let start = self.position;
        if let Some(comma_pos) = self.input[start..].find(',') {
            let end = start + comma_pos;
            self.position = end + 1;
            Some(&self.input[start..end])
        } else {
            self.position = self.input.len();
            Some(&self.input[start..])
        }
    }

    fn current_value(&self) -> &str {
        &self.input[self.position..]
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
        // Comma-separated values: invoke the consumer once per element. Each
        // call drives a single element read (e.g. `read_string`), which pulls
        // the next comma-delimited token via `next_value`. An empty input is
        // an empty list.
        if self.current_value().is_empty() {
            return Ok(());
        }
        let count = self.current_value().matches(',').count() + 1;
        for _ in 0..count {
            consumer(self)?;
        }
        Ok(())
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
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected boolean value"))?;
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid boolean: {value}")))
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected byte value"))?;
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid byte: {value}")))
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected short value"))?;
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid short: {value}")))
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected integer value"))?;
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid integer: {value}")))
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected long value"))?;
        value
            .parse()
            .map_err(|_| SerdeError::invalid_input(format!("invalid long: {value}")))
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected float value"))?;
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
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected double value"))?;
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
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected big integer value"))?;
        use std::str::FromStr;
        BigInteger::from_str(value)
            .map_err(|_| SerdeError::invalid_input(format!("invalid big integer: {value}")))
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected big decimal value"))?;
        use std::str::FromStr;
        BigDecimal::from_str(value)
            .map_err(|_| SerdeError::invalid_input(format!("invalid big decimal: {value}")))
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected string value"))
            .map(|s| s.to_string())
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected blob value"))?;
        let decoded = aws_smithy_types::base64::decode(value)
            .map_err(|e| SerdeError::invalid_input(format!("invalid base64: {e}")))?;
        Ok(Blob::new(decoded))
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        let value = self
            .next_value()
            .ok_or_else(|| SerdeError::invalid_input("expected timestamp value"))?;
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
        self.current_value().is_empty()
    }

    fn container_size(&self) -> Option<usize> {
        // Count commas + 1 for list size estimation
        Some(self.input.matches(',').count() + 1)
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
        assert_eq!(ser.finish(), "a,b,c");
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
    fn test_deserialize_list() {
        let mut deser = HttpStringDeserializer::new("a,b,c");
        let values = vec![
            deser.read_string(&STRING).unwrap(),
            deser.read_string(&STRING).unwrap(),
            deser.read_string(&STRING).unwrap(),
        ];
        assert_eq!(values, vec!["a", "b", "c"]);
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
