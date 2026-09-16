/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::{BigInteger, Blob, DateTime};

/// Macro for delegating method calls to the encoder.
///
/// This macro generates wrapper methods for calling specific encoder methods on the encoder
/// and returning a mutable reference to self for method chaining.
///
/// # Example
///
/// ```ignore
/// delegate_method! {
///     /// Wrapper method for encoding method `encode_str` on the encoder.
///     encode_str_wrapper => encode_str(data: &str);
///     /// Wrapper method for encoding method `encode_int` on the encoder.
///     encode_int_wrapper => encode_int(value: i32);
/// }
/// ```
macro_rules! delegate_method {
    ($($(#[$meta:meta])* $wrapper_name:ident => $encoder_name:ident($($param_name:ident : $param_type:ty),*);)+) => {
        $(
            pub fn $wrapper_name(&mut self, $($param_name: $param_type),*) -> &mut Self {
                self.encoder.$encoder_name($($param_name)*).expect(INFALLIBLE_WRITE);
                self
            }
        )+
    };
}

#[derive(Debug, Clone)]
pub struct Encoder {
    encoder: minicbor::Encoder<Vec<u8>>,
}

/// We always write to a `Vec<u8>`, which is infallible in `minicbor`.
/// <https://docs.rs/minicbor/latest/minicbor/encode/write/trait.Write.html#impl-Write-for-Vec%3Cu8%3E>
const INFALLIBLE_WRITE: &str = "write failed";

impl Encoder {
    pub fn new(writer: Vec<u8>) -> Self {
        Self {
            encoder: minicbor::Encoder::new(writer),
        }
    }

    delegate_method! {
        /// Used when it's not cheap to calculate the size, i.e. when the struct has one or more
        /// `Option`al members.
        begin_map => begin_map();
        /// Begins an indefinite-length array.
        begin_array => begin_array();
        /// Writes a boolean value.
        boolean => bool(x: bool);
        /// Writes a byte value.
        byte => i8(x: i8);
        /// Writes a short value.
        short => i16(x: i16);
        /// Writes an integer value.
        integer => i32(x: i32);
        /// Writes an long value.
        long => i64(x: i64);
        /// Writes an float value.
        float => f32(x: f32);
        /// Writes an double value.
        double => f64(x: f64);
        /// Writes a null tag.
        null => null();
        /// Writes an end tag.
        end => end();
    }

    /// Maximum size of a CBOR type+length header: 1 byte major type + up to 8 bytes for the length.
    const MAX_HEADER_LEN: usize = 9;

    /// Writes a CBOR type+length header directly to the writer.
    ///
    /// Encodes the "additional information" field per RFC 8949 §3:
    /// - 0..=23: length is stored directly in the low 5 bits of the initial byte.
    /// - 24: one-byte uint follows (value 24..=0xff).
    /// - 25: two-byte big-endian uint follows (value 0x100..=0xffff).
    /// - 26: four-byte big-endian uint follows (value 0x1_0000..=0xffff_ffff).
    /// - 27: eight-byte big-endian uint follows (larger values).
    #[inline]
    fn write_type_len(writer: &mut Vec<u8>, major: u8, len: usize) {
        let mut buf = [0u8; Self::MAX_HEADER_LEN];
        let n = match len {
            0..=23 => {
                buf[0] = major | len as u8;
                1
            }
            24..=0xff => {
                buf[0] = major | 24;
                buf[1] = len as u8;
                2
            }
            0x100..=0xffff => {
                buf[0] = major | 25;
                buf[1..3].copy_from_slice(&(len as u16).to_be_bytes());
                3
            }
            0x1_0000..=0xffff_ffff => {
                buf[0] = major | 26;
                buf[1..5].copy_from_slice(&(len as u32).to_be_bytes());
                5
            }
            _ => {
                buf[0] = major | 27;
                buf[1..9].copy_from_slice(&(len as u64).to_be_bytes());
                9
            }
        };
        writer.extend_from_slice(&buf[..n]);
    }

    /// Writes a definite length string. Collapses header+data into a single reserve+write.
    pub fn str(&mut self, x: &str) -> &mut Self {
        let writer = self.encoder.writer_mut();
        let len = x.len();
        writer.reserve(Self::MAX_HEADER_LEN + len);
        Self::write_type_len(writer, 0x60, len);
        writer.extend_from_slice(x.as_bytes());
        self
    }

    /// Writes a `BigInteger` using preferred serialization per RFC 8949 §3.4.3.
    ///
    /// Values that fit in a CBOR major type 0 or 1 integer are encoded directly
    /// (preferred serialization). Larger values use tag 2 (unsigned bignum) or
    /// tag 3 (negative bignum). For tag 3, the byte string encodes `n` where
    /// the value is `-1 - n`.
    pub fn big_integer(&mut self, value: &BigInteger) -> &mut Self {
        use num_bigint::{BigInt, Sign};

        let n: BigInt = value
            .as_ref()
            .parse()
            .expect("BigInteger contains invalid value");
        let (sign, magnitude) = n.to_bytes_be();

        match sign {
            Sign::Plus | Sign::NoSign => {
                // Try preferred serialization as major type 0.
                if magnitude.len() <= 8 {
                    let mut buf = [0u8; 8];
                    buf[8 - magnitude.len()..].copy_from_slice(&magnitude);
                    let val = u64::from_be_bytes(buf);
                    self.encoder.u64(val).expect(INFALLIBLE_WRITE);
                } else {
                    self.encoder
                        .tag(minicbor::data::Tag::new(2))
                        .expect(INFALLIBLE_WRITE);
                    // Preferred serialization: strip leading zeroes.
                    let stripped = strip_leading_zeroes(&magnitude);
                    self.encoder.bytes(stripped).expect(INFALLIBLE_WRITE);
                }
            }
            Sign::Minus => {
                // Tag 3 value = -1 - n, so n = -1 - value = |value| - 1.
                let one = BigInt::from(1u8);
                let n = (-n) - one;
                let (_, n_bytes) = n.to_bytes_be();

                // Try preferred serialization as major type 1.
                if n_bytes.len() <= 8 {
                    let mut buf = [0u8; 8];
                    buf[8 - n_bytes.len()..].copy_from_slice(&n_bytes);
                    let val = u64::from_be_bytes(buf);
                    // Use i128 to represent -1 - val without overflow, then
                    // convert to minicbor::data::Int which covers the full
                    // CBOR major type 1 range.
                    let neg = -1i128 - (val as i128);
                    let int_val = minicbor::data::Int::try_from(neg)
                        .expect("value fits in CBOR integer range");
                    self.encoder.int(int_val).expect(INFALLIBLE_WRITE);
                } else {
                    self.encoder
                        .tag(minicbor::data::Tag::new(3))
                        .expect(INFALLIBLE_WRITE);
                    let stripped = strip_leading_zeroes(&n_bytes);
                    self.encoder.bytes(stripped).expect(INFALLIBLE_WRITE);
                }
            }
        }
        self
    }

    /// Writes a blob from a byte slice. Collapses header+data into a single reserve+write.
    ///
    /// Mirrors [`Self::str`]'s slice-input style. Prefer this over [`Self::blob`]
    /// when the caller already holds a `&[u8]` (e.g. from a schema-serde
    /// `ShapeSerializer::write_blob(_, &[u8])` call) — it avoids needing to
    /// wrap the bytes in a [`Blob`] just to satisfy the API.
    pub fn blob_bytes(&mut self, data: &[u8]) -> &mut Self {
        let writer = self.encoder.writer_mut();
        let len = data.len();
        writer.reserve(Self::MAX_HEADER_LEN + len);
        Self::write_type_len(writer, 0x40, len);
        writer.extend_from_slice(data);
        self
    }

    /// Writes a blob. Collapses header+data into a single reserve+write.
    pub fn blob(&mut self, x: &Blob) -> &mut Self {
        self.blob_bytes(x.as_ref())
    }

    /// Writes a fixed length array of given length.
    pub fn array(&mut self, len: usize) -> &mut Self {
        Self::write_type_len(self.encoder.writer_mut(), 0x80, len);
        self
    }

    /// Writes a fixed length map of given length.
    /// Used when we know the size in advance, i.e.:
    /// - when a struct has all non-`Option`al members.
    /// - when serializing `union` shapes (they can only have one member set).
    /// - when serializing a `map` shape.
    pub fn map(&mut self, len: usize) -> &mut Self {
        Self::write_type_len(self.encoder.writer_mut(), 0xa0, len);
        self
    }

    pub fn timestamp(&mut self, x: &DateTime) -> &mut Self {
        self.encoder
            .tag(minicbor::data::Tag::from(
                minicbor::data::IanaTag::Timestamp,
            ))
            .expect(INFALLIBLE_WRITE);
        self.encoder.f64(x.as_secs_f64()).expect(INFALLIBLE_WRITE);
        self
    }

    pub fn into_writer(self) -> Vec<u8> {
        self.encoder.into_writer()
    }
}

/// Strips leading zero bytes from a big-endian byte slice.
fn strip_leading_zeroes(bytes: &[u8]) -> &[u8] {
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    &bytes[start..]
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_types::Blob;

    /// Verify our `str()` produces byte-identical output to minicbor's.
    #[test]
    fn str_matches_minicbor() {
        let cases = [
            "",                        // len 0
            "a",                       // len 1 (in 0..=23 range)
            "hello world!! test str",  // len 22 (still 0..=23)
            "this is exactly 24 char", // len 24 (0x18, first 1-byte length)
            &"x".repeat(0xff),         // len 255 (max 1-byte length)
            &"y".repeat(0x100),        // len 256 (first 2-byte length)
            &"z".repeat(0x1_0000),     // len 65536 (first 4-byte length)
        ];
        for input in &cases {
            let mut ours = Encoder::new(Vec::new());
            ours.str(input);

            let mut theirs = minicbor::Encoder::new(Vec::new());
            theirs.str(input).unwrap();

            assert_eq!(
                ours.into_writer(),
                theirs.into_writer(),
                "str mismatch for input len={}",
                input.len()
            );
        }
    }

    /// Verify our `blob()` produces byte-identical output to minicbor's.
    #[test]
    fn blob_matches_minicbor() {
        let cases: Vec<Vec<u8>> = vec![
            vec![],               // empty
            vec![0x42],           // 1 byte
            vec![0xAB; 23],       // max inline length
            vec![0xCD; 24],       // first 1-byte length
            vec![0xEF; 0xff],     // max 1-byte length
            vec![0x01; 0x100],    // first 2-byte length
            vec![0x02; 0x1_0000], // first 4-byte length
        ];
        for input in &cases {
            let mut ours = Encoder::new(Vec::new());
            ours.blob(&Blob::new(input.clone()));

            let mut theirs = minicbor::Encoder::new(Vec::new());
            theirs.bytes(input).unwrap();

            assert_eq!(
                ours.into_writer(),
                theirs.into_writer(),
                "blob mismatch for input len={}",
                input.len()
            );
        }
    }

    /// Verify chained `str()` calls don't corrupt encoder state for subsequent writes.
    #[test]
    fn str_chained_matches_minicbor() {
        let mut ours = Encoder::new(Vec::new());
        ours.str("key1").str("value1").str("key2").str("value2");

        let mut theirs = minicbor::Encoder::new(Vec::new());
        theirs
            .str("key1")
            .unwrap()
            .str("value1")
            .unwrap()
            .str("key2")
            .unwrap()
            .str("value2")
            .unwrap();

        assert_eq!(ours.into_writer(), theirs.into_writer());
    }

    /// Verify `str()` works correctly inside a map structure (the real-world hot path).
    #[test]
    fn str_inside_map_matches_minicbor() {
        let mut ours = Encoder::new(Vec::new());
        ours.begin_map().str("TableName").str("my-table").end();

        let mut theirs = minicbor::Encoder::new(Vec::new());
        theirs
            .begin_map()
            .unwrap()
            .str("TableName")
            .unwrap()
            .str("my-table")
            .unwrap()
            .end()
            .unwrap();

        assert_eq!(ours.into_writer(), theirs.into_writer());
    }

    /// Verify `str()` handles multi-byte UTF-8 correctly (CBOR text strings must be valid UTF-8).
    #[test]
    fn str_utf8_matches_minicbor() {
        let cases = [
            "café",          // 2-byte UTF-8
            "日本語",        // 3-byte UTF-8
            "🦀🔥",          // 4-byte UTF-8 (emoji)
            "mixed: aé日🦀", // all byte widths
        ];
        for input in &cases {
            let mut ours = Encoder::new(Vec::new());
            ours.str(input);

            let mut theirs = minicbor::Encoder::new(Vec::new());
            theirs.str(input).unwrap();

            assert_eq!(
                ours.into_writer(),
                theirs.into_writer(),
                "str UTF-8 mismatch for {:?}",
                input
            );
        }
    }

    #[test]
    fn preferred_serialization_small_positive() {
        // Small values use major type 0 directly, not tag 2.
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"0".parse().unwrap());
        assert_eq!(encoder.into_writer(), vec![0x00]); // major type 0, value 0

        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"23".parse().unwrap());
        assert_eq!(encoder.into_writer(), vec![0x17]); // major type 0, value 23

        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"256".parse().unwrap());
        // major type 0, additional info 25 (2-byte), 0x0100
        assert_eq!(encoder.into_writer(), vec![0x19, 0x01, 0x00]);
    }

    #[test]
    fn preferred_serialization_small_negative() {
        // Small negatives use major type 1 directly, not tag 3.
        // Major type 1 value = -1 - argument.
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"-1".parse().unwrap());
        assert_eq!(encoder.into_writer(), vec![0x20]); // -1 = -1-0, argument 0

        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"-42".parse().unwrap());
        // -42 = -1-41, argument 41 = 0x29 (major type 1, additional info 24, value 41)
        assert_eq!(encoder.into_writer(), vec![0x38, 0x29]);
    }

    #[test]
    fn preferred_serialization_u64_max() {
        // u64::MAX = 18446744073709551615 fits in major type 0.
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"18446744073709551615".parse().unwrap());
        let bytes = encoder.into_writer();
        assert_eq!(bytes[0], 0x1b); // major type 0, 8-byte argument
        assert_eq!(
            &bytes[1..],
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn tag2_for_values_exceeding_u64() {
        // 2^64 = 18446744073709551616 requires tag 2.
        // RFC 8949 Appendix A: 0xc249010000000000000000
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"18446744073709551616".parse().unwrap());
        let bytes = encoder.into_writer();
        assert_eq!(
            bytes,
            vec![0xc2, 0x49, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn tag3_negative_bignum_rfc8949_example() {
        // RFC 8949 Appendix A: -18446744073709551617 = 0xc349010000000000000000
        // value = -1 - n, n = 18446744073709551616 = 2^64
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"-18446744073709551617".parse().unwrap());
        let bytes = encoder.into_writer();
        assert_eq!(
            bytes,
            vec![0xc3, 0x49, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn negative_at_major_type_1_boundary() {
        // -18446744073709551616 = -1 - 18446744073709551615 = -1 - u64::MAX
        // This fits in major type 1 with 8-byte argument = u64::MAX.
        let mut encoder = Encoder::new(Vec::new());
        encoder.big_integer(&"-18446744073709551616".parse().unwrap());
        let bytes = encoder.into_writer();
        assert_eq!(bytes[0], 0x3b); // major type 1, 8-byte argument
        assert_eq!(
            &bytes[1..],
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn tag2_strips_leading_zeroes() {
        // A large number whose big-endian representation has no leading zeroes.
        let mut encoder = Encoder::new(Vec::new());
        let large = "123456789012345678901234567890";
        encoder.big_integer(&large.parse().unwrap());
        let bytes = encoder.into_writer();
        assert_eq!(bytes[0], 0xc2); // tag 2
                                    // Verify the byte string has no leading zero bytes.
                                    // bytes[1] is the CBOR byte string length header.
        let payload_start = if bytes[1] < 0x58 { 2 } else { 3 };
        assert_ne!(bytes[payload_start], 0x00);
    }
}
