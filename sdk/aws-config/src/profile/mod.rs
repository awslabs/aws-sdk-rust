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
pub use parser::{load, Profile, ProfileSet, Property};

pub mod credentials;
pub mod region;

#[doc(inline)]
pub use credentials::ProfileFileCredentialsProvider;
#[doc(inline)]
pub use region::ProfileFileRegionProvider;
