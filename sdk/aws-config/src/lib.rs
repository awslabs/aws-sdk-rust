/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rustdoc::missing_crate_level_docs,
    unreachable_pub
)]

//! `aws-config` provides implementations of region and credential resolution.
//!
//! These implementations can be used either via the default chain implementation
//! [`from_env`]/[`ConfigLoader`] or ad-hoc individual credential and region providers.
//!
//! [`ConfigLoader`](ConfigLoader) can combine different configuration sources into an AWS shared-config:
//! [`SdkConfig`](aws_types::SdkConfig). [`SdkConfig`](aws_types::SdkConfig) can be used configure
//! an AWS service client.
//!
//! # Examples
//!
//! Load default SDK configuration:
//! ```no_run
//! # mod aws_sdk_dynamodb {
//! #   pub struct Client;
//! #   impl Client {
//! #     pub fn new(config: &aws_types::SdkConfig) -> Self { Client }
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
//! #     pub fn new(config: &aws_types::SdkConfig) -> Self { Client }
//! #   }
//! # }
//! # async fn docs() {
//! # use aws_config::meta::region::RegionProviderChain;
//! let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
//! let config = aws_config::from_env().region(region_provider).load().await;
//! let client = aws_sdk_dynamodb::Client::new(&config);
//! # }
//! ```
//!
//! Override configuration after construction of `SdkConfig`:
//!
//! ```no_run
//! # use aws_credential_types::provider::ProvideCredentials;
//! # use aws_types::SdkConfig;
//! # mod aws_sdk_dynamodb {
//! #   pub mod config {
//! #     pub struct Builder;
//! #     impl Builder {
//! #       pub fn credentials_provider(
//! #         self,
//! #         credentials_provider: impl aws_credential_types::provider::ProvideCredentials + 'static) -> Self { self }
//! #       pub fn build(self) -> Builder { self }
//! #     }
//! #     impl From<&aws_types::SdkConfig> for Builder {
//! #       fn from(_: &aws_types::SdkConfig) -> Self {
//! #           todo!()
//! #       }
//! #     }
//! #   }
//! #   pub struct Client;
//! #   impl Client {
//! #     pub fn from_conf(conf: config::Builder) -> Self { Client }
//! #     pub fn new(config: &aws_types::SdkConfig) -> Self { Client }
//! #   }
//! # }
//! # async fn docs() {
//! # use aws_config::meta::region::RegionProviderChain;
//! # fn custom_provider(base: &SdkConfig) -> impl ProvideCredentials {
//! #   base.credentials_provider().unwrap().clone()
//! # }
//! let sdk_config = aws_config::load_from_env().await;
//! let custom_credentials_provider = custom_provider(&sdk_config);
//! let dynamo_config = aws_sdk_dynamodb::config::Builder::from(&sdk_config)
//!   .credentials_provider(custom_credentials_provider)
//!   .build();
//! let client = aws_sdk_dynamodb::Client::from_conf(dynamo_config);
//! # }
//! ```

pub use aws_smithy_http::endpoint;
// Re-export types from aws-types
pub use aws_types::{
    app_name::{AppName, InvalidAppName},
    SdkConfig,
};
/// Load default sources for all configuration with override support
pub use loader::ConfigLoader;

#[allow(dead_code)]
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod test_case;

mod fs_util;
mod http_credential_provider;
mod json_credentials;

pub mod connector;
pub mod credential_process;
pub mod default_provider;
pub mod ecs;
pub mod environment;
pub mod imds;
pub mod meta;
pub mod profile;
pub mod provider_config;
pub mod retry;
#[cfg(feature = "credentials-sso")]
pub mod sso;
pub(crate) mod standard_property;
pub mod sts;
pub mod timeout;
pub mod web_identity_token;

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
pub async fn load_from_env() -> aws_types::SdkConfig {
    from_env().load().await
}

mod loader {
    use std::sync::Arc;

    use aws_credential_types::cache::CredentialsCache;
    use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
    use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep, SharedAsyncSleep};
    use aws_smithy_async::time::{SharedTimeSource, TimeSource};
    use aws_smithy_client::http_connector::HttpConnector;
    use aws_smithy_types::retry::RetryConfig;
    use aws_smithy_types::timeout::TimeoutConfig;
    use aws_types::app_name::AppName;
    use aws_types::docs_for;
    use aws_types::os_shim_internal::{Env, Fs};
    use aws_types::SdkConfig;

    use crate::connector::default_connector;
    use crate::default_provider::use_dual_stack::use_dual_stack_provider;
    use crate::default_provider::use_fips::use_fips_provider;
    use crate::default_provider::{app_name, credentials, region, retry_config, timeout_config};
    use crate::meta::region::ProvideRegion;
    use crate::profile::profile_file::ProfileFiles;
    use crate::provider_config::ProviderConfig;

    #[derive(Default, Debug)]
    enum CredentialsProviderOption {
        /// No provider was set by the user. We can set up the default credentials provider chain.
        #[default]
        NotSet,
        /// The credentials provider was explicitly unset. Do not set up a default chain.
        ExplicitlyUnset,
        /// Use the given credentials provider.
        Set(SharedCredentialsProvider),
    }

    /// Load a cross-service [`SdkConfig`](aws_types::SdkConfig) from the environment
    ///
    /// This builder supports overriding individual components of the generated config. Overriding a component
    /// will skip the standard resolution chain from **for that component**. For example,
    /// if you override the region provider, _even if that provider returns None_, the default region provider
    /// chain will not be used.
    #[derive(Default, Debug)]
    pub struct ConfigLoader {
        app_name: Option<AppName>,
        credentials_cache: Option<CredentialsCache>,
        credentials_provider: CredentialsProviderOption,
        endpoint_url: Option<String>,
        region: Option<Box<dyn ProvideRegion>>,
        retry_config: Option<RetryConfig>,
        sleep: Option<SharedAsyncSleep>,
        timeout_config: Option<TimeoutConfig>,
        provider_config: Option<ProviderConfig>,
        http_connector: Option<HttpConnector>,
        profile_name_override: Option<String>,
        profile_files_override: Option<ProfileFiles>,
        use_fips: Option<bool>,
        use_dual_stack: Option<bool>,
        time_source: Option<SharedTimeSource>,
        env: Option<Env>,
        fs: Option<Fs>,
    }

    impl ConfigLoader {
        /// Override the region used to build [`SdkConfig`](aws_types::SdkConfig).
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

        /// Override the retry_config used to build [`SdkConfig`](aws_types::SdkConfig).
        ///
        /// # Examples
        /// ```no_run
        /// # async fn create_config() {
        /// use aws_config::retry::RetryConfig;
        ///
        /// let config = aws_config::from_env()
        ///     .retry_config(RetryConfig::standard().with_max_attempts(2))
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
            self.retry_config = Some(retry_config);
            self
        }

        /// Override the timeout config used to build [`SdkConfig`](aws_types::SdkConfig).
        /// **Note: This only sets timeouts for calls to AWS services.** Timeouts for the credentials
        /// provider chain are configured separately.
        ///
        /// # Examples
        /// ```no_run
        /// # use std::time::Duration;
        /// # async fn create_config() {
        /// use aws_config::timeout::TimeoutConfig;
        ///
        /// let config = aws_config::from_env()
        ///    .timeout_config(
        ///        TimeoutConfig::builder()
        ///            .operation_timeout(Duration::from_secs(5))
        ///            .build()
        ///    )
        ///    .load()
        ///    .await;
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
            self.sleep = Some(SharedAsyncSleep::new(sleep));
            self
        }

        /// Set the time source used for tasks like signing requests
        pub fn time_source(mut self, time_source: impl TimeSource + 'static) -> Self {
            self.time_source = Some(SharedTimeSource::new(time_source));
            self
        }

        /// Override the [`HttpConnector`] for this [`ConfigLoader`]. The connector will be used for
        /// both AWS services and credential providers. When [`HttpConnector::ConnectorFn`] is used,
        /// the connector will be lazily instantiated as needed based on the provided settings.
        ///
        /// **Note**: In order to take advantage of late-configured timeout settings, you MUST use
        /// [`HttpConnector::ConnectorFn`]
        /// when configuring this connector.
        ///
        /// If you wish to use a separate connector when creating clients, use the client-specific config.
        /// ## Examples
        /// ```no_run
        /// # use aws_smithy_async::rt::sleep::SharedAsyncSleep;
        /// use aws_smithy_client::http_connector::HttpConnector;
        /// #[cfg(feature = "client-hyper")]
        /// # async fn create_config() {
        /// use std::time::Duration;
        /// use aws_smithy_client::{Client, hyper_ext};
        /// use aws_smithy_client::erase::DynConnector;
        /// use aws_smithy_client::http_connector::ConnectorSettings;
        ///
        /// let connector_fn = |settings:  &ConnectorSettings, sleep: Option<SharedAsyncSleep>| {
        ///   let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        ///       .with_webpki_roots()
        ///       // NOTE: setting `https_only()` will not allow this connector to work with IMDS.
        ///       .https_only()
        ///       .enable_http1()
        ///       .enable_http2()
        ///       .build();
        ///   let mut smithy_connector = hyper_ext::Adapter::builder()
        ///       // Optionally set things like timeouts as well
        ///       .connector_settings(settings.clone());
        ///   smithy_connector.set_sleep_impl(sleep);
        ///   Some(DynConnector::new(smithy_connector.build(https_connector)))
        /// };
        /// let connector = HttpConnector::ConnectorFn(std::sync::Arc::new(connector_fn));
        /// let sdk_config = aws_config::from_env()
        ///     .http_connector(connector)
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn http_connector(mut self, http_connector: impl Into<HttpConnector>) -> Self {
            self.http_connector = Some(http_connector.into());
            self
        }

        /// Override the credentials cache used to build [`SdkConfig`](aws_types::SdkConfig).
        ///
        /// # Examples
        ///
        /// Override the credentials cache but load the default value for region:
        /// ```no_run
        /// # use aws_credential_types::cache::CredentialsCache;
        /// # async fn create_config() {
        /// let config = aws_config::from_env()
        ///     .credentials_cache(CredentialsCache::lazy())
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn credentials_cache(mut self, credentials_cache: CredentialsCache) -> Self {
            self.credentials_cache = Some(credentials_cache);
            self
        }

        /// Override the credentials provider used to build [`SdkConfig`](aws_types::SdkConfig).
        ///
        /// # Examples
        ///
        /// Override the credentials provider but load the default value for region:
        /// ```no_run
        /// # use aws_credential_types::Credentials;
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
            self.credentials_provider = CredentialsProviderOption::Set(
                SharedCredentialsProvider::new(credentials_provider),
            );
            self
        }

        /// Don't use credentials to sign requests.
        ///
        /// Turning off signing with credentials is necessary in some cases, such as using
        /// anonymous auth for S3, calling operations in STS that don't require a signature,
        /// or using token-based auth.
        ///
        /// # Examples
        ///
        /// Turn off credentials in order to call a service without signing:
        /// ```no_run
        /// # async fn create_config() {
        /// let config = aws_config::from_env()
        ///     .no_credentials()
        ///     .load()
        ///     .await;
        /// # }
        /// ```
        pub fn no_credentials(mut self) -> Self {
            self.credentials_provider = CredentialsProviderOption::ExplicitlyUnset;
            self
        }

        /// Override the name of the app used to build [`SdkConfig`](aws_types::SdkConfig).
        ///
        /// This _optional_ name is used to identify the application in the user agent that
        /// gets sent along with requests.
        ///
        /// # Examples
        /// ```no_run
        /// # async fn create_config() {
        /// use aws_config::AppName;
        /// let config = aws_config::from_env()
        ///     .app_name(AppName::new("my-app-name").expect("valid app name"))
        ///     .load().await;
        /// # }
        /// ```
        pub fn app_name(mut self, app_name: AppName) -> Self {
            self.app_name = Some(app_name);
            self
        }

        /// Provides the ability to programmatically override the profile files that get loaded by the SDK.
        ///
        /// The [`Default`] for `ProfileFiles` includes the default SDK config and credential files located in
        /// `~/.aws/config` and `~/.aws/credentials` respectively.
        ///
        /// Any number of config and credential files may be added to the `ProfileFiles` file set, with the
        /// only requirement being that there is at least one of each. Profile file locations will produce an
        /// error if they don't exist, but the default config/credentials files paths are exempt from this validation.
        ///
        /// # Example: Using a custom profile file path
        ///
        /// ```no_run
        /// use aws_config::profile::{ProfileFileCredentialsProvider, ProfileFileRegionProvider};
        /// use aws_config::profile::profile_file::{ProfileFiles, ProfileFileKind};
        ///
        /// # async fn example() {
        /// let profile_files = ProfileFiles::builder()
        ///     .with_file(ProfileFileKind::Credentials, "some/path/to/credentials-file")
        ///     .build();
        /// let sdk_config = aws_config::from_env()
        ///     .profile_files(profile_files)
        ///     .load()
        ///     .await;
        /// # }
        pub fn profile_files(mut self, profile_files: ProfileFiles) -> Self {
            self.profile_files_override = Some(profile_files);
            self
        }

        /// Override the profile name used by configuration providers
        ///
        /// Profile name is selected from an ordered list of sources:
        /// 1. This override.
        /// 2. The value of the `AWS_PROFILE` environment variable.
        /// 3. `default`
        ///
        /// Each AWS profile has a name. For example, in the file below, the profiles are named
        /// `dev`, `prod` and `staging`:
        /// ```ini
        /// [dev]
        /// ec2_metadata_service_endpoint = http://my-custom-endpoint:444
        ///
        /// [staging]
        /// ec2_metadata_service_endpoint = http://my-custom-endpoint:444
        ///
        /// [prod]
        /// ec2_metadata_service_endpoint = http://my-custom-endpoint:444
        /// ```
        ///
        /// See [Named profiles](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-profiles.html)
        /// for more information about naming profiles.
        ///
        /// # Example: Using a custom profile name
        ///
        /// ```no_run
        /// use aws_config::profile::{ProfileFileCredentialsProvider, ProfileFileRegionProvider};
        /// use aws_config::profile::profile_file::{ProfileFiles, ProfileFileKind};
        ///
        /// # async fn example() {
        /// let sdk_config = aws_config::from_env()
        ///     .profile_name("prod")
        ///     .load()
        ///     .await;
        /// # }
        pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
            self.profile_name_override = Some(profile_name.into());
            self
        }

        /// Override the endpoint URL used for **all** AWS services.
        ///
        /// This method will override the endpoint URL used for **all** AWS services. This primarily
        /// exists to set a static endpoint for tools like `LocalStack`. When sending requests to
        /// production AWS services, this method should only be used for service-specific behavior.
        ///
        /// When this method is used, the [`Region`](aws_types::region::Region) is only used for
        /// signing; it is not used to route the request.
        ///
        /// # Examples
        ///
        /// Use a static endpoint for all services
        /// ```no_run
        /// # async fn create_config() {
        /// let sdk_config = aws_config::from_env()
        ///     .endpoint_url("http://localhost:1234")
        ///     .load()
        ///     .await;
        /// # }
        pub fn endpoint_url(mut self, endpoint_url: impl Into<String>) -> Self {
            self.endpoint_url = Some(endpoint_url.into());
            self
        }

        #[doc = docs_for!(use_fips)]
        pub fn use_fips(mut self, use_fips: bool) -> Self {
            self.use_fips = Some(use_fips);
            self
        }

        #[doc = docs_for!(use_dual_stack)]
        pub fn use_dual_stack(mut self, use_dual_stack: bool) -> Self {
            self.use_dual_stack = Some(use_dual_stack);
            self
        }

        /// Set configuration for all sub-loaders (credentials, region etc.)
        ///
        /// Update the `ProviderConfig` used for all nested loaders. This can be used to override
        /// the HTTPs connector used by providers or to stub in an in memory `Env` or `Fs` for testing.
        ///
        /// # Examples
        /// ```no_run
        /// # #[cfg(feature = "hyper-client")]
        /// # async fn create_config() {
        /// use aws_config::provider_config::ProviderConfig;
        /// let custom_https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        ///     .with_webpki_roots()
        ///     .https_only()
        ///     .enable_http1()
        ///     .build();
        /// let provider_config = ProviderConfig::default().with_tcp_connector(custom_https_connector);
        /// let shared_config = aws_config::from_env().configure(provider_config).load().await;
        /// # }
        /// ```
        #[deprecated(
            note = "Use setters on this builder instead. configure is very hard to use correctly."
        )]
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
        /// be set in the resulting [`SdkConfig`](aws_types::SdkConfig)
        pub async fn load(self) -> SdkConfig {
            let http_connector = self
                .http_connector
                .unwrap_or_else(|| HttpConnector::ConnectorFn(Arc::new(default_connector)));

            let time_source = self.time_source.unwrap_or_default();

            let sleep_impl = if self.sleep.is_some() {
                self.sleep
            } else {
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
            };

            let conf = self
                .provider_config
                .unwrap_or_else(|| {
                    ProviderConfig::init(time_source.clone(), sleep_impl.clone())
                        .with_fs(self.fs.unwrap_or_default())
                        .with_env(self.env.unwrap_or_default())
                        .with_http_connector(http_connector.clone())
                })
                .with_profile_config(self.profile_files_override, self.profile_name_override);
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

            let credentials_provider = match self.credentials_provider {
                CredentialsProviderOption::Set(provider) => Some(provider),
                CredentialsProviderOption::NotSet => {
                    let mut builder =
                        credentials::DefaultCredentialsChain::builder().configure(conf.clone());
                    builder.set_region(region.clone());
                    Some(SharedCredentialsProvider::new(builder.build().await))
                }
                CredentialsProviderOption::ExplicitlyUnset => None,
            };

            let credentials_cache = if credentials_provider.is_some() {
                Some(self.credentials_cache.unwrap_or_else(|| {
                    let mut builder =
                        CredentialsCache::lazy_builder().time_source(conf.time_source());
                    builder.set_sleep(conf.sleep());
                    builder.into_credentials_cache()
                }))
            } else {
                None
            };

            let use_fips = if let Some(use_fips) = self.use_fips {
                Some(use_fips)
            } else {
                use_fips_provider(&conf).await
            };

            let use_dual_stack = if let Some(use_dual_stack) = self.use_dual_stack {
                Some(use_dual_stack)
            } else {
                use_dual_stack_provider(&conf).await
            };

            let mut builder = SdkConfig::builder()
                .region(region)
                .retry_config(retry_config)
                .timeout_config(timeout_config)
                .time_source(time_source)
                .http_connector(http_connector);

            builder.set_app_name(app_name);
            builder.set_credentials_cache(credentials_cache);
            builder.set_credentials_provider(credentials_provider);
            builder.set_sleep_impl(sleep_impl);
            builder.set_endpoint_url(self.endpoint_url);
            builder.set_use_fips(use_fips);
            builder.set_use_dual_stack(use_dual_stack);
            builder.build()
        }
    }

    #[cfg(test)]
    impl ConfigLoader {
        pub(crate) fn env(mut self, env: Env) -> Self {
            self.env = Some(env);
            self
        }

        pub(crate) fn fs(mut self, fs: Fs) -> Self {
            self.fs = Some(fs);
            self
        }
    }

    #[cfg(test)]
    mod test {
        use aws_credential_types::provider::ProvideCredentials;
        use aws_smithy_async::rt::sleep::TokioSleep;
        use aws_smithy_async::time::{StaticTimeSource, TimeSource};
        use aws_smithy_client::erase::DynConnector;
        use aws_smithy_client::never::NeverConnector;
        use aws_smithy_client::test_connection::infallible_connection_fn;
        use aws_types::app_name::AppName;
        use aws_types::os_shim_internal::{Env, Fs};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use std::time::{SystemTime, UNIX_EPOCH};
        use tracing_test::traced_test;

        use crate::profile::profile_file::{ProfileFileKind, ProfileFiles};
        use crate::test_case::{no_traffic_connector, InstantSleep};
        use crate::{from_env, ConfigLoader};

        #[tokio::test]
        #[traced_test]
        async fn provider_config_used() {
            let env = Env::from_slice(&[
                ("AWS_MAX_ATTEMPTS", "10"),
                ("AWS_REGION", "us-west-4"),
                ("AWS_ACCESS_KEY_ID", "akid"),
                ("AWS_SECRET_ACCESS_KEY", "secret"),
            ]);
            let fs =
                Fs::from_slice(&[("test_config", "[profile custom]\nsdk-ua-app-id = correct")]);
            let loader = from_env()
                .sleep_impl(TokioSleep::new())
                .env(env)
                .fs(fs)
                .http_connector(DynConnector::new(NeverConnector::new()))
                .profile_name("custom")
                .profile_files(
                    ProfileFiles::builder()
                        .with_file(ProfileFileKind::Config, "test_config")
                        .build(),
                )
                .load()
                .await;
            assert_eq!(10, loader.retry_config().unwrap().max_attempts());
            assert_eq!("us-west-4", loader.region().unwrap().as_ref());
            assert_eq!(
                "akid",
                loader
                    .credentials_provider()
                    .unwrap()
                    .provide_credentials()
                    .await
                    .unwrap()
                    .access_key_id(),
            );
            assert_eq!(Some(&AppName::new("correct").unwrap()), loader.app_name());
            logs_assert(|lines| {
                let num_config_loader_logs = lines
                    .iter()
                    .filter(|l| l.contains("provider_config_used"))
                    .filter(|l| l.contains("config file loaded"))
                    .count();
                match num_config_loader_logs {
                    0 => Err("no config file logs found!".to_string()),
                    1 => Ok(()),
                    more => Err(format!(
                        "the config file was parsed more than once! (parsed {})",
                        more
                    )),
                }
            });
        }

        fn base_conf() -> ConfigLoader {
            from_env()
                .sleep_impl(InstantSleep)
                .http_connector(no_traffic_connector())
        }

        #[tokio::test]
        async fn load_fips() {
            let conf = base_conf().use_fips(true).load().await;
            assert_eq!(Some(true), conf.use_fips());
        }

        #[tokio::test]
        async fn load_dual_stack() {
            let conf = base_conf().use_dual_stack(false).load().await;
            assert_eq!(Some(false), conf.use_dual_stack());

            let conf = base_conf().load().await;
            assert_eq!(None, conf.use_dual_stack());
        }

        #[tokio::test]
        async fn app_name() {
            let app_name = AppName::new("my-app-name").unwrap();
            let conf = base_conf().app_name(app_name.clone()).load().await;
            assert_eq!(Some(&app_name), conf.app_name());
        }

        #[cfg(all(not(aws_sdk_middleware_mode), feature = "rustls"))]
        #[tokio::test]
        async fn disable_default_credentials() {
            let config = from_env().no_credentials().load().await;
            assert!(config.credentials_cache().is_none());
            assert!(config.credentials_provider().is_none());
        }

        #[tokio::test]
        async fn connector_is_shared() {
            let num_requests = Arc::new(AtomicUsize::new(0));
            let movable = num_requests.clone();
            let conn = infallible_connection_fn(move |_req| {
                movable.fetch_add(1, Ordering::Relaxed);
                http::Response::new("ok!")
            });
            let config = from_env().http_connector(conn.clone()).load().await;
            config
                .credentials_provider()
                .unwrap()
                .provide_credentials()
                .await
                .expect_err("no traffic is allowed");
            let num_requests = num_requests.load(Ordering::Relaxed);
            assert!(num_requests > 0, "{}", num_requests);
        }

        #[tokio::test]
        async fn time_source_is_passed() {
            #[derive(Debug)]
            struct PanicTs;
            impl TimeSource for PanicTs {
                fn now(&self) -> SystemTime {
                    panic!("timesource-was-used")
                }
            }
            let config = from_env()
                .sleep_impl(InstantSleep)
                .time_source(StaticTimeSource::new(UNIX_EPOCH))
                .http_connector(no_traffic_connector())
                .load()
                .await;
            // assert that the innards contain the customized fields
            for inner in ["InstantSleep", "StaticTimeSource"] {
                assert!(
                    format!("{:#?}", config.credentials_cache()).contains(inner),
                    "{:#?}",
                    config.credentials_cache()
                );
                assert!(
                    format!("{:#?}", config.credentials_provider()).contains(inner),
                    "{:#?}",
                    config.credentials_cache()
                );
            }
        }
    }
}
