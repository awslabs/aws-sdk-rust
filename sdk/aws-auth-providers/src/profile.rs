/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Profile File Based Providers
//!
//! Profile file based providers combine two pieces:
//!
//! 1. Parsing and resolution of the assume role chain
//! 2. A user-modifiable hashmap of provider name to provider.
//!
//! Profile file based providers first determine the chain of providers that will be used to load
//! credentials. After determining and validating this chain, a `Vec` of providers will be created.
//!
//! Each subsequent provider will provide boostrap providers to the next provider in order to load
//! the final credentials.
//!
//! This module contains two sub modules:
//! - `repr` which contains an abstract representation of a provider chain and the logic to
//! build it from `~/.aws/credentials` and `~/.aws/config`.
//! - `exec` which contains a chain representation of providers to implement passing bootstrapped credentials
//! through a series of providers.
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};

use aws_types::os_shim_internal::{Env, Fs};
use aws_types::profile::ProfileParseError;

mod exec;
mod repr;

#[doc(hidden)]
pub fn load_profile() -> Result<(), Box<dyn Error>> {
    // remove the non-usage warnings until we're actually using this module
    let _ = repr::resolve_chain(&aws_types::profile::load(&Fs::real(), &Env::real())?)?;
    Ok(())
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ProfileFileError {
    CouldNotParseProfile(ProfileParseError),
    CredentialLoop {
        profiles: Vec<String>,
        next: String,
    },
    MissingCredentialSource {
        profile: String,
        message: Cow<'static, str>,
    },
    InvalidCredentialSource {
        profile: String,
        message: Cow<'static, str>,
    },
    MissingProfile {
        profile: String,
        message: Cow<'static, str>,
    },
}

impl Display for ProfileFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => {
                write!(f, "could not parse profile file: {}", err)
            }
            ProfileFileError::CredentialLoop { profiles, next } => write!(
                f,
                "profile formed an infinite loop. first we loaded {:?}, \
            then attempted to reload {}",
                profiles, next
            ),
            ProfileFileError::MissingCredentialSource { profile, message } => {
                write!(f, "missing credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::InvalidCredentialSource { profile, message } => {
                write!(f, "invalid credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::MissingProfile { profile, message } => {
                write!(f, "profile `{}` was not defined: {}", profile, message)
            }
        }
    }
}

impl Error for ProfileFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => Some(err),
            _ => None,
        }
    }
}
