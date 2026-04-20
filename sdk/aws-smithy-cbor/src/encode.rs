/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::{Blob, DateTime};

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

    /// Writes a blob. Collapses header+data into a single reserve+write.
    pub fn blob(&mut self, x: &Blob) -> &mut Self {
        let data = x.as_ref();
        let writer = self.encoder.writer_mut();
        let len = data.len();
        writer.reserve(Self::MAX_HEADER_LEN + len);
        Self::write_type_len(writer, 0x40, len);
        writer.extend_from_slice(data);
        self
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

#[cfg(test)]
mod tests {
    use super::Encoder;
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
}
