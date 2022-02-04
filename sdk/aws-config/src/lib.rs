/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
#![warn(missing_docs)]

//! `aws-config` provides implementations of region, credential resolution.
//!
//! These implementations can be used either via the default chain implementation
//! [`from_env`]/[`ConfigLoader`] or ad-hoc individual credential and region providers.
//!
//! [`ConfigLoader`](ConfigLoader) can combine different configuration sources into an AWS shared-config:
//! [`Config`](aws_types::config::Config). [`Config`](aws_types::config::Config) can be used configure
//! an AWS service client.
//!
//! # Examples
//!
//! Load default SDK configuration:
//! ```no_run
//! # mod aws_sdk_dynamodb {
//! #   pub struct Client;
//! #   impl Client {
//! #     pub fn new(config: &aws_types::config::Config) -> Self { Client }
//! #   }
//! # }
//! # async fn docs() {
//! let config = aws_config::load_from_env().await;
//! let client = aws_sdk_dynamodb::Client::new(&config);
//! # }
//! ```
//!
//! Load SDK configuration with a region override:
//! ```no_run
//! # mod aws_sdk_dynamodb {
//! #   pub struct Client;
//! #   impl Client {
//! #     pub fn new(config: &aws_types::config::Config) -> Self { Client }
//! #   }
//! # }
//! # async fn docs() {
//! # use aws_config::meta::region::RegionProviderChain;
//! let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
//! let config = aws_config::from_env().region(region_provider).load().await;
//! let client = aws_sdk_dynamodb::Client::new(&config);
//! # }
//! ```

#[allow(dead_code)]
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Providers that implement the default AWS provider chain
pub mod default_provider;

/// Providers that load configuration from environment variables
pub mod environment;

/// Meta-providers that augment existing providers with new behavior
pub mod meta;

pub mod profile;

pub mod sts;

#[cfg(test)]
mod test_case;

pub mod web_identity_token;

pub mod ecs;

pub mod provider_config;

mod cache;

pub mod imds;

mod json_credentials;

mod fs_util;

mod http_credential_provider;

pub mod sso;

pub mod connector;

// Re-export types from smithy-types
pub use aws_smithy_types::retry::RetryConfig;
pub use aws_smithy_types::timeout::TimeoutConfig;

// Re-export types from aws-types
pub use aws_types::app_name::{AppName, InvalidAppName};
pub use aws_types::config::Config;

/// Create an environment loader for AWS Configuration
///
/// # Examples
/// ```no_run
/// # async fn create_config() {
/// use aws_types::region::Region;
/// let config = aws_config::from_env().region("us-east-1").load().await;
/// # }
/// ```
pub fn from_env() -> ConfigLoader {
    ConfigLoader::default()
}

/// Load a default configuration from the environment
///
/// Convenience wrapper equivalent to `aws_config::from_env().load().await`
pub async fn load_from_env() -> aws_types::config::Config {
    from_env().load().await
}

/// Load default sources for all configuration with override support
pub use loader::ConfigLoader;

mod loader {
    use std::sync::Arc;

    use crate::connector::default_connector;
    use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
    use aws_smithy_client::http_connector::{HttpConnector, HttpSettings};
    use aws_smithy_types::retry::RetryConfig;
    use aws_smithy_types::timeout::TimeoutConfig;
    use aws_types::app_name::AppName;
    use aws_types::config::Config;
    use aws_types::credentials::{ProvideCredentials, SharedCredentialsProvider};

    use crate::default_provider::{app_name, credentials, region, retry_config, timeout_config};
    use crate::meta::region::ProvideRegion;
    use crate::provider_config::ProviderConfig;

    /// Load a cross-service [`Config`](aws_types::config::Config) from the environment
    ///
    /// This builder supports overriding individual components of the generated config. Overriding a component
    /// will skip the standard resolution chain from **for that component**. For example,
    /// if you override the region provider, _even if that provider returns None_, the default region provider
    /// chain will not be used.
    #[derive(Default, Debug)]
    pub struct ConfigLoader {
        app_name: Option<AppName>,
        credentials_provider: Option<SharedCredentialsProvider>,
        region: Option<Box<dyn ProvideRegion>>,
        retry_config: Option<RetryConfig>,
        sleep: Option<Arc<dyn AsyncSleep>>,
        timeout_config: Option<TimeoutConfig>,
        provider_config: Option<ProviderConfig>,
        http_connector: Option<HttpConnector>,
    }

    impl ConfigLoader {
        /// Override the region used to build [`Config`](aws_types::config::Config).
        ///
        /// # Examples
        /// ```no_run
        /// # async fn create_config() {
        /// use aws_types::region::Region;
        /// let config = aws_config::from_env()
        ///     .region(Region::new("us-east-1"))
        ///     .load().await;
        /// # }
        /// ```
        pub fn region(mut self, region: impl ProvideRegion + 'static) -> Self {
            self.region = Some(Box::new(region));
            self
        }

        /// Override the retry_config used to build [`Config`](aws_types::config::Config).
        ///
        /// # Examples
        /// ```no_run
        /// # use aws_smithy_types::retry::RetryConfig;
        /// # async fn create_config() {
        ///     let config = aws_config::from_env()
        ///         .retry_config(RetryConfig::new().with_max_attempts(2))
        ///         .load().await;
        /// # }
        /// ```
        pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
            self.retry_config = Some(retry_config);
            self
        }

        /// Override the timeout config used to build [`Config`](aws_types::config::Config).
        /// **Note: This only sets timeouts for calls to AWS services.** Timeouts for the credentials
        /// provider chain are configured separately.
        ///
        /// # Examples
        /// ```no_run
        /// # use std::time::Duration;
        /// # use aws_smithy_types::timeout::TimeoutConfig;
        /// # async fn create_config() {
        ///  let timeout_config = TimeoutConfig::new().with_api_call_timeout(Some(Duration::from_secs(1)));
        ///  let config = aws_config::from_env()
        ///     .timeout_config(timeout_config)
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn timeout_config(mut self, timeout_config: TimeoutConfig) -> Self {
            self.timeout_config = Some(timeout_config);
            self
        }

        /// Override the sleep implementation for this [`ConfigLoader`]. The sleep implementation
        /// is used to create timeout futures.
        pub fn sleep_impl(mut self, sleep: impl AsyncSleep + 'static) -> Self {
            // it's possible that we could wrapping an `Arc in an `Arc` and that's OK
            self.sleep = Some(Arc::new(sleep));
            self
        }

        /// Override the [`HttpConnector`] used to build [`Config`](aws_types::config::Config).
        pub fn http_connector(mut self, http_connector: HttpConnector) -> Self {
            self.http_connector = Some(http_connector);
            self
        }

        /// Override the credentials provider used to build [`Config`](aws_types::config::Config).
        ///
        /// # Examples
        ///
        /// Override the credentials provider but load the default value for region:
        /// ```no_run
        /// # use aws_types::Credentials;
        /// # fn create_my_credential_provider() -> Credentials {
        /// #     Credentials::new("example", "example", None, None, "example")
        /// # }
        /// # async fn create_config() {
        /// let config = aws_config::from_env()
        ///     .credentials_provider(create_my_credential_provider())
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn credentials_provider(
            mut self,
            credentials_provider: impl ProvideCredentials + 'static,
        ) -> Self {
            self.credentials_provider = Some(SharedCredentialsProvider::new(credentials_provider));
            self
        }

        /// Set configuration for all sub-loaders (credentials, region etc.)
        ///
        /// Update the `ProviderConfig` used for all nested loaders. This can be used to override
        /// the HTTPs connector used or to stub in an in memory `Env` or `Fs` for testing.
        ///
        /// # Examples
        /// ```no_run
        /// # async fn docs() {
        /// use aws_config::provider_config::ProviderConfig;
        /// let custom_https_connector = hyper_rustls::HttpsConnectorBuilder::new().
        ///     with_webpki_roots()
        ///     .https_only()
        ///     .enable_http1()
        ///     .build();
        /// let provider_config = ProviderConfig::default().with_tcp_connector(custom_https_connector);
        /// let shared_config = aws_config::from_env().configure(provider_config).load().await;
        /// # }
        /// ```
        pub fn configure(mut self, provider_config: ProviderConfig) -> Self {
            self.provider_config = Some(provider_config);
            self
        }

        /// Load the default configuration chain
        ///
        /// If fields have been overridden during builder construction, the override values will be used.
        ///
        /// Otherwise, the default values for each field will be provided.
        ///
        /// NOTE: When an override is provided, the default implementation is **not** used as a fallback.
        /// This means that if you provide a region provider that does not return a region, no region will
        /// be set in the resulting [`Config`](aws_types::config::Config)
        pub async fn load(self) -> aws_types::config::Config {
            let conf = self.provider_config.unwrap_or_default();
            let region = if let Some(provider) = self.region {
                provider.region().await
            } else {
                region::Builder::default()
                    .configure(&conf)
                    .build()
                    .region()
                    .await
            };

            let retry_config = if let Some(retry_config) = self.retry_config {
                retry_config
            } else {
                retry_config::default_provider()
                    .configure(&conf)
                    .retry_config()
                    .await
            };

            let app_name = if self.app_name.is_some() {
                self.app_name
            } else {
                app_name::default_provider()
                    .configure(&conf)
                    .app_name()
                    .await
            };

            let timeout_config = if let Some(timeout_config) = self.timeout_config {
                timeout_config
            } else {
                timeout_config::default_provider()
                    .configure(&conf)
                    .timeout_config()
                    .await
            };

            let sleep_impl = if self.sleep.is_none() {
                if default_async_sleep().is_none() {
                    tracing::warn!(
                        "An implementation of AsyncSleep was requested by calling default_async_sleep \
                         but no default was set.
                         This happened when ConfigLoader::load was called during Config construction. \
                         You can fix this by setting a sleep_impl on the ConfigLoader before calling \
                         load or by enabling the rt-tokio feature"
                    );
                }
                default_async_sleep()
            } else {
                self.sleep
            };

            let http_connector: HttpConnector = if let Some(http_connector) = self.http_connector {
                http_connector
            } else {
                let settings = HttpSettings::default().with_timeout_config(timeout_config.clone());
                let sleep_impl = sleep_impl.clone();
                HttpConnector::Prebuilt(default_connector(&settings, sleep_impl))
            };

            let credentials_provider = if let Some(provider) = self.credentials_provider {
                provider
            } else {
                let mut builder = credentials::DefaultCredentialsChain::builder().configure(conf);
                builder.set_region(region.clone());
                SharedCredentialsProvider::new(builder.build().await)
            };

            let mut builder = Config::builder()
                .region(region)
                .retry_config(retry_config)
                .timeout_config(timeout_config)
                .credentials_provider(credentials_provider)
                .http_connector(http_connector);

            builder.set_app_name(app_name);
            builder.set_sleep_impl(sleep_impl);
            builder.build()
        }
    }

    #[cfg(test)]
    mod test {
        use crate::from_env;
        use crate::provider_config::ProviderConfig;
        use aws_smithy_client::erase::DynConnector;
        use aws_smithy_client::never::NeverConnector;
        use aws_types::credentials::ProvideCredentials;
        use aws_types::os_shim_internal::Env;

        #[tokio::test]
        async fn provider_config_used() {
            let env = Env::from_slice(&[
                ("AWS_MAX_ATTEMPTS", "10"),
                ("AWS_REGION", "us-west-4"),
                ("AWS_ACCESS_KEY_ID", "akid"),
                ("AWS_SECRET_ACCESS_KEY", "secret"),
            ]);
            let loader = from_env()
                .configure(
                    ProviderConfig::empty()
                        .with_env(env)
                        .with_http_connector(DynConnector::new(NeverConnector::new())),
                )
                .load()
                .await;
            assert_eq!(loader.retry_config().unwrap().max_attempts(), 10);
            assert_eq!(loader.region().unwrap().as_ref(), "us-west-4");
            assert_eq!(
                loader
                    .credentials_provider()
                    .unwrap()
                    .provide_credentials()
                    .await
                    .unwrap()
                    .access_key_id(),
                "akid"
            );
        }
    }
}
