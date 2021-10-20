/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Future utilities and runtime-agnostic abstractions for smithy-rs.
//!
//! Async runtime specific code is abstracted behind async traits, and implementations are
//! provided via feature flag. For now, only Tokio runtime implementations are provided.

pub mod future;
pub mod rt;
