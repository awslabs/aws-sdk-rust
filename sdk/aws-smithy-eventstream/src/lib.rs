/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_debug_implementations,
    /*missing_docs,
    rustdoc::all,*/
    rust_2018_idioms
)]

//! AWS Event Stream frame serialization/deserialization implementation.

mod buf;
pub mod error;
pub mod frame;
pub mod smithy;
pub mod str_bytes;
