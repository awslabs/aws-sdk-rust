/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    // missing_docs,
    // rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

mod urlencoded;
mod xml;

use crate::sealed::GetNormalizedHeader;
use crate::xml::try_xml_equivalent;
use assert_json_diff::assert_json_eq_no_panic;
use aws_smithy_runtime_api::client::http::request::Headers;
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use http::{HeaderMap, Uri};
use pretty_assertions::Comparison;
use std::collections::HashSet;
use std::fmt::{self, Debug};
use thiserror::Error;
use urlencoded::try_url_encoded_form_equivalent;

/// Helper trait for tests for float comparisons
///
/// This trait differs in float's default `PartialEq` implementation by considering all `NaN` values to
/// be equal.
pub trait FloatEquals {
    fn float_equals(&self, other: &Self) -> bool;
}

impl FloatEquals for f64 {
    fn float_equals(&self, other: &Self) -> bool {
        (self.is_nan() && other.is_nan()) || self.eq(other)
    }
}

impl FloatEquals for f32 {
    fn float_equals(&self, other: &Self) -> bool {
        (self.is_nan() && other.is_nan()) || self.eq(other)
    }
}

impl<T> FloatEquals for Option<T>
where
    T: FloatEquals,
{
    fn float_equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(this), Some(other)) => this.float_equals(other),
            (None, None) => true,
            _else => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ProtocolTestFailure {
    #[error("missing query param: expected `{expected}`, found {found:?}")]
    MissingQueryParam {
        expected: String,
        found: Vec<String>,
    },
    #[error("forbidden query param present: `{expected}`")]
    ForbiddenQueryParam { expected: String },
    #[error("required query param missing: `{expected}`")]
    RequiredQueryParam { expected: String },

    #[error("invalid header value for key `{key}`: expected `{expected}`, found `{found}`")]
    InvalidHeader {
        key: String,
        expected: String,
        found: String,
    },
    #[error("missing required header: `{expected}`")]
    MissingHeader { expected: String },
    #[error("Header `{forbidden}` was forbidden but found: `{found}`")]
    ForbiddenHeader { forbidden: String, found: String },
    #[error(
        "body did not match. left=actual, right=expected\n{comparison:?} \n == hint:\n{hint}."
    )]
    BodyDidNotMatch {
        // the comparison includes colorized escapes. PrettyString ensures that even during
        // debug printing, these appear
        comparison: PrettyString,
        hint: String,
    },
    #[error("Expected body to be valid {expected} but instead: {found}")]
    InvalidBodyFormat { expected: String, found: String },
}

/// Check that the protocol test succeeded & print the pretty error
/// if it did not
///
/// The primary motivation is making multiline debug output
/// readable & using the cleaner Display implementation
#[track_caller]
pub fn assert_ok(inp: Result<(), ProtocolTestFailure>) {
    match inp {
        Ok(_) => (),
        Err(e) => {
            eprintln!("{}", e);
            panic!("Protocol test failed");
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
struct QueryParam<'a> {
    key: &'a str,
    value: Option<&'a str>,
}

impl<'a> QueryParam<'a> {
    fn parse(s: &'a str) -> Self {
        let mut parsed = s.split('=');
        QueryParam {
            key: parsed.next().unwrap(),
            value: parsed.next(),
        }
    }
}

fn extract_params(uri: &str) -> HashSet<&str> {
    let query = uri.rsplit_once('?').map(|s| s.1).unwrap_or_default();
    query.split('&').collect()
}

#[track_caller]
pub fn assert_uris_match(left: impl AsRef<str>, right: impl AsRef<str>) {
    let left = left.as_ref();
    let right = right.as_ref();
    if left == right {
        return;
    }
    assert_eq!(
        extract_params(left),
        extract_params(right),
        "Query parameters did not match. left: {}, right: {}",
        left,
        right
    );
    let left: Uri = left.parse().expect("left is not a valid URI");
    let right: Uri = right.parse().expect("left is not a valid URI");
    assert_eq!(left.authority(), right.authority());
    assert_eq!(left.scheme(), right.scheme());
    assert_eq!(left.path(), right.path());
}

pub fn validate_query_string(
    request: &HttpRequest,
    expected_params: &[&str],
) -> Result<(), ProtocolTestFailure> {
    let actual_params = extract_params(request.uri());
    for param in expected_params {
        if !actual_params.contains(param) {
            return Err(ProtocolTestFailure::MissingQueryParam {
                expected: param.to_string(),
                found: actual_params.iter().map(|s| s.to_string()).collect(),
            });
        }
    }
    Ok(())
}

pub fn forbid_query_params(
    request: &HttpRequest,
    forbid_params: &[&str],
) -> Result<(), ProtocolTestFailure> {
    let actual_params: HashSet<QueryParam<'_>> = extract_params(request.uri())
        .iter()
        .map(|param| QueryParam::parse(param))
        .collect();
    let actual_keys: HashSet<&str> = actual_params.iter().map(|param| param.key).collect();
    for param in forbid_params {
        let parsed = QueryParam::parse(param);
        // If the forbidden param is k=v, then forbid this key-value pair
        if actual_params.contains(&parsed) {
            return Err(ProtocolTestFailure::ForbiddenQueryParam {
                expected: param.to_string(),
            });
        }
        // If the assertion is only about a key, then check keys
        if parsed.value.is_none() && actual_keys.contains(parsed.key) {
            return Err(ProtocolTestFailure::ForbiddenQueryParam {
                expected: param.to_string(),
            });
        }
    }
    Ok(())
}

pub fn require_query_params(
    request: &HttpRequest,
    require_keys: &[&str],
) -> Result<(), ProtocolTestFailure> {
    let actual_keys: HashSet<&str> = extract_params(request.uri())
        .iter()
        .map(|param| QueryParam::parse(param).key)
        .collect();
    for key in require_keys {
        if !actual_keys.contains(*key) {
            return Err(ProtocolTestFailure::RequiredQueryParam {
                expected: key.to_string(),
            });
        }
    }
    Ok(())
}

mod sealed {
    pub trait GetNormalizedHeader {
        fn get_header(&self, key: &str) -> Option<String>;
    }
}

impl<'a> GetNormalizedHeader for &'a Headers {
    fn get_header(&self, key: &str) -> Option<String> {
        if !self.contains_key(key) {
            None
        } else {
            Some(self.get_all(key).collect::<Vec<_>>().join(", "))
        }
    }
}

impl<'a> GetNormalizedHeader for &'a HeaderMap {
    fn get_header(&self, key: &str) -> Option<String> {
        if !self.contains_key(key) {
            None
        } else {
            Some(
                self.get_all(key)
                    .iter()
                    .map(|value| std::str::from_utf8(value.as_bytes()).expect("invalid utf-8"))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        }
    }
}

pub fn validate_headers<'a>(
    actual_headers: impl GetNormalizedHeader,
    expected_headers: impl IntoIterator<Item = (impl AsRef<str> + 'a, impl AsRef<str> + 'a)>,
) -> Result<(), ProtocolTestFailure> {
    for (key, expected_value) in expected_headers {
        let key = key.as_ref();
        let expected_value = expected_value.as_ref();
        match actual_headers.get_header(key) {
            None => {
                return Err(ProtocolTestFailure::MissingHeader {
                    expected: key.to_string(),
                })
            }
            Some(actual_value) if actual_value != *expected_value => {
                return Err(ProtocolTestFailure::InvalidHeader {
                    key: key.to_string(),
                    expected: expected_value.to_string(),
                    found: actual_value,
                })
            }
            _ => (),
        }
    }
    Ok(())
}

pub fn forbid_headers(
    headers: impl GetNormalizedHeader,
    forbidden_headers: &[&str],
) -> Result<(), ProtocolTestFailure> {
    for key in forbidden_headers {
        // Protocol tests store header lists as comma-delimited
        if let Some(value) = headers.get_header(key) {
            return Err(ProtocolTestFailure::ForbiddenHeader {
                forbidden: key.to_string(),
                found: format!("{}: {}", key, value),
            });
        }
    }
    Ok(())
}

pub fn require_headers(
    headers: impl GetNormalizedHeader,
    required_headers: &[&str],
) -> Result<(), ProtocolTestFailure> {
    for key in required_headers {
        // Protocol tests store header lists as comma-delimited
        if headers.get_header(key).is_none() {
            return Err(ProtocolTestFailure::MissingHeader {
                expected: key.to_string(),
            });
        }
    }
    Ok(())
}

#[derive(Clone)]
pub enum MediaType {
    /// Json media types are deserialized and compared
    Json,
    /// XML media types are normalized and compared
    Xml,
    /// For x-www-form-urlencoded, do some map order comparison shenanigans
    UrlEncodedForm,
    /// Other media types are compared literally
    Other(String),
}

impl<T: AsRef<str>> From<T> for MediaType {
    fn from(inp: T) -> Self {
        match inp.as_ref() {
            "application/json" => MediaType::Json,
            "application/x-amz-json-1.1" => MediaType::Json,
            "application/xml" => MediaType::Xml,
            "application/x-www-form-urlencoded" => MediaType::UrlEncodedForm,
            other => MediaType::Other(other.to_string()),
        }
    }
}

pub fn validate_body<T: AsRef<[u8]>>(
    actual_body: T,
    expected_body: &str,
    media_type: MediaType,
) -> Result<(), ProtocolTestFailure> {
    let body_str = std::str::from_utf8(actual_body.as_ref());
    match (media_type, body_str) {
        (MediaType::Json, Ok(actual_body)) => try_json_eq(expected_body, actual_body),
        (MediaType::Xml, Ok(actual_body)) => try_xml_equivalent(expected_body, actual_body),
        (MediaType::Json, Err(_)) => Err(ProtocolTestFailure::InvalidBodyFormat {
            expected: "json".to_owned(),
            found: "input was not valid UTF-8".to_owned(),
        }),
        (MediaType::Xml, Err(_)) => Err(ProtocolTestFailure::InvalidBodyFormat {
            expected: "XML".to_owned(),
            found: "input was not valid UTF-8".to_owned(),
        }),
        (MediaType::UrlEncodedForm, Ok(actual_body)) => {
            try_url_encoded_form_equivalent(expected_body, actual_body)
        }
        (MediaType::UrlEncodedForm, Err(_)) => Err(ProtocolTestFailure::InvalidBodyFormat {
            expected: "x-www-form-urlencoded".to_owned(),
            found: "input was not valid UTF-8".to_owned(),
        }),
        (MediaType::Other(media_type), Ok(actual_body)) => {
            if actual_body != expected_body {
                Err(ProtocolTestFailure::BodyDidNotMatch {
                    comparison: pretty_comparison(expected_body, actual_body),
                    hint: format!("media type: {}", media_type),
                })
            } else {
                Ok(())
            }
        }
        // It's not clear from the Smithy spec exactly how a binary / base64 encoded body is supposed
        // to work. Defer implementation for now until an actual test exists.
        (MediaType::Other(_), Err(_)) => {
            unimplemented!("binary/non-utf8 formats not yet supported")
        }
    }
}

#[derive(Eq, PartialEq)]
struct PrettyStr<'a>(&'a str);
impl Debug for PrettyStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Eq, PartialEq)]
pub struct PrettyString(String);
impl Debug for PrettyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn pretty_comparison(expected: &str, actual: &str) -> PrettyString {
    PrettyString(format!(
        "{}",
        Comparison::new(&PrettyStr(expected), &PrettyStr(actual))
    ))
}

fn try_json_eq(expected: &str, actual: &str) -> Result<(), ProtocolTestFailure> {
    let expected_json: serde_json::Value =
        serde_json::from_str(expected).expect("expected value must be valid JSON");
    let actual_json: serde_json::Value =
        serde_json::from_str(actual).map_err(|e| ProtocolTestFailure::InvalidBodyFormat {
            expected: "json".to_owned(),
            found: e.to_string() + actual,
        })?;
    match assert_json_eq_no_panic(&actual_json, &expected_json) {
        Ok(()) => Ok(()),
        Err(message) => Err(ProtocolTestFailure::BodyDidNotMatch {
            comparison: pretty_comparison(expected, actual),
            hint: message,
        }),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        forbid_headers, forbid_query_params, require_headers, require_query_params, validate_body,
        validate_headers, validate_query_string, FloatEquals, MediaType, ProtocolTestFailure,
    };
    use aws_smithy_runtime_api::client::http::request::Headers;
    use aws_smithy_runtime_api::client::orchestrator::HttpRequest;

    fn make_request(uri: &str) -> HttpRequest {
        let mut req = HttpRequest::empty();
        req.set_uri(uri).unwrap();
        req
    }

    #[test]
    fn test_validate_empty_query_string() {
        let request = HttpRequest::empty();
        validate_query_string(&request, &[]).expect("no required params should pass");
        validate_query_string(&request, &["a"]).expect_err("no params provided");
    }

    #[test]
    fn test_validate_query_string() {
        let request = make_request("/foo?a=b&c&d=efg&hello=a%20b");
        validate_query_string(&request, &["a=b"]).expect("a=b is in the query string");
        validate_query_string(&request, &["c", "a=b"])
            .expect("both params are in the query string");
        validate_query_string(&request, &["a=b", "c", "d=efg", "hello=a%20b"])
            .expect("all params are in the query string");
        validate_query_string(&request, &[]).expect("no required params should pass");

        validate_query_string(&request, &["a"]).expect_err("no parameter should match");
        validate_query_string(&request, &["a=bc"]).expect_err("no parameter should match");
        validate_query_string(&request, &["a=bc"]).expect_err("no parameter should match");
        validate_query_string(&request, &["hell=a%20"]).expect_err("no parameter should match");
    }

    #[test]
    fn test_forbid_query_param() {
        let request = make_request("/foo?a=b&c&d=efg&hello=a%20b");
        forbid_query_params(&request, &["a"]).expect_err("a is a query param");
        forbid_query_params(&request, &["not_included"]).expect("query param not included");
        forbid_query_params(&request, &["a=b"]).expect_err("if there is an `=`, match against KV");
        forbid_query_params(&request, &["c"]).expect_err("c is a query param");
        forbid_query_params(&request, &["a=c"]).expect("there is no a=c query param set");
    }

    #[test]
    fn test_require_query_param() {
        let request = make_request("/foo?a=b&c&d=efg&hello=a%20b");
        require_query_params(&request, &["a"]).expect("a is a query param");
        require_query_params(&request, &["not_included"]).expect_err("query param not included");
        require_query_params(&request, &["a=b"]).expect_err("should be matching against keys");
        require_query_params(&request, &["c"]).expect("c is a query param");
    }

    #[test]
    fn test_validate_headers() {
        let mut headers = Headers::new();
        headers.append("x-foo", "foo");
        headers.append("x-foo-list", "foo");
        headers.append("x-foo-list", "bar");
        headers.append("x-inline", "inline, other");

        validate_headers(&headers, [("X-Foo", "foo")]).expect("header present");
        validate_headers(&headers, [("X-Foo", "Foo")]).expect_err("case sensitive");
        validate_headers(&headers, [("x-foo-list", "foo, bar")]).expect("list concat");
        validate_headers(&headers, [("X-Foo-List", "foo")])
            .expect_err("all list members must be specified");
        validate_headers(&headers, [("X-Inline", "inline, other")])
            .expect("inline header lists also work");
        assert_eq!(
            validate_headers(&headers, [("missing", "value")]),
            Err(ProtocolTestFailure::MissingHeader {
                expected: "missing".to_owned()
            })
        );
    }

    #[test]
    fn test_forbidden_headers() {
        let mut headers = Headers::new();
        headers.append("x-foo", "foo");
        assert_eq!(
            forbid_headers(&headers, &["X-Foo"]).expect_err("should be error"),
            ProtocolTestFailure::ForbiddenHeader {
                forbidden: "X-Foo".to_string(),
                found: "X-Foo: foo".to_string()
            }
        );
        forbid_headers(&headers, &["X-Bar"]).expect("header not present");
    }

    #[test]
    fn test_required_headers() {
        let mut headers = Headers::new();
        headers.append("x-foo", "foo");
        require_headers(&headers, &["X-Foo"]).expect("header present");
        require_headers(&headers, &["X-Bar"]).expect_err("header not present");
    }

    #[test]
    fn test_validate_json_body() {
        let expected = r#"{"abc": 5 }"#;
        let actual = r#"   {"abc":   5 }"#;
        validate_body(actual, expected, MediaType::Json).expect("inputs matched as JSON");

        let expected = r#"{"abc": 5 }"#;
        let actual = r#"   {"abc":   6 }"#;
        validate_body(actual, expected, MediaType::Json).expect_err("bodies do not match");
    }

    #[test]
    fn test_validate_xml_body() {
        let expected = r#"<a>
        hello123
        </a>"#;
        let actual = "<a>hello123</a>";
        validate_body(actual, expected, MediaType::Xml).expect("inputs match as XML");
        let expected = r#"<a>
        hello123
        </a>"#;
        let actual = "<a>hello124</a>";
        validate_body(actual, expected, MediaType::Xml).expect_err("inputs are different");
    }

    #[test]
    fn test_validate_non_json_body() {
        let expected = r#"asdf"#;
        let actual = r#"asdf "#;
        validate_body(actual, expected, MediaType::from("something/else"))
            .expect_err("bodies do not match");

        validate_body(expected, expected, MediaType::from("something/else"))
            .expect("inputs matched exactly")
    }

    #[test]
    fn test_validate_headers_http0x() {
        let request = http::Request::builder().header("a", "b").body(()).unwrap();
        validate_headers(request.headers(), [("a", "b")]).unwrap()
    }

    #[test]
    fn test_float_equals() {
        let a = f64::NAN;
        let b = f64::NAN;
        assert_ne!(a, b);
        assert!(a.float_equals(&b));
        assert!(!a.float_equals(&5_f64));

        assert!(5.0.float_equals(&5.0));
        assert!(!5.0.float_equals(&5.1));

        assert!(f64::INFINITY.float_equals(&f64::INFINITY));
        assert!(!f64::INFINITY.float_equals(&f64::NEG_INFINITY));
        assert!(f64::NEG_INFINITY.float_equals(&f64::NEG_INFINITY));
    }
}
