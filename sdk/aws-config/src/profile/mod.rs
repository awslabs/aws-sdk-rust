/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Load configuration from AWS Profiles
//!
//! AWS profiles are typically stored in `~/.aws/config` and `~/.aws/credentials`. For more details
//! see the [`load`](parser::load) function.

mod parser;
#[doc(inline)]
pub use parser::{load, Profile, ProfileParseError, ProfileSet, Property};

pub mod app_name;
pub mod credentials;
pub mod region;
pub mod retry_config;
pub mod timeout_config;

#[doc(inline)]
pub use credentials::ProfileFileCredentialsProvider;
#[doc(inline)]
pub use region::ProfileFileRegionProvider;
