/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    // missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]
#![allow(clippy::new_without_default)]

//! Basic types for the new smithy client orchestrator.

/// A boxed error that is `Send` and `Sync`.
pub mod box_error;

/// Smithy runtime for client orchestration.
pub mod client;

pub mod macros;
