/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::environment::app_name::EnvironmentVariableAppNameProvider;
use crate::profile::app_name;
use crate::provider_config::ProviderConfig;
use aws_types::app_name::AppName;

/// Default App Name Provider chain
///
/// This provider will check the following sources in order:
/// 1. [Environment variables](EnvironmentVariableAppNameProvider)
/// 2. [Profile file](crate::profile::app_name::ProfileFileAppNameProvider)
pub fn default_provider() -> Builder {
    Builder::default()
}

/// Default provider builder for [`AppName`]
#[derive(Default)]
pub struct Builder {
    env_provider: EnvironmentVariableAppNameProvider,
    profile_file: app_name::Builder,
}

impl Builder {
    #[doc(hidden)]
    /// Configure the default chain
    ///
    /// Exposed for overriding the environment when unit-testing providers
    pub fn configure(mut self, configuration: &ProviderConfig) -> Self {
        self.env_provider = EnvironmentVariableAppNameProvider::new_with_env(configuration.env());
        self.profile_file = self.profile_file.configure(configuration);
        self
    }

    /// Override the profile name used by this provider
    pub fn profile_name(mut self, name: &str) -> Self {
        self.profile_file = self.profile_file.profile_name(name);
        self
    }

    /// Build an [`AppName`] from the default chain
    pub async fn app_name(self) -> Option<AppName> {
        self.env_provider
            .app_name()
            .or(self.profile_file.build().app_name().await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider_config::ProviderConfig;
    use crate::test_case::no_traffic_connector;
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

    #[tokio::test]
    async fn load_from_profile() {
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
