/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON codec implementation for schema-based serialization.

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::Schema;
use aws_smithy_types::date_time::Format as TimestampFormat;
use std::sync::Arc;

mod deserializer;
mod serializer;

pub use deserializer::JsonDeserializer;
pub use serializer::JsonSerializer;

/// Maps between Smithy member names and JSON wire field names.
///
/// When `@jsonName` is enabled, the wire name may differ from the member name.
/// This type handles the mapping in both directions and caches the reverse
/// lookup (wire name → member index) per struct schema.
#[derive(Debug)]
enum JsonFieldMapper {
    /// Uses member names directly, ignoring `@jsonName`.
    UseMemberName,
    /// Uses `@jsonName` trait values when present, falling back to member name.
    UseJsonName,
}

impl JsonFieldMapper {
    /// Returns the JSON wire name for a member schema.
    fn member_to_field<'a>(&self, member: &'a Schema) -> Option<&'a str> {
        let name = member.member_name()?;
        match self {
            JsonFieldMapper::UseMemberName => Some(name),
            JsonFieldMapper::UseJsonName => {
                if let Some(jn) = member.json_name() {
                    return Some(jn.value());
                }
                Some(name)
            }
        }
    }

    /// Resolves a JSON wire field name to a member schema within a struct schema.
    fn field_to_member<'s>(&self, schema: &'s Schema, field_name: &str) -> Option<&'s Schema> {
        match self {
            JsonFieldMapper::UseMemberName => schema.member_schema(field_name),
            JsonFieldMapper::UseJsonName => {
                // Check @jsonName on each member. For typical struct sizes
                // (< 50 members), linear scan is faster than a cached HashMap
                // behind a Mutex.
                for member in schema.members() {
                    if let Some(jn) = member.json_name() {
                        if jn.value() == field_name {
                            return Some(member);
                        }
                    } else if member.member_name() == Some(field_name) {
                        return Some(member);
                    }
                }
                None
            }
        }
    }
}

/// Configuration for JSON codec behavior.
///
/// Use the builder methods to construct settings:
/// ```
/// use aws_smithy_json::codec::JsonCodecSettings;
///
/// let settings = JsonCodecSettings::builder()
///     .use_json_name(false)
///     .build();
/// ```
#[derive(Debug)]
pub struct JsonCodecSettings {
    field_mapper: JsonFieldMapper,
    default_timestamp_format: TimestampFormat,
    max_depth: u32,
    enforce_strictness: bool,
    allow_integral_float_numbers: bool,
    /// When `true`, a timestamp must use exactly the wire form its resolved
    /// `@timestampFormat` (or the codec default) prescribes: a JSON number for
    /// `epoch-seconds`, an RFC 3339 string without a UTC offset for `date-time`,
    /// and an IMF-fixdate string for `http-date`. Servers enable this so that
    /// malformed requests are rejected. When `false` (default) the deserializer
    /// is tolerant, as clients are: a number is always read as epoch seconds and
    /// a string for a `date-time` or `epoch-seconds` member is parsed as an
    /// offset-aware `date-time`.
    strict_timestamp_formats: bool,
}

impl JsonCodecSettings {
    /// Creates a builder for `JsonCodecSettings`.
    pub fn builder() -> JsonCodecSettingsBuilder {
        JsonCodecSettingsBuilder::default()
    }

    /// Default timestamp format when not specified by `@timestampFormat` trait.
    pub fn default_timestamp_format(&self) -> TimestampFormat {
        self.default_timestamp_format
    }

    /// Maximum aggregate nesting depth the deserializer will accept before
    /// returning an error. Defends against stack overflow on recursive shapes
    /// and deeply-nested document payloads.
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }

    /// Whether timestamps must use exactly the wire form their format prescribes.
    /// See [`JsonCodecSettingsBuilder::strict_timestamp_formats`].
    pub fn strict_timestamp_formats(&self) -> bool {
        self.strict_timestamp_formats
    }

    /// Returns the JSON wire name for a member schema.
    pub(crate) fn member_to_field<'a>(&self, member: &'a Schema) -> Option<&'a str> {
        self.field_mapper.member_to_field(member)
    }

    /// Resolves a JSON wire field name to a member schema.
    pub(crate) fn field_to_member<'s>(
        &self,
        schema: &'s Schema,
        field_name: &str,
    ) -> Option<&'s Schema> {
        self.field_mapper.field_to_member(schema, field_name)
    }
}

impl Default for JsonCodecSettings {
    fn default() -> Self {
        Self {
            field_mapper: JsonFieldMapper::UseJsonName,
            default_timestamp_format: TimestampFormat::EpochSeconds,
            max_depth: crate::codec::deserializer::MAX_DESERIALIZE_DEPTH,
            enforce_strictness: false,
            allow_integral_float_numbers: false,
            strict_timestamp_formats: false,
        }
    }
}

/// Builder for [`JsonCodecSettings`].
#[derive(Debug, Clone)]
pub struct JsonCodecSettingsBuilder {
    use_json_name: bool,
    default_timestamp_format: TimestampFormat,
    max_depth: u32,
    enforce_strictness: bool,
    allow_integral_float_numbers: bool,
    strict_timestamp_formats: bool,
}

impl Default for JsonCodecSettingsBuilder {
    fn default() -> Self {
        Self {
            use_json_name: true,
            default_timestamp_format: TimestampFormat::EpochSeconds,
            max_depth: crate::codec::deserializer::MAX_DESERIALIZE_DEPTH,
            enforce_strictness: false,
            allow_integral_float_numbers: false,
            strict_timestamp_formats: false,
        }
    }
}

impl JsonCodecSettingsBuilder {
    /// Validates string contents and number syntax, rejects whitespace-only structure
    /// bodies, and checks the range of floating-point epoch timestamps.
    /// Disabled by default. Timestamp wire formats are controlled separately by
    /// [`Self::strict_timestamp_formats`].
    pub fn enforce_strictness(mut self, value: bool) -> Self {
        self.enforce_strictness = value;
        self
    }

    /// Accepts exactly integral JSON decimal/exponent numbers for integer members.
    /// Disabled by default; ordinary integer spellings retain exact parsing.
    pub fn allow_integral_float_numbers(mut self, value: bool) -> Self {
        self.allow_integral_float_numbers = value;
        self
    }

    /// Whether to use the `@jsonName` trait for member names.
    pub fn use_json_name(mut self, value: bool) -> Self {
        self.use_json_name = value;
        self
    }

    /// Default timestamp format when not specified by `@timestampFormat` trait.
    pub fn default_timestamp_format(mut self, value: TimestampFormat) -> Self {
        self.default_timestamp_format = value;
        self
    }

    /// Sets the maximum aggregate nesting depth the deserializer will accept
    /// before returning an error. Defaults to 128.
    pub fn max_depth(mut self, value: u32) -> Self {
        self.max_depth = value;
        self
    }

    /// Require timestamps to use exactly the wire form their resolved format
    /// prescribes: a JSON number for `epoch-seconds`, an RFC 3339 string without
    /// a UTC offset for `date-time`, an IMF-fixdate string for `http-date`.
    /// Off by default; servers turn it on so malformed requests are rejected.
    pub fn strict_timestamp_formats(mut self, value: bool) -> Self {
        self.strict_timestamp_formats = value;
        self
    }

    /// Builds the settings.
    pub fn build(self) -> JsonCodecSettings {
        let field_mapper = if self.use_json_name {
            JsonFieldMapper::UseJsonName
        } else {
            JsonFieldMapper::UseMemberName
        };
        JsonCodecSettings {
            field_mapper,
            default_timestamp_format: self.default_timestamp_format,
            max_depth: self.max_depth,
            enforce_strictness: self.enforce_strictness,
            allow_integral_float_numbers: self.allow_integral_float_numbers,
            strict_timestamp_formats: self.strict_timestamp_formats,
        }
    }
}

/// JSON codec for schema-based serialization and deserialization.
///
/// # Examples
///
/// ```
/// use aws_smithy_json::codec::{JsonCodec, JsonCodecSettings};
/// use aws_smithy_schema::codec::Codec;
///
/// // Create codec with default settings (REST JSON style)
/// let codec = JsonCodec::new(JsonCodecSettings::default());
///
/// // Create codec for AWS JSON RPC (no jsonName, epoch-seconds timestamps)
/// let codec = JsonCodec::new(
///     JsonCodecSettings::builder()
///         .use_json_name(false)
///         .build()
/// );
/// ```
#[derive(Debug)]
pub struct JsonCodec {
    settings: Arc<JsonCodecSettings>,
}

impl JsonCodec {
    /// Creates a new JSON codec with the given settings.
    pub fn new(settings: JsonCodecSettings) -> Self {
        Self {
            settings: Arc::new(settings),
        }
    }

    /// Returns the codec settings.
    pub fn settings(&self) -> &JsonCodecSettings {
        &self.settings
    }
}

impl Default for JsonCodec {
    fn default() -> Self {
        Self::new(JsonCodecSettings::default())
    }
}

impl Codec for JsonCodec {
    type Serializer = JsonSerializer;
    type Deserializer<'a> = JsonDeserializer<'a>;

    fn create_serializer(&self) -> Self::Serializer {
        JsonSerializer::new(self.settings.clone())
    }

    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
        JsonDeserializer::new(input, self.settings.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = JsonCodecSettings::default();
        assert_eq!(
            settings.default_timestamp_format(),
            TimestampFormat::EpochSeconds
        );
    }

    #[test]
    fn test_builder() {
        let settings = JsonCodecSettings::builder()
            .use_json_name(false)
            .default_timestamp_format(TimestampFormat::DateTime)
            .build();
        assert_eq!(
            settings.default_timestamp_format(),
            TimestampFormat::DateTime
        );
    }

    #[test]
    fn test_codec_creation() {
        let codec = JsonCodec::default();
        let _serializer = codec.create_serializer();
        let _deserializer = codec.create_deserializer(b"{}");
    }
}
