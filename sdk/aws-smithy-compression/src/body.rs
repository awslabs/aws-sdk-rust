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
            is_end_stream: bool,
        }
    }

    impl<CR> CompressedBody<SdkBody, CR> {
        /// Given an [`SdkBody`] and a `Box<dyn CompressRequest>`, create a new `CompressedBody<SdkBody, CR>`.
        pub fn new(body: SdkBody, compress_request: CR) -> Self {
            Self {
                body,
                compress_request,
                is_end_stream: false,
            }
        }
    }

    /// Support for the `http-body-1-0` and `http-1-0` crates.
    pub mod http_body_1_x {
        use crate::body::compress::CompressedBody;
        use crate::http::CompressRequest;
        use aws_smithy_runtime_api::box_error::BoxError;
        use aws_smithy_types::body::SdkBody;
        use http_body_1x::{Body, Frame, SizeHint};
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
                    None => {
                        *this.is_end_stream = true;
                        None
                    }
                    other => other,
                })
            }

            fn is_end_stream(&self) -> bool {
                self.is_end_stream
            }

            fn size_hint(&self) -> SizeHint {
                // We can't return a hint because we don't know exactly how
                // compression will affect the content length
                SizeHint::default()
            }
        }
        impl CompressedBody<SdkBody, Box<dyn CompressRequest>> {
            /// Consumes this `CompressedBody` and returns an [`SdkBody`] containing the compressed data.
            ///
            /// This *requires* that the inner `SdkBody` is in-memory (i.e. not streaming). Otherwise, an error is returned.
            /// If compression fails, an error is returned.
            pub fn into_compressed_sdk_body(mut self) -> Result<SdkBody, BoxError> {
                let mut compressed_body = Vec::new();
                let bytes = self.body.bytes().ok_or_else(|| "`into_compressed_sdk_body` requires that the inner body is 'in-memory', but it was streaming".to_string())?;

                self.compress_request
                    .compress_bytes(bytes, &mut compressed_body)?;
                Ok(SdkBody::from(compressed_body))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::body::compress::CompressedBody;
    use crate::{CompressionAlgorithm, CompressionOptions};
    use aws_smithy_types::body::SdkBody;
    use bytes::Buf;
    use bytes_utils::SegmentedBuf;
    use std::io::Read;
    const UNCOMPRESSED_INPUT: &[u8] = b"hello world";
    const COMPRESSED_OUTPUT: &[u8] = &[
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 203, 72, 205, 201, 201, 87, 40, 207, 47, 202, 73, 1, 0,
        133, 17, 74, 13, 11, 0, 0, 0,
    ];

    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_body_is_compressed() {
        let compression_options = CompressionOptions::default()
            .with_min_compression_size_bytes(0)
            .unwrap();
        let compress_request =
            CompressionAlgorithm::Gzip.into_impl_http_body_1_x(&compression_options);
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
