/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_smithy_types::retry::RetryConfig;

use crate::environment::retry_config::EnvironmentVariableRetryConfigProvider;
use crate::profile;
use crate::provider_config::ProviderConfig;

/// Default RetryConfig Provider chain
///
/// Unlike other "providers" `RetryConfig` has no related `RetryConfigProvider` trait. Instead,
/// a builder struct is returned which has a similar API.
///
/// This provider will check the following sources in order:
/// 1. [Environment variables](EnvironmentVariableRetryConfigProvider)
/// 2. [Profile file](crate::profile::retry_config::ProfileFileRetryConfigProvider)
///
/// # Example
///
/// When running [`aws_config::from_env()`](crate::from_env()), a [`ConfigLoader`](crate::ConfigLoader)
/// is created that will then create a [`RetryConfig`] from the default_provider. There is no
/// need to call `default_provider` and the example below is only for illustration purposes.
///
/// ```no_run
/// # use std::error::Error;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn Error>> {
/// use aws_config::default_provider::retry_config;
///
/// // Load a retry config from a specific profile
/// let retry_config = retry_config::default_provider()
///     .profile_name("other_profile")
///     .retry_config()
///     .await;
/// let config = aws_config::from_env()
///     // Override the retry config set by the default profile
///     .retry_config(retry_config)
///     .load()
///     .await;
/// // instantiate a service client:
/// // <my_aws_service>::Client::new(&config);
/// #     Ok(())
/// # }
/// ```
pub fn default_provider() -> Builder {
    Builder::default()
}

/// Builder for RetryConfig that checks the environment and aws profile for configuration
#[derive(Default)]
pub struct Builder {
    env_provider: EnvironmentVariableRetryConfigProvider,
    profile_file: profile::retry_config::Builder,
}

impl Builder {
    /// Configure the default chain
    ///
    /// Exposed for overriding the environment when unit-testing providers
    pub fn configure(mut self, configuration: &ProviderConfig) -> Self {
        self.env_provider =
            EnvironmentVariableRetryConfigProvider::new_with_env(configuration.env());
        self.profile_file = self.profile_file.configure(configuration);
        self
    }

    /// Override the profile name used by this provider
    pub fn profile_name(mut self, name: &str) -> Self {
        self.profile_file = self.profile_file.profile_name(name);
        self
    }

    /// Attempt to create a [RetryConfig](aws_smithy_types::retry::RetryConfig) from following sources in order:
    /// 1. [Environment variables](crate::environment::retry_config::EnvironmentVariableRetryConfigProvider)
    /// 2. [Profile file](crate::profile::retry_config::ProfileFileRetryConfigProvider)
    /// 3. [RetryConfig::default()](aws_smithy_types::retry::RetryConfig::default)
    ///
    /// Precedence is considered on a per-field basis
    ///
    /// # Panics
    ///
    /// - Panics if the `AWS_MAX_ATTEMPTS` env var or `max_attempts` profile var is set to 0
    /// - Panics if the `AWS_RETRY_MODE` env var or `retry_mode` profile var is set to "adaptive" (it's not yet supported)
    pub async fn retry_config(self) -> RetryConfig {
        // Both of these can return errors due to invalid config settings and we want to surface those as early as possible
        // hence, we'll panic if any config values are invalid (missing values are OK though)
        // We match this instead of unwrapping so we can print the error with the `Display` impl instead of the `Debug` impl that unwrap uses
        let builder_from_env = match self.env_provider.retry_config_builder() {
            Ok(retry_config_builder) => retry_config_builder,
            Err(err) => panic!("{}", err),
        };
        let builder_from_profile = match self.profile_file.build().retry_config_builder().await {
            Ok(retry_config_builder) => retry_config_builder,
            Err(err) => panic!("{}", err),
        };

        builder_from_env
            .take_unset_from(builder_from_profile)
            .build()
    }
}
