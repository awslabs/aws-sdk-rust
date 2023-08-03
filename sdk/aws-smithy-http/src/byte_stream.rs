/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
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
//! # {
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
//! # }
//! ```
//!
//! If you want more control over how the file is read, such as specifying the size of the buffer used to read the file
//! or the length of the file, use an [`FsBuilder`](crate::byte_stream::FsBuilder).
//!
//! ```no_run
//! # #[cfg(feature = "rt-tokio")]
//! # {
//! use aws_smithy_http::byte_stream::{ByteStream, Length};
//! use std::path::Path;
//! struct GetObjectInput {
//!     body: ByteStream
//! }
//!
//! async fn bytestream_from_file() -> GetObjectInput {
//!     let bytestream = ByteStream::read_from().path("docs/some-large-file.csv")
//!         .buffer_size(32_784)
//!         .length(Length::Exact(123_456))
//!         .build()
//!         .await
//!         .expect("valid path");
//!     GetObjectInput { body: bytestream }
//! }
//! # }
//! ```

use crate::body::SdkBody;
use crate::byte_stream::error::Error;
use bytes::Buf;
use bytes::Bytes;
use bytes_utils::SegmentedBuf;
use http_body::Body;
use pin_project_lite::pin_project;
use std::io::IoSlice;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(feature = "rt-tokio")]
mod bytestream_util;
#[cfg(feature = "rt-tokio")]
pub use bytestream_util::Length;

pub mod error;

#[cfg(feature = "rt-tokio")]
pub use self::bytestream_util::FsBuilder;

pin_project! {
    /// Stream of binary data
    ///
    /// `ByteStream` wraps a stream of binary data for ease of use.
    ///
    /// ## Getting data out of a `ByteStream`
    ///
    /// `ByteStream` provides two primary mechanisms for accessing the data:
    /// 1. With `.collect()`:
    ///
    ///     [`.collect()`](crate::byte_stream::ByteStream::collect) reads the complete ByteStream into memory and stores it in `AggregatedBytes`,
    ///     a non-contiguous ByteBuffer.
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
    ///     _Note: An import of `StreamExt` is required to use `.try_next()`._
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
    ///     use aws_smithy_http::byte_stream::{ByteStream, AggregatedBytes, error::Error};
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
    /// 3. Via [`.into_async_read()`](crate::byte_stream::ByteStream::into_async_read):
    ///
    ///     _Note: The `rt-tokio` feature must be active to use `.into_async_read()`._
    ///
    ///     It's possible to convert a `ByteStream` into a struct that implements [`tokio::io::AsyncRead`](tokio::io::AsyncRead).
    ///     Then, you can use pre-existing tools like [`tokio::io::BufReader`](tokio::io::BufReader):
    ///     ```no_run
    ///     use aws_smithy_http::byte_stream::ByteStream;
    ///     use aws_smithy_http::body::SdkBody;
    ///     use tokio::io::{AsyncBufReadExt, BufReader};
    ///     #[cfg(feature = "rt-tokio")]
    ///     async fn example() -> std::io::Result<()> {
    ///        let stream = ByteStream::new(SdkBody::from("hello!\nThis is some data"));
    ///        // Wrap the stream in a BufReader
    ///        let buf_reader = BufReader::new(stream.into_async_read());
    ///        let mut lines = buf_reader.lines();
    ///        assert_eq!(lines.next_line().await?, Some("hello!".to_owned()));
    ///        assert_eq!(lines.next_line().await?, Some("This is some data".to_owned()));
    ///        assert_eq!(lines.next_line().await?, None);
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
    #[derive(Debug)]
    pub struct ByteStream {
        #[pin]
        inner: Inner<SdkBody>
    }
}

impl ByteStream {
    /// Create a new `ByteStream` from an [`SdkBody`].
    pub fn new(body: SdkBody) -> Self {
        Self {
            inner: Inner::new(body),
        }
    }

    /// Create a new `ByteStream` from a static byte slice.
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            inner: Inner::new(SdkBody::from(Bytes::from_static(bytes))),
        }
    }

    /// Consumes the ByteStream, returning the wrapped SdkBody
    // Backwards compatibility note: Because SdkBody has a dyn variant,
    // we will always be able to implement this method, even if we stop using
    // SdkBody as the internal representation
    pub fn into_inner(self) -> SdkBody {
        self.inner.body
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
    /// use aws_smithy_http::byte_stream::{ByteStream, error::Error};
    /// async fn get_data() {
    ///     let stream = ByteStream::new(SdkBody::from("hello!"));
    ///     let data: Result<Bytes, Error> = stream.collect().await.map(|data| data.into_bytes());
    /// }
    /// ```
    pub async fn collect(self) -> Result<AggregatedBytes, Error> {
        self.inner.collect().await.map_err(Error::streaming)
    }

    /// Returns a [`FsBuilder`](crate::byte_stream::FsBuilder), allowing you to build a `ByteStream` with
    /// full control over how the file is read (eg. specifying the length of the file or the size of the buffer used to read the file).
    /// ```no_run
    /// # #[cfg(feature = "rt-tokio")]
    /// # {
    /// use aws_smithy_http::byte_stream::{ByteStream, Length};
    ///
    /// async fn bytestream_from_file() -> ByteStream {
    ///     let bytestream = ByteStream::read_from()
    ///         .path("docs/some-large-file.csv")
    ///         // Specify the size of the buffer used to read the file (in bytes, default is 4096)
    ///         .buffer_size(32_784)
    ///         // Specify the length of the file used (skips an additional call to retrieve the size)
    ///         .length(Length::Exact(123_456))
    ///         .build()
    ///         .await
    ///         .expect("valid path");
    ///     bytestream
    /// }
    /// # }
    /// ```
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub fn read_from() -> FsBuilder {
        FsBuilder::new()
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
    /// Note: If you want more control, such as specifying the size of the buffer used to read the file
    /// or the length of the file, use a [`FsBuilder`](crate::byte_stream::FsBuilder) as returned
    /// from `ByteStream::read_from`
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
        FsBuilder::new().path(path).build().await
    }

    /// Create a ByteStream from a file
    ///
    /// NOTE: This will NOT result in a retryable ByteStream. For a ByteStream that can be retried in the case of
    /// upstream failures, use [`ByteStream::from_path`](ByteStream::from_path)
    #[deprecated(
        since = "0.40.0",
        note = "Prefer the more extensible ByteStream::read_from() API"
    )]
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub async fn from_file(file: tokio::fs::File) -> Result<Self, Error> {
        FsBuilder::new().file(file).build().await
    }

    #[cfg(feature = "rt-tokio")]
    /// Convert this `ByteStream` into a struct that implements [`AsyncRead`](tokio::io::AsyncRead).
    ///
    /// # Example
    ///
    /// ```rust
    /// use tokio::io::{BufReader, AsyncBufReadExt};
    /// use aws_smithy_http::byte_stream::ByteStream;
    ///
    /// # async fn dox(my_bytestream: ByteStream) -> std::io::Result<()> {
    /// let mut lines =  BufReader::new(my_bytestream.into_async_read()).lines();
    /// while let Some(line) = lines.next_line().await? {
    ///   // Do something line by line
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_async_read(self) -> impl tokio::io::AsyncRead {
        tokio_util::io::StreamReader::new(self)
    }

    /// Given a function to modify an [`SdkBody`], run it on the `SdkBody` inside this `Bytestream`.
    /// returning a new `Bytestream`.
    pub fn map(self, f: impl Fn(SdkBody) -> SdkBody + Send + Sync + 'static) -> ByteStream {
        ByteStream::new(self.into_inner().map(f))
    }
}

impl Default for ByteStream {
    fn default() -> Self {
        Self {
            inner: Inner {
                body: SdkBody::from(""),
            },
        }
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
        ByteStream::new(SdkBody::from(input))
    }
}

impl futures_core::stream::Stream for ByteStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx).map_err(Error::streaming)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
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

    /// Convert this buffer into an [`Iterator`] of underlying non-contiguous segments of [`Bytes`]
    pub fn into_segments(self) -> impl Iterator<Item = Bytes> {
        self.0.into_inner().into_iter()
    }

    /// Convert this buffer into a `Vec<u8>`
    pub fn to_vec(self) -> Vec<u8> {
        self.0.into_inner().into_iter().flatten().collect()
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

pin_project! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Inner<B> {
        #[pin]
        body: B,
    }
}

impl<B> Inner<B> {
    fn new(body: B) -> Self {
        Self { body }
    }

    async fn collect(self) -> Result<AggregatedBytes, B::Error>
    where
        B: http_body::Body<Data = Bytes>,
    {
        let mut output = SegmentedBuf::new();
        let body = self.body;
        pin_utils::pin_mut!(body);
        while let Some(buf) = body.data().await {
            output.push(buf?);
        }
        Ok(AggregatedBytes(output))
    }
}

const SIZE_HINT_32_BIT_PANIC_MESSAGE: &str = r#"
You're running a 32-bit system and this stream's length is too large to be represented with a usize.
Please limit stream length to less than 4.294Gb or run this program on a 64-bit computer architecture.
"#;

impl<B> futures_core::stream::Stream for Inner<B>
where
    B: http_body::Body<Data = Bytes>,
{
    type Item = Result<Bytes, B::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().body.poll_data(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size_hint = http_body::Body::size_hint(&self.body);
        let lower = size_hint.lower().try_into();
        let upper = size_hint.upper().map(|u| u.try_into()).transpose();

        match (lower, upper) {
            (Ok(lower), Ok(upper)) => (lower, upper),
            (Err(_), _) | (_, Err(_)) => {
                panic!("{}", SIZE_HINT_32_BIT_PANIC_MESSAGE)
            }
        }
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

    #[cfg(feature = "rt-tokio")]
    #[tokio::test]
    async fn bytestream_into_async_read() {
        use super::ByteStream;
        use tokio::io::AsyncBufReadExt;

        let byte_stream = ByteStream::from_static(b"data 1\ndata 2\ndata 3");
        let async_buf_read = tokio::io::BufReader::new(byte_stream.into_async_read());

        let mut lines = async_buf_read.lines();

        assert_eq!(lines.next_line().await.unwrap(), Some("data 1".to_owned()));
        assert_eq!(lines.next_line().await.unwrap(), Some("data 2".to_owned()));
        assert_eq!(lines.next_line().await.unwrap(), Some("data 3".to_owned()));
        assert_eq!(lines.next_line().await.unwrap(), None);
    }
}
