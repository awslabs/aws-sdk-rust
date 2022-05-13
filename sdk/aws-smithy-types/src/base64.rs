/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A correct, small, but not especially fast base64 implementation

use std::error::Error;
use std::fmt;

const BASE64_ENCODE_TABLE: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const BASE64_DECODE_TABLE: &[Option<u8>; 256] = &decode_table();

const PADDING_SENTINEL: u8 = 0xFF;

const fn encode_table_index_of(i: usize) -> Option<u8> {
    let mut index = 0;
    // inline const index-of implementation
    while index < BASE64_ENCODE_TABLE.len() {
        if BASE64_ENCODE_TABLE[index] as usize == i {
            return Some(index as u8);
        }
        index += 1;
    }
    None
}

/// Build a decode table mapping `char as u8` to base64 bit sequences
const fn decode_table() -> [Option<u8>; 256] {
    let mut output = [None; 256];
    let mut i = 0;
    while i < 256 {
        if i == 61 {
            output[i] = Some(PADDING_SENTINEL);
        } else {
            output[i] = encode_table_index_of(i);
        }
        i += 1;
    }
    output
}

/// Encode `input` into base64 using the standard base64 alphabet
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    encode_inner(input.as_ref())
}

/// encode_inner defined to reduce monomorphisation cost
fn encode_inner(inp: &[u8]) -> String {
    // Base 64 encodes groups of 6 bits into characters—this means that each
    // 3 byte group (24 bits) is encoded into 4 base64 characters.
    let char_ct = ((inp.len() + 2) / 3) * 4;
    let mut output = String::with_capacity(char_ct);
    for chunk in inp.chunks(3) {
        let mut block: i32 = 0;
        // Write the chunks into the beginning of a 32 bit int
        for (idx, chunk) in chunk.iter().enumerate() {
            block |= (*chunk as i32) << ((3 - idx) * 8);
        }
        let num_sextets = ((chunk.len() * 8) + 5) / 6;
        for idx in 0..num_sextets {
            let slice = block >> (26 - (6 * idx));
            let idx = (slice as u8) & 0b0011_1111;
            output.push(BASE64_ENCODE_TABLE[idx as usize] as char);
        }
        for _ in 0..(4 - num_sextets) {
            output.push('=');
        }
    }
    // be sure we calculated the size right
    debug_assert_eq!(output.capacity(), char_ct);
    output
}

/// Decode `input` from base64 using the standard base64 alphabet
///
/// If input is not a valid base64 encoded string, this function will return `DecodeError`.
pub fn decode<T: AsRef<str>>(input: T) -> Result<Vec<u8>, DecodeError> {
    decode_inner(input.as_ref())
}

/// Failure to decode a base64 value.
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum DecodeError {
    /// Encountered an invalid byte.
    InvalidByte,
    /// Encountered an invalid base64 padding value.
    InvalidPadding,
    /// Input wasn't long enough to be a valid base64 value.
    InvalidLength,
}

impl Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DecodeError::*;
        match self {
            InvalidByte => write!(f, "invalid byte"),
            InvalidPadding => write!(f, "invalid padding"),
            InvalidLength => write!(f, "invalid length"),
        }
    }
}

fn decode_inner(inp: &str) -> Result<Vec<u8>, DecodeError> {
    // one base64 character is only 6 bits so it can't produce valid data.
    if inp.len() == 1 {
        return Err(DecodeError::InvalidLength);
    }

    // when there's padding, we might slightly over allocate but it significantly simplifies
    // the code to just ignore it.
    let mut ret = Vec::with_capacity((inp.len() + 3) / 4 * 3);

    // 4 base-64 characters = 3 bytes
    // 1. Break the input into 4 character segments
    // 2. Write those segments into an i32
    // 3. Read u8s back out of the i32
    let chunks = inp.as_bytes().chunks(4);
    let mut padding = 0;
    for chunk in chunks {
        // padding should only be set on the last input
        if padding != 0 {
            return Err(DecodeError::InvalidPadding);
        }
        let mut block = 0_i32;
        for (idx, chunk) in chunk.iter().enumerate() {
            let bits = BASE64_DECODE_TABLE[*chunk as usize].ok_or(DecodeError::InvalidByte)?;
            if bits == 0xFF {
                padding += 1;
            } else if padding > 0 {
                // Once you've started padding, you can't stop.
                return Err(DecodeError::InvalidPadding);
            }
            block |= (bits as i32) << (18 - (idx * 6));
        }
        // if we got a short slice, its because of implied padding
        let missing_chars = 4 - chunk.len();
        for i in (padding + missing_chars..3).rev() {
            let byte = ((block >> (i * 8)) & 0xFF) as u8;
            ret.push(byte)
        }
    }

    // The code is much simpler if we _slightly_ over allocate in certain cases
    debug_assert!(ret.capacity() - ret.len() < 4);
    Ok(ret)
}

#[cfg(test)]
mod test {
    use crate::base64::{decode, encode, DecodeError, BASE64_DECODE_TABLE, BASE64_ENCODE_TABLE};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn doesnt_crash_encode(v in any::<Vec<u8>>()) {
            encode(v);
        }

        #[test]
        fn doesnt_crash_decode(v in any::<String>()) {
            let us = decode(&v);
            let correct = ::base64::decode(&v);
            if correct.is_ok() {
                us.expect("we should be able to read all base64 the oracle can");
            }
        }

        #[test]
        fn round_trip(v in any::<Vec<u8>>()) {
            let as_b64 = encode(v.as_slice());
            let decoded = decode(as_b64).unwrap();
            assert_eq!(v, decoded);
        }

        #[test]
        fn vs_oracle(v in any::<Vec<u8>>()) {
            let correct = ::base64::encode(v.as_slice());
            let ours = encode(v.as_slice());
            assert_eq!(ours, correct);
        }
    }

    #[test]
    fn test_base64() {
        assert_eq!(encode("abc"), "YWJj");
        assert_eq!(decode("YWJj").unwrap(), b"abc");
        assert_eq!(decode("YQ==").unwrap(), b"a");
        assert_eq!(encode("anything you want."), "YW55dGhpbmcgeW91IHdhbnQu");
        assert_eq!(encode("anything you want"), "YW55dGhpbmcgeW91IHdhbnQ=");
        assert_eq!(encode("anything you wan"), "YW55dGhpbmcgeW91IHdhbg==");
    }

    #[test]
    fn test_invalid_padding() {
        // no internal padding
        assert_eq!(decode("ab=d"), Err(DecodeError::InvalidPadding));
        // too much padding
        assert_eq!(decode("abcd====="), Err(DecodeError::InvalidPadding));
        // no internal padding
        assert_eq!(decode("abc=defg"), Err(DecodeError::InvalidPadding));
        // not enough padding
        assert_eq!(decode("YQ").unwrap(), b"a");

        // no length-1 inputs are valid
        assert_eq!(decode("a"), Err(DecodeError::InvalidLength));

        // weird edge case, handled as a coincidence
        assert_eq!(decode("====").unwrap(), b"");
    }

    #[test]
    fn test_base64_long() {
        let decoded = "Alas, eleventy-one years is far too short a time to live among such excellent and admirable hobbits. I don't know half of you half as well as I should like, and I like less than half of you half as well as you deserve.";
        let encoded = "QWxhcywgZWxldmVudHktb25lIHllYXJzIGlzIGZhciB0b28gc2hvcnQgYSB0aW1lIHRvIGxpdmUgYW1vbmcgc3VjaCBleGNlbGxlbnQgYW5kIGFkbWlyYWJsZSBob2JiaXRzLiBJIGRvbid0IGtub3cgaGFsZiBvZiB5b3UgaGFsZiBhcyB3ZWxsIGFzIEkgc2hvdWxkIGxpa2UsIGFuZCBJIGxpa2UgbGVzcyB0aGFuIGhhbGYgb2YgeW91IGhhbGYgYXMgd2VsbCBhcyB5b3UgZGVzZXJ2ZS4=";
        assert_eq!(encode(decoded), encoded);
        assert_eq!(decode(encoded).unwrap(), decoded.as_bytes());
    }

    #[test]
    fn test_base64_utf8() {
        let decoded = "ユニコードとはか？";
        let encoded = "44Om44OL44Kz44O844OJ44Go44Gv44GL77yf";
        assert_eq!(encode(decoded), encoded);
        assert_eq!(decode(encoded).unwrap(), decoded.as_bytes());
    }

    #[test]
    fn test_base64_control_chars() {
        let decoded = "hello\tworld\n";
        let encoded = "aGVsbG8Jd29ybGQK";
        assert_eq!(encode(decoded), encoded);
    }

    #[test]
    fn test_decode_table() {
        assert_eq!(BASE64_DECODE_TABLE[0], None);
        assert_eq!(BASE64_DECODE_TABLE['A' as usize], Some(0));
        assert_eq!(BASE64_DECODE_TABLE['B' as usize], Some(1));
        for i in 0..64 {
            let encoded = BASE64_ENCODE_TABLE[i];
            let decoded = BASE64_DECODE_TABLE[encoded as usize];
            assert_eq!(decoded, Some(i as u8))
        }
    }
}
