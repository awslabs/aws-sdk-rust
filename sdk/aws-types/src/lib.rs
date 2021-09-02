/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

pub mod build_metadata;
// internal APIs, may be unstable
pub mod config;
pub mod credentials;
#[doc(hidden)]
pub mod os_shim_internal;
pub mod region;

pub use credentials::Credentials;

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
    pub fn from_static(service: &'static str) -> Self {
        SigningService(Cow::Borrowed(service))
    }
}
