/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */
//! Conversions between `aws-smithy-types` and the types of frequently used Rust libraries.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

#[cfg(any(feature = "convert-time", feature = "convert-chrono"))]
pub mod date_time;

#[cfg(feature = "convert-streams")]
pub mod stream;
