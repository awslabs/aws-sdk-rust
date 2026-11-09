/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functionality for validating an HTTP body against a given precalculated checksum and emitting an
//! error if it doesn't match.

use crate::http::HttpChecksum;

use aws_smithy_types::body::SdkBody;

use bytes::Bytes;
use pin_project_lite::pin_project;

use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};

type Completion = Box<dyn FnOnce(ValidationEnd) + Send + Sync>;

fn complete(completion: &mut Option<Completion>, end: ValidationEnd) {
    if let Some(completion) = completion.take() {
        completion(end);
    }
}

/// Compares the calculated checksum with the precalculated one once the data has ended, and
/// reports the result to the completion callback.
fn verify(
    checksum: Box<dyn HttpChecksum>,
    precalculated_checksum: &Bytes,
    completion: &mut Option<Completion>,
) -> Result<(), aws_smithy_types::body::Error> {
    let actual_checksum = checksum.finalize();
    if *precalculated_checksum == actual_checksum {
        complete(
            completion,
            ValidationEnd::Matched {
                actual: actual_checksum,
            },
        );
        Ok(())
    } else {
        complete(
            completion,
            ValidationEnd::Mismatched {
                expected: precalculated_checksum.clone(),
                actual: actual_checksum.clone(),
            },
        );
        Err(Box::new(Error::ChecksumMismatch {
            expected: precalculated_checksum.clone(),
            actual: actual_checksum,
        }))
    }
}

pin_project! {
    /// A body-wrapper that will calculate the `InnerBody`'s checksum and emit an error if it
    /// doesn't match the precalculated checksum.
    pub struct ChecksumBody<InnerBody> {
        #[pin]
        inner: InnerBody,
        checksum: Option<Box<dyn HttpChecksum>>,
        precalculated_checksum: Bytes,
        completion: Option<Completion>,
    }

    impl<InnerBody> PinnedDrop for ChecksumBody<InnerBody> {
        fn drop(this: Pin<&mut Self>) {
            complete(this.project().completion, ValidationEnd::Dropped);
        }
    }
}

impl ChecksumBody<SdkBody> {
    /// Given an `SdkBody`, a `Box<dyn HttpChecksum>`, and a precalculated checksum represented
    /// as `Bytes`, create a new `ChecksumBody<SdkBody>`.
    pub fn new(
        body: SdkBody,
        checksum: Box<dyn HttpChecksum>,
        precalculated_checksum: Bytes,
    ) -> Self {
        Self {
            inner: body,
            checksum: Some(checksum),
            precalculated_checksum,
            completion: None,
        }
    }

    /// Calls `f` exactly once, when validation ends: when the data ends with a match, before the
    /// mismatch error is yielded, on an inner-body error, or on drop. The data ends at a trailers
    /// frame or at the end of the stream, whichever comes first.
    pub fn with_completion(
        mut self,
        f: impl FnOnce(ValidationEnd) + Send + Sync + 'static,
    ) -> Self {
        self.completion = Some(Box::new(f));
        self
    }

    fn poll_inner(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body_1x::Frame<Bytes>, aws_smithy_types::body::Error>>> {
        use http_body_1x::Body;

        let this = self.project();
        let checksum = this.checksum;

        match this.inner.poll_frame(cx) {
            Poll::Ready(Some(Ok(frame))) => {
                let Some(data) = frame.data_ref() else {
                    // A trailers frame ends the data, and `SdkBody` stops reading at it, so the
                    // checksum is verified here rather than at the end of the stream.
                    let verified = checksum.take().map_or(Ok(()), |checksum| {
                        verify(checksum, this.precalculated_checksum, this.completion)
                    });
                    return Poll::Ready(Some(verified.map(|()| frame)));
                };
                tracing::trace!(
                    "reading {} bytes from the body and updating the checksum calculation",
                    data.len()
                );
                let checksum = match checksum.as_mut() {
                    Some(checksum) => checksum,
                    None => {
                        unreachable!("The checksum must exist because it's only taken out once the data has ended.");
                    }
                };

                checksum.update(data);
                Poll::Ready(Some(Ok(frame)))
            }
            // Once the inner body has stopped returning data, check the checksum
            // and return an error if it doesn't match.
            Poll::Ready(None) => {
                tracing::trace!("finished reading from body, calculating final checksum");
                let checksum = match checksum.take() {
                    Some(checksum) => checksum,
                    None => {
                        // If the checksum was already taken and this was polled again anyways,
                        // then return nothing
                        return Poll::Ready(None);
                    }
                };

                Poll::Ready(
                    verify(checksum, this.precalculated_checksum, this.completion)
                        .err()
                        .map(Err),
                )
            }
            Poll::Ready(Some(Err(e))) => {
                complete(this.completion, ValidationEnd::Errored);
                Poll::Ready(Some(Err(e)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// How validation of a [`ChecksumBody`] ended.
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValidationEnd {
    /// The data ended and the checksum matched.
    Matched {
        /// The checksum calculated from the body, equal to the precalculated one.
        actual: Bytes,
    },
    /// The data ended and the checksum did not match.
    Mismatched {
        /// The precalculated checksum the body was validated against.
        expected: Bytes,
        /// The checksum calculated from the body.
        actual: Bytes,
    },
    /// The inner body returned an error before the data ended.
    Errored,
    /// The body was dropped before the data ended.
    Dropped,
}

/// Errors related to checksum calculation and validation
#[derive(Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// The actual checksum didn't match the expected checksum. The checksummed data has been
    /// altered since the expected checksum was calculated.
    ChecksumMismatch { expected: Bytes, actual: Bytes },
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::ChecksumMismatch { expected, actual } => write!(
                f,
                "body checksum mismatch. expected body checksum to be {} but it was {}",
                hex::encode(expected),
                hex::encode(actual)
            ),
        }
    }
}

impl std::error::Error for Error {}

impl http_body_1x::Body for ChecksumBody<SdkBody> {
    type Data = Bytes;
    type Error = aws_smithy_types::body::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body_1x::Frame<Self::Data>, Self::Error>>> {
        self.poll_inner(cx)
    }
}

#[cfg(test)]
mod tests {
    use crate::body::validate::{ChecksumBody, Error, ValidationEnd};
    use crate::ChecksumAlgorithm;
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::byte_stream::ByteStream;
    use bytes::{Buf, Bytes};
    use bytes_utils::SegmentedBuf;
    use http_body_1x::Frame;
    use http_body_util::BodyExt;
    use std::collections::VecDeque;
    use std::io::Read;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};
    use std::task::{Context, Poll};

    fn calculate_crc32_checksum(input: &str) -> Bytes {
        let checksum =
            crc_fast::checksum(crc_fast::CrcAlgorithm::Crc32IsoHdlc, input.as_bytes()) as u32;

        Bytes::copy_from_slice(&checksum.to_be_bytes())
    }

    #[tokio::test]
    async fn test_checksum_validated_body_errors_on_mismatch() {
        let input_text = "This is some test text for an SdkBody";
        let actual_checksum = calculate_crc32_checksum(input_text);
        let body = SdkBody::from(input_text);
        let non_matching_checksum = Bytes::copy_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let mut body = ChecksumBody::new(
            body,
            "crc32".parse::<ChecksumAlgorithm>().unwrap().into_impl(),
            non_matching_checksum.clone(),
        );

        while let Some(data) = body.frame().await {
            match data {
                Ok(_) => { /* Do nothing */ }
                Err(e) => {
                    match e.downcast_ref::<Error>().unwrap() {
                        Error::ChecksumMismatch { expected, actual } => {
                            assert_eq!(expected, &non_matching_checksum);
                            assert_eq!(actual, &actual_checksum);
                        }
                    }

                    return;
                }
            }
        }

        panic!("didn't hit expected error condition");
    }

    #[tokio::test]
    async fn test_checksum_validated_body_succeeds_on_match() {
        let input_text = "This is some test text for an SdkBody";
        let actual_checksum = calculate_crc32_checksum(input_text);
        let body = SdkBody::from(input_text);
        let http_checksum = "crc32".parse::<ChecksumAlgorithm>().unwrap().into_impl();
        let mut body = ChecksumBody::new(body, http_checksum, actual_checksum);

        let mut output = SegmentedBuf::new();
        while let Some(buf) = body.frame().await {
            let data = buf.unwrap().into_data().unwrap();
            output.push(data);
        }

        let mut output_text = String::new();
        output
            .reader()
            .read_to_string(&mut output_text)
            .expect("Doesn't cause IO errors");
        // Verify data is complete and unaltered
        assert_eq!(input_text, output_text);
    }

    const TEXT: &str = "This is some test text for an SdkBody";

    fn crc32() -> Box<dyn crate::http::HttpChecksum> {
        "crc32".parse::<ChecksumAlgorithm>().unwrap().into_impl()
    }

    /// The end a body of `TEXT` reports when its CRC32 matches.
    fn text_matched() -> ValidationEnd {
        ValidationEnd::Matched {
            actual: calculate_crc32_checksum(TEXT),
        }
    }

    /// Records every call to a completion callback.
    #[derive(Default)]
    struct Recorded(Arc<Mutex<Vec<ValidationEnd>>>);

    impl Recorded {
        fn completion(&self) -> impl FnOnce(ValidationEnd) + Send + Sync + 'static {
            let ends = self.0.clone();
            move |end| ends.lock().unwrap().push(end)
        }

        #[track_caller]
        fn assert_ends(&self, expected: &[ValidationEnd]) {
            assert_eq!(self.0.lock().unwrap().as_slice(), expected);
        }
    }

    /// Yields one data frame, then an error, and never reaches end of stream.
    struct FailsAfterFirstFrame(Option<Bytes>);

    impl http_body_1x::Body for FailsAfterFirstFrame {
        type Data = Bytes;
        type Error = std::io::Error;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
            Poll::Ready(Some(match self.0.take() {
                Some(data) => Ok(Frame::data(data)),
                None => Err(std::io::Error::other("connection reset")),
            }))
        }
    }

    #[tokio::test]
    async fn completion_reports_matched_at_end_of_stream() {
        let recorded = Recorded::default();
        let mut body =
            ChecksumBody::new(SdkBody::from(TEXT), crc32(), calculate_crc32_checksum(TEXT))
                .with_completion(recorded.completion());

        body.frame().await.unwrap().unwrap();
        recorded.assert_ends(&[]);
        assert!(body.frame().await.is_none());
        recorded.assert_ends(&[text_matched()]);

        drop(body);
        recorded.assert_ends(&[text_matched()]);
    }

    #[tokio::test]
    async fn completion_reports_mismatch_before_the_error_is_yielded() {
        let wrong_checksum = Bytes::from_static(&[0x00, 0x00, 0x00, 0x00]);
        let recorded = Recorded::default();
        let mut body = ChecksumBody::new(SdkBody::from(TEXT), crc32(), wrong_checksum.clone())
            .with_completion(recorded.completion());
        let mismatched = ValidationEnd::Mismatched {
            expected: wrong_checksum,
            actual: calculate_crc32_checksum(TEXT),
        };

        body.frame().await.unwrap().unwrap();
        let error = body.frame().await.unwrap().unwrap_err();
        assert!(error.downcast_ref::<Error>().is_some());
        recorded.assert_ends(std::slice::from_ref(&mismatched));

        drop(body);
        recorded.assert_ends(&[mismatched]);
    }

    #[tokio::test]
    async fn completion_reports_errored_when_the_inner_body_fails() {
        let inner = FailsAfterFirstFrame(Some(Bytes::from_static(b"partial")));
        let recorded = Recorded::default();
        let mut body = ChecksumBody::new(
            SdkBody::from_body_1_x(inner),
            crc32(),
            calculate_crc32_checksum("partial"),
        )
        .with_completion(recorded.completion());

        body.frame().await.unwrap().unwrap();
        body.frame().await.unwrap().unwrap_err();
        recorded.assert_ends(&[ValidationEnd::Errored]);

        drop(body);
        recorded.assert_ends(&[ValidationEnd::Errored]);
    }

    #[tokio::test]
    async fn completion_reports_dropped_when_the_body_is_dropped_before_its_end() {
        let recorded = Recorded::default();
        let mut body =
            ChecksumBody::new(SdkBody::from(TEXT), crc32(), calculate_crc32_checksum(TEXT))
                .with_completion(recorded.completion());

        body.frame().await.unwrap().unwrap();
        recorded.assert_ends(&[]);

        drop(body);
        recorded.assert_ends(&[ValidationEnd::Dropped]);
    }

    /// Yields its frames in order, then ends.
    struct Frames(VecDeque<Frame<Bytes>>);

    impl http_body_1x::Body for Frames {
        type Data = Bytes;
        type Error = std::io::Error;

        fn poll_frame(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
            Poll::Ready(self.0.pop_front().map(Ok))
        }
    }

    /// `TEXT` in one data frame, then a trailers frame.
    fn text_then_trailers() -> SdkBody {
        SdkBody::from_body_1_x(Frames(VecDeque::from([
            Frame::data(Bytes::from_static(TEXT.as_bytes())),
            Frame::trailers(http_1x::HeaderMap::new()),
        ])))
    }

    #[tokio::test]
    async fn completion_reports_matched_at_a_trailers_frame() {
        let recorded = Recorded::default();
        let mut body = ChecksumBody::new(
            text_then_trailers(),
            crc32(),
            calculate_crc32_checksum(TEXT),
        )
        .with_completion(recorded.completion());

        body.frame().await.unwrap().unwrap();
        let trailers = body.frame().await.unwrap().unwrap();
        assert!(trailers.is_trailers());
        recorded.assert_ends(&[text_matched()]);
    }

    #[tokio::test]
    async fn a_mismatch_at_a_trailers_frame_fails_the_read() {
        let wrong_checksum = Bytes::from_static(&[0x00, 0x00, 0x00, 0x00]);
        let recorded = Recorded::default();
        let body = ChecksumBody::new(text_then_trailers(), crc32(), wrong_checksum.clone())
            .with_completion(recorded.completion());

        // `SdkBody` stops reading at a trailers frame, so a reader never polls past it.
        let error = ByteStream::new(SdkBody::from_body_1_x(body))
            .collect()
            .await
            .unwrap_err();
        let source = std::error::Error::source(&error).expect("the body's error");
        assert!(source.downcast_ref::<Error>().is_some());
        recorded.assert_ends(&[ValidationEnd::Mismatched {
            expected: wrong_checksum,
            actual: calculate_crc32_checksum(TEXT),
        }]);
    }
}
