/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Utilities for parsing information from headers

use http::header::{HeaderName, ValueIter};
use http::HeaderValue;
use smithy_types::instant::Format;
use smithy_types::Instant;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Output failed to parse in headers")
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
        let mut header = header.to_str().map_err(|_| ParseError)?;
        while !header.is_empty() {
            let (v, next) = Instant::read(header, format, ',').map_err(|_| ParseError)?;
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

/// Read many comma / header delimited values from HTTP headers for `FromStr` types
pub fn read_many<T>(values: ValueIter<HeaderValue>) -> Result<Vec<T>, ParseError>
where
    T: FromStr,
{
    let mut out = vec![];
    for header in values {
        let mut header = header.as_bytes();
        while !header.is_empty() {
            let (v, next) = read_one::<T>(&header)?;
            out.push(v);
            header = next;
        }
    }
    Ok(out)
}

/// Read one comma delimited value for `FromStr` types
pub fn read_one<T>(s: &[u8]) -> Result<(T, &[u8]), ParseError>
where
    T: FromStr,
{
    let (head, rest) = split_at_delim(s);
    let head = std::str::from_utf8(head).map_err(|_| ParseError)?;
    Ok((T::from_str(head.trim()).map_err(|_| ParseError)?, rest))
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
        Err(ParseError)
    }
}

#[cfg(test)]
mod test {
    use crate::header::{headers_for_prefix, read_many, ParseError};
    use std::collections::HashMap;

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
            read_many::<bool>(test_request.headers().get_all("X-Bool-Multi").iter())
                .expect("valid"),
            vec![true, false, true]
        );

        assert_eq!(
            read_many::<bool>(test_request.headers().get_all("X-Bool").iter()).unwrap(),
            vec![true]
        );
        assert_eq!(
            read_many::<bool>(test_request.headers().get_all("X-Bool-Single").iter()).unwrap(),
            vec![true, false, true, true]
        );
        read_many::<bool>(test_request.headers().get_all("X-Bool-Invalid").iter())
            .expect_err("invalid");
    }

    #[test]
    fn read_many_u16() {
        let test_request = http::Request::builder()
            .header("X-Multi", "123,456")
            .header("X-Multi", "789")
            .header("X-Num", "777")
            .header("X-Num-Invalid", "12ef3")
            .header("X-Num-Single", "1,2,3,4,5")
            .body(())
            .unwrap();
        assert_eq!(
            read_many::<u16>(test_request.headers().get_all("X-Multi").iter()).expect("valid"),
            vec![123, 456, 789]
        );

        assert_eq!(
            read_many::<u16>(test_request.headers().get_all("X-Num").iter()).unwrap(),
            vec![777]
        );
        assert_eq!(
            read_many::<u16>(test_request.headers().get_all("X-Num-Single").iter()).unwrap(),
            vec![1, 2, 3, 4, 5]
        );
        read_many::<u16>(test_request.headers().get_all("X-Num-Invalid").iter())
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
        let resp: Result<HashMap<String, Vec<u16>>, ParseError> =
            headers_for_prefix(test_request.headers(), "X-Prefix-")
                .map(|(key, header_name)| {
                    let values = test_request.headers().get_all(header_name);
                    read_many(values.iter()).map(|v| (key.to_string(), v))
                })
                .collect();
        let resp = resp.expect("valid");
        println!("{:?}", resp);
        assert_eq!(resp.get("a"), Some(&vec![123_u16, 456_u16]));
    }
}
