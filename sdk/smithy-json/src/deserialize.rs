/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::escape::unescape_string;
use smithy_types::Number;
use std::borrow::Cow;
use std::fmt;
use std::str::Utf8Error;

pub use crate::escape::Error as EscapeError;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorReason {
    InvalidUtf8,
    InvalidEscape(char),
    InvalidNumber,
    ExpectedLiteral(String),
    UnexpectedControlCharacter(u8),
    UnexpectedToken(char, &'static str),
    UnexpectedEOS,
}
use ErrorReason::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    reason: ErrorReason,
    offset: usize,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error at offset {}: ", self.offset)?;
        match &self.reason {
            InvalidUtf8 => write!(f, "invalid UTF-8 codepoint in JSON stream"),
            InvalidEscape(escape) => write!(f, "invalid JSON escape: \\{}", escape),
            InvalidNumber => write!(f, "invalid number"),
            ExpectedLiteral(literal) => write!(f, "expected literal: {}", literal),
            UnexpectedControlCharacter(value) => write!(
                f,
                "encountered unescaped control character in string: 0x{:X}",
                value
            ),
            UnexpectedToken(token, expected) => write!(
                f,
                "unexpected token '{}'. Expected one of {}",
                token, expected
            ),
            UnexpectedEOS => write!(f, "unexpected end of stream"),
        }
    }
}

impl From<Utf8Error> for ErrorReason {
    fn from(_: Utf8Error) -> Self {
        InvalidUtf8
    }
}

/// New-type around `&str` that indicates the string is an escaped JSON string.
/// Provides functions for retrieving the string in either form.
#[derive(Debug, PartialEq, Eq)]
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

/// Enum representing the different JSON tokens that can be returned by [json_token_iter].
#[derive(Debug, PartialEq)]
pub enum Token<'a> {
    StartArray,
    EndArray,
    ObjectKey(EscapedStr<'a>),
    StartObject,
    EndObject,
    ValueBool(bool),
    ValueNull,
    ValueNumber(Number),
    ValueString(EscapedStr<'a>),
}

/// Returns an Iterator of `Result<Token, Error>` over an slice of bytes.
pub fn json_token_iter(input: &[u8]) -> JsonTokenIterator {
    JsonTokenIterator {
        input,
        index: 0,
        state_stack: vec![State::Initial],
    }
}

/// Internal parser state for the iterator. Used to context between successive `next` calls.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum State {
    /// Entry point. Expecting any JSON value.
    Initial,
    /// Expecting the next token to be the *first* value in an array, or the end of the array.
    ArrayFirstValueOrEnd,
    /// Expecting the next token to the next value in an array, or the end of the array.
    ArrayNextValueOrEnd,
    /// Expecting the next token to be the *first* key in the object, or the end of the object.
    ObjectFirstKeyOrEnd,
    /// Expecting the next token to the next object key, or the end of the object.
    ObjectNextKeyOrEnd,
    /// Expecting the next token to be the value of a field in an object.
    ObjectFieldValue,
}

/// An iterator over a `&[u8]` that yields `Result<Token, Error>` with [Token] being JSON tokens.
/// Construct with [json_token_iter].
pub struct JsonTokenIterator<'a> {
    input: &'a [u8],
    index: usize,
    state_stack: Vec<State>,
}

impl<'a> JsonTokenIterator<'a> {
    /// Previews the next byte.
    fn peek_byte(&self) -> Option<u8> {
        if self.index >= self.input.len() {
            None
        } else {
            Some(self.input[self.index])
        }
    }

    /// Expects there to be another byte coming up, and previews it.
    /// If there isn't, an `UnexpectedEOS` error is returned.
    fn peek_expect(&self) -> Result<u8, Error> {
        self.peek_byte().ok_or_else(|| self.error(UnexpectedEOS))
    }

    /// Advances to the next byte in the stream.
    fn advance(&mut self) {
        if self.index < self.input.len() {
            self.index += 1;
        }
    }

    /// Advances and returns the next byte in the stream.
    fn next_byte(&mut self) -> Option<u8> {
        let next = self.peek_byte();
        self.advance();
        next
    }

    /// Expects there to be another byte coming up, and returns it while advancing.
    /// If there isn't, an `UnexpectedEOS` error is returned.
    fn next_expect(&mut self) -> Result<u8, Error> {
        self.next_byte().ok_or_else(|| self.error(UnexpectedEOS))
    }

    /// Creates an error at the given `offset` in the stream.
    fn error_at(&self, offset: usize, reason: ErrorReason) -> Error {
        Error { reason, offset }
    }

    /// Creates an error at the current offset in the stream.
    fn error(&self, reason: ErrorReason) -> Error {
        self.error_at(self.index, reason)
    }

    /// Advances until it hits a non-whitespace character or the end of the slice.
    fn discard_whitespace(&mut self) {
        while let Some(byte) = self.peek_byte() {
            match byte {
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    /// Returns the top of the state stack (current state).
    fn state(&self) -> State {
        self.state_stack[self.state_stack.len() - 1]
    }

    /// Replaces the top of the state stack with a new `state`.
    fn replace_state(&mut self, state: State) {
        self.state_stack.pop();
        self.state_stack.push(state);
    }

    /// Discards the '{' character and pushes the `ObjectFirstKeyOrEnd` state.
    fn start_object(&mut self) -> Token<'a> {
        let byte = self.next_byte();
        debug_assert_eq!(byte, Some(b'{'));
        self.state_stack.push(State::ObjectFirstKeyOrEnd);
        Token::StartObject
    }

    /// Discards the '}' character and pops the current state.
    fn end_object(&mut self) -> Token<'a> {
        let (byte, state) = (self.next_byte(), self.state_stack.pop());
        debug_assert_eq!(byte, Some(b'}'));
        debug_assert!(
            state == Some(State::ObjectFirstKeyOrEnd) || state == Some(State::ObjectNextKeyOrEnd)
        );
        Token::EndObject
    }

    /// Discards the '[' character and pushes the `ArrayFirstValueOrEnd` state.
    fn start_array(&mut self) -> Token<'a> {
        let byte = self.next_byte();
        debug_assert_eq!(byte, Some(b'['));
        self.state_stack.push(State::ArrayFirstValueOrEnd);
        Token::StartArray
    }

    /// Discards the ']' character and pops the current state.
    fn end_array(&mut self) -> Token<'a> {
        let (byte, state) = (self.next_byte(), self.state_stack.pop());
        debug_assert_eq!(byte, Some(b']'));
        debug_assert!(
            state == Some(State::ArrayFirstValueOrEnd) || state == Some(State::ArrayNextValueOrEnd)
        );
        Token::EndArray
    }

    /// Reads a JSON string out of the stream.
    fn read_string(&mut self) -> Result<&'a str, Error> {
        // Skip the starting quote
        let quote_byte = self.next_byte();
        debug_assert_eq!(quote_byte, Some(b'\"'));

        // Read bytes until a non-escaped end-quote, unescaping sequences as needed on the fly
        let start = self.index;
        loop {
            match self.peek_expect()? {
                b'"' => {
                    let value = std::str::from_utf8(&self.input[start..self.index])
                        .map_err(|_| self.error(InvalidUtf8))?;
                    self.advance();
                    return Ok(value);
                }
                b'\\' => match self.next_expect()? {
                    b'\\' | b'/' | b'"' | b'b' | b'f' | b'n' | b'r' | b't' => self.advance(),
                    b'u' => {
                        if self.index + 4 > self.input.len() {
                            return Err(self.error_at(self.input.len(), UnexpectedEOS));
                        }
                        self.index += 4;
                    }
                    byte => return Err(self.error(InvalidEscape(byte.into()))),
                },
                byte @ 0x00..=0x1F => return Err(self.error(UnexpectedControlCharacter(byte))),
                _ => self.advance(),
            }
        }
    }

    /// Expects the given literal to be next in the stream.
    fn expect_literal(&mut self, expected: &[u8]) -> Result<(), Error> {
        let (start, end) = (self.index, self.index + expected.len());
        if end > self.input.len() {
            return Err(self.error_at(self.input.len(), UnexpectedEOS));
        }
        if expected != &self.input[start..end] {
            return Err(self.error_at(
                start,
                ExpectedLiteral(std::str::from_utf8(expected).unwrap().into()),
            ));
        }
        self.index = end;
        Ok(())
    }

    /// Expects a literal `null` next in the stream.
    fn expect_null(&mut self) -> Result<Token<'a>, Error> {
        self.expect_literal(b"null")?;
        Ok(Token::ValueNull)
    }

    /// Expects a boolean `true` / `false` to be next in the stream and returns its value.
    fn expect_bool(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b't' => {
                self.expect_literal(b"true")?;
                Ok(Token::ValueBool(true))
            }
            b'f' => {
                self.expect_literal(b"false")?;
                Ok(Token::ValueBool(false))
            }
            _ => unreachable!(),
        }
    }

    /// Advances passed the exponent part of a floating point number.
    fn skip_exponent(&mut self) {
        self.advance();
        match self.peek_byte() {
            Some(b'-') => self.advance(),
            Some(b'+') => self.advance(),
            _ => {}
        }
        while let Some(b'0'..=b'9') = self.peek_byte() {
            self.advance();
        }
    }

    /// Advances passed the decimal part of a floating point number.
    fn skip_decimal(&mut self) {
        self.advance();
        while let Some(byte) = self.peek_byte() {
            match byte {
                b'0'..=b'9' => self.advance(),
                b'e' | b'E' => self.skip_exponent(),
                _ => break,
            }
        }
    }

    /// Starting from the current location in the stream, this advances until
    /// it finds a character that doesn't look like its part of a number, and then
    /// returns `(start_index, end_index, negative, floating)`, with `start_index`
    /// and `end_index` representing the slice of the stream that is the number,
    /// `negative` whether or not it is a negative number, and `floating` whether or not
    /// it is a floating point number.
    fn scan_number(&mut self) -> (usize, usize, bool, bool) {
        let start_index = self.index;
        let negative = if self.peek_byte() == Some(b'-') {
            self.advance();
            true
        } else {
            false
        };
        let mut floating = false;
        while let Some(byte) = self.peek_byte() {
            match byte {
                b'0'..=b'9' => self.advance(),
                b'.' => {
                    floating = true;
                    self.skip_decimal();
                }
                b'e' | b'E' => {
                    floating = true;
                    self.skip_exponent();
                }
                _ => break,
            }
        }
        (start_index, self.index, negative, floating)
    }

    /// Expects a number in the stream, and returns its value.
    fn expect_number(&mut self) -> Result<Token<'a>, Error> {
        let (start, end, negative, floating) = self.scan_number();
        let number_slice = &self.input[start..end];

        // Unsafe: we examined every character in the range, and they are all number characters
        debug_assert!(std::str::from_utf8(number_slice).is_ok());
        let number_str = unsafe { std::str::from_utf8_unchecked(number_slice) };

        use std::str::FromStr;
        Ok(Token::ValueNumber(if floating {
            Number::Float(
                f64::from_str(&number_str).map_err(|_| self.error_at(start, InvalidNumber))?,
            )
        } else if negative {
            // If the negative value overflows, then stuff it into an f64
            let positive =
                u64::from_str(&number_str[1..]).map_err(|_| self.error_at(start, InvalidNumber))?;
            let negative = positive.wrapping_neg() as i64;
            if negative > 0 {
                Number::Float(-(positive as f64))
            } else {
                Number::NegInt(negative as i64)
            }
        } else {
            Number::PosInt(
                u64::from_str(&number_str).map_err(|_| self.error_at(start, InvalidNumber))?,
            )
        }))
    }

    /// Reads a value from the stream and returns the next token. For objects and arrays,
    /// the entire object or array will not be ready, but rather, a [Token::StartObject]/[Token::StartArray]
    /// will be returned.
    fn read_value(&mut self) -> Result<Token<'a>, Error> {
        self.discard_whitespace();
        match self.peek_expect()? {
            b'{' => Ok(self.start_object()),
            b'[' => Ok(self.start_array()),
            b'"' => self
                .read_string()
                .map(|s| Token::ValueString(EscapedStr(s))),
            byte => {
                let value = match byte {
                    b'n' => self.expect_null(),
                    b't' | b'f' => self.expect_bool(),
                    b'-' | (b'0'..=b'9') => self.expect_number(),
                    byte => Err(self.error(UnexpectedToken(
                        byte.into(),
                        "'{', '[', '\"', 'null', 'true', 'false', <number>",
                    ))),
                }?;
                // Verify there are no unexpected trailers on the end of the value
                if let Some(byte) = self.peek_byte() {
                    match byte {
                        b' ' | b'\t' | b'\r' | b'\n' | b'}' | b']' | b',' => {}
                        _ => {
                            return Err(self.error(UnexpectedToken(
                                byte.into(),
                                "<whitespace>, '}', ']', ','",
                            )))
                        }
                    }
                }
                Ok(value)
            }
        }
    }

    /// Handles the [State::ArrayFirstValueOrEnd] state.
    fn state_array_first_value_or_end(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b']' => Ok(self.end_array()),
            _ => {
                self.replace_state(State::ArrayNextValueOrEnd);
                self.read_value()
            }
        }
    }

    /// Handles the [State::ArrayNextValueOrEnd] state.
    fn state_array_next_value_or_end(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b']' => Ok(self.end_array()),
            b',' => {
                self.advance();
                self.read_value()
            }
            byte => Err(self.error(UnexpectedToken(byte.into(), "']', ','"))),
        }
    }

    /// Expects an object key.
    fn object_key(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b'"' => {
                self.replace_state(State::ObjectFieldValue);
                self.read_string().map(|s| Token::ObjectKey(EscapedStr(s)))
            }
            byte => Err(self.error(UnexpectedToken(byte.into(), "'\"'"))),
        }
    }

    /// Handles the [State::ObjectFirstKeyOrEnd] state.
    fn state_object_first_key_or_end(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b'}' => Ok(self.end_object()),
            _ => self.object_key(),
        }
    }

    /// Handles the [State::ObjectNextKeyOrEnd] state.
    fn state_object_next_key_or_end(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b'}' => Ok(self.end_object()),
            b',' => {
                self.advance();
                self.discard_whitespace();
                self.object_key()
            }
            byte => Err(self.error(UnexpectedToken(byte.into(), "'}', ','"))),
        }
    }

    /// Handles the [State::ObjectFieldValue] state.
    fn state_object_field_value(&mut self) -> Result<Token<'a>, Error> {
        match self.peek_expect()? {
            b':' => {
                self.advance();
                self.replace_state(State::ObjectNextKeyOrEnd);
                self.read_value()
            }
            byte => Err(self.error(UnexpectedToken(byte.into(), "':'"))),
        }
    }
}

impl<'a> Iterator for JsonTokenIterator<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(self.index <= self.input.len());
        if self.index == self.input.len() {
            return None;
        }

        self.discard_whitespace();
        let result = match self.state() {
            State::Initial => self.peek_byte().map(|_| self.read_value()),
            State::ArrayFirstValueOrEnd => Some(self.state_array_first_value_or_end()),
            State::ArrayNextValueOrEnd => Some(self.state_array_next_value_or_end()),
            State::ObjectFirstKeyOrEnd => Some(self.state_object_first_key_or_end()),
            State::ObjectNextKeyOrEnd => Some(self.state_object_next_key_or_end()),
            State::ObjectFieldValue => Some(self.state_object_field_value()),
        };
        // Invalidate the stream if we encountered an error
        if result.as_ref().map(|r| r.is_err()).unwrap_or(false) {
            self.index = self.input.len();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::deserialize::{json_token_iter, Error, ErrorReason, EscapedStr, Token};
    use proptest::prelude::*;
    use smithy_types::Number;

    #[test]
    fn test_empty() {
        assert_eq!(None, json_token_iter(b"").next());
        assert_eq!(None, json_token_iter(b" ").next());
        assert_eq!(None, json_token_iter(b"\t").next());
    }

    #[test]
    fn test_empty_string() {
        let mut iter = json_token_iter(b"\"\"");
        assert_eq!(Some(Ok(Token::ValueString(EscapedStr("")))), iter.next());
        assert_eq!(None, iter.next());

        let mut iter = json_token_iter(b" \r\n\t \"\"  ");
        assert_eq!(Some(Ok(Token::ValueString(EscapedStr("")))), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_empty_array() {
        let mut iter = json_token_iter(b"[]");
        assert_eq!(Some(Ok(Token::StartArray)), iter.next());
        assert_eq!(Some(Ok(Token::EndArray)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_empty_object() {
        let mut iter = json_token_iter(b"{}");
        assert_eq!(Some(Ok(Token::StartObject)), iter.next());
        assert_eq!(Some(Ok(Token::EndObject)), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_null() {
        assert_eq!(
            Some(Ok(Token::ValueNull)),
            json_token_iter(b" null ").next()
        );

        let tokens: Result<Vec<Token>, Error> = json_token_iter(b"[null, null,null]").collect();
        assert_eq!(
            vec![
                Token::StartArray,
                Token::ValueNull,
                Token::ValueNull,
                Token::ValueNull,
                Token::EndArray
            ],
            tokens.unwrap()
        );

        assert!(json_token_iter(b"n").next().unwrap().is_err());
        assert!(json_token_iter(b"nul").next().unwrap().is_err());
        assert!(json_token_iter(b"nulll").next().unwrap().is_err());
    }

    #[test]
    fn test_bools() {
        assert!(json_token_iter(b"tru").next().unwrap().is_err());
        assert!(json_token_iter(b"truee").next().unwrap().is_err());
        assert!(json_token_iter(b"f").next().unwrap().is_err());
        assert!(json_token_iter(b"falsee").next().unwrap().is_err());
        assert_eq!(
            Some(Ok(Token::ValueBool(true))),
            json_token_iter(b" true ").next()
        );
        assert_eq!(
            Some(Ok(Token::ValueBool(false))),
            json_token_iter(b"false").next()
        );

        let tokens: Result<Vec<Token>, Error> = json_token_iter(b"[true,false]").collect();
        assert_eq!(
            vec![
                Token::StartArray,
                Token::ValueBool(true),
                Token::ValueBool(false),
                Token::EndArray
            ],
            tokens.unwrap()
        );
    }

    proptest! {
        #[test]
        fn string_prop_test(input in ".*") {
            let json: String = serde_json::to_string(&input).unwrap();

            let mut iter = json_token_iter(json.as_bytes());
            assert_eq!(Some(Ok(Token::ValueString(EscapedStr(&json[1..(json.len()-1)])))), iter.next());
            assert_eq!(None, iter.next());
        }

        #[test]
        fn integer_prop_test(input: i64) {
            let json = serde_json::to_string(&input).unwrap();
            let mut iter = json_token_iter(json.as_bytes());
            assert_eq!(Some(Ok(Token::ValueNumber(
                if input < 0 {
                    Number::NegInt(input)
                } else {
                    Number::PosInt(input as u64)
                }))), iter.next());
            assert_eq!(None, iter.next());
        }

        #[test]
        fn float_prop_test(input: f64) {
            let json = serde_json::to_string(&input).unwrap();
            let mut iter = json_token_iter(json.as_bytes());
            assert_eq!(Some(Ok(Token::ValueNumber(Number::Float(input)))), iter.next());
            assert_eq!(None, iter.next());
        }
    }

    #[test]
    fn valid_numbers() {
        let expect = |number, input| {
            assert_eq!(
                Token::ValueNumber(number),
                json_token_iter(input).next().unwrap().unwrap()
            );
        };
        expect(Number::Float(0.0), b"0.");
        expect(Number::Float(0.0), b"0e0");
        expect(Number::Float(0.0), b"0E0");
        expect(Number::Float(10.0), b"1E1");
        expect(Number::Float(10.0), b"1E+1");
        expect(Number::Float(100.0), b"1e+2");

        expect(Number::NegInt(-50000), b"-50000");
        expect(
            Number::Float(-18446744073709551615.0),
            b"-18446744073709551615",
        );
    }

    // These cases actually shouldn't parse according to the spec, but it's easier
    // to be lenient on these, and it doesn't really impact the SDK use-case.
    #[test]
    fn invalid_numbers_we_are_intentionally_accepting() {
        let expect = |number, input| {
            assert_eq!(
                Token::ValueNumber(number),
                json_token_iter(input).next().unwrap().unwrap()
            );
        };

        expect(Number::NegInt(-1), b"-01");
        expect(Number::Float(-2.0), b"-2.");
        expect(Number::Float(0.0), b"0.e1");
        expect(Number::Float(0.002), b"2.e-3");
        expect(Number::Float(2000.0), b"2.e3");
        expect(Number::NegInt(-12), b"-012");
        expect(Number::Float(-0.123), b"-.123");
        expect(Number::Float(1.0), b"1.");
        expect(Number::PosInt(12), b"012");
    }

    #[test]
    fn invalid_numbers() {
        let unexpected_token = |input, token, offset, msg| {
            let tokens: Vec<Result<Token, Error>> = json_token_iter(input).collect();
            assert_eq!(
                vec![Err(Error {
                    reason: ErrorReason::UnexpectedToken(token, msg),
                    offset
                }),],
                tokens,
                "input: \"{}\"",
                std::str::from_utf8(input).unwrap(),
            );
        };

        let invalid_number = |input, offset| {
            let tokens: Vec<Result<Token, Error>> = json_token_iter(input).collect();
            assert_eq!(
                vec![Err(Error {
                    reason: ErrorReason::InvalidNumber,
                    offset
                })],
                tokens,
                "input: \"{}\"",
                std::str::from_utf8(input).unwrap(),
            );
        };

        let unexpected_trailer = "<whitespace>, '}', ']', ','";
        let unexpected_start = "'{', '[', '\"', 'null', 'true', 'false', <number>";

        unexpected_token(b".", '.', 0, unexpected_start);
        unexpected_token(b".0", '.', 0, unexpected_start);
        unexpected_token(b"0-05", '-', 1, unexpected_trailer);
        unexpected_token(b"0x05", 'x', 1, unexpected_trailer);
        unexpected_token(b"123.invalid", 'i', 4, unexpected_trailer);
        unexpected_token(b"123invalid", 'i', 3, unexpected_trailer);
        unexpected_token(b"asdf", 'a', 0, unexpected_start);

        invalid_number(b"-a", 0);
        invalid_number(b"1e", 0);
        invalid_number(b"1e-", 0);

        // Number parsing fails before it even looks at the trailer because of invalid exponent
        invalid_number(b"123.0Einvalid", 0);
    }

    #[test]
    fn test_unclosed_array() {
        let mut iter = json_token_iter(br#" [null "#);
        assert_eq!(Some(Ok(Token::StartArray)), iter.next());
        assert_eq!(Some(Ok(Token::ValueNull)), iter.next());
        assert_eq!(
            Some(Err(Error {
                reason: ErrorReason::UnexpectedEOS,
                offset: 7
            })),
            iter.next()
        );
    }

    #[test]
    fn test_array_with_items() {
        let tokens: Result<Vec<Token>, Error> = json_token_iter(b"[[], {}, \"test\"]").collect();
        assert_eq!(
            vec![
                Token::StartArray,
                Token::StartArray,
                Token::EndArray,
                Token::StartObject,
                Token::EndObject,
                Token::ValueString(EscapedStr("test")),
                Token::EndArray,
            ],
            tokens.unwrap()
        )
    }

    #[test]
    fn test_object_with_items() {
        let tokens: Result<Vec<Token>, Error> = json_token_iter(
            br#"
            { "some_int": 5,
              "some_float": 5.2,
              "some_negative": -5,
              "some_negative_float": -2.4,
              "some_string": "test",
              "some_struct": { "nested": "asdf" },
              "some_array": ["one", "two"] }
            "#,
        )
        .collect();
        assert_eq!(
            vec![
                Token::StartObject,
                Token::ObjectKey(EscapedStr("some_int")),
                Token::ValueNumber(Number::PosInt(5)),
                Token::ObjectKey(EscapedStr("some_float")),
                Token::ValueNumber(Number::Float(5.2)),
                Token::ObjectKey(EscapedStr("some_negative")),
                Token::ValueNumber(Number::NegInt(-5)),
                Token::ObjectKey(EscapedStr("some_negative_float")),
                Token::ValueNumber(Number::Float(-2.4)),
                Token::ObjectKey(EscapedStr("some_string")),
                Token::ValueString(EscapedStr("test")),
                Token::ObjectKey(EscapedStr("some_struct")),
                Token::StartObject,
                Token::ObjectKey(EscapedStr("nested")),
                Token::ValueString(EscapedStr("asdf")),
                Token::EndObject,
                Token::ObjectKey(EscapedStr("some_array")),
                Token::StartArray,
                Token::ValueString(EscapedStr("one")),
                Token::ValueString(EscapedStr("two")),
                Token::EndArray,
                Token::EndObject,
            ],
            tokens.unwrap()
        )
    }

    #[test]
    fn test_object_trailing_comma() {
        let mut iter = json_token_iter(br#" { "test": "trailing", } "#);
        assert_eq!(Some(Ok(Token::StartObject)), iter.next());
        assert_eq!(Some(Ok(Token::ObjectKey(EscapedStr("test")))), iter.next());
        assert_eq!(
            Some(Ok(Token::ValueString(EscapedStr("trailing")))),
            iter.next()
        );
        assert_eq!(
            Some(Err(Error {
                reason: ErrorReason::UnexpectedToken('}', "'\"'"),
                offset: 23,
            })),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn test_object_no_colon() {
        let mut iter = json_token_iter(br#" {"test" "#);
        assert_eq!(Some(Ok(Token::StartObject)), iter.next());
        assert_eq!(Some(Ok(Token::ObjectKey(EscapedStr("test")))), iter.next());
        assert_eq!(
            Some(Err(Error {
                reason: ErrorReason::UnexpectedEOS,
                offset: 9,
            })),
            iter.next()
        );
        assert_eq!(None, iter.next());
    }

    #[test]
    fn unescaped_ctrl_characters() {
        assert!(json_token_iter(b"\"test\x00test\"")
            .next()
            .unwrap()
            .is_err());
        assert!(json_token_iter(b"\"test\ntest\"").next().unwrap().is_err());
        assert!(json_token_iter(b"\"test\ttest\"").next().unwrap().is_err());
    }

    #[test]
    fn escaped_str() {
        let escaped = EscapedStr::new("foo\\nbar");
        assert_eq!("foo\\nbar", escaped.as_escaped_str());
        assert_eq!("foo\nbar", escaped.to_unescaped().unwrap());
    }
}
