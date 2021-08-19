/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Provides functions for calculating Sigv4 signing keys, signatures, and
//! optional utilities for signing HTTP requests and Event Stream messages.

use chrono::{DateTime, Utc};

pub mod sign;

mod date_fmt;

#[cfg(feature = "sign-eventstream")]
pub mod event_stream;

#[cfg(feature = "sign-http")]
pub mod http_request;

/// Parameters to use when signing.
pub struct SigningParams<'a, S> {
    /// Access Key ID to use.
    pub access_key: &'a str,
    /// Secret access key to use.
    pub secret_key: &'a str,
    /// (Optional) Security token to use.
    pub security_token: Option<&'a str>,

    /// Region to sign for.
    pub region: &'a str,
    /// AWS Service Name to sign for.
    pub service_name: &'a str,
    /// Timestamp to use in the signature (should be `Utc::now()` unless testing).
    pub date_time: DateTime<Utc>,

    /// Additional signing settings. These differ between HTTP and Event Stream.
    pub settings: S,
}

/// Container for the signed output and the signature.
pub struct SigningOutput<T> {
    output: T,
    signature: String,
}

impl<T> SigningOutput<T> {
    pub fn new(output: T, signature: String) -> Self {
        Self { output, signature }
    }

    pub fn output(&self) -> &T {
        &self.output
    }

    pub fn signature(&self) -> &str {
        &self.signature
    }

    pub fn into_parts(self) -> (T, String) {
        (self.output, self.signature)
    }
}
