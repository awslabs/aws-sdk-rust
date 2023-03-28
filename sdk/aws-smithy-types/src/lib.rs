/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Protocol-agnostic types for smithy-rs.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

use crate::error::{TryFromNumberError, TryFromNumberErrorKind};
use std::collections::HashMap;

pub mod base64;
pub mod date_time;
pub mod endpoint;
pub mod error;
pub mod primitive;
pub mod retry;
pub mod timeout;

pub use crate::date_time::DateTime;

// TODO(deprecated): Remove deprecated re-export
/// Use [error::ErrorMetadata] instead.
#[deprecated(
    note = "`aws_smithy_types::Error` has been renamed to `aws_smithy_types::error::ErrorMetadata`"
)]
pub use error::ErrorMetadata as Error;

/// Binary Blob Type
///
/// Blobs represent protocol-agnostic binary content.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Blob {
    inner: Vec<u8>,
}

impl Blob {
    /// Creates a new blob from the given `input`.
    pub fn new<T: Into<Vec<u8>>>(input: T) -> Self {
        Blob {
            inner: input.into(),
        }
    }

    /// Consumes the `Blob` and returns a `Vec<u8>` with its contents.
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

/* ANCHOR: document */

/// Document Type
///
/// Document types represents protocol-agnostic open content that is accessed like JSON data.
/// Open content is useful for modeling unstructured data that has no schema, data that can't be
/// modeled using rigid types, or data that has a schema that evolves outside of the purview of a model.
/// The serialization format of a document is an implementation detail of a protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    /// JSON object
    Object(HashMap<String, Document>),
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
}

impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document::Bool(value)
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

impl From<HashMap<String, Document>> for Document {
    fn from(values: HashMap<String, Document>) -> Self {
        Document::Object(values)
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

/// A number type that implements Javascript / JSON semantics, modeled on serde_json:
/// <https://docs.serde.rs/src/serde_json/number.rs.html#20-22>
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    /// Unsigned 64-bit integer value.
    PosInt(u64),
    /// Signed 64-bit integer value. The wrapped value is _always_ negative.
    NegInt(i64),
    /// 64-bit floating-point value.
    Float(f64),
}

/* ANCHOR_END: document */

impl Number {
    /// Converts to an `f64` lossily.
    /// Use `Number::try_from` to make the conversion only if it is not lossy.
    pub fn to_f64_lossy(self) -> f64 {
        match self {
            Number::PosInt(v) => v as f64,
            Number::NegInt(v) => v as f64,
            Number::Float(v) => v as f64,
        }
    }

    /// Converts to an `f32` lossily.
    /// Use `Number::try_from` to make the conversion only if it is not lossy.
    pub fn to_f32_lossy(self) -> f32 {
        match self {
            Number::PosInt(v) => v as f32,
            Number::NegInt(v) => v as f32,
            Number::Float(v) => v as f32,
        }
    }
}

macro_rules! to_unsigned_integer_converter {
    ($typ:ident, $styp:expr) => {
        #[doc = "Converts to a `"]
        #[doc = $styp]
        #[doc = "`. This conversion fails if it is lossy."]
        impl TryFrom<Number> for $typ {
            type Error = TryFromNumberError;

            fn try_from(value: Number) -> Result<Self, Self::Error> {
                match value {
                    Number::PosInt(v) => Ok(Self::try_from(v)?),
                    Number::NegInt(v) => {
                        Err(TryFromNumberErrorKind::NegativeToUnsignedLossyConversion(v).into())
                    }
                    Number::Float(v) => {
                        Err(TryFromNumberErrorKind::FloatToIntegerLossyConversion(v).into())
                    }
                }
            }
        }
    };

    ($typ:ident) => {
        to_unsigned_integer_converter!($typ, stringify!($typ));
    };
}

macro_rules! to_signed_integer_converter {
    ($typ:ident, $styp:expr) => {
        #[doc = "Converts to a `"]
        #[doc = $styp]
        #[doc = "`. This conversion fails if it is lossy."]
        impl TryFrom<Number> for $typ {
            type Error = TryFromNumberError;

            fn try_from(value: Number) -> Result<Self, Self::Error> {
                match value {
                    Number::PosInt(v) => Ok(Self::try_from(v)?),
                    Number::NegInt(v) => Ok(Self::try_from(v)?),
                    Number::Float(v) => {
                        Err(TryFromNumberErrorKind::FloatToIntegerLossyConversion(v).into())
                    }
                }
            }
        }
    };

    ($typ:ident) => {
        to_signed_integer_converter!($typ, stringify!($typ));
    };
}

/// Converts to a `u64`. The conversion fails if it is lossy.
impl TryFrom<Number> for u64 {
    type Error = TryFromNumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        match value {
            Number::PosInt(v) => Ok(v),
            Number::NegInt(v) => {
                Err(TryFromNumberErrorKind::NegativeToUnsignedLossyConversion(v).into())
            }
            Number::Float(v) => {
                Err(TryFromNumberErrorKind::FloatToIntegerLossyConversion(v).into())
            }
        }
    }
}
to_unsigned_integer_converter!(u32);
to_unsigned_integer_converter!(u16);
to_unsigned_integer_converter!(u8);

impl TryFrom<Number> for i64 {
    type Error = TryFromNumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        match value {
            Number::PosInt(v) => Ok(Self::try_from(v)?),
            Number::NegInt(v) => Ok(v),
            Number::Float(v) => {
                Err(TryFromNumberErrorKind::FloatToIntegerLossyConversion(v).into())
            }
        }
    }
}
to_signed_integer_converter!(i32);
to_signed_integer_converter!(i16);
to_signed_integer_converter!(i8);

/// Converts to an `f64`. The conversion fails if it is lossy.
impl TryFrom<Number> for f64 {
    type Error = TryFromNumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        match value {
            // Integers can only be represented with full precision in a float if they fit in the
            // significand, which is 24 bits in `f32` and 53 bits in `f64`.
            // https://github.com/rust-lang/rust/blob/58f11791af4f97572e7afd83f11cffe04bbbd12f/library/core/src/convert/num.rs#L151-L153
            Number::PosInt(v) => {
                if v <= (1 << 53) {
                    Ok(v as Self)
                } else {
                    Err(TryFromNumberErrorKind::U64ToFloatLossyConversion(v).into())
                }
            }
            Number::NegInt(v) => {
                if (-(1 << 53)..=(1 << 53)).contains(&v) {
                    Ok(v as Self)
                } else {
                    Err(TryFromNumberErrorKind::I64ToFloatLossyConversion(v).into())
                }
            }
            Number::Float(v) => Ok(v),
        }
    }
}

/// Converts to an `f64`. The conversion fails if it is lossy.
impl TryFrom<Number> for f32 {
    type Error = TryFromNumberError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        match value {
            Number::PosInt(v) => {
                if v <= (1 << 24) {
                    Ok(v as Self)
                } else {
                    Err(TryFromNumberErrorKind::U64ToFloatLossyConversion(v).into())
                }
            }
            Number::NegInt(v) => {
                if (-(1 << 24)..=(1 << 24)).contains(&v) {
                    Ok(v as Self)
                } else {
                    Err(TryFromNumberErrorKind::I64ToFloatLossyConversion(v).into())
                }
            }
            Number::Float(v) => Err(TryFromNumberErrorKind::F64ToF32LossyConversion(v).into()),
        }
    }
}

#[cfg(test)]
mod number {
    use super::*;
    use crate::error::{TryFromNumberError, TryFromNumberErrorKind};

    macro_rules! to_unsigned_converter_tests {
        ($typ:ident) => {
            assert_eq!($typ::try_from(Number::PosInt(69u64)).unwrap(), 69);

            assert!(matches!(
                $typ::try_from(Number::PosInt(($typ::MAX as u64) + 1u64)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::OutsideIntegerRange(..)
                }
            ));

            assert!(matches!(
                $typ::try_from(Number::NegInt(-1i64)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::NegativeToUnsignedLossyConversion(..)
                }
            ));

            for val in [69.69f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(matches!(
                    $typ::try_from(Number::Float(val)).unwrap_err(),
                    TryFromNumberError {
                        kind: TryFromNumberErrorKind::FloatToIntegerLossyConversion(..)
                    }
                ));
            }
        };
    }

    #[test]
    fn to_u64() {
        assert_eq!(u64::try_from(Number::PosInt(69u64)).unwrap(), 69u64);

        assert!(matches!(
            u64::try_from(Number::NegInt(-1i64)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::NegativeToUnsignedLossyConversion(..)
            }
        ));

        for val in [69.69f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                u64::try_from(Number::Float(val)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::FloatToIntegerLossyConversion(..)
                }
            ));
        }
    }

    #[test]
    fn to_u32() {
        to_unsigned_converter_tests!(u32);
    }

    #[test]
    fn to_u16() {
        to_unsigned_converter_tests!(u16);
    }

    #[test]
    fn to_u8() {
        to_unsigned_converter_tests!(u8);
    }

    macro_rules! to_signed_converter_tests {
        ($typ:ident) => {
            assert_eq!($typ::try_from(Number::PosInt(69u64)).unwrap(), 69);
            assert_eq!($typ::try_from(Number::NegInt(-69i64)).unwrap(), -69);

            assert!(matches!(
                $typ::try_from(Number::PosInt(($typ::MAX as u64) + 1u64)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::OutsideIntegerRange(..)
                }
            ));

            assert!(matches!(
                $typ::try_from(Number::NegInt(($typ::MIN as i64) - 1i64)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::OutsideIntegerRange(..)
                }
            ));

            for val in [69.69f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                assert!(matches!(
                    u64::try_from(Number::Float(val)).unwrap_err(),
                    TryFromNumberError {
                        kind: TryFromNumberErrorKind::FloatToIntegerLossyConversion(..)
                    }
                ));
            }
        };
    }

    #[test]
    fn to_i64() {
        assert_eq!(i64::try_from(Number::PosInt(69u64)).unwrap(), 69);
        assert_eq!(i64::try_from(Number::NegInt(-69i64)).unwrap(), -69);

        for val in [69.69f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                u64::try_from(Number::Float(val)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::FloatToIntegerLossyConversion(..)
                }
            ));
        }
    }

    #[test]
    fn to_i32() {
        to_signed_converter_tests!(i32);
    }

    #[test]
    fn to_i16() {
        to_signed_converter_tests!(i16);
    }

    #[test]
    fn to_i8() {
        to_signed_converter_tests!(i8);
    }

    #[test]
    fn to_f64() {
        assert_eq!(f64::try_from(Number::PosInt(69u64)).unwrap(), 69f64);
        assert_eq!(f64::try_from(Number::NegInt(-69i64)).unwrap(), -69f64);
        assert_eq!(f64::try_from(Number::Float(-69f64)).unwrap(), -69f64);
        assert!(f64::try_from(Number::Float(f64::NAN)).unwrap().is_nan());
        assert_eq!(
            f64::try_from(Number::Float(f64::INFINITY)).unwrap(),
            f64::INFINITY
        );
        assert_eq!(
            f64::try_from(Number::Float(f64::NEG_INFINITY)).unwrap(),
            f64::NEG_INFINITY
        );

        let significand_max_u64: u64 = 1 << 53;
        let significand_max_i64: i64 = 1 << 53;

        assert_eq!(
            f64::try_from(Number::PosInt(significand_max_u64)).unwrap(),
            9007199254740992f64
        );

        assert_eq!(
            f64::try_from(Number::NegInt(significand_max_i64)).unwrap(),
            9007199254740992f64
        );
        assert_eq!(
            f64::try_from(Number::NegInt(-significand_max_i64)).unwrap(),
            -9007199254740992f64
        );

        assert!(matches!(
            f64::try_from(Number::PosInt(significand_max_u64 + 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::U64ToFloatLossyConversion(..)
            }
        ));

        assert!(matches!(
            f64::try_from(Number::NegInt(significand_max_i64 + 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::I64ToFloatLossyConversion(..)
            }
        ));
        assert!(matches!(
            f64::try_from(Number::NegInt(-significand_max_i64 - 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::I64ToFloatLossyConversion(..)
            }
        ));
    }

    #[test]
    fn to_f32() {
        assert_eq!(f32::try_from(Number::PosInt(69u64)).unwrap(), 69f32);
        assert_eq!(f32::try_from(Number::NegInt(-69i64)).unwrap(), -69f32);

        let significand_max_u64: u64 = 1 << 24;
        let significand_max_i64: i64 = 1 << 24;

        assert_eq!(
            f32::try_from(Number::PosInt(significand_max_u64)).unwrap(),
            16777216f32
        );

        assert_eq!(
            f32::try_from(Number::NegInt(significand_max_i64)).unwrap(),
            16777216f32
        );
        assert_eq!(
            f32::try_from(Number::NegInt(-significand_max_i64)).unwrap(),
            -16777216f32
        );

        assert!(matches!(
            f32::try_from(Number::PosInt(significand_max_u64 + 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::U64ToFloatLossyConversion(..)
            }
        ));

        assert!(matches!(
            f32::try_from(Number::NegInt(significand_max_i64 + 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::I64ToFloatLossyConversion(..)
            }
        ));
        assert!(matches!(
            f32::try_from(Number::NegInt(-significand_max_i64 - 1)).unwrap_err(),
            TryFromNumberError {
                kind: TryFromNumberErrorKind::I64ToFloatLossyConversion(..)
            }
        ));

        for val in [69f64, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                f32::try_from(Number::Float(val)).unwrap_err(),
                TryFromNumberError {
                    kind: TryFromNumberErrorKind::F64ToF32LossyConversion(..)
                }
            ));
        }
    }

    #[test]
    fn to_f64_lossy() {
        assert_eq!(Number::PosInt(69u64).to_f64_lossy(), 69f64);
        assert_eq!(
            Number::PosInt((1 << 53) + 1).to_f64_lossy(),
            9007199254740992f64
        );
        assert_eq!(
            Number::NegInt(-(1 << 53) - 1).to_f64_lossy(),
            -9007199254740992f64
        );
    }

    #[test]
    fn to_f32_lossy() {
        assert_eq!(Number::PosInt(69u64).to_f32_lossy(), 69f32);
        assert_eq!(Number::PosInt((1 << 24) + 1).to_f32_lossy(), 16777216f32);
        assert_eq!(Number::NegInt(-(1 << 24) - 1).to_f32_lossy(), -16777216f32);
        assert_eq!(
            Number::Float(1452089033.7674935).to_f32_lossy(),
            1452089100f32
        );
    }
}
