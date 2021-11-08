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
//! Load default SDK configuration:
//! ```rust
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
//! ```rust
//! use aws_config::meta::region::RegionProviderChain;
//! # mod aws_sdk_dynamodb {
//! #   pub struct Client;
//! #   impl Client {
//! #     pub fn new(config: &aws_types::config::Config) -> Self { Client }
//! #   }
//! # }
//! # async fn docs() {
//! let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
//! let config = aws_config::from_env().region(region_provider).load().await;
//! let client = aws_sdk_dynamodb::Client::new(&config);
//! # }
//! ```

#[allow(dead_code)]
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Providers that implement the default AWS provider chain
#[cfg(feature = "default-provider")]
pub mod default_provider;

#[cfg(feature = "environment")]
/// Providers that load configuration from environment variables
pub mod environment;

/// Meta-providers that augment existing providers with new behavior
#[cfg(feature = "meta")]
pub mod meta;

#[cfg(feature = "profile")]
pub mod profile;

#[cfg(feature = "sts")]
pub mod sts;

#[cfg(test)]
mod test_case;

#[cfg(feature = "web-identity-token")]
pub mod web_identity_token;

#[cfg(feature = "http-provider")]
pub mod ecs;

pub mod provider_config;

#[cfg(any(feature = "meta", feature = "default-provider"))]
mod cache;

#[cfg(feature = "imds")]
pub mod imds;

#[cfg(any(feature = "http-provider", feature = "imds"))]
mod json_credentials;

#[cfg(feature = "http-provider")]
mod http_provider;

/// Create an environment loader for AWS Configuration
///
/// # Examples
/// ```rust
/// # async fn create_config() {
/// use aws_types::region::Region;
/// let config = aws_config::from_env().region("us-east-1").load().await;
/// # }
/// ```
#[cfg(feature = "default-provider")]
pub fn from_env() -> ConfigLoader {
    ConfigLoader::default()
}

/// Load a default configuration from the environment
///
/// Convenience wrapper equivalent to `aws_config::from_env().load().await`
#[cfg(feature = "default-provider")]
pub async fn load_from_env() -> aws_types::config::Config {
    from_env().load().await
}

#[cfg(feature = "default-provider")]
/// Load default sources for all configuration with override support
pub use loader::ConfigLoader;

#[cfg(feature = "default-provider")]
mod loader {
    use crate::default_provider::{credentials, region, retry_config};
    use crate::meta::region::ProvideRegion;
    use aws_smithy_types::retry::RetryConfig;
    use aws_types::config::Config;
    use aws_types::credentials::{ProvideCredentials, SharedCredentialsProvider};

    /// Load a cross-service [`Config`](aws_types::config::Config) from the environment
    ///
    /// This builder supports overriding individual components of the generated config. Overriding a component
    /// will skip the standard resolution chain from **for that component**. For example,
    /// if you override the region provider, _even if that provider returns None_, the default region provider
    /// chain will not be used.
    #[derive(Default, Debug)]
    pub struct ConfigLoader {
        region: Option<Box<dyn ProvideRegion>>,
        retry_config: Option<RetryConfig>,
        credentials_provider: Option<SharedCredentialsProvider>,
    }

    impl ConfigLoader {
        /// Override the region used to build [`Config`](aws_types::config::Config).
        ///
        /// # Examples
        /// ```rust
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
        /// ```rust
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

        /// Override the credentials provider used to build [`Config`](aws_types::config::Config).
        /// # Examples
        /// Override the credentials provider but load the default value for region:
        /// ```rust
        /// # use aws_types::Credentials;
        ///  async fn create_config() {
        /// let config = aws_config::from_env()
        ///     .credentials_provider(Credentials::from_keys("accesskey", "secretkey", None))
        ///     .load().await;
        /// # }
        /// ```
        pub fn credentials_provider(
            mut self,
            credentials_provider: impl ProvideCredentials + 'static,
        ) -> Self {
            self.credentials_provider = Some(SharedCredentialsProvider::new(credentials_provider));
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
            let region = if let Some(provider) = self.region {
                provider.region().await
            } else {
                region::default_provider().region().await
            };

            let retry_config = if let Some(retry_config) = self.retry_config {
                retry_config
            } else {
                retry_config::default_provider().retry_config().await
            };

            let credentials_provider = if let Some(provider) = self.credentials_provider {
                provider
            } else {
                let mut builder = credentials::DefaultCredentialsChain::builder();
                builder.set_region(region.clone());
                SharedCredentialsProvider::new(builder.build().await)
            };

            Config::builder()
                .region(region)
                .retry_config(retry_config)
                .credentials_provider(credentials_provider)
                .build()
        }
    }
}

mod connector {

    // create a default connector given the currently enabled cargo features.
    // rustls  | native tls | result
    // -----------------------------
    // yes     | yes        | rustls
    // yes     | no         | rustls
    // no      | yes        | native_tls
    // no      | no         | no default

    use crate::provider_config::HttpSettings;
    use aws_smithy_async::rt::sleep::AsyncSleep;
    use aws_smithy_client::erase::DynConnector;
    use std::sync::Arc;

    // unused when all crate features are disabled
    #[allow(dead_code)]
    pub(crate) fn expect_connector(connector: Option<DynConnector>) -> DynConnector {
        connector.expect("A connector was not available. Either set a custom connector or enable the `rustls` and `native-tls` crate features.")
    }

    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    fn base(
        settings: &HttpSettings,
        sleep: Option<Arc<dyn AsyncSleep>>,
    ) -> aws_smithy_client::hyper_ext::Builder {
        let mut hyper =
            aws_smithy_client::hyper_ext::Adapter::builder().timeout(&settings.timeout_settings);
        if let Some(sleep) = sleep {
            hyper = hyper.sleep_impl(sleep);
        }
        hyper
    }

    #[cfg(feature = "rustls")]
    pub(crate) fn default_connector(
        settings: &HttpSettings,
        sleep: Option<Arc<dyn AsyncSleep>>,
    ) -> Option<DynConnector> {
        let hyper = base(settings, sleep).build(aws_smithy_client::conns::https());
        Some(DynConnector::new(hyper))
    }

    #[cfg(all(not(feature = "rustls"), feature = "native-tls"))]
    pub(crate) fn default_connector(
        settings: &HttpSettings,
        sleep: Option<Arc<dyn AsyncSleep>>,
    ) -> Option<DynConnector> {
        let hyper = base(settings, sleep).build(aws_smithy_client::conns::native_tls());
        Some(DynConnector::new(hyper))
    }

    #[cfg(not(any(feature = "rustls", feature = "native-tls")))]
    pub(crate) fn default_connector(
        _settings: &HttpSettings,
        _sleep: Option<Arc<dyn AsyncSleep>>,
    ) -> Option<DynConnector> {
        None
    }
}
