/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON serializer implementation.

use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::Schema;
use aws_smithy_types::date_time::Format as TimestampFormat;
use aws_smithy_types::primitive::Encoder;
use aws_smithy_types::{BigDecimal, BigInteger, DateTime, DiscriminatedDocument, Document};

use crate::codec::JsonCodecSettings;

use std::sync::Arc;

/// JSON serializer that implements the ShapeSerializer trait.
pub struct JsonSerializer {
    output: String,
    settings: Arc<JsonCodecSettings>,
    // Tracks whether a comma is needed before the next value in the current container.
    needs_comma: bool,
    // When true, the next write_string is a map key — emit "key": instead of ,"key"
    expecting_map_key: bool,
    // Nesting depth of write_map calls. When >0, prefix() restores expecting_map_key
    // after each value write so the next write_string is treated as a key.
    map_depth: usize,
    // True when the current container is a struct/union (so a member schema's
    // member_name should be emitted as a JSON field name). False when the
    // current container is a list or map (so a member schema's member_name —
    // typically the position name "member"/"value" — must NOT leak into the
    // output as a spurious field key). Saved and restored across nested
    // write_struct / write_list / write_map calls.
    in_struct_context: bool,
}

impl JsonSerializer {
    /// Creates a new JSON serializer with the given settings.
    pub(crate) fn new(settings: Arc<JsonCodecSettings>) -> Self {
        Self {
            output: String::new(),
            settings,
            needs_comma: false,
            expecting_map_key: false,
            map_depth: 0,
            // Top-level call sites typically pass a top-level (non-member)
            // schema, so this default value rarely matters. It only affects
            // behavior if the very first write_* call passes a member
            // schema — in which case treating the first frame as struct
            // context preserves backward compatibility with any caller
            // relying on `,"name":` emission at the top level.
            in_struct_context: true,
        }
    }

    /// Finalizes the serialization and returns the output as bytes.
    pub fn finish(self) -> Vec<u8> {
        self.output.into_bytes()
    }

    /// Handles comma separators and member names before writing a value.
    /// When inside a map (map_depth > 0), restores expecting_map_key after
    /// the value so the next write_string is treated as a map key.
    fn prefix(&mut self, schema: &Schema<'_>) {
        if self.needs_comma {
            self.output.push(',');
        }
        // Only emit a JSON field name when we are inside a struct/union
        // body. For list elements and map values the schema may still be a
        // member schema (e.g. an inner aggregate's `_VALUE` carrying
        // `@xmlName` for the XML codec), but that member's name is a
        // *position label* — not a JSON field key.
        if self.in_struct_context {
            if let Some(name) = self.field_name(schema) {
                self.output.push('"');
                self.output.push_str(&crate::escape::escape_string(name));
                self.output.push_str("\":");
            }
        }
        self.needs_comma = true;
        // Inside a map, after writing a value the next write_string should be a key.
        // This is safe because write_string checks expecting_map_key *before* calling
        // prefix(), so this only affects the *next* write_string call.
        if self.map_depth > 0 {
            self.expecting_map_key = true;
        }
    }

    /// Resolves the JSON field name for a member schema.
    fn field_name<'a>(&self, schema: &'a Schema<'a>) -> Option<&'a str> {
        self.settings.member_to_field(schema)
    }

    /// Gets the timestamp format to use, respecting @timestampFormat trait.
    fn get_timestamp_format(&self, schema: &Schema<'_>) -> TimestampFormat {
        if let Some(ts_trait) = schema.timestamp_format() {
            return match ts_trait.format() {
                aws_smithy_schema::traits::TimestampFormat::EpochSeconds => {
                    TimestampFormat::EpochSeconds
                }
                aws_smithy_schema::traits::TimestampFormat::HttpDate => TimestampFormat::HttpDate,
                aws_smithy_schema::traits::TimestampFormat::DateTime => TimestampFormat::DateTime,
            };
        }
        self.settings.default_timestamp_format
    }

    fn write_json_value(&mut self, doc: &Document) -> Result<(), SerdeError> {
        use crate::escape::escape_string;
        use crate::serialize::JsonValueWriter;
        use aws_smithy_types::base64;

        // Walk the [`aws_smithy_types::Document`] directly so blob
        // (base64), timestamp (codec-default format), and bignum
        // variants all serialize correctly.
        //
        // Per the SEP § Document Types rule 8, the document is
        // protocol-agnostic on the serialization side: it carries no
        // per-member format trait (the schema is the carrier of those
        // traits, and a `Document` value erases the schema). Thus
        // timestamp serialization here uses the codec's default format
        // unconditionally — schema-typed members go through the
        // dedicated `write_timestamp` path and inspect
        // `@timestampFormat` themselves, not via `write_document`.
        //
        // [`Document`] has no discriminator slot. Discriminated
        // documents (with a `__type` field) flow through
        // [`Self::write_discriminated_document`] which emits the
        // wrapper's discriminator before delegating here for the
        // payload.
        match doc {
            Document::Null => self.output.push_str("null"),
            Document::Bool(b) => self.output.push_str(if *b { "true" } else { "false" }),
            Document::Number(n) => {
                let writer = JsonValueWriter::new(&mut self.output);
                writer.number(*n);
            }
            Document::String(s) => {
                self.output.push('"');
                self.output.push_str(&escape_string(s));
                self.output.push('"');
            }
            Document::Blob(b) => {
                // base64 alphabet contains no JSON-special characters,
                // but route through `JsonValueWriter::string` to keep
                // the encoding centralized.
                let encoded = base64::encode(b);
                JsonValueWriter::new(&mut self.output).string(&encoded);
            }
            Document::Timestamp(ts) => {
                let format = self.settings.default_timestamp_format;
                JsonValueWriter::new(&mut self.output)
                    .date_time(ts, format)
                    .map_err(|e| {
                        SerdeError::write_failed(format!("failed to format timestamp: {e}"))
                    })?;
            }
            Document::BigInteger(bi) => {
                // Big integers serialize as raw JSON numbers (no quotes,
                // no scientific notation) so receivers with arbitrary-
                // precision parsers can recover the exact value.
                //
                // When [`JsonCodecSettings::use_string_for_arbitrary_precision`]
                // is set, emit as a JSON string instead — for interop
                // with receivers that route every JSON number through
                // `f64` and would otherwise lose precision on values
                // that overflow `f64`.
                if self.settings.use_string_for_arbitrary_precision() {
                    JsonValueWriter::new(&mut self.output).string(bi.as_ref());
                } else {
                    self.output.push_str(bi.as_ref());
                }
            }
            Document::BigDecimal(bd) => {
                if self.settings.use_string_for_arbitrary_precision() {
                    JsonValueWriter::new(&mut self.output).string(bd.as_ref());
                } else {
                    self.output.push_str(bd.as_ref());
                }
            }
            Document::Array(items) => {
                self.output.push('[');
                let mut first = true;
                for item in items {
                    if !first {
                        self.output.push(',');
                    }
                    first = false;
                    self.write_json_value(item)?;
                }
                self.output.push(']');
            }
            Document::Object(entries) => {
                self.output.push('{');
                let mut first = true;
                for (key, value) in entries {
                    if !first {
                        self.output.push(',');
                    }
                    first = false;
                    self.output.push('"');
                    self.output.push_str(&escape_string(key));
                    self.output.push_str("\":");
                    self.write_json_value(value)?;
                }
                self.output.push('}');
            }
            // `Document` is `#[non_exhaustive]`. Future variants need a
            // deliberate decision in this serializer rather than a
            // silent no-op.
            other => {
                return Err(SerdeError::custom(format!(
                    "JSON write_document: unsupported Document variant {other:?}"
                )));
            }
        }
        Ok(())
    }

    /// Serialize a [`DiscriminatedDocument`] to the codec's output.
    ///
    /// When the wrapper carries a discriminator and the inner Document
    /// is a [`Document::Object`], the discriminator is emitted as a
    /// `__type` field at the start of the JSON object per the SEP
    /// § Typed Document Serialization (always serialized as an
    /// absolute shape ID, the FQN form `namespace#name`).
    ///
    /// For non-object inner documents, or when no discriminator is
    /// attached, this delegates to the plain JSON serialization path.
    pub fn write_discriminated_document(
        &mut self,
        schema: &Schema<'_>,
        value: &DiscriminatedDocument,
    ) -> Result<(), SerdeError> {
        use crate::escape::escape_string;

        self.prefix(schema);
        match (value.discriminator(), value.document()) {
            (Some(id), Document::Object(entries)) => {
                // SEP § Typed Document Serialization: a serialized
                // discriminator MUST be an absolute shape id so the
                // document stays context-free for downstream readers.
                // `DiscriminatedDocument::with_discriminator` enforces
                // this at construction; this is a defense-in-depth
                // check at the wire boundary for any document produced
                // internally.
                debug_assert!(
                    id.contains('#'),
                    "discriminator `{id}` must be an absolute shape id (namespace#name) when serialized"
                );
                self.output.push('{');
                self.output.push_str("\"__type\":\"");
                self.output.push_str(&escape_string(id));
                self.output.push('"');
                for (key, val) in entries {
                    self.output.push(',');
                    self.output.push('"');
                    self.output.push_str(&escape_string(key));
                    self.output.push_str("\":");
                    self.write_json_value(val)?;
                }
                self.output.push('}');
                Ok(())
            }
            // No discriminator, or discriminator on a non-object payload
            // — emit the inner document plainly.
            (_, doc) => self.write_json_value(doc),
        }
    }
}

impl aws_smithy_schema::codec::FinishSerializer for JsonSerializer {
    fn finish(self) -> Vec<u8> {
        self.output.into_bytes()
    }
}

impl ShapeSerializer for JsonSerializer {
    fn write_struct(
        &mut self,
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push('{');
        let saved_comma = self.needs_comma;
        let saved_depth = self.map_depth;
        let saved_map_key = self.expecting_map_key;
        let saved_struct_context = self.in_struct_context;
        self.needs_comma = false;
        // Reset map state so struct members don't trigger map-key logic.
        // Restored after the struct body so an enclosing map resumes correctly.
        self.map_depth = 0;
        self.expecting_map_key = false;
        // Mark the body as struct context so member-schema field names emit.
        self.in_struct_context = true;
        value.serialize_members(self)?;
        self.output.push('}');
        self.needs_comma = saved_comma;
        self.map_depth = saved_depth;
        self.expecting_map_key = saved_map_key;
        self.in_struct_context = saved_struct_context;
        Ok(())
    }

    fn write_list(
        &mut self,
        schema: &Schema<'_>,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push('[');
        let saved = self.needs_comma;
        let saved_depth = self.map_depth;
        let saved_map_key = self.expecting_map_key;
        let saved_struct_context = self.in_struct_context;
        self.needs_comma = false;
        // Reset map state so list elements don't trigger map-key logic in prefix().
        self.map_depth = 0;
        self.expecting_map_key = false;
        // Mark the body as non-struct so a nested member schema's
        // member_name (e.g. "member") doesn't leak as a JSON field key.
        self.in_struct_context = false;
        write_elements(self)?;
        self.output.push(']');
        self.needs_comma = saved;
        self.map_depth = saved_depth;
        self.expecting_map_key = saved_map_key;
        self.in_struct_context = saved_struct_context;
        Ok(())
    }

    fn write_map(
        &mut self,
        schema: &Schema<'_>,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push('{');
        let saved_comma = self.needs_comma;
        let saved_map_key = self.expecting_map_key;
        let saved_depth = self.map_depth;
        let saved_struct_context = self.in_struct_context;
        self.needs_comma = false;
        self.expecting_map_key = true;
        // Increment depth so prefix() knows to restore expecting_map_key after
        // each value write. write_string checks expecting_map_key *before* calling
        // prefix(), so the flag only affects the *next* write_string (the next key).
        self.map_depth += 1;
        // Mark the body as non-struct so an inner aggregate's value-schema
        // (a member schema with member_name "value") doesn't emit
        // `"value":` before the actual value.
        self.in_struct_context = false;
        write_entries(self)?;
        self.map_depth = saved_depth;
        self.output.push('}');
        self.needs_comma = saved_comma;
        self.expecting_map_key = saved_map_key;
        self.in_struct_context = saved_struct_context;
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        use std::fmt::Write;
        self.prefix(schema);
        write!(&mut self.output, "{}", value).map_err(|e| SerdeError::write_failed(e.to_string()))
    }

    fn write_short(&mut self, schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        use std::fmt::Write;
        self.prefix(schema);
        write!(&mut self.output, "{}", value).map_err(|e| SerdeError::write_failed(e.to_string()))
    }

    fn write_integer(&mut self, schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        use std::fmt::Write;
        self.prefix(schema);
        write!(&mut self.output, "{}", value).map_err(|e| SerdeError::write_failed(e.to_string()))
    }

    fn write_long(&mut self, schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        use std::fmt::Write;
        self.prefix(schema);
        write!(&mut self.output, "{}", value).map_err(|e| SerdeError::write_failed(e.to_string()))
    }

    fn write_float(&mut self, schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        self.prefix(schema);
        if value.is_nan() {
            self.output.push_str("\"NaN\"");
            Ok(())
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                self.output.push_str("\"Infinity\"");
            } else {
                self.output.push_str("\"-Infinity\"");
            }
            Ok(())
        } else {
            self.output.push_str(Encoder::from(value).encode());
            Ok(())
        }
    }

    fn write_double(&mut self, schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        self.prefix(schema);
        if value.is_nan() {
            self.output.push_str("\"NaN\"");
            Ok(())
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                self.output.push_str("\"Infinity\"");
            } else {
                self.output.push_str("\"-Infinity\"");
            }
            Ok(())
        } else {
            self.output.push_str(Encoder::from(value).encode());
            Ok(())
        }
    }

    fn write_big_integer(
        &mut self,
        schema: &Schema<'_>,
        value: &BigInteger,
    ) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push_str(value.as_ref());
        Ok(())
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema<'_>,
        value: &BigDecimal,
    ) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push_str(value.as_ref());
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        use crate::escape::escape_string;
        if self.expecting_map_key {
            // Map key: comma before (if not first entry), then "key":
            if self.needs_comma {
                self.output.push(',');
            }
            self.output.push('"');
            self.output.push_str(&escape_string(value));
            self.output.push_str("\":");
            // The next write is the value — no comma before it
            self.needs_comma = false;
            self.expecting_map_key = false;
        } else {
            self.prefix(schema);
            self.output.push('"');
            self.output.push_str(&escape_string(value));
            self.output.push('"');
        }
        Ok(())
    }

    fn write_blob(
        &mut self,
        schema: &Schema<'_>,
        value: aws_smithy_types::Blob,
    ) -> Result<(), SerdeError> {
        use aws_smithy_types::base64;
        self.prefix(schema);
        let encoded = base64::encode(value.as_ref());
        self.output.push('"');
        self.output.push_str(&encoded);
        self.output.push('"');
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema<'_>, value: &DateTime) -> Result<(), SerdeError> {
        self.prefix(schema);
        let format = self.get_timestamp_format(schema);
        let formatted = value
            .fmt(format)
            .map_err(|e| SerdeError::write_failed(format!("failed to format timestamp: {e}")))?;

        match format {
            TimestampFormat::EpochSeconds => {
                // Epoch seconds as number
                self.output.push_str(&formatted);
            }
            _ => {
                // Other formats as strings
                self.output.push('"');
                self.output.push_str(&formatted);
                self.output.push('"');
            }
        }
        Ok(())
    }

    fn write_document(&mut self, schema: &Schema<'_>, value: &Document) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.write_json_value(value)?;
        Ok(())
    }

    fn write_null(&mut self, schema: &Schema<'_>) -> Result<(), SerdeError> {
        self.prefix(schema);
        self.output.push_str("null");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::prelude::*;
    use aws_smithy_schema::ShapeType;

    #[test]
    fn test_write_boolean() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_boolean(&BOOLEAN, true).unwrap();
        let output = ser.finish();
        assert_eq!(String::from_utf8(output).unwrap(), "true");
    }

    #[test]
    fn test_write_string() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_string(&STRING, "hello").unwrap();
        let output = ser.finish();
        assert_eq!(String::from_utf8(output).unwrap(), "\"hello\"");
    }

    #[test]
    fn test_write_integer() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_integer(&INTEGER, 42).unwrap();
        let output = ser.finish();
        assert_eq!(String::from_utf8(output).unwrap(), "42");
    }

    #[test]
    fn floats_preserve_decimal_points_and_original_precision() {
        let list_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "Floats"),
            ShapeType::List,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_list(&list_schema, &|s| {
            s.write_float(&FLOAT, 1.0)?;
            s.write_double(&DOUBLE, 1.0)?;
            s.write_float(&FLOAT, 0.0)?;
            s.write_double(&DOUBLE, 0.0)?;
            s.write_float(&FLOAT, -0.0)?;
            s.write_double(&DOUBLE, -0.0)?;
            s.write_float(&FLOAT, 1.01)?;
            s.write_double(&DOUBLE, 1.01)?;
            s.write_float(&FLOAT, 3.15)?;
            s.write_double(&DOUBLE, 3.15)?;
            s.write_float(&FLOAT, f32::NAN)?;
            s.write_double(&DOUBLE, f64::NAN)?;
            s.write_float(&FLOAT, f32::INFINITY)?;
            s.write_double(&DOUBLE, f64::INFINITY)?;
            s.write_float(&FLOAT, f32::NEG_INFINITY)?;
            s.write_double(&DOUBLE, f64::NEG_INFINITY)?;
            Ok(())
        })
        .unwrap();
        // Compare wire bytes: parsing JSON would hide the missing decimal points.
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            r#"[1.0,1.0,0.0,0.0,-0.0,-0.0,1.01,1.01,3.15,3.15,"NaN","NaN","Infinity","Infinity","-Infinity","-Infinity"]"#
        );
    }

    #[test]
    fn test_write_null() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_null(&STRING).unwrap();
        let output = ser.finish();
        assert_eq!(String::from_utf8(output).unwrap(), "null");
    }

    #[test]
    fn write_discriminated_document_emits_absolute_type() {
        let mut obj = aws_smithy_types::document::DocumentObject::new();
        obj.insert(
            "greeting".to_string(),
            Document::String("hello".to_string()),
        );
        let dd = DiscriminatedDocument::new(Document::Object(obj))
            .with_discriminator("com.example#Greeting");
        let schema = aws_smithy_schema::Schema::new(
            aws_smithy_schema::shape_id!("com.example", "Greeting"),
            ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_discriminated_document(&schema, &dd).unwrap();
        let out = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(
            out,
            r#"{"__type":"com.example#Greeting","greeting":"hello"}"#
        );
    }

    // A relative discriminator constructed via the test-only unchecked
    // setter (bypassing the absolute-FQN check in
    // `with_discriminator`) must still be caught at the wire-write site
    // in debug builds.
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "absolute shape id")]
    fn write_discriminated_document_rejects_relative_discriminator() {
        let mut obj = aws_smithy_types::document::DocumentObject::new();
        obj.insert(
            "greeting".to_string(),
            Document::String("hello".to_string()),
        );
        let mut dd = DiscriminatedDocument::new(Document::Object(obj));
        dd.set_discriminator_unchecked("Greeting");
        let schema = aws_smithy_schema::Schema::new(
            aws_smithy_schema::shape_id!("com.example", "Greeting"),
            ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        let _ = ser.write_discriminated_document(&schema, &dd);
    }

    #[test]
    fn test_write_list() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        let list_schema = aws_smithy_schema::Schema::new(
            aws_smithy_schema::shape_id!("test", "List"),
            aws_smithy_schema::ShapeType::List,
        );
        ser.write_list(&list_schema, &|s: &mut dyn ShapeSerializer| {
            s.write_integer(&INTEGER, 1)?;
            s.write_integer(&INTEGER, 2)?;
            s.write_integer(&INTEGER, 3)?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, "[1,2,3]");
    }

    #[test]
    fn test_write_full_object() {
        use aws_smithy_schema::serde::SerializableStruct;

        static ACTIVE_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::Boolean,
            "active",
            0,
        );
        static NAME_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::String,
            "name",
            1,
        );
        static COUNT_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::Integer,
            "count",
            2,
        );
        static PRICE_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::Float,
            "price",
            3,
        );
        static ITEMS_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::List,
            "items",
            4,
        );

        struct TestObject;
        impl SerializableStruct for TestObject {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_boolean(&ACTIVE_MEMBER, true)?;
                s.write_string(&NAME_MEMBER, "test")?;
                s.write_integer(&COUNT_MEMBER, 42)?;
                s.write_float(&PRICE_MEMBER, 3.15)?;
                s.write_list(&ITEMS_MEMBER, &|s| {
                    s.write_integer(&INTEGER, 1)?;
                    s.write_integer(&INTEGER, 2)?;
                    Ok(())
                })?;
                Ok(())
            }
        }

        let struct_schema = aws_smithy_schema::Schema::new(
            aws_smithy_schema::shape_id!("test", "Struct"),
            aws_smithy_schema::ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&struct_schema, &TestObject).unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(
            output,
            r#"{"active":true,"name":"test","count":42,"price":3.15,"items":[1,2]}"#
        );
    }

    #[test]
    fn test_nested_complex_serialization() {
        use aws_smithy_schema::serde::SerializableStruct;

        // Member schemas
        static ID: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Long,
            "id",
            0,
        );
        static NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::String,
            "name",
            1,
        );
        static SCORES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "scores",
            2,
        );
        static ADDRESS: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Structure,
            "address",
            3,
        );
        static COMPANIES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::List,
            "companies",
            4,
        );
        static TAGS: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Map,
            "tags",
            5,
        );
        static STREET: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "street",
            0,
        );
        static CITY: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::String,
            "city",
            1,
        );
        static ZIP: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Address"),
            aws_smithy_schema::ShapeType::Integer,
            "zip",
            2,
        );
        static COMP_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::String,
            "name",
            0,
        );
        static EMPLOYEES: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::List,
            "employees",
            1,
        );
        static METADATA: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Map,
            "metadata",
            2,
        );
        static ACTIVE: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Company"),
            aws_smithy_schema::ShapeType::Boolean,
            "active",
            3,
        );

        struct AddressStruct;
        impl SerializableStruct for AddressStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&STREET, "123 Main St")?;
                s.write_string(&CITY, "Seattle")?;
                s.write_integer(&ZIP, 98101)?;
                Ok(())
            }
        }

        struct CompanyStruct {
            name: &'static str,
            employees: &'static [&'static str],
            metadata: &'static [(&'static str, i32)],
            active: bool,
        }
        impl SerializableStruct for CompanyStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&COMP_NAME, self.name)?;
                s.write_list(&EMPLOYEES, &|s| {
                    for e in self.employees {
                        s.write_string(&STRING, e)?;
                    }
                    Ok(())
                })?;
                s.write_map(&METADATA, &|s| {
                    for (k, v) in self.metadata {
                        s.write_string(&::aws_smithy_schema::prelude::STRING, k)?;
                        s.write_integer(&::aws_smithy_schema::prelude::INTEGER, *v)?;
                    }
                    Ok(())
                })?;
                s.write_boolean(&ACTIVE, self.active)?;
                Ok(())
            }
        }

        struct UserStruct;
        impl SerializableStruct for UserStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_long(&ID, 12345)?;
                s.write_string(&NAME, "John Doe")?;
                s.write_list(&SCORES, &|s| {
                    s.write_double(&DOUBLE, 95.5)?;
                    s.write_double(&DOUBLE, 87.3)?;
                    s.write_double(&DOUBLE, 92.1)?;
                    Ok(())
                })?;
                s.write_struct(&ADDRESS, &AddressStruct)?;
                s.write_list(&COMPANIES, &|s| {
                    let struct_schema = Schema::new(
                        aws_smithy_schema::shape_id!("test", "Company"),
                        aws_smithy_schema::ShapeType::Structure,
                    );
                    s.write_struct(
                        &struct_schema,
                        &CompanyStruct {
                            name: "TechCorp",
                            employees: &["Alice", "Bob"],
                            metadata: &[("founded", 2010), ("size", 500)],
                            active: true,
                        },
                    )?;
                    s.write_struct(
                        &struct_schema,
                        &CompanyStruct {
                            name: "StartupInc",
                            employees: &["Charlie"],
                            metadata: &[("founded", 2020)],
                            active: false,
                        },
                    )?;
                    Ok(())
                })?;
                s.write_map(&TAGS, &|s| {
                    s.write_string(&::aws_smithy_schema::prelude::STRING, "role")?;
                    s.write_string(&::aws_smithy_schema::prelude::STRING, "admin")?;
                    s.write_string(&::aws_smithy_schema::prelude::STRING, "level")?;
                    s.write_string(&::aws_smithy_schema::prelude::STRING, "senior")?;
                    Ok(())
                })?;
                Ok(())
            }
        }

        let struct_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "User"),
            aws_smithy_schema::ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&struct_schema, &UserStruct).unwrap();
        let output = ser.finish();
        // Expected compact JSON (br# avoids escape noise)
        let expected: &[u8] = br#"{"id":12345,"name":"John Doe","scores":[95.5,87.3,92.1],"address":{"street":"123 Main St","city":"Seattle","zip":98101},"companies":[{"name":"TechCorp","employees":["Alice","Bob"],"metadata":{"founded":2010,"size":500},"active":true},{"name":"StartupInc","employees":["Charlie"],"metadata":{"founded":2020},"active":false}],"tags":{"role":"admin","level":"senior"}}"#;
        assert_eq!(output, expected);
    }

    #[test]
    fn test_json_name_serialization() {
        use aws_smithy_schema::serde::SerializableStruct;

        static FOO_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::String,
            "foo",
            0,
        );
        // bar has @jsonName("Baz")
        static BAR_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Integer,
            "bar",
            1,
        )
        .with_json_name("Baz");

        struct TestStruct;
        impl SerializableStruct for TestStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&FOO_MEMBER, "hello")?;
                s.write_integer(&BAR_MEMBER, 42)?;
                Ok(())
            }
        }

        // With use_json_name=true (default), "bar" should serialize as "Baz"
        let struct_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&struct_schema, &TestStruct).unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"foo":"hello","Baz":42}"#);

        // With use_json_name=false, "bar" should stay as "bar"
        let mut ser = JsonSerializer::new(Arc::new(
            JsonCodecSettings::builder().use_json_name(false).build(),
        ));
        ser.write_struct(&struct_schema, &TestStruct).unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"foo":"hello","bar":42}"#);
    }

    /// The same `@jsonName` behaviour, but driven by a schema materialized at
    /// runtime from strings a local arena owns rather than by codegen-emitted
    /// `'static` literals. This is the property the Serialization and Schema
    /// Decoupling SEP requires: schema-driven serde must work identically
    /// regardless of where the schema came from.
    #[test]
    fn test_json_name_serialization_from_runtime_schema() {
        use aws_smithy_schema::serde::SerializableStruct;

        // Arena the caller owns. Nothing is 'static; nothing is leaked.
        let arena: Vec<String> = vec![
            String::from("bar"),        // member name
            String::from("RuntimeBaz"), // @jsonName
        ];

        let bar_member: Schema<'_> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Integer,
            &arena[0],
            0,
        )
        .with_json_name(&arena[1]);

        struct RuntimeStruct<'a>(&'a Schema<'a>);
        impl SerializableStruct for RuntimeStruct<'_> {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_integer(self.0, 42)
            }
        }

        let struct_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "MyStruct"),
            aws_smithy_schema::ShapeType::Structure,
        );

        // The runtime @jsonName must win over the runtime member name.
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&struct_schema, &RuntimeStruct(&bar_member))
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"RuntimeBaz":42}"#);

        // And the opt-out still falls back to the runtime member name.
        let mut ser = JsonSerializer::new(Arc::new(
            JsonCodecSettings::builder().use_json_name(false).build(),
        ));
        ser.write_struct(&struct_schema, &RuntimeStruct(&bar_member))
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"bar":42}"#);
    }

    #[test]
    fn struct_inside_map_serializes_member_names_correctly() {
        // Regression test: when a struct is a map value, the map's expecting_map_key
        // flag must not leak into the struct's member serialization.
        use aws_smithy_schema::serde::{SerializableStruct, ShapeSerializer};

        static INNER_NAME: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Inner"),
            ShapeType::String,
            "name",
            0,
        );
        static INNER_MEMBERS: &[&Schema<'_>] = &[&INNER_NAME];
        static INNER_SCHEMA: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Inner"),
            ShapeType::Structure,
            INNER_MEMBERS,
        );

        struct Inner;
        impl SerializableStruct for Inner {
            fn serialize_members(
                &self,
                ser: &mut dyn ShapeSerializer,
            ) -> Result<(), aws_smithy_schema::serde::SerdeError> {
                ser.write_string(&INNER_NAME, "Alice")
            }
        }

        static MAP_SCHEMA: Schema<'static> = Schema::new(
            aws_smithy_schema::shape_id!("test", "MyMap"),
            ShapeType::Map,
        );

        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_map(&MAP_SCHEMA, &|ser| {
            ser.write_string(&aws_smithy_schema::prelude::STRING, "key1")?;
            ser.write_struct(&INNER_SCHEMA, &Inner)?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"key1":{"name":"Alice"}}"#);
    }

    /// Regression test: when a member schema (e.g. one with
    /// `member_name = "value"`) is passed to `write_map` from inside
    /// another map's value position, the inner `write_map` call must NOT
    /// emit `"value":` before the inner `{...}`.
    ///
    /// Without context-aware prefix suppression, the output would be
    /// `{"outer_k":"value":{"inner_k":"inner_v"}}` (invalid JSON).
    /// With the fix, output is `{"outer_k":{"inner_k":"inner_v"}}`.
    #[test]
    fn map_value_with_value_member_schema_does_not_leak_member_name() {
        static OUTER_VALUE: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "OuterMap", "value"),
            ShapeType::Map,
            "value",
            1,
        );

        let outer_map_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "OuterMap"),
            ShapeType::Map,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_map(&outer_map_schema, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "outer_k")?;
            s.write_map(&OUTER_VALUE, &|s| {
                s.write_string(&aws_smithy_schema::prelude::STRING, "inner_k")?;
                s.write_string(&aws_smithy_schema::prelude::STRING, "inner_v")?;
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"outer_k":{"inner_k":"inner_v"}}"#);
    }

    /// Regression test mirroring the map-of-map case for list-of-map: the
    /// inner map is reached via a member schema whose member name is
    /// "member" (the list's element position). That member name must not
    /// leak into the JSON output.
    #[test]
    fn list_element_with_member_member_schema_does_not_leak_member_name() {
        static OUTER_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "OuterList", "member"),
            ShapeType::Map,
            "member",
            0,
        );

        let outer_list_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "OuterList"),
            ShapeType::List,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_list(&outer_list_schema, &|s| {
            s.write_map(&OUTER_MEMBER, &|s| {
                s.write_string(&aws_smithy_schema::prelude::STRING, "k1")?;
                s.write_string(&aws_smithy_schema::prelude::STRING, "v1")?;
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"[{"k1":"v1"}]"#);
    }

    /// List-of-list with a member schema for the inner list element.
    /// Verifies the suppression also applies to nested `write_list`.
    #[test]
    fn nested_list_with_member_member_schema_does_not_leak_member_name() {
        static OUTER_MEMBER: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "OuterList", "member"),
            ShapeType::List,
            "member",
            0,
        );

        let outer_list_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "OuterList"),
            ShapeType::List,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_list(&outer_list_schema, &|s| {
            s.write_list(&OUTER_MEMBER, &|s| {
                s.write_integer(&aws_smithy_schema::prelude::INTEGER, 1)?;
                s.write_integer(&aws_smithy_schema::prelude::INTEGER, 2)?;
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"[[1,2]]"#);
    }

    /// Positive control: the parent struct context is correctly restored
    /// after a non-struct child exits. A struct with `[string, map, string]`
    /// members must emit field names for ALL three — the suppression
    /// applied while writing the map's body must not bleed into the
    /// sibling string member that follows.
    #[test]
    fn struct_context_restored_after_nested_aggregate_member() {
        static A: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Outer"),
            ShapeType::String,
            "a",
            0,
        );
        static B: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Outer"),
            ShapeType::Map,
            "b",
            1,
        );
        static C: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Outer"),
            ShapeType::String,
            "c",
            2,
        );

        struct Outer;
        impl SerializableStruct for Outer {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&A, "1")?;
                s.write_map(&B, &|s| {
                    s.write_string(&aws_smithy_schema::prelude::STRING, "k")?;
                    s.write_string(&aws_smithy_schema::prelude::STRING, "v")?;
                    Ok(())
                })?;
                s.write_string(&C, "2")?;
                Ok(())
            }
        }

        let outer_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "Outer"),
            ShapeType::Structure,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&outer_schema, &Outer).unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"a":"1","b":{"k":"v"},"c":"2"}"#);
    }

    /// `@jsonName` is part of the same `field_name(schema)` resolution
    /// pipeline that emits `member_name`, so context-aware suppression
    /// must apply equally to it. A member schema in non-struct context
    /// with `@jsonName` set must NOT emit the JSON name as a key.
    #[test]
    fn map_value_with_json_name_member_schema_does_not_leak_json_name() {
        static OUTER_VALUE: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "OuterMap", "value"),
            ShapeType::String,
            "value",
            1,
        )
        .with_json_name("CustomName");

        let outer_map_schema = Schema::new(
            aws_smithy_schema::shape_id!("test", "OuterMap"),
            ShapeType::Map,
        );
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_map(&outer_map_schema, &|s| {
            s.write_string(&aws_smithy_schema::prelude::STRING, "k")?;
            s.write_string(&OUTER_VALUE, "v")?;
            Ok(())
        })
        .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, r#"{"k":"v"}"#);
    }

    // --- write_document gates BigInteger / BigDecimal on use_string_for_arbitrary_precision -----

    fn settings_with_string_for_arbitrary_precision(value: bool) -> Arc<JsonCodecSettings> {
        Arc::new(
            JsonCodecSettings::builder()
                .use_string_for_arbitrary_precision(value)
                .build(),
        )
    }

    #[test]
    fn write_document_emits_big_integer_as_number_by_default() {
        // Default (`use_string_for_arbitrary_precision = false`) emits
        // BigInteger as a raw JSON number — interoperable with arbitrary-
        // precision parsers.
        use std::str::FromStr;
        let bi = BigInteger::from_str("99999999999999999999999").unwrap();
        let doc = Document::BigInteger(bi);
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_document(&aws_smithy_schema::prelude::DOCUMENT, &doc)
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, "99999999999999999999999");
    }

    #[test]
    fn write_document_emits_big_integer_as_string_when_setting_enabled() {
        use std::str::FromStr;
        let bi = BigInteger::from_str("99999999999999999999999").unwrap();
        let doc = Document::BigInteger(bi);
        let mut ser = JsonSerializer::new(settings_with_string_for_arbitrary_precision(true));
        ser.write_document(&aws_smithy_schema::prelude::DOCUMENT, &doc)
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(output, "\"99999999999999999999999\"");
    }

    #[test]
    fn write_document_emits_big_decimal_as_number_by_default() {
        use std::str::FromStr;
        let bd = BigDecimal::from_str("1.234e500").unwrap();
        let doc = Document::BigDecimal(bd);
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_document(&aws_smithy_schema::prelude::DOCUMENT, &doc)
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        // Raw JSON number — no quotes. Exact spelling depends on
        // BigDecimal's normalization, so just assert "no quotes" and
        // that it's not the JSON `null` token.
        assert!(
            !output.starts_with('"'),
            "expected raw number, got {output}"
        );
        assert_ne!(output, "null");
    }

    #[test]
    fn write_document_emits_big_decimal_as_string_when_setting_enabled() {
        use std::str::FromStr;
        let bd = BigDecimal::from_str("1.234e500").unwrap();
        let doc = Document::BigDecimal(bd);
        let mut ser = JsonSerializer::new(settings_with_string_for_arbitrary_precision(true));
        ser.write_document(&aws_smithy_schema::prelude::DOCUMENT, &doc)
            .unwrap();
        let output = String::from_utf8(ser.finish()).unwrap();
        assert!(
            output.starts_with('"'),
            "expected quoted form, got {output}"
        );
        assert!(output.ends_with('"'), "expected quoted form, got {output}");
    }

    // --- Required value-type members are serialized even when the value equals
    // the type's zero/default (e.g. bool `false`). This mirrors the generated
    // non-optional branch `{ let val = &self.x; ser.write_boolean(..) }` (always
    // written, never a skip-if-default), and shapes the ELB
    // `LoadBalancerAttributes { ConnectionDraining { Enabled = false } }` case:
    // the nested structure member is present and `enabled=false` appears.

    static CD_ENABLED: Schema = Schema::new_member(
        aws_smithy_schema::shape_id!("test", "ConnectionDraining$enabled"),
        ShapeType::Boolean,
        "enabled",
        0,
    );
    static LBA_CD: Schema = Schema::new_member(
        aws_smithy_schema::shape_id!("test", "LoadBalancerAttributes$connectionDraining"),
        ShapeType::Structure,
        "connectionDraining",
        0,
    );
    static LBA_SCHEMA: Schema = Schema::new_struct(
        aws_smithy_schema::shape_id!("test", "LoadBalancerAttributes"),
        ShapeType::Structure,
        &[&LBA_CD],
    );

    struct ConnectionDraining {
        enabled: bool,
    }
    impl SerializableStruct for ConnectionDraining {
        fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            // Non-optional value-type member: unconditional write (mirrors codegen).
            let val = &self.enabled;
            ser.write_boolean(&CD_ENABLED, *val)
        }
    }
    struct LoadBalancerAttributes {
        connection_draining: ConnectionDraining,
    }
    impl SerializableStruct for LoadBalancerAttributes {
        fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            ser.write_struct(&LBA_CD, &self.connection_draining)
        }
    }

    #[test]
    fn required_value_type_false_bool_is_serialized() {
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(
            &LBA_SCHEMA,
            &LoadBalancerAttributes {
                connection_draining: ConnectionDraining { enabled: false },
            },
        )
        .unwrap();
        let out = String::from_utf8(ser.finish()).unwrap();
        assert_eq!(
            out, r#"{"connectionDraining":{"enabled":false}}"#,
            "the required-default value-type member must be present with false, not dropped"
        );
    }

    // Tagged-union serialization parity: a union serializes as a single-entry
    // object keyed by the active variant name (`{"variantName": value}`), the
    // same logical shape the CBOR serializer produces. Covers a scalar variant,
    // a struct variant, and two union members in one input.
    #[test]
    fn tagged_union_serializes_as_single_variant_key() {
        static EP_ARN: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Endpoint"),
            ShapeType::String,
            "arn",
            0,
        );
        static EP_CONFIG: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Endpoint"),
            ShapeType::Structure,
            "config",
            1,
        );
        static ENDPOINT_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Endpoint"),
            ShapeType::Union,
            &[&EP_ARN, &EP_CONFIG],
        );
        static EC_NAME: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "EndpointConfig"),
            ShapeType::String,
            "name",
            0,
        );
        static CC_ATTACH: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "CreateConnectionInput"),
            ShapeType::Union,
            "attachPoint",
            0,
        );
        static CC_REMOTE: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "CreateConnectionInput"),
            ShapeType::Union,
            "remoteAccount",
            1,
        );
        static INPUT_SCHEMA: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "CreateConnectionInput"),
            ShapeType::Structure,
            &[&CC_ATTACH, &CC_REMOTE],
        );

        struct EndpointConfig {
            name: &'static str,
        }
        impl SerializableStruct for EndpointConfig {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&EC_NAME, self.name)
            }
        }
        enum Endpoint {
            Arn(&'static str),
            Config(EndpointConfig),
        }
        impl SerializableStruct for Endpoint {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                match self {
                    Endpoint::Arn(v) => s.write_string(&EP_ARN, v),
                    Endpoint::Config(v) => s.write_struct(&EP_CONFIG, v),
                }
            }
        }
        struct Input {
            attach: Endpoint,
            remote: Endpoint,
        }
        impl SerializableStruct for Input {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_struct(&CC_ATTACH, &self.attach)?;
                s.write_struct(&CC_REMOTE, &self.remote)
            }
        }

        // Scalar variant at top level.
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(&ENDPOINT_SCHEMA, &Endpoint::Arn("dxcon-abc"))
            .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            r#"{"arn":"dxcon-abc"}"#
        );

        // Two union members: scalar + struct variant.
        let mut ser = JsonSerializer::new(Arc::new(JsonCodecSettings::default()));
        ser.write_struct(
            &INPUT_SCHEMA,
            &Input {
                attach: Endpoint::Arn("ap"),
                remote: Endpoint::Config(EndpointConfig { name: "ra" }),
            },
        )
        .unwrap();
        assert_eq!(
            String::from_utf8(ser.finish()).unwrap(),
            r#"{"attachPoint":{"arn":"ap"},"remoteAccount":{"config":{"name":"ra"}}}"#
        );
    }
}
