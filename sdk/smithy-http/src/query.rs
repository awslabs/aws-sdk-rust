/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

/// Formatting values into the query string as specified in
/// [httpQuery](https://awslabs.github.io/smithy/1.0/spec/core/http-traits.html#httpquery-trait)
use smithy_types::Instant;
use std::fmt::Debug;

const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

pub fn fmt_default<T: Debug>(t: T) -> String {
    format!("{:?}", t)
}

pub fn fmt_string<T: AsRef<str>>(t: T) -> String {
    let bytes = t.as_ref();
    let final_capacity = bytes
        .chars()
        .map(|c| {
            if is_valid_query(c) {
                1
            } else {
                c.len_utf8() * 3
            }
        })
        .sum();
    let mut out = String::with_capacity(final_capacity);
    for char in bytes.chars() {
        url_encode(char, &mut out);
    }
    debug_assert_eq!(out.capacity(), final_capacity);
    out
}

pub fn fmt_timestamp(t: &Instant, format: smithy_types::instant::Format) -> String {
    fmt_string(t.fmt(format))
}

fn is_valid_query(c: char) -> bool {
    // Although & / = are allowed in the query string, we want to escape them
    let explicit_invalid = |c: char| !matches!(c, '&' | '=');
    let unreserved = |c: char| c.is_alphanumeric() || matches!(c, '-' | '.' | '_' | '~');

    // RFC-3986 §3.3 allows sub-delims (defined in section2.2) to be in the path component.
    // This includes both colon ':' and comma ',' characters.
    // Smithy protocol tests percent encode these expected values though whereas `encodeUrlPath()`
    // does not and follows the RFC. Fixing the tests was discussed but would adversely affect
    // other SDK's and we were asked to work around it.
    // Replace any left over sub-delims with the percent encoded value so that tests can proceed
    // https://tools.ietf.org/html/rfc3986#section-3.3

    let sub_delims = |c: char| match c {
        '!' | '$' | '\'' | '(' | ')' | '*' | '+' | /*',' |*/ ';' => true,
        // TODO: should &/= be url encoded?
        '&' | '=' => false,
        _ => false,
    };
    let p_char = |c: char| unreserved(c) || sub_delims(c) || /* c == ':' || */ c == '@';
    explicit_invalid(c) && (p_char(c) || c == '/' || c == '?')
}

fn url_encode(c: char, buff: &mut String) {
    if is_valid_query(c) {
        buff.push(c)
    } else {
        let mut inner_buff = [0; 4];
        let u8_slice = c.encode_utf8(&mut inner_buff).as_bytes();
        for c in u8_slice {
            let upper = (c & 0xf0) >> 4;
            let lower = c & 0x0f;
            buff.push('%');
            buff.push(HEX_CHARS[upper as usize] as char);
            buff.push(HEX_CHARS[lower as usize] as char);
        }
    }
}

/// Simple abstraction to enable appending params to a string as query params
///
/// ```rust
/// use smithy_http::query::Writer;
/// let mut s = String::from("www.example.com");
/// let mut q = Writer::new(&mut s);
/// q.push_kv("key", "value");
/// q.push_v("another_value");
/// assert_eq!(s, "www.example.com?key=value&another_value");
/// ```
pub struct Writer<'a> {
    out: &'a mut String,
    prefix: char,
}

impl<'a> Writer<'a> {
    pub fn new(out: &'a mut String) -> Self {
        Writer { out, prefix: '?' }
    }

    pub fn push_kv(&mut self, k: &str, v: &str) {
        self.out.push(self.prefix);
        self.out.push_str(k);
        self.out.push('=');
        self.out.push_str(v);
        self.prefix = '&';
    }

    pub fn push_v(&mut self, v: &str) {
        self.out.push(self.prefix);
        self.out.push_str(v);
    }
}

#[cfg(test)]
mod test {
    use crate::query::{fmt_string, is_valid_query};

    #[test]
    fn test_valid_query_chars() {
        assert_eq!(is_valid_query(' '), false);
        assert_eq!(is_valid_query('a'), true);
        assert_eq!(is_valid_query('/'), true);
        assert_eq!(is_valid_query('%'), false);
    }

    #[test]
    fn test_url_encode() {
        assert_eq!(fmt_string("y̆").as_str(), "y%CC%86");
        assert_eq!(fmt_string(" ").as_str(), "%20");
        assert_eq!(fmt_string("foo/baz%20").as_str(), "foo/baz%2520");
        assert_eq!(fmt_string("&=").as_str(), "%26%3D");
        assert_eq!(fmt_string("🐱").as_str(), "%F0%9F%90%B1");
    }
}
