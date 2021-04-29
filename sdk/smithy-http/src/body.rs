/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use bytes::Bytes;
use http::{HeaderMap, HeaderValue};
use http_body::{Body, SizeHint};
use pin_project::pin_project;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::task::{Context, Poll};

pub type Error = Box<dyn StdError + Send + Sync>;

/// SdkBody type
///
/// This is the Body used for dispatching all HTTP Requests.
/// For handling responses, the type of the body will be controlled
/// by the HTTP stack.
///
/// TODO: Consider renaming to simply `Body`, although I'm concerned about naming headaches
/// between hyper::Body and our Body
#[pin_project]
#[derive(Debug)]
pub struct SdkBody(#[pin] Inner);

type BoxBody = http_body::combinators::BoxBody<Bytes, Error>;

#[pin_project(project = InnerProj)]
enum Inner {
    Once(#[pin] Option<Bytes>),
    Streaming(#[pin] hyper::Body),
    Dyn(#[pin] BoxBody),

    /// When a streaming body is transferred out to a stream parser, the body is replaced with
    /// `Taken`. This will return an Error when polled. Attempting to read data out of a `Taken`
    /// Body is a bug.
    Taken,
}

impl Debug for Inner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self {
            i @ Inner::Once(_) | i @ Inner::Streaming(_) | i @ Inner::Taken => i.fmt(f),
            Inner::Dyn(_) => write!(f, "BoxBody"),
        }
    }
}

impl SdkBody {
    /// Construct an SdkBody from a Boxed implementation of http::Body
    pub fn from_dyn(body: BoxBody) -> Self {
        Self(Inner::Dyn(body))
    }

    pub fn taken() -> Self {
        Self(Inner::Taken)
    }

    fn poll_inner(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Error>>> {
        match self.project().0.project() {
            InnerProj::Once(ref mut opt) => {
                let data = opt.take();
                match data {
                    Some(bytes) if bytes.is_empty() => Poll::Ready(None),
                    Some(bytes) => Poll::Ready(Some(Ok(bytes))),
                    None => Poll::Ready(None),
                }
            }
            InnerProj::Streaming(body) => body.poll_data(cx).map_err(|e| e.into()),
            InnerProj::Dyn(box_body) => box_body.poll_data(cx),
            InnerProj::Taken => {
                Poll::Ready(Some(Err("A `Taken` body should never be polled".into())))
            }
        }
    }

    /// If possible, return a reference to this body as `&[u8]`
    ///
    /// If this SdkBody is NOT streaming, this will return the byte slab
    /// If this SdkBody is streaming, this will return `None`
    pub fn bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            Inner::Once(Some(b)) => Some(&b),
            Inner::Once(None) => Some(&[]),
            _ => None,
        }
    }

    pub fn try_clone(&self) -> Option<Self> {
        match &self.0 {
            Inner::Once(bytes) => Some(SdkBody(Inner::Once(bytes.clone()))),
            _ => None,
        }
    }
}

impl From<&str> for SdkBody {
    fn from(s: &str) -> Self {
        SdkBody(Inner::Once(Some(Bytes::copy_from_slice(s.as_bytes()))))
    }
}

impl From<Bytes> for SdkBody {
    fn from(bytes: Bytes) -> Self {
        SdkBody(Inner::Once(Some(bytes)))
    }
}

impl From<hyper::Body> for SdkBody {
    fn from(body: hyper::Body) -> Self {
        SdkBody(Inner::Streaming(body))
    }
}

impl From<Vec<u8>> for SdkBody {
    fn from(data: Vec<u8>) -> Self {
        Self::from(Bytes::from(data))
    }
}

impl http_body::Body for SdkBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.poll_inner(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap<HeaderValue>>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        match &self.0 {
            Inner::Once(None) => true,
            Inner::Once(Some(bytes)) => bytes.is_empty(),
            Inner::Streaming(hyper_body) => hyper_body.is_end_stream(),
            Inner::Dyn(box_body) => box_body.is_end_stream(),
            Inner::Taken => true,
        }
    }

    fn size_hint(&self) -> SizeHint {
        match &self.0 {
            Inner::Once(None) => SizeHint::with_exact(0),
            Inner::Once(Some(bytes)) => SizeHint::with_exact(bytes.len() as u64),
            Inner::Streaming(hyper_body) => hyper_body.size_hint(),
            Inner::Dyn(box_body) => box_body.size_hint(),
            Inner::Taken => SizeHint::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::body::SdkBody;
    use http_body::Body;
    use std::pin::Pin;

    #[test]
    fn valid_size_hint() {
        assert_eq!(SdkBody::from("hello").size_hint().exact(), Some(5));
        assert_eq!(SdkBody::from("").size_hint().exact(), Some(0));
    }

    #[test]
    fn valid_eos() {
        assert_eq!(SdkBody::from("hello").is_end_stream(), false);
        assert_eq!(SdkBody::from("").is_end_stream(), true);
    }

    #[tokio::test]
    async fn http_body_consumes_data() {
        let mut body = SdkBody::from("hello!");
        let mut body = Pin::new(&mut body);
        let data = body.data().await;
        assert!(data.is_some());
        let data = body.data().await;
        assert!(data.is_none());
    }

    #[tokio::test]
    async fn empty_body_returns_none() {
        // Its important to avoid sending empty chunks of data to avoid H2 data frame problems
        let mut body = SdkBody::from("");
        let mut body = Pin::new(&mut body);
        let data = body.data().await;
        assert!(data.is_none());
    }
}
