/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};

#[non_exhaustive]
#[derive(Debug)]
/// An error that occurs during construction of a `timeout::Config`
pub enum ConfigError {
    /// A timeout value was set to an invalid value:
    /// - Any number less than 0
    /// - Infinity or negative infinity
    /// - `NaN`
    InvalidTimeout {
        /// The name of the invalid value
        name: Cow<'static, str>,
        /// The reason that why the timeout was considered invalid
        reason: Cow<'static, str>,
        /// Where the invalid value originated from
        set_by: Cow<'static, str>,
    },
    /// The timeout value couln't be parsed as an `f32`
    ParseError {
        /// The name of the invalid value
        name: Cow<'static, str>,
        /// Where the invalid value originated from
        set_by: Cow<'static, str>,
        /// The source of this error
        source: Box<dyn std::error::Error>,
    },
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ConfigError::*;
        match self {
            InvalidTimeout {
                name,
                set_by,
                reason,
            } => {
                write!(
                    f,
                    "invalid timeout '{}' set by {} is invalid: {}",
                    name, set_by, reason
                )
            }
            ParseError {
                name,
                set_by,
                source,
            } => {
                write!(
                    f,
                    "timeout '{}' set by {} could not be parsed as an f32: {}",
                    name, set_by, source
                )
            }
        }
    }
}
