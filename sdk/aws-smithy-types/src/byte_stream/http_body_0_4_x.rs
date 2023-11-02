/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::body::SdkBody;
use crate::byte_stream::ByteStream;
use bytes::Bytes;

impl ByteStream {
    /// Construct a `ByteStream` from a type that implements [`http_body_0_4::Body<Data = Bytes>`](http_body_0_4::Body).
    ///
    /// _Note: This is only available with `http-body-0-4-x` enabled._
    pub fn from_body_0_4<T, E>(body: T) -> Self
    where
        T: http_body_0_4::Body<Data = Bytes, Error = E> + Send + Sync + 'static,
        E: Into<crate::body::Error> + 'static,
    {
        ByteStream::new(SdkBody::from_body_0_4(body))
    }

    /// Returns a [`FsBuilder`](crate::byte_stream::FsBuilder), allowing you to build a `ByteStream` with
    /// full control over how the file is read (eg. specifying the length of the file or the size of the buffer used to read the file).
    /// ```no_run
    /// # #[cfg(all(feature = "rt-tokio", feature = "http-body-0-4-x"))]
    /// # {
    /// use aws_smithy_types::byte_stream::{ByteStream, Length};
    ///
    /// async fn bytestream_from_file() -> ByteStream {
    ///     let bytestream = ByteStream::read_with_body_0_4_from()
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
    pub fn read_with_body_0_4_from() -> crate::byte_stream::FsBuilder {
        crate::byte_stream::FsBuilder::new_with_body_0_4()
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
    /// from `ByteStream::read_with_body_0_4_from`
    ///
    /// # Examples
    /// ```no_run
    /// use aws_smithy_types::byte_stream::ByteStream;
    /// use std::path::Path;
    ///  async fn make_bytestream() -> ByteStream {
    ///     ByteStream::from_path_body_0_4("docs/rows.csv").await.expect("file should be readable")
    /// }
    /// ```
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub async fn from_path_body_0_4(
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::byte_stream::error::Error> {
        crate::byte_stream::FsBuilder::new_with_body_0_4()
            .path(path)
            .build()
            .await
    }

    /// Create a ByteStream from a file
    ///
    /// NOTE: This will NOT result in a retryable ByteStream. For a ByteStream that can be retried in the case of
    /// upstream failures, use [`ByteStream::from_path_body_0_4`](ByteStream::from_path_body_0_4)
    #[deprecated(
        since = "0.40.0",
        note = "Prefer the more extensible ByteStream::read_from() API"
    )]
    #[cfg(feature = "rt-tokio")]
    #[cfg_attr(docsrs, doc(cfg(feature = "rt-tokio")))]
    pub async fn from_file_body_0_4(
        file: tokio::fs::File,
    ) -> Result<Self, crate::byte_stream::error::Error> {
        crate::byte_stream::FsBuilder::new_with_body_0_4()
            .file(file)
            .build()
            .await
    }
}

#[cfg(feature = "hyper-0-14-x")]
impl From<hyper_0_14::Body> for ByteStream {
    fn from(input: hyper_0_14::Body) -> Self {
        ByteStream::new(SdkBody::from_body_0_4(input))
    }
}

#[cfg(test)]
mod tests {
    use crate::body::SdkBody;
    use crate::byte_stream::Inner;
    use bytes::Bytes;

    #[tokio::test]
    async fn read_from_channel_body() {
        let (mut sender, body) = hyper_0_14::Body::channel();
        let byte_stream = Inner::new(SdkBody::from_body_0_4(body));
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
        use std::io::Write;
        use tempfile::NamedTempFile;
        let mut file = NamedTempFile::new()?;

        for i in 0..10000 {
            writeln!(file, "Brian was here. Briefly. {}", i)?;
        }
        let body = ByteStream::from_path_body_0_4(&file).await?.into_inner();
        // assert that a valid size hint is immediately ready
        assert_eq!(body.content_length(), Some(298890));
        let mut body1 = body.try_clone().expect("retryable bodies are cloneable");
        // read a little bit from one of the clones
        let some_data = body1
            .next()
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
