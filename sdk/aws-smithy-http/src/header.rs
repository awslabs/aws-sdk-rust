/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Utilities for parsing information from headers

use std::borrow::Cow;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use http::header::{HeaderName, ValueIter};
use http::HeaderValue;

use aws_smithy_types::date_time::Format;
use aws_smithy_types::primitive::Parse;
use aws_smithy_types::DateTime;

#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ParseError {
    message: Option<Cow<'static, str>>,
}

impl ParseError {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { message: None }
    }

    pub fn new_with_message(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: Some(message.into()),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Output failed to parse in headers")?;
        if let Some(message) = &self.message {
            write!(f, ". {}", message)?;
        }
        Ok(())
    }
}

impl Error for ParseError {}

/// Read all the dates from the header map at `key` according the `format`
///
/// This is separate from `read_many` below because we need to invoke `DateTime::read` to take advantage
/// of comma-aware parsing
pub fn many_dates(
    values: ValueIter<HeaderValue>,
    format: Format,
) -> Result<Vec<DateTime>, ParseError> {
    let mut out = vec![];
    for header in values {
        let mut header = header
            .to_str()
            .map_err(|_| ParseError::new_with_message("header was not valid utf-8 string"))?;
        while !header.is_empty() {
            let (v, next) = DateTime::read(header, format, ',').map_err(|err| {
                ParseError::new_with_message(format!("header could not be parsed as date: {}", err))
            })?;
            out.push(v);
            header = next;
        }
    }
    Ok(out)
}

pub fn headers_for_prefix<'a>(
    headers: &'a http::HeaderMap,
    key: &'a str,
) -> impl Iterator<Item = (&'a str, &'a HeaderName)> {
    let lower_key = key.to_ascii_lowercase();
    headers
        .keys()
        .filter(move |k| k.as_str().starts_with(&lower_key))
        .map(move |h| (&h.as_str()[key.len()..], h))
}

pub fn read_many_from_str<T: FromStr>(
    values: ValueIter<HeaderValue>,
) -> Result<Vec<T>, ParseError> {
    read_many(values, |v: &str| {
        v.parse()
            .map_err(|_err| ParseError::new_with_message("failed during FromString conversion"))
    })
}

pub fn read_many_primitive<T: Parse>(values: ValueIter<HeaderValue>) -> Result<Vec<T>, ParseError> {
    read_many(values, |v: &str| {
        T::parse_smithy_primitive(v).map_err(|primitive| {
            ParseError::new_with_message(format!(
                "failed reading a list of primitives: {}",
                primitive
            ))
        })
    })
}

/// Read many comma / header delimited values from HTTP headers for `FromStr` types
fn read_many<T>(
    values: ValueIter<HeaderValue>,
    f: impl Fn(&str) -> Result<T, ParseError>,
) -> Result<Vec<T>, ParseError> {
    let mut out = vec![];
    for header in values {
        let mut header = header.as_bytes();
        while !header.is_empty() {
            let (v, next) = read_one(header, &f)?;
            out.push(v);
            header = next;
        }
    }
    Ok(out)
}

/// Read exactly one or none from a headers iterator
///
/// This function does not perform comma splitting like `read_many`
pub fn one_or_none<T: FromStr>(
    mut values: ValueIter<HeaderValue>,
) -> Result<Option<T>, ParseError> {
    let first = match values.next() {
        Some(v) => v,
        None => return Ok(None),
    };
    let value = std::str::from_utf8(first.as_bytes())
        .map_err(|_| ParseError::new_with_message("invalid utf-8"))?;
    match values.next() {
        None => T::from_str(value.trim())
            .map_err(|_| ParseError::new())
            .map(Some),
        Some(_) => Err(ParseError::new_with_message(
            "expected a single value but found multiple",
        )),
    }
}

pub fn set_request_header_if_absent<V>(
    request: http::request::Builder,
    key: HeaderName,
    value: V,
) -> http::request::Builder
where
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
{
    if !request
        .headers_ref()
        .map(|map| map.contains_key(&key))
        .unwrap_or(false)
    {
        request.header(key, value)
    } else {
        request
    }
}

pub fn set_response_header_if_absent<V>(
    response: http::response::Builder,
    key: HeaderName,
    value: V,
) -> http::response::Builder
where
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
{
    if !response
        .headers_ref()
        .map(|map| map.contains_key(&key))
        .unwrap_or(false)
    {
        response.header(key, value)
    } else {
        response
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
        let first = std::str::from_utf8(first)
            .map_err(|_| ParseError::new_with_message("header was not valid utf8"))?;
        Ok((Cow::Borrowed(first), then_comma(next).unwrap()))
    }

    /// Reads a header value that is surrounded by quotation marks and may have escaped
    /// quotes inside of it.
    fn read_quoted_value(input: &[u8]) -> Result<(Cow<'_, str>, &[u8]), ParseError> {
        for index in 0..input.len() {
            match input[index] {
                b'"' if index == 0 || input[index - 1] != b'\\' => {
                    let mut inner =
                        Cow::Borrowed(std::str::from_utf8(&input[0..index]).map_err(|_| {
                            ParseError::new_with_message("header was not valid utf8")
                        })?);
                    inner = replace(inner, "\\\"", "\"");
                    inner = replace(inner, "\\\\", "\\");
                    let rest = then_comma(&input[(index + 1)..])?;
                    return Ok((inner, rest));
                }
                _ => {}
            }
        }
        Err(ParseError::new_with_message(
            "header value had quoted value without end quote",
        ))
    }

    fn then_comma(s: &[u8]) -> Result<&[u8], ParseError> {
        if s.is_empty() {
            Ok(s)
        } else if s.starts_with(b",") {
            Ok(&s[1..])
        } else {
            Err(ParseError::new_with_message("expected delimiter `,`"))
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

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use aws_smithy_types::{date_time::Format, DateTime};
    use http::header::HeaderName;

    use crate::header::{
        headers_for_prefix, many_dates, read_many_from_str, read_many_primitive,
        set_request_header_if_absent, set_response_header_if_absent, ParseError,
    };

    use super::quote_header_value;

    #[test]
    fn put_on_request_if_absent() {
        let builder = http::Request::builder().header("foo", "bar");
        let builder = set_request_header_if_absent(builder, HeaderName::from_static("foo"), "baz");
        let builder =
            set_request_header_if_absent(builder, HeaderName::from_static("other"), "value");
        let req = builder.body(()).expect("valid request");
        assert_eq!(
            req.headers().get_all("foo").iter().collect::<Vec<_>>(),
            vec!["bar"]
        );
        assert_eq!(
            req.headers().get_all("other").iter().collect::<Vec<_>>(),
            vec!["value"]
        );
    }

    #[test]
    fn put_on_response_if_absent() {
        let builder = http::Response::builder().header("foo", "bar");
        let builder = set_response_header_if_absent(builder, HeaderName::from_static("foo"), "baz");
        let builder =
            set_response_header_if_absent(builder, HeaderName::from_static("other"), "value");
        let response = builder.body(()).expect("valid response");
        assert_eq!(
            response.headers().get_all("foo").iter().collect::<Vec<_>>(),
            vec!["bar"]
        );
        assert_eq!(
            response
                .headers()
                .get_all("other")
                .iter()
                .collect::<Vec<_>>(),
            vec!["value"]
        );
    }

    #[test]
    fn parse_floats() {
        let test_request = http::Request::builder()
            .header("X-Float-Multi", "0.0,Infinity,-Infinity,5555.5")
            .header("X-Float-Error", "notafloat")
            .body(())
            .unwrap();
        assert_eq!(
            read_many_primitive::<f32>(test_request.headers().get_all("X-Float-Multi").iter())
                .expect("valid"),
            vec![0.0, f32::INFINITY, f32::NEG_INFINITY, 5555.5]
        );
        assert_eq!(
            read_many_primitive::<f32>(test_request.headers().get_all("X-Float-Error").iter())
                .expect_err("invalid"),
            ParseError::new_with_message(
                "failed reading a list of primitives: failed to parse input as f32"
            )
        )
    }

    #[test]
    fn test_many_dates() {
        let test_request = http::Request::builder()
            .header("Empty", "")
            .header("SingleHttpDate", "Wed, 21 Oct 2015 07:28:00 GMT")
            .header(
                "MultipleHttpDates",
                "Wed, 21 Oct 2015 07:28:00 GMT,Thu, 22 Oct 2015 07:28:00 GMT",
            )
            .header("SingleEpochSeconds", "1234.5678")
            .header("MultipleEpochSeconds", "1234.5678,9012.3456")
            .body(())
            .unwrap();
        let read = |name: &str, format: Format| {
            many_dates(test_request.headers().get_all(name).iter(), format)
        };
        let read_valid = |name: &str, format: Format| read(name, format).expect("valid");
        assert_eq!(
            read_valid("Empty", Format::DateTime),
            Vec::<DateTime>::new()
        );
        assert_eq!(
            read_valid("SingleHttpDate", Format::HttpDate),
            vec![DateTime::from_secs_and_nanos(1445412480, 0)]
        );
        assert_eq!(
            read_valid("MultipleHttpDates", Format::HttpDate),
            vec![
                DateTime::from_secs_and_nanos(1445412480, 0),
                DateTime::from_secs_and_nanos(1445498880, 0)
            ]
        );
        assert_eq!(
            read_valid("SingleEpochSeconds", Format::EpochSeconds),
            vec![DateTime::from_secs_and_nanos(1234, 567_800_000)]
        );
        assert_eq!(
            read_valid("MultipleEpochSeconds", Format::EpochSeconds),
            vec![
                DateTime::from_secs_and_nanos(1234, 567_800_000),
                DateTime::from_secs_and_nanos(9012, 345_600_000)
            ]
        );
    }

    #[test]
    fn read_many_strings() {
        let test_request = http::Request::builder()
            .header("Empty", "")
            .header("Foo", "  foo")
            .header("FooTrailing", "foo   ")
            .header("FooInQuotes", "\"  foo  \"")
            .header("CommaInQuotes", "\"foo,bar\",baz")
            .header("CommaInQuotesTrailing", "\"foo,bar\",baz  ")
            .header("QuoteInQuotes", "\"foo\\\",bar\",\"\\\"asdf\\\"\",baz")
            .header(
                "QuoteInQuotesWithSpaces",
                "\"foo\\\",bar\", \"\\\"asdf\\\"\", baz",
            )
            .header("JunkFollowingQuotes", "\"\\\"asdf\\\"\"baz")
            .header("EmptyQuotes", "\"\",baz")
            .header("EscapedSlashesInQuotes", "foo, \"(foo\\\\bar)\"")
            .body(())
            .unwrap();
        let read =
            |name: &str| read_many_from_str::<String>(test_request.headers().get_all(name).iter());
        let read_valid = |name: &str| read(name).expect("valid");
        assert_eq!(read_valid("Empty"), Vec::<String>::new());
        assert_eq!(read_valid("Foo"), vec!["foo"]);
        assert_eq!(read_valid("FooTrailing"), vec!["foo"]);
        assert_eq!(read_valid("FooInQuotes"), vec!["  foo  "]);
        assert_eq!(read_valid("CommaInQuotes"), vec!["foo,bar", "baz"]);
        assert_eq!(read_valid("CommaInQuotesTrailing"), vec!["foo,bar", "baz"]);
        assert_eq!(
            read_valid("QuoteInQuotes"),
            vec!["foo\",bar", "\"asdf\"", "baz"]
        );
        assert_eq!(
            read_valid("QuoteInQuotesWithSpaces"),
            vec!["foo\",bar", "\"asdf\"", "baz"]
        );
        assert!(read("JunkFollowingQuotes").is_err());
        assert_eq!(read_valid("EmptyQuotes"), vec!["", "baz"]);
        assert_eq!(
            read_valid("EscapedSlashesInQuotes"),
            vec!["foo", "(foo\\bar)"]
        );
    }

    #[test]
    fn read_many_bools() {
        let test_request = http::Request::builder()
            .header("X-Bool-Multi", "true,false")
            .header("X-Bool-Multi", "true")
            .header("X-Bool", "true")
            .header("X-Bool-Invalid", "truth,falsy")
            .header("X-Bool-Single", "true,false,true,true")
            .header("X-Bool-Quoted", "true,\"false\",true,true")
            .body(())
            .unwrap();
        assert_eq!(
            read_many_primitive::<bool>(test_request.headers().get_all("X-Bool-Multi").iter())
                .expect("valid"),
            vec![true, false, true]
        );

        assert_eq!(
            read_many_primitive::<bool>(test_request.headers().get_all("X-Bool").iter()).unwrap(),
            vec![true]
        );
        assert_eq!(
            read_many_primitive::<bool>(test_request.headers().get_all("X-Bool-Single").iter())
                .unwrap(),
            vec![true, false, true, true]
        );
        assert_eq!(
            read_many_primitive::<bool>(test_request.headers().get_all("X-Bool-Quoted").iter())
                .unwrap(),
            vec![true, false, true, true]
        );
        read_many_primitive::<bool>(test_request.headers().get_all("X-Bool-Invalid").iter())
            .expect_err("invalid");
    }

    #[test]
    fn check_read_many_i16() {
        let test_request = http::Request::builder()
            .header("X-Multi", "123,456")
            .header("X-Multi", "789")
            .header("X-Num", "777")
            .header("X-Num-Invalid", "12ef3")
            .header("X-Num-Single", "1,2,3,-4,5")
            .header("X-Num-Quoted", "1, \"2\",3,\"-4\",5")
            .body(())
            .unwrap();
        assert_eq!(
            read_many_primitive::<i16>(test_request.headers().get_all("X-Multi").iter())
                .expect("valid"),
            vec![123, 456, 789]
        );

        assert_eq!(
            read_many_primitive::<i16>(test_request.headers().get_all("X-Num").iter()).unwrap(),
            vec![777]
        );
        assert_eq!(
            read_many_primitive::<i16>(test_request.headers().get_all("X-Num-Single").iter())
                .unwrap(),
            vec![1, 2, 3, -4, 5]
        );
        assert_eq!(
            read_many_primitive::<i16>(test_request.headers().get_all("X-Num-Quoted").iter())
                .unwrap(),
            vec![1, 2, 3, -4, 5]
        );
        read_many_primitive::<i16>(test_request.headers().get_all("X-Num-Invalid").iter())
            .expect_err("invalid");
    }

    #[test]
    fn test_prefix_headers() {
        let test_request = http::Request::builder()
            .header("X-Prefix-A", "123,456")
            .header("X-Prefix-B", "789")
            .header("X-Prefix-C", "777")
            .header("X-Prefix-C", "777")
            .body(())
            .unwrap();
        let resp: Result<HashMap<String, Vec<i16>>, ParseError> =
            headers_for_prefix(test_request.headers(), "X-Prefix-")
                .map(|(key, header_name)| {
                    let values = test_request.headers().get_all(header_name);
                    read_many_primitive(values.iter()).map(|v| (key.to_string(), v))
                })
                .collect();
        let resp = resp.expect("valid");
        assert_eq!(resp.get("a"), Some(&vec![123_i16, 456_i16]));
    }

    #[test]
    fn test_quote_header_value() {
        assert_eq!("", &quote_header_value(""));
        assert_eq!("foo", &quote_header_value("foo"));
        assert_eq!("\"  foo\"", &quote_header_value("  foo"));
        assert_eq!("foo bar", &quote_header_value("foo bar"));
        assert_eq!("\"foo,bar\"", &quote_header_value("foo,bar"));
        assert_eq!("\",\"", &quote_header_value(","));
        assert_eq!("\"\\\"foo\\\"\"", &quote_header_value("\"foo\""));
        assert_eq!("\"\\\"f\\\\oo\\\"\"", &quote_header_value("\"f\\oo\""));
        assert_eq!("\"(\"", &quote_header_value("("));
        assert_eq!("\")\"", &quote_header_value(")"));
    }
}
