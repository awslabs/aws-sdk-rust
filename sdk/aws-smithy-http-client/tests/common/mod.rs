/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Shared support for `aws-smithy-http-client` integration tests.

pub(crate) mod client;

#[cfg(all(feature = "wire-mock", feature = "rustls-aws-lc"))]
pub(crate) mod h2;

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
pub(crate) mod tls;
