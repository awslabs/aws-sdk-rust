/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types for response parsing.

use crate::operation;
use bytes::Bytes;

/// `ParseHttpResponse` is a generic trait for parsing structured data from HTTP responses.
///
/// It is designed to be nearly infinitely flexible, because `Output` is unconstrained, it can be used to support
/// event streams, S3 streaming responses, regular request-response style operations, as well
/// as any other HTTP-based protocol that we manage to come up with.
///
/// The split between `parse_unloaded` and `parse_loaded` enables keeping the parsing code pure and sync
/// whenever possible and delegating the process of actually reading the HTTP response to the caller when
/// the required behavior is simply "read to the end."
///
/// It also enables this critical and core trait to avoid being async, and it makes code that uses
/// the trait easier to test.
pub trait ParseHttpResponse {
    /// Output type of the HttpResponse.
    ///
    /// For request/response style operations, this is typically something like:
    /// `Result<ListTablesResponse, ListTablesError>`
    ///
    /// For streaming operations, this is something like:
    /// `Result<EventStream<TranscribeStreamingEvent>, TranscribeStreamingError>`
    type Output;

    /// Parse an HTTP request without reading the body. If the body must be provided to proceed,
    /// return `None`
    ///
    /// This exists to serve APIs like S3::GetObject where the body is passed directly into the
    /// response and consumed by the client. However, even in the case of S3::GetObject, errors
    /// require reading the entire body.
    ///
    /// This also facilitates `EventStream` and other streaming HTTP protocols by enabling the
    /// handler to take ownership of the HTTP response directly.
    ///
    /// Currently `parse_unloaded` operates on a borrowed HTTP request to enable
    /// the caller to provide a raw HTTP response to the caller for inspection after the response is
    /// returned. For EventStream-like use cases, the caller can use `mem::swap` to replace
    /// the streaming body with an empty body as long as the body implements default.
    ///
    /// We should consider if this is too limiting & if this should take an owned response instead.
    fn parse_unloaded(&self, response: &mut operation::Response) -> Option<Self::Output>;

    /// Parse an HTTP request from a fully loaded body. This is for standard request/response style
    /// APIs like AwsJson 1.0/1.1 and the error path of most streaming APIs
    ///
    /// Using an explicit body type of Bytes here is a conscious decision—If you _really_ need
    /// to precisely control how the data is loaded into memory (e.g. by using `bytes::Buf`), implement
    /// your handler in `parse_unloaded`.
    ///
    /// Production code will never call `parse_loaded` without first calling `parse_unloaded`. However,
    /// in tests it may be easier to use `parse_loaded` directly. It is OK to panic in `parse_loaded`
    /// if `parse_unloaded` will never return `None`, however, it may make your code easier to test if an
    /// implementation is provided.
    fn parse_loaded(&self, response: &http::Response<Bytes>) -> Self::Output;

    /// Returns whether the contents of this response are sensitive
    ///
    /// When this is set to true, wire logging will be disabled
    fn sensitive(&self) -> bool {
        false
    }
}

/// Convenience Trait for non-streaming APIs
///
/// `ParseStrictResponse` enables operations that _never_ need to stream the body incrementally to
/// have cleaner implementations. There is a blanket implementation
pub trait ParseStrictResponse {
    /// The type returned by this parser.
    type Output;

    /// Parse an [`http::Response<Bytes>`] into `Self::Output`.
    fn parse(&self, response: &http::Response<Bytes>) -> Self::Output;

    /// Returns whether the contents of this response are sensitive
    ///
    /// When this is set to true, wire logging will be disabled
    fn sensitive(&self) -> bool {
        false
    }
}

impl<T: ParseStrictResponse> ParseHttpResponse for T {
    type Output = T::Output;

    fn parse_unloaded(&self, _response: &mut operation::Response) -> Option<Self::Output> {
        None
    }

    fn parse_loaded(&self, response: &http::Response<Bytes>) -> Self::Output {
        self.parse(response)
    }

    fn sensitive(&self) -> bool {
        ParseStrictResponse::sensitive(self)
    }
}

#[cfg(test)]
mod test {
    use crate::body::SdkBody;
    use crate::operation;
    use crate::response::ParseHttpResponse;
    use bytes::Bytes;
    use std::mem;

    #[test]
    fn supports_streaming_body() {
        struct S3GetObject {
            _body: SdkBody,
        }

        struct S3GetObjectParser;

        impl ParseHttpResponse for S3GetObjectParser {
            type Output = S3GetObject;

            fn parse_unloaded(&self, response: &mut operation::Response) -> Option<Self::Output> {
                // For responses that pass on the body, use mem::take to leave behind an empty body
                let body = mem::replace(response.http_mut().body_mut(), SdkBody::taken());
                Some(S3GetObject { _body: body })
            }

            fn parse_loaded(&self, _response: &http::Response<Bytes>) -> Self::Output {
                unimplemented!()
            }
        }
    }
}
