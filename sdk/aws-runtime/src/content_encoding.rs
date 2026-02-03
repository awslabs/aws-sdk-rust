/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::config_bag::{Storable, StoreReplace};
use bytes::{Bytes, BytesMut};
use pin_project_lite::pin_project;

use std::pin::Pin;
use std::task::{Context, Poll};

const CRLF: &str = "\r\n";
const CRLF_RAW: &[u8] = b"\r\n";

const CHUNK_TERMINATOR: &str = "0\r\n";
const CHUNK_TERMINATOR_RAW: &[u8] = b"0\r\n";

const TRAILER_SEPARATOR: &[u8] = b":";

/// Content encoding header value constants
pub mod header_value {
    /// Header value denoting "aws-chunked" encoding
    pub const AWS_CHUNKED: &str = "aws-chunked";
}

/// Options used when constructing an [`AwsChunkedBody`].
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct AwsChunkedBodyOptions {
    /// The total size of the stream. Because we only support unsigned encoding
    /// this implies that there will only be a single chunk containing the
    /// underlying payload.
    stream_length: u64,
    /// The length of each trailer sent within an `AwsChunkedBody`. Necessary in
    /// order to correctly calculate the total size of the body accurately.
    trailer_lengths: Vec<u64>,
    /// Whether the aws-chunked encoding is disabled. This could occur, for instance,
    /// if a user specifies a custom checksum, rendering aws-chunked encoding unnecessary.
    disabled: bool,
}

impl Storable for AwsChunkedBodyOptions {
    type Storer = StoreReplace<Self>;
}

impl AwsChunkedBodyOptions {
    /// Create a new [`AwsChunkedBodyOptions`].
    pub fn new(stream_length: u64, trailer_lengths: Vec<u64>) -> Self {
        Self {
            stream_length,
            trailer_lengths,
            disabled: false,
        }
    }

    fn total_trailer_length(&self) -> u64 {
        self.trailer_lengths.iter().sum::<u64>()
            // We need to account for a CRLF after each trailer name/value pair
            + (self.trailer_lengths.len() * CRLF.len()) as u64
    }

    /// Set the stream length in the options
    pub fn with_stream_length(mut self, stream_length: u64) -> Self {
        self.stream_length = stream_length;
        self
    }

    /// Append a trailer length to the options
    pub fn with_trailer_len(mut self, trailer_len: u64) -> Self {
        self.trailer_lengths.push(trailer_len);
        self
    }

    /// Create a new [`AwsChunkedBodyOptions`] with aws-chunked encoding disabled.
    ///
    /// When the option is disabled, the body must not be wrapped in an `AwsChunkedBody`.
    pub fn disable_chunked_encoding() -> Self {
        Self {
            disabled: true,
            ..Default::default()
        }
    }

    /// Return whether aws-chunked encoding is disabled.
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// Return the length of the body after `aws-chunked` encoding is applied
    pub fn encoded_length(&self) -> u64 {
        let mut length = 0;
        if self.stream_length != 0 {
            length += get_unsigned_chunk_bytes_length(self.stream_length);
        }

        // End chunk
        length += CHUNK_TERMINATOR.len() as u64;

        // Trailers
        for len in self.trailer_lengths.iter() {
            length += len + CRLF.len() as u64;
        }

        // Encoding terminator
        length += CRLF.len() as u64;

        length
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AwsChunkedBodyState {
    /// Write out the size of the chunk that will follow. Then, transition into the
    /// `WritingChunk` state.
    WritingChunkSize,
    /// Write out the next chunk of data. Multiple polls of the inner body may need to occur before
    /// all data is written out. Once there is no more data to write, transition into the
    /// `WritingTrailers` state.
    WritingChunk,
    /// Write out all trailers associated with this `AwsChunkedBody` and then transition into the
    /// `Closed` state.
    WritingTrailers,
    /// This is the final state. Write out the body terminator and then remain in this state.
    Closed,
}

pin_project! {
    /// A request body compatible with `Content-Encoding: aws-chunked`. This implementation is only
    /// capable of writing a single chunk and does not support signed chunks.
    ///
    /// Chunked-Body grammar is defined in [ABNF] as:
    ///
    /// ```txt
    /// Chunked-Body    = *chunk
    ///                   last-chunk
    ///                   chunked-trailer
    ///                   CRLF
    ///
    /// chunk           = chunk-size CRLF chunk-data CRLF
    /// chunk-size      = 1*HEXDIG
    /// last-chunk      = 1*("0") CRLF
    /// chunked-trailer = *( entity-header CRLF )
    /// entity-header   = field-name ":" OWS field-value OWS
    /// ```
    /// For more info on what the abbreviations mean, see https://datatracker.ietf.org/doc/html/rfc7230#section-1.2
    ///
    /// [ABNF]:https://en.wikipedia.org/wiki/Augmented_Backus%E2%80%93Naur_form
    #[derive(Debug)]
    pub struct AwsChunkedBody<InnerBody> {
        #[pin]
        inner: InnerBody,
        #[pin]
        state: AwsChunkedBodyState,
        options: AwsChunkedBodyOptions,
        inner_body_bytes_read_so_far: usize,
    }
}

impl<Inner> AwsChunkedBody<Inner> {
    /// Wrap the given body in an outer body compatible with `Content-Encoding: aws-chunked`
    pub fn new(body: Inner, options: AwsChunkedBodyOptions) -> Self {
        Self {
            inner: body,
            state: AwsChunkedBodyState::WritingChunkSize,
            options,
            inner_body_bytes_read_so_far: 0,
        }
    }
}

#[cfg(feature = "http-02x")]
impl<Inner> http_body_04x::Body for AwsChunkedBody<Inner>
where
    Inner: http_body_04x::Body<Data = Bytes, Error = aws_smithy_types::body::Error>,
{
    type Data = Bytes;
    type Error = aws_smithy_types::body::Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        tracing::trace!(state = ?self.state, "polling AwsChunkedBody");
        let mut this = self.project();

        match *this.state {
            AwsChunkedBodyState::WritingChunkSize => {
                if this.options.stream_length == 0 {
                    // If the stream is empty, we skip to writing trailers after writing the CHUNK_TERMINATOR.
                    *this.state = AwsChunkedBodyState::WritingTrailers;
                    tracing::trace!("stream is empty, writing chunk terminator");
                    Poll::Ready(Some(Ok(Bytes::from([CHUNK_TERMINATOR].concat()))))
                } else {
                    *this.state = AwsChunkedBodyState::WritingChunk;
                    // A chunk must be prefixed by chunk size in hexadecimal
                    let chunk_size = format!("{:X?}{CRLF}", this.options.stream_length);
                    tracing::trace!(%chunk_size, "writing chunk size");
                    let chunk_size = Bytes::from(chunk_size);
                    Poll::Ready(Some(Ok(chunk_size)))
                }
            }
            AwsChunkedBodyState::WritingChunk => match this.inner.poll_data(cx) {
                Poll::Ready(Some(Ok(data))) => {
                    tracing::trace!(len = data.len(), "writing chunk data");
                    *this.inner_body_bytes_read_so_far += data.len();
                    Poll::Ready(Some(Ok(data)))
                }
                Poll::Ready(None) => {
                    let actual_stream_length = *this.inner_body_bytes_read_so_far as u64;
                    let expected_stream_length = this.options.stream_length;
                    if actual_stream_length != expected_stream_length {
                        let err = Box::new(AwsChunkedBodyError::StreamLengthMismatch {
                            actual: actual_stream_length,
                            expected: expected_stream_length,
                        });
                        return Poll::Ready(Some(Err(err)));
                    };

                    tracing::trace!("no more chunk data, writing CRLF and chunk terminator");
                    *this.state = AwsChunkedBodyState::WritingTrailers;
                    // Since we wrote chunk data, we end it with a CRLF and since we only write
                    // a single chunk, we write the CHUNK_TERMINATOR immediately after
                    Poll::Ready(Some(Ok(Bytes::from([CRLF, CHUNK_TERMINATOR].concat()))))
                }
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Pending => Poll::Pending,
            },
            AwsChunkedBodyState::WritingTrailers => {
                return match this.inner.poll_trailers(cx) {
                    Poll::Ready(Ok(trailers)) => {
                        *this.state = AwsChunkedBodyState::Closed;
                        let expected_length =
                            http_02x_utils::total_rendered_length_of_trailers(trailers.as_ref());
                        let actual_length = this.options.total_trailer_length();

                        if expected_length != actual_length {
                            let err =
                                Box::new(AwsChunkedBodyError::ReportedTrailerLengthMismatch {
                                    actual: actual_length,
                                    expected: expected_length,
                                });
                            return Poll::Ready(Some(Err(err)));
                        }

                        let mut trailers = http_02x_utils::trailers_as_aws_chunked_bytes(
                            trailers,
                            actual_length + 1,
                        );
                        // Insert the final CRLF to close the body
                        trailers.extend_from_slice(CRLF.as_bytes());

                        Poll::Ready(Some(Ok(trailers.into())))
                    }
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
                };
            }
            AwsChunkedBodyState::Closed => Poll::Ready(None),
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<http_02x::HeaderMap<http_02x::HeaderValue>>, Self::Error>> {
        // Trailers were already appended to the body because of the content encoding scheme
        Poll::Ready(Ok(None))
    }

    fn is_end_stream(&self) -> bool {
        self.state == AwsChunkedBodyState::Closed
    }

    fn size_hint(&self) -> http_body_04x::SizeHint {
        http_body_04x::SizeHint::with_exact(self.options.encoded_length())
    }
}

/// Utility functions to help with the [http_body_04x::Body] trait implementation
#[cfg(feature = "http-02x")]
mod http_02x_utils {
    use super::{CRLF, TRAILER_SEPARATOR};
    use bytes::BytesMut;
    use http_02x::HeaderMap;

    /// Writes trailers out into a `string` and then converts that `String` to a `Bytes` before
    /// returning.
    ///
    /// - Trailer names are separated by a single colon only, no space.
    /// - Trailer names with multiple values will be written out one line per value, with the name
    ///   appearing on each line.
    pub(super) fn trailers_as_aws_chunked_bytes(
        trailer_map: Option<HeaderMap>,
        estimated_length: u64,
    ) -> BytesMut {
        if let Some(trailer_map) = trailer_map {
            let mut current_header_name = None;
            let mut trailers =
                BytesMut::with_capacity(estimated_length.try_into().unwrap_or_default());

            for (header_name, header_value) in trailer_map.into_iter() {
                // When a header has multiple values, the name only comes up in iteration the first time
                // we see it. Therefore, we need to keep track of the last name we saw and fall back to
                // it when `header_name == None`.
                current_header_name = header_name.or(current_header_name);

                // In practice, this will always exist, but `if let` is nicer than unwrap
                if let Some(header_name) = current_header_name.as_ref() {
                    trailers.extend_from_slice(header_name.as_ref());
                    trailers.extend_from_slice(TRAILER_SEPARATOR);
                    trailers.extend_from_slice(header_value.as_bytes());
                    trailers.extend_from_slice(CRLF.as_bytes());
                }
            }

            trailers
        } else {
            BytesMut::new()
        }
    }

    /// Given an optional `HeaderMap`, calculate the total number of bytes required to represent the
    /// `HeaderMap`. If no `HeaderMap` is given as input, return 0.
    ///
    /// - Trailer names are separated by a single colon only, no space.
    /// - Trailer names with multiple values will be written out one line per value, with the name
    ///   appearing on each line.
    pub(super) fn total_rendered_length_of_trailers(trailer_map: Option<&HeaderMap>) -> u64 {
        match trailer_map {
            Some(trailer_map) => trailer_map
                .iter()
                .map(|(trailer_name, trailer_value)| {
                    trailer_name.as_str().len()
                        + TRAILER_SEPARATOR.len()
                        + trailer_value.len()
                        + CRLF.len()
                })
                .sum::<usize>() as u64,
            None => 0,
        }
    }
}

const UNREACHABLE_STATES: &str = "These states already short circuited";

/// Implementing the [http_body_1x::Body] trait
impl<Inner> http_body_1x::Body for AwsChunkedBody<Inner>
where
    Inner: http_body_1x::Body<Data = Bytes, Error = aws_smithy_types::body::Error>,
{
    type Data = Bytes;
    type Error = aws_smithy_types::body::Error;

    fn is_end_stream(&self) -> bool {
        self.state == AwsChunkedBodyState::Closed
    }

    fn size_hint(&self) -> http_body_1x::SizeHint {
        http_body_1x::SizeHint::with_exact(self.options.encoded_length())
    }

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body_1x::Frame<Self::Data>, Self::Error>>> {
        tracing::trace!(state = ?self.state, "polling AwsChunkedBody");
        let mut this = self.project();

        // Both `WritingChunkSize` and `Closed` states short circuit without polling the inner body

        // Initial setup, we do not poll the inner body here
        if *this.state == AwsChunkedBodyState::WritingChunkSize {
            if this.options.stream_length == 0 {
                // If the stream is empty, we skip to writing trailers after writing the CHUNK_TERMINATOR.
                tracing::trace!("stream is empty, writing chunk terminator");
                let frame = http_body_1x::Frame::data(Bytes::from(CHUNK_TERMINATOR));
                *this.state = AwsChunkedBodyState::WritingTrailers;
                return Poll::Ready(Some(Ok(frame)));
            } else {
                // A chunk must be prefixed by chunk size in hexadecimal
                let chunk_size = format!(
                    "{:X?}{}",
                    this.options.stream_length,
                    std::str::from_utf8(CRLF_RAW).unwrap()
                );
                tracing::trace!(%chunk_size, "writing chunk size");
                let chunk_size = http_body_1x::Frame::data(Bytes::from(chunk_size));
                *this.state = AwsChunkedBodyState::WritingChunk;
                return Poll::Ready(Some(Ok(chunk_size)));
            }
        }

        // Polled after completion
        if *this.state == AwsChunkedBodyState::Closed {
            return Poll::Ready(None);
        }

        // For all other states we must poll the inner body
        let maybe_frame = this.inner.poll_frame(cx);
        tracing::trace!(poll_state = ?maybe_frame, "Polling InnerBody");

        match maybe_frame {
            Poll::Ready(Some(Ok(frame))) => match *this.state {
                // Both data chunks and trailers are written as Frame::data so we treat these states similarly
                // Importantly we cannot know that the body data of the InnerBody is exhausted until we see a
                // trailer frame or a Poll::Ready(None)
                AwsChunkedBodyState::WritingChunk => {
                    if frame.is_data() {
                        let data = frame.data_ref().expect("Data frame has data");
                        tracing::trace!(len = data.len(), "Writing chunk data");
                        *this.inner_body_bytes_read_so_far += data.len();
                        Poll::Ready(Some(Ok(frame)))
                    } else {
                        tracing::trace!(
                            "No more chunk data, writing CRLF + CHUNK_TERMINATOR to end the data, and the first trailer frame"
                        );

                        // We exhausted the body data, now check if the length is correct
                        if let Err(poll_stream_len_err) =
                            http_1x_utils::check_for_stream_length_mismatch(
                                *this.inner_body_bytes_read_so_far as u64,
                                this.options.stream_length,
                            )
                        {
                            return poll_stream_len_err;
                        }

                        *this.state = AwsChunkedBodyState::WritingTrailers;
                        let trailers = frame.trailers_ref();

                        // NOTE: there is a subtle logic bug here (which is present in the http-02x implementation as well)
                        // The check for this error assumes that all trailers will come in a single trailer frame. Currently
                        // I believe this will always be the case since the only thing we send trailers for in AwsChunked is
                        // streaming checksums and that is a single trailer value. But it might not always be true. We should
                        // fix this bug when we update the behavior here to match the actual spec.
                        // The fix probably looks like returning Poll::Pending while we buffer all of the trailers and then
                        // comparing the actual length to the expected length before returning a final frame containing all
                        // of the trailers.
                        let actual_length: u64 =
                            http_1x_utils::total_rendered_length_of_trailers(trailers);
                        let expected_length = this.options.total_trailer_length();
                        if expected_length != actual_length {
                            let err =
                                Box::new(AwsChunkedBodyError::ReportedTrailerLengthMismatch {
                                    actual: actual_length,
                                    expected: expected_length,
                                });
                            return Poll::Ready(Some(Err(err)));
                        }

                        // Capacity = actual_length (in case all of the trailers specified in  come in AwsChunkedBodyOptions
                        // come in the first trailer frame which is going to be the case most of the time in practice) + 7
                        // (2 + 3) for the initial CRLF + CHUNK_TERMINATOR to end the chunked data + 2 for the final CRLF
                        // ending the trailers section.
                        let mut buf = BytesMut::with_capacity(actual_length as usize + 7);
                        // End the final data chunk
                        buf.extend_from_slice(&[CRLF_RAW, CHUNK_TERMINATOR_RAW].concat());

                        // We transform the trailers into raw bytes. We can't write them with Frame::trailers
                        // since we must include the CRLF + CHUNK_TERMINATOR that end the body and the CRLFs
                        // after each trailer, so we write them as Frame::data
                        let trailers = http_1x_utils::trailers_as_aws_chunked_bytes(trailers, buf);
                        Poll::Ready(Some(Ok(http_body_1x::Frame::data(trailers.into()))))
                    }
                }
                AwsChunkedBodyState::WritingTrailers => {
                    let trailers = frame.trailers_ref();
                    let actual_length: u64 =
                        http_1x_utils::total_rendered_length_of_trailers(trailers);
                    let buf = BytesMut::with_capacity(actual_length as usize + 7);
                    let trailers = http_1x_utils::trailers_as_aws_chunked_bytes(trailers, buf);
                    Poll::Ready(Some(Ok(http_body_1x::Frame::data(trailers.into()))))
                }
                AwsChunkedBodyState::Closed | AwsChunkedBodyState::WritingChunkSize => {
                    unreachable!("{}", UNREACHABLE_STATES)
                }
            },
            // InnerBody data exhausted, add finalizing bytes depending on current state
            Poll::Ready(None) => {
                let trailers = match *this.state {
                    AwsChunkedBodyState::WritingChunk => {
                        // We exhausted the body data, now check if the length is correct
                        if let Err(poll_stream_len_err) =
                            http_1x_utils::check_for_stream_length_mismatch(
                                *this.inner_body_bytes_read_so_far as u64,
                                this.options.stream_length,
                            )
                        {
                            return poll_stream_len_err;
                        }

                        // Since we exhausted the body data, but are still in the WritingChunk state we did
                        // not poll any trailer frames and we write the CRLF + Chunk terminator to begin the
                        // trailer section plus a single final CRLF to end the (empty) trailer section
                        let mut trailers = BytesMut::with_capacity(7);
                        trailers.extend_from_slice(
                            &[CRLF_RAW, CHUNK_TERMINATOR_RAW, CRLF_RAW].concat(),
                        );
                        trailers
                    }
                    AwsChunkedBodyState::WritingTrailers => {
                        let mut trailers = BytesMut::with_capacity(2);
                        trailers.extend_from_slice(CRLF_RAW);
                        trailers
                    }
                    AwsChunkedBodyState::Closed | AwsChunkedBodyState::WritingChunkSize => {
                        unreachable!("{}", UNREACHABLE_STATES)
                    }
                };

                let frame = http_body_1x::Frame::data(trailers.into());
                *this.state = AwsChunkedBodyState::Closed;
                Poll::Ready(Some(Ok(frame)))
            }
            // Passthrough states
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
        }
    }
}
/// Utility functions to help with the [http_body_1x::Body] trait implementation
mod http_1x_utils {
    use std::task::Poll;

    use super::{CRLF_RAW, TRAILER_SEPARATOR};
    use bytes::{Bytes, BytesMut};
    use http_1x::{HeaderMap, HeaderName};

    /// Writes trailers out into a `string` and then converts that `String` to a `Bytes` before
    /// returning.
    ///
    /// - Trailer names are separated by a single colon only, no space.
    /// - Trailer names with multiple values will be written out one line per value, with the name
    ///   appearing on each line.
    pub(super) fn trailers_as_aws_chunked_bytes(
        trailer_map: Option<&HeaderMap>,
        mut buffer: BytesMut,
    ) -> BytesMut {
        if let Some(trailer_map) = trailer_map {
            let mut current_header_name: Option<HeaderName> = None;

            for (header_name, header_value) in trailer_map.clone().into_iter() {
                // When a header has multiple values, the name only comes up in iteration the first time
                // we see it. Therefore, we need to keep track of the last name we saw and fall back to
                // it when `header_name == None`.
                current_header_name = header_name.or(current_header_name);

                // In practice, this will always exist, but `if let` is nicer than unwrap
                if let Some(header_name) = current_header_name.as_ref() {
                    buffer.extend_from_slice(header_name.as_ref());
                    buffer.extend_from_slice(TRAILER_SEPARATOR);
                    buffer.extend_from_slice(header_value.as_bytes());
                    buffer.extend_from_slice(CRLF_RAW);
                }
            }

            buffer
        } else {
            buffer
        }
    }

    /// Given an optional `HeaderMap`, calculate the total number of bytes required to represent the
    /// `HeaderMap`. If no `HeaderMap` is given as input, return 0.
    ///
    /// - Trailer names are separated by a single colon only, no space.
    /// - Trailer names with multiple values will be written out one line per value, with the name
    ///   appearing on each line.
    pub(super) fn total_rendered_length_of_trailers(trailer_map: Option<&HeaderMap>) -> u64 {
        match trailer_map {
            Some(trailer_map) => trailer_map
                .iter()
                .map(|(trailer_name, trailer_value)| {
                    trailer_name.as_str().len()
                        + TRAILER_SEPARATOR.len()
                        + trailer_value.len()
                        + CRLF_RAW.len()
                })
                .sum::<usize>() as u64,
            None => 0,
        }
    }

    /// This is an ugly return type, but in practice it just returns `Ok(())` if the values match
    /// and `Err(Poll::Ready(Some(Err(AwsChunkedBodyError::StreamLengthMismatch))))` if they don't
    #[allow(clippy::type_complexity)]
    pub(super) fn check_for_stream_length_mismatch(
        actual_stream_length: u64,
        expected_stream_length: u64,
    ) -> Result<(), Poll<Option<Result<http_body_1x::Frame<Bytes>, aws_smithy_types::body::Error>>>>
    {
        if actual_stream_length != expected_stream_length {
            let err = Box::new(super::AwsChunkedBodyError::StreamLengthMismatch {
                actual: actual_stream_length,
                expected: expected_stream_length,
            });
            return Err(Poll::Ready(Some(Err(err))));
        };

        Ok(())
    }
}

/// Errors related to `AwsChunkedBody`
#[derive(Debug)]
enum AwsChunkedBodyError {
    /// Error that occurs when the sum of `trailer_lengths` set when creating an `AwsChunkedBody` is
    /// not equal to the actual length of the trailers returned by the inner `http_body::Body`
    /// implementor. These trailer lengths are necessary in order to correctly calculate the total
    /// size of the body for setting the content length header.
    ReportedTrailerLengthMismatch { actual: u64, expected: u64 },
    /// Error that occurs when the `stream_length` set when creating an `AwsChunkedBody` is not
    /// equal to the actual length of the body returned by the inner `http_body::Body` implementor.
    /// `stream_length` must be correct in order to set an accurate content length header.
    StreamLengthMismatch { actual: u64, expected: u64 },
}

impl std::fmt::Display for AwsChunkedBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReportedTrailerLengthMismatch { actual, expected } => {
                write!(f, "When creating this AwsChunkedBody, length of trailers was reported as {expected}. However, when double checking during trailer encoding, length was found to be {actual} instead.")
            }
            Self::StreamLengthMismatch { actual, expected } => {
                write!(f, "When creating this AwsChunkedBody, stream length was reported as {expected}. However, when double checking during body encoding, length was found to be {actual} instead.")
            }
        }
    }
}

impl std::error::Error for AwsChunkedBodyError {}

// Used for finding how many hexadecimal digits it takes to represent a base 10 integer
fn int_log16<T>(mut i: T) -> u64
where
    T: std::ops::DivAssign + PartialOrd + From<u8> + Copy,
{
    let mut len = 0;
    let zero = T::from(0);
    let sixteen = T::from(16);

    while i > zero {
        i /= sixteen;
        len += 1;
    }

    len
}

fn get_unsigned_chunk_bytes_length(payload_length: u64) -> u64 {
    let hex_repr_len = int_log16(payload_length);
    hex_repr_len + CRLF.len() as u64 + payload_length + CRLF.len() as u64
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    #[cfg(feature = "http-02x")]
    mod http_02x_tests {
        use super::super::{
            http_02x_utils::{total_rendered_length_of_trailers, trailers_as_aws_chunked_bytes},
            AwsChunkedBody, AwsChunkedBodyOptions, CHUNK_TERMINATOR, CRLF,
        };

        use aws_smithy_types::body::SdkBody;
        use bytes::{Buf, Bytes};
        use bytes_utils::SegmentedBuf;
        use http_02x::{HeaderMap, HeaderValue};
        use http_body_04x::{Body, SizeHint};
        use pin_project_lite::pin_project;

        use std::io::Read;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use std::time::Duration;

        pin_project! {
            struct SputteringBody {
                parts: Vec<Option<Bytes>>,
                cursor: usize,
                delay_in_millis: u64,
            }
        }

        impl SputteringBody {
            fn len(&self) -> usize {
                self.parts.iter().flatten().map(|b| b.len()).sum()
            }
        }

        impl Body for SputteringBody {
            type Data = Bytes;
            type Error = aws_smithy_types::body::Error;

            fn poll_data(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
                if self.cursor == self.parts.len() {
                    return Poll::Ready(None);
                }

                let this = self.project();
                let delay_in_millis = *this.delay_in_millis;
                let next_part = this.parts.get_mut(*this.cursor).unwrap().take();

                match next_part {
                    None => {
                        *this.cursor += 1;
                        let waker = cx.waker().clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(delay_in_millis)).await;
                            waker.wake();
                        });
                        Poll::Pending
                    }
                    Some(data) => {
                        *this.cursor += 1;
                        Poll::Ready(Some(Ok(data)))
                    }
                }
            }

            fn poll_trailers(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
            ) -> Poll<Result<Option<HeaderMap<HeaderValue>>, Self::Error>> {
                Poll::Ready(Ok(None))
            }

            fn is_end_stream(&self) -> bool {
                false
            }

            fn size_hint(&self) -> SizeHint {
                SizeHint::new()
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding() {
            let test_fut = async {
                let input_str = "Hello world";
                let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, Vec::new());
                let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

                let mut output = SegmentedBuf::new();
                while let Some(buf) = body.data().await {
                    output.push(buf.unwrap());
                }

                let mut actual_output = String::new();
                output
                    .reader()
                    .read_to_string(&mut actual_output)
                    .expect("Doesn't cause IO errors");

                let expected_output = "B\r\nHello world\r\n0\r\n\r\n";

                assert_eq!(expected_output, actual_output);
                assert!(
                    body.trailers()
                        .await
                        .expect("no errors occurred during trailer polling")
                        .is_none(),
                    "aws-chunked encoded bodies don't have normal HTTP trailers"
                );

                // You can insert a `tokio::time::sleep` here to verify the timeout works as intended
            };

            let timeout_duration = Duration::from_secs(3);
            if tokio::time::timeout(timeout_duration, test_fut)
                .await
                .is_err()
            {
                panic!("test_aws_chunked_encoding timed out after {timeout_duration:?}");
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding_sputtering_body() {
            let test_fut = async {
                let input = SputteringBody {
                    parts: vec![
                        Some(Bytes::from_static(b"chunk 1, ")),
                        None,
                        Some(Bytes::from_static(b"chunk 2, ")),
                        Some(Bytes::from_static(b"chunk 3, ")),
                        None,
                        None,
                        Some(Bytes::from_static(b"chunk 4, ")),
                        Some(Bytes::from_static(b"chunk 5, ")),
                        Some(Bytes::from_static(b"chunk 6")),
                    ],
                    cursor: 0,
                    delay_in_millis: 500,
                };
                let opts = AwsChunkedBodyOptions::new(input.len() as u64, Vec::new());
                let mut body = AwsChunkedBody::new(input, opts);

                let mut output = SegmentedBuf::new();
                while let Some(buf) = body.data().await {
                    output.push(buf.unwrap());
                }

                let mut actual_output = String::new();
                output
                    .reader()
                    .read_to_string(&mut actual_output)
                    .expect("Doesn't cause IO errors");

                let expected_output =
                    "34\r\nchunk 1, chunk 2, chunk 3, chunk 4, chunk 5, chunk 6\r\n0\r\n\r\n";

                assert_eq!(expected_output, actual_output);
                assert!(
                    body.trailers()
                        .await
                        .expect("no errors occurred during trailer polling")
                        .is_none(),
                    "aws-chunked encoded bodies don't have normal HTTP trailers"
                );
            };

            let timeout_duration = Duration::from_secs(3);
            if tokio::time::timeout(timeout_duration, test_fut)
                .await
                .is_err()
            {
                panic!(
                "test_aws_chunked_encoding_sputtering_body timed out after {timeout_duration:?}"
            );
            }
        }

        #[tokio::test]
        #[should_panic = "called `Result::unwrap()` on an `Err` value: ReportedTrailerLengthMismatch { actual: 44, expected: 0 }"]
        async fn test_aws_chunked_encoding_incorrect_trailer_length_panic() {
            let input_str = "Hello world";
            // Test body has no trailers, so this length is incorrect and will trigger an assert panic
            // When the panic occurs, it will actually expect a length of 44. This is because, when using
            // aws-chunked encoding, each trailer will end with a CRLF which is 2 bytes long.
            let wrong_trailer_len = 42;
            let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, vec![wrong_trailer_len]);
            let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

            // We don't care about the body contents but we have to read it all before checking for trailers
            while let Some(buf) = body.data().await {
                drop(buf.unwrap());
            }

            assert!(
                body.trailers()
                    .await
                    .expect("no errors occurred during trailer polling")
                    .is_none(),
                "aws-chunked encoded bodies don't have normal HTTP trailers"
            );
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding_empty_body() {
            let input_str = "";
            let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, Vec::new());
            let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

            let mut output = SegmentedBuf::new();
            while let Some(buf) = body.data().await {
                output.push(buf.unwrap());
            }

            let mut actual_output = String::new();
            output
                .reader()
                .read_to_string(&mut actual_output)
                .expect("Doesn't cause IO errors");

            let expected_output = [CHUNK_TERMINATOR, CRLF].concat();

            assert_eq!(expected_output, actual_output);
            assert!(
                body.trailers()
                    .await
                    .expect("no errors occurred during trailer polling")
                    .is_none(),
                "aws-chunked encoded bodies don't have normal HTTP trailers"
            );
        }

        #[tokio::test]
        async fn test_total_rendered_length_of_trailers() {
            let mut trailers = HeaderMap::new();

            trailers.insert("empty_value", HeaderValue::from_static(""));

            trailers.insert("single_value", HeaderValue::from_static("value 1"));

            trailers.insert("two_values", HeaderValue::from_static("value 1"));
            trailers.append("two_values", HeaderValue::from_static("value 2"));

            trailers.insert("three_values", HeaderValue::from_static("value 1"));
            trailers.append("three_values", HeaderValue::from_static("value 2"));
            trailers.append("three_values", HeaderValue::from_static("value 3"));

            let trailers = Some(trailers);
            let actual_length = total_rendered_length_of_trailers(trailers.as_ref());
            let expected_length =
                (trailers_as_aws_chunked_bytes(trailers, actual_length).len()) as u64;

            assert_eq!(expected_length, actual_length);
        }

        #[tokio::test]
        async fn test_total_rendered_length_of_empty_trailers() {
            let trailers = Some(HeaderMap::new());
            let actual_length = total_rendered_length_of_trailers(trailers.as_ref());
            let expected_length =
                (trailers_as_aws_chunked_bytes(trailers, actual_length).len()) as u64;

            assert_eq!(expected_length, actual_length);
        }
    }

    #[cfg(test)]
    mod http_1x_tests {
        use super::super::{
            http_1x_utils::{total_rendered_length_of_trailers, trailers_as_aws_chunked_bytes},
            AwsChunkedBody, AwsChunkedBodyOptions, CHUNK_TERMINATOR_RAW, CRLF_RAW,
        };

        use aws_smithy_types::body::SdkBody;
        use bytes::{Buf, Bytes, BytesMut};
        use bytes_utils::SegmentedBuf;
        use http_1x::{HeaderMap, HeaderValue};
        use http_body_1x::{Body, Frame, SizeHint};
        use http_body_util::BodyExt;
        use pin_project_lite::pin_project;

        use std::io::Read;
        use std::pin::Pin;
        use std::task::{Context, Poll};
        use std::time::Duration;

        pin_project! {
            struct SputteringBody {
                parts: Vec<Option<Bytes>>,
                cursor: usize,
                delay_in_millis: u64,
            }
        }

        impl SputteringBody {
            fn len(&self) -> usize {
                self.parts.iter().flatten().map(|b| b.len()).sum()
            }
        }

        impl Body for SputteringBody {
            type Data = Bytes;
            type Error = aws_smithy_types::body::Error;

            fn poll_frame(
                self: Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
                if self.cursor == self.parts.len() {
                    return Poll::Ready(None);
                }

                let this = self.project();
                let delay_in_millis = *this.delay_in_millis;
                let next_part = this.parts.get_mut(*this.cursor).unwrap().take();

                match next_part {
                    None => {
                        *this.cursor += 1;
                        let waker = cx.waker().clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(delay_in_millis)).await;
                            waker.wake();
                        });
                        Poll::Pending
                    }
                    Some(data) => {
                        *this.cursor += 1;
                        let frame = Frame::data(data);
                        Poll::Ready(Some(Ok(frame)))
                    }
                }
            }

            fn is_end_stream(&self) -> bool {
                false
            }

            fn size_hint(&self) -> SizeHint {
                SizeHint::new()
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding() {
            let test_fut = async {
                let input_str = "Hello world";
                let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, vec![]);
                let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

                let mut output = SegmentedBuf::new();
                while let Some(Ok(buf)) = body.frame().await {
                    output.push(buf.into_data().unwrap());
                }

                let mut actual_output = String::new();
                output
                    .reader()
                    .read_to_string(&mut actual_output)
                    .expect("Doesn't cause IO errors");

                let expected_output = "B\r\nHello world\r\n0\r\n\r\n";

                assert_eq!(expected_output, actual_output);

                // You can insert a `tokio::time::sleep` here to verify the timeout works as intended
            };

            let timeout_duration = Duration::from_secs(3);
            if tokio::time::timeout(timeout_duration, test_fut)
                .await
                .is_err()
            {
                panic!("test_aws_chunked_encoding timed out after {timeout_duration:?}");
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding_sputtering_body() {
            let test_fut = async {
                let input = SputteringBody {
                    parts: vec![
                        Some(Bytes::from_static(b"chunk 1, ")),
                        None,
                        Some(Bytes::from_static(b"chunk 2, ")),
                        Some(Bytes::from_static(b"chunk 3, ")),
                        None,
                        None,
                        Some(Bytes::from_static(b"chunk 4, ")),
                        Some(Bytes::from_static(b"chunk 5, ")),
                        Some(Bytes::from_static(b"chunk 6")),
                    ],
                    cursor: 0,
                    delay_in_millis: 500,
                };
                let opts = AwsChunkedBodyOptions::new(input.len() as u64, vec![]);
                let mut body = AwsChunkedBody::new(input, opts);

                let mut output = SegmentedBuf::new();
                while let Some(Ok(buf)) = body.frame().await {
                    output.push(buf.into_data().unwrap());
                }

                let mut actual_output = String::new();
                output
                    .reader()
                    .read_to_string(&mut actual_output)
                    .expect("Doesn't cause IO errors");

                let expected_output =
                    "34\r\nchunk 1, chunk 2, chunk 3, chunk 4, chunk 5, chunk 6\r\n0\r\n\r\n";

                assert_eq!(expected_output, actual_output);
            };

            let timeout_duration = Duration::from_secs(3);
            if tokio::time::timeout(timeout_duration, test_fut)
                .await
                .is_err()
            {
                panic!(
                "test_aws_chunked_encoding_sputtering_body timed out after {timeout_duration:?}"
            );
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding_incorrect_trailer_length_panic() {
            let input_str = "Hello world";
            // Test body has no trailers, so this length is incorrect and will trigger an assert panic
            // When the panic occurs, it will actually expect a length of 44. This is because, when using
            // aws-chunked encoding, each trailer will end with a CRLF which is 2 bytes long.
            let wrong_trailer_len = 42;
            let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, vec![wrong_trailer_len]);
            let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

            // We don't care about the body contents but we have to read it all before checking for trailers
            while let Some(Ok(frame)) = body.frame().await {
                assert!(!frame.is_trailers());
            }
        }

        #[tokio::test]
        async fn test_aws_chunked_encoding_empty_body() {
            let input_str = "";
            let opts = AwsChunkedBodyOptions::new(input_str.len() as u64, vec![]);
            let mut body = AwsChunkedBody::new(SdkBody::from(input_str), opts);

            let mut output = SegmentedBuf::new();
            while let Some(Ok(frame)) = body.frame().await {
                output.push(frame.into_data().unwrap());
            }

            let mut actual_output = String::new();
            output
                .reader()
                .read_to_string(&mut actual_output)
                .expect("Doesn't cause IO errors");

            let actual_output = std::str::from_utf8(actual_output.as_bytes()).unwrap();
            let expected_output = [CHUNK_TERMINATOR_RAW, CRLF_RAW].concat();
            let expected_output = std::str::from_utf8(&expected_output).unwrap();

            assert_eq!(expected_output, actual_output);
        }

        #[tokio::test]
        async fn test_total_rendered_length_of_trailers() {
            let mut trailers = HeaderMap::new();

            trailers.insert("empty_value", HeaderValue::from_static(""));

            trailers.insert("single_value", HeaderValue::from_static("value 1"));

            trailers.insert("two_values", HeaderValue::from_static("value 1"));
            trailers.append("two_values", HeaderValue::from_static("value 2"));

            trailers.insert("three_values", HeaderValue::from_static("value 1"));
            trailers.append("three_values", HeaderValue::from_static("value 2"));
            trailers.append("three_values", HeaderValue::from_static("value 3"));

            let trailers = Some(&trailers);
            let actual_length = total_rendered_length_of_trailers(trailers);
            let buf = BytesMut::with_capacity(actual_length as usize);
            let expected_length = (trailers_as_aws_chunked_bytes(trailers, buf).len()) as u64;

            assert_eq!(expected_length, actual_length);
        }

        #[tokio::test]
        async fn test_total_rendered_length_of_empty_trailers() {
            let header_map = HeaderMap::new();
            let trailers = Some(&header_map);
            let actual_length = total_rendered_length_of_trailers(trailers);
            let buf = BytesMut::with_capacity(actual_length as usize);
            let expected_length = (trailers_as_aws_chunked_bytes(trailers, buf).len()) as u64;

            assert_eq!(expected_length, actual_length);
        }
    }
}
