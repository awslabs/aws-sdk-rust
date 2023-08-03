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
use crate::response::ParseHttpResponse;
use crate::result::{SdkError, SdkSuccess};
use bytes::{Buf, Bytes};
use http_body::Body;
use pin_utils::pin_mut;
use std::error::Error;
use std::future::Future;
use tracing::{debug_span, trace, Instrument};

type BoxError = Box<dyn Error + Send + Sync>;

const LOG_SENSITIVE_BODIES: &str = "LOG_SENSITIVE_BODIES";

/// [`AsyncMapRequest`] defines an asynchronous middleware that transforms an [`operation::Request`].
///
/// Typically, these middleware will read configuration from the `PropertyBag` and use it to
/// augment the request.
///
/// Most fundamental middleware is expressed as `AsyncMapRequest`'s synchronous cousin, `MapRequest`,
/// including signing & endpoint resolution. `AsyncMapRequest` is used for async credential
/// retrieval (e.g., from AWS STS's AssumeRole operation).
pub trait AsyncMapRequest {
    /// The type returned when this [`AsyncMapRequest`] encounters an error.
    type Error: Into<BoxError> + 'static;
    /// The type returned when [`AsyncMapRequest::apply`] is called.
    type Future: Future<Output = Result<operation::Request, Self::Error>> + Send + 'static;

    /// Returns the name of this map request operation for inclusion in a tracing span.
    fn name(&self) -> &'static str;

    /// Call this middleware, returning a future that resolves to a request or an error.
    fn apply(&self, request: operation::Request) -> Self::Future;
}

/// [`MapRequest`] defines a synchronous middleware that transforms an [`operation::Request`].
///
/// Typically, these middleware will read configuration from the `PropertyBag` and use it to
/// augment the request. Most fundamental middleware is expressed as `MapRequest`, including
/// signing & endpoint resolution.
///
/// ## Examples
///
/// ```rust
/// # use aws_smithy_http::middleware::MapRequest;
/// # use std::convert::Infallible;
/// # use aws_smithy_http::operation;
/// use http::header::{HeaderName, HeaderValue};
///
/// /// Signaling struct added to the request property bag if a header should be added
/// struct NeedsHeader;
///
/// struct AddHeader(HeaderName, HeaderValue);
///
/// impl MapRequest for AddHeader {
///     type Error = Infallible;
///
///     fn name(&self) -> &'static str {
///         "add_header"
///     }
///
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

    /// Returns the name of this map request operation for inclusion in a tracing span.
    fn name(&self) -> &'static str;

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
    if let Some(parsed_response) =
        debug_span!("parse_unloaded").in_scope(|| handler.parse_unloaded(&mut response))
    {
        trace!(response = ?response, "read HTTP headers for streaming response");
        return sdk_result(parsed_response, response);
    }

    let (http_response, properties) = response.into_parts();
    let (parts, body) = http_response.into_parts();
    let body = match read_body(body).instrument(debug_span!("read_body")).await {
        Ok(body) => body,
        Err(err) => {
            return Err(SdkError::response_error(
                err,
                operation::Response::from_parts(
                    http::Response::from_parts(parts, SdkBody::taken()),
                    properties,
                ),
            ));
        }
    };

    let http_response = http::Response::from_parts(parts, Bytes::from(body));
    if !handler.sensitive()
        || std::env::var(LOG_SENSITIVE_BODIES)
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or_default()
    {
        trace!(http_response = ?http_response, "read HTTP response body");
    } else {
        trace!(http_response = "** REDACTED **. To print, set LOG_SENSITIVE_BODIES=true")
    }
    debug_span!("parse_loaded").in_scope(move || {
        let parsed = handler.parse_loaded(&http_response);
        sdk_result(
            parsed,
            operation::Response::from_parts(http_response.map(SdkBody::from), properties),
        )
    })
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
        Err(err) => Err(SdkError::service_error(err, raw)),
    }
}
