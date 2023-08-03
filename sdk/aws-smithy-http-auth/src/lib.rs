/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(enableNewSmithyRuntimeCleanup): The contents of this crate are moving into aws-smithy-runtime.
// This crate is kept to continue sorting the middleware implementation until it is removed.
// When removing the old implementation, clear out this crate and deprecate it.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

//! Smithy HTTP Auth Types

pub mod api_key;
pub mod definition;
pub mod error;
pub mod location;
