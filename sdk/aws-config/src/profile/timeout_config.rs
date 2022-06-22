/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load timeout configuration properties from an AWS profile

use crate::parsing::parse_str_as_timeout;
use crate::profile::Profile;
use crate::provider_config::ProviderConfig;

use aws_smithy_types::timeout;
use aws_smithy_types::tristate::TriState;
use aws_types::os_shim_internal::{Env, Fs};

use std::time::Duration;

// Currently unsupported timeouts
const PROFILE_VAR_CONNECT_TIMEOUT: &str = "connect_timeout";
const PROFILE_VAR_TLS_NEGOTIATION_TIMEOUT: &str = "tls_negotiation_timeout";
const PROFILE_VAR_READ_TIMEOUT: &str = "read_timeout";

// Supported timeouts
const PROFILE_VAR_API_CALL_ATTEMPT_TIMEOUT: &str = "api_call_attempt_timeout";
const PROFILE_VAR_API_CALL_TIMEOUT: &str = "api_call_timeout";

/// Load timeout configuration properties from a profile file
///
/// This provider will attempt to load AWS shared configuration, then read timeout configuration
/// properties from the active profile. Timeout values represent the number of seconds before timing
/// out and must be non-negative floats or integers. NaN and infinity are also invalid. If at least
/// one of these values is valid, construction will succeed.
///
/// # Examples
///
/// **Sets timeouts for the `default` profile**
/// ```ini
/// [default]
/// api_call_attempt_timeout = 2
/// api_call_timeout = 3
/// ```
///
/// **Sets the `api_call_attempt_timeout` to 0.5 seconds _if and only if_ the `other` profile is selected.**
///
/// ```ini
/// [profile other]
/// api_call_attempt_timeout = 0.5
/// ```
///
/// This provider is part of the [default timeout config provider chain](crate::default_provider::timeout_config).
#[derive(Debug, Default)]
pub struct ProfileFileTimeoutConfigProvider {
    fs: Fs,
    env: Env,
    profile_override: Option<String>,
}

/// Builder for [`ProfileFileTimeoutConfigProvider`]
#[derive(Debug, Default)]
pub struct Builder {
    config: Option<ProviderConfig>,
    profile_override: Option<String>,
}

impl Builder {
    /// Override the configuration for this provider
    pub fn configure(mut self, config: &ProviderConfig) -> Self {
        self.config = Some(config.clone());
        self
    }

    /// Override the profile name used by the [`ProfileFileTimeoutConfigProvider`]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Build a [`ProfileFileTimeoutConfigProvider`] from this builder
    pub fn build(self) -> ProfileFileTimeoutConfigProvider {
        let conf = self.config.unwrap_or_default();
        ProfileFileTimeoutConfigProvider {
            env: conf.env(),
            fs: conf.fs(),
            profile_override: self.profile_override,
        }
    }
}

impl ProfileFileTimeoutConfigProvider {
    /// Create a new [`ProfileFileTimeoutConfigProvider`]
    ///
    /// To override the selected profile, set the `AWS_PROFILE` environment variable or use the [`Builder`].
    pub fn new() -> Self {
        Self {
            fs: Fs::real(),
            env: Env::real(),
            profile_override: None,
        }
    }

    /// [`Builder`] to construct a [`ProfileFileTimeoutConfigProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Attempt to create a new [`timeout::Config`](aws_smithy_types::timeout::Config) from a profile file.
    pub async fn timeout_config(&self) -> Result<timeout::Config, timeout::ConfigError> {
        let profile = match super::parser::load(&self.fs, &self.env).await {
            Ok(profile) => profile,
            Err(err) => {
                tracing::warn!(err = %err, "failed to parse profile, skipping it");
                // return an empty builder
                return Ok(Default::default());
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
                    tracing::warn!(
                        "failed to get selected '{}' profile, skipping it",
                        selected_profile
                    );
                }
                // return an empty config
                return Ok(timeout::Config::new());
            }
        };

        // Warn users that set unsupported timeouts in their profile
        for timeout in [
            PROFILE_VAR_CONNECT_TIMEOUT,
            PROFILE_VAR_TLS_NEGOTIATION_TIMEOUT,
            PROFILE_VAR_READ_TIMEOUT,
        ] {
            warn_if_unsupported_timeout_is_set(selected_profile, timeout);
        }

        let api_call_attempt_timeout = construct_timeout_from_profile_var(
            selected_profile,
            PROFILE_VAR_API_CALL_ATTEMPT_TIMEOUT,
        )?;
        let api_call_timeout =
            construct_timeout_from_profile_var(selected_profile, PROFILE_VAR_API_CALL_TIMEOUT)?;

        let api_timeouts = timeout::Api::new()
            .with_call_timeout(api_call_timeout)
            .with_call_attempt_timeout(api_call_attempt_timeout);

        // Only API-related timeouts are currently supported
        Ok(timeout::Config::new().with_api_timeouts(api_timeouts))
    }
}

fn construct_timeout_from_profile_var(
    profile: &Profile,
    var: &'static str,
) -> Result<TriState<Duration>, timeout::ConfigError> {
    let profile_name = format!("aws profile [{}]", profile.name());
    match profile.get(var) {
        Some(timeout) => {
            parse_str_as_timeout(timeout, var.into(), profile_name.into()).map(TriState::Set)
        }
        None => Ok(TriState::Unset),
    }
}

fn warn_if_unsupported_timeout_is_set(profile: &Profile, var: &'static str) {
    if profile.get(var).is_some() {
        tracing::warn!(
                "Profile '{}' set {} timeout but that feature is currently unimplemented so the setting will be ignored. \
                To help us prioritize support for this feature, please upvote aws-sdk-rust#151 (https://github.com/awslabs/aws-sdk-rust/issues/151)",
            profile.name(),
            var
        )
    }
}
