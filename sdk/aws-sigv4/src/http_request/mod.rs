/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Utilities to sign HTTP requests.
//!
//! # Example: Signing an HTTP request
//!
//! ```rust
//! # fn test() -> Result<(), aws_sigv4::http_request::SigningError> {
//! use aws_sigv4::http_request::{sign, SigningSettings, SigningParams, SignableRequest};
//! use http;
//! use std::time::SystemTime;
//!
//! // Create the request to sign
//! let mut request = http::Request::builder()
//!     .uri("https://some-endpoint.some-region.amazonaws.com")
//!     .body("")
//!     .unwrap();
//!
//! // Set up information and settings for the signing
//! let signing_settings = SigningSettings::default();
//! let signing_params = SigningParams::builder()
//!     .access_key("example access key")
//!     .secret_key("example secret key")
//!     .region("us-east-1")
//!     .service_name("exampleservice")
//!     .time(SystemTime::now())
//!     .settings(signing_settings)
//!     .build()
//!     .unwrap();
//! // Convert the HTTP request into a signable request
//! let signable_request = SignableRequest::from(&request);
//!
//! // Sign and then apply the signature to the request
//! let (signing_instructions, _signature) = sign(signable_request, &signing_params)?.into_parts();
//! signing_instructions.apply_to_request(&mut request);
//! # Ok(())
//! # }
//! ```
//!

mod canonical_request;
mod error;
mod settings;
mod sign;
mod uri_path_normalization;
mod url_escape;

#[cfg(test)]
pub(crate) mod test;

pub use error::SigningError;
pub use settings::{
    PayloadChecksumKind, PercentEncodingMode, SignatureLocation, SigningParams, SigningSettings,
    UriPathNormalizationMode,
};
pub use sign::{sign, SignableBody, SignableRequest};
