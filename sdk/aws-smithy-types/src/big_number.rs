/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Big number types represented as strings.
//!
//! These types are simple string wrappers that allow users to parse and format
//! big numbers using their preferred library.
//!
//! # Accepted input
//!
//! `FromStr` validates the *structure* of the input, not just its character
//! set. `BigInteger` uses the JSON integer grammar with one deliberate
//! relaxation (leading zeros), while `BigDecimal` accepts the forms supported
//! by the CBOR decimal-fraction implementation:
//!
//! ```text
//! BigInteger := '-'? DIGIT+
//! BigDecimal := ('+' | '-')? ( DIGIT+ ( '.' DIGIT* )? | '.' DIGIT+ )
//!               ( ('e' | 'E') ('+' | '-')? DIGIT+ )?
//! ```
//!
//! Both types accept `"-12"` and `"00123"`; `BigDecimal` also accepts `"+5"`,
//! `".5"`, `"5."`, and `"1.23E-10"`. Neither type accepts `"1.2.3"`, `"--5"`,
//! `"1e"`, `"e10"`, or `"-"`.
//! Decimal exponents and the resulting scale must also fit the supported range;
//! otherwise parsing returns [`BigNumberError::ExponentOutOfRange`].
//!
//! Leading zeros are accepted because they have exactly one numeric reading and
//! were accepted by previously released versions. They are *not* valid RFC 8259
//! JSON numbers, so a protocol that emits an arbitrary-precision value as a raw
//! JSON number rejects them at the wire boundary instead.

/// Error type for BigInteger and BigDecimal parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BigNumberError {
    /// The input string is not a valid number format.
    InvalidFormat(String),
    /// The number's exponent is outside the supported range.
    ExponentOutOfRange(String),
}

impl std::fmt::Display for BigNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BigNumberError::InvalidFormat(s) => write!(f, "invalid number format: {s}"),
            BigNumberError::ExponentOutOfRange(s) => {
                write!(f, "number exponent is outside the supported range: {s}")
            }
        }
    }
}

impl std::error::Error for BigNumberError {}

/// `true` iff `s` is non-empty and consists entirely of ASCII digits.
fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Validates that a string is a valid `BigInteger`: `'-'? DIGIT+`.
///
/// A leading `-` is optional; a leading `+` is not accepted. At least one digit
/// is mandatory, so a bare sign (`"-"`, `"+"`) and the empty string are
/// rejected, as are repeated signs (`"--5"`), decimal points and exponents.
///
/// Leading zeros (`"00123"`) are accepted — see the module docs.
fn is_valid_big_integer(s: &str) -> bool {
    is_ascii_digits(s.strip_prefix('-').unwrap_or(s))
}

/// Validates that a string is a supported `BigDecimal`.
///
/// The coefficient accepts an optional leading sign and requires at least one
/// digit across its integer and fractional parts. Consequently, `"+5"`,
/// `".5"`, `"-.5"`, and `"5."` are valid, while `"."`, `"--5"`, and
/// `"1.2.3"` are not. A scientific-notation exponent may also have a sign but
/// must contain digits.
///
/// The exponent and the scale derived from the fraction length and exponent
/// must fit the range supported by the CBOR decimal-fraction implementation.
fn validate_big_decimal(s: &str) -> Result<(), BigNumberError> {
    let invalid_format = || BigNumberError::InvalidFormat(s.to_string());
    let exponent_out_of_range = || BigNumberError::ExponentOutOfRange(s.to_string());

    let (coefficient, exponent) = match s.split_once(['e', 'E']) {
        Some((coefficient, exponent)) => match exponent.parse::<i128>() {
            Ok(exponent) => (coefficient, exponent),
            Err(error)
                if matches!(
                    error.kind(),
                    std::num::IntErrorKind::PosOverflow | std::num::IntErrorKind::NegOverflow
                ) =>
            {
                return Err(exponent_out_of_range());
            }
            Err(_) => return Err(invalid_format()),
        },
        None => (s, 0),
    };

    let coefficient = coefficient.strip_prefix(['-', '+']).unwrap_or(coefficient);
    let (integer, fraction) = coefficient.split_once('.').unwrap_or((coefficient, ""));

    if (integer.is_empty() && fraction.is_empty())
        || !integer
            .bytes()
            .chain(fraction.bytes())
            .all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid_format());
    }

    match (fraction.len() as i128).checked_sub(exponent) {
        Some(scale) if (-(i64::MAX as i128)..=i64::MAX as i128).contains(&scale) => Ok(()),
        _ => Err(exponent_out_of_range()),
    }
}

/// A BigInteger represented as a string.
///
/// This type does not perform arithmetic operations. Users should parse the string
/// with their preferred big integer library.
///
/// See the [module docs](self) for the grammar accepted by `FromStr`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
pub struct BigInteger(String);

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
impl<'de> serde::Deserialize<'de> for BigInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

impl Default for BigInteger {
    fn default() -> Self {
        Self("0".to_string())
    }
}

impl std::str::FromStr for BigInteger {
    type Err = BigNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_valid_big_integer(s) {
            return Err(BigNumberError::InvalidFormat(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for BigInteger {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A big decimal represented as a string.
///
/// This type does not perform arithmetic operations. Users should parse the string
/// with their preferred big decimal library.
///
/// See the [module docs](self) for the grammar accepted by `FromStr`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
pub struct BigDecimal(String);

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
impl<'de> serde::Deserialize<'de> for BigDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        value.parse().map_err(serde::de::Error::custom)
    }
}

impl Default for BigDecimal {
    fn default() -> Self {
        Self("0.0".to_string())
    }
}

impl std::str::FromStr for BigDecimal {
    type Err = BigNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        validate_big_decimal(s)?;
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for BigDecimal {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn big_integer_basic() {
        let bi = BigInteger::from_str("12345678901234567890").unwrap();
        assert_eq!(bi.as_ref(), "12345678901234567890");
    }

    #[test]
    fn big_integer_default() {
        let bi = BigInteger::default();
        assert_eq!(bi.as_ref(), "0");
        // The default must itself be parseable.
        assert_eq!(BigInteger::from_str(bi.as_ref()).unwrap(), bi);
    }

    #[test]
    fn big_decimal_basic() {
        let bd = BigDecimal::from_str("123.456789").unwrap();
        assert_eq!(bd.as_ref(), "123.456789");
    }

    #[test]
    fn big_decimal_default() {
        let bd = BigDecimal::default();
        assert_eq!(bd.as_ref(), "0.0");
        assert_eq!(BigDecimal::from_str(bd.as_ref()).unwrap(), bd);
    }

    #[test]
    fn big_integer_negative() {
        let bi = BigInteger::from_str("-12345").unwrap();
        assert_eq!(bi.as_ref(), "-12345");
    }

    #[test]
    fn big_decimal_scientific() {
        let bd = BigDecimal::from_str("1.23e10").unwrap();
        assert_eq!(bd.as_ref(), "1.23e10");

        let bd = BigDecimal::from_str("1.23E-10").unwrap();
        assert_eq!(bd.as_ref(), "1.23E-10");
    }

    #[test]
    fn big_decimal_accepts_supported_formats() {
        for value in ["0", "+5", "-0.0", ".5", "-.5", "5.", "1.5E+3"] {
            assert!(BigDecimal::from_str(value).is_ok(), "{value}");
        }
    }

    #[test]
    fn big_decimal_rejects_malformed_values() {
        for value in [
            "1.2.3", "-", "+", ".", "e", "E", "1e", "1e+", "--5", "1-2", "12-34", "..", "1.2e3.4",
            "+-1",
        ] {
            assert_eq!(
                BigDecimal::from_str(value),
                Err(BigNumberError::InvalidFormat(value.to_string())),
                "{value}"
            );
        }
    }

    #[test]
    fn big_decimal_rejects_out_of_range_exponents() {
        for value in [
            "1E99999999999999999999",
            "1E-99999999999999999999",
            "1e9223372036854775808",
            "1e-9223372036854775808",
            "1e999999999999999999999999999999999999999",
        ] {
            assert_eq!(
                BigDecimal::from_str(value),
                Err(BigNumberError::ExponentOutOfRange(value.to_string())),
                "{value}"
            );
        }
    }

    #[test]
    fn big_integer_rejects_json_injection() {
        // Reject strings with JSON special characters
        assert!(BigInteger::from_str("123, \"injected\": true").is_err());
        assert!(BigInteger::from_str("123}").is_err());
        assert!(BigInteger::from_str("{\"hacked\": 1}").is_err());
        assert!(BigInteger::from_str("123\"").is_err());
        assert!(BigInteger::from_str("123\\n456").is_err());
    }

    #[test]
    fn big_decimal_rejects_json_injection() {
        assert!(BigDecimal::from_str("123.45, \"injected\": true").is_err());
        assert!(BigDecimal::from_str("123.45}").is_err());
        assert!(BigDecimal::from_str("{\"hacked\": 1.0}").is_err());
    }

    #[test]
    fn big_integer_rejects_invalid_chars() {
        assert!(BigInteger::from_str("abc").is_err());
        assert!(BigInteger::from_str("123abc").is_err());
        assert!(BigInteger::from_str("12 34").is_err());
        assert!(BigInteger::from_str("").is_err());
    }

    #[test]
    fn big_integer_rejects_decimal_and_scientific() {
        // BigInteger should reject decimal points
        assert!(BigInteger::from_str("123.45").is_err());
        assert!(BigInteger::from_str("123.0").is_err());

        // BigInteger should reject scientific notation
        assert!(BigInteger::from_str("1e10").is_err());
        assert!(BigInteger::from_str("1E10").is_err());
        assert!(BigInteger::from_str("1.23e10").is_err());
    }

    #[test]
    fn big_decimal_rejects_invalid_chars() {
        assert!(BigDecimal::from_str("abc").is_err());
        assert!(BigDecimal::from_str("123.45abc").is_err());
        assert!(BigDecimal::from_str("12.34 56").is_err());
        assert!(BigDecimal::from_str("").is_err());
    }

    // --- Structural grammar -----------------------------------------------------

    #[test]
    fn big_integer_rejects_a_leading_plus() {
        // `+123` is not a JSON integer and is not accepted by BigInteger.
        // BigDecimal intentionally accepts a leading plus sign.
        assert!(BigInteger::from_str("+123").is_err());
        assert!(BigInteger::from_str("+0").is_err());
    }

    #[test]
    fn big_numbers_reject_bare_and_repeated_signs() {
        for bad in ["-", "+", "--5", "++5", "-+5", "+-5", "5-", "5+"] {
            assert!(
                BigInteger::from_str(bad).is_err(),
                "BigInteger::from_str({bad:?}) should fail"
            );
            assert!(
                BigDecimal::from_str(bad).is_err(),
                "BigDecimal::from_str({bad:?}) should fail"
            );
        }
    }

    #[test]
    fn big_decimal_rejects_malformed_separators_and_exponents() {
        for bad in [
            "1..2",   // repeated point
            "1.2.3",  // repeated point
            "0.0.0",  // repeated point
            "1e",     // no exponent digits
            "1E",     // no exponent digits
            "1e+",    // sign but no exponent digits
            "1e-",    // sign but no exponent digits
            "e10",    // no mantissa
            "-e10",   // no mantissa after the sign
            "1e2e3",  // repeated exponent
            "1.5e",   // no exponent digits
            "1e2.5",  // fractional exponent
            "1e 2",   // space inside the exponent
            "1 .5",   // space inside the mantissa
            ".",      // point only
            "1.2e3e", // trailing exponent marker
        ] {
            assert!(
                BigDecimal::from_str(bad).is_err(),
                "BigDecimal::from_str({bad:?}) should fail"
            );
        }
    }

    #[test]
    fn big_decimal_accepts_additional_supported_formats() {
        for good in [
            "0",
            "-0",
            "123",
            "-123",
            "1.5",
            "-0.5",
            "1e3",
            "1E3",
            "1e+9",
            "1e-9",
            "1.23e10",
            "1.23E-10",
            "10.0",
            "5e-3",
            "0.123456789012345678901234567890",
            "12345678901234567890.123",
            "1.234e500",
        ] {
            let bd = BigDecimal::from_str(good)
                .unwrap_or_else(|e| panic!("BigDecimal::from_str({good:?}) should succeed: {e}"));
            // The stored text is exactly the input: parsing never rewrites it.
            assert_eq!(bd.as_ref(), good);
        }
    }

    #[test]
    fn big_numbers_still_accept_leading_zeros() {
        // Retained for compatibility with previously released versions:
        // leading zeros have one numeric reading. They are *not* valid RFC
        // 8259 JSON numbers, and are rejected separately when an
        // arbitrary-precision value is emitted as a raw JSON number.
        for good in ["00123", "007", "-01", "0000", "-0000"] {
            let bi = BigInteger::from_str(good)
                .unwrap_or_else(|e| panic!("BigInteger::from_str({good:?}) should succeed: {e}"));
            assert_eq!(bi.as_ref(), good);
            let bd = BigDecimal::from_str(good)
                .unwrap_or_else(|e| panic!("BigDecimal::from_str({good:?}) should succeed: {e}"));
            assert_eq!(bd.as_ref(), good);
        }
        for good in ["00.1", "-01.5", "007e2"] {
            let bd = BigDecimal::from_str(good)
                .unwrap_or_else(|e| panic!("BigDecimal::from_str({good:?}) should succeed: {e}"));
            assert_eq!(bd.as_ref(), good);
        }
    }

    #[test]
    fn big_integer_accepts_only_digits_after_an_optional_minus() {
        for good in ["0", "-0", "1", "-1", "12345678901234567890"] {
            assert!(
                BigInteger::from_str(good).is_ok(),
                "BigInteger::from_str({good:?}) should succeed"
            );
        }
        for bad in ["1_000", "1,000", "0x1f", " 1", "1 ", "\t1", "1\n"] {
            assert!(
                BigInteger::from_str(bad).is_err(),
                "BigInteger::from_str({bad:?}) should fail"
            );
        }
    }

    #[test]
    fn big_number_error_reports_the_offending_input() {
        let err = BigInteger::from_str("+123").unwrap_err();
        assert_eq!(err, BigNumberError::InvalidFormat("+123".to_string()));
        assert!(err.to_string().contains("+123"));
    }

    #[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
    #[test]
    fn serde_deserialization_preserves_big_number_invariants() {
        let integer: BigInteger = serde_json::from_str(r#""00123""#).unwrap();
        assert_eq!(integer.as_ref(), "00123");
        assert!(serde_json::from_str::<BigInteger>(r#""+123""#).is_err());

        for value in ["+5", ".5", "-.5", "5.", "1.5E+3"] {
            let json = format!(r#""{value}""#);
            let decimal: BigDecimal = serde_json::from_str(&json).unwrap();
            assert_eq!(decimal.as_ref(), value);
        }

        for value in ["1.2.3", "--5", "1e"] {
            let json = format!(r#""{value}""#);
            assert!(serde_json::from_str::<BigDecimal>(&json).is_err());
        }

        let error = serde_json::from_str::<BigDecimal>(r#""1E99999999999999999999""#).unwrap_err();
        assert!(error.to_string().contains("outside the supported range"));
    }
}
