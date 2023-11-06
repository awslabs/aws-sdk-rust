/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Functions to create signing keys and calculate signatures.

/// Support for Sigv4 signing
pub mod v4;

/// Support for Sigv4a signing
#[cfg(feature = "sigv4a")]
pub mod v4a;
