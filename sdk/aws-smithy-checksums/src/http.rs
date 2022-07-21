/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Checksum support for HTTP requests and responses.

use crate::{Checksum, Crc32, Crc32c, Md5, Sha1, Sha256};
use std::fmt::{Display, Formatter};

use aws_smithy_types::base64;

use http::header::{HeaderMap, HeaderName, HeaderValue};

// Valid checksum algorithm names
pub const CRC_32_NAME: &str = "crc32";
pub const CRC_32_C_NAME: &str = "crc32c";
pub const SHA_1_NAME: &str = "sha1";
pub const SHA_256_NAME: &str = "sha256";

pub static CRC_32_HEADER_NAME: HeaderName = HeaderName::from_static("x-amz-checksum-crc32");
pub static CRC_32_C_HEADER_NAME: HeaderName = HeaderName::from_static("x-amz-checksum-crc32c");
pub static SHA_1_HEADER_NAME: HeaderName = HeaderName::from_static("x-amz-checksum-sha1");
pub static SHA_256_HEADER_NAME: HeaderName = HeaderName::from_static("x-amz-checksum-sha256");

// Preserved for compatibility purposes. This should never be used by users, only within smithy-rs
pub(crate) const MD5_NAME: &str = "md5";
pub(crate) static MD5_HEADER_NAME: HeaderName = HeaderName::from_static("content-md5");

/// Given a `&str` representing a checksum algorithm, return the corresponding `HeaderName`
/// for that checksum algorithm.
pub fn algorithm_to_header_name(
    checksum_algorithm: &str,
) -> Result<HeaderName, Box<dyn std::error::Error>> {
    if checksum_algorithm.eq_ignore_ascii_case(CRC_32_NAME) {
        Ok(CRC_32_HEADER_NAME.clone())
    } else if checksum_algorithm.eq_ignore_ascii_case(CRC_32_C_NAME) {
        Ok(CRC_32_C_HEADER_NAME.clone())
    } else if checksum_algorithm.eq_ignore_ascii_case(SHA_1_NAME) {
        Ok(SHA_1_HEADER_NAME.clone())
    } else if checksum_algorithm.eq_ignore_ascii_case(SHA_256_NAME) {
        Ok(SHA_256_HEADER_NAME.clone())
    } else if checksum_algorithm.eq_ignore_ascii_case(MD5_NAME) {
        Ok(MD5_HEADER_NAME.clone())
    } else {
        Err(Box::new(Error::UnknownChecksumAlgorithm(
            checksum_algorithm.to_owned(),
        )))
    }
}

/// Given a `HeaderName` representing a checksum algorithm, return the name of that algorithm
/// as a `&'static str`.
pub fn header_name_to_algorithm(
    checksum_header_name: &HeaderName,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    if checksum_header_name == CRC_32_HEADER_NAME {
        Ok(CRC_32_NAME)
    } else if checksum_header_name == CRC_32_C_HEADER_NAME {
        Ok(CRC_32_C_NAME)
    } else if checksum_header_name == SHA_1_HEADER_NAME {
        Ok(SHA_1_NAME)
    } else if checksum_header_name == SHA_256_HEADER_NAME {
        Ok(SHA_256_NAME)
    } else if checksum_header_name == MD5_HEADER_NAME {
        Ok(MD5_NAME)
    } else {
        Err(Box::new(Error::UnknownChecksumHeaderName(
            checksum_header_name.to_owned(),
        )))
    }
}

/// Create a new `Box<dyn HttpChecksum>` from an algorithm name. Valid algorithm names are defined
/// as `static`s in [this module](crate::http).
pub fn new_from_algorithm(
    checksum_algorithm: &str,
) -> Result<Box<dyn HttpChecksum>, Box<dyn std::error::Error>> {
    if checksum_algorithm.eq_ignore_ascii_case(CRC_32_NAME) {
        Ok(Box::new(Crc32::default()))
    } else if checksum_algorithm.eq_ignore_ascii_case(CRC_32_C_NAME) {
        Ok(Box::new(Crc32c::default()))
    } else if checksum_algorithm.eq_ignore_ascii_case(SHA_1_NAME) {
        Ok(Box::new(Sha1::default()))
    } else if checksum_algorithm.eq_ignore_ascii_case(SHA_256_NAME) {
        Ok(Box::new(Sha256::default()))
    } else if checksum_algorithm.eq_ignore_ascii_case(MD5_NAME) {
        // It's possible to create an MD5 and we do this in some situations for compatibility.
        // We deliberately hide this from users so that they don't go using it.
        Ok(Box::new(Md5::default()))
    } else {
        Err(Box::new(Error::UnknownChecksumAlgorithm(
            checksum_algorithm.to_owned(),
        )))
    }
}

#[derive(Debug, PartialEq)]
enum Error {
    UnknownChecksumAlgorithm(String),
    UnknownChecksumHeaderName(HeaderName),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownChecksumAlgorithm(algorithm) => {
                write!(
                    f,
                    // MD5 isn't mentioned because external users shouldn't be using MD5 checksums for anything
                    r#"unknown checksum algorithm "{}", please pass a known algorithm name ("crc32", "crc32c", "sha1", "sha256")"#,
                    algorithm
                )
            }
            Self::UnknownChecksumHeaderName(header_name) => {
                write!(
                    f,
                    // MD5 isn't mentioned because external users shouldn't be using MD5 checksums for anything
                    r#"unknown checksum algorithm "{}", please pass a known checksum header name (
    "x-amz-checksum-crc32",
    "x-amz-checksum-crc32c",
    "x-amz-checksum-sha1",
    "x-amz-checksum-sha256",
)"#,
                    header_name.as_str()
                )
            }
        }
    }
}

impl std::error::Error for Error {}

/// When a response has to be checksum-verified, we have to check possible headers until we find the
/// header with the precalculated checksum. Because a service may send back multiple headers, we have
/// to check them in order based on how fast each checksum is to calculate.
pub const CHECKSUM_ALGORITHMS_IN_PRIORITY_ORDER: [&str; 4] =
    [CRC_32_C_NAME, CRC_32_NAME, SHA_1_NAME, SHA_256_NAME];

/// Checksum algorithms are use to validate the integrity of data. Structs that implement this trait
/// can be used as checksum calculators. This trait requires Send + Sync because these checksums are
/// often used in a threaded context.
pub trait HttpChecksum: Checksum + Send + Sync {
    /// Either return this checksum as a `HeaderMap` containing one HTTP header, or return an error
    /// describing why checksum calculation failed.
    fn headers(self: Box<Self>) -> HeaderMap<HeaderValue> {
        let mut header_map = HeaderMap::new();
        header_map.insert(self.header_name(), self.header_value());

        header_map
    }

    /// Return the `HeaderName` used to represent this checksum algorithm
    fn header_name(&self) -> HeaderName;

    /// Return the calculated checksum as a base64-encoded `HeaderValue`
    fn header_value(self: Box<Self>) -> HeaderValue {
        let hash = self.finalize();
        HeaderValue::from_str(&base64::encode(&hash[..]))
            .expect("base64 encoded bytes are always valid header values")
    }

    /// Return the size of the base64-encoded `HeaderValue` for this checksum
    fn size(&self) -> u64 {
        let trailer_name_size_in_bytes = self.header_name().as_str().len() as u64;
        let base64_encoded_checksum_size_in_bytes = base64::encoded_length(Checksum::size(self));

        trailer_name_size_in_bytes
            // HTTP trailer names and values may be separated by either a single colon or a single
            // colon and a whitespace. In the AWS Rust SDK, we use a single colon.
            + ":".len() as u64
            + base64_encoded_checksum_size_in_bytes
    }
}

impl HttpChecksum for Crc32 {
    fn header_name(&self) -> HeaderName {
        CRC_32_HEADER_NAME.clone()
    }
}

impl HttpChecksum for Crc32c {
    fn header_name(&self) -> HeaderName {
        CRC_32_C_HEADER_NAME.clone()
    }
}

impl HttpChecksum for Sha1 {
    fn header_name(&self) -> HeaderName {
        SHA_1_HEADER_NAME.clone()
    }
}

impl HttpChecksum for Sha256 {
    fn header_name(&self) -> HeaderName {
        SHA_256_HEADER_NAME.clone()
    }
}

impl HttpChecksum for Md5 {
    fn header_name(&self) -> HeaderName {
        MD5_HEADER_NAME.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        new_from_algorithm, HttpChecksum, CRC_32_C_NAME, CRC_32_NAME, SHA_1_NAME, SHA_256_NAME,
    };
    use crate::http::{algorithm_to_header_name, header_name_to_algorithm};
    use aws_smithy_types::base64;
    use bytes::Bytes;
    use http::header::HeaderName;

    #[test]
    fn test_trailer_length_of_crc32_checksum_body() {
        let checksum = new_from_algorithm(CRC_32_NAME).unwrap();
        let expected_size = 29;
        let actual_size = HttpChecksum::size(&*checksum);
        assert_eq!(expected_size, actual_size)
    }

    #[test]
    fn test_trailer_value_of_crc32_checksum_body() {
        let checksum = new_from_algorithm(CRC_32_NAME).unwrap();
        // The CRC32 of an empty string is all zeroes
        let expected_value = Bytes::from_static(b"    ");
        let expected_value = base64::encode(&expected_value);
        let actual_value = checksum.header_value();
        assert_eq!(expected_value, actual_value)
    }

    #[test]
    fn test_trailer_length_of_crc32c_checksum_body() {
        let checksum = new_from_algorithm(CRC_32_C_NAME).unwrap();
        let expected_size = 30;
        let actual_size = HttpChecksum::size(&*checksum);
        assert_eq!(expected_size, actual_size)
    }

    #[test]
    fn test_trailer_value_of_crc32c_checksum_body() {
        let checksum = new_from_algorithm(CRC_32_C_NAME).unwrap();
        // The CRC32C of an empty string is all zeroes
        let expected_value = Bytes::from_static(b"    ");
        let expected_value = base64::encode(&expected_value);
        let actual_value = checksum.header_value();
        assert_eq!(expected_value, actual_value)
    }

    #[test]
    fn test_trailer_length_of_sha1_checksum_body() {
        let checksum = new_from_algorithm(SHA_1_NAME).unwrap();
        let expected_size = 48;
        let actual_size = HttpChecksum::size(&*checksum);
        assert_eq!(expected_size, actual_size)
    }

    #[test]
    fn test_trailer_value_of_sha1_checksum_body() {
        let checksum = new_from_algorithm(SHA_1_NAME).unwrap();
        // The SHA1 of an empty string is da39a3ee5e6b4b0d3255bfef95601890afd80709
        let expected_value = Bytes::from_static(&[
            0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60,
            0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09,
        ]);
        let expected_value = base64::encode(&expected_value);
        let actual_value = checksum.header_value();
        assert_eq!(expected_value, actual_value)
    }

    #[test]
    fn test_trailer_length_of_sha256_checksum_body() {
        let checksum = new_from_algorithm(SHA_256_NAME).unwrap();
        let expected_size = 66;
        let actual_size = HttpChecksum::size(&*checksum);
        assert_eq!(expected_size, actual_size)
    }

    #[test]
    fn test_trailer_value_of_sha256_checksum_body() {
        let checksum = new_from_algorithm(SHA_256_NAME).unwrap();
        // The SHA256 of an empty string is e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let expected_value = Bytes::from_static(&[
            0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14, 0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f,
            0xb9, 0x24, 0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c, 0xa4, 0x95, 0x99, 0x1b,
            0x78, 0x52, 0xb8, 0x55,
        ]);
        let expected_value = base64::encode(&expected_value);
        let actual_value = checksum.header_value();
        assert_eq!(expected_value, actual_value)
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: UnknownChecksumAlgorithm(\"some invalid checksum algorithm\")"]
    fn test_algorithm_to_header_name_returns_error_for_unknown() {
        algorithm_to_header_name("some invalid checksum algorithm").unwrap();
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: UnknownChecksumHeaderName(\"some-invalid-checksum-header-name\")"]
    fn test_header_name_to_algorithm_returns_error_for_unknown() {
        let header_name = HeaderName::from_static("some-invalid-checksum-header-name");
        header_name_to_algorithm(&header_name).unwrap();
    }

    #[test]
    #[should_panic = "called `Result::unwrap()` on an `Err` value: UnknownChecksumAlgorithm(\"some invalid checksum algorithm\")"]
    fn test_new_from_algorithm_returns_error_for_unknown() {
        new_from_algorithm("some invalid checksum algorithm").unwrap();
    }
}
