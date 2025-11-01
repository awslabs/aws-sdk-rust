/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functionality for calculating the checksum of an HTTP body and emitting it as trailers.

use super::ChecksumCache;
use crate::http::HttpChecksum;

use aws_smithy_http::header::append_merge_header_maps;
use aws_smithy_types::body::SdkBody;

use http::HeaderMap;
use http_body::SizeHint;
use pin_project_lite::pin_project;

use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::warn;

pin_project! {
    /// A body-wrapper that will calculate the `InnerBody`'s checksum and emit it as a trailer.
    pub struct ChecksumBody<InnerBody> {
            #[pin]
            body: InnerBody,
            checksum: Option<Box<dyn HttpChecksum>>,
            cache: Option<ChecksumCache>
    }
}

impl ChecksumBody<SdkBody> {
    /// Given an `SdkBody` and a `Box<dyn HttpChecksum>`, create a new `ChecksumBody<SdkBody>`.
    pub fn new(body: SdkBody, checksum: Box<dyn HttpChecksum>) -> Self {
        Self {
            body,
            checksum: Some(checksum),
            cache: None,
        }
    }

    /// Configure a cache for this body.
    ///
    /// When used across multiple requests (e.g. retries) a cached checksum previously
    /// calculated will be favored if available.
    pub fn with_cache(self, cache: ChecksumCache) -> Self {
        Self {
            body: self.body,
            checksum: self.checksum,
            cache: Some(cache),
        }
    }
}

impl http_body::Body for ChecksumBody<SdkBody> {
    type Data = bytes::Bytes;
    type Error = aws_smithy_types::body::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let this = self.project();
        match this.checksum {
            Some(checksum) => {
                let poll_res = this.body.poll_data(cx);
                if let Poll::Ready(Some(Ok(data))) = &poll_res {
                    checksum.update(data);
                }

                poll_res
            }
            None => unreachable!("This can only fail if poll_data is called again after poll_trailers, which is invalid"),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        let this = self.project();
        let poll_res = this.body.poll_trailers(cx);

        if let Poll::Ready(Ok(maybe_inner_trailers)) = poll_res {
            let checksum_headers = if let Some(checksum) = this.checksum.take() {
                let calculated_headers = checksum.headers();

                if let Some(cache) = this.cache {
                    if let Some(cached_headers) = cache.get() {
                        if cached_headers != calculated_headers {
                            warn!(cached = ?cached_headers, calculated = ?calculated_headers, "calculated checksum differs from cached checksum!");
                        }
                        cached_headers
                    } else {
                        cache.set(calculated_headers.clone());
                        calculated_headers
                    }
                } else {
                    calculated_headers
                }
            } else {
                return Poll::Ready(Ok(None));
            };

            return match maybe_inner_trailers {
                Some(inner_trailers) => Poll::Ready(Ok(Some(append_merge_header_maps(
                    inner_trailers,
                    checksum_headers,
                )))),
                None => Poll::Ready(Ok(Some(checksum_headers))),
            };
        }

        poll_res
    }

    fn is_end_stream(&self) -> bool {
        // If inner body is finished and we've already consumed the checksum then we must be
        // at the end of the stream.
        self.body.is_end_stream() && self.checksum.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::ChecksumBody;
    use crate::{http::CRC_32_HEADER_NAME, ChecksumAlgorithm, CRC_32_NAME};
    use aws_smithy_types::base64;
    use aws_smithy_types::body::SdkBody;
    use bytes::Buf;
    use bytes_utils::SegmentedBuf;
    use http_body::Body;
    use std::fmt::Write;
    use std::io::Read;

    fn header_value_as_checksum_string(header_value: &http::HeaderValue) -> String {
        let decoded_checksum = base64::decode(header_value.to_str().unwrap()).unwrap();
        let decoded_checksum = decoded_checksum
            .into_iter()
            .fold(String::new(), |mut acc, byte| {
                write!(acc, "{byte:02X?}").expect("string will always be writeable");
                acc
            });

        format!("0x{decoded_checksum}")
    }

    #[tokio::test]
    async fn test_checksum_body() {
        let input_text = "This is some test text for an SdkBody";
        let body = SdkBody::from(input_text);
        let checksum = CRC_32_NAME
            .parse::<ChecksumAlgorithm>()
            .unwrap()
            .into_impl();
        let mut body = ChecksumBody::new(body, checksum);

        let mut output = SegmentedBuf::new();
        while let Some(buf) = body.data().await {
            output.push(buf.unwrap());
        }

        let mut output_text = String::new();
        output
            .reader()
            .read_to_string(&mut output_text)
            .expect("Doesn't cause IO errors");
        // Verify data is complete and unaltered
        assert_eq!(input_text, output_text);

        let trailers = body
            .trailers()
            .await
            .expect("checksum generation was without error")
            .expect("trailers were set");
        let checksum_trailer = trailers
            .get(CRC_32_HEADER_NAME)
            .expect("trailers contain crc32 checksum");
        let checksum_trailer = header_value_as_checksum_string(checksum_trailer);

        // Known correct checksum for the input "This is some test text for an SdkBody"
        assert_eq!("0x99B01F72", checksum_trailer);
    }
}
