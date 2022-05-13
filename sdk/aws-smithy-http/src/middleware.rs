/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! This modules defines the core, framework agnostic, HTTP middleware interface
//! used by the SDK
//!
//! smithy-middleware-tower provides Tower-specific middleware utilities (todo)

use crate::body::SdkBody;
use crate::operation;
use crate::pin_mut;
use crate::response::ParseHttpResponse;
use crate::result::{SdkError, SdkSuccess};
use bytes::{Buf, Bytes};
use http_body::Body;
use std::error::Error;
use std::future::Future;
use tracing::trace;

type BoxError = Box<dyn Error + Send + Sync>;

/// [`AsyncMapRequest`] defines an asynchronous middleware that transforms an [`operation::Request`].
///
/// Typically, these middleware will read configuration from the `PropertyBag` and use it to
/// augment the request.
///
/// Most fundamental middleware is expressed as `AsyncMapRequest`'s synchronous cousin, `MapRequest`,
/// including signing & endpoint resolution. `AsyncMapRequest` is used for async credential
/// retrieval (e.g., from AWS STS's AssumeRole operation).
pub trait AsyncMapRequest {
    type Error: Into<BoxError> + 'static;
    type Future: Future<Output = Result<operation::Request, Self::Error>> + Send + 'static;

    fn apply(&self, request: operation::Request) -> Self::Future;
}

/// [`MapRequest`] defines a synchronous middleware that transforms an [`operation::Request`].
///
/// Typically, these middleware will read configuration from the `PropertyBag` and use it to
/// augment the request. Most fundamental middleware is expressed as `MapRequest`, including
/// signing & endpoint resolution.
///
/// ```rust
/// # use aws_smithy_http::middleware::MapRequest;
/// # use std::convert::Infallible;
/// # use aws_smithy_http::operation;
/// use http::header::{HeaderName, HeaderValue};
/// struct AddHeader(HeaderName, HeaderValue);
/// /// Signaling struct added to the request property bag if a header should be added
/// struct NeedsHeader;
/// impl MapRequest for AddHeader {
///     type Error = Infallible;
///     fn apply(&self, request: operation::Request) -> Result<operation::Request, Self::Error> {
///         request.augment(|mut request, properties| {
///             if properties.get::<NeedsHeader>().is_some() {
///                 request.headers_mut().append(self.0.clone(), self.1.clone());
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
/// - `O`: The Http response handler that returns `Result<T, E>`
/// - `T`/`E`: `Result<T, E>` returned by `handler`.
pub async fn load_response<T, E, O>(
    mut response: operation::Response,
    handler: &O,
) -> Result<SdkSuccess<T>, SdkError<E>>
where
    O: ParseHttpResponse<Output = Result<T, E>>,
{
    if let Some(parsed_response) = handler.parse_unloaded(&mut response) {
        trace!(response = ?response);
        return sdk_result(parsed_response, response);
    }

    let (http_response, properties) = response.into_parts();
    let (parts, body) = http_response.into_parts();
    let body = match read_body(body).await {
        Ok(body) => body,
        Err(err) => {
            return Err(SdkError::ResponseError {
                raw: operation::Response::from_parts(
                    http::Response::from_parts(parts, SdkBody::taken()),
                    properties,
                ),
                err,
            });
        }
    };

    let http_response = http::Response::from_parts(parts, Bytes::from(body));
    trace!(http_response = ?http_response);
    let parsed = handler.parse_loaded(&http_response);
    sdk_result(
        parsed,
        operation::Response::from_parts(http_response.map(SdkBody::from), properties),
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

/// Convert a `Result<T, E>` into an `SdkResult` that includes the operation response
fn sdk_result<T, E>(
    parsed: Result<T, E>,
    raw: operation::Response,
) -> Result<SdkSuccess<T>, SdkError<E>> {
    match parsed {
        Ok(parsed) => Ok(SdkSuccess { raw, parsed }),
        Err(err) => Err(SdkError::ServiceError { raw, err }),
    }
}
