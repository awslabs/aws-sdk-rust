/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Utilities for parsing information from headers

use http::header::{HeaderName, ValueIter};
use http::HeaderValue;
use smithy_types::instant::Format;
use smithy_types::primitive::Parse;
use smithy_types::Instant;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

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
/// This is separate from `read_many` below because we need to invoke `Instant::read` to take advantage
/// of comma-aware parsing
pub fn many_dates(
    values: ValueIter<HeaderValue>,
    format: Format,
) -> Result<Vec<Instant>, ParseError> {
    let mut out = vec![];
    for header in values {
        let mut header = header
            .to_str()
            .map_err(|_| ParseError::new_with_message("header was not valid utf-8 string"))?;
        while !header.is_empty() {
            let (v, next) = Instant::read(header, format, ',').map_err(|err| {
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
            let (v, next) = read_one(&header, &f)?;
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

pub fn set_header_if_absent(
    request: http::request::Builder,
    key: &'static str,
    value: &'static str,
) -> http::request::Builder {
    if !request
        .headers_ref()
        .map(|map| map.contains_key(key))
        .unwrap_or(false)
    {
        request.header(key, value)
    } else {
        request
    }
}

/// Read one comma delimited value for `FromStr` types
fn read_one<'a, T>(
    s: &'a [u8],
    f: &impl Fn(&str) -> Result<T, ParseError>,
) -> Result<(T, &'a [u8]), ParseError> {
    let (head, rest) = split_at_delim(s);
    let head = std::str::from_utf8(head)
        .map_err(|_| ParseError::new_with_message("header was not valid utf8"))?;
    Ok((f(head.trim())?, rest))
}

fn split_at_delim(s: &[u8]) -> (&[u8], &[u8]) {
    let next_delim = s.iter().position(|b| b == &b',').unwrap_or(s.len());
    let (first, next) = s.split_at(next_delim);
    (first, then_delim(next).unwrap())
}

fn then_delim(s: &[u8]) -> Result<&[u8], ParseError> {
    if s.is_empty() {
        Ok(&s)
    } else if s.starts_with(b",") {
        Ok(&s[1..])
    } else {
        Err(ParseError::new_with_message("expected delimiter `,`"))
    }
}

#[cfg(test)]
mod test {
    use crate::header::{
        headers_for_prefix, read_many_primitive, set_header_if_absent, ParseError,
    };
    use std::collections::HashMap;

    #[test]
    fn put_if_absent() {
        let builder = http::Request::builder().header("foo", "bar");
        let builder = set_header_if_absent(builder, "foo", "baz");
        let builder = set_header_if_absent(builder, "other", "value");
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
    fn read_many_bools() {
        let test_request = http::Request::builder()
            .header("X-Bool-Multi", "true,false")
            .header("X-Bool-Multi", "true")
            .header("X-Bool", "true")
            .header("X-Bool-Invalid", "truth,falsy")
            .header("X-Bool-Single", "true,false,true,true")
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
}
