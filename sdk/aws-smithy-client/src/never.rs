/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Test connectors that never return data

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use http::Uri;
use tower::BoxError;

use aws_smithy_async::future::never::Never;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;

use crate::erase::boxclone::BoxFuture;

/// A service that will never return whatever it is you want
///
/// Returned futures will return Pending forever
#[non_exhaustive]
#[derive(Debug)]
pub struct NeverService<Req, Resp, Err> {
    _resp: PhantomData<(Req, Resp, Err)>,
    invocations: Arc<AtomicUsize>,
}

impl<Req, Resp, Err> Clone for NeverService<Req, Resp, Err> {
    fn clone(&self) -> Self {
        Self {
            _resp: Default::default(),
            invocations: self.invocations.clone(),
        }
    }
}

impl<Req, Resp, Err> Default for NeverService<Req, Resp, Err> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Req, Resp, Err> NeverService<Req, Resp, Err> {
    /// Create a new NeverService
    pub fn new() -> Self {
        NeverService {
            _resp: Default::default(),
            invocations: Default::default(),
        }
    }

    /// Returns the number of invocations made to this service
    pub fn num_calls(&self) -> usize {
        self.invocations.load(Ordering::SeqCst)
    }
}

/// A Connector that can be use with [`Client`](crate::Client) that never returns a response.
pub type NeverConnector =
    NeverService<http::Request<SdkBody>, http::Response<SdkBody>, ConnectorError>;

/// A service where the underlying TCP connection never connects.
pub type NeverConnected = NeverService<Uri, stream::EmptyStream, BoxError>;

/// Streams that never return data
pub(crate) mod stream {
    use std::io::Error;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

    /// A stream that will never return or accept any data
    #[non_exhaustive]
    #[derive(Debug, Default)]
    pub struct EmptyStream;

    impl EmptyStream {
        pub fn new() -> Self {
            Self
        }
    }

    impl AsyncRead for EmptyStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }
    }

    impl AsyncWrite for EmptyStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<Result<usize, Error>> {
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            Poll::Pending
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            Poll::Pending
        }
    }
}

/// A service that will connect but never send any data
#[derive(Clone, Debug, Default)]
pub struct NeverReplies;
impl NeverReplies {
    /// Create a new NeverReplies service
    pub fn new() -> Self {
        Self
    }
}

impl tower::Service<Uri> for NeverReplies {
    type Response = stream::EmptyStream;
    type Error = BoxError;
    type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Uri) -> Self::Future {
        std::future::ready(Ok(stream::EmptyStream::new()))
    }
}

impl<Req, Resp, Err> tower::Service<Req> for NeverService<Req, Resp, Err> {
    type Response = Resp;
    type Error = Err;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Req) -> Self::Future {
        self.invocations.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Never::new().await;
            unreachable!()
        })
    }
}
