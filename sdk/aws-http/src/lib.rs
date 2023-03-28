/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Provides user agent and credentials middleware for the AWS SDK.

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]

/// Credentials middleware
pub mod auth;

/// Recursion Detection middleware
pub mod recursion_detection;

/// AWS-specific retry logic
pub mod retry;

/// User agent middleware
pub mod user_agent;

/// AWS-specific content-encoding tools
pub mod content_encoding;
