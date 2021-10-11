/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Test connectors that never return data

use http::Uri;

use smithy_async::future::never::Never;

use std::marker::PhantomData;

use std::task::{Context, Poll};
use tokio::net::TcpStream;

use crate::erase::boxclone::BoxFuture;
use smithy_http::result::ConnectorError;
use tower::BoxError;

/// A service that will never return whatever it is you want
///
/// Returned futures will return Pending forever
#[non_exhaustive]
#[derive(Debug)]
pub struct NeverService<R> {
    _resp: PhantomData<R>,
}

impl<R> Clone for NeverService<R> {
    fn clone(&self) -> Self {
        Self {
            _resp: Default::default(),
        }
    }
}

impl<R> Default for NeverService<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> NeverService<R> {
    /// Create a new NeverService
    pub fn new() -> Self {
        NeverService {
            _resp: Default::default(),
        }
    }
}

/// Streams that never return data
mod stream {
    use hyper::client::connect::{Connected, Connection};
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

    impl Connection for EmptyStream {
        fn connected(&self) -> Connected {
            Connected::new()
        }
    }
}

/// A service where the underlying TCP connection never connects
pub type NeverConnected = NeverService<TcpStream>;

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

impl<Req, Resp> tower::Service<Req> for NeverService<Resp> {
    type Response = Resp;
    type Error = ConnectorError;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Req) -> Self::Future {
        Box::pin(async move {
            Never::new().await;
            unreachable!()
        })
    }
}
