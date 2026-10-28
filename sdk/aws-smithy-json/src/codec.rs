/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON codec implementation for schema-based serialization.

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::extension::SchemaExtensionKey;
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::date_time::{DateTime, Format as TimestampFormat};
use aws_smithy_types::{DocumentError, DocumentSettings, Number};
use std::sync::Arc;

mod deserializer;
mod serializer;

pub use deserializer::JsonDeserializer;
pub use serializer::JsonSerializer;

/// The JSON object keys (`"name":`) for one member schema, quoted and escaped once
/// and cached on the schema by [`JSON_MEMBER_KEYS`].
///
/// Both [`JsonFieldMapper`] variants are cached because a schema is shared by every
/// codec in the process, and codecs that honor `@jsonName` and codecs that ignore it
/// can serialize the same shape.
#[derive(Debug)]
struct JsonMemberKeys {
    /// `"<member name>":`
    member_name: Box<str>,
    /// `"<@jsonName value>":`, present only when the member has `@jsonName`.
    json_name: Option<Box<str>>,
}

/// Quotes and escapes `name` as a JSON object key, including the trailing colon.
fn encode_key(name: &str) -> Box<str> {
    let escaped = crate::escape::escape_string(name);
    let mut key = String::with_capacity(escaped.len() + 3);
    key.push('"');
    key.push_str(&escaped);
    key.push_str("\":");
    key.into_boxed_str()
}

/// Caches a member schema's JSON object keys, so the serializer writes each field
/// name with one `push_str` instead of quoting and escaping it on every request.
///
/// Only requested for schemas that have a member name; see
/// [`JsonFieldMapper::member_key`].
static JSON_MEMBER_KEYS: SchemaExtensionKey<JsonMemberKeys> =
    SchemaExtensionKey::new(|member| JsonMemberKeys {
        member_name: encode_key(member.member_name().unwrap_or_default()),
        json_name: member
            .json_name()
            .map(|json_name| encode_key(json_name.value())),
    });

/// Maps between Smithy member names and JSON wire field names.
///
/// When `@jsonName` is enabled, the wire name may differ from the member name.
/// This type handles the mapping in both directions.
///
/// The mapper itself is a fieldless `Copy` enum. Encoded keys for serialization are
/// cached on the member schemas (see [`JsonFieldMapper::member_key`]); reverse
/// lookups are a linear scan over the struct's members (see
/// [`JsonFieldMapper::field_to_member`]).
#[derive(Debug, Clone, Copy)]
enum JsonFieldMapper {
    /// Uses member names directly, ignoring `@jsonName`.
    UseMemberName,
    /// Uses `@jsonName` trait values when present, falling back to member name.
    UseJsonName,
}

impl JsonFieldMapper {
    /// Returns the encoded JSON object key (`"name":`) for a member schema, or `None`
    /// if the schema is not a member.
    #[inline]
    fn member_key<'s>(&self, member: &'s Schema<'_>) -> Option<&'s str> {
        // Checked first so that schemas without a field name, such as a top-level
        // structure, never allocate extension storage.
        member.member_name()?;
        let keys = member.extension(&JSON_MEMBER_KEYS);
        Some(match self {
            JsonFieldMapper::UseMemberName => &keys.member_name,
            JsonFieldMapper::UseJsonName => keys.json_name.as_ref().unwrap_or(&keys.member_name),
        })
    }

    /// Resolves a JSON wire field name to a member schema within a struct schema.
    ///
    /// Returns the first member whose wire name matches, so a field name shared by
    /// several members (a member name equal to another member's `@jsonName`) always
    /// resolves to the same member.
    ///
    /// This is `O(M)` in the number of struct members and is called once per field
    /// present on the wire, so deserializing a struct is `O(M²)` in the worst case.
    /// An unknown field costs a full scan before returning `None`.
    ///
    /// **Both** arms scan: `UseJsonName` loops below, and `UseMemberName` delegates
    /// to `Schema::member_schema`, which is itself a `find` over the members.
    ///
    /// Real member counts across AWS models are p50=2, p90=6, p99=20, so this is
    /// not measurable for the large majority of structs.
    fn field_to_member<'s>(
        &self,
        schema: &'s Schema<'s>,
        field_name: &str,
    ) -> Option<&'s Schema<'s>> {
        match self {
            JsonFieldMapper::UseMemberName => schema.member_schema(field_name),
            JsonFieldMapper::UseJsonName => {
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
///
/// # As `DocumentSettings`
///
/// `JsonCodecSettings` implements [`DocumentSettings`]. The
/// [`JsonDeserializer`] attaches `Arc<JsonCodecSettings>` to every
/// document it produces so the format-aware accessors
/// ([`DiscriminatedDocument::as_blob`](aws_smithy_types::DiscriminatedDocument::as_blob),
/// [`DiscriminatedDocument::as_timestamp`](aws_smithy_types::DiscriminatedDocument::as_timestamp))
/// can coerce JSON-encoded blobs (base64 strings) and timestamps
/// (date-time strings or epoch-seconds numbers) back to typed Rust
/// values. The same instance is reused on the serialization side so a
/// document round-trips through `JsonDeserializer` →
/// `JsonSerializer` losslessly.
#[derive(Debug, Clone)]
pub struct JsonCodecSettings {
    field_mapper: JsonFieldMapper,
    default_timestamp_format: TimestampFormat,
    max_depth: u32,
    /// When `true`, a timestamp must use exactly the wire form its resolved
    /// `@timestampFormat` (or the codec default) prescribes: a JSON number for
    /// `epoch-seconds`, an RFC 3339 string without a UTC offset for `date-time`,
    /// and an IMF-fixdate string for `http-date`. Any other form is rejected.
    /// When `false` (default) the deserializer is tolerant: a number is always
    /// read as epoch seconds and a string for a `date-time` or `epoch-seconds`
    /// member is parsed as an offset-aware `date-time`.
    strict_timestamp_formats: bool,
    dispatch_null_members: bool,
    enforce_strictness: bool,
    validate_skipped_values: bool,
    validate_skipped_string_encoding: bool,
    allow_integral_float_numbers: bool,
    allow_leading_zeros: bool,
    allow_trailing_decimal_point: bool,
    allow_empty_integer_part: bool,
    /// Identifies the protocol that produced this codec — used by
    /// `DocumentSettings` for diagnostics on coercion failures.
    /// Default: `aws.smithy.json#JsonCodec`. AWS protocols (awsJson1_0,
    /// awsJson1_1, restJson1) override this to their own shape id.
    protocol_id: ShapeId<'static>,
    /// When `true`, `bigInteger` and `bigDecimal` shapes serialize as
    /// JSON strings to preserve precision for receivers using
    /// `f64`-routed JSON parsers. When `false` (default), they emit as
    /// raw JSON numbers — interoperable with arbitrary-precision JSON
    /// parsers but lossy when the receiver routes through `f64`.
    ///
    /// The read path always accepts both wire forms, regardless of
    /// this setting, so a sender configured for one form interoperates
    /// with a receiver configured for the other.
    use_string_for_arbitrary_precision: bool,
    /// When `true`, a `bigInteger` or `bigDecimal` whose text is a valid decimal but not an
    /// RFC 8259 JSON number (a leading `+`, leading zeros, `.5`, `1.`) is rewritten to the
    /// equivalent JSON number on the raw-number path. When `false` (default) it is rejected with
    /// [`SerdeError::InvalidInput`](aws_smithy_schema::serde::SerdeError). Has no effect when
    /// `use_string_for_arbitrary_precision` is set.
    normalize_arbitrary_precision: bool,
    /// Default Smithy namespace used to resolve relative shape IDs in
    /// JSON `__type` discriminator fields produced by services that
    /// emit relative names (e.g., awsJson1_0/1_1 senders).
    ///
    /// When set, [`crate::codec::deserializer::JsonDeserializer::read_discriminated_document`]
    /// resolves a relative `__type` value (e.g., `"Capacity"`) to its
    /// fully-qualified shape ID (`"<default_namespace>#Capacity"`)
    /// before lifting it onto the resulting [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument)
    /// discriminator slot. Absolute `__type` values (already
    /// containing `#`) are taken as-is regardless of this setting.
    ///
    /// `None` (default) preserves the prior behavior where relative
    /// names are left in the resulting map as plain string entries.
    default_namespace: Option<String>,
}

impl JsonCodecSettings {
    /// Whether null structure members are passed to the consumer instead of skipped.
    /// Schema servers use this to clear optional fields while leaving required validation to builders.
    pub fn dispatch_null_members(&self) -> bool {
        self.dispatch_null_members
    }

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

    /// Whether `bigInteger` and `bigDecimal` shapes emit as JSON
    /// strings (when `true`) or as raw JSON numbers
    /// (when `false`, the default). The read path is always lenient —
    /// `read_big_integer` and `read_big_decimal` accept either wire form.
    pub fn use_string_for_arbitrary_precision(&self) -> bool {
        self.use_string_for_arbitrary_precision
    }

    /// Whether non-JSON-number arbitrary-precision text is rewritten to the equivalent JSON
    /// number (`true`) or rejected (`false`, the default). See the field documentation.
    pub fn normalize_arbitrary_precision(&self) -> bool {
        self.normalize_arbitrary_precision
    }

    /// Default Smithy namespace used to resolve relative shape IDs in
    /// JSON `__type` discriminator fields. `None` if unset.
    ///
    /// See the field-level doc comment for [`Self::default_namespace`]'s
    /// builder method for the resolution behavior.
    pub fn default_namespace(&self) -> Option<&str> {
        self.default_namespace.as_deref()
    }

    /// Returns a [`JsonCodecSettingsBuilder`] pre-populated with this
    /// instance's current values. Useful for protocol wrappers that
    /// need to rebuild a codec with one setting overridden:
    ///
    /// ```
    /// use aws_smithy_json::codec::JsonCodecSettings;
    ///
    /// let original = JsonCodecSettings::default();
    /// let modified = original.to_builder()
    ///     .default_namespace("com.example")
    ///     .build();
    /// ```
    pub fn to_builder(&self) -> JsonCodecSettingsBuilder {
        JsonCodecSettingsBuilder {
            use_json_name: matches!(self.field_mapper, JsonFieldMapper::UseJsonName),
            default_timestamp_format: self.default_timestamp_format,
            max_depth: self.max_depth,
            enforce_strictness: self.enforce_strictness,
            validate_skipped_values: self.validate_skipped_values,
            validate_skipped_string_encoding: self.validate_skipped_string_encoding,
            allow_integral_float_numbers: self.allow_integral_float_numbers,
            allow_leading_zeros: self.allow_leading_zeros,
            allow_trailing_decimal_point: self.allow_trailing_decimal_point,
            allow_empty_integer_part: self.allow_empty_integer_part,
            strict_timestamp_formats: self.strict_timestamp_formats,
            dispatch_null_members: self.dispatch_null_members,
            protocol_id: self.protocol_id.clone(),
            use_string_for_arbitrary_precision: self.use_string_for_arbitrary_precision,
            normalize_arbitrary_precision: self.normalize_arbitrary_precision,
            default_namespace: self.default_namespace.clone(),
        }
    }

    /// Returns the encoded JSON object key (`"name":`) for a member schema.
    #[inline]
    pub(crate) fn member_key<'s>(&self, member: &'s Schema<'_>) -> Option<&'s str> {
        self.field_mapper.member_key(member)
    }

    /// Resolves a JSON wire field name to a member schema.
    pub(crate) fn field_to_member<'s>(
        &self,
        schema: &'s Schema<'s>,
        field_name: &str,
    ) -> Option<&'s Schema<'s>> {
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
            validate_skipped_values: true,
            validate_skipped_string_encoding: false,
            allow_integral_float_numbers: false,
            allow_leading_zeros: false,
            allow_trailing_decimal_point: false,
            allow_empty_integer_part: false,
            strict_timestamp_formats: false,
            dispatch_null_members: false,
            protocol_id: DEFAULT_JSON_CODEC_ID,
            use_string_for_arbitrary_precision: false,
            normalize_arbitrary_precision: false,
            default_namespace: None,
        }
    }
}

/// Sentinel protocol id used by `JsonCodecSettings` when none is
/// supplied — surfaces in `DocumentSettings` diagnostics. AWS
/// protocols override this with their own ids.
const DEFAULT_JSON_CODEC_ID: ShapeId<'static> = shape_id!("aws.smithy.json", "JsonCodec");

/// Selects the timestamp [`Format`](aws_smithy_types::date_time::Format) used
/// to parse a *string*-encoded timestamp, given a codec's configured default
/// format.
///
/// A JSON string never encodes `epoch-seconds` (a number), so every default
/// other than `http-date` resolves to the offset-aware `date-time` form — the
/// common JSON string-timestamp encoding and a lenient superset of strict
/// `date-time`. `http-date` is used as-is.
///
/// Shared by the untyped-document coercion path (`coerce_string_to_timestamp`)
/// and the schema-based read path (`read_timestamp`) so the two string-
/// timestamp paths cannot drift.
pub(crate) fn string_timestamp_format(default: TimestampFormat) -> TimestampFormat {
    match default {
        TimestampFormat::HttpDate => TimestampFormat::HttpDate,
        _ => TimestampFormat::DateTimeWithOffset,
    }
}

/// Builder for [`JsonCodecSettings`].
#[derive(Debug, Clone)]
pub struct JsonCodecSettingsBuilder {
    use_json_name: bool,
    default_timestamp_format: TimestampFormat,
    max_depth: u32,
    strict_timestamp_formats: bool,
    dispatch_null_members: bool,
    enforce_strictness: bool,
    validate_skipped_values: bool,
    validate_skipped_string_encoding: bool,
    allow_integral_float_numbers: bool,
    allow_leading_zeros: bool,
    allow_trailing_decimal_point: bool,
    allow_empty_integer_part: bool,
    protocol_id: ShapeId<'static>,
    use_string_for_arbitrary_precision: bool,
    normalize_arbitrary_precision: bool,
    default_namespace: Option<String>,
}

impl Default for JsonCodecSettingsBuilder {
    fn default() -> Self {
        Self {
            use_json_name: true,
            default_timestamp_format: TimestampFormat::EpochSeconds,
            max_depth: crate::codec::deserializer::MAX_DESERIALIZE_DEPTH,
            enforce_strictness: false,
            validate_skipped_values: true,
            validate_skipped_string_encoding: false,
            allow_integral_float_numbers: false,
            allow_leading_zeros: false,
            allow_trailing_decimal_point: false,
            allow_empty_integer_part: false,
            strict_timestamp_formats: false,
            dispatch_null_members: false,
            protocol_id: DEFAULT_JSON_CODEC_ID,
            use_string_for_arbitrary_precision: false,
            normalize_arbitrary_precision: false,
            default_namespace: None,
        }
    }
}

impl JsonCodecSettingsBuilder {
    /// Enforces JSON grammar and request document validity. Disabled by default.
    pub fn enforce_strictness(mut self, value: bool) -> Self {
        self.enforce_strictness = value;
        self
    }

    /// Whether discarded string contents are validated. Enabled by default.
    ///
    /// When enabled, escape syntax is checked without decoding Unicode escapes.
    /// `enforce_strictness` applies only to values read. When disabled, strings and keys inside
    /// discarded objects are scanned without decoding or validating escapes.
    /// Container syntax, number/literal grammar, truncation, and depth limits
    /// are still checked. Values actually read are unaffected.
    pub fn validate_skipped_values(mut self, value: bool) -> Self {
        self.validate_skipped_values = value;
        self
    }

    /// Checks discarded strings for UTF-8 and unescaped control characters without
    /// decoding or validating their escape sequences. Disabled by default.
    pub fn validate_skipped_string_encoding(mut self, value: bool) -> Self {
        self.validate_skipped_string_encoding = value;
        self
    }

    /// Accepts exactly integral decimal/exponent numbers for integer members.
    /// Disabled by default; ordinary integer spellings retain exact parsing.
    pub fn allow_integral_float_numbers(mut self, value: bool) -> Self {
        self.allow_integral_float_numbers = value;
        self
    }

    /// Permits leading zeros in JSON numbers even with `enforce_strictness(true)`.
    /// Disabled by default. This matches the legacy JSON parser without relaxing
    /// other grammar checks. Applies to read and skipped numbers alike.
    pub fn allow_leading_zeros(mut self, value: bool) -> Self {
        self.allow_leading_zeros = value;
        self
    }

    /// Permits an empty fraction after a decimal point, such as `12.` or `12.e2`,
    /// even with `enforce_strictness(true)`. Disabled by default. Integer members
    /// additionally require `allow_integral_float_numbers(true)` and a whole value.
    /// Applies to read and skipped numbers alike.
    pub fn allow_trailing_decimal_point(mut self, value: bool) -> Self {
        self.allow_trailing_decimal_point = value;
        self
    }

    /// Permits negative fractions with no integer digits, such as `-.123` or
    /// `-.123e2`, even with `enforce_strictness(true)`. Disabled by default.
    /// Under strict number validation, `.123`, `-.`, and `-.e2` remain invalid.
    /// Applies to read and skipped numbers alike.
    pub fn allow_empty_integer_part(mut self, value: bool) -> Self {
        self.allow_empty_integer_part = value;
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
    /// Off by default; when enabled, any other form is rejected.
    pub fn strict_timestamp_formats(mut self, value: bool) -> Self {
        self.strict_timestamp_formats = value;
        self
    }

    /// Pass null structure members to the consumer. Defaults to false, preserving existing
    /// codec callers; schema server readers decide how each null affects their builders.
    pub fn dispatch_null_members(mut self, value: bool) -> Self {
        self.dispatch_null_members = value;
        self
    }

    /// Sets the protocol id surfaced in `DocumentSettings`
    /// diagnostics. Defaults to `aws.smithy.json#JsonCodec`. AWS
    /// protocols (awsJson1_0, awsJson1_1, restJson1) supply their own
    /// shape id so coercion errors point at the originating protocol.
    pub fn protocol_id(mut self, value: ShapeId<'static>) -> Self {
        self.protocol_id = value;
        self
    }

    /// Configures whether `bigInteger` and `bigDecimal` shapes
    /// serialize as JSON strings (when `true`) or as raw JSON numbers
    /// (when `false`, the default).
    ///
    /// This is schema-driven: a `Document` carries no schema and has no
    /// arbitrary-precision variant, so `write_document` is unaffected.
    ///
    /// Set to `true` for interop with receivers using `f64`-routed
    /// JSON parsers that would otherwise lose precision on large
    /// integers or decimals. The read path always accepts both forms.
    pub fn use_string_for_arbitrary_precision(mut self, value: bool) -> Self {
        self.use_string_for_arbitrary_precision = value;
        self
    }

    /// Rewrites arbitrary-precision text that is a decimal but not a JSON number (`+007`, `.5`,
    /// `1.`) to the equivalent JSON number instead of rejecting it. Defaults to `false`, so
    /// existing callers keep rejecting it; AWS servers set it to `true`, matching their lenient
    /// handling of such numbers on the read path (`allow_leading_zeros`,
    /// `allow_trailing_decimal_point`).
    pub fn normalize_arbitrary_precision(mut self, value: bool) -> Self {
        self.normalize_arbitrary_precision = value;
        self
    }

    /// Sets the default Smithy namespace used to resolve relative shape
    /// IDs in JSON `__type` discriminator fields. When set, a relative
    /// `__type: "Capacity"` is lifted to `<namespace>#Capacity` on the
    /// resulting [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument). Absolute `__type` values
    /// (already containing `#`) are taken as-is regardless.
    ///
    /// Defaults to `None` — relative names are not lifted.
    pub fn default_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.default_namespace = Some(namespace.into());
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
            validate_skipped_values: self.validate_skipped_values,
            validate_skipped_string_encoding: self.validate_skipped_string_encoding,
            allow_integral_float_numbers: self.allow_integral_float_numbers,
            allow_leading_zeros: self.allow_leading_zeros,
            allow_trailing_decimal_point: self.allow_trailing_decimal_point,
            allow_empty_integer_part: self.allow_empty_integer_part,
            strict_timestamp_formats: self.strict_timestamp_formats,
            dispatch_null_members: self.dispatch_null_members,
            protocol_id: self.protocol_id,
            use_string_for_arbitrary_precision: self.use_string_for_arbitrary_precision,
            normalize_arbitrary_precision: self.normalize_arbitrary_precision,
            default_namespace: self.default_namespace,
        }
    }
}

impl DocumentSettings for JsonCodecSettings {
    fn protocol_id(&self) -> &str {
        // The internal field is `ShapeId<'static>` (preserved for
        // ergonomic construction via `shape_id!`); the trait surface
        // returns the FQN string per [`DocumentSettings`].
        self.protocol_id.as_str()
    }

    /// Decodes a base64 string into a blob.
    ///
    /// JSON has no native blob type; the
    /// [Smithy spec](https://smithy.io/2.0/spec/simple-types.html#blob)
    /// requires blobs to be transmitted as base64-encoded strings on
    /// the JSON wire. This method consumes those strings on the
    /// deserialization side; the corresponding encode happens in
    /// `JsonSerializer::write_document`.
    fn coerce_string_to_blob(&self, s: &str) -> Result<Vec<u8>, DocumentError> {
        aws_smithy_types::base64::decode(s)
            .map_err(|e| DocumentError::invalid_input(format!("base64 decode failed: {e}")))
    }

    /// Parses a string-formatted timestamp using the codec's default
    /// timestamp format ([`JsonCodecSettings::default_timestamp_format`]).
    ///
    /// Per the SEP, untyped JSON documents have no schema-level
    /// `@timestampFormat` trait to consult, so the codec's default
    /// format is the only signal. AWS JSON / restJson1 default to
    /// `epoch-seconds` — but a string-typed JSON value cannot be
    /// epoch-seconds (which is a number). For this case we attempt
    /// `date-time` parsing as a fallback so common ISO-8601 strings
    /// still coerce.
    fn coerce_string_to_timestamp(&self, s: &str) -> Result<DateTime, DocumentError> {
        // A JSON string can't encode the number-typed `epoch-seconds`, so the
        // parse format is resolved via the shared `string_timestamp_format`
        // helper (also used by the schema-based read path).
        let format = string_timestamp_format(self.default_timestamp_format);
        DateTime::from_str(s, format)
            .map_err(|e| DocumentError::invalid_input(format!("timestamp parse failed: {e}")))
    }

    /// Coerces a number to a timestamp interpreted as
    /// `epoch-seconds`.
    ///
    /// Number-typed JSON values are only valid timestamps when the
    /// codec's default format is `epoch-seconds`. Per the SEP this is
    /// the AWS JSON / restJson1 default. If the codec has been
    /// configured with a non-numeric default format
    /// (`date-time` / `http-date`), this method returns an
    /// `UnsupportedOperation` error — a number value isn't valid for
    /// those formats.
    fn coerce_number_to_timestamp(&self, n: &Number) -> Result<DateTime, DocumentError> {
        if !matches!(self.default_timestamp_format, TimestampFormat::EpochSeconds) {
            return Err(DocumentError::unsupported(format!(
                "JSON codec configured with timestamp format {:?}; \
                 number-to-timestamp coercion only valid for epoch-seconds",
                self.default_timestamp_format
            )));
        }
        Ok(match *n {
            Number::PosInt(u) => {
                if u > i64::MAX as u64 {
                    return Err(DocumentError::invalid_input(format!(
                        "epoch-seconds value {u} overflows i64; cannot construct DateTime"
                    )));
                }
                DateTime::from_secs(u as i64)
            }
            Number::NegInt(i) => DateTime::from_secs(i),
            Number::Float(f) => {
                if !f.is_finite() {
                    return Err(DocumentError::invalid_input(format!(
                        "non-finite epoch-seconds value {f}; cannot construct DateTime"
                    )));
                }
                DateTime::from_secs_f64(f)
            }
        })
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

    #[test]
    fn document_settings_protocol_id_default_is_codec_sentinel() {
        let settings = JsonCodecSettings::default();
        assert_eq!(
            DocumentSettings::protocol_id(&settings),
            "aws.smithy.json#JsonCodec"
        );
    }

    #[test]
    fn document_settings_protocol_id_can_be_overridden() {
        let settings = JsonCodecSettings::builder()
            .protocol_id(shape_id!("aws.protocols", "restJson1"))
            .build();
        assert_eq!(
            DocumentSettings::protocol_id(&settings),
            "aws.protocols#restJson1"
        );
    }

    #[test]
    fn document_settings_coerce_string_to_blob_decodes_base64() {
        let settings = JsonCodecSettings::default();
        // base64 of "hello"
        let bytes = settings.coerce_string_to_blob("aGVsbG8=").unwrap();
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn document_settings_coerce_string_to_blob_returns_error_on_invalid() {
        let settings = JsonCodecSettings::default();
        let err = settings
            .coerce_string_to_blob("not!valid!base64!")
            .unwrap_err();
        match err {
            DocumentError::InvalidInput { .. } => {}
            other => panic!("expected DocumentError::InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn document_settings_coerce_string_to_timestamp_with_date_time_default() {
        let settings = JsonCodecSettings::builder()
            .default_timestamp_format(TimestampFormat::DateTime)
            .build();
        let dt = settings
            .coerce_string_to_timestamp("1970-01-01T00:00:00Z")
            .unwrap();
        assert_eq!(dt.secs(), 0);
    }

    #[test]
    fn document_settings_coerce_string_to_timestamp_falls_back_to_date_time_for_epoch_default() {
        // Default format is EpochSeconds (numeric); a string can't
        // satisfy that, so the impl falls back to date-time parsing.
        let settings = JsonCodecSettings::default();
        let dt = settings
            .coerce_string_to_timestamp("1970-01-01T00:00:00Z")
            .unwrap();
        assert_eq!(dt.secs(), 0);
    }

    #[test]
    fn document_settings_coerce_string_to_timestamp_error() {
        let settings = JsonCodecSettings::default();
        let err = settings
            .coerce_string_to_timestamp("not a timestamp")
            .unwrap_err();
        match err {
            DocumentError::InvalidInput { .. } => {}
            other => panic!("expected DocumentError::InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn document_settings_coerce_number_to_timestamp_epoch_seconds() {
        let settings = JsonCodecSettings::default();
        // PosInt
        let dt = settings
            .coerce_number_to_timestamp(&Number::PosInt(1_700_000_000))
            .unwrap();
        assert_eq!(dt.secs(), 1_700_000_000);
        // NegInt (pre-epoch)
        let dt = settings
            .coerce_number_to_timestamp(&Number::NegInt(-100))
            .unwrap();
        assert_eq!(dt.secs(), -100);
        // Float (fractional seconds)
        let dt = settings
            .coerce_number_to_timestamp(&Number::Float(1.5))
            .unwrap();
        assert_eq!(dt.secs(), 1);
        assert_eq!(dt.subsec_nanos(), 500_000_000);
    }

    #[test]
    fn document_settings_coerce_number_to_timestamp_unsupported_for_string_format() {
        let settings = JsonCodecSettings::builder()
            .default_timestamp_format(TimestampFormat::DateTime)
            .build();
        let err = settings
            .coerce_number_to_timestamp(&Number::PosInt(0))
            .unwrap_err();
        match err {
            DocumentError::UnsupportedOperation { .. } => {}
            other => panic!("expected DocumentError::UnsupportedOperation, got {other:?}"),
        }
    }

    #[test]
    fn document_settings_coerce_number_to_timestamp_rejects_non_finite_float() {
        let settings = JsonCodecSettings::default();
        let err = settings
            .coerce_number_to_timestamp(&Number::Float(f64::NAN))
            .unwrap_err();
        match err {
            DocumentError::InvalidInput { .. } => {}
            other => panic!("expected DocumentError::InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn to_builder_round_trips_every_setting() {
        // Regression guard: `to_builder` names each field explicitly, so a
        // setting added later is silently reset to its default here while still
        // compiling. That is how a codec rebuilt by a protocol wrapper (see
        // `protocol::codec_with_bag_namespace`, which overrides only
        // `default_namespace`) can lose strictness. Every setting below is set
        // to the OPPOSITE of its default so a dropped field fails this test.
        let original = JsonCodecSettings::builder()
            .use_json_name(false)
            .default_timestamp_format(TimestampFormat::HttpDate)
            .max_depth(7)
            .protocol_id(shape_id!("aws.protocols", "restJson1"))
            .use_string_for_arbitrary_precision(true)
            .default_namespace("com.example")
            .enforce_strictness(true)
            .validate_skipped_values(false)
            .allow_integral_float_numbers(true)
            .allow_leading_zeros(true)
            .allow_trailing_decimal_point(true)
            .allow_empty_integer_part(true)
            .strict_timestamp_formats(true)
            .dispatch_null_members(true)
            .build();

        let round_tripped = original.to_builder().build();

        assert!(matches!(
            round_tripped.field_mapper,
            JsonFieldMapper::UseMemberName
        ));
        assert_eq!(
            round_tripped.default_timestamp_format(),
            TimestampFormat::HttpDate
        );
        assert_eq!(round_tripped.max_depth(), 7);
        assert_eq!(
            DocumentSettings::protocol_id(&round_tripped),
            "aws.protocols#restJson1"
        );
        assert!(round_tripped.use_string_for_arbitrary_precision());
        assert_eq!(round_tripped.default_namespace(), Some("com.example"));
        assert!(round_tripped.enforce_strictness);
        assert!(!round_tripped.validate_skipped_values);
        assert!(round_tripped.allow_integral_float_numbers);
        assert!(round_tripped.allow_leading_zeros);
        assert!(round_tripped.allow_trailing_decimal_point);
        assert!(round_tripped.allow_empty_integer_part);
        assert!(round_tripped.strict_timestamp_formats());
        assert!(round_tripped.dispatch_null_members());
    }
}
