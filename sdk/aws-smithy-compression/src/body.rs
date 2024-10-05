/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP body-wrappers that perform request compression

// Putting this in a `mod` since I expect we'll have to handle response
// decompression some day.
/// Functionality for compressing an HTTP request body.
pub mod compress {
    use aws_smithy_types::body::SdkBody;
    use pin_project_lite::pin_project;

    pin_project! {
        /// A `Body` that may compress its data with a `CompressRequest` implementor.
        ///
        /// Compression options may disable request compression for small data payload, or entirely.
        /// Additionally, some services may not support compression.
        pub struct CompressedBody<InnerBody, CompressionImpl> {
            #[pin]
            body: InnerBody,
            compress_request: CompressionImpl,
        }
    }

    impl<CR> CompressedBody<SdkBody, CR> {
        /// Given an [`SdkBody`] and a `Box<dyn CompressRequest>`, create a new `CompressedBody<SdkBody, CR>`.
        pub fn new(body: SdkBody, compress_request: CR) -> Self {
            Self {
                body,
                compress_request,
            }
        }
    }

    /// Support for the `http-body-0-4` and `http-0-2` crates.
    #[cfg(feature = "http-body-0-4-x")]
    pub mod http_body_0_4_x {
        use super::CompressedBody;
        use crate::http::http_body_0_4_x::CompressRequest;
        use aws_smithy_types::body::SdkBody;
        use http_0_2::HeaderMap;
        use http_body_0_4::{Body, SizeHint};
        use std::pin::Pin;
        use std::task::{Context, Poll};

        impl Body for CompressedBody<SdkBody, Box<dyn CompressRequest>> {
            type Data = bytes::Bytes;
            type Error = aws_smithy_types::body::Error;

            fn poll_data(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
                let this = self.project();
                match this.body.poll_data(cx)? {
                    Poll::Ready(Some(data)) => {
                        let mut out = Vec::new();
                        this.compress_request.compress_bytes(&data[..], &mut out)?;
                        Poll::Ready(Some(Ok(out.into())))
                    }
                    Poll::Ready(None) => Poll::Ready(None),
                    Poll::Pending => Poll::Pending,
                }
            }

            fn poll_trailers(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
                let this = self.project();
                this.body.poll_trailers(cx)
            }

            fn is_end_stream(&self) -> bool {
                self.body.is_end_stream()
            }

            fn size_hint(&self) -> SizeHint {
                self.body.size_hint()
            }
        }
    }

    /// Support for the `http-body-1-0` and `http-1-0` crates.
    #[cfg(feature = "http-body-1-x")]
    pub mod http_body_1_x {
        use super::CompressedBody;
        use crate::http::http_body_1_x::CompressRequest;
        use aws_smithy_types::body::SdkBody;
        use http_body_1_0::{Body, Frame, SizeHint};
        use std::pin::Pin;
        use std::task::{ready, Context, Poll};

        impl Body for CompressedBody<SdkBody, Box<dyn CompressRequest>> {
            type Data = bytes::Bytes;
            type Error = aws_smithy_types::body::Error;

            fn poll_frame(
                mut self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
                let this = self.as_mut().project();
                Poll::Ready(match ready!(this.body.poll_frame(cx)) {
                    Some(Ok(f)) => {
                        if f.is_data() {
                            let d = f.into_data().expect("we checked for data first");
                            let mut out = Vec::new();
                            this.compress_request.compress_bytes(&d, &mut out)?;
                            Some(Ok(Frame::data(out.into())))
                        } else if f.is_trailers() {
                            // Trailers don't get compressed.
                            Some(Ok(f))
                        } else {
                            unreachable!("Frame is either data or trailers")
                        }
                    }
                    other => other,
                })
            }

            fn is_end_stream(&self) -> bool {
                self.body.is_end_stream()
            }

            fn size_hint(&self) -> SizeHint {
                self.body.size_hint()
            }
        }
    }
}

#[cfg(any(feature = "http-body-0-4-x", feature = "http-body-1-x"))]
#[cfg(test)]
mod test {
    const UNCOMPRESSED_INPUT: &[u8] = b"hello world";
    const COMPRESSED_OUTPUT: &[u8] = &[
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 203, 72, 205, 201, 201, 87, 40, 207, 47, 202, 73, 1, 0,
        133, 17, 74, 13, 11, 0, 0, 0,
    ];

    #[cfg(feature = "http-body-0-4-x")]
    #[tokio::test]
    async fn test_compressed_body_http_body_0_4_x() {
        use super::super::{CompressionAlgorithm, CompressionOptions};
        use crate::body::compress::CompressedBody;
        use aws_smithy_types::body::SdkBody;
        use bytes::Buf;
        use bytes_utils::SegmentedBuf;
        use http_body_0_4::Body;
        use std::io::Read;

        let compression_algorithm = CompressionAlgorithm::Gzip;
        let compression_options = CompressionOptions::default()
            .with_min_compression_size_bytes(0)
            .unwrap();
        let compress_request =
            compression_algorithm.into_impl_http_body_0_4_x(&compression_options);
        let body = SdkBody::from(UNCOMPRESSED_INPUT);
        let mut compressed_body = CompressedBody::new(body, compress_request);

        let mut output = SegmentedBuf::new();
        while let Some(buf) = compressed_body.data().await {
            output.push(buf.unwrap());
        }

        let mut actual_output = Vec::new();
        output
            .reader()
            .read_to_end(&mut actual_output)
            .expect("Doesn't cause IO errors");
        // Verify data is compressed as expected
        assert_eq!(COMPRESSED_OUTPUT, actual_output);
    }

    #[cfg(feature = "http-body-1-x")]
    #[tokio::test]
    async fn test_compressed_body_http_body_1_x() {
        use super::super::{CompressionAlgorithm, CompressionOptions};
        use crate::body::compress::CompressedBody;
        use aws_smithy_types::body::SdkBody;
        use bytes::Buf;
        use bytes_utils::SegmentedBuf;
        use http_body_util::BodyExt;
        use std::io::Read;

        let compression_algorithm = CompressionAlgorithm::Gzip;
        let compression_options = CompressionOptions::default()
            .with_min_compression_size_bytes(0)
            .unwrap();
        let compress_request = compression_algorithm.into_impl_http_body_1_x(&compression_options);
        let body = SdkBody::from(UNCOMPRESSED_INPUT);
        let mut compressed_body = CompressedBody::new(body, compress_request);

        let mut output = SegmentedBuf::new();

        loop {
            let data = match compressed_body.frame().await {
                Some(Ok(frame)) => frame.into_data(),
                Some(Err(e)) => panic!("Error: {}", e),
                // No more frames, break out of loop
                None => break,
            }
            .expect("frame is OK");
            output.push(data);
        }

        let mut actual_output = Vec::new();
        output
            .reader()
            .read_to_end(&mut actual_output)
            .expect("Doesn't cause IO errors");
        // Verify data is compressed as expected
        assert_eq!(COMPRESSED_OUTPUT, actual_output);
    }
}
