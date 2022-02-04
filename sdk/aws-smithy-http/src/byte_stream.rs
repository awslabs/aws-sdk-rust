/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
//! ByteStream Abstractions
//!
//! When the SDK returns streaming binary data, the inner Http Body is wrapped in [ByteStream](crate::byte_stream::ByteStream). ByteStream provides misuse-resistant
//! primitives to make it easier to handle common patterns with streaming data.
//!
//! # Examples
//!
//! ### Writing a ByteStream into a file:
//! ```no_run
//! use aws_smithy_http::byte_stream::ByteStream;
//! use std::error::Error;
//! use tokio::fs::File;
//! use tokio::io::AsyncWriteExt;
//! struct SynthesizeSpeechOutput {
//!     audio_stream: ByteStream,
//! }
//!
//! async fn audio_to_file(
//!     output: SynthesizeSpeechOutput,
//! ) -> Result<(), Box<dyn Error + Send + Sync>> {
//!     let mut buf = output.audio_stream.collect().await?;
//!     let mut file = File::open("audio.mp3").await?;
//!     file.write_all_buf(&mut buf).await?;
//!     file.flush().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Converting a ByteStream into Bytes
//! ```no_run
//! use bytes::Bytes;
//! use aws_smithy_http::byte_stream::ByteStream;
//! use std::error::Error;
//! struct SynthesizeSpeechOutput {
//!     audio_stream: ByteStream,
//! }
//! async fn load_audio(
//!     output: SynthesizeSpeechOutput,
//! ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
//!     Ok(output.audio_stream.collect().await?.into_bytes())
//! }
//! ```
//!
//! ### Stream a ByteStream into a file
//! The previous example is recommended in cases where loading the entire file into memory first is desirable. For extremely large
//! files, you may wish to stream the data directly to the file system, chunk by chunk. This is posible using the `futures::Stream` implementation.
//!
//! ```no_run
//! use bytes::{Buf, Bytes};
//! use aws_smithy_http::byte_stream::ByteStream;
//! use std::error::Error;
//! use tokio::fs::File;
//! use tokio::io::AsyncWriteExt;
//! use tokio_stream::StreamExt;
//! struct SynthesizeSpeechOutput {
//!     audio_stream: ByteStream,
//! }
//!
//! async fn audio_to_file(
//!     output: SynthesizeSpeechOutput,
//! ) -> Result<(), Box<dyn Error + Send + Sync>> {
//!     let mut file = File::open("audio.mp3").await?;
//!     let mut stream = output.audio_stream;
//!     while let Some(bytes) = stream.next().await {
//!         let bytes: Bytes = bytes?;
//!         file.write_all(&bytes).await?;
//!     }
//!     file.flush().await?;
//!     Ok(())
//! }
//! ```
//!
//! ### Create a ByteStream from a file
//!
//! _Note: This is only available with `rt-tokio` enabled._
//!
//! ```no_run
//! # #[cfg(feature = "rt-tokio")]
//! {
//! use aws_smithy_http::byte_stream::ByteStream;
//! use std::path::Path;
//! struct GetObjectInput {
//!   body: ByteStream
//! }
//!
//! async fn bytestream_from_file() -> GetObjectInput {
//!     let bytestream = ByteStream::from_path("docs/some-large-file.csv")
//!         .await
//!         .expect("valid path");
//!     GetObjectInput { body: bytestream }
//! }
//! }
//! ```

use crate::body::SdkBody;
use bytes::Buf;
use bytes::Bytes;
use bytes_utils::SegmentedBuf;
use http_body::Body;
use pin_project::pin_project;
use std::error::Error as StdError;
use std::fmt::{Debug, Formatter};
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(feature = "rt-tokio")]
mod bytestream_util;

/// Stream of binary data
///
/// `ByteStream` wraps a stream of binary data for ease of use.
///
/// ## Getting data out of a `ByteStream`
///
/// `ByteStream` provides two primary mechanisms for accessing the data:
/// 1. With `.collect()`:
/// [`.collect()`](crate::byte_stream::ByteStream::collect) reads the complete ByteStream into memory and stores it in `AggregatedBytes`,
/// a non-contiguous ByteBuffer.
///     ```no_run
///     use aws_smithy_http::byte_stream::{ByteStream, AggregatedBytes};
///     use aws_smithy_http::body::SdkBody;
///     use bytes::Buf;
///     async fn example() {
///        let stream = ByteStream::new(SdkBody::from("hello! This is some data"));
///        // Load data from the stream into memory:
///        let data = stream.collect().await.expect("error reading data");
///        // collect returns a `bytes::Buf`:
///        println!("first chunk: {:?}", data.chunk());
///     }
///     ```
/// 2. Via [`impl Stream`](futures_core::Stream):
///
///     _Note: An import of `StreamExt` is required to use `try_next()`._
///
///     For use-cases where holding the entire ByteStream in memory is unnecessary, use the
///     `Stream` implementation:
///     ```no_run
///     # mod crc32 {
///     #   pub struct Digest { }
///     #   impl Digest {
///     #       pub fn new() -> Self { Digest {} }
///     #       pub fn write(&mut self, b: &[u8]) { }
///     #       pub fn finish(&self) -> u64 { 6 }
///     #   }
///     # }
///     use aws_smithy_http::byte_stream::{ByteStream, AggregatedBytes, Error};
///     use aws_smithy_http::body::SdkBody;
///     use tokio_stream::StreamExt;
///
///     async fn example() -> Result<(), Error> {
///        let mut stream = ByteStream::from(vec![1, 2, 3, 4, 5, 99]);
///        let mut digest = crc32::Digest::new();
///        while let Some(bytes) = stream.try_next().await? {
///            digest.write(&bytes);
///        }
///        println!("digest: {}", digest.finish());
///        Ok(())
///     }
///     ```
///
/// ## Getting data into a ByteStream
/// ByteStreams can be created in one of three ways:
/// 1. **From in-memory binary data**: ByteStreams created from in-memory data are always retryable. Data
/// will be converted into `Bytes` enabling a cheap clone during retries.
///     ```no_run
///     use bytes::Bytes;
///     use aws_smithy_http::byte_stream::ByteStream;
///     let stream = ByteStream::from(vec![1,2,3]);
///     let stream = ByteStream::from(Bytes::from_static(b"hello!"));
///     ```
///
/// 2. **From a file**: ByteStreams created from a path can be retried. A new file descriptor will be opened if a retry occurs.
///     ```no_run
///     #[cfg(feature = "tokio-rt")]
///     # {
///     use aws_smithy_http::byte_stream::ByteStream;
///     let stream = ByteStream::from_path("big_file.csv");
///     # }
///     ```
///
/// 3. **From an `SdkBody` directly**: For more advanced / custom use cases, a ByteStream can be created directly
/// from an SdkBody. **When created from an SdkBody, care must be taken to ensure retriability.** An SdkBody is retryable
/// when constructed from in-memory data or when using [`SdkBody::retryable`](crate::body::SdkBody::retryable).
///     ```no_run
///     use aws_smithy_http::byte_stream::ByteStream;
///     use aws_smithy_http::body::SdkBody;
///     use bytes::Bytes;
///     let (mut tx, channel_body) = hyper::Body::channel();
///     // this will not be retryable because the SDK has no way to replay this stream
///     let stream = ByteStream::new(SdkBody::from(channel_body));
///     tx.send_data(Bytes::from_static(b"hello world!"));
///     tx.send_data(Bytes::from_static(b"hello again!"));
///     // NOTE! You must ensure that `tx` is dropped to ensure that EOF is sent
///     ```
///
#[pin_project]
#[derive(Debug)]
pub struct ByteStream(#[pin] Inner<SdkBody>);

impl ByteStream {
    pub fn new(body: SdkBody) -> Self {
        Self(Inner::new(body))
    }

    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self(Inner::new(SdkBody::from(Bytes::from_static(bytes))))
    }

    /// Consumes the ByteStream, returning the wrapped SdkBody
    // Backwards compatibility note: Because SdkBody has a dyn variant,
    // we will always be able to implement this method, even if we stop using
    // SdkBody as the internal representation
    pub fn into_inner(self) -> SdkBody {
        self.0.body
    }

    /// Read all the data from this `ByteStream` into memory
    ///
    /// If an error in the underlying stream is encountered, `ByteStreamError` is returned.
    ///
    /// Data is read into an `AggregatedBytes` that stores data non-contiguously as it was received
    /// over the network. If a contiguous slice is required, use `into_bytes()`.
    /// ```no_run
    /// use bytes::Bytes;
    /// use aws_smithy_http::body;
    /// use aws_smithy_http::body::SdkBody;
    /// use aws_smithy_http::byte_stream::{ByteStream, Error};
    /// async fn get_data() {
    ///     let stream = ByteStream::new(SdkBody::from("hello!"));
    ///     let data: Result<Bytes, Error> = stream.collect().await.map(|data| data.into_bytes());
    /// }
    /// ```
    pub async fn collect(self) -> Result<AggregatedBytes, Error> {
        self.0.collect().await.map_err(|err| Error(err))
    }

    /// Create a ByteStream that streams data from the filesystem
    ///
    /// This function creates a retryable ByteStream for a given `path`. The returned ByteStream
    /// will provide a size hint when used as an HTTP body. If the request fails, the read will
    /// begin again by reloading the file handle.
    ///
    /// ## Warning
    /// The contents of the file MUST not change during retries. The length & checksum of the file
    /// will be cached. If the contents of the file change, the operation will almost certainly fail.
    ///
    /// Furthermore, a partial write MAY seek in the file and resume from the previous location.
    ///
    /// # Examples
    /// ```no_run
    /// use aws_smithy_http::byte_stream::ByteStream;
    /// use std::path::Path;
    ///  async fn make_bytestream() -> ByteStream {
    ///     ByteStream::from_path("docs/rows.csv").await.expect("file should be readable")
    /// }
    /// ```
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub async fn from_path(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();
        let sz = tokio::fs::metadata(path)
            .await
            .map_err(|err| Error(err.into()))?
            .len();
        let body_loader = move || {
            SdkBody::from_dyn(http_body::combinators::BoxBody::new(
                bytestream_util::PathBody::from_path(path_buf.as_path(), sz),
            ))
        };
        Ok(ByteStream::new(SdkBody::retryable(body_loader)))
    }

    /// Create a ByteStream from a file
    ///
    /// NOTE: This will NOT result in a retryable ByteStream. For a ByteStream that can be retried in the case of
    /// upstream failures, use [`ByteStream::from_path`](ByteStream::from_path)
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub async fn from_file(file: tokio::fs::File) -> Result<Self, Error> {
        let sz = file
            .metadata()
            .await
            .map_err(|err| Error(err.into()))?
            .len();
        let body = SdkBody::from_dyn(http_body::combinators::BoxBody::new(
            bytestream_util::PathBody::from_file(file, sz),
        ));
        Ok(ByteStream::new(body))
    }
}

impl Default for ByteStream {
    fn default() -> Self {
        Self(Inner {
            body: SdkBody::from(""),
        })
    }
}

impl From<SdkBody> for ByteStream {
    fn from(inp: SdkBody) -> Self {
        ByteStream::new(inp)
    }
}

/// Construct a retryable ByteStream from [`bytes::Bytes`](bytes::Bytes)
impl From<Bytes> for ByteStream {
    fn from(input: Bytes) -> Self {
        ByteStream::new(SdkBody::from(input))
    }
}

/// Construct a retryable ByteStream from a `Vec<u8>`.
///
/// This will convert the `Vec<u8>` into [`bytes::Bytes`](bytes::Bytes) to enable efficient
/// retries.
impl From<Vec<u8>> for ByteStream {
    fn from(input: Vec<u8>) -> Self {
        Self::from(Bytes::from(input))
    }
}

impl From<hyper::Body> for ByteStream {
    fn from(input: hyper::Body) -> Self {
        ByteStream::new(SdkBody::from_dyn(
            input.map_err(|e| e.into_cause().unwrap()).boxed(),
        ))
    }
}

#[derive(Debug)]
pub struct Error(Box<dyn StdError + Send + Sync + 'static>);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.0.as_ref() as _)
    }
}

impl futures_core::stream::Stream for ByteStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().0.poll_next(cx).map_err(|e| Error(e))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Non-contiguous Binary Data Storage
///
/// When data is read from the network, it is read in a sequence of chunks that are not in
/// contiguous memory. [`AggregatedBytes`](crate::byte_stream::AggregatedBytes) provides a view of
/// this data via [`impl Buf`](bytes::Buf) or it can be copied into contiguous storage with
/// [`.into_bytes()`](crate::byte_stream::AggregatedBytes::into_bytes).
#[derive(Debug, Clone)]
pub struct AggregatedBytes(SegmentedBuf<Bytes>);

impl AggregatedBytes {
    /// Convert this buffer into [`Bytes`](bytes::Bytes)
    ///
    /// # Why does this consume `self`?
    /// Technically, [`copy_to_bytes`](bytes::Buf::copy_to_bytes) can be called without ownership of self. However, since this
    /// mutates the underlying buffer such that no data is remaining, it is more misuse resistant to
    /// prevent the caller from attempting to reread the buffer.
    ///
    /// If the caller only holds a mutable reference, they may use [`copy_to_bytes`](bytes::Buf::copy_to_bytes)
    /// directly on `AggregatedBytes`.
    pub fn into_bytes(mut self) -> Bytes {
        self.0.copy_to_bytes(self.0.remaining())
    }
}

impl Buf for AggregatedBytes {
    // Forward all methods that SegmentedBuf has custom implementations of.
    fn remaining(&self) -> usize {
        self.0.remaining()
    }

    fn chunk(&self) -> &[u8] {
        self.0.chunk()
    }

    fn chunks_vectored<'a>(&'a self, dst: &mut [IoSlice<'a>]) -> usize {
        self.0.chunks_vectored(dst)
    }

    fn advance(&mut self, cnt: usize) {
        self.0.advance(cnt)
    }

    fn copy_to_bytes(&mut self, len: usize) -> Bytes {
        self.0.copy_to_bytes(len)
    }
}

#[pin_project]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Inner<B> {
    #[pin]
    body: B,
}

impl<B> Inner<B> {
    pub fn new(body: B) -> Self {
        Self { body }
    }
    pub async fn collect(self) -> Result<AggregatedBytes, B::Error>
    where
        B: http_body::Body<Data = Bytes>,
    {
        let mut output = SegmentedBuf::new();
        let body = self.body;
        crate::pin_mut!(body);
        while let Some(buf) = body.data().await {
            output.push(buf?);
        }
        Ok(AggregatedBytes(output))
    }
}

impl<B> futures_core::stream::Stream for Inner<B>
where
    B: http_body::Body,
{
    type Item = Result<Bytes, B::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.project().body.poll_data(cx) {
            Poll::Ready(Some(Ok(mut data))) => {
                let len = data.chunk().len();
                let bytes = data.copy_to_bytes(len);
                Poll::Ready(Some(Ok(bytes)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size_hint = http_body::Body::size_hint(&self.body);
        (
            size_hint.lower() as usize,
            size_hint.upper().map(|u| u as usize),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::byte_stream::Inner;
    use bytes::Bytes;

    #[tokio::test]
    async fn read_from_string_body() {
        let body = hyper::Body::from("a simple body");
        assert_eq!(
            Inner::new(body)
                .collect()
                .await
                .expect("no errors")
                .into_bytes(),
            Bytes::from("a simple body")
        );
    }

    #[tokio::test]
    async fn read_from_channel_body() {
        let (mut sender, body) = hyper::Body::channel();
        let byte_stream = Inner::new(body);
        tokio::spawn(async move {
            sender.send_data(Bytes::from("data 1")).await.unwrap();
            sender.send_data(Bytes::from("data 2")).await.unwrap();
            sender.send_data(Bytes::from("data 3")).await.unwrap();
        });
        assert_eq!(
            byte_stream.collect().await.expect("no errors").into_bytes(),
            Bytes::from("data 1data 2data 3")
        );
    }

    #[cfg(feature = "rt-tokio")]
    #[tokio::test]
    async fn path_based_bytestreams() -> Result<(), Box<dyn std::error::Error>> {
        use super::ByteStream;
        use bytes::Buf;
        use http_body::Body;
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut file = NamedTempFile::new()?;

        for i in 0..10000 {
            writeln!(file, "Brian was here. Briefly. {}", i)?;
        }
        let body = ByteStream::from_path(&file).await?.into_inner();
        // assert that a valid size hint is immediately ready
        assert_eq!(body.size_hint().exact(), Some(298890));
        let mut body1 = body.try_clone().expect("retryable bodies are cloneable");
        // read a little bit from one of the clones
        let some_data = body1
            .data()
            .await
            .expect("should have some data")
            .expect("read should not fail");
        assert!(!some_data.is_empty());
        // make some more clones
        let body2 = body.try_clone().expect("retryable bodies are cloneable");
        let body3 = body.try_clone().expect("retryable bodies are cloneable");
        let body2 = ByteStream::new(body2).collect().await?.into_bytes();
        let body3 = ByteStream::new(body3).collect().await?.into_bytes();
        assert_eq!(body2, body3);
        assert!(body2.starts_with(b"Brian was here."));
        assert!(body2.ends_with(b"9999\n"));
        assert_eq!(body2.len(), 298890);

        assert_eq!(
            ByteStream::new(body1).collect().await?.remaining(),
            298890 - some_data.len()
        );

        Ok(())
    }
}
