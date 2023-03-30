/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functions to create signing keys and calculate signatures.

use crate::date_time::format_date;
use hmac::{digest::FixedOutput, Hmac, Mac};
use sha2::{Digest, Sha256};
use std::time::SystemTime;

/// HashedPayload = Lowercase(HexEncode(Hash(requestPayload)))
#[allow(dead_code)] // Unused when compiling without certain features
pub(crate) fn sha256_hex_string(bytes: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize_fixed())
}

/// Calculates a Sigv4 signature
pub fn calculate_signature(signing_key: impl AsRef<[u8]>, string_to_sign: &[u8]) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_key.as_ref())
        .expect("HMAC can take key of any size");
    mac.update(string_to_sign);
    hex::encode(mac.finalize_fixed())
}

/// Generates a signing key for Sigv4
pub fn generate_signing_key(
    secret: &str,
    time: SystemTime,
    region: &str,
    service: &str,
) -> impl AsRef<[u8]> {
    // kSecret = your secret access key
    // kDate = HMAC("AWS4" + kSecret, Date)
    // kRegion = HMAC(kDate, Region)
    // kService = HMAC(kRegion, Service)
    // kSigning = HMAC(kService, "aws4_request")

    let secret = format!("AWS4{}", secret);
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_ref()).expect("HMAC can take key of any size");
    mac.update(format_date(time).as_bytes());
    let tag = mac.finalize_fixed();

    // sign region
    let mut mac = Hmac::<Sha256>::new_from_slice(&tag).expect("HMAC can take key of any size");
    mac.update(region.as_bytes());
    let tag = mac.finalize_fixed();

    // sign service
    let mut mac = Hmac::<Sha256>::new_from_slice(&tag).expect("HMAC can take key of any size");
    mac.update(service.as_bytes());
    let tag = mac.finalize_fixed();

    // sign request
    let mut mac = Hmac::<Sha256>::new_from_slice(&tag).expect("HMAC can take key of any size");
    mac.update("aws4_request".as_bytes());
    mac.finalize_fixed()
}

#[cfg(test)]
mod tests {
    use super::{calculate_signature, generate_signing_key};
    use crate::date_time::test_parsers::parse_date_time;
    use crate::http_request::test::test_canonical_request;
    use crate::sign::sha256_hex_string;

    #[test]
    fn test_signature_calculation() {
        let secret = "wJalrXUtnFEMI/K7MDENG+bPxRfiCYEXAMPLEKEY";
        let creq = test_canonical_request("iam");
        let time = parse_date_time("20150830T123600Z").unwrap();

        let derived_key = generate_signing_key(secret, time, "us-east-1", "iam");
        let signature = calculate_signature(derived_key, creq.as_bytes());

        let expected = "5d672d79c15b13162d9279b0855cfba6789a8edb4c82c400e06b5924a6f2b5d7";
        assert_eq!(expected, &signature);
    }

    #[test]
    fn sign_payload_empty_string() {
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let actual = sha256_hex_string([]);
        assert_eq!(expected, actual);
    }
}
