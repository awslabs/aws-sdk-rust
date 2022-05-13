/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::escape::EscapeError;
use std::borrow::Cow;
use std::fmt;
use std::str::Utf8Error;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorReason {
    Custom(Cow<'static, str>),
    ExpectedLiteral(String),
    InvalidEscape(char),
    InvalidNumber,
    InvalidUtf8,
    UnescapeFailed(EscapeError),
    UnexpectedControlCharacter(u8),
    UnexpectedEos,
    UnexpectedToken(char, &'static str),
}
use ErrorReason::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    reason: ErrorReason,
    offset: Option<usize>,
}

impl Error {
    pub fn new(reason: ErrorReason, offset: Option<usize>) -> Self {
        Error { reason, offset }
    }

    /// Returns a custom error without an offset.
    pub fn custom(message: impl Into<Cow<'static, str>>) -> Error {
        Error::new(ErrorReason::Custom(message.into()), None)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(offset) = self.offset {
            write!(f, "Error at offset {}: ", offset)?;
        }
        match &self.reason {
            Custom(msg) => write!(f, "failed to parse JSON: {}", msg),
            ExpectedLiteral(literal) => write!(f, "expected literal: {}", literal),
            InvalidEscape(escape) => write!(f, "invalid JSON escape: \\{}", escape),
            InvalidNumber => write!(f, "invalid number"),
            InvalidUtf8 => write!(f, "invalid UTF-8 codepoint in JSON stream"),
            UnescapeFailed(err) => write!(f, "failed to unescape JSON string: {}", err),
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
            UnexpectedEos => write!(f, "unexpected end of stream"),
        }
    }
}

impl From<Utf8Error> for ErrorReason {
    fn from(_: Utf8Error) -> Self {
        InvalidUtf8
    }
}

impl From<EscapeError> for Error {
    fn from(err: EscapeError) -> Self {
        Error {
            reason: ErrorReason::UnescapeFailed(err),
            offset: None,
        }
    }
}
