/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Event Stream message frame types and serialization/deserialization logic.

use crate::buf::count::CountBuf;
use crate::buf::crc::{CrcBuf, CrcBufMut};
use crate::error::{Error, ErrorKind};
pub use aws_smithy_types::event_stream::DeferredSignerSender;
use aws_smithy_types::event_stream::{DeferredSignerReceiver, Header, HeaderValue, Message};
use aws_smithy_types::str_bytes::StrBytes;
use aws_smithy_types::DateTime;
use bytes::{Buf, BufMut};
use std::fmt;
use std::mem::size_of;

const PRELUDE_LENGTH_BYTES: u32 = 3 * size_of::<u32>() as u32;
const PRELUDE_LENGTH_BYTES_USIZE: usize = PRELUDE_LENGTH_BYTES as usize;
const MESSAGE_CRC_LENGTH_BYTES: u32 = size_of::<u32>() as u32;
const MAX_HEADER_NAME_LEN: usize = 255;
// String and byte array values are prefixed with a 2-byte length, but the protocol limits them to
// `i16::MAX` rather than the `u16::MAX` that field could hold (matching the CRT, Java, and Go).
const MAX_HEADER_VALUE_LEN: usize = i16::MAX as usize;
const MIN_HEADER_LEN: usize = 2;

pub(crate) const TYPE_TRUE: u8 = 0;
pub(crate) const TYPE_FALSE: u8 = 1;
pub(crate) const TYPE_BYTE: u8 = 2;
pub(crate) const TYPE_INT16: u8 = 3;
pub(crate) const TYPE_INT32: u8 = 4;
pub(crate) const TYPE_INT64: u8 = 5;
pub(crate) const TYPE_BYTE_ARRAY: u8 = 6;
pub(crate) const TYPE_STRING: u8 = 7;
pub(crate) const TYPE_TIMESTAMP: u8 = 8;
pub(crate) const TYPE_UUID: u8 = 9;

pub use aws_smithy_types::event_stream::{SignMessage, SignMessageError};

/// Deferred event stream signer to allow a signer to be wired up later.
///
/// HTTP request signing takes place after serialization, and the event stream
/// message stream body is established during serialization. Since event stream
/// signing may need context from the initial HTTP signing operation, this
/// [`DeferredSigner`] is needed to wire up the signer later in the request lifecycle.
///
/// This signer basically just establishes a MPSC channel so that the sender can
/// be placed in the request's config. Then the HTTP signer implementation can
/// retrieve the sender from that config and send an actual signing implementation
/// with all the context needed.
///
/// When an event stream implementation needs to sign a message, the first call to
/// sign will acquire a signing implementation off of the channel and cache it
/// for the remainder of the operation.
#[derive(Debug)]
pub struct DeferredSigner {
    rx: Option<DeferredSignerReceiver>,
    signer: Option<Box<dyn SignMessage + Send + Sync>>,
}

impl DeferredSigner {
    pub fn new() -> (Self, DeferredSignerSender) {
        let (rx, sender) = DeferredSignerSender::new();
        (
            Self {
                rx: Some(rx),
                signer: None,
            },
            sender,
        )
    }

    fn acquire(&mut self) -> &mut (dyn SignMessage + Send + Sync) {
        if self.signer.is_none() {
            self.signer = Some(
                self.rx
                    .take()
                    .expect("only taken once")
                    .recv::<Box<dyn SignMessage + Send + Sync>>()
                    // Fall back to NoOpSigner when no signer is sent (e.g., server-side
                    // event streams or tests that don't configure signing).
                    .unwrap_or_else(|_| Box::new(NoOpSigner {}) as _),
            );
        }
        self.signer.as_mut().unwrap().as_mut()
    }
}

impl SignMessage for DeferredSigner {
    fn sign(&mut self, message: Message) -> Result<Message, SignMessageError> {
        self.acquire().sign(message)
    }

    fn sign_empty(&mut self) -> Option<Result<Message, SignMessageError>> {
        self.acquire().sign_empty()
    }
}

#[derive(Debug)]
pub struct NoOpSigner {}
impl SignMessage for NoOpSigner {
    fn sign(&mut self, message: Message) -> Result<Message, SignMessageError> {
        Ok(message)
    }

    fn sign_empty(&mut self) -> Option<Result<Message, SignMessageError>> {
        None
    }
}

/// Converts a Smithy modeled Event Stream type into a [`Message`].
pub trait MarshallMessage: fmt::Debug {
    /// Smithy modeled input type to convert from.
    type Input;

    fn marshall(&self, input: Self::Input) -> Result<Message, Error>;
}

/// A successfully unmarshalled message that is either an `Event` or an `Error`.
#[derive(Debug)]
pub enum UnmarshalledMessage<T, E> {
    Event(T),
    Error(E),
}

/// Converts an Event Stream [`Message`] into a Smithy modeled type.
pub trait UnmarshallMessage: fmt::Debug {
    /// Smithy modeled type to convert into.
    type Output;
    /// Smithy modeled error to convert into.
    type Error;

    fn unmarshall(
        &self,
        message: &Message,
    ) -> Result<UnmarshalledMessage<Self::Output, Self::Error>, Error>;
}

macro_rules! read_value {
    ($buf:ident, $typ:ident, $size_typ:ident, $read_fn:ident) => {
        if $buf.remaining() >= size_of::<$size_typ>() {
            Ok(HeaderValue::$typ($buf.$read_fn()))
        } else {
            Err(ErrorKind::InvalidHeaderValue.into())
        }
    };
}

fn read_header_value_from<B: Buf>(mut buffer: B) -> Result<HeaderValue, Error> {
    let value_type = buffer.get_u8();
    match value_type {
        TYPE_TRUE => Ok(HeaderValue::Bool(true)),
        TYPE_FALSE => Ok(HeaderValue::Bool(false)),
        TYPE_BYTE => read_value!(buffer, Byte, i8, get_i8),
        TYPE_INT16 => read_value!(buffer, Int16, i16, get_i16),
        TYPE_INT32 => read_value!(buffer, Int32, i32, get_i32),
        TYPE_INT64 => read_value!(buffer, Int64, i64, get_i64),
        TYPE_BYTE_ARRAY | TYPE_STRING => {
            if buffer.remaining() > size_of::<u16>() {
                let len = buffer.get_u16() as usize;
                if buffer.remaining() < len {
                    return Err(ErrorKind::InvalidHeaderValue.into());
                }
                let bytes = buffer.copy_to_bytes(len);
                if value_type == TYPE_STRING {
                    Ok(HeaderValue::String(
                        bytes.try_into().map_err(|_| ErrorKind::InvalidUtf8String)?,
                    ))
                } else {
                    Ok(HeaderValue::ByteArray(bytes))
                }
            } else {
                Err(ErrorKind::InvalidHeaderValue.into())
            }
        }
        TYPE_TIMESTAMP => {
            if buffer.remaining() >= size_of::<i64>() {
                let epoch_millis = buffer.get_i64();
                Ok(HeaderValue::Timestamp(DateTime::from_millis(epoch_millis)))
            } else {
                Err(ErrorKind::InvalidHeaderValue.into())
            }
        }
        TYPE_UUID => read_value!(buffer, Uuid, u128, get_u128),
        _ => Err(ErrorKind::InvalidHeaderValueType(value_type).into()),
    }
}

fn write_header_value_to<B: BufMut>(value: &HeaderValue, mut buffer: B) -> Result<(), Error> {
    use HeaderValue::*;
    match value {
        Bool(val) => buffer.put_u8(if *val { TYPE_TRUE } else { TYPE_FALSE }),
        Byte(val) => {
            buffer.put_u8(TYPE_BYTE);
            buffer.put_i8(*val);
        }
        Int16(val) => {
            buffer.put_u8(TYPE_INT16);
            buffer.put_i16(*val);
        }
        Int32(val) => {
            buffer.put_u8(TYPE_INT32);
            buffer.put_i32(*val);
        }
        Int64(val) => {
            buffer.put_u8(TYPE_INT64);
            buffer.put_i64(*val);
        }
        ByteArray(val) => {
            buffer.put_u8(TYPE_BYTE_ARRAY);
            buffer.put_u16(header_value_len(val.len())?);
            buffer.put_slice(&val[..]);
        }
        String(val) => {
            buffer.put_u8(TYPE_STRING);
            buffer.put_u16(header_value_len(val.as_bytes().len())?);
            buffer.put_slice(&val.as_bytes()[..]);
        }
        Timestamp(time) => {
            buffer.put_u8(TYPE_TIMESTAMP);
            buffer.put_i64(
                time.to_millis()
                    .map_err(|_| ErrorKind::TimestampValueTooLarge(*time))?,
            );
        }
        Uuid(val) => {
            buffer.put_u8(TYPE_UUID);
            buffer.put_u128(*val);
        }
        _ => {
            panic!("matched on unexpected variant in `aws_smithy_types::event_stream::HeaderValue`")
        }
    }
    Ok(())
}

/// Reads a header from the given `buffer`.
fn read_header_from<B: Buf>(mut buffer: B) -> Result<(Header, usize), Error> {
    if buffer.remaining() < MIN_HEADER_LEN {
        return Err(ErrorKind::InvalidHeadersLength.into());
    }

    let mut counting_buf = CountBuf::new(&mut buffer);
    let name_len = counting_buf.get_u8();
    if name_len as usize >= counting_buf.remaining() {
        return Err(ErrorKind::InvalidHeaderNameLength.into());
    }

    let name: StrBytes = counting_buf
        .copy_to_bytes(name_len as usize)
        .try_into()
        .map_err(|_| ErrorKind::InvalidUtf8String)?;
    let value = read_header_value_from(&mut counting_buf)?;
    Ok((Header::new(name, value), counting_buf.into_count()))
}

/// Writes the header to the given `buffer`.
fn write_header_to<B: BufMut>(header: &Header, mut buffer: B) -> Result<(), Error> {
    if header.name().as_bytes().len() > MAX_HEADER_NAME_LEN {
        return Err(ErrorKind::InvalidHeaderNameLength.into());
    }

    buffer.put_u8(u8::try_from(header.name().as_bytes().len()).expect("bounds check above"));
    buffer.put_slice(&header.name().as_bytes()[..]);
    write_header_value_to(header.value(), buffer)
}

/// Writes the given `headers` to a `buffer`.
///
/// If any header fails to serialize, nothing is written to `buffer`.
pub fn write_headers_to<B: BufMut>(headers: &[Header], mut buffer: B) -> Result<(), Error> {
    let mut staged = Vec::with_capacity(headers.len());
    for header in headers {
        write_header_to(header, &mut staged)?;
    }
    buffer.put_slice(&staged);
    Ok(())
}

// Returns (total_len, header_len)
fn read_prelude_from<B: Buf>(mut buffer: B) -> Result<(u32, u32), Error> {
    let mut crc_buffer = CrcBuf::new(&mut buffer);

    // If the buffer doesn't have the entire, then error
    let total_len = crc_buffer.get_u32();
    if crc_buffer.remaining() + size_of::<u32>() < total_len as usize {
        return Err(ErrorKind::InvalidMessageLength.into());
    }

    // Validate the prelude
    let header_len = crc_buffer.get_u32();
    let (expected_crc, prelude_crc) = (crc_buffer.into_crc(), buffer.get_u32());
    if expected_crc != prelude_crc {
        return Err(ErrorKind::PreludeChecksumMismatch(expected_crc, prelude_crc).into());
    }
    // The header length can be 0 or >= 2, but must fit within the frame size
    if header_len == 1 || header_len > max_header_len(total_len)? {
        return Err(ErrorKind::InvalidHeadersLength.into());
    }
    Ok((total_len, header_len))
}

/// Reads a message from the given `buffer`. For streaming use cases, use
/// the [`MessageFrameDecoder`] instead of this.
pub fn read_message_from<B: Buf>(mut buffer: B) -> Result<Message, Error> {
    if buffer.remaining() < PRELUDE_LENGTH_BYTES_USIZE {
        return Err(ErrorKind::InvalidMessageLength.into());
    }

    // Calculate a CRC as we go and read the prelude
    let mut crc_buffer = CrcBuf::new(&mut buffer);
    let (total_len, header_len) = read_prelude_from(&mut crc_buffer)?;

    // Verify we have the full frame before continuing
    let remaining_len = total_len
        .checked_sub(PRELUDE_LENGTH_BYTES)
        .ok_or_else(|| Error::from(ErrorKind::InvalidMessageLength))?;
    if crc_buffer.remaining() < remaining_len as usize {
        return Err(ErrorKind::InvalidMessageLength.into());
    }

    // Read headers
    let mut header_bytes_read = 0;
    let mut headers = Vec::new();
    while header_bytes_read < header_len as usize {
        let (header, bytes_read) = read_header_from(&mut crc_buffer)?;
        header_bytes_read += bytes_read;
        if header_bytes_read > header_len as usize {
            return Err(ErrorKind::InvalidHeaderValue.into());
        }
        headers.push(header);
    }

    // Read payload
    let payload_len = payload_len(total_len, header_len)?;
    let payload = crc_buffer.copy_to_bytes(payload_len as usize);

    let expected_crc = crc_buffer.into_crc();
    let message_crc = buffer.get_u32();
    if expected_crc != message_crc {
        return Err(ErrorKind::MessageChecksumMismatch(expected_crc, message_crc).into());
    }

    Ok(Message::new_from_parts(headers, payload))
}

/// Writes the `message` to the given `buffer`.
pub fn write_message_to(message: &Message, buffer: &mut dyn BufMut) -> Result<(), Error> {
    let mut headers = Vec::new();
    for header in message.headers() {
        write_header_to(header, &mut headers)?;
    }

    let headers_len = checked(headers.len(), ErrorKind::HeadersTooLong.into())?;
    let payload_len = checked(message.payload().len(), ErrorKind::PayloadTooLong.into())?;
    let message_len = [
        PRELUDE_LENGTH_BYTES,
        headers_len,
        payload_len,
        MESSAGE_CRC_LENGTH_BYTES,
    ]
    .iter()
    .try_fold(0u32, |acc, v| {
        acc.checked_add(*v)
            .ok_or_else(|| Error::from(ErrorKind::MessageTooLong))
    })?;

    let mut crc_buffer = CrcBufMut::new(buffer);
    crc_buffer.put_u32(message_len);
    crc_buffer.put_u32(headers_len);
    crc_buffer.put_crc();
    crc_buffer.put(&headers[..]);
    crc_buffer.put(&message.payload()[..]);
    crc_buffer.put_crc();
    Ok(())
}

fn checked<T: TryFrom<U>, U>(from: U, err: Error) -> Result<T, Error> {
    T::try_from(from).map_err(|_| err)
}

/// Returns `len` as the length prefix for a string or byte array header value.
///
/// This must error rather than truncate: the frame's lengths and CRCs are computed from the bytes
/// actually written, so a frame with a truncated value length would still be valid, and a decoder
/// would read the rest of the value as further headers.
fn header_value_len(len: usize) -> Result<u16, Error> {
    if len > MAX_HEADER_VALUE_LEN {
        return Err(ErrorKind::HeaderValueTooLong.into());
    }
    // Lossless: `MAX_HEADER_VALUE_LEN` fits in a `u16`.
    Ok(len as u16)
}

fn max_header_len(total_len: u32) -> Result<u32, Error> {
    total_len
        .checked_sub(PRELUDE_LENGTH_BYTES + MESSAGE_CRC_LENGTH_BYTES)
        .ok_or_else(|| Error::from(ErrorKind::InvalidMessageLength))
}

fn payload_len(total_len: u32, header_len: u32) -> Result<u32, Error> {
    total_len
        .checked_sub(
            header_len
                .checked_add(PRELUDE_LENGTH_BYTES + MESSAGE_CRC_LENGTH_BYTES)
                .ok_or_else(|| Error::from(ErrorKind::InvalidHeadersLength))?,
        )
        .ok_or_else(|| Error::from(ErrorKind::InvalidMessageLength))
}

#[cfg(test)]
mod message_tests {
    use super::{
        read_message_from, MAX_HEADER_NAME_LEN, MAX_HEADER_VALUE_LEN, MESSAGE_CRC_LENGTH_BYTES,
        PRELUDE_LENGTH_BYTES, TYPE_BYTE, TYPE_INT16, TYPE_INT32, TYPE_INT64, TYPE_STRING,
        TYPE_TIMESTAMP, TYPE_UUID,
    };
    use crate::error::ErrorKind;
    use crate::frame::{write_headers_to, write_message_to, Header, HeaderValue, Message};
    use aws_smithy_types::str_bytes::StrBytes;
    use aws_smithy_types::DateTime;
    use bytes::Bytes;

    macro_rules! read_message_expect_err {
        ($bytes:expr, $err:pat) => {
            let result = read_message_from(&mut Bytes::from_static($bytes));
            let result = result.as_ref();
            assert!(result.is_err(), "Expected error, got {:?}", result);
            assert!(
                matches!(result.err().unwrap().kind(), $err),
                "Expected {}, got {:?}",
                stringify!($err),
                result
            );
        };
    }

    #[test]
    fn invalid_messages() {
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_string_value_length"),
            ErrorKind::InvalidHeaderValue
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_string_length_cut_off"),
            ErrorKind::InvalidHeaderValue
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_value_type"),
            ErrorKind::InvalidHeaderValueType(0x60)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_name_length"),
            ErrorKind::InvalidHeaderNameLength
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_headers_length"),
            ErrorKind::InvalidHeadersLength
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_prelude_checksum"),
            ErrorKind::PreludeChecksumMismatch(0x8BB495FB, 0xDEADBEEF)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_message_checksum"),
            ErrorKind::MessageChecksumMismatch(0x01a05860, 0xDEADBEEF)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_name_length_too_long"),
            ErrorKind::InvalidUtf8String
        );
    }

    #[test]
    fn read_message_no_headers() {
        // Test message taken from the CRT:
        // https://github.com/awslabs/aws-c-event-stream/blob/main/tests/message_deserializer_test.c
        let data: &'static [u8] = &[
            0x00, 0x00, 0x00, 0x1D, 0x00, 0x00, 0x00, 0x00, 0xfd, 0x52, 0x8c, 0x5a, 0x7b, 0x27,
            0x66, 0x6f, 0x6f, 0x27, 0x3a, 0x27, 0x62, 0x61, 0x72, 0x27, 0x7d, 0xc3, 0x65, 0x39,
            0x36,
        ];

        let result = read_message_from(&mut Bytes::from_static(data)).unwrap();
        assert_eq!(result.headers(), Vec::new());

        let expected_payload = b"{'foo':'bar'}";
        assert_eq!(expected_payload, result.payload().as_ref());
    }

    #[test]
    fn read_message_one_header() {
        // Test message taken from the CRT:
        // https://github.com/awslabs/aws-c-event-stream/blob/main/tests/message_deserializer_test.c
        let data: &'static [u8] = &[
            0x00, 0x00, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x20, 0x07, 0xFD, 0x83, 0x96, 0x0C, b'c',
            b'o', b'n', b't', b'e', b'n', b't', b'-', b't', b'y', b'p', b'e', 0x07, 0x00, 0x10,
            b'a', b'p', b'p', b'l', b'i', b'c', b'a', b't', b'i', b'o', b'n', b'/', b'j', b's',
            b'o', b'n', 0x7b, 0x27, 0x66, 0x6f, 0x6f, 0x27, 0x3a, 0x27, 0x62, 0x61, 0x72, 0x27,
            0x7d, 0x8D, 0x9C, 0x08, 0xB1,
        ];

        let result = read_message_from(&mut Bytes::from_static(data)).unwrap();
        assert_eq!(
            result.headers(),
            vec![Header::new(
                "content-type",
                HeaderValue::String("application/json".into())
            )]
        );

        let expected_payload = b"{'foo':'bar'}";
        assert_eq!(expected_payload, result.payload().as_ref());
    }

    #[test]
    fn read_all_headers_and_payload() {
        let message = include_bytes!("../test_data/valid_with_all_headers_and_payload");
        let result = read_message_from(&mut Bytes::from_static(message)).unwrap();
        assert_eq!(
            result.headers(),
            vec![
                Header::new("true", HeaderValue::Bool(true)),
                Header::new("false", HeaderValue::Bool(false)),
                Header::new("byte", HeaderValue::Byte(50)),
                Header::new("short", HeaderValue::Int16(20_000)),
                Header::new("int", HeaderValue::Int32(500_000)),
                Header::new("long", HeaderValue::Int64(50_000_000_000)),
                Header::new(
                    "bytes",
                    HeaderValue::ByteArray(Bytes::from(&b"some bytes"[..]))
                ),
                Header::new("str", HeaderValue::String("some str".into())),
                Header::new(
                    "time",
                    HeaderValue::Timestamp(DateTime::from_secs(5_000_000))
                ),
                Header::new(
                    "uuid",
                    HeaderValue::Uuid(0xb79bc914_de21_4e13_b8b2_bc47e85b7f0b)
                ),
            ]
        );

        assert_eq!(b"some payload", result.payload().as_ref());
    }

    #[test]
    fn round_trip_all_headers_payload() {
        let message = Message::new(&b"some payload"[..])
            .add_header(Header::new("true", HeaderValue::Bool(true)))
            .add_header(Header::new("false", HeaderValue::Bool(false)))
            .add_header(Header::new("byte", HeaderValue::Byte(50)))
            .add_header(Header::new("short", HeaderValue::Int16(20_000)))
            .add_header(Header::new("int", HeaderValue::Int32(500_000)))
            .add_header(Header::new("long", HeaderValue::Int64(50_000_000_000)))
            .add_header(Header::new(
                "bytes",
                HeaderValue::ByteArray((&b"some bytes"[..]).into()),
            ))
            .add_header(Header::new("str", HeaderValue::String("some str".into())))
            .add_header(Header::new(
                "time",
                HeaderValue::Timestamp(DateTime::from_secs(5_000_000)),
            ))
            .add_header(Header::new(
                "uuid",
                HeaderValue::Uuid(0xb79bc914_de21_4e13_b8b2_bc47e85b7f0b),
            ));

        let mut actual = Vec::new();
        write_message_to(&message, &mut actual).unwrap();

        let expected = include_bytes!("../test_data/valid_with_all_headers_and_payload").to_vec();
        assert_eq!(expected, actual);

        let result = read_message_from(&mut Bytes::from(actual)).unwrap();
        assert_eq!(message.headers(), result.headers());
        assert_eq!(message.payload().as_ref(), result.payload().as_ref());
    }

    /// Asserts that serialization fails with `$err` and that nothing was written to the buffer.
    macro_rules! write_message_expect_err {
        ($message:expr, $err:pat) => {
            let mut buffer = Vec::new();
            let result = write_message_to(&$message, &mut buffer);
            assert!(result.is_err(), "Expected error, got {:?}", result);
            assert!(
                matches!(result.as_ref().err().unwrap().kind(), $err),
                "Expected {}, got {:?}",
                stringify!($err),
                result
            );
            assert!(
                buffer.is_empty(),
                "rejected message must not emit a partial frame, wrote {} bytes",
                buffer.len()
            );
        };
    }

    fn message_with_header(name: impl Into<StrBytes>, value: HeaderValue) -> Message {
        Message::new(&b"payload"[..]).add_header(Header::new(name, value))
    }

    #[test]
    fn header_name_length_boundaries() {
        let mut buffer = Vec::new();
        let message = message_with_header("n".repeat(MAX_HEADER_NAME_LEN), HeaderValue::Bool(true));
        write_message_to(&message, &mut buffer).unwrap();
        assert_eq!(
            message.headers(),
            read_message_from(&mut Bytes::from(buffer))
                .unwrap()
                .headers()
        );

        write_message_expect_err!(
            message_with_header("n".repeat(MAX_HEADER_NAME_LEN + 1), HeaderValue::Bool(true)),
            ErrorKind::InvalidHeaderNameLength
        );
    }

    #[test]
    fn header_name_length_is_validated_in_bytes_not_chars() {
        // Well under the limit in characters, over it in UTF-8 bytes.
        let name = "€".repeat(MAX_HEADER_NAME_LEN / 2);
        assert!(name.chars().count() < MAX_HEADER_NAME_LEN);
        assert!(name.len() > MAX_HEADER_NAME_LEN);
        write_message_expect_err!(
            message_with_header(name, HeaderValue::Bool(true)),
            ErrorKind::InvalidHeaderNameLength
        );
    }

    #[test]
    fn string_header_value_length_boundaries() {
        let longest = "a".repeat(MAX_HEADER_VALUE_LEN);
        let mut buffer = Vec::new();
        let message = message_with_header("h", HeaderValue::String(longest.clone().into()));
        write_message_to(&message, &mut buffer).unwrap();

        // The max-length value must come back as exactly one header. If its length had been
        // truncated into the 16-bit field, the decoder would resume inside the value and parse
        // the overflow as additional headers.
        let decoded = read_message_from(&mut Bytes::from(buffer)).unwrap();
        assert_eq!(1, decoded.headers().len());
        assert_eq!(message.headers(), decoded.headers());

        write_message_expect_err!(
            message_with_header(
                "h",
                HeaderValue::String("a".repeat(MAX_HEADER_VALUE_LEN + 1).into())
            ),
            ErrorKind::HeaderValueTooLong
        );

        // One past what the 2-byte length field itself can hold. This is the size that wraps to an
        // encoded length of zero in the implementations that don't bound the value at all.
        write_message_expect_err!(
            message_with_header("h", HeaderValue::String("a".repeat(1 << 16).into())),
            ErrorKind::HeaderValueTooLong
        );
    }

    #[test]
    fn byte_array_header_value_length_boundaries() {
        let mut buffer = Vec::new();
        let message = message_with_header(
            "h",
            HeaderValue::ByteArray(Bytes::from(vec![0u8; MAX_HEADER_VALUE_LEN])),
        );
        write_message_to(&message, &mut buffer).unwrap();
        let decoded = read_message_from(&mut Bytes::from(buffer)).unwrap();
        assert_eq!(1, decoded.headers().len());
        assert_eq!(message.headers(), decoded.headers());

        write_message_expect_err!(
            message_with_header(
                "h",
                HeaderValue::ByteArray(Bytes::from(vec![0u8; MAX_HEADER_VALUE_LEN + 1]))
            ),
            ErrorKind::HeaderValueTooLong
        );

        // One past what the 2-byte length field itself can hold, which is the size that wraps to an
        // encoded length of zero when the value isn't bounded at all.
        write_message_expect_err!(
            message_with_header("h", HeaderValue::ByteArray(Bytes::from(vec![0u8; 1 << 16]))),
            ErrorKind::HeaderValueTooLong
        );
    }

    #[test]
    fn string_header_value_length_is_validated_in_bytes_not_chars() {
        // Well under the limit in characters, over it in UTF-8 bytes.
        let value = "€".repeat(MAX_HEADER_VALUE_LEN / 2);
        assert!(value.chars().count() < MAX_HEADER_VALUE_LEN);
        assert!(value.len() > MAX_HEADER_VALUE_LEN);
        write_message_expect_err!(
            message_with_header("h", HeaderValue::String(value.into())),
            ErrorKind::HeaderValueTooLong
        );
    }

    /// A value whose bytes spell out a well-formed header must not be encodable. At exactly 1 << 16
    /// bytes the length field would hold 0, so a decoder would read an empty value and then parse
    /// these bytes as the next header. The forged entry goes first because that is where the
    /// decoder would resume.
    #[test]
    fn oversized_header_value_cannot_smuggle_a_header() {
        let mut smuggled = Vec::new();
        smuggled.push(b":event-type".len() as u8);
        smuggled.extend_from_slice(b":event-type");
        smuggled.push(TYPE_STRING);
        smuggled.extend_from_slice(&(b"forged".len() as u16).to_be_bytes());
        smuggled.extend_from_slice(b"forged");
        smuggled.resize(1 << 16, b'a');

        write_message_expect_err!(
            message_with_header("DataType", HeaderValue::ByteArray(Bytes::from(smuggled))),
            ErrorKind::HeaderValueTooLong
        );
    }

    /// A timestamp whose millisecond representation overflows `i64` is the one fixed-width value
    /// that can fail to serialize, so it must error rather than write a wrong-width field.
    #[test]
    fn write_message_rejects_timestamp_too_large() {
        write_message_expect_err!(
            message_with_header(
                "time",
                HeaderValue::Timestamp(DateTime::from_secs(i64::MAX))
            ),
            ErrorKind::TimestampValueTooLarge(_)
        );
    }

    /// Fixed-width values are unrepresentable at the wrong width on the write side (they are
    /// `i8`/`i16`/`i32`/`i64`/`u128`), so the only place a short one can appear is an inbound
    /// frame. Assert the decoder rejects those rather than under-reading the next header.
    #[test]
    fn truncated_fixed_width_header_values_are_rejected() {
        // (value type, bytes supplied, protocol width)
        let cases = [
            (TYPE_BYTE, 0, 1),
            (TYPE_INT16, 1, 2),
            (TYPE_INT32, 3, 4),
            (TYPE_INT64, 7, 8),
            (TYPE_TIMESTAMP, 7, 8),
            (TYPE_UUID, 15, 16),
        ];
        for (value_type, supplied, width) in cases {
            let mut headers = vec![1u8, b'h', value_type];
            headers.extend_from_slice(&vec![0u8; supplied]);

            let headers_len = u32::try_from(headers.len()).unwrap();
            let total_len = PRELUDE_LENGTH_BYTES + headers_len + MESSAGE_CRC_LENGTH_BYTES;
            let mut frame = Vec::new();
            frame.extend_from_slice(&total_len.to_be_bytes());
            frame.extend_from_slice(&headers_len.to_be_bytes());
            frame.extend_from_slice(&crc32fast::hash(&frame).to_be_bytes());
            frame.extend_from_slice(&headers);
            frame.extend_from_slice(&crc32fast::hash(&frame).to_be_bytes());

            let result = read_message_from(&mut Bytes::from(frame));
            assert!(
                result.is_err(),
                "value type {value_type} with {supplied} of {width} bytes must be rejected, \
                 got {result:?}"
            );
        }
    }

    /// `write_headers_to` is public and writes a header's name and value type before the value
    /// length is validated, so a rejected header must not leave that prefix behind: a decoder would
    /// read the orphaned name and type tag as a header whose value is whatever follows it.
    #[test]
    fn write_headers_to_leaves_no_partial_header() {
        let headers = [
            Header::new("ok", HeaderValue::String("fine".into())),
            Header::new(
                "toolong",
                HeaderValue::String("a".repeat(MAX_HEADER_VALUE_LEN + 1).into()),
            ),
        ];
        // Seeded so the assertion catches an append as well as a bare partial write.
        let mut buffer = b"PREVIOUS-CONTENT".to_vec();
        let result = write_headers_to(&headers, &mut buffer);
        assert!(result.is_err(), "Expected error, got {result:?}");
        assert_eq!(
            b"PREVIOUS-CONTENT".to_vec(),
            buffer,
            "no bytes may be written when a header is rejected"
        );
    }
}

/// A decoder written against the wire specification rather than by reusing [`read_message_from`],
/// so that what [`write_message_to`] produces is checked by a second implementation.
#[cfg(test)]
mod independent_decoder_tests {
    use super::write_message_to;
    use aws_smithy_types::event_stream::{Header, HeaderValue, Message};
    use aws_smithy_types::DateTime;
    use bytes::Bytes;

    #[derive(Debug, PartialEq)]
    enum Value {
        Bool(bool),
        Byte(i8),
        Int16(i16),
        Int32(i32),
        Int64(i64),
        Bytes(Vec<u8>),
        Str(String),
        Timestamp(i64),
        Uuid(u128),
    }

    fn u16_at(b: &[u8], i: usize) -> usize {
        u16::from_be_bytes([b[i], b[i + 1]]) as usize
    }
    fn u32_at(b: &[u8], i: usize) -> usize {
        u32::from_be_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize
    }

    /// The headers and payload of a successfully decoded frame.
    type Decoded = (Vec<(String, Value)>, Vec<u8>);

    /// Decodes a frame per the specification, returning the headers and payload, or the first
    /// invariant it violates.
    fn decode(frame: &[u8]) -> Result<Decoded, String> {
        if frame.len() < 16 {
            return Err("shorter than prelude plus message crc".into());
        }
        if u32_at(frame, 0) != frame.len() {
            return Err(format!(
                "total_length {} != actual {}",
                u32_at(frame, 0),
                frame.len()
            ));
        }
        if crc32fast::hash(&frame[..8]) != u32_at(frame, 8) as u32 {
            return Err("prelude crc mismatch".into());
        }
        let crc_start = frame.len() - 4;
        if crc32fast::hash(&frame[..crc_start]) != u32_at(frame, crc_start) as u32 {
            return Err("message crc mismatch".into());
        }

        let headers_len = u32_at(frame, 4);
        if 12 + headers_len + 4 > frame.len() {
            return Err("headers_length overruns the frame".into());
        }
        let block = &frame[12..12 + headers_len];

        let mut headers = Vec::new();
        let mut i = 0;
        while i < block.len() {
            let name_len = block[i] as usize;
            i += 1;
            if name_len == 0 || i + name_len > block.len() {
                return Err(format!("bad header name length {name_len}"));
            }
            let name = String::from_utf8(block[i..i + name_len].to_vec())
                .map_err(|_| "header name is not utf-8")?;
            i += name_len;
            if i >= block.len() {
                return Err("header has no value type".into());
            }
            let value_type = block[i];
            i += 1;
            // Every fixed-width type must find exactly its protocol width remaining.
            let mut take = |n: usize| -> Result<&[u8], String> {
                if i + n > block.len() {
                    return Err(format!(
                        "type {value_type} needs {n} bytes, {} left",
                        block.len() - i
                    ));
                }
                let out = &block[i..i + n];
                i += n;
                Ok(out)
            };
            let value = match value_type {
                0 => Value::Bool(true),
                1 => Value::Bool(false),
                2 => Value::Byte(take(1)?[0] as i8),
                3 => Value::Int16(i16::from_be_bytes(take(2)?.try_into().unwrap())),
                4 => Value::Int32(i32::from_be_bytes(take(4)?.try_into().unwrap())),
                5 => Value::Int64(i64::from_be_bytes(take(8)?.try_into().unwrap())),
                6 | 7 => {
                    let len = u16_at(take(2)?, 0);
                    let raw = take(len)?.to_vec();
                    if value_type == 7 {
                        Value::Str(String::from_utf8(raw).map_err(|_| "value is not utf-8")?)
                    } else {
                        Value::Bytes(raw)
                    }
                }
                8 => Value::Timestamp(i64::from_be_bytes(take(8)?.try_into().unwrap())),
                9 => Value::Uuid(u128::from_be_bytes(take(16)?.try_into().unwrap())),
                other => return Err(format!("unknown value type {other}")),
            };
            headers.push((name, value));
        }
        // A value length that didn't match its bytes shows up here as leftover or overrun.
        if i != block.len() {
            return Err("header block was not consumed exactly".into());
        }
        Ok((headers, frame[12 + headers_len..crc_start].to_vec()))
    }

    /// Establish that the decoder above is correct, using the reference message from the CRT's own
    /// test suite, before relying on it to judge our encoder.
    #[test]
    fn agrees_with_the_crt_reference_message() {
        let data: &[u8] = &[
            0x00, 0x00, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x20, 0x07, 0xFD, 0x83, 0x96, 0x0C, b'c',
            b'o', b'n', b't', b'e', b'n', b't', b'-', b't', b'y', b'p', b'e', 0x07, 0x00, 0x10,
            b'a', b'p', b'p', b'l', b'i', b'c', b'a', b't', b'i', b'o', b'n', b'/', b'j', b's',
            b'o', b'n', 0x7b, 0x27, 0x66, 0x6f, 0x6f, 0x27, 0x3a, 0x27, 0x62, 0x61, 0x72, 0x27,
            0x7d, 0x8D, 0x9C, 0x08, 0xB1,
        ];
        let (headers, payload) = decode(data).expect("the reference message must decode");
        assert_eq!(
            vec![(
                "content-type".to_string(),
                Value::Str("application/json".into())
            )],
            headers
        );
        assert_eq!(b"{'foo':'bar'}".to_vec(), payload);
    }

    #[test]
    fn every_header_type_survives_a_round_trip() {
        let message = Message::new(&b"some payload"[..])
            .add_header(Header::new("true", HeaderValue::Bool(true)))
            .add_header(Header::new("false", HeaderValue::Bool(false)))
            .add_header(Header::new("byte", HeaderValue::Byte(i8::MIN)))
            .add_header(Header::new("short", HeaderValue::Int16(i16::MIN)))
            .add_header(Header::new("int", HeaderValue::Int32(i32::MIN)))
            .add_header(Header::new("long", HeaderValue::Int64(i64::MIN)))
            .add_header(Header::new(
                "bytes",
                HeaderValue::ByteArray((&b"some bytes"[..]).into()),
            ))
            .add_header(Header::new("str", HeaderValue::String("some str".into())))
            .add_header(Header::new(
                "time",
                HeaderValue::Timestamp(DateTime::from_secs(5_000_000)),
            ))
            .add_header(Header::new("uuid", HeaderValue::Uuid(u128::MAX)));

        let mut buffer = Vec::new();
        write_message_to(&message, &mut buffer).unwrap();
        let (headers, payload) = decode(&buffer).expect("our output must satisfy the spec");

        assert_eq!(
            vec![
                ("true".to_string(), Value::Bool(true)),
                ("false".to_string(), Value::Bool(false)),
                ("byte".to_string(), Value::Byte(i8::MIN)),
                ("short".to_string(), Value::Int16(i16::MIN)),
                ("int".to_string(), Value::Int32(i32::MIN)),
                ("long".to_string(), Value::Int64(i64::MIN)),
                ("bytes".to_string(), Value::Bytes(b"some bytes".to_vec())),
                ("str".to_string(), Value::Str("some str".into())),
                ("time".to_string(), Value::Timestamp(5_000_000_000)),
                ("uuid".to_string(), Value::Uuid(u128::MAX)),
            ],
            headers
        );
        assert_eq!(b"some payload".to_vec(), payload);
    }

    /// The longest permitted value must decode as a single header. A truncated length would leave
    /// the decoder reading the remainder of the value as further headers, which is the smuggling
    /// primitive this bound exists to prevent.
    #[test]
    fn longest_permitted_value_decodes_as_one_header() {
        let value = "a".repeat(super::MAX_HEADER_VALUE_LEN);
        let message = Message::new(&b""[..])
            .add_header(Header::new("h", HeaderValue::String(value.clone().into())));
        let mut buffer = Vec::new();
        write_message_to(&message, &mut buffer).unwrap();

        let (headers, _) = decode(&buffer).expect("the longest permitted value must decode");
        assert_eq!(vec![("h".to_string(), Value::Str(value))], headers);
    }

    /// The same check over a byte array, whose length field is the one the attack overflows.
    #[test]
    fn longest_permitted_byte_array_decodes_as_one_header() {
        let value = vec![b'z'; super::MAX_HEADER_VALUE_LEN];
        let message = Message::new(&b""[..]).add_header(Header::new(
            "h",
            HeaderValue::ByteArray(Bytes::from(value.clone())),
        ));
        let mut buffer = Vec::new();
        write_message_to(&message, &mut buffer).unwrap();

        let (headers, _) = decode(&buffer).expect("the longest permitted value must decode");
        assert_eq!(vec![("h".to_string(), Value::Bytes(value))], headers);
    }
}

/// Return value from [`MessageFrameDecoder`].
#[derive(Debug)]
pub enum DecodedFrame {
    /// There wasn't enough data in the buffer to decode a full message.
    Incomplete,
    /// There was enough data in the buffer to decode.
    Complete(Message),
}

/// Streaming decoder for decoding a [`Message`] from a stream.
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct MessageFrameDecoder {
    prelude: [u8; PRELUDE_LENGTH_BYTES_USIZE],
    prelude_read: bool,
}

impl MessageFrameDecoder {
    /// Returns a new `MessageFrameDecoder`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Determines if the `buffer` has enough data in it to read a full frame.
    /// Returns `Ok(None)` if there's not enough data, or `Some(remaining)` where
    /// `remaining` is the number of bytes after the prelude that belong to the
    /// message that's in the buffer.
    fn remaining_bytes_if_frame_available<B: Buf>(
        &self,
        buffer: &B,
    ) -> Result<Option<usize>, Error> {
        if self.prelude_read {
            let remaining_len = (&self.prelude[..])
                .get_u32()
                .checked_sub(PRELUDE_LENGTH_BYTES)
                .ok_or_else(|| Error::from(ErrorKind::InvalidMessageLength))?;
            if buffer.remaining() >= remaining_len as usize {
                return Ok(Some(remaining_len as usize));
            }
        }
        Ok(None)
    }

    /// Resets the decoder.
    fn reset(&mut self) {
        self.prelude_read = false;
        self.prelude = [0u8; PRELUDE_LENGTH_BYTES_USIZE];
    }

    /// Attempts to decode a [`Message`] from the given `buffer`. This function expects
    /// to be called over and over again with more data in the buffer each time its called.
    /// When there's not enough data to decode a message, it returns `Ok(None)`.
    ///
    /// Once there is enough data to read a message prelude, then it will mutate the `Buf`
    /// position. The state from the reading of the prelude is stored in the decoder so that
    /// the next call will be able to decode the entire message, even though the prelude
    /// is no longer available in the `Buf`.
    pub fn decode_frame<B: Buf>(&mut self, mut buffer: B) -> Result<DecodedFrame, Error> {
        if !self.prelude_read && buffer.remaining() >= PRELUDE_LENGTH_BYTES_USIZE {
            buffer.copy_to_slice(&mut self.prelude);
            self.prelude_read = true;
        }

        if let Some(remaining_len) = self.remaining_bytes_if_frame_available(&buffer)? {
            let mut message_buf = (&self.prelude[..]).chain(buffer.take(remaining_len));
            let result = read_message_from(&mut message_buf).map(DecodedFrame::Complete);
            self.reset();
            return result;
        }

        Ok(DecodedFrame::Incomplete)
    }
}

#[cfg(test)]
mod message_frame_decoder_tests {
    use super::{DecodedFrame, MessageFrameDecoder};
    use crate::frame::read_message_from;
    use bytes::Bytes;
    use bytes_utils::SegmentedBuf;

    #[test]
    fn single_streaming_message() {
        let message = include_bytes!("../test_data/valid_with_all_headers_and_payload");

        let mut decoder = MessageFrameDecoder::new();
        let mut segmented = SegmentedBuf::new();
        for i in 0..(message.len() - 1) {
            segmented.push(&message[i..(i + 1)]);
            if let DecodedFrame::Complete(_) = decoder.decode_frame(&mut segmented).unwrap() {
                panic!("incomplete frame shouldn't result in message");
            }
        }

        segmented.push(&message[(message.len() - 1)..]);
        match decoder.decode_frame(&mut segmented).unwrap() {
            DecodedFrame::Incomplete => panic!("frame should be complete now"),
            DecodedFrame::Complete(actual) => {
                let expected = read_message_from(&mut Bytes::from_static(message)).unwrap();
                assert_eq!(expected, actual);
            }
        }
    }

    fn multiple_streaming_messages_chunk_size(chunk_size: usize) {
        let message1 = include_bytes!("../test_data/valid_with_all_headers_and_payload");
        let message2 = include_bytes!("../test_data/valid_empty_payload");
        let message3 = include_bytes!("../test_data/valid_no_headers");
        let mut repeated = message1.to_vec();
        repeated.extend_from_slice(message2);
        repeated.extend_from_slice(message3);

        let mut decoder = MessageFrameDecoder::new();
        let mut segmented = SegmentedBuf::new();
        let mut decoded = Vec::new();
        for window in repeated.chunks(chunk_size) {
            segmented.push(window);
            match dbg!(decoder.decode_frame(&mut segmented)).unwrap() {
                DecodedFrame::Incomplete => {}
                DecodedFrame::Complete(message) => {
                    decoded.push(message);
                }
            }
        }

        let expected1 = read_message_from(&mut Bytes::from_static(message1)).unwrap();
        let expected2 = read_message_from(&mut Bytes::from_static(message2)).unwrap();
        let expected3 = read_message_from(&mut Bytes::from_static(message3)).unwrap();
        assert_eq!(3, decoded.len());
        assert_eq!(expected1, decoded[0]);
        assert_eq!(expected2, decoded[1]);
        assert_eq!(expected3, decoded[2]);
    }

    #[test]
    fn multiple_streaming_messages() {
        for chunk_size in 1..=11 {
            println!("chunk size: {chunk_size}");
            multiple_streaming_messages_chunk_size(chunk_size);
        }
    }
}

#[cfg(test)]
mod deferred_signer_tests {
    use crate::frame::{DeferredSigner, Header, HeaderValue, Message, SignMessage};
    use bytes::Bytes;

    fn check_send_sync<T: Send + Sync>(value: T) -> T {
        value
    }

    #[test]
    fn deferred_signer() {
        #[derive(Default, Debug)]
        struct TestSigner {
            call_num: i32,
        }
        impl SignMessage for TestSigner {
            fn sign(
                &mut self,
                message: Message,
            ) -> Result<Message, crate::frame::SignMessageError> {
                self.call_num += 1;
                Ok(message.add_header(Header::new("call_num", HeaderValue::Int32(self.call_num))))
            }

            fn sign_empty(&mut self) -> Option<Result<Message, crate::frame::SignMessageError>> {
                None
            }
        }

        let (mut signer, sender) = check_send_sync(DeferredSigner::new());

        sender
            .send(Box::<TestSigner>::default() as Box<dyn SignMessage + Send + Sync>)
            .expect("success");

        let message = signer.sign(Message::new(Bytes::new())).expect("success");
        assert_eq!(1, message.headers()[0].value().as_int32().unwrap());

        let message = signer.sign(Message::new(Bytes::new())).expect("success");
        assert_eq!(2, message.headers()[0].value().as_int32().unwrap());

        assert!(signer.sign_empty().is_none());
    }

    #[test]
    fn deferred_signer_defaults_to_noop_signer() {
        let (mut signer, _sender) = DeferredSigner::new();
        assert_eq!(
            Message::new(Bytes::new()),
            signer.sign(Message::new(Bytes::new())).unwrap()
        );
        assert!(signer.sign_empty().is_none());
    }
}
