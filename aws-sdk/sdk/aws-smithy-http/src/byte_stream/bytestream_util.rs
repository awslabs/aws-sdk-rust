/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use bytes::Bytes;
use futures_core::{ready, Stream};
use http::HeaderMap;
use http_body::{Body, SizeHint};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io;
use tokio_util::io::ReaderStream;

/// An HTTP Body designed to wrap files
///
/// PathBody is a three-phase HTTP body designed to wrap files with three specific features:
/// 1. The underlying file is wrapped with StreamReader to implement HTTP body
/// 2. It can be constructed directly from a path so it's easy to use during retries
/// 3. Provide size hint
pub struct PathBody {
    state: State,
    len: u64,
}

impl PathBody {
    pub fn from_path(path: &Path, len: u64) -> Self {
        PathBody {
            state: State::Unloaded(path.to_path_buf()),
            len,
        }
    }
    pub fn from_file(file: File, len: u64) -> Self {
        PathBody {
            state: State::Loaded(ReaderStream::new(file)),
            len,
        }
    }
}

enum State {
    Unloaded(PathBuf),
    Loading(Pin<Box<dyn Future<Output = io::Result<File>> + Send + Sync + 'static>>),
    Loaded(tokio_util::io::ReaderStream<File>),
}

impl Body for PathBody {
    type Data = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        loop {
            match self.state {
                State::Unloaded(ref path_buf) => {
                    let buf = path_buf.clone();
                    self.state = State::Loading(Box::pin(async move {
                        let file = tokio::fs::File::open(&buf).await?;
                        Ok(file)
                    }));
                }
                State::Loading(ref mut future) => {
                    match ready!(Pin::new(future).poll(cx)) {
                        Ok(file) => {
                            self.state = State::Loaded(ReaderStream::new(file));
                        }
                        Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    };
                }
                State::Loaded(ref mut stream) => {
                    return match ready!(Pin::new(stream).poll_next(cx)) {
                        Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes))),
                        None => Poll::Ready(None),
                        Some(Err(e)) => Poll::Ready(Some(Err(e.into()))),
                    }
                }
            };
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        // fast path end-stream for empty files
        self.len == 0
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len)
    }
}
