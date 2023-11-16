/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Http Response Types

use crate::http::{HeaderValue, Headers, HttpError};
use aws_smithy_types::body::SdkBody;
use http as http0;
use http0::{Extensions, HeaderMap};
use std::fmt;

/// HTTP response status code
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct StatusCode(u16);

impl StatusCode {
    /// True if this is a successful response code (200, 201, etc)
    pub fn is_success(self) -> bool {
        (200..300).contains(&self.0)
    }

    /// True if this response code is a client error (4xx)
    pub fn is_client_error(self) -> bool {
        (400..500).contains(&self.0)
    }

    /// True if this response code is a server error (5xx)
    pub fn is_server_error(self) -> bool {
        (500..600).contains(&self.0)
    }

    /// Return the value of this status code as a `u16`.
    pub fn as_u16(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for StatusCode {
    type Error = HttpError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (100..1000).contains(&value) {
            Ok(StatusCode(value))
        } else {
            Err(HttpError::invalid_status_code())
        }
    }
}

impl From<http0::StatusCode> for StatusCode {
    fn from(value: http0::StatusCode) -> Self {
        Self(value.as_u16())
    }
}

impl From<StatusCode> for u16 {
    fn from(value: StatusCode) -> Self {
        value.0
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// An HTTP Response Type
#[derive(Debug)]
pub struct Response<B = SdkBody> {
    status: StatusCode,
    headers: Headers,
    body: B,
    extensions: Extensions,
}

impl<B> Response<B> {
    /// Converts this response into an http 0.x response.
    ///
    /// Depending on the internal storage type, this operation may be free or it may have an internal
    /// cost.
    pub fn try_into_http02x(self) -> Result<http0::Response<B>, HttpError> {
        let mut res = http::Response::builder()
            .status(
                http0::StatusCode::from_u16(self.status.into())
                    .expect("validated upon construction"),
            )
            .body(self.body)
            .expect("known valid");
        let mut headers = HeaderMap::new();
        headers.extend(
            self.headers
                .headers
                .into_iter()
                .map(|(k, v)| (k, v.into_http02x())),
        );
        *res.headers_mut() = headers;
        *res.extensions_mut() = self.extensions;
        Ok(res)
    }

    /// Update the body of this response to be a new body.
    pub fn map<U>(self, f: impl Fn(B) -> U) -> Response<U> {
        Response {
            status: self.status,
            body: f(self.body),
            extensions: self.extensions,
            headers: self.headers,
        }
    }

    /// Returns a response with the given status and body
    pub fn new(status: StatusCode, body: B) -> Self {
        Self {
            status,
            body,
            extensions: Default::default(),
            headers: Default::default(),
        }
    }

    /// Returns the status code
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns a mutable reference to the status code
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    /// Returns a reference to the header map
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a mutable reference to the header map
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.headers
    }

    /// Returns the body associated with the request
    pub fn body(&self) -> &B {
        &self.body
    }

    /// Returns a mutable reference to the body
    pub fn body_mut(&mut self) -> &mut B {
        &mut self.body
    }

    /// Converts this response into the response body.
    pub fn into_body(self) -> B {
        self.body
    }

    /// Adds an extension to the response extensions
    pub fn add_extension<T: Send + Sync + Clone + 'static>(&mut self, extension: T) {
        self.extensions.insert(extension);
    }
}

impl Response<SdkBody> {
    /// Replaces this response's body with [`SdkBody::taken()`]
    pub fn take_body(&mut self) -> SdkBody {
        std::mem::replace(self.body_mut(), SdkBody::taken())
    }
}

impl<B> TryFrom<http0::Response<B>> for Response<B> {
    type Error = HttpError;

    fn try_from(value: http0::Response<B>) -> Result<Self, Self::Error> {
        if let Some(e) = value
            .headers()
            .values()
            .filter_map(|value| std::str::from_utf8(value.as_bytes()).err())
            .next()
        {
            Err(HttpError::header_was_not_a_string(e))
        } else {
            let (parts, body) = value.into_parts();
            let mut string_safe_headers: HeaderMap<HeaderValue> = Default::default();
            string_safe_headers.extend(
                parts
                    .headers
                    .into_iter()
                    .map(|(k, v)| (k, HeaderValue::from_http02x(v).expect("validated above"))),
            );
            Ok(Self {
                status: StatusCode::try_from(parts.status.as_u16()).expect("validated by http 0.x"),
                body,
                extensions: parts.extensions,
                headers: Headers {
                    headers: string_safe_headers,
                },
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use aws_smithy_types::body::SdkBody;

    #[test]
    fn non_ascii_responses() {
        let response = http::Response::builder()
            .status(200)
            .header("k", "😹")
            .body(SdkBody::empty())
            .unwrap();
        let response: Response = response
            .try_into()
            .expect("failed to convert a non-string header");
        assert_eq!(response.headers().get("k"), Some("😹"))
    }

    #[test]
    fn response_can_be_created() {
        let req = http::Response::builder()
            .status(200)
            .body(SdkBody::from("hello"))
            .unwrap();
        let mut req = super::Response::try_from(req).unwrap();
        req.headers_mut().insert("a", "b");
        assert_eq!("b", req.headers().get("a").unwrap());
        req.headers_mut().append("a", "c");
        assert_eq!("b", req.headers().get("a").unwrap());
        let http0 = req.try_into_http02x().unwrap();
        assert_eq!(200, http0.status().as_u16());
    }

    #[test]
    #[should_panic]
    fn header_panics() {
        let res = http::Response::builder()
            .status(200)
            .body(SdkBody::from("hello"))
            .unwrap();
        let mut res = Response::try_from(res).unwrap();
        let _ = res
            .headers_mut()
            .try_insert("a\nb", "a\nb")
            .expect_err("invalid header");
        let _ = res.headers_mut().insert("a\nb", "a\nb");
    }
}
