/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load retry configuration properties from an AWS profile

use crate::profile::profile_file::ProfileFiles;
use crate::provider_config::ProviderConfig;
use crate::retry::{
    error::RetryConfigError, error::RetryConfigErrorKind, RetryConfigBuilder, RetryMode,
};
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::os_shim_internal::{Env, Fs};
use std::str::FromStr;

/// Load retry configuration properties from a profile file
///
/// This provider will attempt to load AWS shared configuration, then read retry configuration properties
/// from the active profile.
///
#[doc = include_str!("location_of_profile_files.md")]
///
/// # Examples
///
/// **Loads 2 as the `max_attempts` to make when sending a request**
/// ```ini
/// [default]
/// max_attempts = 2
/// ```
///
/// **Loads `standard` as the `retry_mode` _if and only if_ the `other` profile is selected.**
///
/// ```ini
/// [profile other]
/// retry_mode = standard
/// ```
///
/// This provider is part of the [default retry_config provider chain](crate::default_provider::retry_config).
#[derive(Debug, Default)]
pub struct ProfileFileRetryConfigProvider {
    fs: Fs,
    env: Env,
    profile_override: Option<String>,
    profile_files: ProfileFiles,
}

/// Builder for [ProfileFileRetryConfigProvider]
#[derive(Debug, Default)]
pub struct Builder {
    config: Option<ProviderConfig>,
    profile_override: Option<String>,
    profile_files: Option<ProfileFiles>,
}

impl Builder {
    /// Override the configuration for this provider
    pub fn configure(mut self, config: &ProviderConfig) -> Self {
        self.config = Some(config.clone());
        self
    }

    /// Override the profile name used by the [ProfileFileRetryConfigProvider]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Set the profile file that should be used by the [`ProfileFileRetryConfigProvider`]
    pub fn profile_files(mut self, profile_files: ProfileFiles) -> Self {
        self.profile_files = Some(profile_files);
        self
    }

    /// Build a [ProfileFileRetryConfigProvider] from this builder
    pub fn build(self) -> ProfileFileRetryConfigProvider {
        let conf = self.config.unwrap_or_default();
        ProfileFileRetryConfigProvider {
            env: conf.env(),
            fs: conf.fs(),
            profile_override: self.profile_override,
            profile_files: self.profile_files.unwrap_or_default(),
        }
    }
}

impl ProfileFileRetryConfigProvider {
    /// Create a new [ProfileFileRetryConfigProvider]
    ///
    /// To override the selected profile, set the `AWS_PROFILE` environment variable or use the [Builder].
    pub fn new() -> Self {
        Self {
            fs: Fs::real(),
            env: Env::real(),
            profile_override: None,
            profile_files: Default::default(),
        }
    }

    /// [Builder] to construct a [ProfileFileRetryConfigProvider]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Attempt to create a new RetryConfigBuilder from a profile file.
    pub async fn retry_config_builder(&self) -> Result<RetryConfigBuilder, RetryConfigError> {
        let profile = match super::parser::load(&self.fs, &self.env, &self.profile_files).await {
            Ok(profile) => profile,
            Err(err) => {
                tracing::warn!(err = %DisplayErrorContext(&err), "failed to parse profile");
                // return an empty builder
                return Ok(RetryConfigBuilder::new());
            }
        };

        let selected_profile = self
            .profile_override
            .as_deref()
            .unwrap_or_else(|| profile.selected_profile());
        let selected_profile = match profile.get_profile(selected_profile) {
            Some(profile) => profile,
            None => {
                // Only warn if the user specified a profile name to use.
                if self.profile_override.is_some() {
                    tracing::warn!("failed to get selected '{}' profile", selected_profile);
                }
                // return an empty builder
                return Ok(RetryConfigBuilder::new());
            }
        };

        let max_attempts = match selected_profile.get("max_attempts") {
            Some(max_attempts) => match max_attempts.parse::<u32>() {
                Ok(max_attempts) if max_attempts == 0 => {
                    return Err(RetryConfigErrorKind::MaxAttemptsMustNotBeZero {
                        set_by: "aws profile".into(),
                    }
                    .into());
                }
                Ok(max_attempts) => Some(max_attempts),
                Err(source) => {
                    return Err(RetryConfigErrorKind::FailedToParseMaxAttempts {
                        set_by: "aws profile".into(),
                        source,
                    }
                    .into());
                }
            },
            None => None,
        };

        let retry_mode = match selected_profile.get("retry_mode") {
            Some(retry_mode) => match RetryMode::from_str(retry_mode) {
                Ok(retry_mode) => Some(retry_mode),
                Err(retry_mode_err) => {
                    return Err(RetryConfigErrorKind::InvalidRetryMode {
                        set_by: "aws profile".into(),
                        source: retry_mode_err,
                    }
                    .into());
                }
            },
            None => None,
        };

        let mut retry_config_builder = RetryConfigBuilder::new();
        retry_config_builder
            .set_max_attempts(max_attempts)
            .set_mode(retry_mode);

        Ok(retry_config_builder)
    }
}
