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
}

impl std::fmt::Display for BigNumberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BigNumberError::InvalidFormat(s) => write!(f, "invalid number format: {s}"),
        }
    }
}

impl std::error::Error for BigNumberError {}

/// `true` iff `s` is non-empty and consists entirely of ASCII digits.
fn is_ascii_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Validates that a string is a valid BigInteger: an optional leading
/// `-` followed by one or more ASCII digits.
///
/// A leading `+` and any embedded non-digits are rejected. BigIntegers
/// are emitted verbatim as JSON numbers, and a JSON number permits
/// neither a leading `+` nor embedded non-digits, so the stored form
/// must already be a valid JSON integer.
fn is_valid_big_integer(s: &str) -> bool {
    is_ascii_digits(s.strip_prefix('-').unwrap_or(s))
}

/// Validates that a string is a valid BigDecimal, matching the JSON
/// number grammar (minus a leading `+`):
/// `'-'? digits ('.' digits)? (('e' | 'E') ('+' | '-')? digits)?`.
///
/// BigDecimals are emitted verbatim as JSON numbers, so the stored form
/// must be a valid JSON number. Unlike a plain character-set check this
/// rejects malformed inputs such as `"1.2.3"`, `"--5"`, `"1e"`, `"+."`,
/// and a leading `+`.
fn is_valid_big_decimal(s: &str) -> bool {
    let mantissa_and_exp = s.strip_prefix('-').unwrap_or(s);

    let (mantissa, exponent) = match mantissa_and_exp.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (mantissa_and_exp, None),
    };

    let mantissa_ok = match mantissa.split_once('.') {
        Some((int_part, frac_part)) => is_ascii_digits(int_part) && is_ascii_digits(frac_part),
        None => is_ascii_digits(mantissa),
    };

    let exponent_ok = match exponent {
        None => true,
        Some(exponent) => is_ascii_digits(exponent.strip_prefix(['+', '-']).unwrap_or(exponent)),
    };

    mantissa_ok && exponent_ok
}

/// A BigInteger represented as a string.
///
/// This type does not perform arithmetic operations. Users should parse the string
/// with their preferred big integer library.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    derive(serde::Deserialize)
)]
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
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-serialize"),
    derive(serde::Serialize)
)]
#[cfg_attr(
    all(aws_sdk_unstable, feature = "serde-deserialize"),
    derive(serde::Deserialize)
)]
pub struct BigDecimal(String);

impl Default for BigDecimal {
    fn default() -> Self {
        Self("0.0".to_string())
    }
}

impl std::str::FromStr for BigDecimal {
    type Err = BigNumberError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !is_valid_big_decimal(s) {
            return Err(BigNumberError::InvalidFormat(s.to_string()));
        }
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for BigDecimal {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl BigDecimal {
    /// Upper bound on the length of the integer string
    /// [`Self::to_integer_string`] will materialize. A scientific-notation
    /// exponent can request an arbitrarily long run of trailing zeros;
    /// this cap stops a pathological exponent (e.g. `1e1000000000`) from
    /// triggering a huge allocation. ~1M digits is far beyond any
    /// practical value.
    const MAX_INTEGER_DIGITS: usize = 1 << 20;

    /// Returns this decimal truncated toward zero as a plain integer
    /// string (`-?[0-9]+`), or `None` if the integer magnitude is too
    /// large to materialize (see [`Self::MAX_INTEGER_DIGITS`]).
    ///
    /// Fractional digits are dropped — per the SEP, numeric coercion
    /// ignores loss of precision — and any scientific-notation exponent
    /// is expanded so the integer magnitude is preserved. For example
    /// `1.23e10` yields `"12300000000"`, `1.23e1` yields `"12"`, and
    /// `5e-3` yields `"0"`.
    pub(crate) fn to_integer_string(&self) -> Option<String> {
        // `self.0` is always valid per `is_valid_big_decimal`.
        let (negative, rest) = match self.0.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, self.0.as_str()),
        };

        let (mantissa, exp) = match rest.split_once(['e', 'E']) {
            Some((mantissa, exp_str)) => match exp_str.parse::<i64>() {
                Ok(exp) => (mantissa, exp),
                // Exponent doesn't fit in i64: a huge positive exponent is
                // too large to materialize; a huge negative one rounds to
                // zero.
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
                if point > Self::MAX_INTEGER_DIGITS {
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
    fn big_integer_sign_handling() {
        // A leading '-' is allowed; a leading '+' is rejected because it is
        // not valid in a JSON number (big numbers are emitted verbatim).
        assert!(BigInteger::from_str("-123").is_ok());
        assert_eq!(BigInteger::from_str("-123").unwrap().as_ref(), "-123");
        assert!(BigInteger::from_str("+123").is_err());
        assert!(BigInteger::from_str("--5").is_err());
        assert!(BigInteger::from_str("-").is_err());
    }

    #[test]
    fn big_decimal_rejects_invalid_chars() {
        assert!(BigDecimal::from_str("abc").is_err());
        assert!(BigDecimal::from_str("123.45abc").is_err());
        assert!(BigDecimal::from_str("12.34 56").is_err());
        assert!(BigDecimal::from_str("").is_err());
    }

    #[test]
    fn big_integer_rejects_malformed_grammar() {
        for bad in ["+123", "--5", "-", "1.0", "1e3", "", "1 2", "0x1f"] {
            assert!(
                BigInteger::from_str(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
        for good in ["0", "123", "-123", "00123"] {
            assert!(
                BigInteger::from_str(good).is_ok(),
                "expected {good:?} to be accepted"
            );
        }
    }

    #[test]
    fn big_decimal_rejects_malformed_grammar() {
        for bad in [
            "1.2.3", "--5", "1e", "+.", "+1.0", ".5", "1.", "e10", "1e+", "1e2e3", "-", "1..2",
        ] {
            assert!(
                BigDecimal::from_str(bad).is_err(),
                "expected {bad:?} to be rejected"
            );
        }
    }

    #[test]
    fn big_decimal_accepts_valid_grammar() {
        for good in [
            "0", "123", "-123", "1.5", "-0.5", "1.23e10", "1.23E-10", "1e3", "1.0e+9",
        ] {
            assert!(
                BigDecimal::from_str(good).is_ok(),
                "expected {good:?} to be accepted"
            );
        }
    }

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
            ("00123", "123"),           // leading zeros normalized
            ("1.23E10", "12300000000"), // uppercase E
        ];
        for (input, expected) in cases {
            let bd = BigDecimal::from_str(input).unwrap();
            assert_eq!(
                bd.to_integer_string().as_deref(),
                Some(expected),
                "to_integer_string({input:?})"
            );
        }
    }

    #[test]
    fn big_decimal_to_integer_string_guards_pathological_exponent() {
        // Exponent fits in i64 but is absurdly large: refuse to materialize
        // rather than allocating ~10^9 bytes.
        assert_eq!(
            BigDecimal::from_str("1e1000000000")
                .unwrap()
                .to_integer_string(),
            None
        );
        // Exponent overflows i64 entirely.
        assert_eq!(
            BigDecimal::from_str("1e99999999999999999999")
                .unwrap()
                .to_integer_string(),
            None
        );
        // A huge *negative* exponent rounds to zero with no allocation.
        assert_eq!(
            BigDecimal::from_str("1e-99999999999999999999")
                .unwrap()
                .to_integer_string()
                .as_deref(),
            Some("0")
        );
    }
}
