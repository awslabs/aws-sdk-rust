/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Default Provider chains for [`region`](default_provider::region), [`credentials`](default_provider::credentials),
//! [retries](default_provider::retry_config), [timeouts](default_provider::timeout_config) and [app name](default_provider::app_name).
//!
//! Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
//! if you need to set custom configuration options to override the default resolution chain.

/// Default [region](aws_types::region::Region) provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod region {
    use aws_types::region::Region;

    use crate::environment::region::EnvironmentVariableRegionProvider;
    use crate::meta::region::{ProvideRegion, RegionProviderChain};
    use crate::provider_config::ProviderConfig;
    use crate::{imds, profile};

    /// Default Region Provider chain
    ///
    /// This provider will check the following sources in order:
    /// 1. [Environment variables](EnvironmentVariableRegionProvider)
    /// 2. [Profile file](crate::profile::region::ProfileFileRegionProvider)
    /// 3. [EC2 IMDSv2](crate::imds::region)
    pub fn default_provider() -> impl ProvideRegion {
        Builder::default().build()
    }

    /// Default region provider chain
    #[derive(Debug)]
    pub struct DefaultRegionChain(RegionProviderChain);

    impl DefaultRegionChain {
        /// Load a region from this chain
        pub async fn region(&self) -> Option<Region> {
            self.0.region().await
        }

        /// Builder for [`DefaultRegionChain`]
        pub fn builder() -> Builder {
            Builder::default()
        }
    }

    /// Builder for [DefaultRegionChain]
    #[derive(Default)]
    pub struct Builder {
        env_provider: EnvironmentVariableRegionProvider,
        profile_file: profile::region::Builder,
        imds: imds::region::Builder,
    }

    impl Builder {
        #[doc(hidden)]
        /// Configure the default chain
        ///
        /// Exposed for overriding the environment when unit-testing providers
        pub fn configure(mut self, configuration: &ProviderConfig) -> Self {
            self.env_provider =
                EnvironmentVariableRegionProvider::new_with_env(configuration.env());
            self.profile_file = self.profile_file.configure(configuration);
            self.imds = self.imds.configure(configuration);
            self
        }

        /// Override the profile name used by this provider
        pub fn profile_name(mut self, name: &str) -> Self {
            self.profile_file = self.profile_file.profile_name(name);
            self
        }

        /// Build a [DefaultRegionChain]
        pub fn build(self) -> DefaultRegionChain {
            DefaultRegionChain(
                RegionProviderChain::first_try(self.env_provider)
                    .or_else(self.profile_file.build())
                    .or_else(self.imds.build()),
            )
        }
    }

    impl ProvideRegion for DefaultRegionChain {
        fn region(&self) -> crate::meta::region::future::ProvideRegion {
            ProvideRegion::region(&self.0)
        }
    }
}

/// Default retry behavior configuration provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod retry_config {
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
            let builder_from_profile = match self.profile_file.build().retry_config_builder().await
            {
                Ok(retry_config_builder) => retry_config_builder,
                Err(err) => panic!("{}", err),
            };

            builder_from_env
                .take_unset_from(builder_from_profile)
                .build()
        }
    }
}

/// Default app name provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod app_name {
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
            self.env_provider =
                EnvironmentVariableAppNameProvider::new_with_env(configuration.env());
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
}

/// Default timeout configuration provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options to override the default resolution chain.
pub mod timeout_config {
    use aws_smithy_types::timeout::TimeoutConfig;

    use crate::environment::timeout_config::EnvironmentVariableTimeoutConfigProvider;
    use crate::profile;
    use crate::provider_config::ProviderConfig;

    /// Default [`TimeoutConfig`] Provider chain
    ///
    /// Unlike other credentials and region, [`TimeoutConfig`] has no related `TimeoutConfigProvider` trait. Instead,
    /// a builder struct is returned which has a similar API.
    ///
    /// This provider will check the following sources in order:
    /// 1. [Environment variables](EnvironmentVariableTimeoutConfigProvider)
    /// 2. [Profile file](crate::profile::timeout_config::ProfileFileTimeoutConfigProvider) (`~/.aws/config`)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use std::error::Error;
    /// # #[tokio::main]
    /// # async fn main() {
    /// use aws_config::default_provider::timeout_config;
    ///
    /// // Load a timeout config from a specific profile
    /// let timeout_config = timeout_config::default_provider()
    ///     .profile_name("other_profile")
    ///     .timeout_config()
    ///     .await;
    /// let config = aws_config::from_env()
    ///     // Override the timeout config set by the default profile
    ///     .timeout_config(timeout_config)
    ///     .load()
    ///     .await;
    /// // instantiate a service client:
    /// // <my_aws_service>::Client::new(&config);
    /// # }
    /// ```
    pub fn default_provider() -> Builder {
        Builder::default()
    }

    /// Builder for [`TimeoutConfig`] that checks the environment variables and AWS profile files for configuration
    #[derive(Default)]
    pub struct Builder {
        env_provider: EnvironmentVariableTimeoutConfigProvider,
        profile_file: profile::timeout_config::Builder,
    }

    impl Builder {
        /// Configure the default chain
        ///
        /// Exposed for overriding the environment when unit-testing providers
        pub fn configure(mut self, configuration: &ProviderConfig) -> Self {
            self.env_provider =
                EnvironmentVariableTimeoutConfigProvider::new_with_env(configuration.env());
            self.profile_file = self.profile_file.configure(configuration);
            self
        }

        /// Override the profile name used by this provider
        pub fn profile_name(mut self, name: &str) -> Self {
            self.profile_file = self.profile_file.profile_name(name);
            self
        }

        /// Attempt to create a [`TimeoutConfig`](aws_smithy_types::timeout::TimeoutConfig) from following sources in order:
        /// 1. [Environment variables](crate::environment::timeout_config::EnvironmentVariableTimeoutConfigProvider)
        /// 2. [Profile file](crate::profile::timeout_config::ProfileFileTimeoutConfigProvider)
        ///
        /// Precedence is considered on a per-field basis. If no timeout is specified, requests will never time out.
        ///
        /// # Panics
        ///
        /// This will panic if:
        /// - a timeout is set to `NaN`, a negative number, or infinity
        /// - a timeout can't be parsed as a floating point number
        pub async fn timeout_config(self) -> TimeoutConfig {
            // Both of these can return errors due to invalid config settings and we want to surface those as early as possible
            // hence, we'll panic if any config values are invalid (missing values are OK though)
            // We match this instead of unwrapping so we can print the error with the `Display` impl instead of the `Debug` impl that unwrap uses
            let builder_from_env = match self.env_provider.timeout_config() {
                Ok(timeout_config_builder) => timeout_config_builder,
                Err(err) => panic!("{}", err),
            };
            let builder_from_profile = match self.profile_file.build().timeout_config().await {
                Ok(timeout_config_builder) => timeout_config_builder,
                Err(err) => panic!("{}", err),
            };

            let conf = builder_from_env.take_unset_from(builder_from_profile);

            if conf.tls_negotiation_timeout().is_some() {
                tracing::warn!(
                    "A TLS negotiation timeout was set but that feature is currently unimplemented so the setting will be ignored. \
                    To help us prioritize support for this feature, please upvote aws-sdk-rust#151 (https://github.com/awslabs/aws-sdk-rust/issues/151)")
            }

            if conf.connect_timeout().is_some() {
                tracing::warn!(
                    "A connect timeout was set but that feature is currently unimplemented so the setting will be ignored. \
                    To help us prioritize support for this feature, please upvote aws-sdk-rust#151 (https://github.com/awslabs/aws-sdk-rust/issues/151)")
            }

            if conf.read_timeout().is_some() {
                tracing::warn!(
                    "A read timeout was set but that feature is currently unimplemented so the setting will be ignored. \
                    To help us prioritize support for this feature, please upvote aws-sdk-rust#151 (https://github.com/awslabs/aws-sdk-rust/issues/151)")
            }

            conf
        }
    }
}

/// Default credentials provider chain
///
/// Typically, this module is used via [`load_from_env`](crate::load_from_env) or [`from_env`](crate::from_env). It should only be used directly
/// if you need to set custom configuration options like [`region`](credentials::Builder::region) or [`profile_name`](credentials::Builder::profile_name).
pub mod credentials {
    use std::borrow::Cow;

    use aws_types::credentials::{future, ProvideCredentials};

    use crate::environment::credentials::EnvironmentVariableCredentialsProvider;
    use crate::meta::credentials::{CredentialsProviderChain, LazyCachingCredentialsProvider};
    use crate::meta::region::ProvideRegion;
    use crate::provider_config::ProviderConfig;

    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    /// Default Credentials Provider chain
    ///
    /// The region from the default region provider will be used
    pub async fn default_provider() -> impl ProvideCredentials {
        DefaultCredentialsChain::builder().build().await
    }

    /// Default AWS Credential Provider Chain
    ///
    /// Resolution order:
    /// 1. Environment variables: [`EnvironmentVariableCredentialsProvider`](crate::environment::EnvironmentVariableCredentialsProvider)
    /// 2. Shared config (`~/.aws/config`, `~/.aws/credentials`): [`SharedConfigCredentialsProvider`](crate::profile::ProfileFileCredentialsProvider)
    /// 3. [Web Identity Tokens](crate::web_identity_token)
    /// 4. ECS (IAM Roles for Tasks) & General HTTP credentials: [`ecs`](crate::ecs)
    /// 5. [EC2 IMDSv2](crate::imds)
    ///
    /// The outer provider is wrapped in a refreshing cache.
    ///
    /// More providers are a work in progress.
    ///
    /// # Examples
    /// Create a default chain with a custom region:
    /// ```no_run
    /// use aws_types::region::Region;
    /// use aws_config::default_provider::credentials::DefaultCredentialsChain;
    /// let credentials_provider = DefaultCredentialsChain::builder()
    ///     .region(Region::new("us-west-1"))
    ///     .build();
    /// ```
    ///
    /// Create a default chain with no overrides:
    /// ```no_run
    /// use aws_config::default_provider::credentials::DefaultCredentialsChain;
    /// let credentials_provider = DefaultCredentialsChain::builder().build();
    /// ```
    ///
    /// Create a default chain that uses a different profile:
    /// ```no_run
    /// use aws_config::default_provider::credentials::DefaultCredentialsChain;
    /// let credentials_provider = DefaultCredentialsChain::builder()
    ///     .profile_name("otherprofile")
    ///     .build();
    /// ```
    #[derive(Debug)]
    pub struct DefaultCredentialsChain(LazyCachingCredentialsProvider);

    impl DefaultCredentialsChain {
        /// Builder for `DefaultCredentialsChain`
        pub fn builder() -> Builder {
            Builder::default()
        }
    }

    impl ProvideCredentials for DefaultCredentialsChain {
        fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
        where
            Self: 'a,
        {
            self.0.provide_credentials()
        }
    }

    /// Builder for [`DefaultCredentialsChain`](DefaultCredentialsChain)
    #[derive(Default)]
    pub struct Builder {
        profile_file_builder: crate::profile::credentials::Builder,
        web_identity_builder: crate::web_identity_token::Builder,
        imds_builder: crate::imds::credentials::Builder,
        ecs_builder: crate::ecs::Builder,
        credential_cache: crate::meta::credentials::lazy_caching::Builder,
        region_override: Option<Box<dyn ProvideRegion>>,
        region_chain: crate::default_provider::region::Builder,
        conf: Option<ProviderConfig>,
    }

    impl Builder {
        /// Sets the region used when making requests to AWS services
        ///
        /// When unset, the default region resolver chain will be used.
        pub fn region(mut self, region: impl ProvideRegion + 'static) -> Self {
            self.set_region(Some(region));
            self
        }

        /// Sets the region used when making requests to AWS services
        ///
        /// When unset, the default region resolver chain will be used.
        pub fn set_region(&mut self, region: Option<impl ProvideRegion + 'static>) -> &mut Self {
            self.region_override = region.map(|provider| Box::new(provider) as _);
            self
        }

        /// Add an additional credential source for the ProfileProvider
        ///
        /// Assume role profiles may specify named credential sources:
        /// ```ini
        /// [default]
        /// role_arn = arn:aws:iam::123456789:role/RoleA
        /// credential_source = MyCustomProvider
        /// ```
        ///
        /// Typically, these are built-in providers like `Environment`, however, custom sources may
        /// also be used.
        ///
        /// See [`with_custom_provider`](crate::profile::credentials::Builder::with_custom_provider)
        pub fn with_custom_credential_source(
            mut self,
            name: impl Into<Cow<'static, str>>,
            provider: impl ProvideCredentials + 'static,
        ) -> Self {
            self.profile_file_builder = self
                .profile_file_builder
                .with_custom_provider(name, provider);
            self
        }

        /// Override the profile name used by this provider
        ///
        /// When unset, the value of the `AWS_PROFILE` environment variable will be used.
        pub fn profile_name(mut self, name: &str) -> Self {
            self.profile_file_builder = self.profile_file_builder.profile_name(name);
            self.region_chain = self.region_chain.profile_name(name);
            self
        }

        /// Override the configuration used for this provider
        pub fn configure(mut self, config: ProviderConfig) -> Self {
            self.region_chain = self.region_chain.configure(&config);
            self.conf = Some(config);
            self
        }

        /// Creates a `DefaultCredentialsChain`
        ///
        /// ## Panics
        /// This function will panic if no connector has been set and neither `rustls` and `native-tls`
        /// features have both been disabled.
        pub async fn build(self) -> DefaultCredentialsChain {
            let region = match self.region_override {
                Some(provider) => provider.region().await,
                None => self.region_chain.build().region().await,
            };

            let conf = self.conf.unwrap_or_default().with_region(region);

            let env_provider = EnvironmentVariableCredentialsProvider::new_with_env(conf.env());
            let profile_provider = self.profile_file_builder.configure(&conf).build();
            let web_identity_token_provider = self.web_identity_builder.configure(&conf).build();
            let imds_provider = self.imds_builder.configure(&conf).build();
            let ecs_provider = self.ecs_builder.configure(&conf).build();

            let provider_chain = CredentialsProviderChain::first_try("Environment", env_provider)
                .or_else("Profile", profile_provider)
                .or_else("WebIdentityToken", web_identity_token_provider)
                .or_else("EcsContainer", ecs_provider)
                .or_else("Ec2InstanceMetadata", imds_provider);
            let cached_provider = self.credential_cache.configure(&conf).load(provider_chain);

            DefaultCredentialsChain(cached_provider.build())
        }
    }

    #[cfg(test)]
    mod test {
        use tracing_test::traced_test;

        use aws_smithy_types::retry::{RetryConfig, RetryMode};
        use aws_types::credentials::ProvideCredentials;
        use aws_types::os_shim_internal::{Env, Fs};

        use crate::default_provider::credentials::DefaultCredentialsChain;
        use crate::default_provider::retry_config;
        use crate::provider_config::ProviderConfig;
        use crate::test_case::TestEnvironment;

        /// Test generation macro
        ///
        /// # Examples
        /// **Run the test case in `test-data/default-provider-chain/test_name`
        /// ```no_run
        /// make_test!(test_name);
        /// ```
        ///
        /// **Update (responses are replayed but new requests are recorded) the test case**:
        /// ```no_run
        /// make_test!(update: test_name)
        /// ```
        ///
        /// **Run the test case against a real HTTPS connection:**
        /// > Note: Be careful to remove sensitive information before committing. Always use a temporary
        /// > AWS account when recording live traffic.
        /// ```no_run
        /// make_test!(live: test_name)
        /// ```
        macro_rules! make_test {
            ($name: ident) => {
                make_test!($name, execute);
            };
            (update: $name:ident) => {
                make_test!($name, execute_and_update);
            };
            (live: $name:ident) => {
                make_test!($name, execute_from_live_traffic);
            };
            ($name: ident, $func: ident) => {
                #[traced_test]
                #[tokio::test]
                async fn $name() {
                    crate::test_case::TestEnvironment::from_dir(concat!(
                        "./test-data/default-provider-chain/",
                        stringify!($name)
                    ))
                    .unwrap()
                    .$func(|conf| async {
                        crate::default_provider::credentials::Builder::default()
                            .configure(conf)
                            .build()
                            .await
                    })
                    .await
                }
            };
        }

        make_test!(prefer_environment);
        make_test!(profile_static_keys);
        make_test!(web_identity_token_env);
        make_test!(web_identity_source_profile_no_env);
        make_test!(web_identity_token_invalid_jwt);
        make_test!(web_identity_token_source_profile);
        make_test!(web_identity_token_profile);
        make_test!(profile_name);
        make_test!(profile_overrides_web_identity);
        make_test!(imds_token_fail);

        make_test!(imds_no_iam_role);
        make_test!(imds_default_chain_error);
        make_test!(imds_default_chain_success);
        make_test!(imds_assume_role);
        make_test!(imds_config_with_no_creds);
        make_test!(imds_disabled);
        make_test!(imds_default_chain_retries);

        make_test!(ecs_assume_role);
        make_test!(ecs_credentials);

        #[tokio::test]
        async fn profile_name_override() {
            let (_, conf) =
                TestEnvironment::from_dir("./test-data/default-provider-chain/profile_static_keys")
                    .unwrap()
                    .provider_config()
                    .await;
            let provider = DefaultCredentialsChain::builder()
                .profile_name("secondary")
                .configure(conf)
                .build()
                .await;
            let creds = provider
                .provide_credentials()
                .await
                .expect("creds should load");
            assert_eq!(creds.access_key_id(), "correct_key_secondary");
        }

        #[tokio::test]
        #[traced_test]
        #[cfg(feature = "tcp-connector")]
        async fn no_providers_configured_err() {
            use aws_smithy_async::rt::sleep::TokioSleep;
            use aws_smithy_client::erase::boxclone::BoxCloneService;
            use aws_smithy_client::never::NeverConnected;
            use aws_types::credentials::CredentialsError;
            use aws_types::os_shim_internal::TimeSource;

            tokio::time::pause();
            let conf = ProviderConfig::no_configuration()
                .with_tcp_connector(BoxCloneService::new(NeverConnected::new()))
                .with_time_source(TimeSource::real())
                .with_sleep(TokioSleep::new());
            let provider = DefaultCredentialsChain::builder()
                .configure(conf)
                .build()
                .await;
            let creds = provider
                .provide_credentials()
                .await
                .expect_err("no providers enabled");
            assert!(
                matches!(creds, CredentialsError::CredentialsNotLoaded { .. }),
                "should be NotLoaded: {:?}",
                creds
            )
        }

        #[tokio::test]
        async fn test_returns_default_retry_config_from_empty_profile() {
            let env = Env::from_slice(&[("AWS_CONFIG_FILE", "config")]);
            let fs = Fs::from_slice(&[("config", "[default]\n")]);

            let provider_config = ProviderConfig::no_configuration().with_env(env).with_fs(fs);

            let actual_retry_config = retry_config::default_provider()
                .configure(&provider_config)
                .retry_config()
                .await;

            let expected_retry_config = RetryConfig::new();

            assert_eq!(actual_retry_config, expected_retry_config);
            // This is redundant but it's really important to make sure that
            // we're setting these exact values by default so we check twice
            assert_eq!(actual_retry_config.max_attempts(), 3);
            assert_eq!(actual_retry_config.mode(), RetryMode::Standard);
        }

        #[tokio::test]
        async fn test_no_retry_config_in_empty_profile() {
            let env = Env::from_slice(&[("AWS_CONFIG_FILE", "config")]);
            let fs = Fs::from_slice(&[("config", "[default]\n")]);

            let provider_config = ProviderConfig::no_configuration().with_env(env).with_fs(fs);

            let actual_retry_config = retry_config::default_provider()
                .configure(&provider_config)
                .retry_config()
                .await;

            let expected_retry_config = RetryConfig::new();

            assert_eq!(actual_retry_config, expected_retry_config)
        }

        #[tokio::test]
        async fn test_creation_of_retry_config_from_profile() {
            let env = Env::from_slice(&[("AWS_CONFIG_FILE", "config")]);
            // TODO(https://github.com/awslabs/aws-sdk-rust/issues/247): standard is the default mode;
            // this test would be better if it was setting it to adaptive mode
            // adaptive mode is currently unsupported so that would panic
            let fs = Fs::from_slice(&[(
                "config",
                // If the lines with the vars have preceding spaces, they don't get read
                r#"[default]
max_attempts = 1
retry_mode = standard
            "#,
            )]);

            let provider_config = ProviderConfig::no_configuration().with_env(env).with_fs(fs);

            let actual_retry_config = retry_config::default_provider()
                .configure(&provider_config)
                .retry_config()
                .await;

            let expected_retry_config = RetryConfig::new()
                .with_max_attempts(1)
                .with_retry_mode(RetryMode::Standard);

            assert_eq!(actual_retry_config, expected_retry_config)
        }

        #[tokio::test]
        async fn test_env_retry_config_takes_precedence_over_profile_retry_config() {
            let env = Env::from_slice(&[
                ("AWS_CONFIG_FILE", "config"),
                ("AWS_MAX_ATTEMPTS", "42"),
                ("AWS_RETRY_MODE", "standard"),
            ]);
            // TODO(https://github.com/awslabs/aws-sdk-rust/issues/247) standard is the default mode;
            // this test would be better if it was setting it to adaptive mode
            // adaptive mode is currently unsupported so that would panic
            let fs = Fs::from_slice(&[(
                "config",
                // If the lines with the vars have preceding spaces, they don't get read
                r#"[default]
max_attempts = 88
retry_mode = standard
            "#,
            )]);

            let provider_config = ProviderConfig::no_configuration().with_env(env).with_fs(fs);

            let actual_retry_config = retry_config::default_provider()
                .configure(&provider_config)
                .retry_config()
                .await;

            let expected_retry_config = RetryConfig::new()
                .with_max_attempts(42)
                .with_retry_mode(RetryMode::Standard);

            assert_eq!(actual_retry_config, expected_retry_config)
        }

        #[tokio::test]
        #[should_panic = "failed to parse max attempts set by aws profile: invalid digit found in string"]
        async fn test_invalid_profile_retry_config_panics() {
            let env = Env::from_slice(&[("AWS_CONFIG_FILE", "config")]);
            let fs = Fs::from_slice(&[(
                "config",
                // If the lines with the vars have preceding spaces, they don't get read
                r#"[default]
max_attempts = potato
            "#,
            )]);

            let provider_config = ProviderConfig::no_configuration().with_env(env).with_fs(fs);

            let _ = retry_config::default_provider()
                .configure(&provider_config)
                .retry_config()
                .await;
        }
    }
}
