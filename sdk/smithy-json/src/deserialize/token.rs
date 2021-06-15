/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::deserialize::error::{Error, ErrorReason};
use crate::escape::unescape_string;
use smithy_types::instant::Format;
use smithy_types::{base64, Blob, Instant, Number};
use std::borrow::Cow;

pub use crate::escape::Error as EscapeError;

/// New-type around `&str` that indicates the string is an escaped JSON string.
/// Provides functions for retrieving the string in either form.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct EscapedStr<'a>(&'a str);

impl<'a> EscapedStr<'a> {
    pub fn new(value: &'a str) -> EscapedStr<'a> {
        EscapedStr(value)
    }

    /// Returns the escaped string value
    pub fn as_escaped_str(&self) -> &str {
        self.0
    }

    /// Unescapes the string and returns it.
    /// If the string doesn't need unescaping, it will be returned directly.
    pub fn to_unescaped(&self) -> Result<Cow<'a, str>, EscapeError> {
        unescape_string(self.0)
    }
}

/// Represents the location of a token
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Offset(pub usize);

impl Offset {
    /// Creates a custom error from the offset
    pub fn error(&self, msg: Cow<'static, str>) -> Error {
        Error::new(ErrorReason::Custom(msg), Some(self.0))
    }
}

/// Enum representing the different JSON tokens that can be returned by json_token_iter.
#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    StartArray {
        offset: Offset,
    },
    EndArray {
        offset: Offset,
    },
    ObjectKey {
        offset: Offset,
        key: EscapedStr<'a>,
    },
    StartObject {
        offset: Offset,
    },
    EndObject {
        offset: Offset,
    },
    ValueBool {
        offset: Offset,
        value: bool,
    },
    ValueNull {
        offset: Offset,
    },
    ValueNumber {
        offset: Offset,
        value: Number,
    },
    ValueString {
        offset: Offset,
        value: EscapedStr<'a>,
    },
}

impl<'a> Token<'a> {
    pub fn offset(&self) -> Offset {
        use Token::*;
        *match self {
            StartArray { offset } => offset,
            EndArray { offset } => offset,
            ObjectKey { offset, .. } => offset,
            StartObject { offset } => offset,
            EndObject { offset } => offset,
            ValueBool { offset, .. } => offset,
            ValueNull { offset } => offset,
            ValueNumber { offset, .. } => offset,
            ValueString { offset, .. } => offset,
        }
    }

    /// Builds an error from the token's offset
    pub fn error(&self, msg: Cow<'static, str>) -> Error {
        self.offset().error(msg)
    }
}

macro_rules! expect_fn {
    ($name:ident, $token:ident, $doc:tt) => {
        #[doc=$doc]
        pub fn $name(token_result: Option<Result<Token<'_>, Error>>) -> Result<(), Error> {
            match token_result.transpose()? {
                Some(Token::$token { .. }) => Ok(()),
                Some(token) => {
                    Err(token.error(Cow::Borrowed(concat!("expected ", stringify!($token)))))
                }
                None => Err(Error::new(
                    ErrorReason::Custom(Cow::Borrowed(concat!("expected ", stringify!($token)))),
                    None,
                )),
            }
        }
    };
}

expect_fn!(
    expect_start_object,
    StartObject,
    "Expects a [Token::StartObject] token and returns an error if it's not present."
);
expect_fn!(
    expect_start_array,
    StartArray,
    "Expects a [Token::StartArray] token and returns an error if it's not present."
);

macro_rules! expect_value_or_null_fn {
    ($name:ident, $token:ident, $typ:ident, $doc:tt) => {
        #[doc=$doc]
        pub fn $name(token: Option<Result<Token, Error>>) -> Result<Option<$typ>, Error> {
            match token.transpose()? {
                Some(Token::ValueNull { .. }) => Ok(None),
                Some(Token::$token { value, .. }) => Ok(Some(value)),
                _ => Err(Error::custom(concat!(
                    "expected ",
                    stringify!($token),
                    " or ValueNull"
                ))),
            }
        }
    };
}

expect_value_or_null_fn!(expect_bool_or_null, ValueBool, bool, "Expects a [Token::ValueBool] or [Token::ValueNull], and returns the bool value if it's not null.");
expect_value_or_null_fn!(expect_number_or_null, ValueNumber, Number, "Expects a [Token::ValueNumber] or [Token::ValueNull], and returns the [Number] value if it's not null.");
expect_value_or_null_fn!(expect_string_or_null, ValueString, EscapedStr, "Expects a [Token::ValueString] or [Token::ValueNull], and returns the [EscapedStr] value if it's not null.");

/// Expects a [Token::ValueString] or [Token::ValueNull]. If the value is a string, its **unescaped** value will be returned.
pub fn expect_unescaped_string_or_null(
    token: Option<Result<Token<'_>, Error>>,
) -> Result<Option<String>, Error> {
    Ok(match expect_string_or_null(token)? {
        Some(value) => Some(value.to_unescaped()?.to_string()),
        None => None,
    })
}

/// Expects a [Token::ValueString] or [Token::ValueNull]. If the value is a string, it interprets it as a base64 encoded [Blob] value.
pub fn expect_blob_or_null(token: Option<Result<Token<'_>, Error>>) -> Result<Option<Blob>, Error> {
    Ok(match expect_string_or_null(token)? {
        Some(value) => Some(Blob::new(base64::decode(value.as_escaped_str()).map_err(
            |err| {
                Error::new(
                    ErrorReason::Custom(Cow::Owned(format!("failed to decode base64: {}", err))),
                    None,
                )
            },
        )?)),
        None => None,
    })
}

/// Expects a [Token::ValueNull], [Token::ValueString], or [Token::ValueNumber] depending
/// on the passed in `timestamp_format`. If there is a non-null value, it interprets it as an
/// [Instant] in the requested format.
pub fn expect_timestamp_or_null(
    token: Option<Result<Token<'_>, Error>>,
    timestamp_format: Format,
) -> Result<Option<Instant>, Error> {
    Ok(match timestamp_format {
        Format::EpochSeconds => {
            expect_number_or_null(token)?.map(|v| Instant::from_f64(v.to_f64()))
        }
        Format::DateTime | Format::HttpDate => expect_string_or_null(token)?
            .map(|v| Instant::from_str(v.as_escaped_str(), timestamp_format))
            .transpose()
            .map_err(|err| {
                Error::new(
                    ErrorReason::Custom(Cow::Owned(format!("failed to parse timestamp: {}", err))),
                    None,
                )
            })?,
    })
}

/// Skips an entire value in the token stream. Errors if it isn't a value.
pub fn skip_value<'a>(
    tokens: &mut impl Iterator<Item = Result<Token<'a>, Error>>,
) -> Result<(), Error> {
    skip_inner(false, tokens)
}

fn skip_inner<'a>(
    inside_obj_or_array: bool,
    tokens: &mut impl Iterator<Item = Result<Token<'a>, Error>>,
) -> Result<(), Error> {
    loop {
        match tokens.next().transpose()? {
            Some(Token::StartObject { .. }) | Some(Token::StartArray { .. }) => {
                skip_inner(true, tokens)?;
                if !inside_obj_or_array {
                    break;
                }
            }
            Some(Token::EndObject { .. }) | Some(Token::EndArray { .. }) => {
                debug_assert!(inside_obj_or_array);
                break;
            }
            Some(Token::ValueNull { .. })
            | Some(Token::ValueBool { .. })
            | Some(Token::ValueNumber { .. })
            | Some(Token::ValueString { .. }) => {
                if !inside_obj_or_array {
                    break;
                }
            }
            Some(Token::ObjectKey { .. }) => {}
            _ => return Err(Error::custom("expected value")),
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::deserialize::error::ErrorReason::UnexpectedToken;
    use crate::deserialize::json_token_iter;

    pub fn start_array<'a>(offset: usize) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::StartArray {
            offset: Offset(offset),
        }))
    }

    pub fn end_array<'a>(offset: usize) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::EndArray {
            offset: Offset(offset),
        }))
    }

    pub fn start_object<'a>(offset: usize) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::StartObject {
            offset: Offset(offset),
        }))
    }

    pub fn end_object<'a>(offset: usize) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::EndObject {
            offset: Offset(offset),
        }))
    }

    pub fn object_key(offset: usize, key: &str) -> Option<Result<Token, Error>> {
        Some(Ok(Token::ObjectKey {
            offset: Offset(offset),
            key: EscapedStr::new(key),
        }))
    }

    pub fn value_bool<'a>(offset: usize, boolean: bool) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::ValueBool {
            offset: Offset(offset),
            value: boolean,
        }))
    }

    pub fn value_number<'a>(offset: usize, number: Number) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::ValueNumber {
            offset: Offset(offset),
            value: number,
        }))
    }

    pub fn value_null<'a>(offset: usize) -> Option<Result<Token<'a>, Error>> {
        Some(Ok(Token::ValueNull {
            offset: Offset(offset),
        }))
    }

    pub fn value_string(offset: usize, string: &str) -> Option<Result<Token, Error>> {
        Some(Ok(Token::ValueString {
            offset: Offset(offset),
            value: EscapedStr::new(string),
        }))
    }

    #[test]
    fn skip_simple_value() {
        let mut tokens = json_token_iter(b"null true");
        skip_value(&mut tokens).unwrap();
        assert!(matches!(
            tokens.next(),
            Some(Ok(Token::ValueBool { value: true, .. }))
        ))
    }

    #[test]
    fn skip_array() {
        let mut tokens = json_token_iter(b"[1, 2, 3, 4] true");
        skip_value(&mut tokens).unwrap();
        assert!(matches!(
            tokens.next(),
            Some(Ok(Token::ValueBool { value: true, .. }))
        ))
    }

    #[test]
    fn skip_object() {
        let mut tokens = json_token_iter(b"{\"one\": 5, \"two\": 3} true");
        skip_value(&mut tokens).unwrap();
        assert!(matches!(
            tokens.next(),
            Some(Ok(Token::ValueBool { value: true, .. }))
        ))
    }

    #[test]
    fn mismatched_braces() {
        // The skip_value function doesn't need to explicitly handle these cases since
        // token iterator's parser handles them. This test confirms that assumption.
        assert_eq!(
            Err(Error::new(UnexpectedToken(']', "'}', ','"), Some(10),)),
            skip_value(&mut json_token_iter(br#"[{"foo": 5]}"#))
        );
        assert_eq!(
            Err(Error::new(UnexpectedToken(']', "'}', ','"), Some(9),)),
            skip_value(&mut json_token_iter(br#"{"foo": 5]}"#))
        );
        assert_eq!(
            Err(Error::new(UnexpectedToken('}', "']', ','"), Some(4),)),
            skip_value(&mut json_token_iter(br#"[5,6}"#))
        );
    }

    #[test]
    fn skip_nested() {
        let mut tokens = json_token_iter(
            br#"
            {"struct": {"foo": 5, "bar": 11, "arr": [1, 2, 3, {}, 5, []]},
             "arr": [[], [[]], [{"arr":[]}]],
             "simple": "foo"}
            true
        "#,
        );
        skip_value(&mut tokens).unwrap();
        assert!(matches!(
            tokens.next(),
            Some(Ok(Token::ValueBool { value: true, .. }))
        ))
    }

    #[test]
    fn test_expect_start_object() {
        assert_eq!(
            Err(Error::new(
                ErrorReason::Custom("expected StartObject".into()),
                Some(2)
            )),
            expect_start_object(value_bool(2, true))
        );
        assert_eq!(Ok(()), expect_start_object(start_object(0)));
    }

    #[test]
    fn test_expect_start_array() {
        assert_eq!(
            Err(Error::new(
                ErrorReason::Custom("expected StartArray".into()),
                Some(2)
            )),
            expect_start_array(value_bool(2, true))
        );
        assert_eq!(Ok(()), expect_start_array(start_array(0)));
    }

    #[test]
    fn test_expect_string_or_null() {
        assert_eq!(Ok(None), expect_string_or_null(value_null(0)));
        assert_eq!(
            Ok(Some(EscapedStr("test\\n"))),
            expect_string_or_null(value_string(0, "test\\n"))
        );
        assert_eq!(
            Err(Error::custom("expected ValueString or ValueNull")),
            expect_string_or_null(value_bool(0, true))
        );
    }

    #[test]
    fn test_expect_unescaped_string_or_null() {
        assert_eq!(Ok(None), expect_unescaped_string_or_null(value_null(0)));
        assert_eq!(
            Ok(Some("test\n".to_string())),
            expect_unescaped_string_or_null(value_string(0, "test\\n"))
        );
        assert_eq!(
            Err(Error::custom("expected ValueString or ValueNull")),
            expect_unescaped_string_or_null(value_bool(0, true))
        );
    }

    #[test]
    fn test_expect_number_or_null() {
        assert_eq!(Ok(None), expect_number_or_null(value_null(0)));
        assert_eq!(
            Ok(Some(Number::PosInt(5))),
            expect_number_or_null(value_number(0, Number::PosInt(5)))
        );
        assert_eq!(
            Err(Error::custom("expected ValueNumber or ValueNull")),
            expect_number_or_null(value_bool(0, true))
        );
    }

    #[test]
    fn test_expect_blob_or_null() {
        assert_eq!(Ok(None), expect_blob_or_null(value_null(0)));
        assert_eq!(
            Ok(Some(Blob::new(b"hello!".to_vec()))),
            expect_blob_or_null(value_string(0, "aGVsbG8h"))
        );
        assert_eq!(
            Err(Error::custom("expected ValueString or ValueNull")),
            expect_blob_or_null(value_bool(0, true))
        );
    }

    #[test]
    fn test_expect_timestamp_or_null() {
        assert_eq!(
            Ok(None),
            expect_timestamp_or_null(value_null(0), Format::HttpDate)
        );
        assert_eq!(
            Ok(Some(Instant::from_f64(2048.0))),
            expect_timestamp_or_null(value_number(0, Number::Float(2048.0)), Format::EpochSeconds)
        );
        assert_eq!(
            Ok(Some(Instant::from_f64(1445412480.0))),
            expect_timestamp_or_null(
                value_string(0, "Wed, 21 Oct 2015 07:28:00 GMT"),
                Format::HttpDate
            )
        );
        assert_eq!(
            Ok(Some(Instant::from_f64(1445412480.0))),
            expect_timestamp_or_null(value_string(0, "2015-10-21T07:28:00Z"), Format::DateTime)
        );
        assert_eq!(
            Err(Error::custom("expected ValueNumber or ValueNull")),
            expect_timestamp_or_null(value_string(0, "wrong"), Format::EpochSeconds)
        );
        assert_eq!(
            Err(Error::custom("expected ValueString or ValueNull")),
            expect_timestamp_or_null(value_number(0, Number::Float(0.0)), Format::DateTime)
        );
    }
}
