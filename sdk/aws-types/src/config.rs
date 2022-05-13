/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![deny(missing_docs)]

//! AWS Shared Config _(deprecated, replaced with [`sdk_config`](crate::sdk_config))_
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

#[deprecated(since = "0.9.0", note = "renamed to crate::SdkConfig")]
/// AWS Shared Configuration
pub type Config = super::SdkConfig;
