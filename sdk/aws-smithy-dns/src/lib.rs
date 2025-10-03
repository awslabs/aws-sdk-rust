/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
/* End of automatically managed default lints */

//! Built-in DNS resolver implementations for smithy-rs clients.

#[cfg(all(feature = "hickory-dns", not(target_family = "wasm")))]
pub mod hickory;

#[cfg(all(feature = "hickory-dns", not(target_family = "wasm")))]
pub use hickory::HickoryDnsResolver;
