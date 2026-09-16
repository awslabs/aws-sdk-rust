/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON deserializer implementation.

use aws_smithy_schema::serde::SerdeError;
use aws_smithy_schema::serde::ShapeDeserializer;
use aws_smithy_schema::Schema;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

use crate::codec::JsonCodecSettings;
use crate::deserialize::{json_token_iter, Token};

use std::sync::Arc;

/// Maximum recursion depth for deserialization. Payloads nested deeper than
/// this will produce a [`SerdeError`] instead of risking a stack overflow.
/// Matches the default used by `serde_json`.
pub(crate) const MAX_DESERIALIZE_DEPTH: u32 = 128;

/// JSON deserializer that implements the ShapeDeserializer trait.
pub struct JsonDeserializer<'a> {
    input: &'a [u8],
    position: usize,
    settings: Arc<JsonCodecSettings>,
    depth: u32,
}

impl<'a> JsonDeserializer<'a> {
    /// Creates a new JSON deserializer with the given settings.
    pub(crate) fn new(input: &'a [u8], settings: Arc<JsonCodecSettings>) -> Self {
        Self {
            input,
            position: 0,
            settings,
            depth: 0,
        }
    }

    /// Resolves a JSON field name to a member schema.
    fn resolve_member<'s>(&self, schema: &'s Schema, field_name: &str) -> Option<&'s Schema> {
        self.settings.field_to_member(schema, field_name)
    }

    fn remaining(&self) -> &[u8] {
        &self.input[self.position..]
    }

    fn advance_by(&mut self, n: usize) {
        self.position = (self.position + n).min(self.input.len());
    }

    /// Parse a JSON quoted string key directly from bytes, advancing past it.
    /// Assumes the current position is at the opening `"`.
    /// Returns a borrowed `&str` when no escape sequences are present (common case),
    /// avoiding a heap allocation per JSON key.
    fn parse_key(&mut self) -> Result<std::borrow::Cow<'a, str>, SerdeError> {
        let start = self.position + 1; // skip opening quote
        self.position += 1;
        let input = self.input;
        let remaining = &input[start..];
        let mut i = 0;
        let mut has_escapes = false;
        let mut found_end = false;
        while i < remaining.len() {
            match remaining[i] {
                b'"' => {
                    found_end = true;
                    break;
                }
                b'\\' => {
                    has_escapes = true;
                    i += 2;
                }
                0..=31 if self.settings.enforce_strictness => {
                    return Err(SerdeError::InvalidInput {
                        message: "raw control character in string".into(),
                    });
                }
                _ => i += 1,
            }
        }
        if !found_end {
            return Err(SerdeError::InvalidInput {
                message: "unterminated string key".into(),
            });
        }
        self.position = start + i + 1; // advance past key bytes + closing quote
        let key_bytes = &input[start..start + i];
        if has_escapes {
            let raw = std::str::from_utf8(key_bytes).map_err(|e| SerdeError::InvalidInput {
                message: e.to_string(),
            })?;
            Ok(std::borrow::Cow::Owned(
                crate::escape::unescape_string(raw)
                    .map_err(|e| SerdeError::InvalidInput {
                        message: e.to_string(),
                    })?
                    .into_owned(),
            ))
        } else {
            Ok(std::borrow::Cow::Borrowed(
                std::str::from_utf8(key_bytes).map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })?,
            ))
        }
    }
}

impl<'a> ShapeDeserializer for JsonDeserializer<'a> {
    fn read_struct(
        &mut self,
        schema: &Schema,
        consumer: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        // Expect opening brace
        self.skip_whitespace();
        if self.remaining().is_empty() {
            if self.settings.enforce_strictness && (!self.input.is_empty() || self.depth != 1) {
                return Err(SerdeError::InvalidInput {
                    message: "expected object".into(),
                });
            }
            // Treat empty input as an empty object (e.g., empty HTTP response body)
            self.depth -= 1;
            return Ok(());
        }
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::TypeMismatch {
                message: "expected object".into(),
            });
        }
        self.advance_by(1);

        let mut first = true;
        loop {
            // Stop at the end of the object; otherwise the next key/value pair starts here.
            if self.next_element(first, b'}', "object")? {
                break;
            }
            first = false;
            if self.remaining().first() != Some(&b'"') {
                return Err(SerdeError::InvalidInput {
                    message: "expected object key".into(),
                });
            }

            // Parse the key directly from bytes
            let key_str = self.parse_key()?;

            // Skip whitespace and expect colon
            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::InvalidInput {
                    message: "expected colon after key".into(),
                });
            }
            self.advance_by(1);
            self.skip_whitespace();

            // Process the value — skip nulls (they represent absent optional members)
            let rem = self.remaining();
            if rem.starts_with(b"null") && !rem.get(4).is_some_and(|b| b.is_ascii_alphanumeric()) {
                self.advance_by(4);
            } else if let Some(member_schema) = self.resolve_member(schema, &key_str) {
                consumer(member_schema, self)?;
            } else {
                self.skip_value()?;
            }
        }

        self.leave_container()?;
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::TypeMismatch {
                message: "expected array".into(),
            });
        }
        self.advance_by(1);

        let mut first = true;
        loop {
            if self.next_element(first, b']', "array")? {
                break;
            }
            first = false;
            consumer(self)?;
        }

        self.leave_container()?;
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::TypeMismatch {
                message: "expected object".into(),
            });
        }
        self.advance_by(1);

        let mut first = true;
        loop {
            if self.next_element(first, b'}', "object")? {
                break;
            }
            first = false;
            if self.remaining().first() != Some(&b'"') {
                return Err(SerdeError::InvalidInput {
                    message: "expected key".into(),
                });
            }

            let key = self.parse_key()?;

            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::InvalidInput {
                    message: "expected colon".into(),
                });
            }
            self.advance_by(1);
            self.skip_whitespace();

            consumer(key.into_owned(), self)?;
        }

        self.leave_container()?;
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema) -> Result<bool, SerdeError> {
        self.skip_whitespace();
        let rem = self.remaining();
        if rem.starts_with(b"true") {
            self.advance_by(4);
            Ok(true)
        } else if rem.starts_with(b"false") {
            self.advance_by(5);
            Ok(false)
        } else {
            Err(SerdeError::TypeMismatch {
                message: "expected boolean".into(),
            })
        }
    }

    fn read_byte(&mut self, _schema: &Schema) -> Result<i8, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i8::try_from(n).map_err(|_| SerdeError::InvalidInput {
                message: "value out of range for byte".into(),
            })
        })
    }

    fn read_short(&mut self, _schema: &Schema) -> Result<i16, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i16::try_from(n).map_err(|_| SerdeError::InvalidInput {
                message: "value out of range for short".into(),
            })
        })
    }

    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i32::try_from(n).map_err(|_| SerdeError::InvalidInput {
                message: "value out of range for integer".into(),
            })
        })
    }

    fn read_long(&mut self, _schema: &Schema) -> Result<i64, SerdeError> {
        self.read_integer_value()
    }

    fn read_float(&mut self, _schema: &Schema) -> Result<f32, SerdeError> {
        self.read_float_value().map(|f| f as f32)
    }

    fn read_double(&mut self, _schema: &Schema) -> Result<f64, SerdeError> {
        self.read_float_value()
    }

    fn read_big_integer(&mut self, _schema: &Schema) -> Result<BigInteger, SerdeError> {
        use std::str::FromStr;
        self.skip_whitespace();
        match self.remaining().first() {
            Some(b'-') | Some(b'0'..=b'9') => {
                let start = self.position;
                self.consume_number()?;
                let num_str =
                    std::str::from_utf8(&self.input[start..self.position]).map_err(|e| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                BigInteger::from_str(num_str).map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })
            }
            _ => Err(SerdeError::TypeMismatch {
                message: "expected number".into(),
            }),
        }
    }

    fn read_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, SerdeError> {
        use std::str::FromStr;
        self.skip_whitespace();
        match self.remaining().first() {
            Some(b'-') | Some(b'0'..=b'9') => {
                let start = self.position;
                self.consume_number()?;
                let num_str =
                    std::str::from_utf8(&self.input[start..self.position]).map_err(|e| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                BigDecimal::from_str(num_str).map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })
            }
            _ => Err(SerdeError::TypeMismatch {
                message: "expected number".into(),
            }),
        }
    }

    fn read_string(&mut self, _schema: &Schema) -> Result<String, SerdeError> {
        self.skip_whitespace();
        let pos = self.position;
        let input = self.input;
        let rem = &input[pos..];
        if rem.first() != Some(&b'"') {
            return Err(SerdeError::TypeMismatch {
                message: "expected string".into(),
            });
        }
        // Scan for end of string, tracking whether escapes are present
        let mut i = 1;
        let mut has_escape = false;
        while i < rem.len() {
            if rem[i] == b'\\' {
                has_escape = true;
                i += 2;
            } else if rem[i] == b'"' {
                let raw = &input[pos + 1..pos + i];
                self.position = pos + i + 1;
                if !has_escape {
                    return std::str::from_utf8(raw).map(|s| s.to_owned()).map_err(|e| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    });
                }
                let s = std::str::from_utf8(raw).map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })?;
                return crate::deserialize::EscapedStr::new(s)
                    .to_unescaped()
                    .map(|s| s.into_owned())
                    .map_err(|e| SerdeError::InvalidInput {
                        message: e.to_string(),
                    });
            } else {
                if self.settings.enforce_strictness && rem[i] < 32 {
                    return Err(SerdeError::InvalidInput {
                        message: "raw control character in string".into(),
                    });
                }
                i += 1;
            }
        }
        Err(SerdeError::InvalidInput {
            message: "unterminated string".into(),
        })
    }

    fn read_blob(&mut self, _schema: &Schema) -> Result<Blob, SerdeError> {
        let s = self.read_string(_schema)?;
        let decoded =
            aws_smithy_types::base64::decode(&s).map_err(|e| SerdeError::InvalidInput {
                message: format!("invalid base64: {}", e),
            })?;
        Ok(Blob::new(decoded))
    }

    fn read_string_list(&mut self, _schema: &Schema) -> Result<Vec<String>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::TypeMismatch {
                message: "expected array".into(),
            });
        }
        self.advance_by(1);
        let mut out = Vec::new();
        let mut first = true;
        loop {
            if self.next_element(first, b']', "array")? {
                break;
            }
            first = false;
            out.push(self.read_string(_schema)?);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_blob_list(&mut self, _schema: &Schema) -> Result<Vec<Blob>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::TypeMismatch {
                message: "expected array".into(),
            });
        }
        self.advance_by(1);
        let mut out = Vec::new();
        let mut first = true;
        loop {
            if self.next_element(first, b']', "array")? {
                break;
            }
            first = false;
            out.push(self.read_blob(_schema)?);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_integer_list(&mut self, _schema: &Schema) -> Result<Vec<i32>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::TypeMismatch {
                message: "expected array".into(),
            });
        }
        self.advance_by(1);
        let mut out = Vec::new();
        let mut first = true;
        loop {
            if self.next_element(first, b']', "array")? {
                break;
            }
            first = false;
            out.push(self.read_integer(_schema)?);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_long_list(&mut self, _schema: &Schema) -> Result<Vec<i64>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::TypeMismatch {
                message: "expected array".into(),
            });
        }
        self.advance_by(1);
        let mut out = Vec::new();
        let mut first = true;
        loop {
            if self.next_element(first, b']', "array")? {
                break;
            }
            first = false;
            out.push(self.read_long(_schema)?);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_string_string_map(
        &mut self,
        _schema: &Schema,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::TypeMismatch {
                message: "expected object".into(),
            });
        }
        self.advance_by(1);
        let mut out = std::collections::HashMap::new();
        let mut first = true;
        loop {
            if self.next_element(first, b'}', "object")? {
                break;
            }
            first = false;
            if self.remaining().first() != Some(&b'"') {
                return Err(SerdeError::InvalidInput {
                    message: "expected key".into(),
                });
            }
            let key = self.parse_key()?;
            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::InvalidInput {
                    message: "expected colon".into(),
                });
            }
            self.advance_by(1);
            self.skip_whitespace();
            let val = self.read_string(_schema)?;
            out.insert(key.into_owned(), val);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_timestamp(&mut self, schema: &Schema) -> Result<DateTime, SerdeError> {
        use aws_smithy_schema::traits::TimestampFormat;
        use aws_smithy_types::date_time::Format;

        self.skip_whitespace();
        let strict = self.settings.strict_timestamp_formats();
        // The wire form the member prescribes: its `@timestampFormat`, else the codec default.
        let format = match schema.timestamp_format().map(|t| t.format()) {
            Some(TimestampFormat::EpochSeconds) => Format::EpochSeconds,
            Some(TimestampFormat::DateTime) => Format::DateTime,
            Some(TimestampFormat::HttpDate) => Format::HttpDate,
            None => self.settings.default_timestamp_format(),
        };
        match self.remaining().first() {
            Some(b'"') => {
                let string_format = if strict {
                    // Strict: the string must be in exactly the prescribed format, and
                    // `epoch-seconds` is never a string. This mirrors the token-based
                    // parser's `expect_timestamp_or_null`, which the legacy server uses.
                    if matches!(format, Format::EpochSeconds) {
                        return Err(SerdeError::TypeMismatch {
                            message: "expected a JSON number for an epoch-seconds timestamp".into(),
                        });
                    }
                    format
                } else {
                    // Lenient (unchanged behavior): an explicit `@timestampFormat` of
                    // `http-date` or `epoch-seconds` is honored as written; `date-time` and
                    // members without the trait parse as offset-aware `date-time`.
                    match schema.timestamp_format().map(|t| t.format()) {
                        Some(TimestampFormat::HttpDate) => Format::HttpDate,
                        Some(TimestampFormat::EpochSeconds) => Format::EpochSeconds,
                        _ => Format::DateTimeWithOffset,
                    }
                };
                let s = self.read_string(schema)?;
                DateTime::from_str(&s, string_format)
                    .map_err(|e| SerdeError::custom(format!("invalid timestamp string: {e}")))
            }
            Some(b'-') | Some(b'0'..=b'9') => {
                if strict && !matches!(format, Format::EpochSeconds) {
                    return Err(SerdeError::TypeMismatch {
                        message: "expected a JSON string for a date-time or http-date timestamp"
                            .into(),
                    });
                }
                // Numeric timestamp: epoch seconds.
                let start = self.position;
                self.consume_number()?;
                let num_str =
                    std::str::from_utf8(&self.input[start..self.position]).map_err(|e| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
                    let f: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                    // The i64 range as f64. `i64::MAX as f64` rounds up to 2^63, which is
                    // why the range is half-open: 2^63 itself overflows `floor() as i64`
                    // and would saturate silently in `DateTime::from_secs_f64`.
                    const I64_MIN_F64: f64 = i64::MIN as f64;
                    const I64_MAX_F64: f64 = i64::MAX as f64;
                    if self.settings.enforce_strictness
                        && (!f.is_finite() || !(I64_MIN_F64..I64_MAX_F64).contains(&f))
                    {
                        return Err(SerdeError::InvalidInput {
                            message: "epoch-seconds value out of range".into(),
                        });
                    }
                    Ok(DateTime::from_secs_f64(f))
                } else if num_str.starts_with('-') {
                    let n: i64 = num_str.parse().map_err(|e: std::num::ParseIntError| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(DateTime::from_secs(n))
                } else {
                    let n: u64 = num_str.parse().map_err(|e: std::num::ParseIntError| {
                        SerdeError::InvalidInput {
                            message: e.to_string(),
                        }
                    })?;
                    Ok(DateTime::from_secs(n as i64))
                }
            }
            _ => Err(SerdeError::TypeMismatch {
                message: "expected timestamp".into(),
            }),
        }
    }

    fn read_document(&mut self, _schema: &Schema) -> Result<Document, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        let result = match self.remaining().first() {
            Some(b'"') => Ok(Document::String(self.read_string(_schema)?)),
            Some(b't') | Some(b'f') => Ok(Document::Bool(self.read_boolean(_schema)?)),
            Some(b'n') => {
                if self.remaining().starts_with(b"null") {
                    self.advance_by(4);
                    Ok(Document::Null)
                } else {
                    Err(SerdeError::InvalidInput {
                        message: "unexpected token in document".into(),
                    })
                }
            }
            Some(b'{') => {
                self.advance_by(1);
                let mut map = std::collections::HashMap::new();
                let mut first = true;
                loop {
                    if self.next_element(first, b'}', "document object")? {
                        break;
                    }
                    first = false;
                    if self.remaining().first() != Some(&b'"') {
                        return Err(SerdeError::InvalidInput {
                            message: "expected object key in document".into(),
                        });
                    }
                    let key = self.parse_key()?.into_owned();
                    self.skip_whitespace();
                    if self.remaining().first() != Some(&b':') {
                        return Err(SerdeError::InvalidInput {
                            message: "expected colon in document object".into(),
                        });
                    }
                    self.advance_by(1);
                    let value = self.read_document(_schema)?;
                    map.insert(key, value);
                }
                Ok(Document::Object(map))
            }
            Some(b'[') => {
                self.advance_by(1);
                let mut arr = Vec::new();
                let mut first = true;
                loop {
                    if self.next_element(first, b']', "document array")? {
                        break;
                    }
                    first = false;
                    arr.push(self.read_document(_schema)?);
                }
                Ok(Document::Array(arr))
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => {
                // Parse number — determine if integer or float
                if self.settings.enforce_strictness {
                    let start = self.position;
                    self.consume_number()?;
                    self.position = start;
                }
                let rem = self.remaining();
                let mut len = 0;
                let mut is_float = false;
                let mut is_negative = false;
                for (i, &b) in rem.iter().enumerate() {
                    if b == b'-' && i == 0 {
                        is_negative = true;
                        len += 1;
                    } else if b.is_ascii_digit() || b == b'+' {
                        len += 1;
                    } else if b == b'.' || b == b'e' || b == b'E' {
                        is_float = true;
                        len += 1;
                    } else {
                        break;
                    }
                }
                let pos = self.position;
                self.advance_by(len);
                let s = std::str::from_utf8(&self.input[pos..pos + len]).map_err(|e| {
                    SerdeError::InvalidInput {
                        message: e.to_string(),
                    }
                })?;
                if is_float {
                    let f = s.parse::<f64>().map_err(|e| SerdeError::InvalidInput {
                        message: e.to_string(),
                    })?;
                    Ok(Document::Number(aws_smithy_types::Number::Float(f)))
                } else if is_negative {
                    let n = s.parse::<i64>().map_err(|e| SerdeError::InvalidInput {
                        message: e.to_string(),
                    })?;
                    Ok(Document::Number(aws_smithy_types::Number::NegInt(n)))
                } else {
                    let n = s.parse::<u64>().map_err(|e| SerdeError::InvalidInput {
                        message: e.to_string(),
                    })?;
                    Ok(Document::Number(aws_smithy_types::Number::PosInt(n)))
                }
            }
            _ => Err(SerdeError::InvalidInput {
                message: "unexpected token in document".into(),
            }),
        };
        if result.is_ok() {
            self.leave_container()?;
        }
        result
    }

    fn is_null(&self) -> bool {
        let remaining = self.remaining();
        remaining.len() >= 4
            && &remaining[..4] == b"null"
            && !remaining.get(4).is_some_and(|b| b.is_ascii_alphanumeric())
    }

    fn read_null(&mut self) -> Result<(), SerdeError> {
        self.skip_whitespace();
        if self.is_null() {
            self.advance_by(4);
        }
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        let mut iter = json_token_iter(self.remaining());
        match iter.next()? {
            Ok(Token::StartArray { .. }) => {
                let mut count = 0;
                let mut depth = 1;
                for token in iter {
                    match token {
                        Ok(Token::StartArray { .. }) | Ok(Token::StartObject { .. }) => {
                            if depth == 1 {
                                count += 1;
                            }
                            depth += 1;
                        }
                        Ok(Token::EndArray { .. }) | Ok(Token::EndObject { .. }) => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(count);
                            }
                        }
                        Ok(Token::ValueBool { .. })
                        | Ok(Token::ValueNull { .. })
                        | Ok(Token::ValueString { .. })
                        | Ok(Token::ValueNumber { .. })
                            if depth == 1 =>
                        {
                            count += 1
                        }
                        _ => {}
                    }
                }
                None
            }
            Ok(Token::StartObject { .. }) => {
                let mut count = 0;
                let mut depth = 1;
                for token in iter {
                    match token {
                        Ok(Token::StartArray { .. }) | Ok(Token::StartObject { .. }) => depth += 1,
                        Ok(Token::EndArray { .. }) | Ok(Token::EndObject { .. }) => {
                            depth -= 1;
                            if depth == 0 {
                                return Some(count);
                            }
                        }
                        Ok(Token::ObjectKey { .. }) if depth == 1 => count += 1,
                        _ => {}
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl<'a> JsonDeserializer<'a> {
    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() {
            match self.input[self.position] {
                b' ' | b'\t' | b'\n' | b'\r' => self.position += 1,
                _ => break,
            }
        }
    }

    /// Positions the deserializer at the next element of a container whose closing
    /// delimiter is `close`, enforcing JSON's separator rules: elements are separated by
    /// exactly one comma, and no comma may precede the closing delimiter.
    ///
    /// Returns `true` when the closing delimiter was reached and consumed, `false` when the
    /// next element starts at the current position. `first` is whether no element has been
    /// read from this container yet.
    fn next_element(&mut self, first: bool, close: u8, what: &str) -> Result<bool, SerdeError> {
        self.skip_whitespace();
        match self.remaining().first().copied() {
            Some(b) if b == close => {
                self.advance_by(1);
                Ok(true)
            }
            None => Err(SerdeError::InvalidInput {
                message: format!("unexpected end of input in {what}"),
            }),
            Some(b',') if first => Err(SerdeError::InvalidInput {
                message: format!("unexpected `,` before the first element of {what}"),
            }),
            Some(b',') => {
                self.advance_by(1);
                self.skip_whitespace();
                match self.remaining().first().copied() {
                    Some(b) if b == close => Err(SerdeError::InvalidInput {
                        message: format!("trailing `,` in {what}"),
                    }),
                    Some(b',') => Err(SerdeError::InvalidInput {
                        message: format!("repeated `,` in {what}"),
                    }),
                    None => Err(SerdeError::InvalidInput {
                        message: format!("unexpected end of input in {what}"),
                    }),
                    Some(_) => Ok(false),
                }
            }
            Some(_) if first => Ok(false),
            Some(_) => Err(SerdeError::InvalidInput {
                message: format!("expected `,` between elements of {what}"),
            }),
        }
    }

    fn enter_container(&mut self) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        Ok(())
    }

    /// Leaves a container. When it was the outermost value of the input, nothing but
    /// whitespace may follow it: a body such as `{"a": 1}abc` is malformed JSON and is
    /// rejected here, matching the token-based parser's "found more JSON tokens after
    /// completing parsing" check.
    fn leave_container(&mut self) -> Result<(), SerdeError> {
        self.depth -= 1;
        if self.depth == 0 {
            self.skip_whitespace();
            if !self.remaining().is_empty() {
                return Err(SerdeError::InvalidInput {
                    message: "unexpected trailing characters after the JSON value".into(),
                });
            }
        }
        Ok(())
    }

    /// Skips a JSON number, validating it against the RFC 8259 grammar
    /// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`. Whatever follows is left for the
    /// enclosing container's separator check, so `01` fails there as "expected `,`".
    fn skip_number(&mut self) -> Result<(), SerdeError> {
        let rem = self.remaining();
        let invalid = || SerdeError::InvalidInput {
            message: "invalid number".into(),
        };
        let mut i = 0;
        if rem.get(i) == Some(&b'-') {
            i += 1;
        }
        match rem.get(i) {
            Some(b'0') => i += 1,
            Some(b'1'..=b'9') => {
                while rem.get(i).is_some_and(|b| b.is_ascii_digit()) {
                    i += 1;
                }
            }
            _ => return Err(invalid()),
        }
        if rem.get(i) == Some(&b'.') {
            i += 1;
            let start = i;
            while rem.get(i).is_some_and(|b| b.is_ascii_digit()) {
                i += 1;
            }
            if i == start {
                return Err(invalid());
            }
        }
        if matches!(rem.get(i), Some(b'e') | Some(b'E')) {
            i += 1;
            if matches!(rem.get(i), Some(b'+') | Some(b'-')) {
                i += 1;
            }
            let start = i;
            while rem.get(i).is_some_and(|b| b.is_ascii_digit()) {
                i += 1;
            }
            if i == start {
                return Err(invalid());
            }
        }
        self.advance_by(i);
        Ok(())
    }

    /// Skips a JSON string, accepting exactly the escape sequences `read_string` accepts.
    fn skip_string(&mut self) -> Result<(), SerdeError> {
        if self.settings.enforce_strictness {
            return self.parse_key().map(|_| ());
        }
        let rem = self.remaining();
        debug_assert_eq!(rem.first(), Some(&b'"'));
        let mut i = 1;
        loop {
            match rem.get(i) {
                None => {
                    return Err(SerdeError::InvalidInput {
                        message: "unterminated string".into(),
                    })
                }
                Some(b'"') => {
                    i += 1;
                    break;
                }
                Some(b'\\') => match rem.get(i + 1) {
                    Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => i += 2,
                    Some(b'u')
                        if rem
                            .get(i + 2..i + 6)
                            .is_some_and(|hex| hex.iter().all(u8::is_ascii_hexdigit)) =>
                    {
                        i += 6
                    }
                    _ => {
                        return Err(SerdeError::InvalidInput {
                            message: "invalid escape sequence in string".into(),
                        })
                    }
                },
                Some(_) => i += 1,
            }
        }
        self.advance_by(i);
        Ok(())
    }

    fn consume_number(&mut self) -> Result<(), SerdeError> {
        if self.settings.enforce_strictness {
            self.skip_number()?;
            // Allow whitespace, JSON delimiters, or end of input after the number.
            if self
                .remaining()
                .first()
                .is_some_and(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'))
            {
                return Err(SerdeError::InvalidInput {
                    message: "invalid number boundary".into(),
                });
            }
            return Ok(());
        }
        let mut len = 0;
        for &b in self.remaining() {
            if b.is_ascii_digit() || b == b'-' || b == b'.' || b == b'e' || b == b'E' || b == b'+' {
                len += 1;
            } else {
                break;
            }
        }
        self.advance_by(len);
        Ok(())
    }

    /// Skips one JSON value, validating its syntax as it goes so that an unknown member
    /// cannot smuggle malformed JSON past the deserializer. Like `read_string`, raw control
    /// characters inside strings are rejected only when strictness is enabled.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        self.skip_whitespace();
        match self.remaining().first().copied() {
            Some(b'{') => {
                self.enter_container()?;
                self.advance_by(1);
                let mut first = true;
                loop {
                    if self.next_element(first, b'}', "object")? {
                        break;
                    }
                    first = false;
                    if self.remaining().first() != Some(&b'"') {
                        return Err(SerdeError::InvalidInput {
                            message: "expected object key".into(),
                        });
                    }
                    self.parse_key()?;
                    self.skip_whitespace();
                    if self.remaining().first() != Some(&b':') {
                        return Err(SerdeError::InvalidInput {
                            message: "expected colon after key".into(),
                        });
                    }
                    self.advance_by(1);
                    self.skip_value()?;
                }
                self.depth -= 1;
                Ok(())
            }
            Some(b'[') => {
                self.enter_container()?;
                self.advance_by(1);
                let mut first = true;
                loop {
                    if self.next_element(first, b']', "array")? {
                        break;
                    }
                    first = false;
                    self.skip_value()?;
                }
                self.depth -= 1;
                Ok(())
            }
            Some(b'"') => self.skip_string(),
            Some(b't') => self.skip_literal(b"true"),
            Some(b'f') => self.skip_literal(b"false"),
            Some(b'n') => self.skip_literal(b"null"),
            Some(c) if c == b'-' || c.is_ascii_digit() => self.skip_number(),
            Some(_) => Err(SerdeError::InvalidInput {
                message: "unexpected token in skip_value".into(),
            }),
            None => Err(SerdeError::InvalidInput {
                message: "unexpected end of input".into(),
            }),
        }
    }

    fn skip_literal(&mut self, literal: &'static [u8]) -> Result<(), SerdeError> {
        if !self.remaining().starts_with(literal) {
            return Err(SerdeError::InvalidInput {
                message: format!("expected `{}`", String::from_utf8_lossy(literal)),
            });
        }
        self.advance_by(literal.len());
        Ok(())
    }

    fn read_integer_value(&mut self) -> Result<i64, SerdeError> {
        self.skip_whitespace();
        if self.settings.enforce_strictness || self.settings.allow_integral_float_numbers {
            let start = self.position;
            self.skip_number()?;
            if self
                .remaining()
                .first()
                .is_some_and(|b| !matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b',' | b']' | b'}'))
            {
                return Err(SerdeError::InvalidInput {
                    message: "invalid number boundary".into(),
                });
            }
            let text = std::str::from_utf8(&self.input[start..self.position]).map_err(|e| {
                SerdeError::InvalidInput {
                    message: e.to_string(),
                }
            })?;
            if self.settings.allow_integral_float_numbers && text.contains(['.', 'e', 'E']) {
                return parse_integral_decimal(text);
            }
            return text
                .parse()
                .map_err(|e: std::num::ParseIntError| SerdeError::InvalidInput {
                    message: e.to_string(),
                });
        }
        let rem = self.remaining();
        let mut len = 0;
        for &b in rem {
            if b.is_ascii_digit() || b == b'-' || b == b'+' {
                len += 1;
            } else {
                break;
            }
        }
        if len == 0 {
            return Err(SerdeError::TypeMismatch {
                message: "expected integer".into(),
            });
        }
        let s = std::str::from_utf8(&rem[..len]).map_err(|e| SerdeError::InvalidInput {
            message: e.to_string(),
        })?;
        let n = s.parse::<i64>().map_err(|e| SerdeError::InvalidInput {
            message: e.to_string(),
        })?;
        self.advance_by(len);
        Ok(n)
    }

    fn read_float_value(&mut self) -> Result<f64, SerdeError> {
        self.skip_whitespace();
        let rem = self.remaining();
        // A string may only carry a non-finite value, which JSON cannot express as a number:
        // `"NaN"`, `"Infinity"`, `"-Infinity"`. A quoted finite number such as `"123"` is a
        // type mismatch; the restJson1 protocol tests
        // `RestJsonBodyFloatMalformedValueRejected_case0` and
        // `RestJsonBodyDoubleMalformedValueRejected_case0` require it to be rejected. This
        // mirrors the token-based parser (`expect_number_or_null`), which parses the string
        // with `aws_smithy_types::primitive::Parse` and then rejects finite values, so the
        // spellings Rust's `f64::from_str` accepts for non-finite values (e.g. `"nan"`) are
        // accepted here too.
        if rem.first() == Some(&b'"') {
            let s = self.read_string(&aws_smithy_schema::prelude::STRING)?;
            let value = match s.as_str() {
                "NaN" => f64::NAN,
                "Infinity" => f64::INFINITY,
                "-Infinity" => f64::NEG_INFINITY,
                other => other.parse::<f64>().map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })?,
            };
            if value.is_finite() {
                return Err(SerdeError::TypeMismatch {
                    message: format!(
                        "only `Infinity`, `-Infinity`, `NaN` can represent a float as a string but found `{s}`"
                    ),
                });
            }
            return Ok(value);
        }
        if self.settings.enforce_strictness {
            let start = self.position;
            self.consume_number()?;
            return std::str::from_utf8(&self.input[start..self.position])
                .map_err(|e| SerdeError::InvalidInput {
                    message: e.to_string(),
                })?
                .parse()
                .map_err(|e: std::num::ParseFloatError| SerdeError::InvalidInput {
                    message: e.to_string(),
                });
        }
        let mut len = 0;
        for &b in rem {
            if b.is_ascii_digit() || b == b'-' || b == b'+' || b == b'.' || b == b'e' || b == b'E' {
                len += 1;
            } else {
                break;
            }
        }
        if len == 0 {
            return Err(SerdeError::TypeMismatch {
                message: "expected number".into(),
            });
        }
        let s = std::str::from_utf8(&rem[..len]).map_err(|e| SerdeError::InvalidInput {
            message: e.to_string(),
        })?;
        let n = s.parse::<f64>().map_err(|e| SerdeError::InvalidInput {
            message: e.to_string(),
        })?;
        self.advance_by(len);
        Ok(n)
    }
}

/// Converts a JSON number written with a fraction or exponent (`1.0`, `1e3`,
/// `1.50E1`) to an `i64`, exactly, or fails if it is not a whole number in range.
///
/// The number is treated as `digits × 10^shift`, where `digits` is the mantissa
/// with the `.` removed and `shift = exponent − (digits after the '.')`. For
/// `1.50E1`: digits `150`, shift `1 − 2 = −1`, value `150 × 10^-1 = 15`. The
/// integer keeps the first `kept = significant digits + shift` of them; any it
/// drops must be zeros, and it can have at most the 19 digits an `i64` holds.
///
/// The arithmetic never goes through `f64`: an `f64` has a 53-bit significand,
/// so not every integer above 2^53 is representable. `9007199254740993.0`
/// (2^53 + 1) parses to the nearest `f64`, which is 2^53, and would come out
/// as `9007199254740992` with no error. Accumulating into an `i128` instead
/// cannot overflow for 19 digits, and `i64::try_from` does the range check.
///
/// `text` has already passed `skip_number`, so it is a well-formed JSON number.
/// An exponent that does not fit an `i64` is rejected even when the digits are
/// all zero.
fn parse_integral_decimal(text: &str) -> Result<i64, SerdeError> {
    let invalid = || SerdeError::InvalidInput {
        message: "number is fractional or outside integer range".into(),
    };
    let negative = text.starts_with('-');
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = unsigned.split_once(['e', 'E']).unwrap_or((unsigned, "0"));
    let (int_part, frac_part) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let exponent: i64 = exponent.parse().map_err(|_| invalid())?;

    let all_digits = int_part.bytes().chain(frac_part.bytes());
    let leading_zeros = all_digits.clone().take_while(|b| *b == b'0').count();
    let digits = all_digits.skip(leading_zeros).map(|b| (b - b'0') as i128);
    let significant = (int_part.len() + frac_part.len() - leading_zeros) as i64;
    if significant == 0 {
        return Ok(0);
    }
    let shift = exponent
        .checked_sub(frac_part.len() as i64)
        .ok_or_else(invalid)?;

    // Fewer than 1 kept digit means the value is a fraction; more than 19
    // cannot fit an i64.
    let kept = significant.checked_add(shift).ok_or_else(invalid)?;
    if !(1..=19).contains(&kept) {
        return Err(invalid());
    }
    let take = kept.min(significant) as usize;
    let mut acc: i128 = 0;
    for (i, d) in digits.enumerate() {
        if i < take {
            acc = acc * 10 + d; // at most 19 digits: cannot overflow i128
        } else if d != 0 {
            return Err(invalid()); // a dropped digit was not zero: fractional
        }
    }
    if shift > 0 {
        acc *= 10_i128.pow(shift as u32); // kept <= 19 keeps this below 10^19
    }
    i64::try_from(if negative { -acc } else { acc }).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_schema() -> &'static aws_smithy_schema::Schema {
        &aws_smithy_schema::prelude::STRING
    }

    #[test]
    fn test_read_boolean() {
        let mut deser = JsonDeserializer::new(b"true", Arc::new(JsonCodecSettings::default()));
        assert!(deser.read_boolean(dummy_schema()).unwrap());

        let mut deser = JsonDeserializer::new(b"false", Arc::new(JsonCodecSettings::default()));
        assert!(!(deser.read_boolean(dummy_schema()).unwrap()));
    }

    #[test]
    fn test_read_integer() {
        let mut deser = JsonDeserializer::new(b"42", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.read_integer(dummy_schema()).unwrap(), 42);

        let mut deser = JsonDeserializer::new(b"-123", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.read_integer(dummy_schema()).unwrap(), -123);
    }

    #[test]
    fn test_read_long() {
        let mut deser = JsonDeserializer::new(
            b"9223372036854775807",
            Arc::new(JsonCodecSettings::default()),
        );
        assert_eq!(deser.read_long(dummy_schema()).unwrap(), i64::MAX);
    }

    #[test]
    fn test_read_float() {
        let mut deser = JsonDeserializer::new(b"3.15", Arc::new(JsonCodecSettings::default()));
        assert!((deser.read_float(dummy_schema()).unwrap() - 3.15).abs() < 0.01);
    }

    #[test]
    fn test_read_double() {
        let mut deser = JsonDeserializer::new(b"2.72", Arc::new(JsonCodecSettings::default()));
        assert!((deser.read_double(dummy_schema()).unwrap() - 2.72).abs() < 0.001);
    }

    #[test]
    fn test_read_string() {
        let mut deser =
            JsonDeserializer::new(br#""hello world""#, Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.read_string(dummy_schema()).unwrap(), "hello world");

        let mut deser =
            JsonDeserializer::new(br#""hello\nworld""#, Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.read_string(dummy_schema()).unwrap(), "hello\nworld");
    }

    #[test]
    fn test_is_null() {
        let deser = JsonDeserializer::new(b"null", Arc::new(JsonCodecSettings::default()));
        assert!(deser.is_null());

        let deser = JsonDeserializer::new(b"42", Arc::new(JsonCodecSettings::default()));
        assert!(!deser.is_null());
    }

    #[test]
    fn test_read_byte_range() {
        let mut deser = JsonDeserializer::new(b"127", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.read_byte(dummy_schema()).unwrap(), 127);

        let mut deser = JsonDeserializer::new(b"128", Arc::new(JsonCodecSettings::default()));
        assert!(deser.read_byte(dummy_schema()).is_err());
    }

    #[test]
    fn test_read_struct() {
        use aws_smithy_schema::Schema;

        #[derive(Debug, Default, PartialEq)]
        struct Person {
            first_name: String,
            last_name: String,
            age: i32,
        }

        static FIRST_NAME: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::String,
            "firstName",
            0,
        );
        static LAST_NAME: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::String,
            "lastName",
            1,
        );
        static AGE: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::Integer,
            "age",
            2,
        );
        static PERSON_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::Structure,
            &[&FIRST_NAME, &LAST_NAME, &AGE],
        );

        fn consume_person(
            person: &mut Person,
            schema: &Schema,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            match schema.member_name() {
                Some("firstName") => person.first_name = deser.read_string(schema)?,
                Some("lastName") => person.last_name = deser.read_string(schema)?,
                Some("age") => person.age = deser.read_integer(schema)?,
                _ => {}
            }
            Ok(())
        }

        let json = br#"{"lastName":"Smithy","firstName":"Alice","age":30}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let mut person = Person::default();
        deser
            .read_struct(&PERSON_SCHEMA, &mut |member, d| {
                consume_person(&mut person, member, d)
            })
            .unwrap();
        assert_eq!(
            person,
            Person {
                first_name: "Alice".to_string(),
                last_name: "Smithy".to_string(),
                age: 30
            }
        );

        let json =
            br#"{"firstName":          "Alice","age":12345678,     "lastName":"\"Smithy\""}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let mut person = Person::default();
        deser
            .read_struct(&PERSON_SCHEMA, &mut |member, d| {
                consume_person(&mut person, member, d)
            })
            .unwrap();
        assert_eq!(
            person,
            Person {
                first_name: "Alice".to_string(),
                last_name: "\"Smithy\"".to_string(),
                age: 12345678
            }
        );
    }

    #[test]
    fn test_read_list() {
        let json = b"[1, 2, 3, 4, 5]";
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let capacity = deser.container_size().unwrap_or(0);
        let mut result = Vec::with_capacity(capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_list(dummy_schema(), &mut |deser| {
                result.push(deser.read_integer(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result, vec![1, 2, 3, 4, 5]);
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);

        let json = b"[]";
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let capacity = deser.container_size().unwrap_or(0);
        let mut result = Vec::<i32>::with_capacity(capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_list(dummy_schema(), &mut |deser| {
                result.push(deser.read_integer(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result, Vec::<i32>::new());
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);

        let json = br#"["hello", "world"]"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let capacity = deser.container_size().unwrap_or(0);
        let mut result = Vec::with_capacity(capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_list(dummy_schema(), &mut |deser| {
                result.push(deser.read_string(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result, vec!["hello", "world"]);
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);
    }

    #[test]
    fn test_container_size() {
        let deser =
            JsonDeserializer::new(b"[1, 2, 3, 4, 5]", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.container_size(), Some(5));

        let deser = JsonDeserializer::new(b"[]", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.container_size(), Some(0));

        let deser = JsonDeserializer::new(
            br#"{"a": 1, "b": 2, "c": 3}"#,
            Arc::new(JsonCodecSettings::default()),
        );
        assert_eq!(deser.container_size(), Some(3));

        let deser = JsonDeserializer::new(b"{}", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.container_size(), Some(0));

        let deser = JsonDeserializer::new(
            b"[[1, 2], [3, 4], [5, 6]]",
            Arc::new(JsonCodecSettings::default()),
        );
        assert_eq!(deser.container_size(), Some(3));

        let deser = JsonDeserializer::new(b"42", Arc::new(JsonCodecSettings::default()));
        assert_eq!(deser.container_size(), None);
    }

    #[test]
    fn test_read_map() {
        use std::collections::HashMap;

        let json = br#"{"a": 1, "b": 2, "c": 3}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let calculated_capacity = deser.container_size().unwrap_or(0);
        let mut result = HashMap::with_capacity(calculated_capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_map(dummy_schema(), &mut |key, deser| {
                result.insert(key, deser.read_integer(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result.get("a"), Some(&1));
        assert_eq!(result.get("b"), Some(&2));
        assert_eq!(result.get("c"), Some(&3));
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);

        let json = b"{}";
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let calculated_capacity = deser.container_size().unwrap_or(0);
        let mut result = HashMap::<String, i32>::with_capacity(calculated_capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_map(dummy_schema(), &mut |key, deser| {
                result.insert(key, deser.read_integer(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result, HashMap::<String, i32>::new());
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);

        let json = br#"{"name": "Alice", "city": "Seattle"}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let calculated_capacity = deser.container_size().unwrap_or(0);
        let mut result = HashMap::with_capacity(calculated_capacity);
        let allocated_capacity = result.capacity();
        deser
            .read_map(dummy_schema(), &mut |key, deser| {
                result.insert(key, deser.read_string(dummy_schema())?);
                Ok(())
            })
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result.get("name"), Some(&"Alice".to_string()));
        assert_eq!(result.get("city"), Some(&"Seattle".to_string()));
        // Ensure no more memory was allocated for the container
        assert_eq!(result.capacity(), allocated_capacity);
    }

    #[test]
    fn test_nested_complex_deserialization() {
        use aws_smithy_schema::Schema;
        use std::collections::HashMap;

        #[derive(Debug, Default, PartialEq)]
        struct Address {
            street: String,
            city: String,
            zip: i32,
        }
        #[derive(Debug, Default, PartialEq)]
        struct Company {
            name: String,
            employees: Vec<String>,
            metadata: HashMap<String, i32>,
            active: bool,
        }
        #[derive(Debug, Default, PartialEq)]
        struct User {
            id: i64,
            name: String,
            scores: Vec<f64>,
            address: Address,
            companies: Vec<Company>,
            tags: HashMap<String, String>,
        }

        // Address members & schema
        static ADDR_STREET: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "street",
            0,
        );
        static ADDR_CITY: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "city",
            1,
        );
        static ADDR_ZIP: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::Integer,
            "zip",
            2,
        );
        static ADDRESS_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::Structure,
            &[&ADDR_STREET, &ADDR_CITY, &ADDR_ZIP],
        );

        // Company members & schema
        static COMP_NAME: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::String,
            "name",
            0,
        );
        static COMP_EMPLOYEES: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::List,
            "employees",
            1,
        );
        static COMP_METADATA: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Map,
            "metadata",
            2,
        );
        static COMP_ACTIVE: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Boolean,
            "active",
            3,
        );
        static COMPANY_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Structure,
            &[&COMP_NAME, &COMP_EMPLOYEES, &COMP_METADATA, &COMP_ACTIVE],
        );

        // User members & schema
        static USER_ID: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Long,
            "id",
            0,
        );
        static USER_NAME: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::String,
            "name",
            1,
        );
        static USER_SCORES: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "scores",
            2,
        );
        static USER_ADDRESS: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Structure,
            "address",
            3,
        );
        static USER_COMPANIES: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "companies",
            4,
        );
        static USER_TAGS: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Map,
            "tags",
            5,
        );
        static USER_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Structure,
            &[
                &USER_ID,
                &USER_NAME,
                &USER_SCORES,
                &USER_ADDRESS,
                &USER_COMPANIES,
                &USER_TAGS,
            ],
        );

        fn consume_address(
            addr: &mut Address,
            schema: &Schema,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            match schema.member_name() {
                Some("street") => addr.street = deser.read_string(schema)?,
                Some("city") => addr.city = deser.read_string(schema)?,
                Some("zip") => addr.zip = deser.read_integer(schema)?,
                _ => {}
            }
            Ok(())
        }

        fn consume_company(
            comp: &mut Company,
            schema: &Schema,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            match schema.member_name() {
                Some("name") => comp.name = deser.read_string(schema)?,
                Some("active") => comp.active = deser.read_boolean(schema)?,
                Some("employees") => {
                    let mut v = Vec::new();
                    deser.read_list(schema, &mut |d| {
                        v.push(d.read_string(dummy_schema())?);
                        Ok(())
                    })?;
                    comp.employees = v;
                }
                Some("metadata") => {
                    let mut m = HashMap::new();
                    deser.read_map(schema, &mut |k, d| {
                        m.insert(k, d.read_integer(dummy_schema())?);
                        Ok(())
                    })?;
                    comp.metadata = m;
                }
                _ => {}
            }
            Ok(())
        }

        fn consume_user(
            user: &mut User,
            schema: &Schema,
            deser: &mut dyn ShapeDeserializer,
        ) -> Result<(), SerdeError> {
            match schema.member_name() {
                Some("id") => user.id = deser.read_long(schema)?,
                Some("name") => user.name = deser.read_string(schema)?,
                Some("scores") => {
                    let mut v = Vec::new();
                    deser.read_list(schema, &mut |d| {
                        v.push(d.read_double(dummy_schema())?);
                        Ok(())
                    })?;
                    user.scores = v;
                }
                Some("address") => {
                    let mut addr = Address::default();
                    deser.read_struct(&ADDRESS_SCHEMA, &mut |member, d| {
                        consume_address(&mut addr, member, d)
                    })?;
                    user.address = addr;
                }
                Some("companies") => {
                    let mut v = Vec::new();
                    deser.read_list(schema, &mut |d| {
                        let mut comp = Company::default();
                        d.read_struct(&COMPANY_SCHEMA, &mut |member, d| {
                            consume_company(&mut comp, member, d)
                        })?;
                        v.push(comp);
                        Ok(())
                    })?;
                    user.companies = v;
                }
                Some("tags") => {
                    let mut m = HashMap::new();
                    deser.read_map(schema, &mut |k, d| {
                        m.insert(k, d.read_string(dummy_schema())?);
                        Ok(())
                    })?;
                    user.tags = m;
                }
                _ => {}
            }
            Ok(())
        }

        let json = br#"{
            "id": 12345,
            "name": "John Doe",
            "scores": [95.5, 87.3, 92.1],
            "address": {
                "street": "123 Main St",
                "city": "Seattle",
                "zip": 98101
            },
            "companies": [
                {
                    "name": "TechCorp",
                    "employees": ["Alice", "Bob"],
                    "metadata": {"founded": 2010, "size": 500},
                    "active": true
                },
                {
                    "name": "StartupInc",
                    "employees": ["Charlie"],
                    "metadata": {"founded": 2020},
                    "active": false
                }
            ],
            "tags": {"role": "admin", "level": "senior"}
        }"#;

        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let mut user = User::default();
        deser
            .read_struct(&USER_SCHEMA, &mut |member, d| {
                consume_user(&mut user, member, d)
            })
            .unwrap();

        assert_eq!(user.id, 12345);
        assert_eq!(user.name, "John Doe");
        assert_eq!(user.scores, vec![95.5, 87.3, 92.1]);
        assert_eq!(user.address.street, "123 Main St");
        assert_eq!(user.address.city, "Seattle");
        assert_eq!(user.address.zip, 98101);
        assert_eq!(user.companies.len(), 2);
        assert_eq!(user.companies[0].name, "TechCorp");
        assert_eq!(user.companies[0].employees, vec!["Alice", "Bob"]);
        assert_eq!(user.companies[0].metadata.get("founded"), Some(&2010));
        assert_eq!(user.companies[0].metadata.get("size"), Some(&500));
        assert!(user.companies[0].active);
        assert_eq!(user.companies[1].name, "StartupInc");
        assert_eq!(user.companies[1].employees, vec!["Charlie"]);
        assert_eq!(user.companies[1].metadata.get("founded"), Some(&2020));
        assert!(!user.companies[1].active);
        assert_eq!(user.tags.get("role"), Some(&"admin".to_string()));
        assert_eq!(user.tags.get("level"), Some(&"senior".to_string()));
    }

    #[test]
    fn test_json_name_deserialization() {
        use aws_smithy_schema::Schema;

        static FOO_MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::String,
            "foo",
            0,
        );
        // "bar" member has @jsonName("Baz")
        static BAR_MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Integer,
            "bar",
            1,
        )
        .with_json_name("Baz");
        static STRUCT_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Structure,
            &[&FOO_MEMBER, &BAR_MEMBER],
        );

        let json = br#"{"foo":"hello","Baz":42}"#;

        // With use_json_name=true, "Baz" resolves to the "bar" member
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let (mut foo, mut bar) = (None::<String>, None::<i32>);
        deser
            .read_struct(&STRUCT_SCHEMA, &mut |member, d| {
                match member.member_name() {
                    Some("foo") => foo = Some(d.read_string(member)?),
                    Some("bar") => bar = Some(d.read_integer(member)?),
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(foo.as_deref(), Some("hello"));
        assert_eq!(bar, Some(42));

        // With use_json_name=false, "Baz" is unknown and gets skipped
        let mut deser = JsonDeserializer::new(
            json,
            Arc::new(JsonCodecSettings::builder().use_json_name(false).build()),
        );
        let (mut foo, mut bar) = (None::<String>, None::<i32>);
        deser
            .read_struct(&STRUCT_SCHEMA, &mut |member, d| {
                match member.member_name() {
                    Some("foo") => foo = Some(d.read_string(member)?),
                    Some("bar") => bar = Some(d.read_integer(member)?),
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(foo.as_deref(), Some("hello"));
        assert_eq!(bar, None); // "Baz" not recognized without jsonName
    }

    fn timestamp_schema() -> &'static aws_smithy_schema::Schema {
        &aws_smithy_schema::prelude::TIMESTAMP
    }

    #[test]
    fn test_read_timestamp_positive_integer() {
        let mut deser =
            JsonDeserializer::new(b"1700000000", Arc::new(JsonCodecSettings::default()));
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs(1700000000));
    }

    #[test]
    fn test_read_timestamp_negative_integer() {
        let mut deser = JsonDeserializer::new(b"-1000", Arc::new(JsonCodecSettings::default()));
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs(-1000));
    }

    #[test]
    fn test_read_timestamp_float() {
        // This is the format DynamoDB uses: epoch seconds with fractional part
        let mut deser =
            JsonDeserializer::new(b"1.615218678973E9", Arc::new(JsonCodecSettings::default()));
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs_f64(1.615218678973E9));
    }

    #[test]
    fn test_read_timestamp_float_simple() {
        let mut deser =
            JsonDeserializer::new(b"1700000000.5", Arc::new(JsonCodecSettings::default()));
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs_f64(1700000000.5));
    }

    #[test]
    fn test_read_timestamp_string_datetime() {
        let mut deser = JsonDeserializer::new(
            br#""2023-11-14T22:13:20Z""#,
            Arc::new(JsonCodecSettings::default()),
        );
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs(1700000000));
    }

    #[test]
    fn test_read_timestamp_invalid() {
        let mut deser = JsonDeserializer::new(b"true", Arc::new(JsonCodecSettings::default()));
        assert!(deser.read_timestamp(timestamp_schema()).is_err());
    }

    #[test]
    fn test_skip_value_empty_array() {
        // Regression: skip_value failed on [] because json_token_iter can't parse ']' as a value start
        use aws_smithy_schema::ShapeType;
        static KNOWN_MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::String,
            "known",
            0,
        );
        static MEMBERS: &[&Schema] = &[&KNOWN_MEMBER];
        static TEST_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::Structure,
            MEMBERS,
        );

        let json = br#"{"known":"yes","Items":[],"extra":true}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let mut known_val = String::new();
        deser
            .read_struct(&TEST_SCHEMA, &mut |member, deser| {
                if member.member_name() == Some("known") {
                    known_val = deser.read_string(dummy_schema())?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(known_val, "yes");
    }

    #[test]
    fn test_skip_value_nested_objects() {
        use aws_smithy_schema::ShapeType;
        static D_MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::String,
            "d",
            0,
        );
        static MEMBERS: &[&Schema] = &[&D_MEMBER];
        static TEST_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::Structure,
            MEMBERS,
        );

        let json = br#"{"a":{"b":[1,2,{"c":3}]},"d":"ok"}"#;
        let mut deser = JsonDeserializer::new(json, Arc::new(JsonCodecSettings::default()));
        let mut d_val = String::new();
        deser
            .read_struct(&TEST_SCHEMA, &mut |member, deser| {
                if member.member_name() == Some("d") {
                    d_val = deser.read_string(dummy_schema())?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(d_val, "ok");
    }

    // Regression tests for bugs discovered by fuzzing (see PR #4608).
    // These exercise the same pathological inputs outside of the fuzz
    // harness so the fixes stay protected even if the fuzz targets are
    // removed or regress.

    #[test]
    fn regression_truncated_list_does_not_infinite_loop() {
        // Bug 1: `read_list` used to call the consumer on empty input
        // (because `first()` returned `None`, not `Some(&b']')`) and
        // loop forever because the position never advanced.
        let mut deser = JsonDeserializer::new(b"[", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_list(dummy_schema(), &mut |d| {
                d.read_integer(dummy_schema()).map(|_| ())
            })
            .expect_err("truncated list input must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_truncated_string_list_does_not_infinite_loop() {
        // Same bug class as above, but for the specialized
        // `read_string_list` helper.
        let mut deser = JsonDeserializer::new(b"[", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_string_list(dummy_schema())
            .expect_err("truncated string list input must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_truncated_document_array_does_not_infinite_loop() {
        // Array branch of `read_document` had the same loop bug.
        let mut deser = JsonDeserializer::new(b"[", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_document(dummy_schema())
            .expect_err("truncated document array must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_unterminated_object_key_does_not_panic() {
        // Bug 2: `parse_key` used to advance past the end of the input
        // when the closing quote was missing, which made the next call
        // to `remaining()` panic with an out-of-range slice index.
        use aws_smithy_schema::Schema;

        static MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "m",
            0,
        );
        static SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER],
        );

        let mut deser = JsonDeserializer::new(br#"{""#, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_struct(&SCHEMA, &mut |_, _| Ok(()))
            .expect_err("unterminated object key must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_skip_value_truncated_true_does_not_panic() {
        // Bug 3: `skip_value` used to blindly `advance_by(4)` on
        // `Some(b't')`, running off the end of the buffer when the
        // input was shorter than `true`.
        use aws_smithy_schema::Schema;

        static MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "known",
            0,
        );
        static SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER],
        );

        // `"":t"` — unknown key `""` whose value starts with `t` but
        // isn't the literal `true`. This drives `skip_value` into the
        // `Some(b't')` arm with fewer than 4 bytes remaining.
        let input = br#"{"":t"#;
        let mut deser = JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_struct(&SCHEMA, &mut |_, _| Ok(()))
            .expect_err("truncated `true` literal must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_skip_value_rejects_malformed_true_literal() {
        // Reviewer follow-up on Bug 3: a length check alone is not
        // enough — input like `t!!!` has four bytes but isn't `true`.
        // `skip_value` must validate the literal content, not just
        // that there are enough bytes to advance past.
        use aws_smithy_schema::Schema;

        static MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "known",
            0,
        );
        static SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER],
        );

        for bad in [
            &br#"{"":t!!!}"#[..],
            &br#"{"":f!!!!}"#[..],
            &br#"{"":n!!!}"#[..],
        ] {
            let mut deser = JsonDeserializer::new(bad, Arc::new(JsonCodecSettings::default()));
            let result = deser.read_struct(&SCHEMA, &mut |_, _| Ok(()));
            assert!(
                matches!(result, Err(SerdeError::InvalidInput { .. })),
                "expected InvalidInput for malformed input {:?}, got {:?}",
                std::str::from_utf8(bad).unwrap_or("<non-utf8>"),
                result
            );
        }
    }

    #[test]
    fn regression_truncated_struct_does_not_infinite_loop() {
        // Reviewer follow-up: `read_struct` now uses an explicit `None`
        // arm inside the loop, so a truncated `{` input is rejected as
        // InvalidInput instead of relying on a downstream check to
        // catch it. This mirrors the fix for `read_list`.
        use aws_smithy_schema::Schema;

        static MEMBER: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "m",
            0,
        );
        static SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER],
        );

        let mut deser = JsonDeserializer::new(b"{", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_struct(&SCHEMA, &mut |_, _| Ok(()))
            .expect_err("truncated struct input must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn regression_truncated_map_does_not_infinite_loop() {
        // Same as above but for `read_map`.
        let mut deser = JsonDeserializer::new(b"{", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_map(dummy_schema(), &mut |_, _| Ok(()))
            .expect_err("truncated map input must be rejected");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    // ---- Recursion-depth guard tests ----
    //
    // These verify that deeply-nested payloads produce a clean `SerdeError`
    // instead of a stack overflow. They exercise the same recursion pattern
    // an attacker would: a JSON object/list/map/document nested far past the
    // `MAX_DESERIALIZE_DEPTH` limit.

    /// Builds `open.repeat(n) + close.repeat(n)`. For lists this is valid JSON
    /// (nested empty arrays). For objects this is malformed, but the depth
    /// guard fires before the malformed region is reached so reject-tests work
    /// either way.
    fn build_nested(open: &str, close: &str, n: usize) -> Vec<u8> {
        build_nested_with_inner(open, "", close, n)
    }

    /// Builds `open.repeat(n) + inner + close.repeat(n)` — always valid JSON
    /// given a valid `inner`. Used for accept-under-limit tests.
    fn build_nested_with_inner(open: &str, inner: &str, close: &str, n: usize) -> Vec<u8> {
        let mut out = String::with_capacity(open.len() * n + inner.len() + close.len() * n);
        for _ in 0..n {
            out.push_str(open);
        }
        out.push_str(inner);
        for _ in 0..n {
            out.push_str(close);
        }
        out.into_bytes()
    }

    /// A self-referential struct schema: member `"a"` points back to the parent
    /// schema. This is the minimum schema needed to exercise the depth guard
    /// through `read_struct`'s consumer-callback path (without it, unknown
    /// members go through `skip_value` which is iterative).
    fn recursive_struct_schema() -> &'static Schema {
        static MEMBER_A: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Rec"),
            aws_smithy_schema::ShapeType::Structure,
            "a",
            0,
        );
        static RECURSIVE: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Rec"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER_A],
        );
        &RECURSIVE
    }

    /// Consumer that re-enters `read_struct` to exercise the depth guard.
    fn recursive_struct_consumer(
        _member: &Schema,
        deser: &mut dyn ShapeDeserializer,
    ) -> Result<(), SerdeError> {
        deser.read_struct(recursive_struct_schema(), &mut recursive_struct_consumer)
    }

    /// Consumer that re-enters `read_list` to exercise the depth guard.
    fn recursive_list_consumer(deser: &mut dyn ShapeDeserializer) -> Result<(), SerdeError> {
        deser.read_list(dummy_schema(), &mut recursive_list_consumer)
    }

    /// Consumer that re-enters `read_map` to exercise the depth guard.
    fn recursive_map_consumer(
        _key: String,
        deser: &mut dyn ShapeDeserializer,
    ) -> Result<(), SerdeError> {
        deser.read_map(dummy_schema(), &mut recursive_map_consumer)
    }

    fn assert_depth_error(err: SerdeError) {
        match err {
            SerdeError::Custom { ref message } => {
                assert!(
                    message.contains("maximum nesting depth exceeded"),
                    "wrong error message: {message}"
                );
            }
            other => panic!("expected Custom depth error, got {other:?}"),
        }
    }

    #[test]
    fn depth_limit_read_struct_rejects_deeply_nested() {
        // 200 levels — well past the 128 limit. The depth guard fires at
        // level 129 before we reach the ending, so the payload doesn't need
        // to be valid JSON all the way through.
        let payload = build_nested(r#"{"a":"#, "}", 200);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_struct(recursive_struct_schema(), &mut recursive_struct_consumer)
            .expect_err("deeply nested struct must be rejected");
        assert_depth_error(err);
    }

    #[test]
    fn depth_limit_read_struct_accepts_under_limit() {
        // 100 levels of `{"a":` wrapped around an empty `{}` → 101 actual
        // `read_struct` calls, which is under the 128 limit.
        let payload = build_nested_with_inner(r#"{"a":"#, "{}", "}", 100);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        deser
            .read_struct(recursive_struct_schema(), &mut recursive_struct_consumer)
            .expect("100-level nesting should succeed");
    }

    #[test]
    fn depth_limit_read_list_rejects_deeply_nested() {
        let payload = build_nested("[", "]", 200);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_list(dummy_schema(), &mut recursive_list_consumer)
            .expect_err("deeply nested list must be rejected");
        assert_depth_error(err);
    }

    #[test]
    fn depth_limit_read_list_accepts_under_limit() {
        // `[[[...[]]]]` — 100 nested empty lists is valid JSON on its own.
        let payload = build_nested("[", "]", 100);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        deser
            .read_list(dummy_schema(), &mut recursive_list_consumer)
            .expect("100-level nesting should succeed");
    }

    #[test]
    fn depth_limit_read_map_rejects_deeply_nested() {
        let payload = build_nested(r#"{"k":"#, "}", 200);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_map(dummy_schema(), &mut recursive_map_consumer)
            .expect_err("deeply nested map must be rejected");
        assert_depth_error(err);
    }

    #[test]
    fn depth_limit_read_map_accepts_under_limit() {
        let payload = build_nested_with_inner(r#"{"k":"#, "{}", "}", 100);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        deser
            .read_map(dummy_schema(), &mut recursive_map_consumer)
            .expect("100-level nesting should succeed");
    }

    #[test]
    fn depth_limit_read_document_rejects_deeply_nested_object() {
        // `read_document` is directly self-recursive on `{` and `[` branches;
        // no external consumer is needed.
        let payload = build_nested(r#"{"k":"#, "}", 200);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_document(dummy_schema())
            .expect_err("deeply nested document must be rejected");
        assert_depth_error(err);
    }

    #[test]
    fn depth_limit_read_document_rejects_deeply_nested_array() {
        let payload = build_nested("[", "]", 200);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_document(dummy_schema())
            .expect_err("deeply nested document array must be rejected");
        assert_depth_error(err);
    }

    #[test]
    fn depth_limit_read_document_accepts_under_limit() {
        let payload = build_nested_with_inner(r#"{"k":"#, "{}", "}", 100);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(JsonCodecSettings::default()));
        deser
            .read_document(dummy_schema())
            .expect("100-level nesting should succeed");
    }

    #[test]
    fn depth_limit_respects_custom_max_depth_setting() {
        // A custom, tighter limit trips before the default 128 would.
        let settings = JsonCodecSettings::builder().max_depth(16).build();

        // 20 levels exceeds the custom limit of 16.
        let payload = build_nested("[", "]", 20);
        let mut deser = JsonDeserializer::new(&payload, Arc::new(settings));
        let err = deser
            .read_list(dummy_schema(), &mut recursive_list_consumer)
            .expect_err("20-level nesting must exceed custom limit of 16");
        assert_depth_error(err);

        // 10 levels fits inside the custom limit.
        let settings_ok = JsonCodecSettings::builder().max_depth(16).build();
        let payload_ok = build_nested("[", "]", 10);
        let mut deser_ok = JsonDeserializer::new(&payload_ok, Arc::new(settings_ok));
        deser_ok
            .read_list(dummy_schema(), &mut recursive_list_consumer)
            .expect("10-level nesting should succeed under custom limit");
    }

    mod separators {
        //! JSON syntax the deserializer must reject: separators (exactly one `,` between
        //! elements, none before a closing delimiter), bytes after the top-level value, and
        //! malformed values inside skipped unknown members. Smithy protocol tests
        //! `RestJsonInvalidJsonBody_case7` (`{"int": 10,}`) and `RestJsonInvalidJsonBody_case1`
        //! (`{ "int": 10 }abc`) require the first two.
        use super::*;
        use aws_smithy_schema::{shape_id, ShapeType};

        static INT: Schema = Schema::new_member(
            shape_id!("test", "Input", "int"),
            ShapeType::Integer,
            "int",
            0,
        );
        static LIST: Schema = Schema::new_member(
            shape_id!("test", "Input", "list"),
            ShapeType::List,
            "list",
            1,
        );
        static INPUT: Schema = Schema::new_struct(
            shape_id!("test", "Input"),
            ShapeType::Structure,
            &[&INT, &LIST],
        );

        fn deser(input: &[u8]) -> JsonDeserializer<'_> {
            JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()))
        }

        /// Reads `INPUT` and returns `(int, list)`; unknown members are skipped.
        fn read_input(body: &[u8]) -> Result<(Option<i32>, Vec<i32>), SerdeError> {
            let mut d = deser(body);
            let mut int = None;
            let mut list = Vec::new();
            d.read_struct(&INPUT, &mut |member, d| {
                match member.member_index() {
                    Some(0) => int = Some(d.read_integer(member)?),
                    Some(1) => d.read_list(member, &mut |d| {
                        list.push(d.read_integer(member)?);
                        Ok(())
                    })?,
                    _ => {}
                }
                Ok(())
            })?;
            Ok((int, list))
        }

        #[test]
        fn well_formed_input_is_read() {
            assert_eq!(
                read_input(br#"{ "int": 10 , "list": [ 1 , 2 ] }"#).unwrap(),
                (Some(10), vec![1, 2])
            );
            assert_eq!(read_input(br#"{}"#).unwrap(), (None, vec![]));
            assert_eq!(read_input(br#"{"list":[]}"#).unwrap(), (None, vec![]));
            assert_eq!(
                read_input(b"  { \"int\": 1 }  \n").unwrap(),
                (Some(1), vec![])
            );
        }

        #[test]
        fn trailing_comma_in_object_is_rejected() {
            assert!(read_input(br#"{"int": 10,}"#).is_err());
        }

        #[test]
        fn leading_and_repeated_commas_in_object_are_rejected() {
            assert!(read_input(br#"{,"int": 10}"#).is_err());
            assert!(read_input(br#"{"int": 10,,"list": []}"#).is_err());
            assert!(read_input(br#"{"int": 10,,}"#).is_err());
        }

        #[test]
        fn missing_comma_in_object_is_rejected() {
            assert!(read_input(br#"{"int": 10 "list": []}"#).is_err());
        }

        #[test]
        fn commas_in_array_are_checked() {
            assert!(read_input(br#"{"list": [1,]}"#).is_err());
            assert!(read_input(br#"{"list": [,1]}"#).is_err());
            assert!(read_input(br#"{"list": [1,,2]}"#).is_err());
            assert!(read_input(br#"{"list": [1 2]}"#).is_err());
        }

        #[test]
        fn skipped_unknown_members_are_still_validated() {
            assert_eq!(
                read_input(br#"{"unknown": {"a": [1, {"b": null}], "c": "x"}, "int": 1}"#).unwrap(),
                (Some(1), vec![])
            );
            assert!(read_input(br#"{"unknown": [1,,2], "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": {"a": 1,}, "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": "unterminated, "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": tru, "int": 1}"#).is_err());
        }

        #[test]
        fn trailing_characters_after_the_top_level_value_are_rejected() {
            assert!(read_input(br#"{ "int": 10 }abc"#).is_err());
            assert!(read_input(br#"{ "int": 10 },"#).is_err());
            assert!(read_input(br#"{ "int": 10 }{}"#).is_err());
            assert!(deser(br#"[1]x"#).read_integer_list(&LIST).is_err());
            assert!(deser(br#"{"k":"v"} {}"#)
                .read_string_string_map(&LIST)
                .is_err());
            assert!(deser(br#"{"a":1}]"#).read_document(&INT).is_err());
            assert!(deser(br#"[1,2]"#).read_document(&INT).is_ok());
        }

        #[test]
        fn skipped_numbers_must_match_the_json_grammar() {
            for ok in [
                b"0" as &[u8],
                b"-0",
                b"10",
                b"-1.5",
                b"1e3",
                b"1E+3",
                b"2.5e-7",
            ] {
                let body = [b"{\"unknown\": " as &[u8], ok, b", \"int\": 1}"].concat();
                assert_eq!(
                    read_input(&body).unwrap(),
                    (Some(1), vec![]),
                    "{}",
                    String::from_utf8_lossy(ok)
                );
            }
            for bad in [
                b"01" as &[u8],
                b"1.",
                b"--1",
                b"+1",
                b".5",
                b"1e",
                b"1e+",
                b"-",
                b"0x1",
            ] {
                let body = [b"{\"unknown\": " as &[u8], bad, b", \"int\": 1}"].concat();
                assert!(
                    read_input(&body).is_err(),
                    "{}",
                    String::from_utf8_lossy(bad)
                );
            }
        }

        #[test]
        fn skipped_strings_must_use_valid_escapes() {
            assert_eq!(
                read_input(br#"{"unknown": "a\"b\\c\/\b\f\n\r\t\u00e9", "int": 1}"#).unwrap(),
                (Some(1), vec![])
            );
            assert!(read_input(br#"{"unknown": "\q", "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": "\u12", "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": "\uZZZZ", "int": 1}"#).is_err());
            assert!(read_input(br#"{"unknown": "abc\", "int": 1}"#).is_err());
        }

        #[test]
        fn fast_path_collections_check_separators() {
            assert_eq!(
                deser(br#"["a","b"]"#).read_string_list(&LIST).unwrap(),
                vec!["a", "b"]
            );
            assert!(deser(br#"["a",]"#).read_string_list(&LIST).is_err());
            assert!(deser(br#"[,"a"]"#).read_string_list(&LIST).is_err());
            assert!(deser(br#"[1,,2]"#).read_integer_list(&LIST).is_err());
            assert!(deser(br#"[1,]"#).read_long_list(&LIST).is_err());
            assert!(deser(br#"["YQ==",]"#).read_blob_list(&LIST).is_err());
            assert_eq!(
                deser(br#"{"k":"v"}"#)
                    .read_string_string_map(&LIST)
                    .unwrap()
                    .len(),
                1
            );
            assert!(deser(br#"{"k":"v",}"#)
                .read_string_string_map(&LIST)
                .is_err());
            assert!(deser(br#"{"k":"v" "j":"w"}"#)
                .read_string_string_map(&LIST)
                .is_err());
        }

        #[test]
        fn generic_map_checks_separators() {
            let mut count = 0;
            assert!(deser(br#"{"a":1,"b":2}"#)
                .read_map(&LIST, &mut |_, d| {
                    count += 1;
                    d.read_integer(&INT).map(|_| ())
                })
                .is_ok());
            assert_eq!(count, 2);
            assert!(deser(br#"{"a":1,}"#)
                .read_map(&LIST, &mut |_, d| d.read_integer(&INT).map(|_| ()))
                .is_err());
        }

        #[test]
        fn documents_check_separators() {
            assert!(deser(br#"{"a":[1,2],"b":{"c":true}}"#)
                .read_document(&INT)
                .is_ok());
            assert!(deser(br#"{"a":1,}"#).read_document(&INT).is_err());
            assert!(deser(br#"[1,,2]"#).read_document(&INT).is_err());
            assert!(deser(br#"{,"a":1}"#).read_document(&INT).is_err());
        }
    }

    mod float_strings {
        //! A float or double member may carry a string only for a non-finite value
        //! (`"NaN"`, `"Infinity"`, `"-Infinity"`). Smithy protocol tests
        //! `RestJsonBodyFloatMalformedValueRejected_case0` and
        //! `RestJsonBodyDoubleMalformedValueRejected_case0` (`"123"`) require quoted finite
        //! numbers to be rejected.
        use super::*;
        use aws_smithy_schema::{shape_id, ShapeType};

        static F: Schema =
            Schema::new_member(shape_id!("test", "Input", "f"), ShapeType::Float, "f", 0);
        static D: Schema =
            Schema::new_member(shape_id!("test", "Input", "d"), ShapeType::Double, "d", 1);
        static INPUT: Schema =
            Schema::new_struct(shape_id!("test", "Input"), ShapeType::Structure, &[&F, &D]);

        fn read(body: &[u8]) -> Result<(Option<f32>, Option<f64>), SerdeError> {
            let mut deser = JsonDeserializer::new(body, Arc::new(JsonCodecSettings::default()));
            let (mut f, mut d) = (None, None);
            deser.read_struct(&INPUT, &mut |m, x| {
                match m.member_index() {
                    Some(0) => f = Some(x.read_float(m)?),
                    Some(1) => d = Some(x.read_double(m)?),
                    _ => {}
                }
                Ok(())
            })?;
            Ok((f, d))
        }

        #[test]
        fn numbers_and_the_three_special_strings_are_read() {
            assert_eq!(
                read(br#"{"f": 1.5, "d": -2}"#).unwrap(),
                (Some(1.5), Some(-2.0))
            );
            let (f, d) = read(br#"{"f": "NaN", "d": "-Infinity"}"#).unwrap();
            assert!(f.unwrap().is_nan());
            assert_eq!(d, Some(f64::NEG_INFINITY));
            assert_eq!(
                read(br#"{"f": "Infinity"}"#).unwrap().0,
                Some(f32::INFINITY)
            );
        }

        #[test]
        fn quoted_finite_numbers_are_rejected() {
            assert!(read(br#"{"f": "123"}"#).is_err());
            assert!(read(br#"{"d": "123"}"#).is_err());
            assert!(read(br#"{"d": "1.5e3"}"#).is_err());
            assert!(read(br#"{"d": "-0"}"#).is_err());
        }

        #[test]
        fn non_numeric_strings_are_rejected() {
            assert!(read(br#"{"d": "abc"}"#).is_err());
            assert!(read(br#"{"d": ""}"#).is_err());
            assert!(read(br#"{"d": "1 2"}"#).is_err());
        }

        #[test]
        fn alternative_non_finite_spellings_match_the_legacy_parser() {
            // `aws_smithy_types::primitive::Parse` falls back to `f64::from_str`, which
            // accepts these case-insensitively; only finite results are rejected.
            assert!(read(br#"{"f": "nan"}"#).unwrap().0.unwrap().is_nan());
            assert_eq!(read(br#"{"d": "inf"}"#).unwrap().1, Some(f64::INFINITY));
            assert_eq!(
                read(br#"{"d": "-infinity"}"#).unwrap().1,
                Some(f64::NEG_INFINITY)
            );
        }
    }

    mod timestamp_formats {
        //! With `strict_timestamp_formats`, a timestamp must use exactly the wire form its
        //! format prescribes, as the restJson1 `MalformedTimestampBody*` protocol tests require.
        //! Without it the lenient client behavior is unchanged.
        use super::*;
        use aws_smithy_schema::traits::TimestampFormat;
        use aws_smithy_schema::{shape_id, ShapeType};

        static DEFAULT: Schema =
            Schema::new_member(shape_id!("test", "T", "d"), ShapeType::Timestamp, "d", 0);
        static DATE_TIME: Schema =
            Schema::new_member(shape_id!("test", "T", "dt"), ShapeType::Timestamp, "dt", 1)
                .with_timestamp_format(TimestampFormat::DateTime);
        static HTTP_DATE: Schema =
            Schema::new_member(shape_id!("test", "T", "hd"), ShapeType::Timestamp, "hd", 2)
                .with_timestamp_format(TimestampFormat::HttpDate);
        static EPOCH: Schema =
            Schema::new_member(shape_id!("test", "T", "es"), ShapeType::Timestamp, "es", 3)
                .with_timestamp_format(TimestampFormat::EpochSeconds);

        /// 2018-01-09T20:51:21Z
        const T: i64 = 1515531081;

        fn read(strict: bool, schema: &Schema, input: &[u8]) -> Result<DateTime, SerdeError> {
            let settings = JsonCodecSettings::builder()
                .strict_timestamp_formats(strict)
                .build();
            JsonDeserializer::new(input, Arc::new(settings)).read_timestamp(schema)
        }

        fn assert_rejected(schema: &Schema, inputs: &[&[u8]]) {
            for input in inputs {
                assert!(
                    read(true, schema, input).is_err(),
                    "{} must be rejected",
                    String::from_utf8_lossy(input)
                );
            }
        }

        #[test]
        fn strict_epoch_seconds_default_requires_a_number() {
            assert_eq!(
                read(true, &DEFAULT, b"1515531081").unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(true, &DEFAULT, b"1515531081.1234").unwrap(),
                DateTime::from_secs_f64(1515531081.1234)
            );
            assert_rejected(
                &DEFAULT,
                &[
                    br#""1985-04-12T23:20:50.52Z""#,
                    br#""1985-04-12T23:20:50Z""#,
                    br#""1996-12-19T16:39:57-08:00""#,
                    br#""1515531081""#,
                    br#""1515531081.1234""#,
                    br#""Tue, 29 Apr 2014 18:30:38 GMT""#,
                    br#""Infinity""#,
                    br#""NaN""#,
                    b"true",
                ],
            );
        }

        #[test]
        fn strict_explicit_epoch_seconds_requires_a_number() {
            assert_eq!(
                read(true, &EPOCH, b"1515531081").unwrap(),
                DateTime::from_secs(T)
            );
            assert_rejected(&EPOCH, &[br#""2018-01-09T20:51:21Z""#, br#""1515531081""#]);
        }

        #[test]
        fn strict_date_time_requires_an_rfc3339_utc_string() {
            assert_eq!(
                read(true, &DATE_TIME, br#""2018-01-09T20:51:21Z""#).unwrap(),
                DateTime::from_secs(T)
            );
            assert_rejected(
                &DATE_TIME,
                &[
                    b"1515531081",
                    b"1515531081.1234",
                    br#""1996-12-19T16:39:57-08:00""#,
                    br#""1996-12-19T16:39:57+00""#,
                    br#""1996-12-19T16:39:57""#,
                    br#""Tue, 29 Apr 2014 18:30:38 GMT""#,
                ],
            );
        }

        #[test]
        fn strict_http_date_requires_an_imf_fixdate_string() {
            assert_eq!(
                read(true, &HTTP_DATE, br#""Tue, 09 Jan 2018 20:51:21 GMT""#).unwrap(),
                DateTime::from_secs(T)
            );
            assert_rejected(
                &HTTP_DATE,
                &[
                    b"1515531081",
                    b"1515531081.1234",
                    br#""1985-04-12T23:20:50.52Z""#,
                    br#""1996-12-19T16:39:57-08:00""#,
                ],
            );
        }

        #[test]
        fn lenient_default_keeps_the_client_behavior() {
            // A number is epoch seconds whatever the format; a string for a `date-time`
            // member or a member without the trait parses as offset-aware `date-time`;
            // explicit `http-date` and `epoch-seconds` traits are honored for strings.
            assert_eq!(
                read(false, &DATE_TIME, b"1515531081").unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(false, &HTTP_DATE, b"1515531081").unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(false, &DEFAULT, br#""2018-01-09T20:51:21Z""#).unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(false, &DATE_TIME, br#""2018-01-09T21:51:21+01:00""#).unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(false, &HTTP_DATE, br#""Tue, 09 Jan 2018 20:51:21 GMT""#).unwrap(),
                DateTime::from_secs(T)
            );
            assert_eq!(
                read(false, &EPOCH, br#""1515531081""#).unwrap(),
                DateTime::from_secs(T)
            );
        }
    }
}

#[cfg(test)]
mod strictness_tests {
    use super::*;

    #[test]
    fn strict_strings_and_client_defaults() {
        for input in [b"\"raw\ncontrol\"".as_slice(), b"\"raw\x00control\""] {
            let mut client = JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()));
            assert!(client.read_string(dummy_schema()).is_ok());
            for key in [false, true] {
                let mut server = JsonDeserializer::new(
                    input,
                    Arc::new(
                        JsonCodecSettings::builder()
                            .enforce_strictness(true)
                            .build(),
                    ),
                );
                assert!(if key {
                    server.parse_key().map(|_| ())
                } else {
                    server.read_string(dummy_schema()).map(|_| ())
                }
                .is_err());
                let mut server = JsonDeserializer::new(
                    input,
                    Arc::new(
                        JsonCodecSettings::builder()
                            .enforce_strictness(true)
                            .build(),
                    ),
                );
                assert!(server.skip_string().is_err());
            }
        }
        for input in [b"\"ok\\n\\u0041\"".as_slice(), "\"hello 世界\"".as_bytes()] {
            let mut server = JsonDeserializer::new(
                input,
                Arc::new(
                    JsonCodecSettings::builder()
                        .enforce_strictness(true)
                        .build(),
                ),
            );
            assert!(server.skip_string().is_ok());
        }
        for strict in [false, true] {
            let settings = Arc::new(
                JsonCodecSettings::builder()
                    .enforce_strictness(strict)
                    .build(),
            );
            for input in [b"\"\xff\"".as_slice(), b"\"\xc0\x80\""] {
                assert!(JsonDeserializer::new(input, settings.clone())
                    .read_string(dummy_schema())
                    .is_err());
                assert!(JsonDeserializer::new(input, settings.clone())
                    .parse_key()
                    .is_err());
            }
        }
        let bad = b"\"\xff\"";
        assert!(
            JsonDeserializer::new(bad, Arc::new(JsonCodecSettings::default()))
                .skip_string()
                .is_ok()
        );
        assert!(JsonDeserializer::new(
            bad,
            Arc::new(
                JsonCodecSettings::builder()
                    .enforce_strictness(true)
                    .build()
            )
        )
        .skip_string()
        .is_err());
    }

    #[test]
    fn parse_integral_decimal_exact_cases() {
        for (input, expected) in [
            ("1.0", Some(1)),
            ("1e3", Some(1000)),
            ("12.0", Some(12)),
            ("100E0", Some(100)),
            ("1.50E1", Some(15)),
            ("1234567890.000", Some(1234567890)),
            ("0.5e1", Some(5)),
            ("0.05e2", Some(5)),
            ("100e-2", Some(1)),
            ("12300e-2", Some(123)),
            ("1.23e2", Some(123)),
            ("0.0", Some(0)),
            ("-0e5", Some(0)),
            ("0e-5", Some(0)),
            ("0e400", Some(0)),
            ("-123456789012345e3", Some(-123456789012345000)),
            ("9007199254740993.0", Some(9007199254740993)),
            ("9223372036854775807e0", Some(i64::MAX)),
            ("-9223372036854775808.0", Some(i64::MIN)),
            ("1.5", None),
            ("2.5e0", None),
            ("123e-2", None),
            ("1e-5", None),
            ("1e-400", None),
            ("1e19", None),
            ("1e30", None),
            ("9223372036854775808.0", None),
            ("-9223372036854775809e0", None),
            ("0e999999999999999999999", None),
        ] {
            assert_eq!(parse_integral_decimal(input).ok(), expected, "{input}");
        }
    }

    #[test]
    fn exact_integral_numbers_and_strict_grammar() {
        let settings = Arc::new(
            JsonCodecSettings::builder()
                .enforce_strictness(true)
                .allow_integral_float_numbers(true)
                .build(),
        );
        for (input, expected) in [
            ("1.0", 1),
            ("1e3", 1000),
            ("1.20e1", 12),
            ("9007199254740993", 9007199254740993),
            ("9223372036854775807.0", i64::MAX),
            ("-9223372036854775808e0", i64::MIN),
        ] {
            assert_eq!(
                JsonDeserializer::new(input.as_bytes(), settings.clone())
                    .read_long(dummy_schema())
                    .unwrap(),
                expected,
                "{input}"
            );
        }
        for input in [
            "0e999999999999999999999",
            "1.01",
            "1e-3",
            "9223372036854775808.0",
            "-9223372036854775809.0",
            "1e10000",
            "12.",
            "01",
            "+1",
            "1e",
            "--1",
            "1e+",
            "1.0.0",
        ] {
            assert!(
                JsonDeserializer::new(input.as_bytes(), settings.clone())
                    .read_long(dummy_schema())
                    .is_err(),
                "{input}"
            );
        }
        for input in ["12.", "01", "+1", "1e", "1e+", "--1", "1.0.0"] {
            assert!(
                JsonDeserializer::new(input.as_bytes(), settings.clone())
                    .read_double(dummy_schema())
                    .is_err(),
                "{input}"
            );
        }
        assert_eq!(
            JsonDeserializer::new(b"1.0", Arc::new(JsonCodecSettings::default()))
                .read_long(dummy_schema())
                .unwrap(),
            1
        );
        assert_eq!(
            JsonDeserializer::new(b"12.", Arc::new(JsonCodecSettings::default()))
                .read_double(dummy_schema())
                .unwrap(),
            12.0
        );
        for strict in [false, true] {
            let settings = Arc::new(
                JsonCodecSettings::builder()
                    .enforce_strictness(strict)
                    .allow_integral_float_numbers(true)
                    .build(),
            );
            assert_eq!(
                JsonDeserializer::new(b"127.0", settings.clone())
                    .read_byte(dummy_schema())
                    .unwrap(),
                127
            );
            assert!(JsonDeserializer::new(b"128.0", settings.clone())
                .read_byte(dummy_schema())
                .is_err());
            assert!(JsonDeserializer::new(b"32768e0", settings.clone())
                .read_short(dummy_schema())
                .is_err());
            assert!(JsonDeserializer::new(b"2147483648.0", settings.clone())
                .read_integer(dummy_schema())
                .is_err());
            assert!(JsonDeserializer::new(b"12.", settings)
                .read_long(dummy_schema())
                .is_err());
        }
        for input in ["1e999", "-1e999", "9223372036854775808.0"] {
            assert!(JsonDeserializer::new(input.as_bytes(), settings.clone())
                .read_timestamp(dummy_schema())
                .is_err());
        }
        for (input, valid) in [(b"".as_slice(), true), (b" \n", false), (b"{}", true)] {
            assert_eq!(
                JsonDeserializer::new(input, settings.clone())
                    .read_struct(dummy_schema(), &mut |_, _| Ok(()))
                    .is_ok(),
                valid
            );
        }
    }

    fn dummy_schema() -> &'static Schema {
        &aws_smithy_schema::prelude::STRING
    }
}
