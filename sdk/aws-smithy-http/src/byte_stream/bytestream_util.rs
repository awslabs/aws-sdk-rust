/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::body::SdkBody;
use crate::byte_stream::{error::Error, error::ErrorKind, ByteStream};
use bytes::Bytes;
use futures_core::ready;
use http::HeaderMap;
use http_body::{Body, SizeHint};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

// 4KB corresponds to the default buffer size used by Tokio's ReaderStream
const DEFAULT_BUFFER_SIZE: usize = 4096;
// By default, read files from their start
const DEFAULT_OFFSET: u64 = 0;

/// An HTTP Body designed to wrap files
///
/// PathBody is a three-phase HTTP body designed to wrap files with three specific features:
/// 1. The underlying file is wrapped with StreamReader to implement HTTP body
/// 2. It can be constructed directly from a path so it's easy to use during retries
/// 3. Provide size hint
struct PathBody {
    state: State,
    // The number of bytes to read
    length: u64,
    buffer_size: usize,
    // The byte-offset to start reading from
    offset: Option<u64>,
}

impl PathBody {
    fn from_path(path_buf: PathBuf, length: u64, buffer_size: usize, offset: Option<u64>) -> Self {
        PathBody {
            state: State::Unloaded(path_buf),
            length,
            buffer_size,
            offset,
        }
    }

    fn from_file(file: File, length: u64, buffer_size: usize) -> Self {
        PathBody {
            state: State::Loaded(ReaderStream::with_capacity(file.take(length), buffer_size)),
            length,
            buffer_size,
            /// The file used to create this `PathBody` should have already had an offset applied
            offset: None,
        }
    }
}

/// Builder for creating [`ByteStreams`](ByteStream) from a file/path, with full control over advanced options.
///
/// Example usage:
/// ```no_run
/// # #[cfg(feature = "rt-tokio")]
/// # {
/// use aws_smithy_http::byte_stream::{ByteStream, Length};
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
///         .length(Length::UpTo(123_456))
///         .build()
///         .await
///         .expect("valid path");
///     GetObjectInput { body: bytestream }
/// }
/// # }
/// ```
#[allow(missing_debug_implementations)]
pub struct FsBuilder {
    file: Option<File>,
    path: Option<PathBuf>,
    length: Option<Length>,
    buffer_size: usize,
    offset: Option<u64>,
}

impl Default for FsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// The length (in bytes) to read. Determines whether or not a short read counts as an error.
#[allow(missing_debug_implementations)]
pub enum Length {
    /// Read this number of bytes exactly. Returns an error if the file is smaller than expected.
    Exact(u64),
    /// Read up to this number of bytes. May read less than the specified amount if the file
    /// is smaller than expected.
    UpTo(u64),
}

impl FsBuilder {
    /// Create a new [`FsBuilder`] (using a default read buffer of 4096 bytes).
    ///
    /// You must then call either [`file`](FsBuilder::file) or [`path`](FsBuilder::path) to specify what to read from.
    pub fn new() -> Self {
        FsBuilder {
            buffer_size: DEFAULT_BUFFER_SIZE,
            file: None,
            length: None,
            offset: None,
            path: None,
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
    pub fn file(mut self, file: File) -> Self {
        self.file = Some(file);
        self
    }

    /// Specify the length to read (in bytes).
    ///
    /// By pre-specifying the length, this API skips an additional call to retrieve the size from file-system metadata.
    ///
    /// When used in conjunction with [`offset`](FsBuilder::offset), allows for reading a single "chunk" of a file.
    pub fn length(mut self, length: Length) -> Self {
        self.length = Some(length);
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

    /// Specify the offset to start reading from (in bytes)
    ///
    /// When used in conjunction with [`length`](FsBuilder::length), allows for reading a single "chunk" of a file.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Returns a [`ByteStream`](ByteStream) from this builder.
    pub async fn build(self) -> Result<ByteStream, Error> {
        if self.path.is_some() && self.file.is_some() {
            panic!("The 'file' and 'path' options on an FsBuilder are mutually exclusive but both were set. Please set only one")
        };

        let buffer_size = self.buffer_size;
        let offset = self.offset.unwrap_or(DEFAULT_OFFSET);
        // Checking the file length like this does have a cost, but the benefit is that we can
        // notify users when file/chunk is smaller than expected.
        let file_length = self.get_file_size().await?;
        if offset > file_length {
            return Err(ErrorKind::OffsetLargerThanFileSize.into());
        }

        let length = match self.length {
            Some(Length::Exact(length)) => {
                if length > file_length - offset {
                    return Err(ErrorKind::LengthLargerThanFileSizeMinusReadOffset.into());
                }
                length
            }
            Some(Length::UpTo(length)) => length,
            None => file_length - offset,
        };

        if let Some(path) = self.path {
            let body_loader = move || {
                // If an offset was provided, seeking will be handled in `PathBody::poll_data` each
                // time the file is loaded.
                SdkBody::from_dyn(http_body::combinators::BoxBody::new(PathBody::from_path(
                    path.clone(),
                    length,
                    buffer_size,
                    self.offset,
                )))
            };

            Ok(ByteStream::new(SdkBody::retryable(body_loader)))
        } else if let Some(mut file) = self.file {
            // When starting from a `File`, we need to do our own seeking
            if offset != 0 {
                let _s = file.seek(io::SeekFrom::Start(offset)).await?;
            }

            let body = SdkBody::from_dyn(http_body::combinators::BoxBody::new(
                PathBody::from_file(file, length, buffer_size),
            ));

            Ok(ByteStream::new(body))
        } else {
            panic!("FsBuilder constructed without a file or a path")
        }
    }

    async fn get_file_size(&self) -> Result<u64, Error> {
        Ok(match self.path.as_ref() {
            Some(path) => tokio::fs::metadata(path).await,
            // If it's not path-based then it's file-based
            None => self.file.as_ref().unwrap().metadata().await,
        }
        .map(|metadata| metadata.len())?)
    }
}

enum State {
    Unloaded(PathBuf),
    Loading(Pin<Box<dyn Future<Output = io::Result<File>> + Send + Sync + 'static>>),
    Loaded(ReaderStream<io::Take<File>>),
}

impl Body for PathBody {
    type Data = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let offset = self.offset.unwrap_or(DEFAULT_OFFSET);
        loop {
            match self.state {
                State::Unloaded(ref path_buf) => {
                    let buf = path_buf.clone();
                    self.state = State::Loading(Box::pin(async move {
                        let mut file = File::open(&buf).await?;

                        if offset != 0 {
                            let _s = file.seek(io::SeekFrom::Start(offset)).await?;
                        }

                        Ok(file)
                    }));
                }
                State::Loading(ref mut future) => {
                    match ready!(Pin::new(future).poll(cx)) {
                        Ok(file) => {
                            self.state = State::Loaded(ReaderStream::with_capacity(
                                file.take(self.length),
                                self.buffer_size,
                            ));
                        }
                        Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    };
                }
                State::Loaded(ref mut stream) => {
                    use futures_core::Stream;
                    return match ready!(Pin::new(stream).poll_next(cx)) {
                        Some(Ok(bytes)) => Poll::Ready(Some(Ok(bytes))),
                        None => Poll::Ready(None),
                        Some(Err(e)) => Poll::Ready(Some(Err(e.into()))),
                    };
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
        // fast path end-stream for empty streams
        self.length == 0
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.length)
    }
}

#[cfg(test)]
mod test {
    use super::FsBuilder;
    use crate::byte_stream::{ByteStream, Length};
    use bytes::Buf;
    use http_body::Body;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn path_based_bytestreams_with_builder() {
        let mut file = NamedTempFile::new().unwrap();

        for i in 0..10000 {
            writeln!(file, "Brian was here. Briefly. {}", i).unwrap();
        }
        let file_length = file
            .as_file()
            .metadata()
            .expect("file metadata is accessible")
            .len();

        let body = FsBuilder::new()
            .path(&file)
            .buffer_size(16384)
            .length(Length::Exact(file_length))
            .build()
            .await
            .unwrap()
            .into_inner();

        // assert that the specified length is used as size hint
        assert_eq!(body.size_hint().exact(), Some(file_length));

        let mut body1 = body.try_clone().expect("retryable bodies are cloneable");
        // read a little bit from one of the clones
        let some_data = body1
            .data()
            .await
            .expect("should have some data")
            .expect("read should not fail");
        // The size of one read should be equal to that of the buffer size
        assert_eq!(some_data.len(), 16384);

        assert_eq!(
            ByteStream::new(body1).collect().await.unwrap().remaining() as u64,
            file_length - some_data.len() as u64
        );
    }

    #[tokio::test]
    async fn fsbuilder_length_is_used_as_size_hint() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            "A very long sentence that's clearly longer than a single byte."
        )
        .unwrap();
        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        let body = FsBuilder::new()
            .path(&file)
            // The file is longer than 1 byte, let's see if this is used to generate the size hint
            .length(Length::Exact(1))
            .build()
            .await
            .unwrap()
            .into_inner();

        assert_eq!(body.size_hint().exact(), Some(1));
    }

    #[tokio::test]
    async fn fsbuilder_respects_length() {
        let mut file = NamedTempFile::new().unwrap();
        let line_0 = "Line 0\n";
        let line_1 = "Line 1\n";

        write!(file, "{}", line_0).unwrap();
        write!(file, "{}", line_1).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        let body = FsBuilder::new()
            .path(&file)
            // We're going to read line 0 only
            .length(Length::Exact(line_0.len() as u64))
            .build()
            .await
            .unwrap();

        let data = body.collect().await.unwrap().into_bytes();
        let data_str = String::from_utf8(data.to_vec()).unwrap();

        assert_eq!(&data_str, line_0);
    }

    #[tokio::test]
    async fn fsbuilder_length_exact() {
        let mut file = NamedTempFile::new().unwrap();
        let test_sentence = "This sentence is 30 bytes long";
        assert_eq!(test_sentence.len(), 30);
        write!(file, "{}", test_sentence).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        assert!(FsBuilder::new()
            .path(&file)
            // The file is 30 bytes so this is fine
            .length(Length::Exact(29))
            .build()
            .await
            .is_ok());

        assert!(FsBuilder::new()
            .path(&file)
            // The file is 30 bytes so this is fine
            .length(Length::Exact(30))
            .build()
            .await
            .is_ok());

        assert!(FsBuilder::new()
            .path(&file)
            // Larger than 30 bytes, this will cause an error
            .length(Length::Exact(31))
            .build()
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fsbuilder_supports_offset() {
        let mut file = NamedTempFile::new().unwrap();
        let line_0 = "Line 0\n";
        let line_1 = "Line 1\n";

        write!(file, "{}", line_0).unwrap();
        write!(file, "{}", line_1).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        let body = FsBuilder::new()
            .path(&file)
            // We're going to skip the first line by using offset
            .offset(line_0.len() as u64)
            .build()
            .await
            .unwrap();

        let data = body.collect().await.unwrap().into_bytes();
        let data_str = String::from_utf8(data.to_vec()).unwrap();

        assert_eq!(&data_str, line_1);
    }

    #[tokio::test]
    async fn fsbuilder_offset_and_length_work_together() {
        let mut file = NamedTempFile::new().unwrap();
        let line_0 = "Line 0\n";
        let line_1 = "Line 1\n";
        let line_2 = "Line 2\n";

        write!(file, "{}", line_0).unwrap();
        write!(file, "{}", line_1).unwrap();
        write!(file, "{}", line_2).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        let body = FsBuilder::new()
            .path(&file)
            // We're going to skip line 0 by using offset
            .offset(line_0.len() as u64)
            // We want to read only line 1 and stop before we get to line 2
            .length(Length::Exact(line_1.len() as u64))
            .build()
            .await
            .unwrap();

        let data = body.collect().await.unwrap().into_bytes();
        let data_str = String::from_utf8(data.to_vec()).unwrap();

        assert_eq!(&data_str, line_1);
    }

    #[tokio::test]
    async fn fsbuilder_with_offset_greater_than_file_length_returns_error() {
        let mut file = NamedTempFile::new().unwrap();
        let line_0 = "Line 0\n";
        let line_1 = "Line 1\n";

        write!(file, "{}", line_0).unwrap();
        write!(file, "{}", line_1).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        assert_eq!(
            FsBuilder::new()
                .path(&file)
                // We're going to skip all file contents by setting an offset
                // much larger than the file size
                .offset(9000)
                .build()
                .await
                .unwrap_err()
                .to_string(),
            "offset must be less than or equal to file size but was greater than"
        );
    }

    #[tokio::test]
    async fn fsbuilder_with_length_greater_than_file_length_reads_everything() {
        let mut file = NamedTempFile::new().unwrap();
        let line_0 = "Line 0\n";
        let line_1 = "Line 1\n";

        write!(file, "{}", line_0).unwrap();
        write!(file, "{}", line_1).unwrap();

        // Ensure that the file was written to
        file.flush().expect("flushing is OK");

        let body = FsBuilder::new()
            .path(&file)
            .length(Length::UpTo(9000))
            .build()
            .await
            .unwrap();

        let data = body.collect().await.unwrap().into_bytes();
        let data_str = String::from_utf8(data.to_vec()).unwrap();

        assert_eq!(data_str, format!("{}{}", line_0, line_1));
    }

    #[tokio::test]
    async fn fsbuilder_can_be_used_for_chunking() {
        let mut file = NamedTempFile::new().unwrap();
        let mut in_memory_copy_of_file_contents = String::new();
        // I put these two write loops in separate blocks so that the traits wouldn't conflict
        {
            use std::io::Write;
            for i in 0..1000 {
                writeln!(file, "Line {:04}", i).unwrap();
            }
        }

        {
            use std::fmt::Write;
            for i in 0..1000 {
                writeln!(in_memory_copy_of_file_contents, "Line {:04}", i).unwrap();
            }
            // Check we wrote the lines
            assert!(!in_memory_copy_of_file_contents.is_empty());
        }

        let file_size = file.as_file().metadata().unwrap().len();
        // Check that our in-memory copy has the same size as the file
        assert_eq!(file_size, in_memory_copy_of_file_contents.len() as u64);
        let file_path = file.path().to_path_buf();
        let chunks = 7;
        let chunk_size = file_size / chunks;

        let mut byte_streams = Vec::new();
        for i in 0..chunks {
            let length = if i == chunks - 1 {
                // If we're on the last chunk, the length to read might be less than a whole chunk.
                // We subtract the size of all previous chunks from the total file size to get the
                // size of the final chunk.
                file_size - (i * chunk_size)
            } else {
                chunk_size
            };

            let byte_stream = FsBuilder::new()
                .path(&file_path)
                .offset(i * chunk_size)
                .length(Length::Exact(length))
                .build()
                .await
                .unwrap();

            byte_streams.push(byte_stream);
        }

        let mut collected_bytes = Vec::new();

        for byte_stream in byte_streams.into_iter() {
            let bytes = byte_stream.collect().await.unwrap().into_bytes();
            collected_bytes.push(bytes);
        }

        let bytes = collected_bytes.concat();
        let data_str = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(data_str, in_memory_copy_of_file_contents);
    }
}
