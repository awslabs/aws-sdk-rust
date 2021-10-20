/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Event Stream message frame types and serialization/deserialization logic.

use crate::buf::count::CountBuf;
use crate::buf::crc::{CrcBuf, CrcBufMut};
use crate::error::Error;
use crate::str_bytes::StrBytes;
use bytes::{Buf, BufMut, Bytes};
use std::convert::{TryFrom, TryInto};
use std::error::Error as StdError;
use std::fmt;
use std::mem::size_of;

const PRELUDE_LENGTH_BYTES: u32 = 3 * size_of::<u32>() as u32;
const PRELUDE_LENGTH_BYTES_USIZE: usize = PRELUDE_LENGTH_BYTES as usize;
const MESSAGE_CRC_LENGTH_BYTES: u32 = size_of::<u32>() as u32;
const MAX_HEADER_NAME_LEN: usize = 255;
const MIN_HEADER_LEN: usize = 2;

pub type SignMessageError = Box<dyn StdError + Send + Sync + 'static>;

/// Signs an Event Stream message.
pub trait SignMessage: fmt::Debug {
    fn sign(&mut self, message: Message) -> Result<Message, SignMessageError>;

    fn sign_empty(&mut self) -> Result<Message, SignMessageError>;
}

/// Converts a Smithy modeled Event Stream type into a [`Message`](Message).
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

/// Converts an Event Stream [`Message`](Message) into a Smithy modeled type.
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

mod value {
    use crate::error::Error;
    use crate::frame::checked;
    use crate::str_bytes::StrBytes;
    use aws_smithy_types::Instant;
    use bytes::{Buf, BufMut, Bytes};
    use std::convert::TryInto;
    use std::mem::size_of;

    const TYPE_TRUE: u8 = 0;
    const TYPE_FALSE: u8 = 1;
    const TYPE_BYTE: u8 = 2;
    const TYPE_INT16: u8 = 3;
    const TYPE_INT32: u8 = 4;
    const TYPE_INT64: u8 = 5;
    const TYPE_BYTE_ARRAY: u8 = 6;
    const TYPE_STRING: u8 = 7;
    const TYPE_TIMESTAMP: u8 = 8;
    const TYPE_UUID: u8 = 9;

    /// Event Stream frame header value.
    #[non_exhaustive]
    #[derive(Clone, Debug, PartialEq)]
    pub enum HeaderValue {
        Bool(bool),
        Byte(i8),
        Int16(i16),
        Int32(i32),
        Int64(i64),
        ByteArray(Bytes),
        String(StrBytes),
        Timestamp(Instant),
        Uuid(u128),
    }

    impl HeaderValue {
        pub fn as_bool(&self) -> Result<bool, &Self> {
            match self {
                HeaderValue::Bool(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_byte(&self) -> Result<i8, &Self> {
            match self {
                HeaderValue::Byte(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_int16(&self) -> Result<i16, &Self> {
            match self {
                HeaderValue::Int16(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_int32(&self) -> Result<i32, &Self> {
            match self {
                HeaderValue::Int32(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_int64(&self) -> Result<i64, &Self> {
            match self {
                HeaderValue::Int64(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_byte_array(&self) -> Result<&Bytes, &Self> {
            match self {
                HeaderValue::ByteArray(value) => Ok(value),
                _ => Err(self),
            }
        }

        pub fn as_string(&self) -> Result<&StrBytes, &Self> {
            match self {
                HeaderValue::String(value) => Ok(value),
                _ => Err(self),
            }
        }

        pub fn as_timestamp(&self) -> Result<Instant, &Self> {
            match self {
                HeaderValue::Timestamp(value) => Ok(*value),
                _ => Err(self),
            }
        }

        pub fn as_uuid(&self) -> Result<u128, &Self> {
            match self {
                HeaderValue::Uuid(value) => Ok(*value),
                _ => Err(self),
            }
        }
    }

    macro_rules! read_value {
        ($buf:ident, $typ:ident, $size_typ:ident, $read_fn:ident) => {
            if $buf.remaining() >= size_of::<$size_typ>() {
                Ok(HeaderValue::$typ($buf.$read_fn()))
            } else {
                Err(Error::InvalidHeaderValue)
            }
        };
    }

    impl HeaderValue {
        pub(super) fn read_from<B: Buf>(mut buffer: B) -> Result<HeaderValue, Error> {
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
                            return Err(Error::InvalidHeaderValue);
                        }
                        let bytes = buffer.copy_to_bytes(len);
                        if value_type == TYPE_STRING {
                            Ok(HeaderValue::String(
                                bytes.try_into().map_err(|_| Error::InvalidUtf8String)?,
                            ))
                        } else {
                            Ok(HeaderValue::ByteArray(bytes))
                        }
                    } else {
                        Err(Error::InvalidHeaderValue)
                    }
                }
                TYPE_TIMESTAMP => {
                    if buffer.remaining() >= size_of::<i64>() {
                        let epoch_millis = buffer.get_i64();
                        Ok(HeaderValue::Timestamp(Instant::from_epoch_millis(
                            epoch_millis,
                        )))
                    } else {
                        Err(Error::InvalidHeaderValue)
                    }
                }
                TYPE_UUID => read_value!(buffer, Uuid, u128, get_u128),
                _ => Err(Error::InvalidHeaderValueType(value_type)),
            }
        }

        pub(super) fn write_to<B: BufMut>(&self, mut buffer: B) -> Result<(), Error> {
            use HeaderValue::*;
            match self {
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
                    buffer.put_u16(checked(val.len(), Error::HeaderValueTooLong)?);
                    buffer.put_slice(&val[..]);
                }
                String(val) => {
                    buffer.put_u8(TYPE_STRING);
                    buffer.put_u16(checked(val.as_bytes().len(), Error::HeaderValueTooLong)?);
                    buffer.put_slice(&val.as_bytes()[..]);
                }
                Timestamp(time) => {
                    buffer.put_u8(TYPE_TIMESTAMP);
                    buffer.put_i64(
                        time.to_epoch_millis()
                            .map_err(|_| Error::TimestampValueTooLarge(*time))?,
                    );
                }
                Uuid(val) => {
                    buffer.put_u8(TYPE_UUID);
                    buffer.put_u128(*val);
                }
            }
            Ok(())
        }
    }

    #[cfg(feature = "derive-arbitrary")]
    impl<'a> arbitrary::Arbitrary<'a> for HeaderValue {
        fn arbitrary(unstruct: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            let value_type: u8 = unstruct.int_in_range(0..=9)?;
            Ok(match value_type {
                TYPE_TRUE => HeaderValue::Bool(true),
                TYPE_FALSE => HeaderValue::Bool(false),
                TYPE_BYTE => HeaderValue::Byte(i8::arbitrary(unstruct)?),
                TYPE_INT16 => HeaderValue::Int16(i16::arbitrary(unstruct)?),
                TYPE_INT32 => HeaderValue::Int32(i32::arbitrary(unstruct)?),
                TYPE_INT64 => HeaderValue::Int64(i64::arbitrary(unstruct)?),
                TYPE_BYTE_ARRAY => {
                    HeaderValue::ByteArray(Bytes::from(Vec::<u8>::arbitrary(unstruct)?))
                }
                TYPE_STRING => HeaderValue::String(StrBytes::from(String::arbitrary(unstruct)?)),
                TYPE_TIMESTAMP => {
                    HeaderValue::Timestamp(Instant::from_epoch_seconds(i64::arbitrary(unstruct)?))
                }
                TYPE_UUID => HeaderValue::Uuid(u128::arbitrary(unstruct)?),
                _ => unreachable!(),
            })
        }
    }
}

pub use value::HeaderValue;

/// Event Stream header.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "derive-arbitrary", derive(arbitrary::Arbitrary))]
pub struct Header {
    name: StrBytes,
    value: HeaderValue,
}

impl Header {
    /// Creates a new header with the given `name` and `value`.
    pub fn new(name: impl Into<StrBytes>, value: HeaderValue) -> Header {
        Header {
            name: name.into(),
            value,
        }
    }

    /// Returns the header name.
    pub fn name(&self) -> &StrBytes {
        &self.name
    }

    /// Returns the header value.
    pub fn value(&self) -> &HeaderValue {
        &self.value
    }

    /// Reads a header from the given `buffer`.
    fn read_from<B: Buf>(mut buffer: B) -> Result<(Header, usize), Error> {
        if buffer.remaining() < MIN_HEADER_LEN {
            return Err(Error::InvalidHeadersLength);
        }

        let mut counting_buf = CountBuf::new(&mut buffer);
        let name_len = counting_buf.get_u8();
        if name_len as usize >= counting_buf.remaining() {
            return Err(Error::InvalidHeaderNameLength);
        }

        let name: StrBytes = counting_buf
            .copy_to_bytes(name_len as usize)
            .try_into()
            .map_err(|_| Error::InvalidUtf8String)?;
        let value = HeaderValue::read_from(&mut counting_buf)?;
        Ok((Header::new(name, value), counting_buf.into_count()))
    }

    /// Writes the header to the given `buffer`.
    fn write_to<B: BufMut>(&self, mut buffer: B) -> Result<(), Error> {
        if self.name.as_bytes().len() > MAX_HEADER_NAME_LEN {
            return Err(Error::InvalidHeaderNameLength);
        }

        buffer.put_u8(u8::try_from(self.name.as_bytes().len()).expect("bounds check above"));
        buffer.put_slice(&self.name.as_bytes()[..]);
        self.value.write_to(buffer)
    }
}

/// Writes the given `headers` to a `buffer`.
pub fn write_headers_to<B: BufMut>(headers: &[Header], mut buffer: B) -> Result<(), Error> {
    for header in headers {
        header.write_to(&mut buffer)?;
    }
    Ok(())
}

/// Event Stream message.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    headers: Vec<Header>,
    payload: Bytes,
}

impl Message {
    /// Creates a new message with the given `payload`. Headers can be added later.
    pub fn new(payload: impl Into<Bytes>) -> Message {
        Message {
            headers: Vec::new(),
            payload: payload.into(),
        }
    }

    /// Creates a message with the given `headers` and `payload`.
    pub fn new_from_parts(headers: Vec<Header>, payload: impl Into<Bytes>) -> Self {
        Self {
            headers,
            payload: payload.into(),
        }
    }

    /// Adds a header to the message.
    pub fn add_header(mut self, header: Header) -> Self {
        self.headers.push(header);
        self
    }

    /// Returns all headers.
    pub fn headers(&self) -> &[Header] {
        &self.headers
    }

    /// Returns the payload bytes.
    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    // Returns (total_len, header_len)
    fn read_prelude_from<B: Buf>(mut buffer: B) -> Result<(u32, u32), Error> {
        let mut crc_buffer = CrcBuf::new(&mut buffer);

        // If the buffer doesn't have the entire, then error
        let total_len = crc_buffer.get_u32();
        if crc_buffer.remaining() + size_of::<u32>() < total_len as usize {
            return Err(Error::InvalidMessageLength);
        }

        // Validate the prelude
        let header_len = crc_buffer.get_u32();
        let (expected_crc, prelude_crc) = (crc_buffer.into_crc(), buffer.get_u32());
        if expected_crc != prelude_crc {
            return Err(Error::PreludeChecksumMismatch(expected_crc, prelude_crc));
        }
        // The header length can be 0 or >= 2, but must fit within the frame size
        if header_len == 1 || header_len > max_header_len(total_len)? {
            return Err(Error::InvalidHeadersLength);
        }
        Ok((total_len, header_len))
    }

    /// Reads a message from the given `buffer`. For streaming use cases, use
    /// the [`MessageFrameDecoder`] instead of this.
    pub fn read_from<B: Buf>(mut buffer: B) -> Result<Message, Error> {
        if buffer.remaining() < PRELUDE_LENGTH_BYTES_USIZE {
            return Err(Error::InvalidMessageLength);
        }

        // Calculate a CRC as we go and read the prelude
        let mut crc_buffer = CrcBuf::new(&mut buffer);
        let (total_len, header_len) = Self::read_prelude_from(&mut crc_buffer)?;

        // Verify we have the full frame before continuing
        let remaining_len = total_len
            .checked_sub(PRELUDE_LENGTH_BYTES)
            .ok_or(Error::InvalidMessageLength)?;
        if crc_buffer.remaining() < remaining_len as usize {
            return Err(Error::InvalidMessageLength);
        }

        // Read headers
        let mut header_bytes_read = 0;
        let mut headers = Vec::new();
        while header_bytes_read < header_len as usize {
            let (header, bytes_read) = Header::read_from(&mut crc_buffer)?;
            header_bytes_read += bytes_read;
            if header_bytes_read > header_len as usize {
                return Err(Error::InvalidHeaderValue);
            }
            headers.push(header);
        }

        // Read payload
        let payload_len = payload_len(total_len, header_len)?;
        let payload = crc_buffer.copy_to_bytes(payload_len as usize);

        let expected_crc = crc_buffer.into_crc();
        let message_crc = buffer.get_u32();
        if expected_crc != message_crc {
            return Err(Error::MessageChecksumMismatch(expected_crc, message_crc));
        }

        Ok(Message { headers, payload })
    }

    /// Writes the message to the given `buffer`.
    pub fn write_to(&self, buffer: &mut dyn BufMut) -> Result<(), Error> {
        let mut headers = Vec::new();
        for header in &self.headers {
            header.write_to(&mut headers)?;
        }

        let headers_len = checked(headers.len(), Error::HeadersTooLong)?;
        let payload_len = checked(self.payload.len(), Error::PayloadTooLong)?;
        let message_len = [
            PRELUDE_LENGTH_BYTES,
            headers_len,
            payload_len,
            MESSAGE_CRC_LENGTH_BYTES,
        ]
        .iter()
        .try_fold(0u32, |acc, v| {
            acc.checked_add(*v).ok_or(Error::MessageTooLong)
        })?;

        let mut crc_buffer = CrcBufMut::new(buffer);
        crc_buffer.put_u32(message_len);
        crc_buffer.put_u32(headers_len);
        crc_buffer.put_crc();
        crc_buffer.put(&headers[..]);
        crc_buffer.put(&self.payload[..]);
        crc_buffer.put_crc();
        Ok(())
    }
}

#[cfg(feature = "derive-arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for Message {
    fn arbitrary(unstruct: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let headers: arbitrary::Result<Vec<Header>> = unstruct.arbitrary_iter()?.collect();
        Ok(Message {
            headers: headers?,
            payload: Bytes::from(Vec::<u8>::arbitrary(unstruct)?),
        })
    }
}

fn checked<T: TryFrom<U>, U>(from: U, err: Error) -> Result<T, Error> {
    T::try_from(from).map_err(|_| err)
}

fn max_header_len(total_len: u32) -> Result<u32, Error> {
    total_len
        .checked_sub(PRELUDE_LENGTH_BYTES + MESSAGE_CRC_LENGTH_BYTES)
        .ok_or(Error::InvalidMessageLength)
}

fn payload_len(total_len: u32, header_len: u32) -> Result<u32, Error> {
    total_len
        .checked_sub(
            header_len
                .checked_add(PRELUDE_LENGTH_BYTES + MESSAGE_CRC_LENGTH_BYTES)
                .ok_or(Error::InvalidHeadersLength)?,
        )
        .ok_or(Error::InvalidMessageLength)
}

#[cfg(test)]
mod message_tests {
    use crate::error::Error;
    use crate::frame::{Header, HeaderValue, Message};
    use aws_smithy_types::Instant;
    use bytes::Bytes;

    macro_rules! read_message_expect_err {
        ($bytes:expr, $err:pat) => {
            let result = Message::read_from(&mut Bytes::from_static($bytes));
            assert!(
                matches!(&result.as_ref(), &Err($err)),
                "Expected {}, got {:?}",
                stringify!(Err($err)),
                result
            );
        };
    }

    #[test]
    fn invalid_messages() {
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_string_value_length"),
            Error::InvalidHeaderValue
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_string_length_cut_off"),
            Error::InvalidHeaderValue
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_value_type"),
            Error::InvalidHeaderValueType(0x60)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_name_length"),
            Error::InvalidHeaderNameLength
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_headers_length"),
            Error::InvalidHeadersLength
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_prelude_checksum"),
            Error::PreludeChecksumMismatch(0x8BB495FB, 0xDEADBEEF)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_message_checksum"),
            Error::MessageChecksumMismatch(0x01a05860, 0xDEADBEEF)
        );
        read_message_expect_err!(
            include_bytes!("../test_data/invalid_header_name_length_too_long"),
            Error::InvalidUtf8String
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

        let result = Message::read_from(&mut Bytes::from_static(&data)).unwrap();
        assert_eq!(result.headers(), Vec::new());

        let expected_payload = b"{'foo':'bar'}";
        assert_eq!(expected_payload, result.payload.as_ref());
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

        let result = Message::read_from(&mut Bytes::from_static(&data)).unwrap();
        assert_eq!(
            result.headers(),
            vec![Header::new(
                "content-type",
                HeaderValue::String("application/json".into())
            )]
        );

        let expected_payload = b"{'foo':'bar'}";
        assert_eq!(expected_payload, result.payload.as_ref());
    }

    #[test]
    fn read_all_headers_and_payload() {
        let message = include_bytes!("../test_data/valid_with_all_headers_and_payload");
        let result = Message::read_from(&mut Bytes::from_static(message)).unwrap();
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
                    HeaderValue::Timestamp(Instant::from_epoch_seconds(5_000_000))
                ),
                Header::new(
                    "uuid",
                    HeaderValue::Uuid(0xb79bc914_de21_4e13_b8b2_bc47e85b7f0b)
                ),
            ]
        );

        assert_eq!(b"some payload", result.payload.as_ref());
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
                HeaderValue::Timestamp(Instant::from_epoch_seconds(5_000_000)),
            ))
            .add_header(Header::new(
                "uuid",
                HeaderValue::Uuid(0xb79bc914_de21_4e13_b8b2_bc47e85b7f0b),
            ));

        let mut actual = Vec::new();
        message.write_to(&mut actual).unwrap();

        let expected = include_bytes!("../test_data/valid_with_all_headers_and_payload").to_vec();
        assert_eq!(expected, actual);

        let result = Message::read_from(&mut Bytes::from(actual)).unwrap();
        assert_eq!(message.headers(), result.headers());
        assert_eq!(message.payload().as_ref(), result.payload.as_ref());
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
                .ok_or(Error::InvalidMessageLength)?;
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
            let result = Message::read_from(&mut message_buf).map(DecodedFrame::Complete);
            self.reset();
            return result;
        }

        Ok(DecodedFrame::Incomplete)
    }
}

#[cfg(test)]
mod message_frame_decoder_tests {
    use super::{DecodedFrame, MessageFrameDecoder};
    use crate::frame::Message;
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
                let expected = Message::read_from(&mut Bytes::from_static(message)).unwrap();
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

        let expected1 = Message::read_from(&mut Bytes::from_static(message1)).unwrap();
        let expected2 = Message::read_from(&mut Bytes::from_static(message2)).unwrap();
        let expected3 = Message::read_from(&mut Bytes::from_static(message3)).unwrap();
        assert_eq!(3, decoded.len());
        assert_eq!(expected1, decoded[0]);
        assert_eq!(expected2, decoded[1]);
        assert_eq!(expected3, decoded[2]);
    }

    #[test]
    fn multiple_streaming_messages() {
        for chunk_size in 1..=11 {
            println!("chunk size: {}", chunk_size);
            multiple_streaming_messages_chunk_size(chunk_size);
        }
    }
}
