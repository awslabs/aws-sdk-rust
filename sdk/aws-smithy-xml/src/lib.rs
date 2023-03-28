/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    // missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

//! Abstractions for Smithy
//! [XML Binding Traits](https://awslabs.github.io/smithy/1.0/spec/core/xml-traits.html)

pub mod decode;
pub mod encode;
mod escape;
mod unescape;
