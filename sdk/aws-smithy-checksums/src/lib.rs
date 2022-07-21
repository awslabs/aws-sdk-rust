/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Checksum calculation and verification callbacks

use aws_smithy_http::callback::BodyCallback;
use aws_smithy_types::base64;

use http::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::Digest;
use std::io::Write;

const CRC_32_NAME: &str = "x-amz-checksum-crc32";
const CRC_32_C_NAME: &str = "x-amz-checksum-crc32c";
const SHA_1_NAME: &str = "x-amz-checksum-sha1";
const SHA_256_NAME: &str = "x-amz-checksum-sha256";

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Default)]
struct Crc32callback {
    hasher: crc32fast::Hasher,
}

impl Crc32callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.hasher.update(bytes);

        Ok(())
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        let mut header_map = HeaderMap::new();
        let key = HeaderName::from_static(CRC_32_NAME);
        // We clone the hasher because `Hasher::finalize` consumes `self`
        let hash = self.hasher.clone().finalize();
        let value = HeaderValue::from_str(&base64::encode(u32::to_be_bytes(hash)))
            .expect("base64 will always produce valid header values from checksums");

        header_map.insert(key, value);

        Ok(Some(header_map))
    }
}

impl BodyCallback for Crc32callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.update(bytes)
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        self.trailers()
    }

    fn make_new(&self) -> Box<dyn BodyCallback> {
        Box::new(Crc32callback::default())
    }
}

#[derive(Debug, Default)]
struct Crc32cCallback {
    state: Option<u32>,
}

impl Crc32cCallback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.state = match self.state {
            Some(crc) => Some(crc32c::crc32c_append(crc, bytes)),
            None => Some(crc32c::crc32c(bytes)),
        };

        Ok(())
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        let mut header_map = HeaderMap::new();
        let key = HeaderName::from_static(CRC_32_C_NAME);
        // If no data was provided to this callback and no CRC was ever calculated, return zero as the checksum.
        let hash = self.state.unwrap_or_default();
        let value = HeaderValue::from_str(&base64::encode(u32::to_be_bytes(hash)))
            .expect("base64 will always produce valid header values from checksums");

        header_map.insert(key, value);

        Ok(Some(header_map))
    }
}

impl BodyCallback for Crc32cCallback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.update(bytes)
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        self.trailers()
    }

    fn make_new(&self) -> Box<dyn BodyCallback> {
        Box::new(Crc32cCallback::default())
    }
}

#[derive(Debug, Default)]
struct Sha1Callback {
    hasher: sha1::Sha1,
}

impl Sha1Callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.hasher.write_all(bytes)?;

        Ok(())
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        let mut header_map = HeaderMap::new();
        let key = HeaderName::from_static(SHA_1_NAME);
        // We clone the hasher because `Hasher::finalize` consumes `self`
        let hash = self.hasher.clone().finalize();
        let value = HeaderValue::from_str(&base64::encode(&hash[..]))
            .expect("base64 will always produce valid header values from checksums");

        header_map.insert(key, value);

        Ok(Some(header_map))
    }
}

impl BodyCallback for Sha1Callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.update(bytes)
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        self.trailers()
    }

    fn make_new(&self) -> Box<dyn BodyCallback> {
        Box::new(Sha1Callback::default())
    }
}

#[derive(Debug, Default)]
struct Sha256Callback {
    hasher: sha2::Sha256,
}

impl Sha256Callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.hasher.write_all(bytes)?;

        Ok(())
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        let mut header_map = HeaderMap::new();
        let key = HeaderName::from_static(SHA_256_NAME);
        // We clone the hasher because `Hasher::finalize` consumes `self`
        let hash = self.hasher.clone().finalize();
        let value = HeaderValue::from_str(&base64::encode(&hash[..]))
            .expect("base64 will always produce valid header values from checksums");

        header_map.insert(key, value);

        Ok(Some(header_map))
    }
}

impl BodyCallback for Sha256Callback {
    fn update(&mut self, bytes: &[u8]) -> Result<(), BoxError> {
        self.update(bytes)
    }

    fn trailers(&self) -> Result<Option<HeaderMap<HeaderValue>>, BoxError> {
        self.trailers()
    }

    fn make_new(&self) -> Box<dyn BodyCallback> {
        Box::new(Sha256Callback::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Crc32cCallback, Crc32callback, Sha1Callback, Sha256Callback, CRC_32_C_NAME, CRC_32_NAME,
        SHA_1_NAME, SHA_256_NAME,
    };

    use aws_smithy_types::base64;
    use http::HeaderValue;
    use pretty_assertions::assert_eq;

    const TEST_DATA: &str = r#"test data"#;

    fn header_value_as_checksum_string(header_value: &HeaderValue) -> String {
        let decoded_checksum = base64::decode(header_value.to_str().unwrap()).unwrap();
        let decoded_checksum = decoded_checksum
            .into_iter()
            .map(|byte| format!("{:02X?}", byte))
            .collect::<String>();

        format!("0x{}", decoded_checksum)
    }

    #[test]
    fn test_crc32_checksum() {
        let mut checksum_callback = Crc32callback::default();
        checksum_callback.update(TEST_DATA.as_bytes()).unwrap();
        let checksum_callback_result = checksum_callback.trailers().unwrap().unwrap();
        let encoded_checksum = checksum_callback_result.get(CRC_32_NAME).unwrap();
        let decoded_checksum = header_value_as_checksum_string(encoded_checksum);

        let expected_checksum = "0xD308AEB2";

        assert_eq!(decoded_checksum, expected_checksum);
    }

    #[test]
    fn test_crc32c_checksum() {
        let mut checksum_callback = Crc32cCallback::default();
        checksum_callback.update(TEST_DATA.as_bytes()).unwrap();
        let checksum_callback_result = checksum_callback.trailers().unwrap().unwrap();
        let encoded_checksum = checksum_callback_result.get(CRC_32_C_NAME).unwrap();
        let decoded_checksum = header_value_as_checksum_string(encoded_checksum);

        let expected_checksum = "0x3379B4CA";

        assert_eq!(decoded_checksum, expected_checksum);
    }

    #[test]
    fn test_sha1_checksum() {
        let mut checksum_callback = Sha1Callback::default();
        checksum_callback.update(TEST_DATA.as_bytes()).unwrap();
        let checksum_callback_result = checksum_callback.trailers().unwrap().unwrap();
        let encoded_checksum = checksum_callback_result.get(SHA_1_NAME).unwrap();
        let decoded_checksum = header_value_as_checksum_string(encoded_checksum);

        let expected_checksum = "0xF48DD853820860816C75D54D0F584DC863327A7C";

        assert_eq!(decoded_checksum, expected_checksum);
    }

    #[test]
    fn test_sha256_checksum() {
        let mut checksum_callback = Sha256Callback::default();
        checksum_callback.update(TEST_DATA.as_bytes()).unwrap();
        let checksum_callback_result = checksum_callback.trailers().unwrap().unwrap();
        let encoded_checksum = checksum_callback_result.get(SHA_256_NAME).unwrap();
        let decoded_checksum = header_value_as_checksum_string(encoded_checksum);

        let expected_checksum =
            "0x916F0027A575074CE72A331777C3478D6513F786A591BD892DA1A577BF2335F9";

        assert_eq!(decoded_checksum, expected_checksum);
    }
}
