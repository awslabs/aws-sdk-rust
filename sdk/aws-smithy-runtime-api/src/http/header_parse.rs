/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Low-level parsing of Smithy values out of HTTP header text.
//!
//! <div class="warning">
//!
//! Apart from [`ParseError`], which is re-exported as
//! [`aws_smithy_runtime_api::http::ParseError`](crate::http::ParseError), the items in this
//! module are an implementation detail shared between `aws-smithy-http` and
//! `aws-smithy-schema`. They are **not** a stable API and may change in a minor release.
//! Depend on [`aws_smithy_http::header`] instead.
//!
//! </div>
//!
//! These primitives operate on iterators of raw header bytes (or `&str`) rather than on any
//! HTTP library's types, which is why they can live here without pulling an HTTP crate or
//! `aws-smithy-http`'s dependency tree into schema-based serialization.
//!
//! [`aws_smithy_http::header`]: https://docs.rs/aws-smithy-http/latest/aws_smithy_http/header/index.html

use aws_smithy_types::date_time::Format;
use aws_smithy_types::primitive::Parse;
use aws_smithy_types::DateTime;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::str::FromStr;

/// An error was encountered while parsing a header
#[derive(Debug)]
pub struct ParseError {
    message: Cow<'static, str>,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

impl ParseError {
    /// Create a new parse error with the given `message`
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// Attach a source to this error.
    pub fn with_source(self, source: impl Into<Box<dyn Error + Send + Sync + 'static>>) -> Self {
        Self {
            source: Some(source.into()),
            ..self
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "output failed to parse in headers: {}", self.message)
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|err| err.as_ref() as _)
    }
}

pub(crate) const NON_UTF8_HEADER: &str = "header was not valid utf-8";

/// Interpret raw header bytes as UTF-8, or fail with a [`ParseError`].
fn str_from_utf8(bytes: &[u8]) -> Result<&str, ParseError> {
    std::str::from_utf8(bytes).map_err(|_| ParseError::new(NON_UTF8_HEADER))
}

/// Read all the dates from the header map at `key` according the `format`
///
/// This is separate from `read_many_bytes` below because we need to invoke `DateTime::read` to take
/// advantage of comma-aware parsing
pub fn many_dates<'a>(
    values: impl Iterator<Item = &'a str>,
    format: Format,
) -> Result<Vec<DateTime>, ParseError> {
    many_dates_bytes(values.map(str::as_bytes), format)
}

/// Read all the dates from raw header values according to the `format`
///
/// Like [`many_dates`], but accepts raw bytes. A value that is not valid UTF-8 produces a
/// [`ParseError`]: every Smithy protocol encodes timestamps as ASCII, so such a value is
/// always malformed.
pub fn many_dates_bytes<'a>(
    values: impl Iterator<Item = &'a [u8]>,
    format: Format,
) -> Result<Vec<DateTime>, ParseError> {
    let mut out = vec![];
    for header in values {
        let mut header = str_from_utf8(header)?;
        while !header.is_empty() {
            let (v, next) = DateTime::read(header, format, ',').map_err(|err| {
                ParseError::new(format!("header could not be parsed as date: {err}"))
            })?;
            out.push(v);
            header = next;
        }
    }
    Ok(out)
}

/// Returns an iterator over pairs where the first element is the unprefixed header name that
/// starts with the input `key` prefix, and the second element is the full header name.
///
/// `header_names` must be normalized (lowercase) names, as HTTP libraries produce them. A name
/// matches when it starts with `key` lowercased, so a modeled prefix such as `X-Meta-` matches
/// the wire name `x-meta-foo`.
///
/// This is done without allocating a lowercased copy of `key`, because it runs per prefix-bound
/// member on every response. A prefix with no uppercase letters is already its own lowercase
/// form, so a plain byte comparison is the whole answer; only an uppercase prefix pays for
/// lowercasing each byte as it compares.
pub fn headers_for_prefix<'a>(
    header_names: impl Iterator<Item = &'a str>,
    key: &'a str,
) -> impl Iterator<Item = (&'a str, &'a str)> {
    let prefix = key.as_bytes();
    let has_uppercase = prefix.iter().any(u8::is_ascii_uppercase);
    header_names
        .filter(move |k| {
            let Some(head) = k.as_bytes().get(..prefix.len()) else {
                return false;
            };
            if has_uppercase {
                head.iter()
                    .zip(prefix)
                    .all(|(h, p)| *h == p.to_ascii_lowercase())
            } else {
                head == prefix
            }
        })
        // Splitting at `key.len()` is a character boundary: those bytes just compared equal
        // (ignoring ASCII case) to `key`'s bytes, so they agree with `key`'s boundaries.
        .map(move |k| (&k[key.len()..], k))
}

/// Convert a `HeaderValue` into a `Vec<T>` where `T: FromStr`
pub fn read_many_from_str<'a, T: FromStr>(
    values: impl Iterator<Item = &'a str>,
) -> Result<Vec<T>, ParseError>
where
    T::Err: Error + Send + Sync + 'static,
{
    read_many_from_str_bytes(values.map(str::as_bytes))
}

/// Convert raw header values into a `Vec<T>` where `T: FromStr`
///
/// Like [`read_many_from_str`], but accepts raw bytes. A value that is not valid UTF-8
/// produces a [`ParseError`].
pub fn read_many_from_str_bytes<'a, T: FromStr>(
    values: impl Iterator<Item = &'a [u8]>,
) -> Result<Vec<T>, ParseError>
where
    T::Err: Error + Send + Sync + 'static,
{
    read_many_bytes(values, |v: &str| {
        v.parse().map_err(|err| {
            ParseError::new("failed during `FromString` conversion").with_source(err)
        })
    })
}

/// Convert a `HeaderValue` into a `Vec<T>` where `T: Parse`
pub fn read_many_primitive<'a, T: Parse>(
    values: impl Iterator<Item = &'a str>,
) -> Result<Vec<T>, ParseError> {
    read_many_primitive_bytes(values.map(str::as_bytes))
}

/// Convert raw header values into a `Vec<T>` where `T: Parse`
///
/// Like [`read_many_primitive`], but accepts raw bytes. A value that is not valid UTF-8
/// produces a [`ParseError`]: Smithy primitives are ASCII, so such a value is always
/// malformed.
pub fn read_many_primitive_bytes<'a, T: Parse>(
    values: impl Iterator<Item = &'a [u8]>,
) -> Result<Vec<T>, ParseError> {
    read_many_bytes(values, |v: &str| {
        T::parse_smithy_primitive(v)
            .map_err(|err| ParseError::new("failed reading a list of primitives").with_source(err))
    })
}

/// Read many comma / header delimited values from raw HTTP header bytes
fn read_many_bytes<'a, T>(
    values: impl Iterator<Item = &'a [u8]>,
    f: impl Fn(&str) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    let mut out = vec![];
    for header in values {
        let mut header = header;
        while !header.is_empty() {
            let (v, next) = read_one(header, &f)?;
            out.push(v);
            header = next;
        }
    }
    Ok(out)
}

/// Read comma / header delimited values, keeping only the first, and require that the
/// values held at most one item.
///
/// This is the cardinality rule a scalar (non-list) member bound to a header follows. It
/// exists separately from [`read_many_bytes`] so a scalar read does not allocate a `Vec`
/// that is immediately discarded.
///
/// Every token is still parsed, so a malformed later value reports its own parse error
/// rather than being masked by the cardinality error — matching the behavior of parsing
/// into a `Vec` and then checking its length.
fn one_of_many_bytes<'a, T>(
    values: impl Iterator<Item = &'a [u8]>,
    f: impl Fn(&str) -> Result<T, ParseError>,
) -> Result<Option<T>, ParseError> {
    let mut first = None;
    let mut count = 0usize;
    for header in values {
        let mut header = header;
        while !header.is_empty() {
            let (value, next) = read_one(header, &f)?;
            count += 1;
            if first.is_none() {
                first = Some(value);
            }
            header = next;
        }
    }
    if count > 1 {
        return Err(ParseError::new(format!(
            "expected one item but found {count}"
        )));
    }
    Ok(first)
}

/// Read a single comma-delimited primitive from raw header values
///
/// This is the scalar counterpart to [`read_many_primitive_bytes`]: it applies the same
/// tokenizing and parsing but returns at most one value and fails if the header carried
/// more than one. `Ok(None)` means the values produced no tokens at all, which the
/// generated legacy path reports as an absent member.
pub fn one_primitive_or_none_bytes<'a, T: Parse>(
    values: impl Iterator<Item = &'a [u8]>,
) -> Result<Option<T>, ParseError> {
    one_of_many_bytes(values, |v: &str| {
        T::parse_smithy_primitive(v)
            .map_err(|err| ParseError::new("failed reading a list of primitives").with_source(err))
    })
}

/// Read a single comma-delimited `FromStr` value from raw header values
///
/// This is the scalar counterpart to [`read_many_from_str_bytes`]. See
/// [`one_primitive_or_none_bytes`] for the cardinality and `Ok(None)` semantics.
pub fn one_from_str_or_none_bytes<'a, T: FromStr>(
    values: impl Iterator<Item = &'a [u8]>,
) -> Result<Option<T>, ParseError>
where
    T::Err: Error + Send + Sync + 'static,
{
    one_of_many_bytes(values, |v: &str| {
        v.parse().map_err(|err| {
            ParseError::new("failed during `FromString` conversion").with_source(err)
        })
    })
}

/// Read a single date from raw header values according to `format`
///
/// This is the scalar counterpart to [`many_dates_bytes`], and like it uses
/// [`DateTime::read`] so an HTTP-date containing a comma is not mistaken for two values.
/// See [`one_primitive_or_none_bytes`] for the cardinality and `Ok(None)` semantics.
pub fn one_date_or_none_bytes<'a>(
    values: impl Iterator<Item = &'a [u8]>,
    format: Format,
) -> Result<Option<DateTime>, ParseError> {
    let mut first = None;
    let mut count = 0usize;
    for header in values {
        let mut header = str_from_utf8(header)?;
        while !header.is_empty() {
            let (value, next) = DateTime::read(header, format, ',').map_err(|err| {
                ParseError::new(format!("header could not be parsed as date: {err}"))
            })?;
            count += 1;
            if first.is_none() {
                first = Some(value);
            }
            header = next;
        }
    }
    if count > 1 {
        return Err(ParseError::new(format!(
            "expected one item but found {count}"
        )));
    }
    Ok(first)
}

/// Conditionally quotes and escapes a header value if the header value contains a comma or quote.
pub fn quote_header_value<'a>(value: impl Into<Cow<'a, str>>) -> Cow<'a, str> {
    let value = value.into();
    if value.trim().len() != value.len()
        || value.contains('"')
        || value.contains(',')
        || value.contains('(')
        || value.contains(')')
    {
        Cow::Owned(format!(
            "\"{}\"",
            value.replace('\\', "\\\\").replace('"', "\\\"")
        ))
    } else {
        value
    }
}

/// Read exactly one or none from a headers iterator
///
/// This function does not perform comma splitting like [`read_many_from_str`]
pub fn one_or_none<'a, T: FromStr>(
    values: impl Iterator<Item = &'a str>,
) -> Result<Option<T>, ParseError>
where
    T::Err: Error + Send + Sync + 'static,
{
    one_or_none_bytes(values.map(str::as_bytes))
}

/// Read exactly one or none from a raw header bytes iterator
///
/// Like [`one_or_none`], but accepts raw bytes. A value that is not valid UTF-8 produces a
/// [`ParseError`].
///
/// This function does not perform comma splitting like [`read_many_from_str_bytes`].
pub fn one_or_none_bytes<'a, T: FromStr>(
    mut values: impl Iterator<Item = &'a [u8]>,
) -> Result<Option<T>, ParseError>
where
    T::Err: Error + Send + Sync + 'static,
{
    let first = match values.next() {
        Some(v) => v,
        None => return Ok(None),
    };
    match values.next() {
        // Checked before the UTF-8 conversion so that the "multiple values" error keeps
        // precedence, matching `one_or_none`.
        None => T::from_str(str_from_utf8(first)?.trim())
            .map_err(|err| ParseError::new("failed to parse string").with_source(err))
            .map(Some),
        Some(_) => Err(ParseError::new(
            "expected a single value but found multiple",
        )),
    }
}

/// Functions for parsing multiple comma-delimited header values out of a
/// single header. This parsing adheres to
/// [RFC-7230's specification of header values](https://datatracker.ietf.org/doc/html/rfc7230#section-3.2.6).
mod parse_multi_header {
    use super::ParseError;
    use std::borrow::Cow;

    fn trim(s: Cow<'_, str>) -> Cow<'_, str> {
        match s {
            Cow::Owned(s) => Cow::Owned(s.trim().into()),
            Cow::Borrowed(s) => Cow::Borrowed(s.trim()),
        }
    }

    fn replace<'a>(value: Cow<'a, str>, pattern: &str, replacement: &str) -> Cow<'a, str> {
        if value.contains(pattern) {
            Cow::Owned(value.replace(pattern, replacement))
        } else {
            value
        }
    }

    /// Reads a single value out of the given input, and returns a tuple containing
    /// the parsed value and the remainder of the slice that can be used to parse
    /// more values.
    pub(crate) fn read_value(input: &[u8]) -> Result<(Cow<'_, str>, &[u8]), ParseError> {
        for (index, &byte) in input.iter().enumerate() {
            let current_slice = &input[index..];
            match byte {
                b' ' | b'\t' => { /* skip whitespace */ }
                b'"' => return read_quoted_value(&current_slice[1..]),
                _ => {
                    let (value, rest) = read_unquoted_value(current_slice)?;
                    return Ok((trim(value), rest));
                }
            }
        }

        // We only end up here if the entire header value was whitespace or empty
        Ok((Cow::Borrowed(""), &[]))
    }

    fn read_unquoted_value(input: &[u8]) -> Result<(Cow<'_, str>, &[u8]), ParseError> {
        let next_delim = input.iter().position(|&b| b == b',').unwrap_or(input.len());
        let (first, next) = input.split_at(next_delim);
        let first =
            std::str::from_utf8(first).map_err(|_| ParseError::new(super::NON_UTF8_HEADER))?;
        Ok((Cow::Borrowed(first), then_comma(next).unwrap()))
    }

    /// Reads a header value that is surrounded by quotation marks and may have escaped
    /// quotes inside of it.
    fn read_quoted_value(input: &[u8]) -> Result<(Cow<'_, str>, &[u8]), ParseError> {
        for index in 0..input.len() {
            match input[index] {
                b'"' if index == 0 || input[index - 1] != b'\\' => {
                    let mut inner = Cow::Borrowed(
                        std::str::from_utf8(&input[0..index])
                            .map_err(|_| ParseError::new(super::NON_UTF8_HEADER))?,
                    );
                    inner = replace(inner, "\\\"", "\"");
                    inner = replace(inner, "\\\\", "\\");
                    let rest = then_comma(&input[(index + 1)..])?;
                    return Ok((inner, rest));
                }
                _ => {}
            }
        }
        Err(ParseError::new(
            "header value had quoted value without end quote",
        ))
    }

    fn then_comma(s: &[u8]) -> Result<&[u8], ParseError> {
        if s.is_empty() {
            Ok(s)
        } else if s.starts_with(b",") {
            Ok(&s[1..])
        } else {
            Err(ParseError::new("expected delimiter `,`"))
        }
    }
}

/// Read one comma delimited value for `FromStr` types
fn read_one<'a, T>(
    s: &'a [u8],
    f: &impl Fn(&str) -> Result<T, ParseError>,
) -> Result<(T, &'a [u8]), ParseError> {
    let (value, rest) = parse_multi_header::read_value(s)?;
    Ok((f(&value)?, rest))
}

#[cfg(test)]
mod test {
    //! These exercise the parsers directly through iterators. `aws-smithy-http` keeps its own
    //! copy of the equivalent HTTP-typed tests, which additionally proves that the re-exports
    //! from `aws_smithy_http::header` still behave identically.

    use super::{
        headers_for_prefix, many_dates, many_dates_bytes, one_date_or_none_bytes,
        one_from_str_or_none_bytes, one_or_none, one_or_none_bytes, one_primitive_or_none_bytes,
        read_many_from_str, read_many_from_str_bytes, read_many_primitive,
        read_many_primitive_bytes, ParseError,
    };
    use aws_smithy_types::error::display::DisplayErrorContext;
    use aws_smithy_types::{date_time::Format, DateTime};
    use std::collections::HashMap;

    #[test]
    fn parse_floats() {
        assert_eq!(
            read_many_primitive::<f32>(["0.0,Infinity,-Infinity,5555.5"].into_iter())
                .expect("valid"),
            vec![0.0, f32::INFINITY, f32::NEG_INFINITY, 5555.5]
        );
        let message = format!(
            "{}",
            DisplayErrorContext(
                read_many_primitive::<f32>(["notafloat"].into_iter()).expect_err("invalid")
            )
        );
        let expected = "output failed to parse in headers: failed reading a list of primitives: failed to parse input as f32";
        assert!(
            message.starts_with(expected),
            "expected '{message}' to start with '{expected}'"
        );
    }

    #[test]
    fn test_many_dates() {
        assert_eq!(
            many_dates([""].into_iter(), Format::DateTime).expect("valid"),
            Vec::<DateTime>::new()
        );
        assert_eq!(
            many_dates(
                ["Wed, 21 Oct 2015 07:28:00 GMT"].into_iter(),
                Format::HttpDate
            )
            .expect("valid"),
            vec![DateTime::from_secs_and_nanos(1445412480, 0)]
        );
        assert_eq!(
            many_dates(
                ["Wed, 21 Oct 2015 07:28:00 GMT,Thu, 22 Oct 2015 07:28:00 GMT"].into_iter(),
                Format::HttpDate
            )
            .expect("valid"),
            vec![
                DateTime::from_secs_and_nanos(1445412480, 0),
                DateTime::from_secs_and_nanos(1445498880, 0)
            ]
        );
        assert_eq!(
            many_dates(["1234.5678"].into_iter(), Format::EpochSeconds).expect("valid"),
            vec![DateTime::from_secs_and_nanos(1234, 567_800_000)]
        );
        assert_eq!(
            many_dates(["1234.5678,9012.3456"].into_iter(), Format::EpochSeconds).expect("valid"),
            vec![
                DateTime::from_secs_and_nanos(1234, 567_800_000),
                DateTime::from_secs_and_nanos(9012, 345_600_000)
            ]
        );
    }

    // A lone 0xE9 is a valid HTTP header octet (obs-text per RFC 7230) but is not valid UTF-8.
    const NON_UTF8_VALUE: &[u8] = b"value-\xe9";

    #[test]
    fn bytes_helpers_agree_with_str_helpers_on_valid_utf8() {
        assert_eq!(
            one_or_none::<String>(["  foo  "].into_iter()).unwrap(),
            one_or_none_bytes::<String>([b"  foo  ".as_slice()].into_iter()).unwrap(),
        );
        assert_eq!(
            read_many_from_str::<String>(["\"foo,bar\",baz"].into_iter()).unwrap(),
            read_many_from_str_bytes::<String>([b"\"foo,bar\",baz".as_slice()].into_iter())
                .unwrap(),
        );
        assert_eq!(
            read_many_primitive::<i16>(["1,2", "3"].into_iter()).unwrap(),
            read_many_primitive_bytes::<i16>([b"1,2".as_slice(), b"3".as_slice()].into_iter())
                .unwrap(),
        );
        assert_eq!(
            many_dates(
                ["Mon, 16 Dec 2019 23:48:18 GMT"].into_iter(),
                Format::HttpDate
            )
            .unwrap(),
            many_dates_bytes(
                [b"Mon, 16 Dec 2019 23:48:18 GMT".as_slice()].into_iter(),
                Format::HttpDate
            )
            .unwrap(),
        );
    }

    #[test]
    fn bytes_helpers_reject_non_utf8() {
        let expected = "header was not valid utf-8";

        let err = one_or_none_bytes::<String>([NON_UTF8_VALUE].into_iter()).expect_err("non-utf8");
        assert!(err.to_string().contains(expected), "{err}");

        let err =
            read_many_from_str_bytes::<String>([NON_UTF8_VALUE].into_iter()).expect_err("non-utf8");
        assert!(err.to_string().contains(expected), "{err}");

        let err =
            read_many_primitive_bytes::<i16>([NON_UTF8_VALUE].into_iter()).expect_err("non-utf8");
        assert!(err.to_string().contains(expected), "{err}");

        let err =
            many_dates_bytes([NON_UTF8_VALUE].into_iter(), Format::HttpDate).expect_err("non-utf8");
        assert!(err.to_string().contains(expected), "{err}");
    }

    #[test]
    fn one_or_none_bytes_reports_multiple_before_non_utf8() {
        // `one_or_none` checks for multiple values before inspecting the first one; the bytes
        // variant must keep that precedence.
        let err = one_or_none_bytes::<String>([NON_UTF8_VALUE, b"second".as_slice()].into_iter())
            .expect_err("multiple values");
        assert!(
            err.to_string()
                .contains("expected a single value but found multiple"),
            "{err}"
        );
    }

    #[test]
    fn read_many_strings() {
        let read = |value: &str| read_many_from_str::<String>([value].into_iter());
        let read_valid = |value: &str| read(value).expect("valid");
        assert_eq!(read_valid(""), Vec::<String>::new());
        assert_eq!(read_valid("  foo"), vec!["foo"]);
        assert_eq!(read_valid("foo   "), vec!["foo"]);
        assert_eq!(read_valid("\"  foo  \""), vec!["  foo  "]);
        assert_eq!(read_valid("\"foo,bar\",baz"), vec!["foo,bar", "baz"]);
        assert_eq!(read_valid("\"foo,bar\",baz  "), vec!["foo,bar", "baz"]);
        assert_eq!(
            read_valid("\"foo\\\",bar\",\"\\\"asdf\\\"\",baz"),
            vec!["foo\",bar", "\"asdf\"", "baz"]
        );
        assert_eq!(
            read_valid("\"foo\\\",bar\", \"\\\"asdf\\\"\", baz"),
            vec!["foo\",bar", "\"asdf\"", "baz"]
        );
        assert!(read("\"\\\"asdf\\\"\"baz").is_err());
        assert_eq!(read_valid("\"\",baz"), vec!["", "baz"]);
        assert_eq!(
            read_valid("foo, \"(foo\\\\bar)\""),
            vec!["foo", "(foo\\bar)"]
        );
    }

    #[test]
    fn read_many_bools() {
        assert_eq!(
            read_many_primitive::<bool>(["true,false", "true"].into_iter()).expect("valid"),
            vec![true, false, true]
        );
        assert_eq!(
            read_many_primitive::<bool>(["true"].into_iter()).unwrap(),
            vec![true]
        );
        assert_eq!(
            read_many_primitive::<bool>(["true,false,true,true"].into_iter()).unwrap(),
            vec![true, false, true, true]
        );
        assert_eq!(
            read_many_primitive::<bool>(["true,\"false\",true,true"].into_iter()).unwrap(),
            vec![true, false, true, true]
        );
        read_many_primitive::<bool>(["truth,falsy"].into_iter()).expect_err("invalid");
    }

    #[test]
    fn check_read_many_i16() {
        assert_eq!(
            read_many_primitive::<i16>(["123,456", "789"].into_iter()).expect("valid"),
            vec![123, 456, 789]
        );
        assert_eq!(
            read_many_primitive::<i16>(["777"].into_iter()).unwrap(),
            vec![777]
        );
        assert_eq!(
            read_many_primitive::<i16>(["1,2,3,-4,5"].into_iter()).unwrap(),
            vec![1, 2, 3, -4, 5]
        );
        assert_eq!(
            read_many_primitive::<i16>(["1, \"2\",3,\"-4\",5"].into_iter()).unwrap(),
            vec![1, 2, 3, -4, 5]
        );
        read_many_primitive::<i16>(["12ef3"].into_iter()).expect_err("invalid");
    }

    #[test]
    fn test_prefix_headers() {
        // Normalized (lowercased) wire names, as `Headers::iter` yields them, matched against a
        // mixed-case modeled prefix.
        let names = ["x-prefix-a", "x-prefix-b", "x-prefix-c", "other"];
        let values: HashMap<&str, Vec<&str>> = [
            ("x-prefix-a", vec!["123,456"]),
            ("x-prefix-b", vec!["789"]),
            ("x-prefix-c", vec!["777", "777"]),
            ("other", vec!["1"]),
        ]
        .into_iter()
        .collect();
        let resp: Result<HashMap<String, Vec<i16>>, ParseError> =
            headers_for_prefix(names.into_iter(), "x-prefix-")
                .map(|(key, header_name)| {
                    read_many_primitive(values[header_name].iter().copied())
                        .map(|v| (key.to_string(), v))
                })
                .collect();
        let resp = resp.expect("valid");
        assert_eq!(resp.get("a"), Some(&vec![123_i16, 456_i16]));
        assert_eq!(resp.get("b"), Some(&vec![789_i16]));
        assert_eq!(resp.get("c"), Some(&vec![777_i16, 777_i16]));
        assert_eq!(resp.get("other"), None);
    }

    #[test]
    fn prefix_matching_lowercases_the_modeled_prefix() {
        let names = ["x-meta-foo", "x-meta-bar"];
        let matched: Vec<_> = headers_for_prefix(names.into_iter(), "X-Meta-").collect();
        assert_eq!(
            matched,
            vec![("foo", "x-meta-foo"), ("bar", "x-meta-bar")],
            "a mixed-case modeled prefix must match normalized wire names"
        );
    }

    #[test]
    fn every_spelling_of_a_prefix_matches_the_same_normalized_names() {
        // Names arrive normalized to lowercase, so the prefix is matched as if lowercased —
        // the behavior of the original `to_ascii_lowercase` + `starts_with`, without the
        // allocation. A shorter name must not panic on the split.
        for prefix in ["X-Meta-", "x-meta-", "X-META-", "x-Meta-"] {
            let names = ["x-meta-foo", "x-meta-bar", "x-other", "x-me", "x-metadata"];
            let matched: Vec<_> = headers_for_prefix(names.into_iter(), prefix).collect();
            assert_eq!(
                matched,
                vec![("foo", "x-meta-foo"), ("bar", "x-meta-bar")],
                "prefix {prefix:?} must match the same names"
            );
        }
    }

    #[test]
    fn a_prefix_with_no_letters_matches_exactly() {
        let names = ["1-2-a", "1-2-", "1-3-a"];
        let matched: Vec<_> = headers_for_prefix(names.into_iter(), "1-2-").collect();
        assert_eq!(matched, vec![("a", "1-2-a"), ("", "1-2-")]);
    }

    #[test]
    fn a_prefix_matches_the_whole_name_and_yields_an_empty_key() {
        let names = ["x-meta-"];
        let matched: Vec<_> = headers_for_prefix(names.into_iter(), "x-meta-").collect();
        assert_eq!(matched, vec![("", "x-meta-")]);
    }

    /// The scalar helpers must agree with "parse into a `Vec`, then require `len() <= 1`",
    /// which is what the generated legacy header path does. They exist only to avoid the
    /// discarded `Vec`, not to change any outcome.
    #[test]
    fn scalar_helpers_agree_with_the_many_value_helpers() {
        let cases: &[&[&[u8]]] = &[
            &[],
            &[b""],
            &[b"", b""],
            &[b"5"],
            &[b" 5 "],
            &[b"5", b""],
            &[b"5,6"],
            &[b"5", b"6"],
            &[b"notanint"],
            &[b"5,notanint"],
        ];
        for values in cases {
            let many = read_many_primitive_bytes::<i32>(values.iter().copied());
            let one = one_primitive_or_none_bytes::<i32>(values.iter().copied());
            match many {
                Ok(mut parsed) if parsed.len() <= 1 => assert_eq!(
                    one.as_ref().ok().and_then(|v| v.as_ref()),
                    parsed.pop().as_ref(),
                    "values {values:?} must produce the same single value"
                ),
                Ok(parsed) => {
                    let message = format!(
                        "{}",
                        DisplayErrorContext(&one.expect_err("cardinality must fail"))
                    );
                    assert!(
                        message.contains(&format!("expected one item but found {}", parsed.len())),
                        "values {values:?} must report the legacy cardinality message, got '{message}'"
                    );
                }
                // A parse error must stay a parse error and keep precedence over cardinality.
                Err(expected) => {
                    let expected = format!("{}", DisplayErrorContext(&expected));
                    let actual = format!(
                        "{}",
                        DisplayErrorContext(&one.expect_err("parse error must propagate"))
                    );
                    assert_eq!(
                        expected, actual,
                        "values {values:?} must report the same error"
                    );
                }
            }
        }
    }

    #[test]
    fn scalar_from_str_helper_agrees_with_the_many_value_helper() {
        assert_eq!(
            one_from_str_or_none_bytes::<String>([b"a".as_slice()].into_iter()).expect("valid"),
            Some("a".to_string())
        );
        // Unlike `one_or_none_bytes`, this path splits on commas, so a comma-bearing value
        // is two items and therefore a cardinality error.
        let err = one_from_str_or_none_bytes::<String>([b"a,b".as_slice()].into_iter())
            .expect_err("two items");
        assert!(
            format!("{}", DisplayErrorContext(&err)).contains("expected one item but found 2"),
            "expected a cardinality error"
        );
        assert_eq!(
            one_from_str_or_none_bytes::<String>([b"".as_slice()].into_iter()).expect("valid"),
            None,
            "an empty value produces no tokens, which the legacy path reports as absent"
        );
    }

    #[test]
    fn scalar_date_helper_agrees_with_many_dates() {
        // The comma inside an HTTP-date must not be read as a value delimiter; this is why
        // dates need their own helper rather than the comma tokenizer.
        let http_date = b"Wed, 21 Oct 2015 07:28:00 GMT".as_slice();
        assert_eq!(
            one_date_or_none_bytes([http_date].into_iter(), Format::HttpDate).expect("valid"),
            Some(many_dates_bytes([http_date].into_iter(), Format::HttpDate).expect("valid")[0])
        );
        assert_eq!(
            one_date_or_none_bytes([b"".as_slice()].into_iter(), Format::HttpDate).expect("valid"),
            None
        );
        let two = b"Wed, 21 Oct 2015 07:28:00 GMT,Wed, 21 Oct 2015 07:28:01 GMT".as_slice();
        assert_eq!(
            many_dates_bytes([two].into_iter(), Format::HttpDate)
                .expect("valid")
                .len(),
            2,
            "precondition: this value really does hold two dates"
        );
        let err =
            one_date_or_none_bytes([two].into_iter(), Format::HttpDate).expect_err("two items");
        assert!(
            format!("{}", DisplayErrorContext(&err)).contains("expected one item but found 2"),
            "expected a cardinality error"
        );
    }

    #[test]
    fn scalar_helpers_reject_non_utf8() {
        let non_utf8 = b"value-\xe9".as_slice();
        for message in [
            format!(
                "{}",
                DisplayErrorContext(
                    &one_primitive_or_none_bytes::<i32>([non_utf8].into_iter())
                        .expect_err("not utf-8")
                )
            ),
            format!(
                "{}",
                DisplayErrorContext(
                    &one_from_str_or_none_bytes::<String>([non_utf8].into_iter())
                        .expect_err("not utf-8")
                )
            ),
            format!(
                "{}",
                DisplayErrorContext(
                    &one_date_or_none_bytes([non_utf8].into_iter(), Format::HttpDate)
                        .expect_err("not utf-8")
                )
            ),
        ] {
            assert!(
                message.contains(super::NON_UTF8_HEADER),
                "expected '{message}' to report the non-UTF-8 reason"
            );
        }
    }
}
