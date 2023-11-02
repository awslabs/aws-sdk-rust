/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Http Request Types

use aws_smithy_types::body::SdkBody;
use http as http0;
use http::header::{InvalidHeaderName, InvalidHeaderValue};
use http::uri::InvalidUri;
use http0::header::Iter;
use http0::uri::PathAndQuery;
use http0::{Extensions, HeaderMap, Method};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::{FromStr, Utf8Error};

#[derive(Debug)]
/// An HTTP Request Type
pub struct Request<B = SdkBody> {
    body: B,
    uri: Uri,
    method: Method,
    extensions: Extensions,
    headers: Headers,
}

/// A Request URI
#[derive(Debug, Clone)]
pub struct Uri {
    as_string: String,
    parsed: http0::Uri,
}

impl Uri {
    /// Sets `endpoint` as the endpoint for a URL.
    ///
    /// An `endpoint` MUST contain a scheme and authority.
    /// An `endpoint` MAY contain a port and path.
    ///
    /// An `endpoint` MUST NOT contain a query
    pub fn set_endpoint(&mut self, endpoint: &str) -> Result<(), HttpError> {
        let endpoint: http0::Uri = endpoint.parse().map_err(HttpError::invalid_uri)?;
        let endpoint = endpoint.into_parts();
        let authority = endpoint
            .authority
            .ok_or_else(|| HttpError::new("endpoint must contain authority"))?;
        let scheme = endpoint
            .scheme
            .ok_or_else(|| HttpError::new("endpoint must have scheme"))?;
        let new_uri = http0::Uri::builder()
            .authority(authority)
            .scheme(scheme)
            .path_and_query(merge_paths(endpoint.path_and_query, &self.parsed).as_ref())
            .build()
            .map_err(HttpError::new)?;
        self.as_string = new_uri.to_string();
        self.parsed = new_uri;
        Ok(())
    }
}

fn merge_paths(endpoint_path: Option<PathAndQuery>, uri: &http0::Uri) -> Cow<'_, str> {
    let uri_path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("");
    let endpoint_path = match endpoint_path {
        None => return Cow::Borrowed(uri_path_and_query),
        Some(path) => path,
    };
    if let Some(query) = endpoint_path.query() {
        tracing::warn!(query = %query, "query specified in endpoint will be ignored during endpoint resolution");
    }
    let endpoint_path = endpoint_path.path();
    if endpoint_path.is_empty() {
        Cow::Borrowed(uri_path_and_query)
    } else {
        let ep_no_slash = endpoint_path.strip_suffix('/').unwrap_or(endpoint_path);
        let uri_path_no_slash = uri_path_and_query
            .strip_prefix('/')
            .unwrap_or(uri_path_and_query);
        Cow::Owned(format!("{}/{}", ep_no_slash, uri_path_no_slash))
    }
}

impl TryFrom<String> for Uri {
    type Error = HttpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parsed = value.parse().map_err(HttpError::invalid_uri)?;
        Ok(Uri {
            as_string: value,
            parsed,
        })
    }
}

impl<'a> TryFrom<&'a str> for Uri {
    type Error = HttpError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl From<http0::Uri> for Uri {
    fn from(value: http::Uri) -> Self {
        Self {
            as_string: value.to_string(),
            parsed: value,
        }
    }
}

impl<B> TryInto<http0::Request<B>> for Request<B> {
    type Error = HttpError;

    fn try_into(self) -> Result<http::Request<B>, Self::Error> {
        self.into_http02x()
    }
}

impl<B> Request<B> {
    /// Converts this request into an http 0.x request.
    ///
    /// Depending on the internal storage type, this operation may be free or it may have an internal
    /// cost.
    pub fn into_http02x(self) -> Result<http0::Request<B>, HttpError> {
        let mut req = http::Request::builder()
            .uri(self.uri.parsed)
            .method(self.method)
            .body(self.body)
            .expect("known valid");
        let mut headers = HeaderMap::new();
        headers.extend(
            self.headers
                .headers
                .into_iter()
                .map(|(k, v)| (k, v.into_http02x())),
        );
        *req.headers_mut() = headers;
        *req.extensions_mut() = self.extensions;
        Ok(req)
    }

    /// Update the body of this request to be a new body.
    pub fn map<U>(self, f: impl Fn(B) -> U) -> Request<U> {
        Request {
            body: f(self.body),
            uri: self.uri,
            method: self.method,
            extensions: self.extensions,
            headers: self.headers,
        }
    }

    /// Returns a GET request with no URI
    pub fn new(body: B) -> Self {
        Self {
            body,
            uri: Uri::from(http0::Uri::from_static("/")),
            method: Method::GET,
            extensions: Default::default(),
            headers: Default::default(),
        }
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

    /// Returns the method associated with this request
    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    /// Returns the URI associated with this request
    pub fn uri(&self) -> &str {
        &self.uri.as_string
    }

    /// Returns a mutable reference the the URI of this http::Request
    pub fn uri_mut(&mut self) -> &mut Uri {
        &mut self.uri
    }

    /// Sets the URI of this request
    pub fn set_uri<U>(&mut self, uri: U) -> Result<(), U::Error>
    where
        U: TryInto<Uri>,
    {
        let uri = uri.try_into()?;
        self.uri = uri;
        Ok(())
    }

    /// Adds an extension to the request extensions
    pub fn add_extension<T: Send + Sync + Clone + 'static>(&mut self, extension: T) {
        self.extensions.insert(extension);
    }
}

impl Request<SdkBody> {
    /// Attempts to clone this request
    ///
    /// If the body is cloneable, this will clone the request. Otherwise `None` will be returned
    pub fn try_clone(&self) -> Option<Self> {
        let body = self.body().try_clone()?;
        Some(Self {
            body,
            uri: self.uri.clone(),
            method: self.method.clone(),
            extensions: Extensions::new(),
            headers: self.headers.clone(),
        })
    }

    /// Replaces this requests body with [`SdkBody::taken()`]
    pub fn take_body(&mut self) -> SdkBody {
        std::mem::replace(self.body_mut(), SdkBody::taken())
    }

    /// Create a GET request to `/` with an empty body
    pub fn empty() -> Self {
        Self::new(SdkBody::empty())
    }

    /// Creates a GET request to `uri` with an empty body
    pub fn get(uri: impl AsRef<str>) -> Result<Self, HttpError> {
        let mut req = Self::new(SdkBody::empty());
        req.set_uri(uri.as_ref())?;
        Ok(req)
    }
}

impl<B> TryFrom<http0::Request<B>> for Request<B> {
    type Error = HttpError;

    fn try_from(value: http::Request<B>) -> Result<Self, Self::Error> {
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
                body,
                uri: parts.uri.into(),
                method: parts.method.clone(),
                extensions: parts.extensions,
                headers: Headers {
                    headers: string_safe_headers,
                },
            })
        }
    }
}

/// An immutable view of request headers
#[derive(Clone, Default, Debug)]
pub struct Headers {
    headers: HeaderMap<HeaderValue>,
}

impl<'a> IntoIterator for &'a Headers {
    type Item = (&'a str, &'a str);
    type IntoIter = HeadersIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        HeadersIter {
            inner: self.headers.iter(),
        }
    }
}

/// An Iterator over headers
pub struct HeadersIter<'a> {
    inner: Iter<'a, HeaderValue>,
}

impl<'a> Iterator for HeadersIter<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(k, v)| (k.as_str(), v.as_ref()))
    }
}

impl Headers {
    /// Create an empty header map
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value for a given key
    ///
    /// If multiple values are associated, the first value is returned
    /// See [HeaderMap::get]
    pub fn get(&self, key: impl AsRef<str>) -> Option<&str> {
        self.headers.get(key.as_ref()).map(|v| v.as_ref())
    }

    /// Returns all values for a given key
    pub fn get_all(&self, key: impl AsRef<str>) -> impl Iterator<Item = &str> {
        self.headers
            .get_all(key.as_ref())
            .iter()
            .map(|v| v.as_ref())
    }

    /// Returns an iterator over the headers
    pub fn iter(&self) -> HeadersIter<'_> {
        HeadersIter {
            inner: self.headers.iter(),
        }
    }

    /// Returns the total number of **values** stored in the map
    pub fn len(&self) -> usize {
        self.headers.len()
    }

    /// Returns true if there are no headers
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if this header is present
    pub fn contains_key(&self, key: &str) -> bool {
        self.headers.contains_key(key)
    }

    /// Insert a value into the headers structure.
    ///
    /// This will *replace* any existing value for this key. Returns the previous associated value if any.
    ///
    /// # Panics
    /// If the key or value are not valid ascii, this function will panic.
    pub fn insert(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Option<String> {
        self.try_insert(key, value)
            .expect("HeaderName or HeaderValue was invalid")
    }

    /// Insert a value into the headers structure.
    ///
    /// This will *replace* any existing value for this key. Returns the previous associated value if any.
    ///
    /// If the key or value are not valid ascii, an error is returned
    pub fn try_insert(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Result<Option<String>, HttpError> {
        let key = header_name(key.into_maybe_static()?)?;
        let value = header_value(value.into_maybe_static()?)?;
        Ok(self
            .headers
            .insert(key, value)
            .map(|old_value| old_value.into()))
    }

    /// Appends a value to a given key
    ///
    /// If the key or value are NOT valid ascii, an error is returned
    pub fn try_append(
        &mut self,
        key: impl AsHeaderComponent,
        value: impl AsHeaderComponent,
    ) -> Result<bool, HttpError> {
        let key = header_name(key.into_maybe_static()?)?;
        let value = header_value(value.into_maybe_static()?)?;
        Ok(self.headers.append(key, value))
    }

    /// Removes all headers with a given key
    ///
    /// If there are multiple entries for this key, the first entry is returned
    pub fn remove(&mut self, key: &str) -> Option<HeaderValue> {
        self.headers.remove(key)
    }

    /// Appends a value to a given key
    ///
    /// # Panics
    /// If the key or value are NOT valid ascii, this function will panic
    pub fn append(&mut self, key: impl AsHeaderComponent, value: impl AsHeaderComponent) -> bool {
        self.try_append(key, value)
            .expect("HeaderName or HeaderValue was invalid")
    }
}

use sealed::AsHeaderComponent;

mod sealed {
    use super::*;
    /// Trait defining things that may be converted into a header component (name or value)
    pub trait AsHeaderComponent {
        /// If the component can be represented as a Cow<'static, str>, return it
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError>;

        /// If a component is already internally represented as a `http02x::HeaderName`, return it
        fn repr_as_http02x_header_name(self) -> Result<http0::HeaderName, Self>
        where
            Self: Sized,
        {
            Err(self)
        }
    }

    impl AsHeaderComponent for &'static str {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Borrowed(self))
        }
    }

    impl AsHeaderComponent for String {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Owned(self))
        }
    }

    impl AsHeaderComponent for Cow<'static, str> {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(self)
        }
    }

    impl AsHeaderComponent for http0::HeaderValue {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(Cow::Owned(
                std::str::from_utf8(self.as_bytes())
                    .map_err(HttpError::header_was_not_a_string)?
                    .to_string(),
            ))
        }
    }

    impl AsHeaderComponent for http0::HeaderName {
        fn into_maybe_static(self) -> Result<MaybeStatic, HttpError> {
            Ok(self.to_string().into())
        }

        fn repr_as_http02x_header_name(self) -> Result<http0::HeaderName, Self>
        where
            Self: Sized,
        {
            Ok(self)
        }
    }
}

mod header_value {
    use super::http0;
    use std::str::Utf8Error;

    /// HeaderValue type
    ///
    /// **Note**: Unlike `HeaderValue` in `http`, this only supports UTF-8 header values
    #[derive(Debug, Clone)]
    pub struct HeaderValue {
        _private: http0::HeaderValue,
    }

    impl HeaderValue {
        pub(crate) fn from_http02x(value: http0::HeaderValue) -> Result<Self, Utf8Error> {
            let _ = std::str::from_utf8(value.as_bytes())?;
            Ok(Self { _private: value })
        }

        pub(crate) fn into_http02x(self) -> http0::HeaderValue {
            self._private
        }
    }

    impl AsRef<str> for HeaderValue {
        fn as_ref(&self) -> &str {
            std::str::from_utf8(self._private.as_bytes())
                .expect("unreachable—only strings may be stored")
        }
    }

    impl From<HeaderValue> for String {
        fn from(value: HeaderValue) -> Self {
            value.as_ref().to_string()
        }
    }
}

use crate::box_error::BoxError;
pub use header_value::HeaderValue;

impl HeaderValue {
    /// Returns the string representation of this header value
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl FromStr for HeaderValue {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HeaderValue::try_from(s.to_string())
    }
}

impl TryFrom<String> for HeaderValue {
    type Error = HttpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(HeaderValue::from_http02x(
            http0::HeaderValue::try_from(value).map_err(HttpError::invalid_header_value)?,
        )
        .expect("input was a string"))
    }
}

type MaybeStatic = Cow<'static, str>;

#[derive(Debug)]
/// An error occurred constructing an Http Request.
///
/// This is normally due to configuration issues, internal SDK bugs, or other user error.
pub struct HttpError(BoxError);

impl HttpError {
    // TODO(httpRefactor): Add better error internals
    fn new<E: Into<Box<dyn Error + Send + Sync + 'static>>>(err: E) -> Self {
        HttpError(err.into())
    }

    fn invalid_header_value(err: InvalidHeaderValue) -> Self {
        Self(err.into())
    }

    fn header_was_not_a_string(err: Utf8Error) -> Self {
        Self(err.into())
    }

    fn invalid_header_name(err: InvalidHeaderName) -> Self {
        Self(err.into())
    }

    fn invalid_uri(err: InvalidUri) -> Self {
        Self(err.into())
    }
}

impl Display for HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "an error occurred creating an HTTP Request")
    }
}

impl Error for HttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.0.as_ref())
    }
}

fn header_name(name: impl AsHeaderComponent) -> Result<http0::HeaderName, HttpError> {
    name.repr_as_http02x_header_name().or_else(|name| {
        name.into_maybe_static().and_then(|cow| {
            if cow.chars().any(|c| c.is_uppercase()) {
                return Err(HttpError::new("Header names must be all lower case"));
            }
            match cow {
                Cow::Borrowed(staticc) => Ok(http0::HeaderName::from_static(staticc)),
                Cow::Owned(s) => {
                    http0::HeaderName::try_from(s).map_err(HttpError::invalid_header_name)
                }
            }
        })
    })
}

fn header_value(value: MaybeStatic) -> Result<HeaderValue, HttpError> {
    let header = match value {
        Cow::Borrowed(b) => http0::HeaderValue::from_static(b),
        Cow::Owned(s) => {
            http0::HeaderValue::try_from(s).map_err(HttpError::invalid_header_value)?
        }
    };
    HeaderValue::from_http02x(header).map_err(HttpError::new)
}

#[cfg(test)]
mod test {
    use crate::client::orchestrator::HttpRequest;
    use aws_smithy_types::body::SdkBody;
    use http::header::{AUTHORIZATION, CONTENT_LENGTH};
    use http::{HeaderValue, Uri};

    #[test]
    fn headers_can_be_any_string() {
        let _: HeaderValue = "😹".parse().expect("can be any string");
        let _: HeaderValue = "abcd".parse().expect("can be any string");
        let _ = "a\nb"
            .parse::<HeaderValue>()
            .expect_err("cannot contain control characters");
    }

    #[test]
    fn non_ascii_requests() {
        let request = http::Request::builder()
            .header("k", "😹")
            .body(SdkBody::empty())
            .unwrap();
        let request: HttpRequest = request
            .try_into()
            .expect("failed to convert a non-string header");
        assert_eq!(request.headers().get("k"), Some("😹"))
    }

    #[test]
    fn request_can_be_created() {
        let req = http::Request::builder()
            .uri("http://foo.com")
            .body(SdkBody::from("hello"))
            .unwrap();
        let mut req = super::Request::try_from(req).unwrap();
        req.headers_mut().insert("a", "b");
        assert_eq!(req.headers().get("a").unwrap(), "b");
        req.headers_mut().append("a", "c");
        assert_eq!(req.headers().get("a").unwrap(), "b");
        let http0 = req.into_http02x().unwrap();
        assert_eq!(http0.uri(), "http://foo.com");
    }

    #[test]
    fn uri_mutations() {
        let req = http::Request::builder()
            .uri("http://foo.com")
            .body(SdkBody::from("hello"))
            .unwrap();
        let mut req = super::Request::try_from(req).unwrap();
        assert_eq!(req.uri(), "http://foo.com/");
        req.set_uri("http://bar.com").unwrap();
        assert_eq!(req.uri(), "http://bar.com");
        let http0 = req.into_http02x().unwrap();
        assert_eq!(http0.uri(), "http://bar.com");
    }

    #[test]
    #[should_panic]
    fn header_panics() {
        let req = http::Request::builder()
            .uri("http://foo.com")
            .body(SdkBody::from("hello"))
            .unwrap();
        let mut req = super::Request::try_from(req).unwrap();
        let _ = req
            .headers_mut()
            .try_insert("a\nb", "a\nb")
            .expect_err("invalid header");
        let _ = req.headers_mut().insert("a\nb", "a\nb");
    }

    #[test]
    fn try_clone_clones_all_data() {
        let request = ::http::Request::builder()
            .uri(Uri::from_static("https://www.amazon.com"))
            .method("POST")
            .header(CONTENT_LENGTH, 456)
            .header(AUTHORIZATION, "Token: hello")
            .body(SdkBody::from("hello world!"))
            .expect("valid request");
        let request: super::Request = request.try_into().unwrap();
        let cloned = request.try_clone().expect("request is cloneable");

        assert_eq!("https://www.amazon.com/", cloned.uri());
        assert_eq!("POST", cloned.method());
        assert_eq!(2, cloned.headers().len());
        assert_eq!("Token: hello", cloned.headers().get(AUTHORIZATION).unwrap(),);
        assert_eq!("456", cloned.headers().get(CONTENT_LENGTH).unwrap());
        assert_eq!("hello world!".as_bytes(), cloned.body().bytes().unwrap());
    }
}
