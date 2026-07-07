/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;

use aws_smithy_types::{BigInteger, Blob, DateTime};
use minicbor::decode::Error;

use crate::data::Type;

/// Provides functions for decoding a CBOR object with a known schema.
///
/// Although CBOR is a self-describing format, this decoder is tailored for cases where the schema
/// is known in advance. Therefore, the caller can determine which object key exists at the current
/// position by calling `str` method, and call the relevant function based on the predetermined schema
/// for that key. If an unexpected key is encountered, the caller can use the `skip` method to skip
/// over the element.
#[derive(Debug, Clone)]
pub struct Decoder<'b> {
    decoder: minicbor::Decoder<'b>,
}

/// When any of the decode methods are called they look for that particular data type at the current
/// position. If the CBOR data tag does not match the type, a `DeserializeError` is returned.
#[derive(Debug)]
pub struct DeserializeError {
    #[allow(dead_code)]
    _inner: Error,
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._inner.fmt(f)
    }
}

impl std::error::Error for DeserializeError {}

impl DeserializeError {
    pub(crate) fn new(inner: Error) -> Self {
        Self { _inner: inner }
    }

    /// More than one union variant was detected: `unexpected_type` was unexpected.
    pub fn unexpected_union_variant(unexpected_type: Type, at: usize) -> Self {
        Self {
            _inner: Error::type_mismatch(unexpected_type.into_minicbor_type())
                .with_message("encountered unexpected union variant; expected end of union")
                .at(at),
        }
    }

    /// Unknown union variant was detected. Servers reject unknown union varaints.
    pub fn unknown_union_variant(variant_name: &str, at: usize) -> Self {
        Self {
            _inner: Error::message(format!("encountered unknown union variant {variant_name}"))
                .at(at),
        }
    }

    /// More than one union variant was detected, but we never even got to parse the first one.
    /// We immediately raise this error when detecting a union serialized as a fixed-length CBOR
    /// map whose length (specified upfront) is a value different than 1.
    pub fn mixed_union_variants(at: usize) -> Self {
        Self {
            _inner: Error::message(
                "encountered mixed variants in union; expected a single union variant to be set",
            )
            .at(at),
        }
    }

    /// Expected end of stream but more data is available.
    pub fn expected_end_of_stream(at: usize) -> Self {
        Self {
            _inner: Error::message("encountered additional data; expected end of stream").at(at),
        }
    }

    /// Returns a custom error with an offset.
    pub fn custom(message: impl Into<Cow<'static, str>>, at: usize) -> Self {
        Self {
            _inner: Error::message(message.into()).at(at),
        }
    }

    /// An unexpected type was encountered.
    // We handle this one when decoding sparse collections: we have to expect either a `null` or an
    // item, so we try decoding both.
    pub fn is_type_mismatch(&self) -> bool {
        self._inner.is_type_mismatch()
    }
}

/// Macro for delegating method calls to the decoder.
///
/// This macro generates wrapper methods for calling specific methods on the decoder and returning
/// the result with error handling.
///
/// # Example
///
/// ```ignore
/// delegate_method! {
///     /// Wrapper method for encoding method `encode_str` on the decoder.
///     encode_str_wrapper => encode_str(String);
///     /// Wrapper method for encoding method `encode_int` on the decoder.
///     encode_int_wrapper => encode_int(i32);
/// }
/// ```
macro_rules! delegate_method {
    ($($(#[$meta:meta])* $wrapper_name:ident => $encoder_name:ident($result_type:ty);)+) => {
        $(
            pub fn $wrapper_name(&mut self) -> Result<$result_type, DeserializeError> {
                self.decoder.$encoder_name().map_err(DeserializeError::new)
            }
        )+
    };
}

impl<'b> Decoder<'b> {
    pub fn new(bytes: &'b [u8]) -> Self {
        Self {
            decoder: minicbor::Decoder::new(bytes),
        }
    }

    pub fn datatype(&self) -> Result<Type, DeserializeError> {
        self.decoder
            .datatype()
            .map(Type::new)
            .map_err(DeserializeError::new)
    }

    delegate_method! {
        /// Skips the current CBOR element.
        skip => skip(());
        /// Reads a boolean at the current position.
        boolean => bool(bool);
        /// Reads a byte at the current position.
        byte => i8(i8);
        /// Reads a short at the current position.
        short => i16(i16);
        /// Reads a integer at the current position.
        integer => i32(i32);
        /// Reads a long at the current position.
        long => i64(i64);
        /// Reads a float at the current position.
        float => f32(f32);
        /// Reads a double at the current position.
        double => f64(f64);
        /// Reads a null CBOR element at the current position.
        null => null(());
        /// Returns the number of elements in a definite list. For indefinite lists it returns a `None`.
        list => array(Option<u64>);
        /// Returns the number of elements in a definite map. For indefinite map it returns a `None`.
        map => map(Option<u64>);
    }

    /// Returns the current position of the buffer, which will be decoded when any of the methods is called.
    pub fn position(&self) -> usize {
        self.decoder.position()
    }

    /// Set the current decode position.
    pub fn set_position(&mut self, pos: usize) {
        self.decoder.set_position(pos)
    }

    /// Returns a `Cow::Borrowed(&str)` if the element at the current position in the buffer is a definite
    /// length string. Otherwise, it returns a `Cow::Owned(String)` if the element at the current position is an
    /// indefinite-length string. An error is returned if the element is neither a definite length nor an
    /// indefinite-length string.
    pub fn str(&mut self) -> Result<Cow<'b, str>, DeserializeError> {
        let bookmark = self.decoder.position();
        match self.decoder.str() {
            Ok(str_value) => Ok(Cow::Borrowed(str_value)),
            Err(e) if e.is_type_mismatch() => {
                // Move the position back to the start of the CBOR element and then try
                // decoding it as an indefinite length string.
                self.decoder.set_position(bookmark);
                Ok(Cow::Owned(self.string()?))
            }
            Err(e) => Err(DeserializeError::new(e)),
        }
    }

    /// Allocates and returns a `String` if the element at the current position in the buffer is either a
    /// definite-length or an indefinite-length string. Otherwise, an error is returned if the element is not a string type.
    pub fn string(&mut self) -> Result<String, DeserializeError> {
        let mut iter = self.decoder.str_iter().map_err(DeserializeError::new)?;
        let head = iter.next();

        let decoded_string = match head {
            None => String::new(),
            Some(head) => {
                let mut combined_chunks = String::from(head.map_err(DeserializeError::new)?);
                for chunk in iter {
                    combined_chunks.push_str(chunk.map_err(DeserializeError::new)?);
                }
                combined_chunks
            }
        };

        Ok(decoded_string)
    }

    /// Returns a `blob` if the element at the current position in the buffer is a byte string. Otherwise,
    /// a `DeserializeError` error is returned.
    pub fn blob(&mut self) -> Result<Blob, DeserializeError> {
        let iter = self.decoder.bytes_iter().map_err(DeserializeError::new)?;
        let parts: Vec<&[u8]> = iter
            .collect::<Result<_, _>>()
            .map_err(DeserializeError::new)?;

        Ok(if parts.len() == 1 {
            Blob::new(parts[0]) // Directly convert &[u8] to Blob if there's only one part.
        } else {
            Blob::new(parts.concat()) // Concatenate all parts into a single Blob.
        })
    }

    /// Returns a `DateTime` if the element at the current position in the buffer is a `timestamp`. Otherwise,
    /// a `DeserializeError` error is returned.
    pub fn timestamp(&mut self) -> Result<DateTime, DeserializeError> {
        let tag = self.decoder.tag().map_err(DeserializeError::new)?;
        let timestamp_tag = minicbor::data::Tag::from(minicbor::data::IanaTag::Timestamp);

        if tag != timestamp_tag {
            Err(DeserializeError::new(Error::message(
                "expected timestamp tag",
            )))
        } else {
            // RFC 8949 §3.4.2: tag 1 content MUST be int OR float.
            // Values that are more granular than millisecond precision SHOULD be truncated to fit
            // millisecond precision for epoch-seconds:
            // https://smithy.io/2.0/spec/protocol-traits.html#timestamp-formats
            //
            // Without truncation, the `RpcV2CborDateTimeWithFractionalSeconds` protocol test would
            // fail since the upstream test expect `123000000` in subsec but the decoded actual
            // subsec would be `123000025`.
            // https://github.com/smithy-lang/smithy/blob/6466fe77c65b8a17b219f0b0a60c767915205f95/smithy-protocol-tests/model/rpcv2Cbor/fractional-seconds.smithy#L17
            let epoch_seconds = match self.decoder.datatype().map_err(DeserializeError::new)? {
                minicbor::data::Type::F16
                | minicbor::data::Type::F32
                | minicbor::data::Type::F64 => self.decoder.f64().map_err(DeserializeError::new)?,
                _ => self.decoder.i64().map_err(DeserializeError::new)? as f64,
            };
            let mut result = DateTime::from_secs_f64(epoch_seconds);
            let subsec_nanos = result.subsec_nanos();
            result.set_subsec_nanos((subsec_nanos / 1_000_000) * 1_000_000);
            Ok(result)
        }
    }

    /// Returns a `BigInteger` from either a CBOR tag 2/3 (bignum) or a plain integer.
    ///
    /// Per RFC 8949 §3.4.3, tag 2 encodes unsigned bignum `n` and tag 3 encodes
    /// negative bignum `-1 - n`, where `n` is the unsigned integer from the byte
    /// string in network byte order. Plain CBOR integers (major types 0 and 1)
    /// are also accepted per preferred serialization rules.
    pub fn big_integer(&mut self) -> Result<BigInteger, DeserializeError> {
        use num_bigint::BigInt;

        match self.decoder.datatype().map_err(DeserializeError::new)? {
            minicbor::data::Type::Tag => {
                let tag = self.decoder.tag().map_err(DeserializeError::new)?;
                let bytes = self.decoder.bytes().map_err(DeserializeError::new)?;
                let n = BigInt::from_bytes_be(num_bigint::Sign::Plus, bytes);

                let value = match tag.as_u64() {
                    2 => n,
                    3 => -n - 1, // tag 3 value = -1 - n
                    _ => {
                        return Err(DeserializeError::new(Error::message(
                            "expected CBOR tag 2 (positive bignum) or tag 3 (negative bignum)",
                        )));
                    }
                };
                value
                    .to_string()
                    .parse()
                    .map_err(|_| DeserializeError::new(Error::message("invalid bignum value")))
            }
            minicbor::data::Type::U8
            | minicbor::data::Type::U16
            | minicbor::data::Type::U32
            | minicbor::data::Type::U64 => {
                let value = self.decoder.u64().map_err(DeserializeError::new)?;
                value
                    .to_string()
                    .parse()
                    .map_err(|_| DeserializeError::new(Error::message("invalid integer value")))
            }
            minicbor::data::Type::I8
            | minicbor::data::Type::I16
            | minicbor::data::Type::I32
            | minicbor::data::Type::I64 => {
                let value = self.decoder.i64().map_err(DeserializeError::new)?;
                value
                    .to_string()
                    .parse()
                    .map_err(|_| DeserializeError::new(Error::message("invalid integer value")))
            }
            // Int covers CBOR major type 1 values that exceed i64 range
            // (argument > i64::MAX, i.e. values from -2^64 to -(i64::MAX+2)).
            minicbor::data::Type::Int => {
                let int_val = self.decoder.int().map_err(DeserializeError::new)?;
                let value: i128 = int_val.into();
                BigInt::from(value)
                    .to_string()
                    .parse()
                    .map_err(|_| DeserializeError::new(Error::message("invalid integer value")))
            }
            _ => Err(DeserializeError::new(Error::message(
                "expected CBOR integer or bignum tag",
            ))),
        }
    }
}

#[allow(dead_code)] // to avoid `never constructed` warning
#[derive(Debug)]
pub struct ArrayIter<'a, 'b, T> {
    inner: minicbor::decode::ArrayIter<'a, 'b, T>,
}

impl<'b, T: minicbor::Decode<'b, ()>> Iterator for ArrayIter<'_, 'b, T> {
    type Item = Result<T, DeserializeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|opt| opt.map_err(DeserializeError::new))
    }
}

#[allow(dead_code)] // to avoid `never constructed` warning
#[derive(Debug)]
pub struct MapIter<'a, 'b, K, V> {
    inner: minicbor::decode::MapIter<'a, 'b, K, V>,
}

impl<'b, K, V> Iterator for MapIter<'_, 'b, K, V>
where
    K: minicbor::Decode<'b, ()>,
    V: minicbor::Decode<'b, ()>,
{
    type Item = Result<(K, V), DeserializeError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|opt| opt.map_err(DeserializeError::new))
    }
}

pub fn set_optional<B, F>(builder: B, decoder: &mut Decoder, f: F) -> Result<B, DeserializeError>
where
    F: Fn(B, &mut Decoder) -> Result<B, DeserializeError>,
{
    match decoder.datatype()? {
        crate::data::Type::Null => {
            decoder.null()?;
            Ok(builder)
        }
        _ => f(builder, decoder),
    }
}

#[cfg(test)]
mod tests {
    use crate::Decoder;
    use aws_smithy_types::date_time::Format;

    #[test]
    fn test_definite_str_is_cow_borrowed() {
        // Definite length key `thisIsAKey`.
        let definite_bytes = [
            0x6a, 0x74, 0x68, 0x69, 0x73, 0x49, 0x73, 0x41, 0x4b, 0x65, 0x79,
        ];
        let mut decoder = Decoder::new(&definite_bytes);
        let member = decoder.str().expect("could not decode str");
        assert_eq!(member, "thisIsAKey");
        assert!(matches!(member, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn test_indefinite_str_is_cow_owned() {
        // Indefinite length key `this`, `Is`, `A` and `Key`.
        let indefinite_bytes = [
            0x7f, 0x64, 0x74, 0x68, 0x69, 0x73, 0x62, 0x49, 0x73, 0x61, 0x41, 0x63, 0x4b, 0x65,
            0x79, 0xff,
        ];
        let mut decoder = Decoder::new(&indefinite_bytes);
        let member = decoder.str().expect("could not decode str");
        assert_eq!(member, "thisIsAKey");
        assert!(matches!(member, std::borrow::Cow::Owned(_)));
    }

    #[test]
    fn test_empty_str_works() {
        let bytes = [0x60];
        let mut decoder = Decoder::new(&bytes);
        let member = decoder.str().expect("could not decode empty str");
        assert_eq!(member, "");
    }

    #[test]
    fn test_empty_blob_works() {
        let bytes = [0x40];
        let mut decoder = Decoder::new(&bytes);
        let member = decoder.blob().expect("could not decode an empty blob");
        assert_eq!(member, aws_smithy_types::Blob::new([]));
    }

    #[test]
    fn test_indefinite_length_blob() {
        // Indefinite length blob containing bytes corresponding to `indefinite-byte, chunked, on each comma`.
        // https://cbor.nemo157.com/#type=hex&value=bf69626c6f6256616c75655f50696e646566696e6974652d627974652c49206368756e6b65642c4e206f6e206561636820636f6d6d61ffff
        let indefinite_bytes = [
            0x5f, 0x50, 0x69, 0x6e, 0x64, 0x65, 0x66, 0x69, 0x6e, 0x69, 0x74, 0x65, 0x2d, 0x62,
            0x79, 0x74, 0x65, 0x2c, 0x49, 0x20, 0x63, 0x68, 0x75, 0x6e, 0x6b, 0x65, 0x64, 0x2c,
            0x4e, 0x20, 0x6f, 0x6e, 0x20, 0x65, 0x61, 0x63, 0x68, 0x20, 0x63, 0x6f, 0x6d, 0x6d,
            0x61, 0xff,
        ];
        let mut decoder = Decoder::new(&indefinite_bytes);
        let member = decoder.blob().expect("could not decode blob");
        assert_eq!(
            member,
            aws_smithy_types::Blob::new("indefinite-byte, chunked, on each comma".as_bytes())
        );
    }

    #[test]
    fn test_timestamp_should_be_truncated_to_fit_millisecond_precision() {
        // Input bytes are derived from the `RpcV2CborDateTimeWithFractionalSeconds` protocol test,
        // extracting portion representing a timestamp value.
        let bytes = [
            0xc1, 0xfb, 0x41, 0xcc, 0x37, 0xdb, 0x38, 0x0f, 0xbe, 0x77, 0xff,
        ];
        let mut decoder = Decoder::new(&bytes);
        let timestamp = decoder.timestamp().expect("should decode timestamp");
        assert_eq!(
            timestamp,
            aws_smithy_types::date_time::DateTime::from_str(
                "2000-01-02T20:34:56.123Z",
                Format::DateTime
            )
            .unwrap()
        );
    }

    #[test]
    fn big_integer_round_trip_positive() {
        for value in ["0", "1", "23", "256", "65535", "18446744073709551615"] {
            let mut encoder = crate::Encoder::new(Vec::new());
            encoder.big_integer(&value.parse().unwrap());
            let bytes = encoder.into_writer();
            let mut decoder = Decoder::new(&bytes);
            let result = decoder.big_integer().expect("should decode");
            assert_eq!(result.as_ref(), value, "round-trip failed for {value}");
        }
    }

    #[test]
    fn big_integer_round_trip_negative() {
        for value in ["-1", "-42", "-256", "-18446744073709551616"] {
            let mut encoder = crate::Encoder::new(Vec::new());
            encoder.big_integer(&value.parse().unwrap());
            let bytes = encoder.into_writer();
            let mut decoder = Decoder::new(&bytes);
            let result = decoder.big_integer().expect("should decode");
            assert_eq!(result.as_ref(), value, "round-trip failed for {value}");
        }
    }

    #[test]
    fn big_integer_round_trip_large() {
        let large_pos = "123456789012345678901234567890";
        let large_neg = "-123456789012345678901234567890";
        for value in [large_pos, large_neg] {
            let mut encoder = crate::Encoder::new(Vec::new());
            encoder.big_integer(&value.parse().unwrap());
            let bytes = encoder.into_writer();
            let mut decoder = Decoder::new(&bytes);
            let result = decoder.big_integer().expect("should decode");
            assert_eq!(result.as_ref(), value, "round-trip failed for {value}");
        }
    }

    #[test]
    fn big_integer_rfc8949_appendix_a_positive() {
        // RFC 8949 Appendix A: 18446744073709551616 (2^64) = 0xc249010000000000000000
        let bytes = [
            0xc2, 0x49, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut decoder = Decoder::new(&bytes);
        let result = decoder.big_integer().expect("should decode");
        assert_eq!(result.as_ref(), "18446744073709551616");
    }

    #[test]
    fn big_integer_rfc8949_appendix_a_negative() {
        // RFC 8949 Appendix A: -18446744073709551617 = 0xc349010000000000000000
        let bytes = [
            0xc3, 0x49, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let mut decoder = Decoder::new(&bytes);
        let result = decoder.big_integer().expect("should decode");
        assert_eq!(result.as_ref(), "-18446744073709551617");
    }

    #[test]
    fn big_integer_from_plain_cbor_unsigned() {
        let mut enc = minicbor::Encoder::new(Vec::new());
        enc.u64(9999).unwrap();
        let bytes = enc.into_writer();
        let mut decoder = Decoder::new(&bytes);
        let result = decoder.big_integer().expect("should decode plain integer");
        assert_eq!(result.as_ref(), "9999");
    }

    #[test]
    fn big_integer_from_plain_cbor_negative() {
        let mut enc = minicbor::Encoder::new(Vec::new());
        enc.i64(-500).unwrap();
        let bytes = enc.into_writer();
        let mut decoder = Decoder::new(&bytes);
        let result = decoder
            .big_integer()
            .expect("should decode negative plain integer");
        assert_eq!(result.as_ref(), "-500");
    }

    #[test]
    fn big_integer_from_plain_cbor_positive_signed() {
        // A positive value such as +123 is encoded as CBOR major type 0 (unsigned)
        // per preferred serialization and must decode back to the same value.
        let mut enc = minicbor::Encoder::new(Vec::new());
        enc.i64(123).unwrap();
        let bytes = enc.into_writer();
        let mut decoder = Decoder::new(&bytes);
        let result = decoder
            .big_integer()
            .expect("should decode positive plain integer");
        assert_eq!(result.as_ref(), "123");
    }

    #[test]
    fn big_integer_tag3_empty_byte_string() {
        // Tag 3 with empty byte string = -1 - 0 = -1
        let bytes = [0xc3, 0x40]; // tag 3, empty byte string
        let mut decoder = Decoder::new(&bytes);
        let result = decoder.big_integer().expect("should decode");
        assert_eq!(result.as_ref(), "-1");
    }

    #[test]
    fn big_integer_tag2_empty_byte_string() {
        // Tag 2 with empty byte string = 0
        let bytes = [0xc2, 0x40]; // tag 2, empty byte string
        let mut decoder = Decoder::new(&bytes);
        let result = decoder.big_integer().expect("should decode");
        assert_eq!(result.as_ref(), "0");
    }

    #[test]
    fn big_integer_rejects_invalid_tag() {
        // Tag 4 (decimal fraction) should be rejected.
        let bytes = [0xc4, 0x82, 0x21, 0x19, 0x6a, 0xb3];
        let mut decoder = Decoder::new(&bytes);
        assert!(decoder.big_integer().is_err());
    }

    #[test]
    fn big_integer_decode_major_type_1_exceeding_i64() {
        // CBOR major type 1 with argument u64::MAX (0x3b + 8 bytes of 0xff).
        // Value = -1 - u64::MAX = -18446744073709551616.
        // This exercises the minicbor Type::Int path in the decoder.
        let bytes = [0x3b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let mut decoder = Decoder::new(&bytes);
        let result = decoder
            .big_integer()
            .expect("should decode major type 1 > i64::MAX");
        assert_eq!(result.as_ref(), "-18446744073709551616");
    }

    #[test]
    fn test_timestamp_integer_epoch_seconds() {
        // RFC 8949 §3.4.2: tag 1 content MUST be int OR float.
        // tag(1) + uint(1700000000) = 0xc1 0x1a 0x65 0x53 0xf1 0x00
        let bytes = [0xc1u8, 0x1a, 0x65, 0x53, 0xf1, 0x00];
        let mut decoder = Decoder::new(&bytes);
        let timestamp = decoder
            .timestamp()
            .expect("should decode integer timestamp");
        assert_eq!(timestamp, aws_smithy_types::DateTime::from_secs(1700000000));
    }
}
