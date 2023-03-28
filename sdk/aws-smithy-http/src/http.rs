/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use http::{HeaderMap, HeaderValue};

/// Trait for accessing HTTP headers.
///
/// Useful for generic impls so that they can access headers via trait bounds.
pub trait HttpHeaders {
    /// Returns a reference to the associated header map.
    fn http_headers(&self) -> &HeaderMap<HeaderValue>;

    /// Returns a mutable reference to the associated header map.
    fn http_headers_mut(&mut self) -> &mut HeaderMap<HeaderValue>;
}

impl<B> HttpHeaders for http::Response<B> {
    fn http_headers(&self) -> &HeaderMap<HeaderValue> {
        self.headers()
    }

    fn http_headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        self.headers_mut()
    }
}

impl HttpHeaders for crate::operation::Response {
    fn http_headers(&self) -> &HeaderMap<HeaderValue> {
        self.http().http_headers()
    }

    fn http_headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
        self.http_mut().http_headers_mut()
    }
}
