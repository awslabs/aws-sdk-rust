/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! [`DocumentShapeDeserializer`] — a [`ShapeDeserializer`]
//! implementation that walks an [`aws_smithy_types::Document`] tree.
//!
//! Generated `deserialize` methods on Smithy shapes call the
//! [`ShapeDeserializer`] interface to drive structure / list / map
//! consumer dispatch and read scalar leaves. Pointing such generated
//! code at this deserializer reifies a [`Document`] tree as the
//! corresponding typed shape — the inverse of [`DocumentShapeSerializer`].
//!
//! The deserializer holds a borrow of the source [`Document`] so reads
//! are zero-copy where possible (e.g. [`String`] reads still clone the
//! payload, but list/map navigation does not allocate).
//!
//! # Member-name resolution
//!
//! For struct reads, member dispatch uses the document's keys (not the
//! schema's member list) so that documents created from a typed shape
//! by [`DocumentShapeSerializer`] round-trip — those use Smithy member
//! names. Wire-name resolution for `@jsonName` / `@xmlName`-style
//! renames is the responsibility of the protocol's deserializer
//! (`JsonDeserializer`, etc.) — not this generic Document walker. The
//! `JsonFieldMapper` machinery in `aws-smithy-json` performs that
//! mapping during the codec stage; this deserializer simply matches
//! Smithy member names against document keys.
//!
//! Members present in the schema but absent from the document are not
//! reported: generated builders default unset optional members to
//! `None` and required-member enforcement is the builder's
//! responsibility, not the deserializer's.
//!
//! [`DocumentShapeSerializer`]: super::DocumentShapeSerializer

use std::str::FromStr;
use std::sync::Arc;

use aws_smithy_types::date_time::Format;
use aws_smithy_types::{
    BigDecimal, BigInteger, Blob, DateTime, Document, DocumentSettings, Number,
};

use crate::serde::{capped_container_size, SerdeError, ShapeDeserializer};
use crate::Schema;

/// Walks a [`Document`] tree via the [`ShapeDeserializer`] interface.
///
/// See the module-level documentation for an overview.
///
/// # Example
///
/// Use [`DiscriminatedDocument::as_shape`](crate::document::DiscriminatedDocumentExt::as_shape)
/// for the common case of consuming a [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument)
/// via a generated `deserialize` method:
///
/// ```ignore
/// use aws_smithy_schema::document::DiscriminatedDocumentExt;
///
/// let person: Person = doc.as_shape(|deser| Person::deserialize(deser))?;
/// ```
///
/// Direct construction is useful when consuming a sub-document outside
/// the standard entry point:
///
/// ```ignore
/// use aws_smithy_schema::document::DocumentShapeDeserializer;
/// use aws_smithy_schema::serde::ShapeDeserializer;
///
/// let mut deser = DocumentShapeDeserializer::new(&doc);
/// let s = deser.read_string(&aws_smithy_schema::prelude::STRING)?;
/// ```
#[derive(Debug)]
pub struct DocumentShapeDeserializer<'a> {
    /// The document this deserializer is currently positioned at.
    cursor: &'a Document,
    /// Optional codec settings used for format-aware coercion when
    /// the cursor doesn't match a native variant. For example: a
    /// document parsed from JSON wire bytes encodes blobs as strings
    /// (base64) and may encode timestamps as strings or numbers; with
    /// settings attached, [`Self::read_blob`] / [`Self::read_timestamp`]
    /// fall back through [`DocumentSettings::coerce_string_to_blob`] /
    /// [`DocumentSettings::coerce_string_to_timestamp`] /
    /// [`DocumentSettings::coerce_number_to_timestamp`] when the
    /// native variant isn't present.
    ///
    /// `None` for serialize-side documents (e.g. those produced by
    /// [`super::DocumentShapeSerializer`]) that always carry native
    /// variants — no coercion required.
    settings: Option<Arc<dyn DocumentSettings>>,
}

impl<'a> DocumentShapeDeserializer<'a> {
    /// Creates a deserializer positioned at the given document.
    ///
    /// The lifetime parameter is the borrow lifetime of `document` —
    /// the cursor is just a `&Document`. [`Document`] itself has no
    /// lifetime parameter (it is fully owned), so this `'a` is purely
    /// the per-call borrow lifetime.
    ///
    /// No format-aware coercion is performed; this deserializer is
    /// variant-only. For coercion of wire-encoded blobs and timestamps
    /// (e.g. JSON's base64-string blobs, epoch-seconds-number
    /// timestamps), use [`Self::new_with_settings`].
    pub fn new(document: &'a Document) -> Self {
        Self {
            cursor: document,
            settings: None,
        }
    }

    /// Creates a deserializer positioned at the given document with
    /// codec settings attached.
    ///
    /// With settings present, [`Self::read_blob`] and
    /// [`Self::read_timestamp`] consult the settings to coerce
    /// non-native variants — JSON wire bytes encode blobs as
    /// base64 strings and may encode timestamps as strings or
    /// numbers, so a document parsed from JSON via
    /// [`crate::codec::Codec::create_deserializer`] needs settings
    /// for those round-trips to succeed.
    pub fn new_with_settings(
        document: &'a Document,
        settings: Option<Arc<dyn DocumentSettings>>,
    ) -> Self {
        Self {
            cursor: document,
            settings,
        }
    }
}

/// Interprets a [`Number`] as epoch seconds, retaining fractional
/// seconds when present.
///
/// The deterministic default used by [`read_timestamp`](DocumentShapeDeserializer::read_timestamp)
/// when no protocol settings are attached; the inverse of what
/// [`DocumentShapeSerializer`](super::DocumentShapeSerializer) writes
/// for a `timestamp` shape.
fn number_to_timestamp(n: &Number) -> Result<DateTime, SerdeError> {
    match n {
        Number::PosInt(v) => i64::try_from(*v)
            .map(DateTime::from_secs)
            .map_err(|_| SerdeError::custom(format!("epoch seconds {v} out of range"))),
        Number::NegInt(v) => Ok(DateTime::from_secs(*v)),
        Number::Float(v) => {
            if !v.is_finite() {
                return Err(SerdeError::custom(format!(
                    "epoch seconds {v} is not finite"
                )));
            }
            Ok(DateTime::from_secs_f64(*v))
        }
    }
}

/// Builds a `TypeMismatch` error message for a read that expected one
/// kind of value and found another.
fn type_mismatch(expected: &str, found: &Document) -> SerdeError {
    SerdeError::type_mismatch(format!(
        "expected {expected} document, got {}",
        kind_name(found)
    ))
}

/// Human-readable name for a [`Document`] variant, used in error
/// messages for type-mismatch diagnostics.
fn kind_name(d: &Document) -> &'static str {
    match d {
        Document::Null => "null",
        Document::Bool(_) => "boolean",
        Document::Number(_) => "number",
        Document::String(_) => "string",
        Document::Array(_) => "list",
        Document::Object(_) => "map",
    }
}

/// Resolves a wire-level map key to a member of `schema`, matching
/// against [`Schema::member_name`] directly.
///
/// Wire-name resolution for `@jsonName` / `@xmlName`-style renames is
/// the responsibility of the protocol's deserializer (which has access
/// to the codec settings); this generic Document walker matches Smithy
/// member names only. Documents produced by [`DocumentShapeSerializer`](super::DocumentShapeSerializer)
/// always use Smithy member names, so the round-trip is exact.
fn resolve_member<'s>(schema: &'s Schema<'s>, wire_name: &str) -> Option<&'s Schema<'s>> {
    let idx = schema
        .members()
        .iter()
        .position(|m| m.member_name() == Some(wire_name))?;
    schema.member_schema_by_index(idx)
}

// -- Numeric coercion ------------------------------------------------
//
// [`Document`] carries exactly the six JSON-shaped variants published in
// `aws-smithy-types`, and nothing more: no numeric-coercion accessors,
// no arbitrary-precision accessors. Every Smithy-numeric coercion
// therefore lives here, on the schema side, where the schema has already
// established which Smithy type the caller is asking for.
//
// The signed-integer coercions accept the bounded integer variants of
// [`Number`] (`PosInt` / `NegInt`) and report overflow. They also accept a
// `Number::Float` whose value is an integer that fits the target (`42.0`
// reads as `42`), because `DiscriminatedDocument::shape_type` reports such
// a value as `integer` or `long` per the SEP's ambiguous-number rule, and a
// reported type must be readable. A float with a fractional part, or a
// non-finite one, is rejected as lossy rather than truncated: the SEP's
// "ignore loss of precision" wording would allow truncation, but legacy JSON
// deserialization rejects it, and silently dropping digits is the riskier
// choice. The reverse direction is lossless and is allowed —
// [`coerce_float`] / [`coerce_double`] take integer sources.
//
// `Document` has no arbitrary-precision variant either, so the
// schema-driven legacy representation carries `bigInteger` /
// `bigDecimal` as [`Document::String`]. [`coerce_big_integer`] /
// [`coerce_big_decimal`] accept a string source for exactly that
// reason: they are only reached once a schema has established the
// target shape is arbitrary-precision.

/// Coerces a numeric [`Document`] to a signed integer target type.
///
/// Used by [`ShapeDeserializer::read_byte`] / `read_short` /
/// `read_integer` / `read_long`.
///
/// The generic `T: TryFrom<i64> + TryFrom<u64>` bounds route
/// `Number::PosInt(u64)` and `Number::NegInt(i64)` through the standard
/// library's range-checked narrowing impls; no bespoke arithmetic.
///
/// A `Number::Float` is accepted when it is an integer within range (see
/// [`float_as_i64`]); a fractional or non-finite float produces
/// [`SerdeError::NumericCoercionLossy`]. Out-of-range sources produce
/// [`SerdeError::NumericCoercionOverflow`].
fn coerce_signed<T>(doc: &Document, name: &str) -> Result<T, SerdeError>
where
    T: TryFrom<i64> + TryFrom<u64>,
{
    match doc {
        Document::Number(Number::PosInt(v)) => {
            T::try_from(*v).map_err(|_| SerdeError::numeric_coercion_overflow(name, v.to_string()))
        }
        Document::Number(Number::NegInt(v)) => {
            T::try_from(*v).map_err(|_| SerdeError::numeric_coercion_overflow(name, v.to_string()))
        }
        Document::Number(Number::Float(v)) => {
            if !is_integral(*v) {
                return Err(SerdeError::numeric_coercion_lossy(name, v.to_string()));
            }
            float_as_i64(*v)
                .and_then(|i| T::try_from(i).ok())
                .ok_or_else(|| SerdeError::numeric_coercion_overflow(name, v.to_string()))
        }
        other => Err(type_mismatch(name, other)),
    }
}

/// Whether `v` is finite and has no fractional part.
fn is_integral(v: f64) -> bool {
    v.is_finite() && v.fract() == 0.0
}

/// The value of `v` as an `i64`, when `v` is an integer within `i64`'s range.
///
/// The upper bound is exclusive and written as 2⁶³ because `i64::MAX as f64` rounds *up* to
/// 2⁶³, which is one past `i64::MAX`. `i64::MIN` (-2⁶³) is exactly representable, so the lower
/// bound is inclusive. Shared with `DiscriminatedDocument::shape_type` so the reported type and
/// the reader agree.
pub(crate) fn float_as_i64(v: f64) -> Option<i64> {
    const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;
    (is_integral(v) && (-TWO_POW_63..TWO_POW_63).contains(&v)).then_some(v as i64)
}

/// Coerces a numeric [`Document`] to `f64`.
///
/// Accepts every numeric variant: integer sources widen losslessly.
fn coerce_double(doc: &Document) -> Result<f64, SerdeError> {
    match doc {
        Document::Number(Number::PosInt(v)) => Ok(*v as f64),
        Document::Number(Number::NegInt(v)) => Ok(*v as f64),
        Document::Number(Number::Float(f)) => Ok(*f),
        other => Err(type_mismatch("double", other)),
    }
}

/// Coerces a numeric [`Document`] to `f32`.
///
/// `Number::Float` is already `f64` and is narrowed by `as` cast
/// (precision loss accepted per SEP).
fn coerce_float(doc: &Document) -> Result<f32, SerdeError> {
    Ok(coerce_double(doc)? as f32)
}

/// Coerces a [`Document`] to a [`BigInteger`].
///
/// Coerces from `Number::PosInt` / `Number::NegInt` by string-formatting
/// (always lossless). A `Number::Float` is accepted when its value is an
/// integer, at any magnitude, since every finite integral `f64` has an exact
/// decimal form; a fractional or non-finite float produces
/// [`SerdeError::NumericCoercionLossy`].
///
/// A [`Document::String`] source is accepted because the schema-driven
/// legacy representation of an arbitrary-precision shape is its numeric
/// text. A string holding a decimal point or exponent is truncated
/// toward zero, expanding scientific notation so the integer magnitude
/// is preserved.
fn coerce_big_integer(doc: &Document) -> Result<BigInteger, SerdeError> {
    match doc {
        Document::Number(Number::PosInt(v)) => parse_big_integer(&v.to_string()),
        Document::Number(Number::NegInt(v)) => parse_big_integer(&v.to_string()),
        Document::Number(Number::Float(v)) => {
            if !is_integral(*v) {
                return Err(SerdeError::numeric_coercion_lossy(
                    "bigInteger",
                    v.to_string(),
                ));
            }
            // `{:.0}` prints every digit of an integral `f64` (no exponent); `0.0`
            // keeps `-0.0` from printing as "-0".
            parse_big_integer(&format!("{:.0}", v + 0.0))
        }
        Document::String(s) => {
            if let Ok(bi) = BigInteger::from_str(s) {
                return Ok(bi);
            }
            // Not integral text — try the decimal grammar and truncate
            // toward zero, expanding any scientific-notation exponent so
            // the magnitude is preserved. A naive split at the first
            // '.' / 'e' / 'E' produces a silent wrong value (e.g.
            // "1.23e10" -> "1"); dropping only the fractional digits is
            // the SEP's "ignore loss of precision" rule.
            let bd = BigDecimal::from_str(s).map_err(|e| invalid_input("bigInteger", s, &e))?;
            // `BigDecimal::from_str` guarantees the decimal grammar, which
            // is exactly what the magnitude algorithm needs. Re-check it
            // here anyway: `big_decimal_to_integer_string` only
            // `debug_assert!`s the precondition, so in release builds a
            // future relaxation of `FromStr` would silently produce a
            // wrong value rather than an error.
            if !is_unambiguous_decimal(bd.as_ref()) {
                return Err(SerdeError::invalid_input(format!(
                    "cannot coerce {:?} to bigInteger: not a decimal number",
                    bd.as_ref()
                )));
            }
            let int_part = big_decimal_to_integer_string(&bd).ok_or_else(|| {
                SerdeError::custom(format!(
                    "cannot coerce bigDecimal {} to bigInteger: integer magnitude too large",
                    bd.as_ref()
                ))
            })?;
            parse_big_integer(&int_part)
        }
        other => Err(type_mismatch("bigInteger", other)),
    }
}

/// Coerces a [`Document`] to a [`BigDecimal`].
///
/// Coerces from [`Number`] by string-formatting (`Number::Float`'s
/// `to_string` produces a `BigDecimal`-parseable form). A
/// [`Document::String`] source is accepted for the same reason as in
/// [`coerce_big_integer`].
fn coerce_big_decimal(doc: &Document) -> Result<BigDecimal, SerdeError> {
    match doc {
        Document::String(s) => parse_big_decimal(s),
        Document::Number(Number::PosInt(v)) => parse_big_decimal(&v.to_string()),
        Document::Number(Number::NegInt(v)) => parse_big_decimal(&v.to_string()),
        Document::Number(Number::Float(f)) => {
            if !f.is_finite() {
                return Err(SerdeError::custom(format!(
                    "cannot coerce non-finite float {f} to bigDecimal"
                )));
            }
            parse_big_decimal(&f.to_string())
        }
        other => Err(type_mismatch("bigDecimal", other)),
    }
}

/// Parses integral text into a [`BigInteger`], reporting malformed input
/// as [`SerdeError::InvalidInput`].
fn parse_big_integer(s: &str) -> Result<BigInteger, SerdeError> {
    BigInteger::from_str(s).map_err(|e| invalid_input("bigInteger", s, &e))
}

/// Parses decimal text into a [`BigDecimal`], reporting malformed input
/// as [`SerdeError::InvalidInput`].
fn parse_big_decimal(s: &str) -> Result<BigDecimal, SerdeError> {
    BigDecimal::from_str(s).map_err(|e| invalid_input("bigDecimal", s, &e))
}

/// Builds an `InvalidInput` error for arbitrary-precision parse
/// failures.
fn invalid_input(target: &str, value: &str, err: &dyn std::fmt::Display) -> SerdeError {
    SerdeError::invalid_input(format!("cannot parse {value:?} as {target}: {err}"))
}

/// `true` iff `s` is non-empty and consists entirely of ASCII digits.
fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// `true` iff `text` is a decimal number [`big_decimal_to_integer_string`]
/// can read unambiguously:
/// `('+' | '-')? (digits ('.' digits*)? | '.' digits+)`
/// `(('e' | 'E') ('+' | '-')? digits+)?`.
///
/// This is the same structural grammar [`BigDecimal`]'s `FromStr` enforces,
/// including leading signs and omitted integer or fractional digit runs. It is
/// kept as an explicit precondition check because [`big_decimal_to_integer_string`]
/// would return a silently wrong value rather than an error on text with no
/// single numeric reading (`"1.2.3"` would truncate to `"1"`), and its own
/// guard is a `debug_assert!` that disappears in release builds.
///
/// Deliberately *permissive* about leading zeros, matching `FromStr`:
/// `"00123"` has one reading, and the algorithm normalizes it to `"123"`.
/// (The stricter RFC 8259 check, which does forbid leading zeros, belongs
/// on the wire boundary in the JSON codec's serializer — there the
/// constraint is the output format, not readability.)
fn is_unambiguous_decimal(text: &str) -> bool {
    let rest = text.strip_prefix(['-', '+']).unwrap_or(text);

    let (mantissa, exponent) = match rest.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (rest, None),
    };

    let mantissa_ok = match mantissa.split_once('.') {
        Some((int_part, frac_part)) => {
            (!int_part.is_empty() || !frac_part.is_empty())
                && int_part.bytes().all(|b| b.is_ascii_digit())
                && frac_part.bytes().all(|b| b.is_ascii_digit())
        }
        None => is_ascii_digits(mantissa),
    };

    let exponent_ok = match exponent {
        None => true,
        Some(exponent) => is_ascii_digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)),
    };

    mantissa_ok && exponent_ok
}

/// Upper bound on the length of the integer string
/// [`big_decimal_to_integer_string`] will materialize. A
/// scientific-notation exponent can request an arbitrarily long run of
/// trailing zeros; this cap stops a pathological exponent (e.g.
/// `1e1000000000`) from triggering a huge allocation. ~1M digits is far
/// beyond any practical value.
const MAX_INTEGER_DIGITS: usize = 1 << 20;

/// Returns `bd` truncated toward zero as a plain integer string
/// (`-?[0-9]+`), or `None` if the integer magnitude is too large to
/// materialize (see [`MAX_INTEGER_DIGITS`]).
///
/// Fractional digits are dropped — per the SEP, numeric coercion ignores
/// loss of precision — and any scientific-notation exponent is expanded
/// so the integer magnitude is preserved. For example `1.23e10` yields
/// `"12300000000"`, `1.23e1` yields `"12"`, and `5e-3` yields `"0"`.
///
/// # Precondition
///
/// `bd` must satisfy [`is_unambiguous_decimal`]. That is implied by
/// `BigDecimal::from_str` today, which enforces the same grammar, but this
/// function returns a value rather than an error when given text with no
/// single numeric reading — so the precondition is checked explicitly by
/// the caller and only `debug_assert!`ed here.
fn big_decimal_to_integer_string(bd: &BigDecimal) -> Option<String> {
    debug_assert!(
        is_unambiguous_decimal(bd.as_ref()),
        "big_decimal_to_integer_string requires decimal text, got {:?}",
        bd.as_ref()
    );
    let text = bd.as_ref();
    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };

    let (mantissa, exp) = match rest.split_once(['e', 'E']) {
        Some((mantissa, exp_str)) => match exp_str.parse::<i64>() {
            Ok(exp) => (mantissa, exp),
            // Exponent doesn't fit in i64: a huge positive exponent is
            // too large to materialize. `BigDecimal::from_str`'s scale bound
            // prevents an i64-underflowing negative exponent from reaching
            // this branch today; retain the negative case as defense in depth.
            Err(_) => {
                return if exp_str.starts_with('-') {
                    Some("0".to_string())
                } else {
                    None
                }
            }
        },
        None => (rest, 0),
    };

    let (int_digits, frac_digits) = match mantissa.split_once('.') {
        Some((int_digits, frac_digits)) => (int_digits, frac_digits),
        None => (mantissa, ""),
    };

    // Position of the decimal point from the left of the combined
    // `int_digits ++ frac_digits` run, shifted right by the exponent.
    // `saturating_add` keeps a near-`i64::MAX` exponent from overflowing
    // (it then trips the size cap below).
    let point = (int_digits.len() as i64).saturating_add(exp);

    let int_part = if point <= 0 {
        // The whole value is fractional.
        "0".to_string()
    } else {
        let point = point as usize;
        let mut digits = String::with_capacity(int_digits.len() + frac_digits.len());
        digits.push_str(int_digits);
        digits.push_str(frac_digits);

        if point >= digits.len() {
            // Decimal point at or beyond the last digit: pad with zeros.
            if point > MAX_INTEGER_DIGITS {
                return None;
            }
            digits.push_str(&"0".repeat(point - digits.len()));
            digits
        } else {
            // Decimal point falls within the digit run; drop the rest.
            digits.truncate(point);
            digits
        }
    };

    // Normalize leading zeros, keeping at least one digit.
    let normalized = match int_part.trim_start_matches('0') {
        "" => "0",
        trimmed => trimmed,
    };

    Some(if negative && normalized != "0" {
        format!("-{normalized}")
    } else {
        normalized.to_string()
    })
}

impl<'a> ShapeDeserializer for DocumentShapeDeserializer<'a> {
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let map = self
            .cursor
            .as_object()
            .ok_or_else(|| type_mismatch("struct (map)", self.cursor))?;
        for (key, value) in map {
            let Some(member_schema) = resolve_member(schema, key) else {
                // Unknown member — silently ignore. Matches the
                // tolerant "ignore unknown fields" behavior of the
                // JSON deserializer.
                continue;
            };
            let mut sub = Self::new_with_settings(value, self.settings.clone());
            consumer(member_schema, &mut sub)?;
        }
        Ok(())
    }

    fn read_list(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let items = self
            .cursor
            .as_array()
            .ok_or_else(|| type_mismatch("list", self.cursor))?;
        for item in items {
            let mut sub = Self::new_with_settings(item, self.settings.clone());
            consumer(&mut sub)?;
        }
        Ok(())
    }

    fn read_map(
        &mut self,
        _schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let entries = self
            .cursor
            .as_object()
            .ok_or_else(|| type_mismatch("map", self.cursor))?;
        for (key, value) in entries {
            let mut sub = Self::new_with_settings(value, self.settings.clone());
            consumer(key.clone(), &mut sub)?;
        }
        Ok(())
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.cursor
            .as_bool()
            .ok_or_else(|| type_mismatch("boolean", self.cursor))
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        coerce_signed::<i8>(self.cursor, "byte")
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        coerce_signed::<i16>(self.cursor, "short")
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        coerce_signed::<i32>(self.cursor, "integer")
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        coerce_signed::<i64>(self.cursor, "long")
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        coerce_float(self.cursor)
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        coerce_double(self.cursor)
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        // `Document` has no arbitrary-precision variant: `coerce_big_integer`
        // widens the numeric variants and reverses the legacy string
        // representation that `DocumentShapeSerializer` writes.
        coerce_big_integer(self.cursor)
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        coerce_big_decimal(self.cursor)
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        match self.cursor {
            Document::String(s) => Ok(s.clone()),
            other => Err(type_mismatch("string", other)),
        }
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        // `Document` has no blob variant. The legacy representation of a
        // `blob` shape is a base64-encoded string.
        //
        // With settings attached, the codec decides the decoding (e.g.
        // the JSON codec's `coerce_string_to_blob`). With no settings —
        // the case for a document built by `DocumentShapeSerializer`
        // outside a protocol context — fall back to standard base64,
        // which is exactly what the serializer wrote.
        match (self.cursor, &self.settings) {
            (Document::String(s), Some(settings)) => settings
                .coerce_string_to_blob(s)
                .map(Blob::new)
                .map_err(SerdeError::from),
            (Document::String(s), None) => aws_smithy_types::base64::decode(s)
                .map(Blob::new)
                .map_err(|e| {
                    SerdeError::custom(format!("cannot base64-decode string as blob: {e}"))
                }),
            (other, _) => Err(type_mismatch("blob", other)),
        }
    }

    fn read_timestamp(&mut self, _schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        // `Document` has no timestamp variant. The legacy representation
        // written by `DocumentShapeSerializer` is epoch seconds as a
        // number; a document parsed off a JSON-family wire may instead
        // hold a formatted string.
        //
        // With settings attached, the codec's `DocumentSettings` supplies
        // the format and the right `coerce_*_to_timestamp` is dispatched
        // by source variant. With no settings, fall back to the
        // deterministic defaults: epoch seconds for a number, RFC-3339
        // for a string.
        match (self.cursor, &self.settings) {
            (Document::Number(n), Some(settings)) => settings
                .coerce_number_to_timestamp(n)
                .map_err(SerdeError::from),
            (Document::Number(n), None) => number_to_timestamp(n),
            (Document::String(s), Some(settings)) => settings
                .coerce_string_to_timestamp(s)
                .map_err(SerdeError::from),
            (Document::String(s), None) => DateTime::from_str(s, Format::DateTime).map_err(|e| {
                SerdeError::custom(format!("cannot parse string as a date-time timestamp: {e}"))
            }),
            (other, _) => Err(type_mismatch("timestamp", other)),
        }
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<Document, SerdeError> {
        Ok(self.cursor.clone())
    }

    fn is_null(&self) -> bool {
        matches!(self.cursor, Document::Null)
    }

    /// Nothing to advance. The cursor points at an already-parsed node of an owned
    /// document tree, and `read_struct` hands each member its own sub-deserializer, so
    /// declining a value has no effect on the parent's iteration.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        let raw = match self.cursor {
            Document::Array(items) => items.len(),
            Document::Object(entries) => entries.len(),
            _ => return None,
        };
        Some(capped_container_size(raw))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::document::DocumentShapeSerializer;
    use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
    use crate::{prelude, shape_id, Schema, ShapeId, ShapeType};

    use aws_smithy_types::Number;

    // -- Test schemas ----------------------------------------------------

    const PERSON_ID: ShapeId<'static> = shape_id!("smithy.example", "Person");
    const PERSON_NAME_ID: ShapeId<'static> = shape_id!("smithy.example", "Person", "name");
    const PERSON_AGE_ID: ShapeId<'static> = shape_id!("smithy.example", "Person", "age");

    static PERSON_NAME_MEMBER: Schema<'static> =
        Schema::new_member(PERSON_NAME_ID, ShapeType::String, "name", 0);
    static PERSON_AGE_MEMBER: Schema<'static> =
        Schema::new_member(PERSON_AGE_ID, ShapeType::Integer, "age", 1);
    static PERSON_SCHEMA: Schema<'static> = Schema::new_struct(
        PERSON_ID,
        ShapeType::Structure,
        &[&PERSON_NAME_MEMBER, &PERSON_AGE_MEMBER],
    );

    /// Test struct + builder pair to drive `read_struct` consumer
    /// dispatch the same way generated code does.
    #[derive(Debug, Default, PartialEq)]
    struct Person {
        name: Option<String>,
        age: Option<i32>,
    }

    impl SerializableStruct for Person {
        fn schema(&self) -> &Schema<'_> {
            &PERSON_SCHEMA
        }

        fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            if let Some(n) = &self.name {
                ser.write_string(&PERSON_NAME_MEMBER, n)?;
            }
            if let Some(a) = self.age {
                ser.write_integer(&PERSON_AGE_MEMBER, a)?;
            }
            Ok(())
        }
    }

    fn deserialize_person(deser: &mut dyn ShapeDeserializer) -> Result<Person, SerdeError> {
        let mut out = Person::default();
        deser.read_struct(&PERSON_SCHEMA, &mut |member, sub| {
            match member.member_index() {
                Some(0) => out.name = Some(sub.read_string(member)?),
                Some(1) => out.age = Some(sub.read_integer(member)?),
                _ => {}
            }
            Ok(())
        })?;
        Ok(out)
    }

    // -- Scalars ---------------------------------------------------------

    #[test]
    fn read_string_returns_value() {
        let doc = Document::String("hello".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.read_string(&prelude::STRING).unwrap(), "hello");
    }

    #[test]
    fn read_string_on_non_string_errors() {
        let doc = Document::Number(Number::PosInt(1));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_string(&prelude::STRING).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    #[test]
    fn read_integer_with_coercion() {
        // Integer narrowing follows SEP rules (precision loss ignored).
        let doc = Document::Number(Number::PosInt(42));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.read_integer(&prelude::INTEGER).unwrap(), 42);
    }

    #[test]
    fn read_integer_overflow_errors() {
        let doc = Document::Number(Number::PosInt(u64::MAX));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_integer(&prelude::INTEGER).unwrap_err();
        assert!(matches!(err, SerdeError::NumericCoercionOverflow { .. }));
    }

    #[test]
    fn read_boolean_returns_value() {
        let doc = Document::Bool(true);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.read_boolean(&prelude::BOOLEAN).unwrap());
    }

    #[test]
    fn read_blob_decodes_base64_string_without_settings() {
        // `Document` has no blob variant: the legacy representation is a
        // base64 string, and with no settings attached the deserializer
        // decodes it with standard base64 — exactly what
        // `DocumentShapeSerializer::write_blob` wrote.
        let doc = Document::String("YWJjZA==".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let blob = deser.read_blob(&prelude::BLOB).unwrap();
        assert_eq!(blob.as_ref(), b"abcd");
    }

    #[test]
    fn read_blob_on_malformed_base64_errors() {
        let doc = Document::String("not base64!!!".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_blob(&prelude::BLOB).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("base64"), "unexpected error message: {msg}");
    }

    #[test]
    fn read_blob_on_non_string_errors() {
        let doc = Document::Number(Number::PosInt(1));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_blob(&prelude::BLOB).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    #[test]
    fn read_timestamp_defaults_to_epoch_seconds_without_settings() {
        let doc = Document::Number(Number::PosInt(1234));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser.read_timestamp(&prelude::TIMESTAMP).unwrap(),
            DateTime::from_secs(1234)
        );

        let doc = Document::Number(Number::NegInt(-9));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser.read_timestamp(&prelude::TIMESTAMP).unwrap(),
            DateTime::from_secs(-9)
        );
    }

    #[test]
    fn read_timestamp_retains_fractional_seconds() {
        let doc = Document::Number(Number::Float(1234.5));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser.read_timestamp(&prelude::TIMESTAMP).unwrap(),
            DateTime::from_secs_f64(1234.5)
        );
    }

    #[test]
    fn read_timestamp_parses_date_time_string_without_settings() {
        let doc = Document::String("1970-01-01T00:00:00Z".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser.read_timestamp(&prelude::TIMESTAMP).unwrap(),
            DateTime::from_secs(0)
        );
    }

    #[test]
    fn read_timestamp_on_non_coercible_variant_errors() {
        let doc = Document::Bool(true);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deser.read_timestamp(&prelude::TIMESTAMP).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    #[test]
    fn read_big_integer_and_decimal_reverse_the_string_representation() {
        // The legacy representation of an arbitrary-precision shape is
        // its numeric text, so precision beyond f64's range survives.
        let big = "12345678901234567890123456789";
        let doc = Document::String(big.to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap()
                .as_ref(),
            big
        );

        let dec = "12345678901234567890123456789.25";
        let doc = Document::String(dec.to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(
            deser
                .read_big_decimal(&prelude::BIG_DECIMAL)
                .unwrap()
                .as_ref(),
            dec
        );
    }

    #[test]
    fn is_null_for_null_document() {
        let doc = Document::Null;
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.is_null());
    }

    #[test]
    fn is_null_false_for_non_null() {
        let doc = Document::String("not-null".to_string());
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(!deser.is_null());
    }

    // -- Aggregates ------------------------------------------------------

    #[test]
    fn read_list_iterates_elements() {
        let doc = Document::Array(vec![
            Document::String("a".to_string()),
            Document::String("b".to_string()),
            Document::String("c".to_string()),
        ]);
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut collected = Vec::new();
        deser
            .read_list(&prelude::DOCUMENT, &mut |sub| {
                collected.push(sub.read_string(&prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected, ["a", "b", "c"]);
    }

    #[test]
    fn read_map_iterates_entries() {
        let doc = Document::Object(HashMap::from([
            ("k1".to_string(), Document::String("v1".to_string())),
            ("k2".to_string(), Document::String("v2".to_string())),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut collected = HashMap::new();
        deser
            .read_map(&prelude::DOCUMENT, &mut |key, sub| {
                collected.insert(key, sub.read_string(&prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(collected.get("k1").map(String::as_str), Some("v1"));
        assert_eq!(collected.get("k2").map(String::as_str), Some("v2"));
    }

    #[test]
    fn container_size_on_list() {
        let doc = Document::Array(vec![Document::Null; 5]);
        let deser = DocumentShapeDeserializer::new(&doc);
        assert_eq!(deser.container_size(), Some(5));
    }

    #[test]
    fn container_size_on_scalar_is_none() {
        let doc = Document::String("foo".to_string());
        let deser = DocumentShapeDeserializer::new(&doc);
        assert!(deser.container_size().is_none());
    }

    // -- Struct round-trip ----------------------------------------------

    #[test]
    fn read_struct_with_consumer_dispatch() {
        let doc = Document::Object(HashMap::from([
            ("name".to_string(), Document::String("Alex".to_string())),
            ("age".to_string(), Document::Number(Number::PosInt(30))),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(
            person,
            Person {
                name: Some("Alex".into()),
                age: Some(30),
            }
        );
    }

    #[test]
    fn read_struct_with_missing_optional_member() {
        // Document only has `name`; `age` is missing.
        let doc = Document::Object(HashMap::from([(
            "name".to_string(),
            Document::String("Sam".to_string()),
        )]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(
            person,
            Person {
                name: Some("Sam".into()),
                age: None,
            }
        );
    }

    #[test]
    fn read_struct_ignores_unknown_members() {
        let doc = Document::Object(HashMap::from([
            ("name".to_string(), Document::String("Joe".to_string())),
            (
                "unknown_field".to_string(),
                Document::String("ignored".to_string()),
            ),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let person = deserialize_person(&mut deser).unwrap();
        assert_eq!(person.name.as_deref(), Some("Joe"));
    }

    #[test]
    fn read_struct_on_non_map_errors() {
        let doc = Document::String("not-a-struct".to_string());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let err = deserialize_person(&mut deser).unwrap_err();
        assert!(matches!(err, SerdeError::TypeMismatch { .. }));
    }

    // -- Round-trip with the serializer ---------------------------------

    #[test]
    fn round_trip_through_document() {
        let original = Person {
            name: Some("Iago".into()),
            age: Some(7),
        };
        // serialize → DiscriminatedDocument
        let mut ser = DocumentShapeSerializer::new();
        ser.write_struct(&PERSON_SCHEMA, &original).unwrap();
        let doc = ser.finish().unwrap();
        // deserialize the inner Document → typed
        let mut deser = DocumentShapeDeserializer::new(doc.document());
        let restored = deserialize_person(&mut deser).unwrap();
        assert_eq!(restored, original);
        // discriminator is preserved at the wrapper level
        assert_eq!(doc.discriminator(), Some("smithy.example#Person"));
    }

    // -- Numeric coercion (moved off `Document`) -------------------------
    //
    // These exercise the private coercion helpers in this module through
    // the `ShapeDeserializer` surface. The logic used to live on
    // `Document` itself as additive `as_byte` / `coerce_big_integer`
    // accessors; `Document` is now exactly the released type, so the
    // coercion — and its coverage — lives here.

    /// Convenience: build a deserializer over a numeric document.
    fn num(n: Number) -> Document {
        Document::Number(n)
    }

    #[test]
    fn read_signed_integers_coerce_across_bounded_integer_sources() {
        let doc = num(Number::PosInt(42));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_byte(&prelude::BYTE)
                .unwrap(),
            42
        );
        let doc = num(Number::NegInt(-42));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_byte(&prelude::BYTE)
                .unwrap(),
            -42
        );

        let doc = num(Number::PosInt(1234));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_short(&prelude::SHORT)
                .unwrap(),
            1234
        );

        let doc = num(Number::PosInt(i32::MAX as u64));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_integer(&prelude::INTEGER)
                .unwrap(),
            i32::MAX
        );
        let doc = num(Number::NegInt(i32::MIN as i64));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_integer(&prelude::INTEGER)
                .unwrap(),
            i32::MIN
        );

        let doc = num(Number::PosInt(i64::MAX as u64));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_long(&prelude::LONG)
                .unwrap(),
            i64::MAX
        );
    }

    #[test]
    fn read_byte_overflow_populates_target_and_value() {
        let doc = num(Number::PosInt(200));
        let err = DocumentShapeDeserializer::new(&doc)
            .read_byte(&prelude::BYTE)
            .unwrap_err();
        match err {
            SerdeError::NumericCoercionOverflow { target, value } => {
                assert_eq!(target, "byte");
                assert_eq!(value, "200");
            }
            other => panic!("expected NumericCoercionOverflow, got {other:?}"),
        }
    }

    #[test]
    fn read_signed_integers_overflow_at_the_edges() {
        // One below i32::MIN: `-(i32::MAX + 1) == i32::MIN` exactly, so
        // we need one further to overflow.
        let doc = num(Number::NegInt(i64::from(i32::MIN) - 1));
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_integer(&prelude::INTEGER)
                .unwrap_err(),
            SerdeError::NumericCoercionOverflow { .. }
        ));

        let doc = num(Number::PosInt(u64::MAX));
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_long(&prelude::LONG)
                .unwrap_err(),
            SerdeError::NumericCoercionOverflow { .. }
        ));
    }

    #[test]
    fn read_byte_on_non_numeric_is_type_mismatch() {
        let doc = Document::Bool(true);
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_byte(&prelude::BYTE)
                .unwrap_err(),
            SerdeError::TypeMismatch { .. }
        ));
    }

    #[test]
    fn integer_reads_accept_integral_floats_and_reject_fractional_ones() {
        let read_all = |value: f64| {
            let doc = num(Number::Float(value));
            let de = || DocumentShapeDeserializer::new(&doc);
            (
                de().read_byte(&prelude::BYTE),
                de().read_short(&prelude::SHORT),
                de().read_integer(&prelude::INTEGER),
                de().read_long(&prelude::LONG),
                de().read_big_integer(&prelude::BIG_INTEGER),
            )
        };
        let (b, sh, i, l, bi) = read_all(42.0);
        assert_eq!(
            (b.unwrap(), sh.unwrap(), i.unwrap(), l.unwrap()),
            (42, 42, 42, 42)
        );
        assert_eq!(bi.unwrap().as_ref(), "42");
        assert_eq!(read_all(-0.0).2.unwrap(), 0);
        assert_eq!(read_all(-0.0).4.unwrap().as_ref(), "0");

        // Never truncated: a fractional or non-finite float is lossy for every integer read.
        for value in [42.7, -0.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let (b, sh, i, l, bi) = read_all(value);
            for err in [b.err(), sh.err(), i.err(), l.err(), bi.err()] {
                assert!(
                    matches!(err, Some(SerdeError::NumericCoercionLossy { .. })),
                    "{value}: {err:?}"
                );
            }
        }
    }

    #[test]
    fn integral_float_range_checks_are_exact() {
        let long = |v: f64| {
            DocumentShapeDeserializer::new(&num(Number::Float(v))).read_long(&prelude::LONG)
        };
        let int = |v: f64| {
            DocumentShapeDeserializer::new(&num(Number::Float(v))).read_integer(&prelude::INTEGER)
        };
        const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0;
        // -2^63 is i64::MIN exactly; 2^63 is what `i64::MAX as f64` rounds to and is out of range.
        assert_eq!(long(-TWO_POW_63).unwrap(), i64::MIN);
        assert!(matches!(
            long(TWO_POW_63),
            Err(SerdeError::NumericCoercionOverflow { .. })
        ));
        assert_eq!(int(2_147_483_647.0).unwrap(), i32::MAX);
        assert_eq!(int(-2_147_483_648.0).unwrap(), i32::MIN);
        assert!(matches!(
            int(2_147_483_648.0),
            Err(SerdeError::NumericCoercionOverflow { .. })
        ));
        // Beyond i64, an integral float is still a bigInteger, digit for digit.
        let big = DocumentShapeDeserializer::new(&num(Number::Float(1e20)))
            .read_big_integer(&prelude::BIG_INTEGER)
            .unwrap();
        assert_eq!(big.as_ref(), "100000000000000000000");
        let big = DocumentShapeDeserializer::new(&num(Number::Float(TWO_POW_63)))
            .read_big_integer(&prelude::BIG_INTEGER)
            .unwrap();
        assert_eq!(big.as_ref(), "9223372036854775808");
    }

    #[test]
    fn read_double_and_float_widen_from_integer_sources() {
        // Integer → float IS allowed: the integer value is exactly
        // representable. Only the reverse crossover is forbidden.
        let doc = num(Number::PosInt(42));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_double(&prelude::DOUBLE)
                .unwrap(),
            42.0
        );
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_float(&prelude::FLOAT)
                .unwrap(),
            42.0_f32
        );

        let doc = num(Number::NegInt(-42));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_double(&prelude::DOUBLE)
                .unwrap(),
            -42.0
        );

        let doc = num(Number::Float(1.5));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_float(&prelude::FLOAT)
                .unwrap(),
            1.5_f32
        );
    }

    #[test]
    fn read_double_carries_non_finite_floats_intact() {
        // The codec layer owns wire-format rendering of NaN / Infinity;
        // the Document path must simply not mangle them.
        let doc = num(Number::Float(f64::NAN));
        assert!(DocumentShapeDeserializer::new(&doc)
            .read_double(&prelude::DOUBLE)
            .unwrap()
            .is_nan());

        let doc = num(Number::Float(f64::INFINITY));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_double(&prelude::DOUBLE)
                .unwrap(),
            f64::INFINITY
        );

        let doc = num(Number::Float(f64::NEG_INFINITY));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_double(&prelude::DOUBLE)
                .unwrap(),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn read_big_integer_truncates_a_decimal_string_as_text() {
        // Truncation happens *as a string*, NOT via f64 (which would
        // lose precision past 2^53).
        let doc = Document::String("12345678901234567890.123".to_owned());
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap()
                .as_ref(),
            "12345678901234567890"
        );
    }

    #[test]
    fn read_big_integer_expands_scientific_notation() {
        // Splitting at the first 'e' would return "1" for "1.23e10" —
        // off by ten orders of magnitude.
        let doc = Document::String("1.23e10".to_owned());
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap()
                .as_ref(),
            "12300000000"
        );

        let doc = Document::String("1e30".to_owned());
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap()
                .as_ref(),
            "1000000000000000000000000000000"
        );
    }

    #[test]
    fn read_big_integer_errors_on_unmaterializable_exponent() {
        // A pathological exponent must error, not allocate gigabytes or
        // return a wrong value.
        let doc = Document::String("1e1000000000".to_owned());
        let err = DocumentShapeDeserializer::new(&doc)
            .read_big_integer(&prelude::BIG_INTEGER)
            .unwrap_err();
        assert!(
            err.to_string().contains("too large"),
            "expected a 'too large' error, got: {err}"
        );
    }

    #[test]
    fn read_big_integer_rejects_non_numeric_text() {
        let doc = Document::String("not a number".to_owned());
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap_err(),
            SerdeError::InvalidInput { .. }
        ));
    }

    #[test]
    fn read_big_numbers_coerce_from_integer_sources() {
        let doc = num(Number::PosInt(7));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap()
                .as_ref(),
            "7"
        );
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_decimal(&prelude::BIG_DECIMAL)
                .unwrap()
                .as_ref(),
            "7"
        );

        let doc = num(Number::NegInt(-7));
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_decimal(&prelude::BIG_DECIMAL)
                .unwrap()
                .as_ref(),
            "-7"
        );
    }

    #[test]
    fn read_big_decimal_rejects_non_finite_floats() {
        // Distinct from `read_big_integer`: Float→BigDecimal is accepted
        // for finite values (a string-format conversion), but non-finite
        // values have no decimal form.
        let doc = num(Number::Float(f64::INFINITY));
        let err = DocumentShapeDeserializer::new(&doc)
            .read_big_decimal(&prelude::BIG_DECIMAL)
            .unwrap_err();
        match err {
            SerdeError::Custom { message } => assert!(message.contains("non-finite")),
            other => panic!("expected Custom non-finite error, got {other:?}"),
        }
    }

    #[test]
    fn big_number_reads_reject_structural_variants() {
        let doc = Document::Array(vec![]);
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap_err(),
            SerdeError::TypeMismatch { .. }
        ));
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_decimal(&prelude::BIG_DECIMAL)
                .unwrap_err(),
            SerdeError::TypeMismatch { .. }
        ));
        assert!(matches!(
            DocumentShapeDeserializer::new(&doc)
                .read_double(&prelude::DOUBLE)
                .unwrap_err(),
            SerdeError::TypeMismatch { .. }
        ));
    }

    #[test]
    fn big_integer_legacy_string_round_trips() {
        // Full circle: BigInteger -> legacy String representation ->
        // BigInteger.
        let bi = BigInteger::from_str("98765432109876543210").unwrap();
        let doc = Document::String(bi.as_ref().to_owned());
        assert_eq!(
            DocumentShapeDeserializer::new(&doc)
                .read_big_integer(&prelude::BIG_INTEGER)
                .unwrap(),
            bi
        );
    }

    // -- big_decimal_to_integer_string (moved off `BigDecimal`) ----------

    #[test]
    fn big_decimal_to_integer_string_truncates_and_expands() {
        // (input, expected truncated-toward-zero integer string)
        let cases = [
            ("0", "0"),
            ("123", "123"),
            ("123.99", "123"), // fractional digits dropped
            ("-123.99", "-123"),
            ("0.5", "0"),
            ("-0.5", "0"),              // negative zero normalizes to "0"
            ("1.23e10", "12300000000"), // exponent expanded, magnitude kept
            ("1.23e1", "12"),           // 12.3 -> 12
            ("1.5e1", "15"),
            ("1e3", "1000"),
            ("5e-3", "0"), // 0.005 -> 0
            ("-1.23e10", "-12300000000"),
            ("10.0", "10"),
            ("+123", "123"),            // explicit positive sign removed
            (".5", "0"),                // omitted integer digit run
            ("-.5", "0"),               // omitted integer digits, negative zero normalized
            ("1.", "1"),                // omitted fractional digit run
            ("+1.23e1", "12"),          // sign and exponent expansion
            ("00123", "123"),           // leading zeros normalized
            ("1.23E10", "12300000000"), // uppercase E
        ];
        for (input, expected) in cases {
            let bd = BigDecimal::from_str(input).unwrap();
            assert_eq!(
                big_decimal_to_integer_string(&bd).as_deref(),
                Some(expected),
                "big_decimal_to_integer_string({input:?})"
            );
        }
    }

    #[test]
    fn big_decimal_to_integer_string_guards_pathological_exponent() {
        // Exponent fits in i64 but is absurdly large: refuse to
        // materialize rather than allocating ~10^9 bytes.
        assert_eq!(
            big_decimal_to_integer_string(&BigDecimal::from_str("1e1000000000").unwrap()),
            None
        );
        // The exponent overflows i64, while the fraction-adjusted scale is
        // still within the range accepted by BigDecimal.
        assert_eq!(
            big_decimal_to_integer_string(
                &BigDecimal::from_str("1.0e9223372036854775808").unwrap()
            ),
            None
        );
        // An i64-underflowing negative exponent cannot reach this helper:
        // BigDecimal rejects it because its resulting scale is out of range.
    }
}

/// Tests for the [`ShapeDeserializer::skip_value`] contract.
///
/// This deserializer walks an already-parsed document tree and hands each member its own
/// sub-deserializer, so skipping must be a no-op: there is no shared cursor to advance.
#[cfg(test)]
mod skip_value_contract {
    use super::*;
    use crate::serde::ShapeDeserializer;
    use crate::{shape_id, Schema, ShapeType};
    use std::collections::HashMap;

    static NAME: Schema<'static> = Schema::new_member(
        shape_id!("smithy.example", "Person", "name"),
        ShapeType::String,
        "name",
        0,
    );
    static AGE: Schema<'static> = Schema::new_member(
        shape_id!("smithy.example", "Person", "age"),
        ShapeType::Integer,
        "age",
        1,
    );
    static PERSON: Schema<'static> = Schema::new_struct(
        shape_id!("smithy.example", "Person"),
        ShapeType::Structure,
        &[&NAME, &AGE],
    );

    #[test]
    fn skipping_one_member_leaves_the_others_readable() {
        let doc = Document::Object(HashMap::from([
            ("name".to_string(), Document::String("Alex".into())),
            (
                "age".to_string(),
                Document::Number(aws_smithy_types::Number::PosInt(30)),
            ),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut age = None;
        deser
            .read_struct(&PERSON, &mut |member, d| {
                match member.member_index() {
                    Some(0) => d.skip_value()?,
                    Some(1) => age = Some(d.read_integer(member)?),
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(age, Some(30));
    }

    #[test]
    fn skipping_an_aggregate_member_is_a_no_op() {
        // Nested content must not disturb anything, since the sub-deserializer is a
        // separate view over its own node.
        let doc = Document::Object(HashMap::from([
            (
                "name".to_string(),
                Document::Array(vec![
                    Document::String("nested".into()),
                    Document::Object(HashMap::from([(
                        "age".to_string(),
                        Document::Number(aws_smithy_types::Number::PosInt(999)),
                    )])),
                ]),
            ),
            (
                "age".to_string(),
                Document::Number(aws_smithy_types::Number::PosInt(30)),
            ),
        ]));
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let mut age = None;
        deser
            .read_struct(&PERSON, &mut |member, d| {
                match member.member_index() {
                    Some(0) => d.skip_value()?,
                    Some(1) => age = Some(d.read_integer(member)?),
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(age, Some(30), "the decoy nested `age` must not be read");
    }

    #[test]
    fn skip_value_does_not_consume_the_cursor() {
        // Skipping then reading the same value must still work: the override must not
        // have side effects.
        let doc = Document::String("still here".into());
        let mut deser = DocumentShapeDeserializer::new(&doc);
        let dynamic: &mut dyn ShapeDeserializer = &mut deser;
        dynamic.skip_value().unwrap();
        assert_eq!(deser.read_string(&NAME).unwrap(), "still here");
    }
}
