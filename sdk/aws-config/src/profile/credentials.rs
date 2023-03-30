/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Profile File Based Credential Providers
//!
//! Profile file based providers combine two pieces:
//!
//! 1. Parsing and resolution of the assume role chain
//! 2. A user-modifiable hashmap of provider name to provider.
//!
//! Profile file based providers first determine the chain of providers that will be used to load
//! credentials. After determining and validating this chain, a `Vec` of providers will be created.
//!
//! Each subsequent provider will provide boostrap providers to the next provider in order to load
//! the final credentials.
//!
//! This module contains two sub modules:
//! - `repr` which contains an abstract representation of a provider chain and the logic to
//! build it from `~/.aws/credentials` and `~/.aws/config`.
//! - `exec` which contains a chain representation of providers to implement passing bootstrapped credentials
//! through a series of providers.

use crate::profile::credentials::exec::named::NamedProviderFactory;
use crate::profile::credentials::exec::{ClientConfiguration, ProviderChain};
use crate::profile::parser::ProfileFileLoadError;
use crate::profile::profile_file::ProfileFiles;
use crate::profile::Profile;
use crate::provider_config::ProviderConfig;
use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_smithy_types::error::display::DisplayErrorContext;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tracing::Instrument;

mod exec;
mod repr;

impl ProvideCredentials for ProfileFileCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.load_credentials())
    }
}

/// AWS Profile based credentials provider
///
/// This credentials provider will load credentials from `~/.aws/config` and `~/.aws/credentials`.
/// The locations of these files are configurable via environment variables, see [below](#location-of-profile-files).
///
/// Generally, this will be constructed via the default provider chain, however, it can be manually
/// constructed with the builder:
/// ```rust,no_run
/// use aws_config::profile::ProfileFileCredentialsProvider;
/// let provider = ProfileFileCredentialsProvider::builder().build();
/// ```
///
/// _Note: Profile providers, when called, will load and parse the profile from the file system
/// only once. Parsed file contents will be cached indefinitely._
///
/// This provider supports several different credentials formats:
/// ### Credentials defined explicitly within the file
/// ```ini
/// [default]
/// aws_access_key_id = 123
/// aws_secret_access_key = 456
/// ```
///
/// ### Assume Role Credentials loaded from a credential source
/// ```ini
/// [default]
/// role_arn = arn:aws:iam::123456789:role/RoleA
/// credential_source = Environment
/// ```
///
/// NOTE: Currently only the `Environment` credential source is supported although it is possible to
/// provide custom sources:
/// ```no_run
/// use aws_credential_types::provider::{self, future, ProvideCredentials};
/// use aws_config::profile::ProfileFileCredentialsProvider;
/// #[derive(Debug)]
/// struct MyCustomProvider;
/// impl MyCustomProvider {
///     async fn load_credentials(&self) -> provider::Result {
///         todo!()
///     }
/// }
///
/// impl ProvideCredentials for MyCustomProvider {
///   fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials where Self: 'a {
///         future::ProvideCredentials::new(self.load_credentials())
///     }
/// }
/// # if cfg!(any(feature = "rustls", feature = "native-tls")) {
/// let provider = ProfileFileCredentialsProvider::builder()
///     .with_custom_provider("Custom", MyCustomProvider)
///     .build();
/// }
/// ```
///
/// ### Assume role credentials from a source profile
/// ```ini
/// [default]
/// role_arn = arn:aws:iam::123456789:role/RoleA
/// source_profile = base
///
/// [profile base]
/// aws_access_key_id = 123
/// aws_secret_access_key = 456
/// ```
///
/// Other more complex configurations are possible, consult `test-data/assume-role-tests.json`.
///
/// ### Credentials loaded from an external process
/// ```ini
/// [default]
/// credential_process = /opt/bin/awscreds-custom --username helen
/// ```
///
/// An external process can be used to provide credentials.
///
/// ### Loading Credentials from SSO
/// ```ini
/// [default]
/// sso_start_url = https://example.com/start
/// sso_region = us-east-2
/// sso_account_id = 123456789011
/// sso_role_name = readOnly
/// region = us-west-2
/// ```
///
/// SSO can also be used as a source profile for assume role chains.
///
#[doc = include_str!("location_of_profile_files.md")]
#[derive(Debug)]
pub struct ProfileFileCredentialsProvider {
    factory: NamedProviderFactory,
    client_config: ClientConfiguration,
    provider_config: ProviderConfig,
}

impl ProfileFileCredentialsProvider {
    /// Builder for this credentials provider
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn load_credentials(&self) -> provider::Result {
        let inner_provider = build_provider_chain(&self.provider_config, &self.factory)
            .await
            .map_err(|err| match err {
                ProfileFileError::NoProfilesDefined
                | ProfileFileError::ProfileDidNotContainCredentials { .. } => {
                    CredentialsError::not_loaded(err)
                }
                _ => CredentialsError::invalid_configuration(format!(
                    "ProfileFile provider could not be built: {}",
                    &err
                )),
            })?;
        let mut creds = match inner_provider
            .base()
            .provide_credentials()
            .instrument(tracing::debug_span!("load_base_credentials"))
            .await
        {
            Ok(creds) => {
                tracing::info!(creds = ?creds, "loaded base credentials");
                creds
            }
            Err(e) => {
                tracing::warn!(error = %DisplayErrorContext(&e), "failed to load base credentials");
                return Err(CredentialsError::provider_error(e));
            }
        };
        for provider in inner_provider.chain().iter() {
            let next_creds = provider
                .credentials(creds, &self.client_config)
                .instrument(tracing::debug_span!("load_assume_role", provider = ?provider))
                .await;
            match next_creds {
                Ok(next_creds) => {
                    tracing::info!(creds = ?next_creds, "loaded assume role credentials");
                    creds = next_creds
                }
                Err(e) => {
                    tracing::warn!(provider = ?provider, "failed to load assume role credentials");
                    return Err(CredentialsError::provider_error(e));
                }
            }
        }
        Ok(creds)
    }
}

/// An Error building a Credential source from an AWS Profile
#[derive(Debug)]
#[non_exhaustive]
pub enum ProfileFileError {
    /// The profile was not a valid AWS profile
    #[non_exhaustive]
    InvalidProfile(ProfileFileLoadError),

    /// No profiles existed (the profile was empty)
    #[non_exhaustive]
    NoProfilesDefined,

    /// The profile did not contain any credential information
    #[non_exhaustive]
    ProfileDidNotContainCredentials {
        /// The name of the profile
        profile: String,
    },

    /// The profile contained an infinite loop of `source_profile` references
    #[non_exhaustive]
    CredentialLoop {
        /// Vec of profiles leading to the loop
        profiles: Vec<String>,
        /// The next profile that caused the loop
        next: String,
    },

    /// The profile was missing a credential source
    #[non_exhaustive]
    MissingCredentialSource {
        /// The name of the profile
        profile: String,
        /// Error message
        message: Cow<'static, str>,
    },
    /// The profile contained an invalid credential source
    #[non_exhaustive]
    InvalidCredentialSource {
        /// The name of the profile
        profile: String,
        /// Error message
        message: Cow<'static, str>,
    },
    /// The profile referred to a another profile by name that was not defined
    #[non_exhaustive]
    MissingProfile {
        /// The name of the profile
        profile: String,
        /// Error message
        message: Cow<'static, str>,
    },
    /// The profile referred to `credential_source` that was not defined
    #[non_exhaustive]
    UnknownProvider {
        /// The name of the provider
        name: String,
    },
}

impl ProfileFileError {
    fn missing_field(profile: &Profile, field: &'static str) -> Self {
        ProfileFileError::MissingProfile {
            profile: profile.name().to_string(),
            message: format!("`{}` was missing", field).into(),
        }
    }
}

impl Error for ProfileFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileError::InvalidProfile(err) => Some(err),
            _ => None,
        }
    }
}

impl Display for ProfileFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileError::InvalidProfile(err) => {
                write!(f, "invalid profile: {}", err)
            }
            ProfileFileError::CredentialLoop { profiles, next } => write!(
                f,
                "profile formed an infinite loop. first we loaded {:?}, \
            then attempted to reload {}",
                profiles, next
            ),
            ProfileFileError::MissingCredentialSource { profile, message } => {
                write!(f, "missing credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::InvalidCredentialSource { profile, message } => {
                write!(f, "invalid credential source in `{}`: {}", profile, message)
            }
            ProfileFileError::MissingProfile { profile, message } => {
                write!(f, "profile `{}` was not defined: {}", profile, message)
            }
            ProfileFileError::UnknownProvider { name } => write!(
                f,
                "profile referenced `{}` provider but that provider is not supported",
                name
            ),
            ProfileFileError::NoProfilesDefined => write!(f, "No profiles were defined"),
            ProfileFileError::ProfileDidNotContainCredentials { profile } => write!(
                f,
                "profile `{}` did not contain credential information",
                profile
            ),
        }
    }
}

/// Builder for [`ProfileFileCredentialsProvider`]
#[derive(Debug, Default)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    profile_override: Option<String>,
    profile_files: Option<ProfileFiles>,
    custom_providers: HashMap<Cow<'static, str>, Arc<dyn ProvideCredentials>>,
}

impl Builder {
    /// Override the configuration for the [`ProfileFileCredentialsProvider`]
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn test() {
    /// use aws_config::profile::ProfileFileCredentialsProvider;
    /// use aws_config::provider_config::ProviderConfig;
    /// let provider = ProfileFileCredentialsProvider::builder()
    ///     .configure(&ProviderConfig::with_default_region().await)
    ///     .build();
    /// # }
    /// ```
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    /// Adds a custom credential source
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use aws_credential_types::provider::{self, future, ProvideCredentials};
    /// use aws_config::profile::ProfileFileCredentialsProvider;
    /// #[derive(Debug)]
    /// struct MyCustomProvider;
    /// impl MyCustomProvider {
    ///     async fn load_credentials(&self) -> provider::Result {
    ///         todo!()
    ///     }
    /// }
    ///
    /// impl ProvideCredentials for MyCustomProvider {
    ///   fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials where Self: 'a {
    ///         future::ProvideCredentials::new(self.load_credentials())
    ///     }
    /// }
    ///
    /// # if cfg!(any(feature = "rustls", feature = "native-tls")) {
    /// let provider = ProfileFileCredentialsProvider::builder()
    ///     .with_custom_provider("Custom", MyCustomProvider)
    ///     .build();
    /// # }
    /// ```
    pub fn with_custom_provider(
        mut self,
        name: impl Into<Cow<'static, str>>,
        provider: impl ProvideCredentials + 'static,
    ) -> Self {
        self.custom_providers
            .insert(name.into(), Arc::new(provider));
        self
    }

    /// Override the profile name used by the [`ProfileFileCredentialsProvider`]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Set the profile file that should be used by the [`ProfileFileCredentialsProvider`]
    pub fn profile_files(mut self, profile_files: ProfileFiles) -> Self {
        self.profile_files = Some(profile_files);
        self
    }

    /// Builds a [`ProfileFileCredentialsProvider`]
    pub fn build(self) -> ProfileFileCredentialsProvider {
        let build_span = tracing::debug_span!("build_profile_provider");
        let _enter = build_span.enter();
        let conf = self
            .provider_config
            .unwrap_or_default()
            .with_profile_config(self.profile_files, self.profile_override);
        let mut named_providers = self.custom_providers.clone();
        named_providers
            .entry("Environment".into())
            .or_insert_with(|| {
                Arc::new(crate::environment::credentials::EnvironmentVariableCredentialsProvider::new_with_env(
                    conf.env(),
                ))
            });

        named_providers
            .entry("Ec2InstanceMetadata".into())
            .or_insert_with(|| {
                Arc::new(
                    crate::imds::credentials::ImdsCredentialsProvider::builder()
                        .configure(&conf)
                        .build(),
                )
            });

        named_providers
            .entry("EcsContainer".into())
            .or_insert_with(|| {
                Arc::new(
                    crate::ecs::EcsCredentialsProvider::builder()
                        .configure(&conf)
                        .build(),
                )
            });
        let factory = exec::named::NamedProviderFactory::new(named_providers);
        let core_client = conf.sts_client();

        ProfileFileCredentialsProvider {
            factory,
            client_config: ClientConfiguration {
                sts_client: core_client,
                region: conf.region(),
            },
            provider_config: conf,
        }
    }
}

async fn build_provider_chain(
    provider_config: &ProviderConfig,
    factory: &NamedProviderFactory,
) -> Result<ProviderChain, ProfileFileError> {
    let profile_set = provider_config
        .try_profile()
        .await
        .map_err(|parse_err| ProfileFileError::InvalidProfile(parse_err.clone()))?;
    let repr = repr::resolve_chain(profile_set)?;
    tracing::info!(chain = ?repr, "constructed abstract provider from config file");
    exec::ProviderChain::from_repr(provider_config, repr, factory)
}

#[cfg(test)]
mod test {
    use tracing_test::traced_test;

    use crate::profile::credentials::Builder;
    use crate::test_case::TestEnvironment;

    macro_rules! make_test {
        ($name: ident) => {
            #[traced_test]
            #[tokio::test]
            async fn $name() {
                TestEnvironment::from_dir(concat!(
                    "./test-data/profile-provider/",
                    stringify!($name)
                ))
                .await
                .unwrap()
                .execute(|conf| async move { Builder::default().configure(&conf).build() })
                .await
            }
        };
    }

    make_test!(e2e_assume_role);
    make_test!(empty_config);
    make_test!(retry_on_error);
    make_test!(invalid_config);
    make_test!(region_override);
    make_test!(credential_process);
    make_test!(credential_process_failure);
    make_test!(credential_process_invalid);
}
