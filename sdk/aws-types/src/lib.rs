/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Cross-service types for the AWS SDK.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

pub mod app_name;
pub mod build_metadata;
#[deprecated(since = "0.9.0", note = "renamed to sdk_config")]
pub mod config;
pub mod endpoint;
#[doc(hidden)]
pub mod os_shim_internal;
pub mod region;
pub mod sdk_config;

pub use aws_smithy_client::http_connector;
pub use sdk_config::SdkConfig;

use std::borrow::Cow;

/// The name of the service used to sign this request
///
/// Generally, user code should never interact with `SigningService` directly
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SigningService(Cow<'static, str>);
impl AsRef<str> for SigningService {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SigningService {
    /// Creates a `SigningService` from a static str.
    pub fn from_static(service: &'static str) -> Self {
        SigningService(Cow::Borrowed(service))
    }
}

impl From<String> for SigningService {
    fn from(service: String) -> Self {
        SigningService(Cow::Owned(service))
    }
}

impl From<&'static str> for SigningService {
    fn from(service: &'static str) -> Self {
        Self::from_static(service)
    }
}
