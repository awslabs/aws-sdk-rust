/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
//! ByteStream Abstractions
//!
//! When the SDK returns streaming binary data, the inner Http Body is wrapped in [ByteStream](crate::byte_stream::ByteStream). ByteStream provides misuse-resistant
//! primitives to make it easier to handle common patterns with streaming data.
//!
//! ## Examples:
//!
//! ### Writing a ByteStream into a file:
//! ```rust
//! use bytes::Buf;
//! use smithy_http::byte_stream::ByteStream;
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
//!     while buf.has_remaining() {
//!         file.write_buf(&mut buf).await?;
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ### Converting a ByteStream into Bytes
//! ```rust
//! use bytes::Bytes;
//! use smithy_http::byte_stream::ByteStream;
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
//! ```rust
//! use bytes::{Buf, Bytes};
//! use smithy_http::byte_stream::ByteStream;
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
//!     Ok(())
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

/// Stream of binary data
///
/// `ByteStream` wraps a stream of binary data for ease of use.
///
/// `ByteStream` provides two primary mechanisms for accessing the data:
/// 1. With `.collect()`:
/// [`.collect()`](crate::byte_stream::ByteStream::collect) reads the complete ByteStream into memory and stores it in `AggregatedBytes`,
/// a non-contiguous ByteBuffer.
///     ```rust
///     use smithy_http::byte_stream::{ByteStream, AggregatedBytes};
///     use smithy_http::body::SdkBody;
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
///     **Note**: An import of `StreamExt` is required to use `try_next()`.
///     For use-cases where holding the entire ByteStream in memory is unecessary, use the
///     `Stream` implementation:
///     ```rust
///     # mod crc32 {
///     #   pub struct Digest { }
///     #   impl Digest {
///     #       pub fn new() -> Self { Digest {} }
///     #       pub fn write(&mut self, b: &[u8]) { }
///     #       pub fn finish(&self) -> u64 { 6 }
///     #   }
///     # }
///     use smithy_http::byte_stream::{ByteStream, AggregatedBytes, Error};
///     use smithy_http::body::SdkBody;
///     use tokio_stream::StreamExt;
///
///     async fn example() -> Result<(), Error> {
///        let mut stream = ByteStream::new(SdkBody::from("hello! This is some data"));
///        let mut digest = crc32::Digest::new();
///        while let Some(bytes) = stream.try_next().await? {
///            digest.write(&bytes);
///        }
///        println!("digest: {}", digest.finish());
///        Ok(())
///     }
///     ```
///
/// `ByteStream`
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
    /// ```rust
    /// use bytes::Bytes;
    /// use smithy_http::body;
    /// use smithy_http::body::SdkBody;
    /// use smithy_http::byte_stream::{ByteStream, Error};
    /// async fn get_data() {
    ///     let stream = ByteStream::new(SdkBody::from("hello!"));
    ///     let data: Result<Bytes, Error> = stream.collect().await.map(|data| data.into_bytes());
    /// }
    /// ```
    pub async fn collect(self) -> Result<AggregatedBytes, Error> {
        self.0.collect().await.map_err(|err| Error(err))
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
}
