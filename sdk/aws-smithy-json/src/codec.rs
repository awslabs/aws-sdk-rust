/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! JSON codec implementation for schema-based serialization.

use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::date_time::{DateTime, Format as TimestampFormat};
use aws_smithy_types::{DocumentError, DocumentSettings, Number};
use std::sync::Arc;

mod deserializer;
mod serializer;

pub use deserializer::JsonDeserializer;
pub use serializer::JsonSerializer;

/// Maps between Smithy member names and JSON wire field names.
///
/// When `@jsonName` is enabled, the wire name may differ from the member name.
/// This type handles the mapping in both directions.
///
/// Nothing is cached, and cannot be: this is a fieldless `Copy` enum with
/// nowhere to store a cache. Reverse lookups are a linear scan over the
/// struct's members — see [`JsonFieldMapper::field_to_member`].
#[derive(Debug, Clone, Copy)]
enum JsonFieldMapper {
    /// Uses member names directly, ignoring `@jsonName`.
    UseMemberName,
    /// Uses `@jsonName` trait values when present, falling back to member name.
    UseJsonName,
}

impl JsonFieldMapper {
    /// Returns the JSON wire name for a member schema.
    fn member_to_field<'a>(&self, member: &'a Schema<'a>) -> Option<&'a str> {
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
    ///
    /// This is `O(M)` in the number of struct members and is called once per field
    /// present on the wire, so deserializing a struct is `O(M²)` in the worst case.
    /// An unknown field costs a full scan before returning `None`.
    ///
    /// **Both** arms scan: `UseJsonName` loops below, and `UseMemberName` delegates
    /// to `Schema::member_schema`, which is itself a `find` over the members.
    ///
    /// Real member counts across AWS models are p50=2, p90=6, p99=20, so this is
    /// not measurable for the large majority of structs. If it ever needs fixing,
    /// the fix is a codegen-emitted name→index table, not a runtime cache:
    /// `Schema::member_schema_by_index` is already `O(1)` and generated code
    /// already matches on `member_index()` downstream, so only the name→index hop
    /// is linear.
    fn field_to_member<'s>(
        &self,
        schema: &'s Schema<'s>,
        field_name: &str,
    ) -> Option<&'s Schema<'s>> {
        match self {
            JsonFieldMapper::UseMemberName => schema.member_schema(field_name),
            JsonFieldMapper::UseJsonName => {
                // Check @jsonName on each member. A HashMap cache behind a Mutex
                // would be slower than this scan at realistic member counts; see
                // the doc comment for the approach that would actually help.
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
/// `Document` it produces so the format-aware accessors
/// ([`Document::as_blob`](aws_smithy_types::Document::as_blob),
/// [`Document::as_timestamp`](aws_smithy_types::Document::as_timestamp))
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
    /// Identifies the protocol that produced this codec — used by
    /// `DocumentSettings` for diagnostics on coercion failures.
    /// Default: `aws.smithy.json#JsonCodec`. AWS protocols (awsJson1_0,
    /// awsJson1_1, restJson1) override this to their own shape id.
    protocol_id: ShapeId<'static>,
    /// When `true`, [`Document::BigInteger`](aws_smithy_types::Document::BigInteger)
    /// and [`Document::BigDecimal`](aws_smithy_types::Document::BigDecimal)
    /// serialize as JSON strings to preserve precision for receivers
    /// using `f64`-routed JSON parsers. When `false` (default), they
    /// emit as raw JSON numbers — interoperable with arbitrary-
    /// precision JSON parsers but lossy when the receiver routes
    /// through `f64`.
    ///
    /// The read path always accepts both wire forms, regardless of
    /// this setting, so a sender configured for one form interoperates
    /// with a receiver configured for the other.
    use_string_for_arbitrary_precision: bool,
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

    /// Whether [`Document::BigInteger`](aws_smithy_types::Document::BigInteger)
    /// and [`Document::BigDecimal`](aws_smithy_types::Document::BigDecimal)
    /// emit as JSON strings (when `true`) or as raw JSON numbers
    /// (when `false`, the default). The read path is always lenient —
    /// `read_big_integer` and `read_big_decimal` accept either wire form.
    pub fn use_string_for_arbitrary_precision(&self) -> bool {
        self.use_string_for_arbitrary_precision
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
            protocol_id: self.protocol_id.clone(),
            use_string_for_arbitrary_precision: self.use_string_for_arbitrary_precision,
            default_namespace: self.default_namespace.clone(),
            enforce_strictness: self.enforce_strictness,
            allow_integral_float_numbers: self.allow_integral_float_numbers,
            strict_timestamp_formats: self.strict_timestamp_formats,
        }
    }

    /// Whether timestamps must use exactly the wire form their format prescribes.
    /// See [`JsonCodecSettingsBuilder::strict_timestamp_formats`].
    pub fn strict_timestamp_formats(&self) -> bool {
        self.strict_timestamp_formats
    }

    /// Returns the JSON wire name for a member schema.
    pub(crate) fn member_to_field<'a>(&self, member: &'a Schema<'a>) -> Option<&'a str> {
        self.field_mapper.member_to_field(member)
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
            protocol_id: DEFAULT_JSON_CODEC_ID,
            use_string_for_arbitrary_precision: false,
            default_namespace: None,
            enforce_strictness: false,
            allow_integral_float_numbers: false,
            strict_timestamp_formats: false,
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
    protocol_id: ShapeId<'static>,
    use_string_for_arbitrary_precision: bool,
    default_namespace: Option<String>,
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
            protocol_id: DEFAULT_JSON_CODEC_ID,
            use_string_for_arbitrary_precision: false,
            default_namespace: None,
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

    /// Sets the protocol id surfaced in `DocumentSettings`
    /// diagnostics. Defaults to `aws.smithy.json#JsonCodec`. AWS
    /// protocols (awsJson1_0, awsJson1_1, restJson1) supply their own
    /// shape id so coercion errors point at the originating protocol.
    pub fn protocol_id(mut self, value: ShapeId<'static>) -> Self {
        self.protocol_id = value;
        self
    }

    /// Configures whether [`Document::BigInteger`](aws_smithy_types::Document::BigInteger)
    /// and [`Document::BigDecimal`](aws_smithy_types::Document::BigDecimal)
    /// serialize as JSON strings (when `true`) or as raw JSON numbers
    /// (when `false`, the default).
    ///
    /// Set to `true` for interop with receivers using `f64`-routed
    /// JSON parsers that would otherwise lose precision on large
    /// integers or decimals. The read path always accepts both forms.
    pub fn use_string_for_arbitrary_precision(mut self, value: bool) -> Self {
        self.use_string_for_arbitrary_precision = value;
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
            protocol_id: self.protocol_id,
            use_string_for_arbitrary_precision: self.use_string_for_arbitrary_precision,
            default_namespace: self.default_namespace,
            enforce_strictness: self.enforce_strictness,
            allow_integral_float_numbers: self.allow_integral_float_numbers,
            strict_timestamp_formats: self.strict_timestamp_formats,
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
            .allow_integral_float_numbers(true)
            .strict_timestamp_formats(true)
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
        assert!(round_tripped.allow_integral_float_numbers);
        assert!(round_tripped.strict_timestamp_formats());
    }

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
}
