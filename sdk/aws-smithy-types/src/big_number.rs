/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Big number types represented as strings.
//!
//! These types are simple string wrappers that allow users to parse and format
//! big numbers using their preferred library.

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

/// Validates that a string is a valid BigInteger format.
/// Only allows digits and an optional leading sign.
fn is_valid_big_integer(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // Check first character (can be sign or digit)
    match chars.next() {
        Some('-') | Some('+') | Some('0'..='9') => {}
        _ => return false,
    }

    // Rest must be digits only
    chars.all(|c| c.is_ascii_digit())
}

/// Validates that a string is a valid BigDecimal format.
/// Allows digits, sign, decimal point, and scientific notation.
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigInteger(String);

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BigDecimal(String);

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
    fn big_integer_accepts_signs() {
        assert!(BigInteger::from_str("+123").is_ok());
        assert!(BigInteger::from_str("-123").is_ok());
        assert_eq!(BigInteger::from_str("+123").unwrap().as_ref(), "+123");
    }

    #[test]
    fn big_decimal_rejects_invalid_chars() {
        assert!(BigDecimal::from_str("abc").is_err());
        assert!(BigDecimal::from_str("123.45abc").is_err());
        assert!(BigDecimal::from_str("12.34 56").is_err());
        assert!(BigDecimal::from_str("").is_err());
    }
}
