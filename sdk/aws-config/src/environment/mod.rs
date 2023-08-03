/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Providers that load configuration from environment variables

/// Load app name from the environment
pub mod app_name;

use std::error::Error;
use std::fmt::{Display, Formatter};

/// Load credentials from the environment
pub mod credentials;
pub use credentials::EnvironmentVariableCredentialsProvider;

/// Load regions from the environment
pub mod region;
pub use region::EnvironmentVariableRegionProvider;

#[derive(Debug)]
pub(crate) struct InvalidBooleanValue {
    value: String,
}

impl Display for InvalidBooleanValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} was not a valid boolean", self.value)
    }
}

impl Error for InvalidBooleanValue {}

pub(crate) fn parse_bool(value: &str) -> Result<bool, InvalidBooleanValue> {
    if value.eq_ignore_ascii_case("false") {
        Ok(false)
    } else if value.eq_ignore_ascii_case("true") {
        Ok(true)
    } else {
        Err(InvalidBooleanValue {
            value: value.to_string(),
        })
    }
}
