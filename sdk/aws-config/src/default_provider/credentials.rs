/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use aws_types::credentials;
use std::borrow::Cow;

use aws_types::credentials::{future, ProvideCredentials};
use tracing::Instrument;

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

    async fn credentials(&self) -> credentials::Result {
        self.0
            .provide_credentials()
            .instrument(tracing::info_span!("provide_credentials", provider = %"default_chain"))
            .await
    }
}

impl ProvideCredentials for DefaultCredentialsChain {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
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

    make_test!(sso_assume_role);
    make_test!(sso_no_token_file);

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
