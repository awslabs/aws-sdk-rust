/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON deserializer implementation.

use aws_smithy_schema::serde::SerdeError;
use aws_smithy_schema::serde::ShapeDeserializer;
use aws_smithy_schema::Schema;
use aws_smithy_types::{
    BigDecimal, BigInteger, Blob, DateTime, DiscriminatedDocument, Document, DocumentSettings,
    Number,
};

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
    fn resolve_member<'s>(
        &self,
        schema: &'s Schema<'s>,
        field_name: &str,
    ) -> Option<&'s Schema<'s>> {
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
                    return Err(SerdeError::invalid_input("raw control character in string"));
                }
                _ => i += 1,
            }
        }
        if !found_end {
            return Err(SerdeError::invalid_input("unterminated string key"));
        }
        self.position = start + i + 1; // advance past key bytes + closing quote
        let key_bytes = &input[start..start + i];
        if has_escapes {
            let raw = std::str::from_utf8(key_bytes)
                .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
            Ok(std::borrow::Cow::Owned(
                crate::escape::unescape_string(raw)
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?
                    .into_owned(),
            ))
        } else {
            Ok(std::borrow::Cow::Borrowed(
                std::str::from_utf8(key_bytes)
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?,
            ))
        }
    }

    /// Predicate: is `s` a well-formed absolute Smithy shape ID of
    /// the form `namespace#ShapeName` (no member component, no nested
    /// `#`, both segments non-empty)?
    ///
    /// Used by [`Self::resolve_shape_id`] to short-circuit absolute
    /// names and skip namespace prepending. Returning `false` for an
    /// input that contains `#` indicates a malformed value (e.g.
    /// nested `#`, empty namespace) — caller falls back to leaving
    /// the `__type` key in the resulting map so a stray
    /// `__type: "garbage#"` doesn't poison the surrounding document
    /// parse.
    fn is_absolute_shape_id(s: &str) -> bool {
        let Some((namespace, shape_name)) = s.split_once('#') else {
            return false;
        };
        !namespace.is_empty()
            && !shape_name.is_empty()
            && !shape_name.contains('#')
            && !shape_name.contains('$')
            && !namespace.contains('$')
    }

    /// Predicate: is `s` a syntactically valid Smithy shape name
    /// (the part after `#` in an absolute shape ID)?
    ///
    /// Smithy identifiers per the spec begin with an ASCII letter or
    /// underscore and continue with ASCII letters, digits, or
    /// underscores. Returning `false` causes
    /// [`Self::resolve_shape_id`] to leave the candidate value as a
    /// regular `Document::String` in the resulting map rather than
    /// produce a syntactically invalid synthesized shape ID like
    /// `<namespace>#foo bar`.
    fn is_relative_shape_name(s: &str) -> bool {
        let mut bytes = s.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == b'_') {
            return false;
        }
        bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
    }

    /// Resolves a `__type` field value to a fully-qualified shape ID
    /// string. Absolute IDs are returned as-is; relative names
    /// (no `#`) are resolved against `default_ns` if both are set
    /// and the name is syntactically valid. Returns `None` to signal
    /// the lift should be skipped (caller leaves the value in the map).
    fn resolve_shape_id(s: &str, default_ns: Option<&str>) -> Option<String> {
        if s.contains('#') {
            // Absolute form (or malformed) — only lift if syntactically
            // valid. The `default_namespace` setting is intentionally
            // ignored on this path: an explicit absolute `__type`
            // overrides the codec's default namespace.
            if Self::is_absolute_shape_id(s) {
                Some(s.to_owned())
            } else {
                None
            }
        } else {
            // Relative form — resolve against `default_namespace` if
            // set and the name is syntactically valid. Without a
            // configured namespace the lift is skipped (today's
            // legacy behavior preserved).
            match default_ns {
                Some(ns) if Self::is_relative_shape_name(s) => Some(format!("{ns}#{s}")),
                _ => None,
            }
        }
    }
}

impl<'a> ShapeDeserializer for JsonDeserializer<'a> {
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        // Expect opening brace
        self.skip_whitespace();
        if self.remaining().is_empty() {
            if self.settings.enforce_strictness && (!self.input.is_empty() || self.depth != 1) {
                return Err(SerdeError::invalid_input("expected object"));
            }
            // Treat empty input as an empty object (e.g., empty HTTP response body)
            self.depth -= 1;
            return Ok(());
        }
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::type_mismatch("expected object"));
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
                return Err(SerdeError::invalid_input("expected object key"));
            }

            // Parse the key directly from bytes
            let key_str = self.parse_key()?;

            // Skip whitespace and expect colon
            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::invalid_input("expected colon after key"));
            }
            self.advance_by(1);
            self.skip_whitespace();

            // Process the value — skip nulls (they represent absent optional members)
            let rem = self.remaining();
            if rem.starts_with(b"null") && !rem.get(4).is_some_and(|b| b.is_ascii_alphanumeric()) {
                self.advance_by(4);
            } else if let Some(member_schema) = self.resolve_member(schema, &key_str) {
                consumer(member_schema, self)?;
            } else if &*key_str == "__type" {
                // Protocol discriminator, never a member.
                self.skip_value()?;
            } else {
                // Report the unknown key so a union consumer can act on it: the prelude
                // `DOCUMENT` schema has no member index, so generated code lands in its
                // fallthrough arm. The consumer may read the value (as a document); if it
                // does not, the position is unchanged and the value is skipped here.
                let start = self.position;
                consumer(&aws_smithy_schema::prelude::DOCUMENT, self)?;
                if self.position == start {
                    self.skip_value()?;
                }
            }
        }

        self.leave_container()?;
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::type_mismatch("expected array"));
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
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::type_mismatch("expected object"));
        }
        self.advance_by(1);

        let mut first = true;
        loop {
            if self.next_element(first, b'}', "object")? {
                break;
            }
            first = false;
            if self.remaining().first() != Some(&b'"') {
                return Err(SerdeError::invalid_input("expected key"));
            }

            let key = self.parse_key()?;

            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::invalid_input("expected colon"));
            }
            self.advance_by(1);
            self.skip_whitespace();

            consumer(key.into_owned(), self)?;
        }

        self.leave_container()?;
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.skip_whitespace();
        let rem = self.remaining();
        if rem.starts_with(b"true") {
            self.advance_by(4);
            Ok(true)
        } else if rem.starts_with(b"false") {
            self.advance_by(5);
            Ok(false)
        } else {
            Err(SerdeError::type_mismatch("expected boolean"))
        }
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i8::try_from(n).map_err(|_| SerdeError::invalid_input("value out of range for byte"))
        })
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i16::try_from(n).map_err(|_| SerdeError::invalid_input("value out of range for short"))
        })
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.read_integer_value().and_then(|n| {
            i32::try_from(n)
                .map_err(|_| SerdeError::invalid_input("value out of range for integer"))
        })
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.read_integer_value()
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.read_float_value().map(|f| f as f32)
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.read_float_value()
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        use std::str::FromStr;
        self.skip_whitespace();
        match self.remaining().first() {
            Some(b'-') | Some(b'0'..=b'9') => {
                let start = self.position;
                self.consume_number()?;
                let num_str = std::str::from_utf8(&self.input[start..self.position])
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
                BigInteger::from_str(num_str).map_err(|e| SerdeError::invalid_input(e.to_string()))
            }
            // String-encoded big integers are produced by senders
            // configured with `use_string_for_arbitrary_precision`. The
            // read side accepts both wire forms unconditionally so a
            // sender configured for one form interoperates with a
            // receiver configured for the other.
            Some(b'"') => {
                let s = self.read_string(_schema)?;
                BigInteger::from_str(&s).map_err(|e| SerdeError::invalid_input(e.to_string()))
            }
            _ => Err(SerdeError::type_mismatch("expected number or string")),
        }
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        use std::str::FromStr;
        self.skip_whitespace();
        match self.remaining().first() {
            Some(b'-') | Some(b'0'..=b'9') => {
                let start = self.position;
                self.consume_number()?;
                let num_str = std::str::from_utf8(&self.input[start..self.position])
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
                BigDecimal::from_str(num_str).map_err(|e| SerdeError::invalid_input(e.to_string()))
            }
            // String-encoded big decimals — see the symmetric comment
            // on `read_big_integer`.
            Some(b'"') => {
                let s = self.read_string(_schema)?;
                BigDecimal::from_str(&s).map_err(|e| SerdeError::invalid_input(e.to_string()))
            }
            _ => Err(SerdeError::type_mismatch("expected number or string")),
        }
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.skip_whitespace();
        let pos = self.position;
        let input = self.input;
        let rem = &input[pos..];
        if rem.first() != Some(&b'"') {
            return Err(SerdeError::type_mismatch("expected string"));
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
                    return std::str::from_utf8(raw)
                        .map(|s| s.to_owned())
                        .map_err(|e| SerdeError::invalid_input(e.to_string()));
                }
                let s = std::str::from_utf8(raw)
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
                return crate::deserialize::EscapedStr::new(s)
                    .to_unescaped()
                    .map(|s| s.into_owned())
                    .map_err(|e| SerdeError::invalid_input(e.to_string()));
            } else {
                if self.settings.enforce_strictness && rem[i] < 32 {
                    return Err(SerdeError::invalid_input("raw control character in string"));
                }
                i += 1;
            }
        }
        Err(SerdeError::invalid_input("unterminated string"))
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        let s = self.read_string(_schema)?;
        let decoded = aws_smithy_types::base64::decode(&s)
            .map_err(|e| SerdeError::invalid_input(format!("invalid base64: {}", e)))?;
        Ok(Blob::new(decoded))
    }

    fn read_string_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::type_mismatch("expected array"));
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

    fn read_blob_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<Blob>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::type_mismatch("expected array"));
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

    fn read_integer_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::type_mismatch("expected array"));
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

    fn read_long_list(&mut self, _schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'[') {
            return Err(SerdeError::type_mismatch("expected array"));
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
        _schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        if self.remaining().first() != Some(&b'{') {
            return Err(SerdeError::type_mismatch("expected object"));
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
                return Err(SerdeError::invalid_input("expected key"));
            }
            let key = self.parse_key()?;
            self.skip_whitespace();
            if self.remaining().first() != Some(&b':') {
                return Err(SerdeError::invalid_input("expected colon"));
            }
            self.advance_by(1);
            self.skip_whitespace();
            let val = self.read_string(_schema)?;
            out.insert(key.into_owned(), val);
        }
        self.leave_container()?;
        Ok(out)
    }

    fn read_timestamp(&mut self, schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
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
                        return Err(SerdeError::type_mismatch(
                            "expected a JSON number for an epoch-seconds timestamp",
                        ));
                    }
                    format
                } else {
                    // Lenient (unchanged behavior): an explicit `@timestampFormat` of
                    // `http-date` or `epoch-seconds` is honored as written, and `date-time`
                    // parses as offset-aware `date-time`. With no `@timestampFormat` trait,
                    // honor the codec's configured default format. A JSON string can't be
                    // `epoch-seconds` (a number), so that case resolves to the offset-aware
                    // date-time form. Shared with
                    // `JsonCodecSettings::coerce_string_to_timestamp` so the typed and
                    // untyped string-timestamp paths can't drift.
                    match schema.timestamp_format().map(|t| t.format()) {
                        Some(TimestampFormat::HttpDate) => Format::HttpDate,
                        Some(TimestampFormat::EpochSeconds) => Format::EpochSeconds,
                        Some(TimestampFormat::DateTime) => Format::DateTimeWithOffset,
                        None => crate::codec::string_timestamp_format(
                            self.settings.default_timestamp_format(),
                        ),
                    }
                };
                let s = self.read_string(schema)?;
                DateTime::from_str(&s, string_format)
                    .map_err(|e| SerdeError::custom(format!("invalid timestamp string: {e}")))
            }
            Some(b'-') | Some(b'0'..=b'9') => {
                if strict && !matches!(format, Format::EpochSeconds) {
                    return Err(SerdeError::type_mismatch(
                        "expected a JSON string for a date-time or http-date timestamp",
                    ));
                }
                // Numeric timestamp: epoch seconds.
                let start = self.position;
                self.consume_number()?;
                let num_str = std::str::from_utf8(&self.input[start..self.position])
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
                if num_str.contains('.') || num_str.contains('e') || num_str.contains('E') {
                    let f: f64 = num_str.parse().map_err(|e: std::num::ParseFloatError| {
                        SerdeError::invalid_input(e.to_string())
                    })?;
                    // The i64 range as f64. `i64::MAX as f64` rounds up to 2^63, which is
                    // why the range is half-open: 2^63 itself overflows `floor() as i64`
                    // and would saturate silently in `DateTime::from_secs_f64`.
                    const I64_MIN_F64: f64 = i64::MIN as f64;
                    const I64_MAX_F64: f64 = i64::MAX as f64;
                    if self.settings.enforce_strictness
                        && (!f.is_finite() || !(I64_MIN_F64..I64_MAX_F64).contains(&f))
                    {
                        return Err(SerdeError::invalid_input(
                            "epoch-seconds value out of range",
                        ));
                    }
                    Ok(DateTime::from_secs_f64(f))
                } else if num_str.starts_with('-') {
                    let n: i64 = num_str.parse().map_err(|e: std::num::ParseIntError| {
                        SerdeError::invalid_input(e.to_string())
                    })?;
                    Ok(DateTime::from_secs(n))
                } else {
                    let n: u64 = num_str.parse().map_err(|e: std::num::ParseIntError| {
                        SerdeError::invalid_input(e.to_string())
                    })?;
                    if n > i64::MAX as u64 {
                        return Err(SerdeError::invalid_input(format!(
                            "epoch-seconds value {n} overflows i64; cannot construct DateTime"
                        )));
                    }
                    Ok(DateTime::from_secs(n as i64))
                }
            }
            _ => Err(SerdeError::type_mismatch("expected timestamp")),
        }
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        self.depth += 1;
        if self.depth > self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.skip_whitespace();
        let result: Result<Document, SerdeError> = match self.remaining().first() {
            Some(b'"') => Ok(Document::String(self.read_string(_schema)?)),
            Some(b't') | Some(b'f') => Ok(Document::Bool(self.read_boolean(_schema)?)),
            Some(b'n') => {
                if self.remaining().starts_with(b"null") {
                    self.advance_by(4);
                    Ok(Document::Null)
                } else {
                    Err(SerdeError::invalid_input("unexpected token in document"))
                }
            }
            Some(b'{') => {
                self.advance_by(1);
                let mut map = aws_smithy_types::document::DocumentObject::new();
                let mut first = true;
                loop {
                    if self.next_element(first, b'}', "document object")? {
                        break;
                    }
                    first = false;
                    if self.remaining().first() != Some(&b'"') {
                        return Err(SerdeError::invalid_input("expected object key in document"));
                    }
                    let key = self.parse_key()?.into_owned();
                    self.skip_whitespace();
                    if self.remaining().first() != Some(&b':') {
                        return Err(SerdeError::invalid_input(
                            "expected colon in document object",
                        ));
                    }
                    self.advance_by(1);
                    self.skip_whitespace();
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
                // Parse a JSON number into [`Document::Number`].
                // Range determines which `Number` variant carries it
                // (PosInt / NegInt / Float).
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
                let s = std::str::from_utf8(&self.input[pos..pos + len])
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
                // Number variant selection follows the SEP "Reporting
                // `Document` ambiguous shape types" guidance: pick the
                // first container from `int -> long -> bigInteger ->
                // double -> bigDecimal` that holds the value without
                // loss of precision. `byte`, `intEnum`, `short`, and
                // `float` are intentionally skipped per the same SEP
                // guidance.
                //
                // The `Number` enum collapses `int`/`long` into
                // `NegInt(i64)` and `PosInt(u64)`, so we only need
                // three buckets here at the wire layer: `Number`
                // (fits in i64 / u64 / finite f64), `BigInteger`
                // (overflows i64 / u64), and `BigDecimal` (decimal
                // value overflows f64 to non-finite).
                use std::str::FromStr;
                if is_float {
                    match s.parse::<f64>() {
                        Ok(f) if f.is_finite() => Ok(Document::Number(Number::Float(f))),
                        // Either `f64` parse failed or yielded
                        // `+/-Infinity` (overflow). Fall through to
                        // `BigDecimal` so the source-string precision
                        // is preserved. `BigDecimal::from_str` will
                        // surface a real parse error if the input is
                        // also malformed for arbitrary precision.
                        _ => BigDecimal::from_str(s)
                            .map(Document::BigDecimal)
                            .map_err(|e| SerdeError::invalid_input(e.to_string())),
                    }
                } else if is_negative {
                    match s.parse::<i64>() {
                        Ok(n) => Ok(Document::Number(Number::NegInt(n))),
                        // Source string overflowed `i64`. Preserve
                        // precision by routing to `BigInteger`.
                        Err(_) => BigInteger::from_str(s)
                            .map(Document::BigInteger)
                            .map_err(|e| SerdeError::invalid_input(e.to_string())),
                    }
                } else {
                    match s.parse::<u64>() {
                        Ok(n) => Ok(Document::Number(Number::PosInt(n))),
                        // Source string overflowed `u64`. Preserve
                        // precision by routing to `BigInteger`.
                        Err(_) => BigInteger::from_str(s)
                            .map(Document::BigInteger)
                            .map_err(|e| SerdeError::invalid_input(e.to_string())),
                    }
                }
            }
            _ => Err(SerdeError::invalid_input("unexpected token in document")),
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
    /// Reads a JSON document from the input, lifts an optional
    /// top-level `__type` field into the result's discriminator slot,
    /// and attaches the codec's [`DocumentSettings`] to the result so
    /// downstream accessors like
    /// [`DiscriminatedDocument::as_blob`](aws_smithy_types::DiscriminatedDocument::as_blob)
    /// and
    /// [`DiscriminatedDocument::as_timestamp`](aws_smithy_types::DiscriminatedDocument::as_timestamp)
    /// can perform JSON-specific coercion (base64-decoded blobs,
    /// format-aware timestamps).
    ///
    /// # `__type` discriminator lift
    ///
    /// When the parsed top-level value is a JSON object containing a
    /// `__type` key whose value is an absolute Smithy shape ID
    /// (`namespace#ShapeName`), the lift extracts that ID into the
    /// resulting [`DiscriminatedDocument`]'s discriminator slot and
    /// drops the key from the result map.
    ///
    /// Falls back to leaving `__type` in the map when the value is
    /// not a string, or is a relative shape name (no `#`) and no
    /// `default_namespace` is configured. When a `default_namespace`
    /// is set, a relative name is resolved against it before lifting.
    ///
    /// # Example
    ///
    /// ```
    /// use aws_smithy_json::codec::JsonCodec;
    /// use aws_smithy_schema::codec::Codec;
    ///
    /// let codec = JsonCodec::default();
    /// let bytes = br#"{"__type":"smithy.example#Bird","name":"Iago"}"#;
    /// let mut deser = codec.create_deserializer(bytes);
    /// let doc = deser.read_discriminated_document().unwrap();
    ///
    /// // `__type` was lifted into the discriminator slot.
    /// assert_eq!(doc.discriminator(), Some("smithy.example#Bird"));
    ///
    /// // The map no longer contains `__type`.
    /// let map = doc.document().as_object().unwrap();
    /// assert_eq!(map.len(), 1);
    /// assert_eq!(map.get("name").and_then(|d| d.as_string()), Some("Iago"));
    /// ```
    pub fn read_discriminated_document(&mut self) -> Result<DiscriminatedDocument, SerdeError> {
        // Use the trait method's walker and post-hoc lift the
        // top-level `__type` field. The lift cost is one HashMap
        // probe + one `String` clone — straightforward because
        // [`Document`] is fully owned and [`DiscriminatedDocument`]
        // carries an `Option<String>` discriminator.
        let dummy_schema = &aws_smithy_schema::prelude::DOCUMENT;
        let mut doc = self.read_document(dummy_schema)?;
        let mut discriminator: Option<String> = None;
        if let Document::Object(ref mut map) = doc {
            // Pull the candidate value first (borrowing), validate it,
            // then remove from the map only when the lift succeeds.
            // This keeps a malformed `__type` value in the map so the
            // caller can post-process if it wants to.
            let lift_candidate = match map.get("__type") {
                Some(Document::String(s)) => {
                    Self::resolve_shape_id(s, self.settings.default_namespace())
                }
                _ => None,
            };
            if let Some(id) = lift_candidate {
                map.remove("__type");
                discriminator = Some(id);
            }
        }

        let settings: Arc<dyn DocumentSettings> = self.settings.clone();
        let mut wrapper = DiscriminatedDocument::new(doc).with_settings(settings);
        if let Some(id) = discriminator {
            wrapper = wrapper.with_discriminator(id);
        }
        Ok(wrapper)
    }

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
            None => Err(SerdeError::invalid_input(format!(
                "unexpected end of input in {what}"
            ))),
            Some(b',') if first => Err(SerdeError::invalid_input(format!(
                "unexpected `,` before the first element of {what}"
            ))),
            Some(b',') => {
                self.advance_by(1);
                self.skip_whitespace();
                match self.remaining().first().copied() {
                    Some(b) if b == close => {
                        Err(SerdeError::invalid_input(format!("trailing `,` in {what}")))
                    }
                    Some(b',') => Err(SerdeError::invalid_input(format!("repeated `,` in {what}"))),
                    None => Err(SerdeError::invalid_input(format!(
                        "unexpected end of input in {what}"
                    ))),
                    Some(_) => Ok(false),
                }
            }
            Some(_) if first => Ok(false),
            Some(_) => Err(SerdeError::invalid_input(format!(
                "expected `,` between elements of {what}"
            ))),
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
                return Err(SerdeError::invalid_input(
                    "unexpected trailing characters after the JSON value",
                ));
            }
        }
        Ok(())
    }

    /// Skips a JSON number, validating it against the RFC 8259 grammar
    /// `-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`. Whatever follows is left for the
    /// enclosing container's separator check, so `01` fails there as "expected `,`".
    fn skip_number(&mut self) -> Result<(), SerdeError> {
        let rem = self.remaining();
        let invalid = || SerdeError::invalid_input("invalid number");
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
                None => return Err(SerdeError::invalid_input("unterminated string")),
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
                        return Err(SerdeError::invalid_input(
                            "invalid escape sequence in string",
                        ))
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
                return Err(SerdeError::invalid_input("invalid number boundary"));
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
                        return Err(SerdeError::invalid_input("expected object key"));
                    }
                    self.parse_key()?;
                    self.skip_whitespace();
                    if self.remaining().first() != Some(&b':') {
                        return Err(SerdeError::invalid_input("expected colon after key"));
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
            Some(_) => Err(SerdeError::invalid_input("unexpected token in skip_value")),
            None => Err(SerdeError::invalid_input("unexpected end of input")),
        }
    }

    fn skip_literal(&mut self, literal: &'static [u8]) -> Result<(), SerdeError> {
        if !self.remaining().starts_with(literal) {
            return Err(SerdeError::invalid_input(format!(
                "expected `{}`",
                String::from_utf8_lossy(literal)
            )));
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
                return Err(SerdeError::invalid_input("invalid number boundary"));
            }
            let text = std::str::from_utf8(&self.input[start..self.position])
                .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
            if self.settings.allow_integral_float_numbers && text.contains(['.', 'e', 'E']) {
                return parse_integral_decimal(text);
            }
            return text
                .parse()
                .map_err(|e: std::num::ParseIntError| SerdeError::invalid_input(e.to_string()));
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
            return Err(SerdeError::type_mismatch("expected integer"));
        }
        // Reject a floating-point value for an integer member. Without this,
        // `1.5` would read `1` and leave `.5` in the stream, surfacing as a
        // confusing downstream parse error instead of a clean TypeMismatch.
        if let Some(&next) = rem.get(len) {
            if next == b'.' || next == b'e' || next == b'E' {
                return Err(SerdeError::type_mismatch(
                    "expected integer, found floating-point number",
                ));
            }
        }
        let s = std::str::from_utf8(&rem[..len])
            .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
        let n = s
            .parse::<i64>()
            .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
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
                other => other
                    .parse::<f64>()
                    .map_err(|e| SerdeError::invalid_input(e.to_string()))?,
            };
            if value.is_finite() {
                return Err(SerdeError::type_mismatch(format!(
                        "only `Infinity`, `-Infinity`, `NaN` can represent a float as a string but found `{s}`"
                    )));
            }
            return Ok(value);
        }
        if self.settings.enforce_strictness {
            let start = self.position;
            self.consume_number()?;
            return std::str::from_utf8(&self.input[start..self.position])
                .map_err(|e| SerdeError::invalid_input(e.to_string()))?
                .parse()
                .map_err(|e: std::num::ParseFloatError| SerdeError::invalid_input(e.to_string()));
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
            return Err(SerdeError::type_mismatch("expected number"));
        }
        let s = std::str::from_utf8(&rem[..len])
            .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
        let n = s
            .parse::<f64>()
            .map_err(|e| SerdeError::invalid_input(e.to_string()))?;
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
    let invalid = || SerdeError::invalid_input("number is fractional or outside integer range");
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

    fn dummy_schema() -> &'static aws_smithy_schema::Schema<'static> {
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

        static FIRST_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::String,
            "firstName",
            0,
        );
        static LAST_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::String,
            "lastName",
            1,
        );
        static AGE: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::Integer,
            "age",
            2,
        );
        static PERSON_SCHEMA: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Person"),
            aws_smithy_schema::ShapeType::Structure,
            &[&FIRST_NAME, &LAST_NAME, &AGE],
        );

        fn consume_person(
            person: &mut Person,
            schema: &Schema<'_>,
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
        static ADDR_STREET: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "street",
            0,
        );
        static ADDR_CITY: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "city",
            1,
        );
        static ADDR_ZIP: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::Integer,
            "zip",
            2,
        );
        static ADDRESS_SCHEMA: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::Structure,
            &[&ADDR_STREET, &ADDR_CITY, &ADDR_ZIP],
        );

        // Company members & schema
        static COMP_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::String,
            "name",
            0,
        );
        static COMP_EMPLOYEES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::List,
            "employees",
            1,
        );
        static COMP_METADATA: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Map,
            "metadata",
            2,
        );
        static COMP_ACTIVE: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Boolean,
            "active",
            3,
        );
        static COMPANY_SCHEMA: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Structure,
            &[&COMP_NAME, &COMP_EMPLOYEES, &COMP_METADATA, &COMP_ACTIVE],
        );

        // User members & schema
        static USER_ID: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Long,
            "id",
            0,
        );
        static USER_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::String,
            "name",
            1,
        );
        static USER_SCORES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "scores",
            2,
        );
        static USER_ADDRESS: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Structure,
            "address",
            3,
        );
        static USER_COMPANIES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "companies",
            4,
        );
        static USER_TAGS: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Map,
            "tags",
            5,
        );
        static USER_SCHEMA: Schema<'static> = Schema::new_struct(
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
            schema: &Schema<'_>,
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
            schema: &Schema<'_>,
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
            schema: &Schema<'_>,
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

        static FOO_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::String,
            "foo",
            0,
        );
        // "bar" member has @jsonName("Baz")
        static BAR_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Integer,
            "bar",
            1,
        )
        .with_json_name("Baz");
        static STRUCT_SCHEMA: Schema<'static> = Schema::new_struct(
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

    fn timestamp_schema() -> &'static aws_smithy_schema::Schema<'static> {
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
    fn test_read_timestamp_string_honors_httpdate_default() {
        // No `@timestampFormat` trait on the schema, so the codec's default
        // format governs string parsing. With an http-date default, an
        // http-date string must parse. (Before L2a the string branch
        // hardcoded date-time and this errored.)
        let settings = JsonCodecSettings::builder()
            .default_timestamp_format(aws_smithy_types::date_time::Format::HttpDate)
            .build();
        let mut deser =
            JsonDeserializer::new(br#""Tue, 14 Nov 2023 22:13:20 GMT""#, Arc::new(settings));
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs(1700000000));
    }

    #[test]
    fn test_read_timestamp_string_default_epoch_still_parses_datetime() {
        // Regression guard for the common case: with the default
        // (epoch-seconds) format, a date-time *string* still parses via the
        // offset-aware fallback — the shared `string_timestamp_format` helper
        // must not break this.
        let mut deser = JsonDeserializer::new(
            br#""2023-11-14T22:13:20Z""#,
            Arc::new(JsonCodecSettings::default()),
        );
        let ts = deser.read_timestamp(timestamp_schema()).unwrap();
        assert_eq!(ts, DateTime::from_secs(1700000000));
    }

    #[test]
    fn test_read_timestamp_rejects_u64_overflow() {
        // A positive epoch-seconds value beyond i64::MAX must error rather
        // than silently wrapping through `as i64`.
        let big = (i64::MAX as u64 + 1).to_string();
        let mut deser =
            JsonDeserializer::new(big.as_bytes(), Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_timestamp(timestamp_schema())
            .expect_err("u64 epoch overflowing i64 must error");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    #[test]
    fn test_read_integer_rejects_float() {
        // An integer member must reject a floating-point value rather than
        // reading the integer part and leaving the fraction in the stream.
        let mut deser = JsonDeserializer::new(b"1.5", Arc::new(JsonCodecSettings::default()));
        let err = deser
            .read_integer(&aws_smithy_schema::prelude::INTEGER)
            .expect_err("a float must not deserialize as an integer");
        assert!(
            matches!(err, SerdeError::TypeMismatch { .. }),
            "expected TypeMismatch, got {err:?}"
        );
    }

    #[test]
    fn test_skip_value_empty_array() {
        // Regression: skip_value failed on [] because json_token_iter can't parse ']' as a value start
        use aws_smithy_schema::ShapeType;
        static KNOWN_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::String,
            "known",
            0,
        );
        static MEMBERS: &[&Schema<'_>] = &[&KNOWN_MEMBER];
        static TEST_SCHEMA: Schema<'static> = Schema::new_struct(
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
        static D_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            ShapeType::String,
            "d",
            0,
        );
        static MEMBERS: &[&Schema<'_>] = &[&D_MEMBER];
        static TEST_SCHEMA: Schema<'static> = Schema::new_struct(
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

        static MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "m",
            0,
        );
        static SCHEMA: Schema<'static> = Schema::new_struct(
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

        static MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "known",
            0,
        );
        static SCHEMA: Schema<'static> = Schema::new_struct(
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

        static MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "known",
            0,
        );
        static SCHEMA: Schema<'static> = Schema::new_struct(
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

        static MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "S"),
            aws_smithy_schema::ShapeType::String,
            "m",
            0,
        );
        static SCHEMA: Schema<'static> = Schema::new_struct(
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
    fn recursive_struct_schema() -> &'static Schema<'static> {
        static MEMBER_A: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Rec"),
            aws_smithy_schema::ShapeType::Structure,
            "a",
            0,
        );
        static RECURSIVE: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Rec"),
            aws_smithy_schema::ShapeType::Structure,
            &[&MEMBER_A],
        );
        &RECURSIVE
    }

    /// Consumer that re-enters `read_struct` to exercise the depth guard.
    fn recursive_struct_consumer(
        _member: &Schema<'_>,
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
            SerdeError::Custom { ref message, .. } => {
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

    // --- read_document big-number fall-through (SEP "Reporting `Document` ambiguous shape types") ---

    #[test]
    fn read_document_keeps_in_range_int_as_pos_int() {
        // Baseline: a value that fits in `u64` continues to land in
        // `Number::PosInt` — no behavior change for the common case.
        let mut deser = JsonDeserializer::new(b"42", Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::Number(Number::PosInt(n)) => assert_eq!(n, 42),
            other => panic!("expected Number::PosInt(42), got {other:?}"),
        }
    }

    #[test]
    fn read_document_keeps_in_range_negative_int_as_neg_int() {
        let mut deser = JsonDeserializer::new(b"-123", Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::Number(Number::NegInt(n)) => assert_eq!(n, -123),
            other => panic!("expected Number::NegInt(-123), got {other:?}"),
        }
    }

    #[test]
    fn read_document_keeps_finite_float_as_number_float() {
        let mut deser = JsonDeserializer::new(b"1.5", Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::Number(Number::Float(f)) => assert_eq!(f, 1.5),
            other => panic!("expected Number::Float(1.5), got {other:?}"),
        }
    }

    #[test]
    fn read_document_lifts_oversize_positive_int_to_big_integer() {
        // 23-digit integer overflows `u64` (max is ~1.84e19, 20 digits).
        // Today (pre-fix) this errored out; per SEP it must be lifted
        // to `Document::BigInteger` to preserve precision.
        let bytes = b"99999999999999999999999";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::BigInteger(bi) => {
                assert_eq!(bi.as_ref(), "99999999999999999999999");
            }
            other => panic!("expected Document::BigInteger, got {other:?}"),
        }
    }

    #[test]
    fn read_document_lifts_oversize_negative_int_to_big_integer() {
        // 23-digit negative integer overflows `i64` (min is ~-9.2e18,
        // 19 digits). Same SEP fall-through as the positive case.
        let bytes = b"-99999999999999999999999";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::BigInteger(bi) => {
                assert_eq!(bi.as_ref(), "-99999999999999999999999");
            }
            other => panic!("expected Document::BigInteger, got {other:?}"),
        }
    }

    #[test]
    fn read_document_lifts_oversize_decimal_to_big_decimal() {
        // `1e500` overflows `f64` to `+Infinity`. Today (pre-fix) this
        // silently produced `Number::Float(infinity)` — precision
        // destroyed. Per SEP it must be lifted to `Document::BigDecimal`.
        let bytes = b"1e500";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::BigDecimal(bd) => {
                // `BigDecimal` may normalize the source; assert that
                // it round-trips back to a finite, oversize-decimal
                // representation rather than `inf`.
                let s = bd.as_ref();
                assert!(!s.contains("inf"), "expected finite repr, got {s}");
                assert!(!s.contains("Inf"), "expected finite repr, got {s}");
            }
            other => panic!("expected Document::BigDecimal, got {other:?}"),
        }
    }

    #[test]
    fn read_document_lifts_oversize_negative_decimal_to_big_decimal() {
        // Symmetric `-Infinity` overflow case.
        let bytes = b"-1.234e500";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::BigDecimal(bd) => {
                let s = bd.as_ref();
                assert!(s.starts_with('-'), "expected negative repr, got {s}");
                assert!(
                    !s.to_lowercase().contains("inf"),
                    "expected finite repr, got {s}"
                );
            }
            other => panic!("expected Document::BigDecimal, got {other:?}"),
        }
    }

    #[test]
    fn read_document_keeps_max_u64_as_pos_int() {
        // `u64::MAX` fits in `u64` exactly. Boundary check that the
        // overflow fall-through doesn't fire on values that DO fit.
        let bytes = b"18446744073709551615";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::Number(Number::PosInt(n)) => assert_eq!(n, u64::MAX),
            other => panic!("expected Number::PosInt(u64::MAX), got {other:?}"),
        }
    }

    #[test]
    fn read_document_lifts_just_over_u64_max() {
        // `u64::MAX + 1`. First value that overflows.
        let bytes = b"18446744073709551616";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        match deser.read_document(dummy_schema()).unwrap() {
            Document::BigInteger(bi) => {
                assert_eq!(bi.as_ref(), "18446744073709551616");
            }
            other => panic!("expected Document::BigInteger, got {other:?}"),
        }
    }

    // --- read_big_integer / read_big_decimal accept JSON strings (`use_string_for_arbitrary_precision`) ---

    #[test]
    fn read_big_integer_accepts_json_number() {
        let bytes = b"99999999999999999999999";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let bi = deser.read_big_integer(dummy_schema()).unwrap();
        assert_eq!(bi.as_ref(), "99999999999999999999999");
    }

    #[test]
    fn read_big_integer_accepts_json_string() {
        // A sender configured with `use_string_for_arbitrary_precision`
        // emits BigIntegers as JSON strings. The receiver must accept
        // either form regardless of its own setting — leniency is
        // unconditional on the read side.
        let bytes = br#""99999999999999999999999""#;
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let bi = deser.read_big_integer(dummy_schema()).unwrap();
        assert_eq!(bi.as_ref(), "99999999999999999999999");
    }

    #[test]
    fn read_big_integer_rejects_other_types() {
        // Booleans, arrays, etc. are not valid wire forms for BigInteger.
        let bytes = b"true";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let err = deser.read_big_integer(dummy_schema()).unwrap_err();
        assert!(
            matches!(err, SerdeError::TypeMismatch { .. }),
            "expected TypeMismatch, got {err:?}"
        );
    }

    #[test]
    fn read_big_decimal_accepts_json_number() {
        let bytes = b"1.234567890123456789012345";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let bd = deser.read_big_decimal(dummy_schema()).unwrap();
        assert_eq!(bd.as_ref(), "1.234567890123456789012345");
    }

    #[test]
    fn read_big_decimal_accepts_json_string() {
        let bytes = br#""1.234567890123456789012345""#;
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let bd = deser.read_big_decimal(dummy_schema()).unwrap();
        assert_eq!(bd.as_ref(), "1.234567890123456789012345");
    }

    #[test]
    fn read_big_decimal_rejects_other_types() {
        let bytes = b"[]";
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let err = deser.read_big_decimal(dummy_schema()).unwrap_err();
        assert!(
            matches!(err, SerdeError::TypeMismatch { .. }),
            "expected TypeMismatch, got {err:?}"
        );
    }

    #[test]
    fn read_big_integer_rejects_malformed_string() {
        // A JSON string of non-digits is well-formed JSON but invalid
        // BigInteger content. `BigInteger::from_str` should surface
        // an `InvalidInput` error.
        let bytes = br#""not-a-number""#;
        let mut deser = JsonDeserializer::new(bytes, Arc::new(JsonCodecSettings::default()));
        let err = deser.read_big_integer(dummy_schema()).unwrap_err();
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {err:?}"
        );
    }

    // --- read_discriminated_document attaches DocumentSettings ----------------------------

    #[test]
    fn read_discriminated_document_attaches_settings() {
        // Settings live on the [`DiscriminatedDocument`] wrapper, not
        // on individual data nodes. `read_discriminated_document` is
        // the entry point that attaches them.
        let mut deser = JsonDeserializer::new(b"\"hello\"", Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().unwrap();
        assert!(
            wrapper.settings().is_some(),
            "settings should be attached to the wrapper produced by \
             read_discriminated_document"
        );
    }

    #[test]
    fn read_discriminated_document_settings_enable_blob_coercion() {
        // "aGVsbG8=" is base64("hello"). Without settings, as_blob on a
        // String document returns an error. With settings attached by
        // `read_discriminated_document`, JsonCodecSettings::coerce_string_to_blob
        // decodes it.
        let mut deser =
            JsonDeserializer::new(br#""aGVsbG8=""#, Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().unwrap();
        let bytes = wrapper
            .as_blob()
            .expect("base64 string should decode to blob");
        assert_eq!(&*bytes, b"hello");
    }

    #[test]
    fn read_discriminated_document_settings_enable_timestamp_coercion() {
        // EpochSeconds is the JSON default; a number value coerces via
        // `coerce_number_to_timestamp`.
        let mut deser =
            JsonDeserializer::new(b"1577836800", Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().unwrap();
        let ts = wrapper
            .as_timestamp()
            .expect("epoch-seconds number should decode to timestamp");
        assert_eq!(ts.secs(), 1577836800);
    }

    // -- `__type` discriminator lift -----------------------------------------------------------

    #[test]
    fn read_discriminated_document_lifts_absolute_type_into_discriminator() {
        // An absolute `__type` value lands in the wrapper's
        // discriminator slot (as an owned `String` FQN), and the
        // `__type` key is dropped from the result map.
        let input = br#"{"__type":"smithy.example#Bird","name":"Iago"}"#;
        let mut deser = JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().expect("parse succeeds");

        let id = wrapper
            .discriminator()
            .expect("absolute __type lifted into discriminator");
        assert_eq!(id, "smithy.example#Bird");

        let map = wrapper.document().as_object().expect("top-level is map");
        assert!(
            !map.contains_key("__type"),
            "__type must be dropped from result map after lift"
        );
        assert_eq!(map.get("name").and_then(Document::as_string), Some("Iago"));
    }

    #[test]
    fn read_discriminated_document_relative_type_stays_in_map_when_no_default_namespace() {
        // Without a configured `default_namespace` on the codec
        // settings, a relative `__type` (no `#`) is left in the map
        // as a regular key. The discriminator slot remains unset.
        // With a namespace set, see
        // `read_discriminated_document_lifts_relative_with_namespace`.
        let input = br#"{"__type":"Bird","name":"Iago"}"#;
        let mut deser = JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().expect("parse succeeds");

        assert!(
            wrapper.discriminator().is_none(),
            "relative __type without default_namespace must not be lifted"
        );
        let map = wrapper.document().as_object().expect("top-level is map");
        assert_eq!(
            map.get("__type").and_then(Document::as_string),
            Some("Bird")
        );
    }

    #[test]
    fn read_discriminated_document_lifts_only_top_level() {
        // Only the top-level wrapper has a discriminator slot. A
        // `__type` field appearing INSIDE a nested object stays as a
        // regular map entry — there's no per-node discriminator to
        // lift it into.
        let input = br#"{"outer":{"__type":"smithy.example#Inner"}}"#;
        let mut deser = JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()));
        let wrapper = deser.read_discriminated_document().expect("parse succeeds");

        // Outer wrapper has no discriminator (the top-level object
        // didn't carry `__type` at its own level).
        assert!(wrapper.discriminator().is_none());
        // The inner object's `__type` remains as a regular string
        // entry in the nested object's map.
        let inner = wrapper
            .document()
            .as_object()
            .and_then(|m| m.get("outer"))
            .and_then(Document::as_object)
            .expect("outer key present and is object");
        assert_eq!(
            inner.get("__type").and_then(Document::as_string),
            Some("smithy.example#Inner"),
            "nested __type must remain as a regular map entry: \
             only the top-level wrapper has a discriminator slot",
        );
    }

    #[test]
    fn read_discriminated_document_malformed_type_stays_in_map() {
        // `__type` whose value isn't a well-formed absolute shape ID
        // (member-component, multiple `#`, empty parts) stays in the
        // map as a regular string entry and discriminator is None.
        for malformed in [
            "\"smithy.example#Foo$bar\"", // member component not allowed
            "\"smithy.example#\"",        // empty shape name
            "\"#smithy.example\"",        // empty namespace
            "\"no#hash#twice\"",          // multiple `#`
        ] {
            let body = format!(r#"{{"__type":{malformed}}}"#);
            let mut deser =
                JsonDeserializer::new(body.as_bytes(), Arc::new(JsonCodecSettings::default()));
            let wrapper = deser.read_discriminated_document().expect("parse succeeds");
            assert!(
                wrapper.discriminator().is_none(),
                "malformed __type {malformed} must not be lifted"
            );
            let map = wrapper.document().as_object().expect("top-level is map");
            assert!(
                map.contains_key("__type"),
                "malformed __type {malformed} must remain in map"
            );
        }
    }

    #[test]
    fn read_discriminated_document_non_string_type_stays_in_map() {
        // `__type` with a non-string value (number, object, array,
        // bool, null) stays in the map.
        for non_string in [
            r#"42"#,
            r#"true"#,
            r#"null"#,
            r#"["a"]"#,
            r#"{"nested":true}"#,
        ] {
            let body = format!(r#"{{"__type":{non_string}}}"#);
            let mut deser =
                JsonDeserializer::new(body.as_bytes(), Arc::new(JsonCodecSettings::default()));
            let wrapper = deser.read_discriminated_document().expect("parse succeeds");
            assert!(
                wrapper.discriminator().is_none(),
                "non-string __type {non_string} must not be lifted"
            );
            let map = wrapper.document().as_object().expect("top-level is map");
            assert!(
                map.contains_key("__type"),
                "non-string __type {non_string} must remain in map"
            );
        }
    }

    // --- read_discriminated_document with default_namespace --------------------------------

    fn settings_with_default_namespace(ns: &str) -> Arc<JsonCodecSettings> {
        Arc::new(JsonCodecSettings::builder().default_namespace(ns).build())
    }

    #[test]
    fn read_discriminated_document_lifts_relative_with_namespace() {
        // With a configured `default_namespace`, a relative `__type`
        // is prepended with `<namespace>#` and lifted into the
        // discriminator slot. The key is removed from the resulting
        // map (matching the absolute-form behavior).
        let input = br#"{"__type":"Capacity","CapacityUnits":1.0}"#;
        let settings = settings_with_default_namespace("com.amazonaws.dynamodb");
        let mut deser = JsonDeserializer::new(input, settings);
        let wrapper = deser.read_discriminated_document().expect("parse succeeds");

        assert_eq!(
            wrapper.discriminator(),
            Some("com.amazonaws.dynamodb#Capacity"),
            "relative __type must be resolved against default_namespace"
        );
        let map = wrapper.document().as_object().expect("top-level is map");
        assert!(
            !map.contains_key("__type"),
            "lifted __type must be removed from the map"
        );
        assert!(
            map.contains_key("CapacityUnits"),
            "non-discriminator keys remain"
        );
    }

    #[test]
    fn read_discriminated_document_absolute_overrides_default_namespace() {
        // An absolute `__type` (with `#`) is taken as-is even when a
        // default_namespace is set. Different namespaces in __type vs
        // settings is a real scenario for cross-namespace discriminators.
        let input = br#"{"__type":"smithy.example#Bird","name":"Iago"}"#;
        let settings = settings_with_default_namespace("com.amazonaws.dynamodb");
        let mut deser = JsonDeserializer::new(input, settings);
        let wrapper = deser.read_discriminated_document().expect("parse succeeds");

        assert_eq!(
            wrapper.discriminator(),
            Some("smithy.example#Bird"),
            "absolute __type must override default_namespace"
        );
    }

    #[test]
    fn read_discriminated_document_invalid_relative_name_stays_in_map() {
        // A `__type` value that contains characters illegal in a
        // Smithy identifier (e.g. whitespace, punctuation) must not
        // be synthesized into a malformed shape ID. The lift is
        // skipped and the value stays as a regular map entry.
        for invalid in ["foo bar", "foo-bar", "foo.bar", "1leading-digit", ""] {
            let body = format!(r#"{{"__type":"{invalid}"}}"#);
            let settings = settings_with_default_namespace("com.example");
            let mut deser = JsonDeserializer::new(body.as_bytes(), settings);
            let wrapper = deser.read_discriminated_document().expect("parse succeeds");
            assert!(
                wrapper.discriminator().is_none(),
                "invalid relative name {invalid:?} must not be lifted"
            );
            let map = wrapper.document().as_object().expect("top-level is map");
            assert!(
                map.contains_key("__type"),
                "invalid relative name {invalid:?} must remain in map"
            );
        }
    }

    #[test]
    fn read_discriminated_document_accepts_relative_names_with_underscore() {
        // Smithy identifiers allow leading underscore + digits in
        // subsequent positions. Cover edge cases.
        for valid in ["_Bird", "Bird1", "_a_b_c", "Foo_Bar"] {
            let body = format!(r#"{{"__type":"{valid}"}}"#);
            let settings = settings_with_default_namespace("ns");
            let mut deser = JsonDeserializer::new(body.as_bytes(), settings);
            let wrapper = deser.read_discriminated_document().expect("parse succeeds");
            assert_eq!(
                wrapper.discriminator(),
                Some(format!("ns#{valid}").as_str()),
                "valid Smithy identifier {valid:?} must be lifted"
            );
        }
    }
}

/// Deserialization tests for unions, focused on three cases that are easy to
/// get wrong in a schema-based deserializer:
///
/// 1. a union member whose value is **itself a union** (union-in-union),
/// 2. an explicitly-null union member (especially the first one), and
/// 3. an empty / all-null union object.
///
/// These mirror the generated union deserializer exactly: a `read_struct`
/// over a `ShapeType::Union` schema, a consumer that dispatches on
/// `member_index()`, and `ok_or_else(...)` when no variant was set. A
/// union-valued member recurses into the inner union's own `deserialize`,
/// which performs its own self-contained `read_struct`. Because
/// `JsonDeserializer::read_struct` consumes its own object braces and never
/// relies on shared "resume" state, the recursion is safe and a null / empty
/// union can neither panic nor corrupt the surrounding stream.
#[cfg(test)]
mod union_deserialization_tests {
    use super::*;
    use aws_smithy_schema::{shape_id, ShapeType};

    // --- union-in-union model ---
    //
    // Mirrors the real `TargetConfiguration -> mcp -> McpTargetConfiguration`
    // shape: an outer union whose `mcp` member targets an inner union whose
    // `lambda` member is a string.

    static INNER_LAMBDA: Schema<'static> = Schema::new_member(
        shape_id!("test", "InnerUnion"),
        ShapeType::String,
        "lambda",
        0,
    );
    static INNER_UNION_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "InnerUnion"),
        ShapeType::Union,
        &[&INNER_LAMBDA],
    );

    static OUTER_MCP: Schema<'static> =
        Schema::new_member(shape_id!("test", "OuterUnion"), ShapeType::Union, "mcp", 0);
    static OUTER_UNION_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "OuterUnion"),
        ShapeType::Union,
        &[&OUTER_MCP],
    );

    #[derive(Debug, PartialEq)]
    enum InnerUnion {
        Lambda(String),
        Unknown,
    }

    #[derive(Debug, PartialEq)]
    enum OuterUnion {
        Mcp(InnerUnion),
        Unknown,
    }

    fn deserialize_inner(deser: &mut dyn ShapeDeserializer) -> Result<InnerUnion, SerdeError> {
        let mut result: Option<InnerUnion> = None;
        deser.read_struct(&INNER_UNION_SCHEMA, &mut |member, d| {
            result = Some(match member.member_index() {
                Some(0) => InnerUnion::Lambda(d.read_string(member)?),
                _ => InnerUnion::Unknown,
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    fn deserialize_outer(deser: &mut dyn ShapeDeserializer) -> Result<OuterUnion, SerdeError> {
        let mut result: Option<OuterUnion> = None;
        deser.read_struct(&OUTER_UNION_SCHEMA, &mut |member, d| {
            result = Some(match member.member_index() {
                // The union-valued member recurses into the inner union's own
                // deserialize, which does its own self-contained read_struct.
                Some(0) => OuterUnion::Mcp(deserialize_inner(d)?),
                _ => OuterUnion::Unknown,
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    // --- explicit-null / empty / all-null union model ---

    static CHOICE_A: Schema<'static> =
        Schema::new_member(shape_id!("test", "Choice"), ShapeType::String, "a", 0);
    static CHOICE_B: Schema<'static> =
        Schema::new_member(shape_id!("test", "Choice"), ShapeType::String, "b", 1);
    static CHOICE_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "Choice"),
        ShapeType::Union,
        &[&CHOICE_A, &CHOICE_B],
    );

    #[derive(Debug, PartialEq)]
    enum Choice {
        A(String),
        B(String),
        Unknown,
    }

    fn deserialize_choice(deser: &mut dyn ShapeDeserializer) -> Result<Choice, SerdeError> {
        let mut result: Option<Choice> = None;
        deser.read_struct(&CHOICE_SCHEMA, &mut |member, d| {
            result = Some(match member.member_index() {
                Some(0) => Choice::A(d.read_string(member)?),
                Some(1) => Choice::B(d.read_string(member)?),
                _ => Choice::Unknown,
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    // A struct that holds a union followed by a plain string member. Used to
    // prove that reading the union does not corrupt the stream for the
    // trailing sibling.
    static HOLDER_CHOICE: Schema<'static> =
        Schema::new_member(shape_id!("test", "Holder"), ShapeType::Union, "choice", 0);
    static HOLDER_TRAILING: Schema<'static> = Schema::new_member(
        shape_id!("test", "Holder"),
        ShapeType::String,
        "trailing",
        1,
    );
    static HOLDER_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "Holder"),
        ShapeType::Structure,
        &[&HOLDER_CHOICE, &HOLDER_TRAILING],
    );

    fn deser(input: &[u8]) -> JsonDeserializer<'_> {
        JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()))
    }

    #[test]
    fn union_member_targeting_a_union_deserializes_without_panic() {
        // The exact shape class that crashed the Go SDK: a union member whose
        // value is itself a union.
        let mut d = deser(br#"{"mcp":{"lambda":"arn:aws:lambda:fn"}}"#);
        let got = deserialize_outer(&mut d).expect("union-in-union must deserialize");
        assert_eq!(
            got,
            OuterUnion::Mcp(InnerUnion::Lambda("arn:aws:lambda:fn".to_string()))
        );
    }

    #[test]
    fn explicit_null_first_union_member_resolves_later_variant_without_stream_corruption() {
        // The first union member is explicitly null; the real variant is the
        // second member. read_struct skips the null member and still consumes
        // the union's closing brace, so the trailing sibling of the enclosing
        // struct parses correctly (i.e. no stream corruption).
        let mut choice: Option<Choice> = None;
        let mut trailing: Option<String> = None;
        let mut d = deser(br#"{"choice":{"a":null,"b":"x"},"trailing":"ok"}"#);
        d.read_struct(&HOLDER_SCHEMA, &mut |member, sub| {
            match member.member_index() {
                Some(0) => choice = Some(deserialize_choice(sub)?),
                Some(1) => trailing = Some(sub.read_string(member)?),
                _ => {}
            }
            Ok(())
        })
        .expect("holder with a null-first-member union must parse");
        assert_eq!(choice, Some(Choice::B("x".to_string())));
        assert_eq!(
            trailing,
            Some("ok".to_string()),
            "trailing sibling must survive: reading the union must not corrupt the stream"
        );
    }

    #[test]
    fn empty_union_object_yields_clean_error_not_panic() {
        let mut d = deser(br#"{}"#);
        let err = deserialize_choice(&mut d).unwrap_err();
        assert!(
            err.to_string().contains("expected a union variant"),
            "empty union object must be a clean error, got {err:?}"
        );
    }

    #[test]
    fn all_null_union_members_yield_clean_error_not_panic() {
        let mut d = deser(br#"{"a":null,"b":null}"#);
        let err = deserialize_choice(&mut d).unwrap_err();
        assert!(
            err.to_string().contains("expected a union variant"),
            "all-null union must be a clean error, got {err:?}"
        );
    }

    #[test]
    fn list_of_union_in_union_deserializes() {
        // union-in-union nested inside a list element position.
        let mut d = deser(br#"[{"mcp":{"lambda":"a"}},{"mcp":{"lambda":"b"}}]"#);
        let mut out = Vec::new();
        d.read_list(&aws_smithy_schema::prelude::STRING, &mut |el| {
            out.push(deserialize_outer(el)?);
            Ok(())
        })
        .expect("list of union-in-union must deserialize");
        assert_eq!(
            out,
            vec![
                OuterUnion::Mcp(InnerUnion::Lambda("a".to_string())),
                OuterUnion::Mcp(InnerUnion::Lambda("b".to_string())),
            ]
        );
    }

    #[test]
    fn map_of_union_in_union_deserializes() {
        // union-in-union nested inside a map value position.
        let mut d = deser(br#"{"first":{"mcp":{"lambda":"a"}},"second":{"mcp":{"lambda":"b"}}}"#);
        let mut out = std::collections::HashMap::new();
        d.read_map(&aws_smithy_schema::prelude::STRING, &mut |k, v| {
            out.insert(k, deserialize_outer(v)?);
            Ok(())
        })
        .expect("map of union-in-union must deserialize");
        assert_eq!(
            out.get("first"),
            Some(&OuterUnion::Mcp(InnerUnion::Lambda("a".to_string())))
        );
        assert_eq!(
            out.get("second"),
            Some(&OuterUnion::Mcp(InnerUnion::Lambda("b".to_string())))
        );
    }
}

/// `read_struct` reports keys that name no member: the consumer is called with the
/// prelude `DOCUMENT` schema (no member index) positioned at the value. These tests
/// pin down what is reported, what is not, and that the stream stays intact whether
/// or not the consumer reads the value.
#[cfg(test)]
mod unknown_member_tests {
    use super::*;
    use aws_smithy_schema::{shape_id, ShapeType};

    static A: Schema<'static> =
        Schema::new_member(shape_id!("test", "S"), ShapeType::String, "a", 0);
    static S: Schema<'static> =
        Schema::new_struct(shape_id!("test", "S"), ShapeType::Structure, &[&A]);

    static U_INT: Schema<'static> =
        Schema::new_member(shape_id!("test", "U"), ShapeType::Integer, "int", 0);
    static U_STR: Schema<'static> =
        Schema::new_member(shape_id!("test", "U"), ShapeType::String, "string", 1);
    static U: Schema<'static> =
        Schema::new_struct(shape_id!("test", "U"), ShapeType::Union, &[&U_INT, &U_STR]);

    fn deser(input: &[u8]) -> JsonDeserializer<'_> {
        JsonDeserializer::new(input, Arc::new(JsonCodecSettings::default()))
    }

    /// Reads `S`, recording every unknown key (and its value when asked) and returning `a`.
    fn read_s(input: &[u8], read_unknown_value: bool) -> (Option<String>, Vec<Option<Document>>) {
        let mut a = None;
        let mut unknown = Vec::new();
        deser(input)
            .read_struct(&S, &mut |member, d| {
                match member.member_index() {
                    Some(0) => a = Some(d.read_string(member)?),
                    _ => {
                        assert_eq!(member.shape_type(), ShapeType::Document);
                        unknown.push(if read_unknown_value {
                            Some(d.read_document(member)?)
                        } else {
                            None
                        });
                    }
                }
                Ok(())
            })
            .unwrap();
        (a, unknown)
    }

    #[test]
    fn unknown_key_is_reported_and_the_consumer_can_read_the_value() {
        let (a, unknown) = read_s(br#"{"zzz":{"deep":[1,2]},"a":"x"}"#, true);
        assert_eq!(a.as_deref(), Some("x"));
        assert_eq!(unknown.len(), 1);
        match &unknown[0] {
            Some(Document::Object(map)) => assert!(map.contains_key("deep")),
            other => panic!("expected the object value, got {other:?}"),
        }
    }

    #[test]
    fn unknown_key_value_is_skipped_when_the_consumer_does_not_read_it() {
        // Nested containers and a `}` inside a string must not confuse the skip.
        let (a, unknown) = read_s(
            br#"{"zzz":{"deep":[1,{"q":"}"}]},"a":"x","yyy":"tail"}"#,
            false,
        );
        assert_eq!(a.as_deref(), Some("x"));
        assert_eq!(unknown.len(), 2);
    }

    #[test]
    fn type_discriminator_is_not_reported() {
        let (a, unknown) = read_s(br#"{"__type":"ns#Foo","a":"x"}"#, false);
        assert_eq!(a.as_deref(), Some("x"));
        assert!(unknown.is_empty(), "`__type` must not be reported");
    }

    #[test]
    fn null_valued_unknown_key_is_not_reported() {
        let (a, unknown) = read_s(br#"{"zzz":null,"a":"x"}"#, false);
        assert_eq!(a.as_deref(), Some("x"));
        assert!(unknown.is_empty(), "a null value is an absent member");
    }

    #[test]
    fn struct_consumer_that_ignores_unknown_keys_is_unaffected() {
        let mut a = None;
        deser(br#"{"zzz":[1,2,3],"a":"x"}"#)
            .read_struct(&S, &mut |member, d| {
                if member.member_index() == Some(0) {
                    a = Some(d.read_string(member)?);
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(a.as_deref(), Some("x"));
    }

    /// The shape of a generated server union deserializer: a second key of any kind is
    /// "mixed variants", and an unknown key is an error.
    fn read_union(input: &[u8]) -> Result<String, SerdeError> {
        let mut result: Option<String> = None;
        deser(input).read_struct(&U, &mut |member, d| {
            if result.is_some() {
                return Err(SerdeError::invalid_input(
                    "encountered mixed variants in union",
                ));
            }
            result = Some(match member.member_index() {
                Some(0) => format!("int={}", d.read_integer(member)?),
                Some(1) => format!("string={}", d.read_string(member)?),
                _ => return Err(SerdeError::invalid_input("unexpected union variant")),
            });
            Ok(())
        })?;
        result.ok_or_else(|| SerdeError::custom("expected a union variant"))
    }

    #[test]
    fn union_consumer_sees_known_and_unknown_keys() {
        assert_eq!(read_union(br#"{"int":2}"#).unwrap(), "int=2");
        let err = read_union(br#"{"int":2,"string":"three"}"#).unwrap_err();
        assert!(err.to_string().contains("mixed variants"), "{err}");
        let err = read_union(br#"{"int":2,"unknownField":"three"}"#).unwrap_err();
        assert!(err.to_string().contains("mixed variants"), "{err}");
        let err = read_union(br#"{"unknownField":"three"}"#).unwrap_err();
        assert!(
            err.to_string().contains("unexpected union variant"),
            "{err}"
        );
        // `__type` is a discriminator, not a variant.
        assert_eq!(
            read_union(br#"{"__type":"ns#U","int":2}"#).unwrap(),
            "int=2"
        );
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
        // With defaults, neither opt-in path is taken. Note the default for an
        // *integer* member is stricter on this branch than the truncating
        // behavior #4851 was written against: a floating-point value is now a
        // clean `TypeMismatch` rather than reading `1` and leaving `.0` in the
        // stream to surface as a confusing downstream parse error. See
        // `test_read_integer_rejects_float`. This still discriminates the
        // opt-in path, which returns `Ok(1)` for the same input.
        assert!(matches!(
            JsonDeserializer::new(b"1.0", Arc::new(JsonCodecSettings::default()))
                .read_long(dummy_schema()),
            Err(SerdeError::TypeMismatch { .. })
        ));
        // `read_double` is unaffected: a trailing-dot double still parses by default.
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

    fn dummy_schema() -> &'static Schema<'static> {
        &aws_smithy_schema::prelude::STRING
    }
}
