/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use bytes::Bytes;
use futures_core::{ready, Stream};
use http::HeaderMap;
use http_body::{Body, SizeHint};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io;
use tokio_util::io::ReaderStream;

use crate::body::SdkBody;

use super::{ByteStream, Error};

// 4KB corresponds to the default buffer size used by Tokio's ReaderStream
const DEFAULT_BUFFER_SIZE: usize = 4096;

/// An HTTP Body designed to wrap files
///
/// PathBody is a three-phase HTTP body designed to wrap files with three specific features:
/// 1. The underlying file is wrapped with StreamReader to implement HTTP body
/// 2. It can be constructed directly from a path so it's easy to use during retries
/// 3. Provide size hint
struct PathBody {
    state: State,
    file_size: u64,
    buffer_size: usize,
}

impl PathBody {
    fn from_path(path_buf: PathBuf, file_size: u64, buffer_size: usize) -> Self {
        PathBody {
            state: State::Unloaded(path_buf),
            file_size,
            buffer_size,
        }
    }
    fn from_file(file: File, file_size: u64, buffer_size: usize) -> Self {
        PathBody {
            state: State::Loaded(ReaderStream::with_capacity(file, buffer_size)),
            file_size,
            buffer_size,
        }
    }
}

/// Builder for creating [`ByteStreams`](crate::byte_stream::ByteStream) from a file/path, with full control over advanced options.
///
/// Example usage:
/// ```no_run
/// # #[cfg(feature = "rt-tokio")]
/// # {
/// use aws_smithy_http::byte_stream::ByteStream;
/// use std::path::Path;
/// struct GetObjectInput {
///     body: ByteStream
/// }
///
/// async fn bytestream_from_file() -> GetObjectInput {
///     let bytestream = ByteStream::read_from()
///         .path("docs/some-large-file.csv")
///         // Specify the size of the buffer used to read the file (in bytes, default is 4096)
///         .buffer_size(32_784)
///         // Specify the length of the file used (skips an additional call to retrieve the size)
///         .file_size(123_456)
///         .build()
///         .await
///         .expect("valid path");
///     GetObjectInput { body: bytestream }
/// }
/// # }
/// ```
pub struct FsBuilder {
    file: Option<tokio::fs::File>,
    path: Option<PathBuf>,
    file_size: Option<u64>,
    buffer_size: usize,
}

impl Default for FsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FsBuilder {
    /// Create a new [`FsBuilder`] (using a default read buffer of 4096 bytes).
    ///
    /// You must then call either [`file`](FsBuilder::file) or [`path`](FsBuilder::path) to specify what to read from.
    pub fn new() -> Self {
        FsBuilder {
            file: None,
            path: None,
            file_size: None,
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }

    /// Sets the path to read from.
    ///
    /// NOTE: The resulting ByteStream (after calling [build](FsBuilder::build)) will be retryable.
    /// The returned ByteStream will provide a size hint when used as an HTTP body.
    /// If the request fails, the read will begin again by reloading the file handle.
    pub fn path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Sets the file to read from.
    ///
    /// NOTE: The resulting ByteStream (after calling [build](FsBuilder::build)) will not be a retryable ByteStream.
    /// For a ByteStream that can be retried in the case of upstream failures, use [`FsBuilder::path`](FsBuilder::path).
    pub fn file(mut self, file: tokio::fs::File) -> Self {
        self.file = Some(file);
        self
    }

    /// Specify the length of the file to read (in bytes).
    ///
    /// By pre-specifying the length of the file, this API skips an additional call to retrieve the size from file-system metadata.
    pub fn file_size(mut self, file_size: u64) -> Self {
        self.file_size = Some(file_size);
        self
    }

    /// Specify the size of the buffer used to read the file (in bytes).
    ///
    /// Increasing the read buffer capacity to higher values than the default (4096 bytes) can result in a large reduction
    /// in CPU usage, at the cost of memory increase.
    pub fn buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Returns a [`ByteStream`](crate::byte_stream::ByteStream) from this builder.
    /// NOTE: If both [`file`](FsBuilder::file) and [`path`](FsBuilder::path) have been called for this FsBuilder, `build` will
    /// read from the path specified by [`path`](FsBuilder::path).
    ///
    /// # Panics
    ///
    /// Panics if neither of the `file` or`path` setters were called.
    pub async fn build(self) -> Result<ByteStream, Error> {
        let buffer_size = self.buffer_size;

        if let Some(path) = self.path {
            let path_buf = path.to_path_buf();
            let file_size = self.file_size.unwrap_or(
                tokio::fs::metadata(path)
                    .await
                    .map_err(|err| Error(err.into()))?
                    .len(),
            );

            let body_loader = move || {
                SdkBody::from_dyn(http_body::combinators::BoxBody::new(PathBody::from_path(
                    path_buf.clone(),
                    file_size,
                    buffer_size,
                )))
            };
            Ok(ByteStream::new(SdkBody::retryable(body_loader)))
        } else if let Some(file) = self.file {
            let file_size = self.file_size.unwrap_or(
                file.metadata()
                    .await
                    .map_err(|err| Error(err.into()))?
                    .len(),
            );

            let body = SdkBody::from_dyn(http_body::combinators::BoxBody::new(
                PathBody::from_file(file, file_size, buffer_size),
            ));

            Ok(ByteStream::new(body))
        } else {
            panic!("FsBuilder constructed without a file or a path")
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
                            self.state =
                                State::Loaded(ReaderStream::with_capacity(file, self.buffer_size));
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
        self.file_size == 0
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.file_size)
    }
}
