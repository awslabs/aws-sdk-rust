/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::profile::ProfileParseError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

/// Failed to read or parse the profile file(s)
#[derive(Debug, Clone)]
pub enum ProfileFileLoadError {
    /// The profile could not be parsed
    #[non_exhaustive]
    ParseError(ProfileParseError),

    /// Attempt to read the AWS config file (`~/.aws/config` by default) failed with a filesystem error.
    #[non_exhaustive]
    CouldNotReadFile(CouldNotReadProfileFile),
}

impl Display for ProfileFileLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileLoadError::ParseError(_err) => {
                write!(f, "could not parse profile file")
            }
            ProfileFileLoadError::CouldNotReadFile(err) => {
                write!(f, "could not read file `{}`", err.path.display())
            }
        }
    }
}

impl Error for ProfileFileLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileLoadError::ParseError(err) => Some(err),
            ProfileFileLoadError::CouldNotReadFile(details) => Some(&details.cause),
        }
    }
}

impl From<ProfileParseError> for ProfileFileLoadError {
    fn from(err: ProfileParseError) -> Self {
        ProfileFileLoadError::ParseError(err)
    }
}

/// An error encountered while reading the AWS config file
#[derive(Debug, Clone)]
pub struct CouldNotReadProfileFile {
    pub(crate) path: PathBuf,
    pub(crate) cause: Arc<std::io::Error>,
}
