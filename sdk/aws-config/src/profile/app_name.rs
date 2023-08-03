/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load an app name from an AWS profile

use super::profile_file::ProfileFiles;
use crate::provider_config::ProviderConfig;
use aws_types::app_name::AppName;

/// Loads an app name from a profile file
///
/// This provider will attempt to shared AWS shared configuration and then read the
/// `sdk-ua-app-id` property from the active profile.
///
#[doc = include_str!("location_of_profile_files.md")]
///
/// # Examples
///
/// **Loads "my-app" as the app name**
/// ```ini
/// [default]
/// sdk-ua-app-id = my-app
/// ```
///
/// **Loads "my-app" as the app name _if and only if_ the `AWS_PROFILE` environment variable
/// is set to `other`.**
/// ```ini
/// [profile other]
/// sdk-ua-app-id = my-app
/// ```
///
/// This provider is part of the [default app name provider chain](crate::default_provider::app_name).
#[derive(Debug, Default)]
#[deprecated(
    note = "This is unused and is deprecated for backwards compatibility. It will be removed in a future release."
)]
pub struct ProfileFileAppNameProvider {
    provider_config: ProviderConfig,
}

#[allow(deprecated)]
impl ProfileFileAppNameProvider {
    /// Create a new [ProfileFileAppNameProvider}
    ///
    /// To override the selected profile, set the `AWS_PROFILE` environment variable or use the [`Builder`].
    pub fn new() -> Self {
        Self {
            provider_config: ProviderConfig::default(),
        }
    }

    /// [`Builder`] to construct a [`ProfileFileAppNameProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Parses the profile config and attempts to find an app name.
    pub async fn app_name(&self) -> Option<AppName> {
        let app_id = self.provider_config.profile().await?.get("sdk-ua-app-id")?;
        match AppName::new(app_id.to_owned()) {
            Ok(app_name) => Some(app_name),
            Err(err) => {
                tracing::warn!(err = %err, "`sdk-ua-app-id` property `{}` was invalid", app_id);
                None
            }
        }
    }
}

/// Builder for [ProfileFileAppNameProvider]
#[derive(Debug, Default)]
#[allow(deprecated)]
pub struct Builder {
    config: Option<ProviderConfig>,
    profile_override: Option<String>,
    profile_files: Option<ProfileFiles>,
}

#[allow(deprecated)]
impl Builder {
    /// Override the configuration for this provider
    pub fn configure(mut self, config: &ProviderConfig) -> Self {
        self.config = Some(config.clone());
        self
    }

    /// Override the profile name used by the [ProfileFileAppNameProvider]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Build a [ProfileFileAppNameProvider] from this builder
    #[allow(deprecated)]
    pub fn build(self) -> ProfileFileAppNameProvider {
        let conf = self
            .config
            .unwrap_or_default()
            .with_profile_config(self.profile_files, self.profile_override);
        ProfileFileAppNameProvider {
            provider_config: conf,
        }
    }
}
