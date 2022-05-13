/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::body::SdkBody;
use crate::property_bag::{PropertyBag, SharedPropertyBag};
use aws_smithy_types::date_time::DateTimeFormatError;
use http::uri::InvalidUri;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Debug)]
pub struct Metadata {
    operation: Cow<'static, str>,
    service: Cow<'static, str>,
}

impl Metadata {
    pub fn name(&self) -> &str {
        &self.operation
    }

    pub fn service(&self) -> &str {
        &self.service
    }

    pub fn new(
        operation: impl Into<Cow<'static, str>>,
        service: impl Into<Cow<'static, str>>,
    ) -> Self {
        Metadata {
            operation: operation.into(),
            service: service.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Parts<H, R> {
    pub response_handler: H,
    pub retry_policy: R,
    pub metadata: Option<Metadata>,
}

/// An error occurred attempting to build an `Operation` from an input
///
/// These are almost always due to user error caused by limitations of specific fields due to
/// protocol serialization (e.g. fields that can only be a subset ASCII because they are serialized
/// as the name of an HTTP header)
#[non_exhaustive]
#[derive(Debug)]
pub enum BuildError {
    /// A field contained an invalid value
    InvalidField {
        field: &'static str,
        details: String,
    },
    /// A field was missing
    MissingField {
        field: &'static str,
        details: &'static str,
    },
    /// The serializer could not serialize the input
    SerializationError(SerializationError),

    /// The serializer did not produce a valid URI
    ///
    /// This typically indicates that a field contained invalid characters.
    InvalidUri {
        uri: String,
        err: InvalidUri,
        message: Cow<'static, str>,
    },

    /// An error occurred request construction
    Other(Box<dyn Error + Send + Sync + 'static>),
}

impl From<SerializationError> for BuildError {
    fn from(err: SerializationError) -> Self {
        BuildError::SerializationError(err)
    }
}

impl From<DateTimeFormatError> for BuildError {
    fn from(err: DateTimeFormatError) -> Self {
        BuildError::from(SerializationError::from(err))
    }
}

impl Display for BuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::InvalidField { field, details } => write!(
                f,
                "Invalid field in input: {} (Details: {})",
                field, details
            ),
            BuildError::MissingField { field, details } => {
                write!(f, "{} was missing. {}", field, details)
            }
            BuildError::SerializationError(inner) => {
                write!(f, "failed to serialize input: {}", inner)
            }
            BuildError::Other(inner) => write!(f, "error during request construction: {}", inner),
            BuildError::InvalidUri { uri, err, message } => {
                write!(
                    f,
                    "generated URI `{}` was not a valid URI ({}): {}",
                    uri, err, message
                )
            }
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BuildError::SerializationError(inner) => Some(inner as _),
            BuildError::Other(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum SerializationError {
    #[non_exhaustive]
    CannotSerializeUnknownVariant { union: &'static str },
    #[non_exhaustive]
    DateTimeFormatError { cause: DateTimeFormatError },
}

impl SerializationError {
    pub fn unknown_variant(union: &'static str) -> Self {
        Self::CannotSerializeUnknownVariant { union }
    }
}

impl Display for SerializationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotSerializeUnknownVariant { union } => write!(
                f,
                "Cannot serialize `{}::Unknown`. Unknown union variants cannot be serialized. \
                This can occur when round-tripping a response from the server that was not \
                recognized by the SDK. Consider upgrading to the latest version of the SDK.",
                union
            ),
            Self::DateTimeFormatError { cause } => write!(f, "{}", cause),
        }
    }
}

impl Error for SerializationError {}

impl From<DateTimeFormatError> for SerializationError {
    fn from(err: DateTimeFormatError) -> SerializationError {
        SerializationError::DateTimeFormatError { cause: err }
    }
}

#[derive(Debug)]
pub struct Operation<H, R> {
    request: Request,
    parts: Parts<H, R>,
}

impl<H, R> Operation<H, R> {
    pub fn into_request_response(self) -> (Request, Parts<H, R>) {
        (self.request, self.parts)
    }
    pub fn from_parts(request: Request, parts: Parts<H, R>) -> Self {
        Self { request, parts }
    }

    pub fn properties_mut(&mut self) -> impl DerefMut<Target = PropertyBag> + '_ {
        self.request.properties_mut()
    }

    pub fn properties(&self) -> impl Deref<Target = PropertyBag> + '_ {
        self.request.properties()
    }

    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.parts.metadata = Some(metadata);
        self
    }

    pub fn with_retry_policy<R2>(self, retry_policy: R2) -> Operation<H, R2> {
        Operation {
            request: self.request,
            parts: Parts {
                response_handler: self.parts.response_handler,
                retry_policy,
                metadata: self.parts.metadata,
            },
        }
    }

    pub fn retry_policy(&self) -> &R {
        &self.parts.retry_policy
    }

    pub fn try_clone(&self) -> Option<Self>
    where
        H: Clone,
        R: Clone,
    {
        let request = self.request.try_clone()?;
        Some(Self {
            request,
            parts: self.parts.clone(),
        })
    }
}

impl<H> Operation<H, ()> {
    pub fn new(request: Request, response_handler: H) -> Self {
        Operation {
            request,
            parts: Parts {
                response_handler,
                retry_policy: (),
                metadata: None,
            },
        }
    }
}

/// Operation request type that associates a property bag with an underlying HTTP request.
/// This type represents the request in the Tower `Service` in middleware so that middleware
/// can share information with each other via the properties.
#[derive(Debug)]
pub struct Request {
    /// The underlying HTTP Request
    inner: http::Request<SdkBody>,

    /// Property bag of configuration options
    ///
    /// Middleware can read and write from the property bag and use its
    /// contents to augment the request (see [`Request::augment`](Request::augment))
    properties: SharedPropertyBag,
}

impl Request {
    /// Creates a new operation `Request` with the given `inner` HTTP request.
    pub fn new(inner: http::Request<SdkBody>) -> Self {
        Request {
            inner,
            properties: SharedPropertyBag::new(),
        }
    }

    /// Creates a new operation `Request` from its parts.
    pub fn from_parts(inner: http::Request<SdkBody>, properties: SharedPropertyBag) -> Self {
        Request { inner, properties }
    }

    /// Allows modification of the HTTP request and associated properties with a fallible closure.
    pub fn augment<T>(
        self,
        f: impl FnOnce(http::Request<SdkBody>, &mut PropertyBag) -> Result<http::Request<SdkBody>, T>,
    ) -> Result<Request, T> {
        let inner = {
            let properties: &mut PropertyBag = &mut self.properties.acquire_mut();
            f(self.inner, properties)?
        };
        Ok(Request {
            inner,
            properties: self.properties,
        })
    }

    /// Gives mutable access to the properties.
    pub fn properties_mut(&mut self) -> impl DerefMut<Target = PropertyBag> + '_ {
        self.properties.acquire_mut()
    }

    /// Gives readonly access to the properties.
    pub fn properties(&self) -> impl Deref<Target = PropertyBag> + '_ {
        self.properties.acquire()
    }

    /// Gives mutable access to the underlying HTTP request.
    pub fn http_mut(&mut self) -> &mut http::Request<SdkBody> {
        &mut self.inner
    }

    /// Gives readonly access to the underlying HTTP request.
    pub fn http(&self) -> &http::Request<SdkBody> {
        &self.inner
    }

    /// Attempts to clone the operation `Request`. This can fail if the
    /// request body can't be cloned, such as if it is being streamed and the
    /// stream can't be recreated.
    pub fn try_clone(&self) -> Option<Request> {
        let cloned_body = self.inner.body().try_clone()?;
        let mut cloned_request = http::Request::builder()
            .uri(self.inner.uri().clone())
            .method(self.inner.method());
        *cloned_request
            .headers_mut()
            .expect("builder has not been modified, headers must be valid") =
            self.inner.headers().clone();
        let inner = cloned_request
            .body(cloned_body)
            .expect("a clone of a valid request should be a valid request");
        Some(Request {
            inner,
            properties: self.properties.clone(),
        })
    }

    /// Consumes the operation `Request` and returns the underlying HTTP request and properties.
    pub fn into_parts(self) -> (http::Request<SdkBody>, SharedPropertyBag) {
        (self.inner, self.properties)
    }
}

/// Operation response type that associates a property bag with an underlying HTTP response.
/// This type represents the response in the Tower `Service` in middleware so that middleware
/// can share information with each other via the properties.
#[derive(Debug)]
pub struct Response {
    /// The underlying HTTP Response
    inner: http::Response<SdkBody>,

    /// Property bag of configuration options
    properties: SharedPropertyBag,
}

impl Response {
    /// Creates a new operation `Response` with the given `inner` HTTP response.
    pub fn new(inner: http::Response<SdkBody>) -> Self {
        Response {
            inner,
            properties: SharedPropertyBag::new(),
        }
    }

    /// Gives mutable access to the properties.
    pub fn properties_mut(&mut self) -> impl DerefMut<Target = PropertyBag> + '_ {
        self.properties.acquire_mut()
    }

    /// Gives readonly access to the properties.
    pub fn properties(&self) -> impl Deref<Target = PropertyBag> + '_ {
        self.properties.acquire()
    }

    /// Gives mutable access to the underlying HTTP response.
    pub fn http_mut(&mut self) -> &mut http::Response<SdkBody> {
        &mut self.inner
    }

    /// Gives readonly access to the underlying HTTP response.
    pub fn http(&self) -> &http::Response<SdkBody> {
        &self.inner
    }

    /// Consumes the operation `Request` and returns the underlying HTTP response and properties.
    pub fn into_parts(self) -> (http::Response<SdkBody>, SharedPropertyBag) {
        (self.inner, self.properties)
    }

    /// Creates a new operation `Response` from an HTTP response and property bag.
    pub fn from_parts(inner: http::Response<SdkBody>, properties: SharedPropertyBag) -> Self {
        Response { inner, properties }
    }
}

#[cfg(test)]
mod test {
    use crate::body::SdkBody;
    use crate::operation::Request;
    use http::header::{AUTHORIZATION, CONTENT_LENGTH};
    use http::Uri;

    #[test]
    fn try_clone_clones_all_data() {
        let mut request = Request::new(
            http::Request::builder()
                .uri(Uri::from_static("http://www.amazon.com"))
                .method("POST")
                .header(CONTENT_LENGTH, 456)
                .header(AUTHORIZATION, "Token: hello")
                .body(SdkBody::from("hello world!"))
                .expect("valid request"),
        );
        request.properties_mut().insert("hello");
        let cloned = request.try_clone().expect("request is cloneable");

        let (request, config) = cloned.into_parts();
        assert_eq!(request.uri(), &Uri::from_static("http://www.amazon.com"));
        assert_eq!(request.method(), "POST");
        assert_eq!(request.headers().len(), 2);
        assert_eq!(
            request.headers().get(AUTHORIZATION).unwrap(),
            "Token: hello"
        );
        assert_eq!(request.headers().get(CONTENT_LENGTH).unwrap(), "456");
        assert_eq!(request.body().bytes().unwrap(), "hello world!".as_bytes());
        assert_eq!(config.acquire().get::<&str>(), Some(&"hello"));
    }
}
