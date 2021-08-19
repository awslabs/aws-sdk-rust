/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Functions to create signing keys and calculate signatures.

use crate::date_fmt::format_date;
use chrono::{Date, Utc};
use ring::{
    digest::{self},
    hmac::{self, Key, Tag},
};

/// HashedPayload = Lowercase(HexEncode(Hash(requestPayload)))
#[allow(dead_code)] // Unused when compiling without certain features
pub(crate) fn sha256_hex_string(bytes: impl AsRef<[u8]>) -> String {
    // hex::encode returns a lowercase string
    hex::encode(digest::digest(&digest::SHA256, bytes.as_ref()))
}

/// Calculates a Sigv4 signature
pub fn calculate_signature(signing_key: Tag, string_to_sign: &[u8]) -> String {
    let s_key = Key::new(hmac::HMAC_SHA256, signing_key.as_ref());
    let tag = hmac::sign(&s_key, string_to_sign);
    hex::encode(tag)
}

/// Generates a signing key for Sigv4
pub fn generate_signing_key(
    secret: &str,
    date: Date<Utc>,
    region: &str,
    service: &str,
) -> hmac::Tag {
    // kSecret = your secret access key
    // kDate = HMAC("AWS4" + kSecret, Date)
    // kRegion = HMAC(kDate, Region)
    // kService = HMAC(kRegion, Service)
    // kSigning = HMAC(kService, "aws4_request")

    let secret = format!("AWS4{}", secret);
    let secret = hmac::Key::new(hmac::HMAC_SHA256, &secret.as_bytes());
    let tag = hmac::sign(&secret, format_date(&date).as_bytes());

    // sign region
    let key = hmac::Key::new(hmac::HMAC_SHA256, tag.as_ref());
    let tag = hmac::sign(&key, region.as_bytes());

    // sign service
    let key = hmac::Key::new(hmac::HMAC_SHA256, tag.as_ref());
    let tag = hmac::sign(&key, service.as_bytes());

    // sign request
    let key = hmac::Key::new(hmac::HMAC_SHA256, tag.as_ref());
    hmac::sign(&key, "aws4_request".as_bytes())
}
