/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP Version-related code

use http::Version as HttpVersion;
use once_cell::sync::Lazy;

/// A list of supported or desired HttpVersions. Typically use when requesting an HTTP Client from a
/// client cache.
pub type HttpVersionList = Vec<HttpVersion>;

/// The default list of desired HTTP protocol versions to use when making requests
pub static DEFAULT_HTTP_VERSION_LIST: Lazy<HttpVersionList> =
    Lazy::new(|| vec![HttpVersion::HTTP_11]);
