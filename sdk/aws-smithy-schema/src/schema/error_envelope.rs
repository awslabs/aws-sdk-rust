/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared helpers for parsing AWS protocol error envelopes.
//!
//! The JSON and CBOR protocols both extract an error code from a response
//! envelope and honor the awsQueryCompatible `X-Amzn-Query-Error` header.
//! The wire conventions for the code (a `namespace#Name` shape id, optionally
//! with a trailing `:url` suffix) and for that header are identical across the
//! two protocols, so the parsing lives here instead of being duplicated in
//! each codec.

use aws_smithy_runtime_api::http::Headers;

/// Strips a leading `namespace#` prefix and a trailing `:url` suffix from a
/// wire-format error code, leaving just the bare shape name.
///
/// Examples:
/// - `"FooError"` → `"FooError"`
/// - `"com.example#FooError"` → `"FooError"`
/// - `"FooError:http://internal.example.com/coral/.../"` → `"FooError"`
/// - `"com.example#FooError:http://..."` → `"FooError"`
pub fn sanitize_error_code(error_code: &str) -> &str {
    // Strip a trailing URL-style suffix (`:url`) first ...
    let error_code = match error_code.find(':') {
        Some(idx) => &error_code[..idx],
        None => error_code,
    };
    // ... then a leading `namespace#` prefix.
    match error_code.find('#') {
        Some(idx) => &error_code[idx + 1..],
        None => error_code,
    }
}

/// Parses an `X-Amzn-Query-Error: <code>;<type>` header into its `(code, type)`
/// halves, as emitted by awsQueryCompatible services.
///
/// Returns `None` if the header is absent or has no `;` separator.
pub fn parse_query_compatible_header(headers: &Headers) -> Option<(&str, &str)> {
    let value = headers.get("x-amzn-query-error")?;
    value
        .find(';')
        .map(|idx| (&value[..idx], &value[idx + 1..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::http::Headers;

    #[test]
    fn sanitize_strips_namespace_and_url() {
        assert_eq!(sanitize_error_code("FooError"), "FooError");
        assert_eq!(sanitize_error_code("com.example#FooError"), "FooError");
        assert_eq!(
            sanitize_error_code("FooError:http://internal.example.com/x"),
            "FooError"
        );
        assert_eq!(
            sanitize_error_code("com.example#FooError:http://internal/x"),
            "FooError"
        );
    }

    #[test]
    fn query_compatible_header_splits_on_semicolon() {
        let mut headers = Headers::new();
        headers.insert("x-amzn-query-error", "AWS.SimpleQueueService.Foo;Sender");
        assert_eq!(
            parse_query_compatible_header(&headers),
            Some(("AWS.SimpleQueueService.Foo", "Sender"))
        );
    }

    #[test]
    fn query_compatible_header_absent_or_malformed_is_none() {
        let empty = Headers::new();
        assert_eq!(parse_query_compatible_header(&empty), None);

        let mut no_separator = Headers::new();
        no_separator.insert("x-amzn-query-error", "NoSemicolon");
        assert_eq!(parse_query_compatible_header(&no_separator), None);
    }
}
