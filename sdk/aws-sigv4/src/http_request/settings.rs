/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use http::header::{HeaderName, USER_AGENT};
use std::time::Duration;

/// HTTP signing parameters
pub type SigningParams<'a> = crate::SigningParams<'a, SigningSettings>;

/// HTTP-specific signing settings
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub struct SigningSettings {
    /// Specifies how to encode the request URL when signing. Some services do not decode
    /// the path prior to checking the signature, requiring clients to actually _double-encode_
    /// the URI in creating the canonical request in order to pass a signature check.
    pub percent_encoding_mode: PercentEncodingMode,

    /// Add an additional checksum header
    pub payload_checksum_kind: PayloadChecksumKind,

    /// Where to put the signature
    pub signature_location: SignatureLocation,

    /// For presigned requests, how long the presigned request is valid for
    pub expires_in: Option<Duration>,

    /// Headers that should be excluded from the signing process
    pub excluded_headers: Option<Vec<HeaderName>>,
}

/// HTTP payload checksum type
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum PayloadChecksumKind {
    /// Add x-amz-checksum-sha256 to the canonical request
    ///
    /// This setting is required for S3
    XAmzSha256,

    /// Do not add an additional header when creating the canonical request
    ///
    /// This is "normal mode" and will work for services other than S3
    NoHeader,
}

/// Config value to specify how to encode the request URL when signing.
///
/// We assume the URI will be encoded _once_ prior to transmission. Some services
/// do not decode the path prior to checking the signature, requiring clients to actually
/// _double-encode_ the URI in creating the canonical request in order to pass a signature check.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum PercentEncodingMode {
    /// Re-encode the resulting URL (e.g. %30 becomes `%2530)
    Double,

    /// Take the resulting URL as-is
    Single,
}

impl Default for SigningSettings {
    fn default() -> Self {
        // The user agent header should not be signed because it may be altered by proxies
        const EXCLUDED_HEADERS: [HeaderName; 1] = [USER_AGENT];

        Self {
            percent_encoding_mode: PercentEncodingMode::Double,
            payload_checksum_kind: PayloadChecksumKind::NoHeader,
            signature_location: SignatureLocation::Headers,
            expires_in: None,
            excluded_headers: Some(EXCLUDED_HEADERS.to_vec()),
        }
    }
}

/// Where to place signing values in the HTTP request
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SignatureLocation {
    /// Place the signature in the request headers
    Headers,
    /// Place the signature in the request query parameters
    QueryParams,
}
