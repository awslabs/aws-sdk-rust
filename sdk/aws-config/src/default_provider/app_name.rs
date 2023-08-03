/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::provider_config::ProviderConfig;
use crate::standard_property::{PropertyResolutionError, StandardProperty};
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::app_name::{AppName, InvalidAppName};

/// Default App Name Provider chain
///
/// This provider will check the following sources in order:
/// 1. Environment variables: `AWS_SDK_UA_APP_ID`
/// 2. Profile files from the key `sdk_ua_app_id`
///
#[doc = include_str!("../profile/location_of_profile_files.md")]
///
/// # Examples
///
/// **Loads "my-app" as the app name**
/// ```ini
/// [default]
/// sdk_ua_app_id = my-app
/// ```
///
/// **Loads "my-app" as the app name _if and only if_ the `AWS_PROFILE` environment variable
/// is set to `other`.**
/// ```ini
/// [profile other]
/// sdk_ua_app_id = my-app
/// ```
pub fn default_provider() -> Builder {
    Builder::default()
}

/// Default provider builder for [`AppName`]
#[derive(Debug, Default)]
pub struct Builder {
    provider_config: ProviderConfig,
}

impl Builder {
    #[doc(hidden)]
    /// Configure the default chain
    ///
    /// Exposed for overriding the environment when unit-testing providers
    pub fn configure(self, configuration: &ProviderConfig) -> Self {
        Self {
            provider_config: configuration.clone(),
        }
    }

    /// Override the profile name used by this provider
    pub fn profile_name(mut self, name: &str) -> Self {
        self.provider_config = self.provider_config.with_profile_name(name.to_string());
        self
    }

    async fn fallback_app_name(
        &self,
    ) -> Result<Option<AppName>, PropertyResolutionError<InvalidAppName>> {
        StandardProperty::new()
            .profile("sdk-ua-app-id")
            .validate(&self.provider_config, |name| AppName::new(name.to_string()))
            .await
    }

    /// Build an [`AppName`] from the default chain
    pub async fn app_name(self) -> Option<AppName> {
        let standard = StandardProperty::new()
            .env("AWS_SDK_UA_APP_ID")
            .profile("sdk_ua_app_id")
            .validate(&self.provider_config, |name| AppName::new(name.to_string()))
            .await;
        let with_fallback = match standard {
            Ok(None) => self.fallback_app_name().await,
            other => other,
        };

        with_fallback.map_err(
                |err| tracing::warn!(err = %DisplayErrorContext(&err), "invalid value for App Name setting"),
            )
            .unwrap_or(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::profile_file::{ProfileFileKind, ProfileFiles};
    use crate::provider_config::ProviderConfig;
    use crate::test_case::{no_traffic_connector, InstantSleep};
    use aws_types::os_shim_internal::{Env, Fs};

    #[tokio::test]
    async fn prefer_env_to_profile() {
        let fs = Fs::from_slice(&[("test_config", "[default]\nsdk-ua-app-id = wrong")]);
        let env = Env::from_slice(&[
            ("AWS_CONFIG_FILE", "test_config"),
            ("AWS_SDK_UA_APP_ID", "correct"),
        ]);
        let app_name = Builder::default()
            .configure(
                &ProviderConfig::no_configuration()
                    .with_fs(fs)
                    .with_env(env)
                    .with_http_connector(no_traffic_connector()),
            )
            .app_name()
            .await;

        assert_eq!(Some(AppName::new("correct").unwrap()), app_name);
    }

    // test that overriding profile_name on the root level is deprecated
    #[tokio::test]
    async fn profile_name_override() {
        let fs = Fs::from_slice(&[("test_config", "[profile custom]\nsdk_ua_app_id = correct")]);
        let conf = crate::from_env()
            .sleep_impl(InstantSleep)
            .fs(fs)
            .http_connector(no_traffic_connector())
            .profile_name("custom")
            .profile_files(
                ProfileFiles::builder()
                    .with_file(ProfileFileKind::Config, "test_config")
                    .build(),
            )
            .load()
            .await;
        assert_eq!(conf.app_name(), Some(&AppName::new("correct").unwrap()));
    }

    #[tokio::test]
    async fn load_from_profile() {
        let fs = Fs::from_slice(&[("test_config", "[default]\nsdk_ua_app_id = correct")]);
        let env = Env::from_slice(&[("AWS_CONFIG_FILE", "test_config")]);
        let app_name = Builder::default()
            .configure(
                &ProviderConfig::empty()
                    .with_fs(fs)
                    .with_env(env)
                    .with_http_connector(no_traffic_connector()),
            )
            .app_name()
            .await;

        assert_eq!(Some(AppName::new("correct").unwrap()), app_name);
    }

    #[tokio::test]
    async fn load_from_profile_old_name() {
        let fs = Fs::from_slice(&[("test_config", "[default]\nsdk-ua-app-id = correct")]);
        let env = Env::from_slice(&[("AWS_CONFIG_FILE", "test_config")]);
        let app_name = Builder::default()
            .configure(
                &ProviderConfig::empty()
                    .with_fs(fs)
                    .with_env(env)
                    .with_http_connector(no_traffic_connector()),
            )
            .app_name()
            .await;

        assert_eq!(Some(AppName::new("correct").unwrap()), app_name);
    }
}
