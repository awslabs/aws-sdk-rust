/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
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
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use aws_types::credentials::{self, future, CredentialsError, ProvideCredentials};
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::region::Region;
use tracing::Instrument;

use crate::connector::expect_connector;
use crate::meta::region::ProvideRegion;
use crate::profile::credentials::exec::named::NamedProviderFactory;
use crate::profile::credentials::exec::{ClientConfiguration, ProviderChain};
use crate::profile::parser::ProfileParseError;
use crate::provider_config::ProviderConfig;
use smithy_client::erase::DynConnector;

mod exec;
mod repr;

impl ProvideCredentials for ProfileFileCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.load_credentials().instrument(tracing::info_span!(
            "load_credentials",
            provider = "Profile"
        )))
    }
}

/// AWS Profile based credentials provider
///
/// This credentials provider will load credentials from `~/.aws/config` and `~/.aws/credentials`.
/// The locations of these files are configurable, see [`profile::load`](crate::profile::load).
///
/// Generally, this will be constructed via the default provider chain, however, it can be manually
/// constructed with the builder:
/// ```rust,no_run
/// use aws_config::profile::ProfileFileCredentialsProvider;
/// let provider = ProfileFileCredentialsProvider::builder().build();
/// ```
///
/// **Note:** Profile providers to not implement any caching. They will reload and reparse the profile
/// from the file system when called. See [lazy_caching](crate::meta::credentials::LazyCachingCredentialsProvider) for
/// more information about caching.
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
/// ```rust
/// use aws_types::credentials::{self, ProvideCredentials, future};
/// use aws_config::profile::ProfileFileCredentialsProvider;
/// #[derive(Debug)]
/// struct MyCustomProvider;
/// impl MyCustomProvider {
///     async fn load_credentials(&self) -> credentials::Result {
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
#[derive(Debug)]
pub struct ProfileFileCredentialsProvider {
    factory: NamedProviderFactory,
    client_config: ClientConfiguration,
    fs: Fs,
    env: Env,
    region: Option<Region>,
    connector: DynConnector,
    profile_override: Option<String>,
}

impl ProfileFileCredentialsProvider {
    /// Builder for this credentials provider
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn load_credentials(&self) -> credentials::Result {
        // 1. grab a read lock, use it to see if the base profile has already been loaded
        // 2. If it's loaded, great, lets use it.
        //    If not, upgrade to a write lock and use that to load the profile file.
        // 3. Finally, downgrade to ensure no one swapped in the intervening time, then use try_load()
        //    to pull the new state.
        let profile = build_provider_chain(
            &self.fs,
            &self.env,
            &self.region,
            &self.connector,
            &self.factory,
            self.profile_override.as_deref(),
        )
        .await;
        let inner_provider = profile.map_err(|err| match err {
            ProfileFileError::NoProfilesDefined
            | ProfileFileError::ProfileDidNotContainCredentials { .. } => {
                CredentialsError::CredentialsNotLoaded
            }
            _ => CredentialsError::InvalidConfiguration(
                format!("ProfileFile provider could not be built: {}", &err).into(),
            ),
        })?;
        let mut creds = match inner_provider
            .base()
            .provide_credentials()
            .instrument(tracing::info_span!("load_base_credentials"))
            .await
        {
            Ok(creds) => {
                tracing::info!(creds = ?creds, "loaded base credentials");
                creds
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to load base credentials");
                return Err(CredentialsError::ProviderError(e.into()));
            }
        };
        for provider in inner_provider.chain().iter() {
            let next_creds = provider
                .credentials(creds, &self.client_config)
                .instrument(tracing::info_span!("load_assume_role", provider = ?provider))
                .await;
            match next_creds {
                Ok(next_creds) => {
                    tracing::info!(creds = ?next_creds, "loaded assume role credentials");
                    creds = next_creds
                }
                Err(e) => {
                    tracing::warn!(provider = ?provider, "failed to load assume role credentials");
                    return Err(CredentialsError::ProviderError(e.into()));
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
    CouldNotParseProfile(ProfileParseError),

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

impl Display for ProfileFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => {
                write!(f, "could not parse profile file: {}", err)
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

impl Error for ProfileFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ProfileFileError::CouldNotParseProfile(err) => Some(err),
            _ => None,
        }
    }
}

/// Builder for [`ProfileFileCredentialsProvider`]
#[derive(Default)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    profile_override: Option<String>,
    custom_providers: HashMap<Cow<'static, str>, Arc<dyn ProvideCredentials>>,
}

impl Builder {
    /// Override the configuration for the [`ProfileFileCredentialsProvider`]
    ///
    /// # Examples
    /// ```rust
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
    /// ```rust
    /// use aws_types::credentials::{self, ProvideCredentials, future};
    /// use aws_config::profile::ProfileFileCredentialsProvider;
    /// #[derive(Debug)]
    /// struct MyCustomProvider;
    /// impl MyCustomProvider {
    ///     async fn load_credentials(&self) -> credentials::Result {
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

    /// Builds a [`ProfileFileCredentialsProvider`]
    pub fn build(self) -> ProfileFileCredentialsProvider {
        let build_span = tracing::info_span!("build_profile_provider");
        let _enter = build_span.enter();
        let conf = self.provider_config.unwrap_or_default();
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
        // TODO: ECS, IMDS, and other named providers
        let factory = exec::named::NamedProviderFactory::new(named_providers);
        let connector = expect_connector(conf.default_connector());
        let core_client = aws_hyper::Client::new(connector.clone());

        ProfileFileCredentialsProvider {
            factory,
            client_config: ClientConfiguration {
                core_client,
                region: conf.region(),
            },
            fs: conf.fs(),
            env: conf.env(),
            region: conf.region(),
            connector,
            profile_override: self.profile_override,
        }
    }
}

async fn build_provider_chain(
    fs: &Fs,
    env: &Env,
    region: &dyn ProvideRegion,
    connector: &DynConnector,
    factory: &NamedProviderFactory,
    profile_override: Option<&str>,
) -> Result<ProviderChain, ProfileFileError> {
    let profile_set = super::parser::load(&fs, &env).await.map_err(|err| {
        tracing::warn!(err = %err, "failed to parse profile");
        ProfileFileError::CouldNotParseProfile(err)
    })?;
    let repr = repr::resolve_chain(&profile_set, profile_override)?;
    tracing::info!(chain = ?repr, "constructed abstract provider from config file");
    exec::ProviderChain::from_repr(fs.clone(), connector, region.region().await, repr, &factory)
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
}
