/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP request and response types

mod error;
mod extensions;
mod headers;
mod non_utf8;
mod request;
mod response;

// Low-level header parsing shared with `aws-smithy-schema`. Hidden because, apart from
// `ParseError` below, it is not a stable API: `aws_smithy_http::header` is the supported
// path for these helpers. It is `pub` rather than `pub(crate)` only so that
// `aws-smithy-http` and `aws-smithy-schema` can reach one implementation of the parsers
// instead of maintaining two that can drift.
#[doc(hidden)]
pub mod header_parse;

pub use error::HttpError;
pub use header_parse::ParseError;
pub use headers::{HeaderValue, Headers, HeadersIter};
pub use non_utf8::NonUtf8HeaderHandling;
pub use request::{Request, RequestParts};
pub use response::{Response, StatusCode};
