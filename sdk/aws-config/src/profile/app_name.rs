/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load an app name from an AWS profile

use crate::provider_config::ProviderConfig;
use aws_types::app_name::AppName;
use aws_types::os_shim_internal::{Env, Fs};

/// Loads an app name from a profile file
///
/// This provider will attempt to shared AWS shared configuration and then read the
/// `sdk-ua-app-id` property from the active profile.
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
pub struct ProfileFileAppNameProvider {
    fs: Fs,
    env: Env,
    profile_override: Option<String>,
}

impl ProfileFileAppNameProvider {
    /// Create a new [ProfileFileAppNameProvider}
    ///
    /// To override the selected profile, set the `AWS_PROFILE` environment variable or use the [`Builder`].
    pub fn new() -> Self {
        Self {
            fs: Fs::real(),
            env: Env::real(),
            profile_override: None,
        }
    }

    /// [`Builder`] to construct a [`ProfileFileAppNameProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Parses the profile config and attempts to find an app name.
    pub async fn app_name(&self) -> Option<AppName> {
        let profile = super::parser::load(&self.fs, &self.env)
            .await
            .map_err(|err| tracing::warn!(err = %err, "failed to parse profile"))
            .ok()?;
        let selected_profile_name = self
            .profile_override
            .as_deref()
            .unwrap_or_else(|| profile.selected_profile());
        let selected_profile = profile.get_profile(selected_profile_name)?;
        selected_profile
            .get("sdk-ua-app-id")
            .map(|name| match AppName::new(name.to_owned()) {
                Ok(app_name) => Some(app_name),
                Err(err) => {
                    tracing::warn!(err = %err, "`sdk-ua-app-id` property in profile `{}` was invalid", selected_profile_name);
                    None
                }
            })
            .flatten()
    }
}

/// Builder for [ProfileFileAppNameProvider]
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

    /// Override the profile name used by the [ProfileFileAppNameProvider]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Build a [ProfileFileAppNameProvider] from this builder
    pub fn build(self) -> ProfileFileAppNameProvider {
        let conf = self.config.unwrap_or_default();
        ProfileFileAppNameProvider {
            env: conf.env(),
            fs: conf.fs(),
            profile_override: self.profile_override,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProfileFileAppNameProvider;
    use crate::provider_config::ProviderConfig;
    use crate::test_case::no_traffic_connector;
    use aws_sdk_sts::AppName;
    use aws_types::os_shim_internal::{Env, Fs};
    use tracing_test::traced_test;

    fn provider_config(config_contents: &str) -> ProviderConfig {
        let fs = Fs::from_slice(&[("test_config", config_contents)]);
        let env = Env::from_slice(&[("AWS_CONFIG_FILE", "test_config")]);
        ProviderConfig::empty()
            .with_fs(fs)
            .with_env(env)
            .with_http_connector(no_traffic_connector())
    }

    fn default_provider(config_contents: &str) -> ProfileFileAppNameProvider {
        ProfileFileAppNameProvider::builder()
            .configure(&provider_config(config_contents))
            .build()
    }

    #[tokio::test]
    async fn no_app_name() {
        assert_eq!(None, default_provider("[default]\n").app_name().await);
    }

    #[tokio::test]
    async fn app_name_default_profile() {
        assert_eq!(
            Some(AppName::new("test").unwrap()),
            default_provider("[default]\nsdk-ua-app-id = test")
                .app_name()
                .await
        );
    }

    #[tokio::test]
    async fn app_name_other_profiles() {
        let config = "\
            [default]\n\
            sdk-ua-app-id = test\n\
            \n\
            [profile other]\n\
            sdk-ua-app-id = bar\n
        ";
        assert_eq!(
            Some(AppName::new("bar").unwrap()),
            ProfileFileAppNameProvider::builder()
                .profile_name("other")
                .configure(&provider_config(config))
                .build()
                .app_name()
                .await
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn invalid_app_name() {
        assert_eq!(
            None,
            default_provider("[default]\nsdk-ua-app-id = definitely invalid")
                .app_name()
                .await
        );
        assert!(logs_contain(
            "`sdk-ua-app-id` property in profile `default` was invalid"
        ));
    }
}
