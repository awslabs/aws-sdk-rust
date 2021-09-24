/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Default Provider chains for [`region`](default_provider::region) and [`credentials`](default_provider::credentials).
//!
//! Unless specific configuration is required, these should be constructed via [`ConfigLoader`](crate::ConfigLoader).
//!
//!

/// Default region provider chain
pub mod region {

    use crate::environment::region::EnvironmentVariableRegionProvider;
    use crate::meta::region::{ProvideRegion, RegionProviderChain};
    use crate::{imds, profile};

    use crate::provider_config::ProviderConfig;

    use aws_types::region::Region;

    /// Default Region Provider chain
    ///
    /// This provider will check the following sources in order:
    /// 1. [Environment variables](EnvironmentVariableRegionProvider)
    /// 2. [Profile file](crate::profile::region::ProfileFileRegionProvider)
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

/// Default credentials provider chain
pub mod credentials {
    use crate::environment::credentials::EnvironmentVariableCredentialsProvider;
    use crate::meta::credentials::{CredentialsProviderChain, LazyCachingCredentialsProvider};
    use crate::meta::region::ProvideRegion;
    use aws_types::credentials::{future, ProvideCredentials};

    use crate::provider_config::ProviderConfig;

    use std::borrow::Cow;

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
    ///
    /// The outer provider is wrapped in a refreshing cache.
    ///
    /// More providers are a work in progress.
    ///
    /// # Examples
    /// Create a default chain with a custom region:
    /// ```rust
    /// use aws_types::region::Region;
    /// use aws_config::default_provider::credentials::DefaultCredentialsChain;
    /// let credentials_provider = DefaultCredentialsChain::builder()
    ///     .region(Region::new("us-west-1"))
    ///     .build();
    /// ```
    ///
    /// Create a default chain with no overrides:
    /// ```rust
    /// use aws_config::default_provider::credentials::DefaultCredentialsChain;
    /// let credentials_provider = DefaultCredentialsChain::builder().build();
    /// ```
    ///
    /// Create a default chain that uses a different profile:
    /// ```rust
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

            let profile_provider = self.profile_file_builder.configure(&conf).build();
            let env_provider = EnvironmentVariableCredentialsProvider::new_with_env(conf.env());
            let web_identity_token_provider = self.web_identity_builder.configure(&conf).build();
            let imds_provider = self.imds_builder.configure(&conf).build();

            let provider_chain = CredentialsProviderChain::first_try("Environment", env_provider)
                .or_else("Profile", profile_provider)
                .or_else("WebIdentityToken", web_identity_token_provider)
                .or_else("Ec2InstanceMetadata", imds_provider);
            let cached_provider = self.credential_cache.configure(&conf).load(provider_chain);
            DefaultCredentialsChain(cached_provider.build())
        }
    }

    #[cfg(test)]
    mod test {

        /// Test generation macro
        ///
        /// # Examples
        /// **Run the test case in `test-data/default-provider-chain/test_name`
        /// ```rust
        /// make_test!(test_name);
        /// ```
        ///
        /// **Update (responses are replayed but new requests are recorded) the test case**:
        /// ```rust
        /// make_test!(update: test_name)
        /// ```
        ///
        /// **Run the test case against a real HTTPS connection:**
        /// > Note: Be careful to remove sensitive information before commiting. Always use a temporary
        /// > AWS account when recording live traffic.
        /// ```rust
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

        use crate::default_provider::credentials::DefaultCredentialsChain;
        use crate::test_case::TestEnvironment;
        use aws_types::credentials::ProvideCredentials;
        use tracing_test::traced_test;

        make_test!(prefer_environment);
        make_test!(profile_static_keys);
        make_test!(web_identity_token_env);
        make_test!(web_identity_source_profile_no_env);
        make_test!(web_identity_token_invalid_jwt);
        make_test!(web_identity_token_source_profile);
        make_test!(web_identity_token_profile);
        make_test!(profile_overrides_web_identity);
        make_test!(imds_token_fail);

        make_test!(imds_no_iam_role);
        make_test!(imds_default_chain_error);
        make_test!(imds_default_chain_success);
        make_test!(imds_assume_role);
        make_test!(imds_disabled);
        make_test!(imds_default_chain_retries);

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
    }
}
