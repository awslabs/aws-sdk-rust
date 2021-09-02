/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! AWS Signature Authentication Package
//!
//! In the future, additional signature algorithms can be enabled as Cargo Features.

#[cfg(feature = "sign-eventstream")]
pub mod event_stream;

pub mod middleware;
pub mod signer;
