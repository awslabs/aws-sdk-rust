/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! This modules defines the core, framework agnostic, HTTP middleware interface
//! used by the SDK
//!
//! smithy-middleware-tower provides Tower-specific middleware utilities (todo)

use crate::operation;
use crate::pin_mut;
use crate::response::ParseHttpResponse;
use crate::result::{SdkError, SdkSuccess};
use bytes::{Buf, Bytes};
use http::Response;
use http_body::Body;
use std::error::Error;

/// Body for debugging purposes
///
/// When receiving data from the AWS services, it is often helpful to be able to see the response
/// body that a service generated. When the SDK has fully buffered the body into memory, this
/// facilitates straightforward debugging of the response.
///
/// Take care when calling the debug implementation to avoid printing responses from sensitive operations.
#[derive(Debug)]
pub struct ResponseBody(Inner);

impl ResponseBody {
    /// Load a response body from a static string
    pub fn from_static(s: &'static str) -> Self {
        ResponseBody(Inner::Bytes(Bytes::from_static(s.as_bytes())))
    }

    /// Returns the raw bytes of this response
    ///
    /// When the response has been buffered into memory, the bytes are returned
    /// If the response is streaming or errored during the read process, `None` is returned.
    pub fn bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            Inner::Bytes(bytes) => Some(&bytes),
            _ => None,
        }
    }
}

/// Private ResponseBody internals
#[derive(Debug)]
enum Inner {
    Bytes(bytes::Bytes),
    Streaming,
    Err,
}

type BoxError = Box<dyn Error + Send + Sync>;

/// [`MapRequest`] defines a synchronous middleware that transforms an [`operation::Request`].
///
/// Typically, these middleware will read configuration from the `PropertyBag` and use it to
/// augment the request. Most fundamental middleware is expressed as `MapRequest`, including
/// signing & endpoint resolution.
///
/// ```rust
/// # use smithy_http::middleware::MapRequest;
/// # use std::convert::Infallible;
/// # use smithy_http::operation;
/// use http::header::{HeaderName, HeaderValue};
/// struct AddHeader(HeaderName, HeaderValue);
/// /// Signaling struct added to the request property bag if a header should be added
/// struct NeedsHeader;
/// impl MapRequest for AddHeader {
///     type Error = Infallible;
///     fn apply(&self, request: operation::Request) -> Result<operation::Request, Self::Error> {
///         request.augment(|mut request, properties| {
///             if properties.get::<NeedsHeader>().is_some() {
///                 request.headers_mut().append(
///                     self.0.clone(),
///                     self.1.clone(),
///                 );
///             }
///             Ok(request)
///         })
///     }
/// }
/// ```
pub trait MapRequest {
    /// The Error type returned by this operation.
    ///
    /// If this middleware never fails use [std::convert::Infallible] or similar.
    type Error: Into<BoxError>;

    /// Apply this middleware to a request.
    ///
    /// Typically, implementations will use [`request.augment`](crate::operation::Request::augment)
    /// to be able to transform an owned `http::Request`.
    fn apply(&self, request: operation::Request) -> Result<operation::Request, Self::Error>;
}

/// Load a response using `handler` to parse the results.
///
/// This function is intended to be used on the response side of a middleware chain.
///
/// Success and failure will be split and mapped into `SdkSuccess` and `SdkError`.
/// Generic Parameters:
/// - `B`: The Response Body
/// - `O`: The Http response handler that returns `Result<T, E>`
/// - `T`/`E`: `Result<T, E>` returned by `handler`.
pub async fn load_response<B, T, E, O>(
    mut response: http::Response<B>,
    handler: &O,
) -> Result<SdkSuccess<T>, SdkError<E>>
where
    B: http_body::Body,
    B::Error: Into<BoxError>,
    O: ParseHttpResponse<B, Output = Result<T, E>>,
{
    if let Some(parsed_response) = handler.parse_unloaded(&mut response) {
        return sdk_result(
            parsed_response,
            response.map(|_| ResponseBody(Inner::Streaming)),
        );
    }
    let (parts, body) = response.into_parts();

    let body = match read_body(body).await {
        Ok(body) => body,
        Err(e) => {
            return Err(SdkError::ResponseError {
                raw: Response::from_parts(parts, ResponseBody(Inner::Err)),
                err: e.into(),
            });
        }
    };

    let response = Response::from_parts(parts, Bytes::from(body));
    let parsed = handler.parse_loaded(&response);
    sdk_result(
        parsed,
        response.map(|body| ResponseBody(Inner::Bytes(body))),
    )
}

async fn read_body<B: http_body::Body>(body: B) -> Result<Vec<u8>, B::Error> {
    let mut output = Vec::new();
    pin_mut!(body);
    while let Some(buf) = body.data().await {
        let mut buf = buf?;
        while buf.has_remaining() {
            output.extend_from_slice(buf.chunk());
            buf.advance(buf.chunk().len())
        }
    }
    Ok(output)
}

/// Convert a `Result<T, E>` into an `SdkResult` that includes the raw HTTP response
fn sdk_result<T, E>(
    parsed: Result<T, E>,
    raw: http::Response<ResponseBody>,
) -> Result<SdkSuccess<T>, SdkError<E>> {
    match parsed {
        Ok(parsed) => Ok(SdkSuccess { raw, parsed }),
        Err(err) => Err(SdkError::ServiceError { raw, err }),
    }
}
