/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Protocol-agnostic types for smithy-rs.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
pub mod base64;
/// A typemap for storing configuration.
pub mod config_bag;
pub mod date_time;
pub mod endpoint;
pub mod error;
pub mod primitive;
pub mod retry;
pub mod timeout;

/// Utilities for type erasure.
pub mod type_erasure;

mod blob;
mod document;
mod number;

pub use blob::Blob;
pub use date_time::DateTime;
pub use document::Document;
// TODO(deprecated): Remove deprecated re-export
/// Use [error::ErrorMetadata] instead.
#[deprecated(
    note = "`aws_smithy_types::Error` has been renamed to `aws_smithy_types::error::ErrorMetadata`"
)]
pub use error::ErrorMetadata as Error;
pub use number::Number;
