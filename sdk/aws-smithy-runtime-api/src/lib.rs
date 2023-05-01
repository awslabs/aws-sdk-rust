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

//! Basic types for the new smithy client orchestrator.

/// Smithy runtime for client orchestration.
pub mod client;

/// A typemap for storing configuration.
pub mod config_bag;

/// Utilities for type erasure.
pub mod type_erasure;
