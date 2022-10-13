/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! This module defines types that describe timeouts that can be applied to various stages of the
//! Smithy networking stack.

mod config;
mod error;

pub use config::{OperationTimeoutConfig, TimeoutConfig, TimeoutConfigBuilder};
pub use error::ConfigError;
