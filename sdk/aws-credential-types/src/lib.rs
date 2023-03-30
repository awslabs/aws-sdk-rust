/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! `aws-credential-types` provides types concerned with AWS SDK credentials including:
//! * Traits for credentials providers and for credentials caching
//! * An opaque struct representing credentials
//! * Concrete implementations of credentials caching

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rustdoc::missing_crate_level_docs,
    unreachable_pub
)]

pub mod cache;
pub mod credential_fn;
mod credentials_impl;
pub mod provider;
#[doc(hidden)]
pub mod time_source;

pub use credentials_impl::Credentials;
