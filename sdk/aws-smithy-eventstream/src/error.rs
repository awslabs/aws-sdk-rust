/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_smithy_types::DateTime;
use std::error::Error as StdError;
use std::fmt;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    HeadersTooLong,
    HeaderValueTooLong,
    InvalidHeaderNameLength,
    InvalidHeaderValue,
    InvalidHeaderValueType(u8),
    InvalidHeadersLength,
    InvalidMessageLength,
    InvalidUtf8String,
    MessageChecksumMismatch(u32, u32),
    MessageTooLong,
    PayloadTooLong,
    PreludeChecksumMismatch(u32, u32),
    TimestampValueTooLarge(DateTime),
    Marshalling(String),
    Unmarshalling(String),
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            HeadersTooLong => write!(f, "headers too long to fit in event stream frame"),
            HeaderValueTooLong => write!(f, "header value too long to fit in event stream frame"),
            InvalidHeaderNameLength => write!(f, "invalid header name length"),
            InvalidHeaderValue => write!(f, "invalid header value"),
            InvalidHeaderValueType(val) => write!(f, "invalid header value type: {}", val),
            InvalidHeadersLength => write!(f, "invalid headers length"),
            InvalidMessageLength => write!(f, "invalid message length"),
            InvalidUtf8String => write!(f, "encountered invalid UTF-8 string"),
            MessageChecksumMismatch(expected, actual) => write!(
                f,
                "message checksum 0x{:X} didn't match expected checksum 0x{:X}",
                actual, expected
            ),
            MessageTooLong => write!(f, "message too long to fit in event stream frame"),
            PayloadTooLong => write!(f, "message payload too long to fit in event stream frame"),
            PreludeChecksumMismatch(expected, actual) => write!(
                f,
                "prelude checksum 0x{:X} didn't match expected checksum 0x{:X}",
                actual, expected
            ),
            TimestampValueTooLarge(time) => write!(
                f,
                "timestamp value {:?} is too large to fit into an i64",
                time
            ),
            Marshalling(error) => write!(f, "failed to marshall message: {}", error),
            Unmarshalling(error) => write!(f, "failed to unmarshall message: {}", error),
        }
    }
}
