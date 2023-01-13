/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load configuration from AWS Profiles
//!
//! AWS profiles are typically stored in `~/.aws/config` and `~/.aws/credentials`. For more details
//! see the [`load`](parser::load) function.

mod parser;

// This can't be included in the other `pub use` statement until
// https://github.com/rust-lang/rust/pull/87487 is fixed by upgrading
// to Rust 1.60
#[doc(inline)]
pub use parser::ProfileParseError;
#[doc(inline)]
pub use parser::{load, Profile, ProfileFileLoadError, ProfileSet, Property};

pub mod app_name;
pub mod credentials;
pub mod profile_file;
pub mod region;

#[doc(inline)]
pub use credentials::ProfileFileCredentialsProvider;
#[doc(inline)]
pub use region::ProfileFileRegionProvider;
