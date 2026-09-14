/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
mod de;
mod discriminated;
#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
mod doc_error;
pub mod document_object;
mod error;
#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
mod ser;
mod settings;

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
pub use de::from_document;
pub use discriminated::DiscriminatedDocument;
#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
pub use doc_error::DocError;
pub use document_object::DocumentObject;
pub use error::DocumentError;
#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
pub use ser::to_document;
pub use settings::DocumentSettings;

use crate::{BigDecimal, BigInteger, DateTime, Number};
use std::borrow::Cow;
use std::str::FromStr;

#[cfg(any(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    all(aws_sdk_unstable, feature = "serde-serialize")
))]
use serde;

/// Document Type
///
/// Document types represents protocol-agnostic open content that is accessed like JSON data.
/// Open content is useful for modeling unstructured data that has no schema, data that can't be
/// modeled using rigid types, or data that has a schema that evolves outside of the purview of a model.
/// The serialization format of a document is an implementation detail of a protocol.
///
/// `Document` represents the full Smithy data model: the JSON-shaped variants
/// ([`Document::Object`], [`Document::Array`], [`Document::Number`], [`Document::String`],
/// [`Document::Bool`], [`Document::Null`]), plus the variants for Smithy types that JSON cannot
/// natively encode ([`Document::Blob`], [`Document::Timestamp`], [`Document::BigInteger`],
/// [`Document::BigDecimal`]).
///
/// The enum is `#[non_exhaustive]`. Future Smithy data-model additions can be added without a
/// breaking change. Pattern matches that need to compile across versions should include a wildcard
/// arm.
///
/// ## Optional `serde` representation
///
/// Under the unstable `serde-serialize` / `serde-deserialize` features (with `--cfg
/// aws_sdk_unstable`), `Document` derives `serde::Serialize` / `serde::Deserialize` with
/// `#[serde(untagged)]`. Untagged deserialization resolves a value to the first variant, in
/// declaration order, whose serialized shape matches the input. The variants that have no native
/// JSON form all serialize to JSON strings — [`Document::Blob`] as base64, [`Document::Timestamp`]
/// as a date-time string, and [`Document::BigInteger`] / [`Document::BigDecimal`] as their numeric
/// text — so each deserializes back as [`Document::String`] rather than its original variant. This
/// optional representation therefore does **not** round-trip the `Blob`, `Timestamp`, `BigInteger`,
/// or `BigDecimal` variants. Faithful round-tripping of those variants is the job of a protocol
/// codec, which carries the schema or protocol settings needed to interpret the string; the caveat
/// here is specific to the `serde`-derive path and does not affect the codec (de)serializers.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    derive(serde::Deserialize)
)]
#[cfg_attr(
    any(
        all(aws_sdk_unstable, feature = "serde-deserialize"),
        all(aws_sdk_unstable, feature = "serde-serialize")
    ),
    serde(untagged)
)]
pub enum Document {
    /// JSON object
    Object(DocumentObject),
    /// JSON array
    Array(Vec<Document>),
    /// JSON number
    Number(Number),
    /// JSON string
    String(String),
    /// JSON boolean
    Bool(bool),
    /// JSON null
    Null,
    /// Smithy blob — a sequence of bytes.
    ///
    /// JSON has no native blob type and transmits blobs as base64-encoded strings; over JSON a blob
    /// member is parsed as [`Document::String`] and the schema-driven coercion path is responsible
    /// for the base64 decode. The native [`Document::Blob`] variant exists for protocols whose wire
    /// format encodes blobs natively (e.g. CBOR's byte string, major type 2) and for documents
    /// constructed directly from typed shapes.
    Blob(Vec<u8>),
    /// Smithy timestamp.
    ///
    /// JSON has no native timestamp type. As with [`Document::Blob`], the native variant is
    /// produced by protocols with a native timestamp encoding (e.g. CBOR's tag 1) and by typed
    /// shape construction; JSON-side timestamps live in [`Document::String`] or
    /// [`Document::Number`] and are parsed by the schema-driven path.
    Timestamp(DateTime),
    /// Arbitrary-precision integer.
    ///
    /// `BigInteger` is bounded only by available memory; values that fit in `i64` are typically
    /// represented as [`Document::Number`] with a `Number::PosInt` / `Number::NegInt` variant.
    /// `Document::BigInteger` is used for values outside the `i64` range and for shapes whose
    /// schema requires arbitrary precision regardless of the actual value.
    BigInteger(BigInteger),
    /// Arbitrary-precision decimal.
    ///
    /// As with [`Document::BigInteger`], values within `f64`'s representable range can use
    /// [`Document::Number`] with `Number::Float`. `Document::BigDecimal` is used for arbitrary
    /// precision regardless of magnitude.
    BigDecimal(BigDecimal),
}

impl Document {
    /// Returns the inner map value if this `Document` is an object.
    pub fn as_object(&self) -> Option<&DocumentObject> {
        if let Self::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    /// Returns the mutable inner map value if this `Document` is an object.
    pub fn as_object_mut(&mut self) -> Option<&mut DocumentObject> {
        if let Self::Object(object) = self {
            Some(object)
        } else {
            None
        }
    }

    /// Returns the inner array value if this `Document` is an array.
    pub fn as_array(&self) -> Option<&Vec<Document>> {
        if let Self::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    /// Returns the mutable inner array value if this `Document` is an array.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Document>> {
        if let Self::Array(array) = self {
            Some(array)
        } else {
            None
        }
    }

    /// Returns the inner number value if this `Document` is a number.
    pub fn as_number(&self) -> Option<&Number> {
        if let Self::Number(number) = self {
            Some(number)
        } else {
            None
        }
    }

    /// Returns the inner string value if this `Document` is a string.
    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(string) = self {
            Some(string)
        } else {
            None
        }
    }

    /// Returns the inner boolean value if this `Document` is a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(boolean) = self {
            Some(*boolean)
        } else {
            None
        }
    }

    /// Returns `Some(())` if this `Document` is a null.
    pub fn as_null(&self) -> Option<()> {
        if let Self::Null = self {
            Some(())
        } else {
            None
        }
    }

    /// Returns `true` if this `Document` is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Returns `true` if this `Document` is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// Returns `true` if this `Document` is a number.
    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    /// Returns `true` if this `Document` is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// Returns `true` if this `Document` is a bool.
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    /// Returns `true` if this `Document` is a boolean.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    // -- Accessors and predicates for the new Smithy variants -----------

    /// Returns the inner blob value if this `Document` is a [`Document::Blob`].
    ///
    /// Does not coerce — a [`Document::String`] containing base64-encoded bytes returns `None`.
    /// Coercion lives on [`DiscriminatedDocument`], which carries the protocol settings needed
    /// to disambiguate (e.g. base64-decode for JSON).
    pub fn as_blob(&self) -> Option<&[u8]> {
        if let Self::Blob(b) = self {
            Some(b.as_slice())
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner blob if this `Document` is a [`Document::Blob`].
    pub fn as_blob_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Self::Blob(b) = self {
            Some(b)
        } else {
            None
        }
    }

    /// Returns the inner timestamp if this `Document` is a [`Document::Timestamp`].
    ///
    /// Does not coerce — a [`Document::String`] containing an RFC-3339 timestamp returns `None`.
    /// Coercion lives on [`DiscriminatedDocument`], which carries the protocol settings needed
    /// to disambiguate (e.g. parse a string as a timestamp using the codec's default format).
    pub fn as_timestamp(&self) -> Option<DateTime> {
        if let Self::Timestamp(t) = self {
            Some(*t)
        } else {
            None
        }
    }

    /// Returns the inner [`BigInteger`] if this `Document` is a [`Document::BigInteger`].
    ///
    /// Pure variant check. For coercion across numeric variants (e.g. `Number(PosInt(_))` →
    /// `BigInteger`), use [`Document::coerce_big_integer`].
    pub fn as_big_integer(&self) -> Option<&BigInteger> {
        if let Self::BigInteger(bi) = self {
            Some(bi)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner [`BigInteger`] if this `Document` is a
    /// [`Document::BigInteger`].
    pub fn as_big_integer_mut(&mut self) -> Option<&mut BigInteger> {
        if let Self::BigInteger(bi) = self {
            Some(bi)
        } else {
            None
        }
    }

    /// Returns the inner [`BigDecimal`] if this `Document` is a [`Document::BigDecimal`].
    ///
    /// Pure variant check. For coercion across numeric variants, use
    /// [`Document::coerce_big_decimal`].
    pub fn as_big_decimal(&self) -> Option<&BigDecimal> {
        if let Self::BigDecimal(bd) = self {
            Some(bd)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner [`BigDecimal`] if this `Document` is a
    /// [`Document::BigDecimal`].
    pub fn as_big_decimal_mut(&mut self) -> Option<&mut BigDecimal> {
        if let Self::BigDecimal(bd) = self {
            Some(bd)
        } else {
            None
        }
    }

    /// Returns `true` if this `Document` is a blob.
    pub fn is_blob(&self) -> bool {
        matches!(self, Self::Blob(_))
    }

    /// Returns `true` if this `Document` is a timestamp.
    pub fn is_timestamp(&self) -> bool {
        matches!(self, Self::Timestamp(_))
    }

    /// Returns `true` if this `Document` is a [`BigInteger`].
    pub fn is_big_integer(&self) -> bool {
        matches!(self, Self::BigInteger(_))
    }

    /// Returns `true` if this `Document` is a [`BigDecimal`].
    pub fn is_big_decimal(&self) -> bool {
        matches!(self, Self::BigDecimal(_))
    }

    // -- Numeric coercion -----------------------------------------------
    //
    // Per the SEP "Number coercion" rules, signed-integer accessors
    // coerce across the bounded integer variants of `Number`
    // (`PosInt` / `NegInt`) and across `BigInteger` / `BigDecimal`,
    // narrowing on overflow.
    //
    // Crucially, they do **not** accept `Number::Float` — the SEP forbids
    // crossing the integer/float logical-kind boundary. `as_float` /
    // `as_double` accept integer sources losslessly going the other way.
    //
    // The arbitrary-precision coercions live as `coerce_big_integer` /
    // `coerce_big_decimal` (rather than overloading the type-checking
    // `as_big_integer` / `as_big_decimal` accessors above) so the
    // type-check vs. coercion split is visible at the call site.

    /// Coerces this document's value to a Smithy `byte` (i8).
    ///
    /// Accepts the `PosInt` / `NegInt` variants of [`Number`], plus
    /// [`BigInteger`] and [`BigDecimal`] (the latter truncates the
    /// fractional part). Returns [`DocumentError::TypeMismatch`] for
    /// `Number::Float` and non-numeric variants.
    pub fn as_byte(&self) -> Result<i8, DocumentError> {
        coerce_signed::<i8>(self, "byte")
    }

    /// Coerces this document's value to a Smithy `short` (i16). See [`Document::as_byte`].
    pub fn as_short(&self) -> Result<i16, DocumentError> {
        coerce_signed::<i16>(self, "short")
    }

    /// Coerces this document's value to a Smithy `integer` (i32). See [`Document::as_byte`].
    pub fn as_integer(&self) -> Result<i32, DocumentError> {
        coerce_signed::<i32>(self, "integer")
    }

    /// Coerces this document's value to a Smithy `long` (i64). See [`Document::as_byte`].
    pub fn as_long(&self) -> Result<i64, DocumentError> {
        coerce_signed::<i64>(self, "long")
    }

    /// Coerces this document's value to a Smithy `float` (f32).
    ///
    /// Accepts every numeric variant. Integer-source values widen to f32. `Number::Float` is
    /// already f64 and is narrowed by `as` cast (precision loss accepted per SEP). Out-of-range
    /// `BigInteger` / `BigDecimal` parses are reported as [`DocumentError::InvalidInput`].
    pub fn as_float(&self) -> Result<f32, DocumentError> {
        Ok(self.as_double()? as f32)
    }

    /// Coerces this document's value to a Smithy `double` (f64).
    ///
    /// Accepts every numeric variant. [`BigInteger`] / [`BigDecimal`] sources route through
    /// `f64::from_str` (precision loss accepted per SEP).
    pub fn as_double(&self) -> Result<f64, DocumentError> {
        match self {
            Self::Number(Number::PosInt(v)) => Ok(*v as f64),
            Self::Number(Number::NegInt(v)) => Ok(*v as f64),
            Self::Number(Number::Float(f)) => Ok(*f),
            Self::BigInteger(bi) => bi
                .as_ref()
                .parse::<f64>()
                .map_err(|e| invalid_input("double", bi.as_ref(), &e)),
            Self::BigDecimal(bd) => bd
                .as_ref()
                .parse::<f64>()
                .map_err(|e| invalid_input("double", bd.as_ref(), &e)),
            other => Err(type_mismatch_for("double", other)),
        }
    }

    /// Coerces this document's value to a [`BigInteger`].
    ///
    /// Returns the variant directly when it is already a [`BigInteger`]. Coerces from
    /// [`Number::PosInt`] / [`Number::NegInt`] by string-formatting (always lossless).
    /// `Number::Float` is rejected with [`DocumentError::TypeMismatch`] — per SEP, no
    /// integer/float crossover. `BigDecimal` sources are truncated toward zero,
    /// expanding scientific notation so the integer magnitude is preserved.
    pub fn coerce_big_integer(&self) -> Result<BigInteger, DocumentError> {
        match self {
            Self::BigInteger(bi) => Ok(bi.clone()),
            Self::Number(Number::PosInt(v)) => BigInteger::from_str(&v.to_string())
                .map_err(|e| invalid_input("bigInteger", &v.to_string(), &e)),
            Self::Number(Number::NegInt(v)) => BigInteger::from_str(&v.to_string())
                .map_err(|e| invalid_input("bigInteger", &v.to_string(), &e)),
            // No int/float crossover per SEP §"Number coercion".
            // Callers wanting a float-to-integer coercion should call `as_long` first.
            Self::Number(Number::Float(_)) => Err(type_mismatch(
                "cannot coerce float to bigInteger without explicit narrowing",
            )),
            Self::BigDecimal(bd) => {
                // Truncate toward zero, expanding any scientific-notation
                // exponent so the magnitude is preserved. A naive split at
                // the first '.' / 'e' / 'E' produced a silent wrong value
                // (e.g. "1.23e10" -> "1"); dropping only the fractional
                // digits is the SEP's "ignore loss of precision" rule.
                let int_part = bd.to_integer_string().ok_or_else(|| {
                    DocumentError::custom(format!(
                        "cannot coerce bigDecimal {} to bigInteger: integer magnitude too large",
                        bd.as_ref()
                    ))
                })?;
                BigInteger::from_str(&int_part)
                    .map_err(|e| invalid_input("bigInteger", bd.as_ref(), &e))
            }
            other => Err(type_mismatch_for("bigInteger", other)),
        }
    }

    /// Coerces this document's value to a [`BigDecimal`].
    ///
    /// Returns the variant directly when it is already a [`BigDecimal`]. Coerces from
    /// [`BigInteger`] (the BigInteger string is always a valid BigDecimal). Coerces from
    /// [`Number`] by string-formatting (`Number::Float`'s `to_string` produces a `BigDecimal`-
    /// parseable form).
    pub fn coerce_big_decimal(&self) -> Result<BigDecimal, DocumentError> {
        match self {
            Self::BigDecimal(bd) => Ok(bd.clone()),
            Self::BigInteger(bi) => BigDecimal::from_str(bi.as_ref())
                .map_err(|e| invalid_input("bigDecimal", bi.as_ref(), &e)),
            Self::Number(Number::PosInt(v)) => BigDecimal::from_str(&v.to_string())
                .map_err(|e| invalid_input("bigDecimal", &v.to_string(), &e)),
            Self::Number(Number::NegInt(v)) => BigDecimal::from_str(&v.to_string())
                .map_err(|e| invalid_input("bigDecimal", &v.to_string(), &e)),
            Self::Number(Number::Float(f)) => {
                if !f.is_finite() {
                    return Err(DocumentError::custom(format!(
                        "cannot coerce non-finite float {f} to bigDecimal"
                    )));
                }
                BigDecimal::from_str(&f.to_string())
                    .map_err(|e| invalid_input("bigDecimal", &f.to_string(), &e))
            }
            other => Err(type_mismatch_for("bigDecimal", other)),
        }
    }
}

/// The default value is `Document::Null`.
impl Default for Document {
    fn default() -> Self {
        Self::Null
    }
}

impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document::Bool(value)
    }
}

impl<'a> From<&'a str> for Document {
    fn from(value: &'a str) -> Self {
        Document::String(value.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Document {
    fn from(value: Cow<'a, str>) -> Self {
        Document::String(value.into_owned())
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document::String(value)
    }
}

impl From<Vec<Document>> for Document {
    fn from(values: Vec<Document>) -> Self {
        Document::Array(values)
    }
}

impl From<DocumentObject> for Document {
    fn from(object: DocumentObject) -> Self {
        Document::Object(object)
    }
}

impl From<std::collections::HashMap<String, Document>> for Document {
    /// Converts a [`HashMap`](std::collections::HashMap) to a [`Document::Object`].
    ///
    /// Iteration order in the resulting document follows the
    /// (unspecified) iteration order of the source `HashMap`. For
    /// callers that care about iteration order, build a
    /// [`DocumentObject`] directly with `insert` and pass it to
    /// `Document::Object` instead.
    fn from(values: std::collections::HashMap<String, Document>) -> Self {
        Document::Object(DocumentObject::from(values))
    }
}

impl From<u64> for Document {
    fn from(value: u64) -> Self {
        Document::Number(Number::PosInt(value))
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document::Number(Number::NegInt(value))
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document::Number(Number::NegInt(value as i64))
    }
}

impl From<f64> for Document {
    fn from(value: f64) -> Self {
        Document::Number(Number::Float(value))
    }
}

impl From<Number> for Document {
    fn from(value: Number) -> Self {
        Document::Number(value)
    }
}

impl<T> From<Option<T>> for Document
where
    Document: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => inner.into(),
            None => Document::Null,
        }
    }
}

// -- Helpers ---------------------------------------------------------
//
// These are private to the document module. They implement the
// SEP "Number coercion" rules and the variant-name-to-display-name
// mapping for error messages.

/// Coerces any numeric `Document` to a signed integer target type.
///
/// Used by [`Document::as_byte`] / [`Document::as_short`] /
/// [`Document::as_integer`] / [`Document::as_long`].
///
/// The target's representable range is read from
/// `T: Bounded` (which is implemented below for i8/i16/i32/i64).
/// The generic `T: TryFrom<i64> + TryFrom<u64>` bounds let us route
/// `Number::PosInt(u64)` and `Number::NegInt(i64)` through the
/// standard library's range-checked narrowing impls; no bespoke
/// arithmetic.
///
/// Per the SEP, `Number::Float` is rejected with `TypeMismatch`
/// (no integer/float crossover). Out-of-range integer sources
/// produce `NumericCoercionOverflow`.
fn coerce_signed<T>(doc: &Document, name: &str) -> Result<T, DocumentError>
where
    T: TryFrom<i64> + TryFrom<u64> + Bounded,
    f64: NarrowAs<T>,
{
    match doc {
        Document::Number(Number::PosInt(v)) => {
            T::try_from(*v).map_err(|_| overflow(name, format_args!("{v}")))
        }
        Document::Number(Number::NegInt(v)) => {
            T::try_from(*v).map_err(|_| overflow(name, format_args!("{v}")))
        }
        // No int/float crossover. Per SEP §"Number coercion": a Float
        // source must NOT silently truncate into an integer accessor.
        // Callers wanting that behavior can call `as_double()` first
        // and cast explicitly.
        Document::Number(Number::Float(_)) => Err(type_mismatch(format!(
            "cannot coerce float to {name} without explicit narrowing"
        ))),
        Document::BigInteger(bi) => {
            // Direct parse → narrow. `i64::from_str` rejects
            // arbitrarily-large bigintegers cleanly as overflow.
            let parsed = i64::from_str(bi.as_ref())
                .map_err(|_| overflow(name, format_args!("{}", bi.as_ref())))?;
            T::try_from(parsed).map_err(|_| overflow(name, format_args!("{parsed}")))
        }
        Document::BigDecimal(bd) => {
            // Per SEP "ignore precision loss" — go through f64,
            // narrow with range check. Values that overflow target
            // range produce overflow.
            let f = bd
                .as_ref()
                .parse::<f64>()
                .map_err(|e| invalid_input(name, bd.as_ref(), &e))?;
            narrow_float::<T>(f, T::MIN_F64, T::MAX_F64, name)
        }
        other => Err(type_mismatch_for(name, other)),
    }
}

/// Helper trait that exposes a target signed-integer type's
/// representable range as `f64`. Used by [`narrow_float`] for the
/// `BigDecimal` → bounded-integer coercion path.
trait Bounded {
    const MIN_F64: f64;
    const MAX_F64: f64;
}

impl Bounded for i8 {
    const MIN_F64: f64 = i8::MIN as f64;
    const MAX_F64: f64 = i8::MAX as f64;
}
impl Bounded for i16 {
    const MIN_F64: f64 = i16::MIN as f64;
    const MAX_F64: f64 = i16::MAX as f64;
}
impl Bounded for i32 {
    const MIN_F64: f64 = i32::MIN as f64;
    const MAX_F64: f64 = i32::MAX as f64;
}
impl Bounded for i64 {
    const MIN_F64: f64 = i64::MIN as f64;
    const MAX_F64: f64 = i64::MAX as f64;
}

/// Helper trait that maps a target signed integer type to the `as`
/// cast used after a range check. We define this as a trait so
/// [`coerce_signed`] can be generic over the target type.
trait NarrowAs<T> {
    fn narrow(self) -> T;
}

impl NarrowAs<i8> for f64 {
    fn narrow(self) -> i8 {
        self as i8
    }
}
impl NarrowAs<i16> for f64 {
    fn narrow(self) -> i16 {
        self as i16
    }
}
impl NarrowAs<i32> for f64 {
    fn narrow(self) -> i32 {
        self as i32
    }
}
impl NarrowAs<i64> for f64 {
    fn narrow(self) -> i64 {
        self as i64
    }
}

/// Range-checks a float against `[min, max]` and narrows to `T` via
/// the [`NarrowAs`] trait. Returns overflow on out-of-range or
/// non-finite inputs.
fn narrow_float<T>(f: f64, min: f64, max: f64, name: &str) -> Result<T, DocumentError>
where
    f64: NarrowAs<T>,
{
    if !f.is_finite() {
        return Err(DocumentError::custom(format!(
            "cannot coerce non-finite float {f} to {name}"
        )));
    }
    if !(min..=max).contains(&f) {
        return Err(overflow(name, format_args!("{f}")));
    }
    Ok(<f64 as NarrowAs<T>>::narrow(f))
}

/// Constructs a `TypeMismatch` error from a free-form message.
fn type_mismatch(message: impl Into<String>) -> DocumentError {
    DocumentError::type_mismatch(message)
}

/// Constructs a `TypeMismatch` error describing the actual variant.
fn type_mismatch_for(expected: &str, found: &Document) -> DocumentError {
    let found_name = match found {
        Document::Null => "null",
        Document::Bool(_) => "boolean",
        Document::Number(_) => "number",
        Document::BigInteger(_) => "bigInteger",
        Document::BigDecimal(_) => "bigDecimal",
        Document::String(_) => "string",
        Document::Blob(_) => "blob",
        Document::Timestamp(_) => "timestamp",
        Document::Array(_) => "array",
        Document::Object(_) => "object",
    };
    type_mismatch(format!("expected {expected}, found {found_name}"))
}

/// Constructs a numeric overflow error with target/value populated for
/// diagnostics.
fn overflow(target: &str, value: std::fmt::Arguments<'_>) -> DocumentError {
    DocumentError::numeric_coercion_overflow(target, value.to_string())
}

/// Constructs an `InvalidInput` error for parse failures (e.g. a malformed `BigDecimal` string
/// when going through `f64::from_str`).
fn invalid_input(target: &str, value: &str, err: &dyn std::fmt::Display) -> DocumentError {
    DocumentError::invalid_input(format!("cannot parse {value:?} as {target}: {err}"))
}

/* ANCHOR END: document */

#[cfg(test)]
mod extended_variant_tests {
    //! Tests for the extended Smithy data-model variants: the
    //! [`Document::Blob`] / [`Document::Timestamp`] /
    //! [`Document::BigInteger`] / [`Document::BigDecimal`] variants,
    //! their type-checking accessors and predicates, the numeric
    //! coercion accessors ([`Document::as_byte`] through
    //! [`Document::as_double`]), and the [`Document::coerce_big_integer`]
    //! / [`Document::coerce_big_decimal`] paths.
    //!
    //! These do not depend on the legacy serde feature gates — they
    //! exercise functionality available in any build of the crate.

    use super::{Document, DocumentError};
    use crate::{BigDecimal, BigInteger, DateTime, Number};
    use std::str::FromStr;

    // -- New variant constructors and type-check accessors --------------

    #[test]
    fn blob_variant_round_trips_via_accessor() {
        let d = Document::Blob(b"abcd".to_vec());
        assert!(d.is_blob());
        assert_eq!(d.as_blob(), Some(b"abcd".as_slice()));
    }

    #[test]
    fn timestamp_variant_round_trips_via_accessor() {
        let ts = DateTime::from_secs(0);
        let d = Document::Timestamp(ts);
        assert!(d.is_timestamp());
        assert_eq!(d.as_timestamp(), Some(ts));
    }

    #[test]
    fn big_integer_variant_round_trips_via_accessor() {
        let bi = BigInteger::from_str("12345678901234567890").unwrap();
        let d = Document::BigInteger(bi.clone());
        assert!(d.is_big_integer());
        assert_eq!(d.as_big_integer(), Some(&bi));
    }

    #[test]
    fn big_decimal_variant_round_trips_via_accessor() {
        let bd = BigDecimal::from_str("12345.678").unwrap();
        let d = Document::BigDecimal(bd.clone());
        assert!(d.is_big_decimal());
        assert_eq!(d.as_big_decimal(), Some(&bd));
    }

    #[test]
    fn type_check_accessors_do_not_coerce_across_variants() {
        // A JSON-side base64 string is `Document::String`; it must NOT
        // surface as a blob through the type-checking accessor. Coercion
        // (when the schema says it's a blob) lives on
        // `DiscriminatedDocument`.
        let s = Document::String("YWJjZA==".to_owned());
        assert_eq!(s.as_blob(), None);
        assert!(!s.is_blob());

        // Same for `as_timestamp` / `as_big_integer` / `as_big_decimal`
        // — none of them coerce.
        assert_eq!(s.as_timestamp(), None);
        assert_eq!(s.as_big_integer(), None);
        assert_eq!(s.as_big_decimal(), None);

        // Conversely, the new variants don't surface as the legacy
        // accessors either.
        let blob = Document::Blob(b"hi".to_vec());
        assert_eq!(blob.as_string(), None);
        assert!(!blob.is_string());
    }

    #[test]
    fn mut_accessors_let_callers_mutate_in_place() {
        let mut d = Document::Blob(vec![1, 2, 3]);
        if let Some(b) = d.as_blob_mut() {
            b.push(4);
        }
        assert_eq!(d.as_blob(), Some([1, 2, 3, 4].as_slice()));

        let mut d = Document::BigInteger(BigInteger::from_str("0").unwrap());
        if let Some(bi) = d.as_big_integer_mut() {
            *bi = BigInteger::from_str("42").unwrap();
        }
        assert_eq!(d.as_big_integer().unwrap().as_ref(), "42");
    }

    // -- Numeric coercion: happy path -----------------------------------

    #[test]
    fn as_byte_coerces_across_signed_integer_sources() {
        assert_eq!(Document::Number(Number::PosInt(42)).as_byte().unwrap(), 42);
        assert_eq!(
            Document::Number(Number::NegInt(-42)).as_byte().unwrap(),
            -42
        );
        let bi = BigInteger::from_str("100").unwrap();
        assert_eq!(Document::BigInteger(bi).as_byte().unwrap(), 100);
        let bd = BigDecimal::from_str("100.5").unwrap();
        assert_eq!(Document::BigDecimal(bd).as_byte().unwrap(), 100);
    }

    #[test]
    fn as_short_widens_from_smaller_sources() {
        assert_eq!(
            Document::Number(Number::PosInt(1234)).as_short().unwrap(),
            1234
        );
    }

    #[test]
    fn as_integer_widens_from_smaller_sources_and_narrows_when_in_range() {
        assert_eq!(
            Document::Number(Number::PosInt(i32::MAX as u64))
                .as_integer()
                .unwrap(),
            i32::MAX
        );
        assert_eq!(
            Document::Number(Number::NegInt(i32::MIN as i64))
                .as_integer()
                .unwrap(),
            i32::MIN
        );
    }

    #[test]
    fn as_long_handles_max_pos_int() {
        assert_eq!(
            Document::Number(Number::PosInt(i64::MAX as u64))
                .as_long()
                .unwrap(),
            i64::MAX
        );
    }

    #[test]
    fn as_double_widens_from_integer_sources() {
        // Integer-source values must be losslessly convertible to
        // f64 / f32 (going integer → float is allowed; the reverse is
        // not — see the no-crossover tests below).
        assert_eq!(
            Document::Number(Number::PosInt(42)).as_double().unwrap(),
            42.0
        );
        assert_eq!(
            Document::Number(Number::NegInt(-42)).as_double().unwrap(),
            -42.0
        );
        assert_eq!(
            Document::Number(Number::Float(2.5)).as_double().unwrap(),
            2.5
        );
    }

    #[test]
    fn as_float_inherits_from_as_double() {
        assert_eq!(
            Document::Number(Number::Float(1.5)).as_float().unwrap(),
            1.5_f32
        );
    }

    // -- Numeric coercion: overflow and type-mismatch -------------------

    #[test]
    fn as_byte_overflow_emits_typed_variant() {
        // The overflow() helper populates target/value rather than
        // returning a Custom variant.
        let err = Document::Number(Number::PosInt(200)).as_byte().unwrap_err();
        match err {
            DocumentError::NumericCoercionOverflow { target, value } => {
                assert_eq!(target, "byte");
                assert_eq!(value, "200");
            }
            other => panic!("expected NumericCoercionOverflow, got {other:?}"),
        }
    }

    #[test]
    fn as_byte_type_mismatch_for_non_numeric() {
        let err = Document::String("not a number".to_owned())
            .as_byte()
            .unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    #[test]
    fn as_integer_overflow_when_long_doesnt_fit() {
        // One below i32::MIN — the smallest negative not representable
        // as i32. Note `-(i32::MAX + 1) == i32::MIN` exactly, so we
        // need to go one further to overflow.
        let too_negative = i64::from(i32::MIN) - 1;
        let err = Document::Number(Number::NegInt(too_negative))
            .as_integer()
            .unwrap_err();
        assert!(matches!(err, DocumentError::NumericCoercionOverflow { .. }));
    }

    #[test]
    fn as_long_overflows_on_max_pos_int_above_i64_max() {
        let err = Document::Number(Number::PosInt(u64::MAX))
            .as_long()
            .unwrap_err();
        assert!(matches!(err, DocumentError::NumericCoercionOverflow { .. }));
    }

    // -- No integer/float crossover -------------------------------------

    #[test]
    fn as_byte_rejects_float_source_even_with_zero_fractional() {
        // A Float source value must NOT silently truncate into an
        // integer accessor, even when the fractional part is zero.
        // The caller can call `as_double()` first and cast
        // explicitly if they want that behavior.
        let err = Document::Number(Number::Float(42.0)).as_byte().unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));

        let err = Document::Number(Number::Float(42.7)).as_byte().unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    #[test]
    fn as_short_as_integer_as_long_all_reject_float_source() {
        // Same rule across the entire bounded-integer family.
        let f = Document::Number(Number::Float(1.0));
        assert!(matches!(
            f.as_short().unwrap_err(),
            DocumentError::TypeMismatch { .. }
        ));
        assert!(matches!(
            f.as_integer().unwrap_err(),
            DocumentError::TypeMismatch { .. }
        ));
        assert!(matches!(
            f.as_long().unwrap_err(),
            DocumentError::TypeMismatch { .. }
        ));
    }

    #[test]
    fn coerce_big_integer_rejects_float_source() {
        // Same rule on the arbitrary-precision side.
        let err = Document::Number(Number::Float(42.0))
            .coerce_big_integer()
            .unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    #[test]
    fn as_float_accepts_integer_source_losslessly() {
        // The reverse direction (integer → float) IS allowed per wiki
        // §8.2 — a Smithy `integer` member can populate a Smithy `float`
        // member because the integer value is exactly representable.
        assert_eq!(
            Document::Number(Number::PosInt(42)).as_float().unwrap(),
            42.0_f32
        );
        assert_eq!(
            Document::Number(Number::NegInt(-42)).as_double().unwrap(),
            -42.0_f64
        );
    }

    // -- bigInteger / bigDecimal not silently routed through f64 --------

    #[test]
    fn coerce_big_integer_truncates_big_decimal_at_decimal_point() {
        // A BigDecimal source must be truncated *as a string* at the
        // decimal point, NOT routed through f64 (which would lose
        // precision for values past 2^53).
        let bd = BigDecimal::from_str("12345678901234567890.123").unwrap();
        let bi = Document::BigDecimal(bd).coerce_big_integer().unwrap();
        // The 20-digit integer survives intact — clearly larger than
        // f64's lossless integer range (≈ 16 digits).
        assert_eq!(bi.as_ref(), "12345678901234567890");
    }

    #[test]
    fn coerce_big_integer_expands_scientific_notation_big_decimal() {
        // Regression: a BigDecimal in scientific notation must coerce to
        // its true integer magnitude, not be truncated at the 'e'. The
        // previous implementation split at the first 'e' and returned "1"
        // for "1.23e10" — off by ten orders of magnitude.
        let bd = BigDecimal::from_str("1.23e10").unwrap();
        let bi = Document::BigDecimal(bd).coerce_big_integer().unwrap();
        assert_eq!(bi.as_ref(), "12300000000");

        // A value beyond f64's lossless range, as the JSON wire path
        // produces it (oversize decimal lifted to BigDecimal).
        let bd = BigDecimal::from_str("1e30").unwrap();
        let bi = Document::BigDecimal(bd).coerce_big_integer().unwrap();
        assert_eq!(bi.as_ref(), "1000000000000000000000000000000");
    }

    #[test]
    fn coerce_big_integer_errors_on_unmaterializable_big_decimal() {
        // A pathological exponent must error, not allocate gigabytes or
        // return a wrong value.
        let bd = BigDecimal::from_str("1e1000000000").unwrap();
        let err = Document::BigDecimal(bd).coerce_big_integer().unwrap_err();
        assert!(
            err.to_string().contains("too large"),
            "expected a 'too large' error, got: {err}"
        );
    }

    #[test]
    fn coerce_big_decimal_widens_from_big_integer_lossless() {
        // BigInteger string is always a valid BigDecimal; large values
        // must survive intact.
        let big_str = "12345678901234567890123456789";
        let bi = BigInteger::from_str(big_str).unwrap();
        let bd = Document::BigInteger(bi).coerce_big_decimal().unwrap();
        assert_eq!(bd.as_ref(), big_str);
    }

    #[test]
    fn as_double_on_big_integer_is_documented_lossy_path() {
        // The `as_double` accessor is explicitly the lossy path. Wiki
        // §8.1 says bigInteger / bigDecimal must NOT be silently routed
        // through f64 by default — and our coercion accessors honor
        // that (via `coerce_big_integer` / `coerce_big_decimal` keeping
        // string precision). `as_double` is opt-in: callers who ask for
        // an f64 accept the precision loss.
        //
        // The precise loss is implementation-defined; we just check
        // that the call succeeds and that big numbers round-trip
        // through `coerce_big_integer` losslessly (above).
        let bi = BigInteger::from_str("12345678901234567890").unwrap();
        let f = Document::BigInteger(bi).as_double().unwrap();
        assert!(f.is_finite());
    }

    // -- Special floats round-trip via Number ---------------------------

    #[test]
    fn special_floats_round_trip_via_as_double() {
        // CBOR uses native float encoding (major type 7) for these
        // values; the JSON serializer renders them as the strings
        // `"NaN"` / `"Infinity"` / `"-Infinity"`. The codec layer
        // owns wire-format rendering. This test simply checks that
        // `Number::Float` carries non-finite values intact through
        // the `Document` API so the codec layer has something to
        // render.
        assert!(Document::Number(Number::Float(f64::NAN))
            .as_double()
            .unwrap()
            .is_nan());
        assert_eq!(
            Document::Number(Number::Float(f64::INFINITY))
                .as_double()
                .unwrap(),
            f64::INFINITY
        );
        assert_eq!(
            Document::Number(Number::Float(f64::NEG_INFINITY))
                .as_double()
                .unwrap(),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn special_floats_in_coerce_big_integer_are_rejected() {
        // Going from a non-finite f64 to bigInteger has no meaningful
        // result; we surface it as a typed error rather than producing
        // a malformed BigInteger string.
        //
        // (Reached via the float-source branch; same arm rejects all
        // floats — see `coerce_big_integer_rejects_float_source` above
        // for the finite case.)
        let err = Document::Number(Number::Float(f64::NAN))
            .coerce_big_integer()
            .unwrap_err();
        assert!(matches!(err, DocumentError::TypeMismatch { .. }));
    }

    #[test]
    fn special_floats_in_coerce_big_decimal_are_rejected() {
        // Distinct from coerce_big_integer: Float→BigDecimal is
        // accepted for finite values (it's a string format conversion),
        // but non-finite values are explicitly rejected.
        let err = Document::Number(Number::Float(f64::INFINITY))
            .coerce_big_decimal()
            .unwrap_err();
        match err {
            DocumentError::Custom { message } => {
                assert!(message.contains("non-finite"));
            }
            other => panic!("expected Custom non-finite error, got {other:?}"),
        }
    }

    // -- Non-exhaustive exhaustive-match guard --------------------------
    //
    // The consumer-facing `#[non_exhaustive]` contract (a downstream exhaustive
    // `match` on `Document` must include a wildcard arm) cannot be exercised
    // from inside this crate, where the attribute has no effect. It is tested
    // from a separate downstream crate by the `trybuild` compile-fail case in
    // `tests/document_non_exhaustive.rs`.
}

#[cfg(test)]
#[cfg(all(
    aws_sdk_unstable,
    feature = "serde-serialize",
    feature = "serde-deserialize"
))]
mod test {
    use super::{from_document, to_document, Document};
    use crate::document::DocumentObject;
    use crate::Number;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    /// Helper: serialize a value to Document and verify the result.
    fn test_to_document_ok<T>(cases: &[(T, Document)])
    where
        T: Serialize + std::fmt::Debug,
    {
        for (value, expected) in cases {
            let doc = to_document(value).unwrap();
            assert_eq!(&doc, expected, "to_document({:?})", value);
        }
    }

    /// Helper: round-trip T → Document → T.
    fn test_roundtrip<T>(cases: &[T])
    where
        T: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug + Clone,
    {
        for value in cases {
            let doc = to_document(value).unwrap();
            let roundtripped: T = from_document(doc).unwrap();
            assert_eq!(&roundtripped, value, "roundtrip failed for {:?}", value);
        }
    }

    // ========================================================================
    // Null / Unit
    // ========================================================================

    #[test]
    fn test_null() {
        test_to_document_ok(&[((), Document::Null)]);

        let v: () = from_document(Document::Null).unwrap();
        assert_eq!(v, ());
    }

    // ========================================================================
    // Booleans
    // ========================================================================

    #[test]
    fn test_bool() {
        test_to_document_ok(&[(true, Document::Bool(true)), (false, Document::Bool(false))]);
        test_roundtrip(&[true, false]);
    }

    // ========================================================================
    // Unsigned integers
    // ========================================================================

    #[test]
    fn test_u8() {
        test_to_document_ok(&[
            (0u8, Document::Number(Number::PosInt(0))),
            (u8::MAX, Document::Number(Number::PosInt(u8::MAX as u64))),
        ]);
        test_roundtrip(&[0u8, 1, 127, u8::MAX]);
    }

    #[test]
    fn test_u16() {
        test_to_document_ok(&[
            (0u16, Document::Number(Number::PosInt(0))),
            (u16::MAX, Document::Number(Number::PosInt(u16::MAX as u64))),
        ]);
        test_roundtrip(&[0u16, 1, u16::MAX]);
    }

    #[test]
    fn test_u32() {
        test_to_document_ok(&[
            (0u32, Document::Number(Number::PosInt(0))),
            (u32::MAX, Document::Number(Number::PosInt(u32::MAX as u64))),
        ]);
        test_roundtrip(&[0u32, 1, u32::MAX]);
    }

    #[test]
    fn test_u64() {
        test_to_document_ok(&[
            (0u64, Document::Number(Number::PosInt(0))),
            (u64::MAX, Document::Number(Number::PosInt(u64::MAX))),
        ]);
        test_roundtrip(&[0u64, 1, u64::MAX]);
    }

    // ========================================================================
    // Signed integers
    // ========================================================================

    #[test]
    fn test_i8() {
        test_to_document_ok(&[
            (0i8, Document::Number(Number::PosInt(0))),
            (-1i8, Document::Number(Number::NegInt(-1))),
            (i8::MIN, Document::Number(Number::NegInt(i8::MIN as i64))),
            (i8::MAX, Document::Number(Number::PosInt(i8::MAX as u64))),
        ]);
        test_roundtrip(&[0i8, -1, 1, i8::MIN, i8::MAX]);
    }

    #[test]
    fn test_i16() {
        test_to_document_ok(&[
            (0i16, Document::Number(Number::PosInt(0))),
            (i16::MIN, Document::Number(Number::NegInt(i16::MIN as i64))),
            (i16::MAX, Document::Number(Number::PosInt(i16::MAX as u64))),
        ]);
        test_roundtrip(&[0i16, -1, i16::MIN, i16::MAX]);
    }

    #[test]
    fn test_i32() {
        test_to_document_ok(&[
            (0i32, Document::Number(Number::PosInt(0))),
            (i32::MIN, Document::Number(Number::NegInt(i32::MIN as i64))),
            (i32::MAX, Document::Number(Number::PosInt(i32::MAX as u64))),
        ]);
        test_roundtrip(&[0i32, -1, i32::MIN, i32::MAX]);
    }

    #[test]
    fn test_i64() {
        test_to_document_ok(&[
            (0i64, Document::Number(Number::PosInt(0))),
            (-1i64, Document::Number(Number::NegInt(-1))),
            (i64::MIN, Document::Number(Number::NegInt(i64::MIN))),
            (i64::MAX, Document::Number(Number::PosInt(i64::MAX as u64))),
        ]);
        test_roundtrip(&[0i64, -1, i64::MIN, i64::MAX]);
    }

    // ========================================================================
    // Floats
    // ========================================================================

    #[test]
    fn test_f32() {
        test_to_document_ok(&[
            (0.0f32, Document::Number(Number::Float(0.0))),
            (3.5f32, Document::Number(Number::Float(3.5))),
            (-1.5f32, Document::Number(Number::Float(-1.5))),
        ]);
        test_roundtrip(&[0.0f32, 3.5, -1.5, f32::MIN, f32::MAX]);
    }

    #[test]
    fn test_f64() {
        test_to_document_ok(&[
            (0.0f64, Document::Number(Number::Float(0.0))),
            (3.1f64, Document::Number(Number::Float(3.1))),
            (-1.5f64, Document::Number(Number::Float(-1.5))),
            (f64::MIN, Document::Number(Number::Float(f64::MIN))),
            (f64::MAX, Document::Number(Number::Float(f64::MAX))),
            (f64::EPSILON, Document::Number(Number::Float(f64::EPSILON))),
        ]);
        test_roundtrip(&[0.0f64, 3.1, -1.5, 0.5, f64::MIN, f64::MAX]);
    }

    #[test]
    fn test_nonfinite_floats() {
        // NaN, +Inf, -Inf all serialize to the float Document representation
        let doc = to_document(&f64::NAN).unwrap();
        match doc {
            Document::Number(Number::Float(v)) => assert!(v.is_nan()),
            other => panic!("expected NaN float, got {:?}", other),
        }

        let doc = to_document(&f64::INFINITY).unwrap();
        assert_eq!(doc, Document::Number(Number::Float(f64::INFINITY)));

        let doc = to_document(&f64::NEG_INFINITY).unwrap();
        assert_eq!(doc, Document::Number(Number::Float(f64::NEG_INFINITY)));
    }

    // ========================================================================
    // Strings
    // ========================================================================

    #[test]
    fn test_string() {
        test_to_document_ok(&[
            (String::new(), Document::String(String::new())),
            ("hello".to_owned(), Document::String("hello".to_owned())),
            (
                "with\nnewline".to_owned(),
                Document::String("with\nnewline".to_owned()),
            ),
            (
                "unicode: \u{1F600}".to_owned(),
                Document::String("unicode: \u{1F600}".to_owned()),
            ),
        ]);
        test_roundtrip(&[
            String::new(),
            "foo".to_owned(),
            "bar\tbaz".to_owned(),
            "\u{3A3}".to_owned(),
        ]);
    }

    #[test]
    fn test_str_ref() {
        let doc = to_document(&"borrowed str").unwrap();
        assert_eq!(doc, Document::String("borrowed str".to_owned()));
    }

    #[test]
    fn test_char() {
        let doc = to_document(&'a').unwrap();
        assert_eq!(doc, Document::String("a".to_owned()));

        let doc = to_document(&'\u{1F600}').unwrap();
        assert_eq!(doc, Document::String("\u{1F600}".to_owned()));
    }

    // ========================================================================
    // Option
    // ========================================================================

    #[test]
    fn test_option() {
        test_to_document_ok(&[
            (None::<String>, Document::Null),
            (
                Some("jodhpurs".to_owned()),
                Document::String("jodhpurs".to_owned()),
            ),
        ]);
        test_to_document_ok(&[
            (None::<u32>, Document::Null),
            (Some(42u32), Document::Number(Number::PosInt(42))),
        ]);
        test_roundtrip(&[None::<u32>, Some(5), Some(0)]);
        test_roundtrip(&[None::<String>, Some("x".to_owned())]);
    }

    // ========================================================================
    // Sequences / Arrays
    // ========================================================================

    #[test]
    fn test_vec_empty() {
        let doc = to_document(&Vec::<i32>::new()).unwrap();
        assert_eq!(doc, Document::Array(vec![]));

        let v: Vec<i32> = from_document(Document::Array(vec![])).unwrap();
        assert_eq!(v, Vec::<i32>::new());
    }

    #[test]
    fn test_vec_integers() {
        test_to_document_ok(&[(
            vec![1u64, 2, 3],
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::Number(Number::PosInt(2)),
                Document::Number(Number::PosInt(3)),
            ]),
        )]);
        test_roundtrip(&[vec![1i32, -2, 3], vec![], vec![0]]);
    }

    #[test]
    fn test_vec_mixed_via_document() {
        // Vec<Document> allows mixed types
        let mixed = vec![
            Document::Bool(true),
            Document::Null,
            Document::String("foo".to_owned()),
            Document::Number(Number::PosInt(42)),
        ];
        let doc = Document::Array(mixed.clone());
        let roundtripped: Vec<Document> = from_document(doc.clone()).unwrap();
        assert_eq!(roundtripped, mixed);
    }

    #[test]
    fn test_nested_vec() {
        test_roundtrip(&[
            vec![vec![1u32, 2], vec![], vec![3]],
            vec![vec![], vec![], vec![]],
        ]);
    }

    #[test]
    fn test_tuple() {
        let doc = to_document(&(5u32,)).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![Document::Number(Number::PosInt(5))])
        );

        let doc = to_document(&(1u32, "abc", true)).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::String("abc".to_owned()),
                Document::Bool(true),
            ])
        );

        test_roundtrip(&[(1u32, 2u32), (0, u32::MAX)]);
        test_roundtrip(&[(1i32, "hello".to_owned(), true)]);
    }

    // ========================================================================
    // Maps / Objects
    // ========================================================================

    #[test]
    fn test_map_empty() {
        let map: HashMap<String, u32> = HashMap::new();
        let doc = to_document(&map).unwrap();
        assert_eq!(doc, Document::Object(DocumentObject::new()));
        test_roundtrip(&[HashMap::<String, u32>::new()]);
    }

    #[test]
    fn test_map_string_keys() {
        let mut map = HashMap::new();
        map.insert("a".to_owned(), 1u32);
        map.insert("b".to_owned(), 2u32);
        test_roundtrip(&[map]);
    }

    #[test]
    fn test_map_integer_keys() {
        // Integer keys get serialized as their string representation
        let mut map = HashMap::new();
        map.insert(1u32, "one".to_owned());
        map.insert(2u32, "two".to_owned());

        let doc = to_document(&map).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("1") || obj.contains_key("2"));
    }

    #[test]
    fn test_nested_map() {
        let mut inner = HashMap::new();
        inner.insert("x".to_owned(), 10u32);

        let mut outer = HashMap::new();
        outer.insert("inner".to_owned(), inner.clone());

        test_roundtrip(&[outer]);
    }

    // ========================================================================
    // Structs
    // ========================================================================

    #[test]
    fn test_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Inner {
            a: (),
            b: usize,
            c: Vec<String>,
        }

        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Outer {
            inner: Vec<Inner>,
        }

        let outer = Outer {
            inner: vec![Inner {
                a: (),
                b: 2,
                c: vec!["abc".to_owned(), "xyz".to_owned()],
            }],
        };

        let doc = to_document(&outer).unwrap();
        assert!(doc.is_object());
        let roundtripped: Outer = from_document(doc).unwrap();
        assert_eq!(outer, roundtripped);

        // Empty inner
        test_roundtrip(&[Outer { inner: vec![] }]);
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Wrapper(u32);

        test_to_document_ok(&[(Wrapper(123), Document::Number(Number::PosInt(123)))]);
        test_roundtrip(&[Wrapper(0), Wrapper(u32::MAX)]);
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
        struct Unit;

        test_to_document_ok(&[(Unit, Document::Null)]);
        let v: Unit = from_document(Document::Null).unwrap();
        assert_eq!(v, Unit);
    }

    // ========================================================================
    // Enums
    // ========================================================================

    #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
    enum Animal {
        Dog,
        Frog(String, Vec<isize>),
        Cat { age: usize, name: String },
        AntHive(Vec<String>),
    }

    #[test]
    fn test_enum_unit_variant() {
        let doc = to_document(&Animal::Dog).unwrap();
        assert_eq!(doc, Document::String("Dog".to_owned()));
        test_roundtrip(&[Animal::Dog]);
    }

    #[test]
    fn test_enum_tuple_variant() {
        let frog = Animal::Frog("Henry".to_owned(), vec![349, 102]);
        let doc = to_document(&frog).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("Frog"));
        test_roundtrip(&[
            Animal::Frog("Henry".to_owned(), vec![]),
            Animal::Frog("Henry".to_owned(), vec![349, 102]),
        ]);
    }

    #[test]
    fn test_enum_struct_variant() {
        let cat = Animal::Cat {
            age: 5,
            name: "Kate".to_owned(),
        };
        let doc = to_document(&cat).unwrap();
        assert!(doc.is_object());
        let obj = doc.as_object().unwrap();
        assert!(obj.contains_key("Cat"));
        test_roundtrip(&[cat]);
    }

    #[test]
    fn test_enum_newtype_variant() {
        let hive = Animal::AntHive(vec!["Bob".to_owned(), "Stuart".to_owned()]);
        test_roundtrip(&[hive]);
    }

    // ========================================================================
    // Bytes
    // ========================================================================

    #[test]
    fn test_bytes() {
        // Bytes serialize as an array of numbers
        let data: &[u8] = &[1, 2, 3];
        let doc = to_document(&data).unwrap();
        assert_eq!(
            doc,
            Document::Array(vec![
                Document::Number(Number::PosInt(1)),
                Document::Number(Number::PosInt(2)),
                Document::Number(Number::PosInt(3)),
            ])
        );

        let empty: &[u8] = &[];
        let doc = to_document(&empty).unwrap();
        assert_eq!(doc, Document::Array(vec![]));
    }

    // ========================================================================
    // Error cases
    // ========================================================================

    #[test]
    fn test_deserialize_wrong_type() {
        let result = from_document::<bool>(Document::String("not a bool".to_owned()));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("invalid type"),
            "unexpected error message: {}",
            err
        );
    }

    #[test]
    fn test_deserialize_missing_field() {
        #[derive(Debug, Deserialize)]
        struct Required {
            #[allow(dead_code)]
            x: u32,
        }

        let doc = Document::Object(DocumentObject::new());
        let result = from_document::<Required>(doc);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_serialize_non_string_map_key_rejected() {
        // Map keys that cannot be coerced to strings should fail
        use std::collections::HashMap;
        let mut map: HashMap<Option<u32>, u32> = HashMap::new();
        map.insert(None, 1);

        let result = to_document(&map);
        assert!(result.is_err());
    }

    // ========================================================================
    // serde_json compatibility (existing test)
    // ========================================================================

    #[test]
    fn test_serde_json_compatibility() {
        let mut map: HashMap<String, Document> = HashMap::new();
        map.insert("hello".into(), "world".to_string().into());
        map.insert("pos_int".into(), Document::Number(Number::PosInt(1).into()));
        map.insert(
            "neg_int".into(),
            Document::Number(Number::NegInt(-1).into()),
        );
        map.insert(
            "float".into(),
            Document::Number(Number::Float(0.1 + 0.2).into()),
        );
        map.insert("true".into(), true.into());
        map.insert("false".into(), false.into());
        map.insert(
            "array".into(),
            vec![
                map.clone().into(),
                "hello-world".to_string().into(),
                true.into(),
                false.into(),
            ]
            .into(),
        );
        map.insert("map".into(), map.clone().into());
        map.insert("null".into(), Document::Null);
        let obj = Document::Object(map.into());

        let target_file = include_str!("../../test_data/serialize_document.json");
        let json: Result<serde_json::Value, _> = serde_json::from_str(target_file);
        assert_eq!(serde_json::to_value(&obj).unwrap(), json.unwrap());
        let doc: Result<Document, _> = serde_json::from_str(target_file);
        assert_eq!(obj, doc.unwrap());
    }
}
