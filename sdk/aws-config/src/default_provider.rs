/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Default Provider chains for [`region`](default_provider::region), [`credentials`](default_provider::credentials),
//! [retries](default_provider::retry_config), [timeouts](default_provider::timeout_config) and [app name](default_provider::app_name).
//!
//! Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
//! if you need to set custom configuration options to override the default resolution chain.

/// Default [region](aws_types::region::Region) provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod region;

/// Default retry behavior configuration provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod retry_config;

/// Default app name provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod app_name;

/// Default timeout configuration provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod timeout_config;

/// Default credentials provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options like [`region`](credentials::Builder::region) or [`profile_name`](credentials::Builder::profile_name).
pub mod credentials;
