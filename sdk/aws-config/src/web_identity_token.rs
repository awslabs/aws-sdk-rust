/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Load Credentials from Web Identity Tokens
//!
//! Web identity tokens can be loaded from file. The path may be set in one of three ways:
//! 1. [Environment Variables](#environment-variable-configuration)
//! 2. [AWS profile](#aws-profile-configuration) defined in `~/.aws/config`
//! 3. Static configuration via [`static_configuration`](Builder::static_configuration)
//!
//! **Note:** [WebIdentityTokenCredentialsProvider] is part of the [default provider chain](crate::default_provider).
//! Unless you need specific behavior or configuration overrides, it is recommended to use the
//! default chain instead of using this provider directly. This client should be considered a "low level"
//! client as it does not include caching or profile-file resolution when used in isolation.
//!
//! ## Environment Variable Configuration
//! WebIdentityTokenCredentialProvider will load the following environment variables:
//! - `AWS_WEB_IDENTITY_TOKEN_FILE`: **required**, location to find the token file containing a JWT token
//! - `AWS_ROLE_ARN`: **required**, role ARN to assume
//! - `AWS_IAM_ROLE_SESSION_NAME`: **optional**: Session name to use when assuming the role
//!
//! ## AWS Profile Configuration
//! **Note:** Configuration of the web identity token provider via a shared profile is only supported
//! when using the [`ProfileFileCredentialsProvider`](crate::profile::credentials).
//!
//! Web identity token credentials can be loaded from `~/.aws/config` in two ways:
//! 1. Directly:
//!   ```ini
//!   [profile default]
//!   role_arn = arn:aws:iam::1234567890123:role/RoleA
//!   web_identity_token_file = /token.jwt
//!   ```
//!
//! 2. As a source profile for another role:
//!
//!   ```ini
//!   [profile default]
//!   role_arn = arn:aws:iam::123456789:role/RoleA
//!   source_profile = base
//!
//!   [profile base]
//!   role_arn = arn:aws:iam::123456789012:role/s3-reader
//!   web_identity_token_file = /token.jwt
//!   ```
//!
//! # Examples
//! Web Identity Token providers are part of the [default chain](crate::default_provider::credentials).
//! However, they may be directly constructed if you don't want to use the default provider chain.
//! Unless overridden with [`static_configuration`](Builder::static_configuration), the provider will
//! load configuration from environment variables.
//!
//! ```rust
//! # async fn test() {
//! use aws_config::web_identity_token::WebIdentityTokenCredentialsProvider;
//! use aws_config::provider_config::ProviderConfig;
//! let provider = WebIdentityTokenCredentialsProvider::builder()
//!     .configure(&ProviderConfig::with_default_region().await)
//!     .build();
//! # }
//! ```
use aws_sdk_sts::Region;
use aws_types::os_shim_internal::{Env, Fs};

use crate::connector::expect_connector;
use crate::provider_config::ProviderConfig;
use crate::sts;
use aws_types::credentials::{self, future, CredentialsError, ProvideCredentials};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use tracing::Instrument;

const ENV_VAR_TOKEN_FILE: &str = "AWS_WEB_IDENTITY_TOKEN_FILE";
const ENV_VAR_ROLE_ARN: &str = "AWS_ROLE_ARN";
const ENV_VAR_SESSION_NAME: &str = "AWS_ROLE_SESSION_NAME";

/// Credential provider to load credentials from Web Identity  Tokens
///
/// See Module documentation for more details
#[derive(Debug)]
pub struct WebIdentityTokenCredentialsProvider {
    source: Source,
    fs: Fs,
    client: aws_hyper::StandardClient,
    region: Option<Region>,
}

impl WebIdentityTokenCredentialsProvider {
    /// Builder for this credentials provider
    pub fn builder() -> Builder {
        Builder::default()
    }
}

#[derive(Debug)]
enum Source {
    Env(Env),
    Static(StaticConfiguration),
}

/// Statically configured WebIdentityToken configuration
#[derive(Debug, Clone)]
pub struct StaticConfiguration {
    /// Location of the file containing the web identity token
    pub web_identity_token_file: PathBuf,

    /// RoleArn to assume
    pub role_arn: String,

    /// Session name to use when assuming the role
    pub session_name: String,
}

impl ProvideCredentials for WebIdentityTokenCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

impl WebIdentityTokenCredentialsProvider {
    fn source(&self) -> Result<Cow<StaticConfiguration>, CredentialsError> {
        match &self.source {
            Source::Env(env) => {
                let token_file = env
                    .get(ENV_VAR_TOKEN_FILE)
                    .map_err(|_| CredentialsError::CredentialsNotLoaded)?;
                let role_arn = env.get(ENV_VAR_ROLE_ARN).map_err(|_| {
                    CredentialsError::InvalidConfiguration(
                        "AWS_ROLE_ARN environment variable must be set".into(),
                    )
                })?;
                let session_name = env
                    .get(ENV_VAR_SESSION_NAME)
                    .unwrap_or_else(|_| sts::util::default_session_name("web-identity-token"));
                Ok(Cow::Owned(StaticConfiguration {
                    web_identity_token_file: token_file.into(),
                    role_arn,
                    session_name,
                }))
            }
            Source::Static(conf) => Ok(Cow::Borrowed(conf)),
        }
    }
    async fn credentials(&self) -> credentials::Result {
        let conf = self.source()?;
        load_credentials(
            &self.fs,
            &self.client,
            &self.region.as_ref().cloned().ok_or_else(|| {
                CredentialsError::InvalidConfiguration(
                    "region is required for WebIdentityTokenProvider".into(),
                )
            })?,
            &conf.web_identity_token_file,
            &conf.role_arn,
            &conf.session_name,
        )
        .instrument(tracing::info_span!(
            "load_credentials",
            provider = "WebIdentityToken"
        ))
        .await
    }
}

/// Builder for [`WebIdentityTokenCredentialsProvider`](WebIdentityTokenCredentialsProvider)
#[derive(Default)]
pub struct Builder {
    source: Option<Source>,
    config: Option<ProviderConfig>,
}

impl Builder {
    /// Configure generic options of the [WebIdentityTokenCredentialsProvider]
    ///
    /// # Examples
    /// ```rust
    /// # async fn test() {
    /// use aws_config::web_identity_token::WebIdentityTokenCredentialsProvider;
    /// use aws_config::provider_config::ProviderConfig;
    /// let provider = WebIdentityTokenCredentialsProvider::builder()
    ///     .configure(&ProviderConfig::with_default_region().await)
    ///     .build();
    /// # }
    /// ```
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.config = Some(provider_config.clone());
        self
    }

    /// Configure this builder to use  [`StaticConfiguration`](StaticConfiguration)
    ///
    /// WebIdentityToken providers load credentials from the file system. The file system path used
    /// may either determine be loaded from environment variables (default), or via a statically
    /// configured path.
    pub fn static_configuration(mut self, config: StaticConfiguration) -> Self {
        self.source = Some(Source::Static(config));
        self
    }

    /// Build a [`WebIdentityTokenCredentialsProvider`]
    ///
    /// ## Panics
    /// If no connector has been enabled via crate features and no connector has been provided via the
    /// builder, this function will panic.
    pub fn build(self) -> WebIdentityTokenCredentialsProvider {
        let conf = self.config.unwrap_or_default();
        let connector = expect_connector(conf.default_connector());
        let client = aws_hyper::Client::new(connector);
        let source = self.source.unwrap_or_else(|| Source::Env(conf.env()));
        WebIdentityTokenCredentialsProvider {
            source,
            fs: conf.fs(),
            client,
            region: conf.region(),
        }
    }
}

async fn load_credentials(
    fs: &Fs,
    client: &aws_hyper::StandardClient,
    region: &Region,
    token_file: impl AsRef<Path>,
    role_arn: &str,
    session_name: &str,
) -> credentials::Result {
    let token = fs
        .read_to_end(token_file)
        .await
        .map_err(|err| CredentialsError::ProviderError(err.into()))?;
    let token = String::from_utf8(token).map_err(|_utf_8_error| {
        CredentialsError::Unhandled("WebIdentityToken was not valid UTF-8".into())
    })?;
    let conf = aws_sdk_sts::Config::builder()
        .region(region.clone())
        .build();

    let operation = aws_sdk_sts::operation::AssumeRoleWithWebIdentity::builder()
        .role_arn(role_arn)
        .role_session_name(session_name)
        .web_identity_token(token)
        .build()
        .expect("valid operation")
        .make_operation(&conf)
        .expect("valid operation");
    let resp = client.call(operation).await.map_err(|sdk_error| {
        tracing::warn!(error = ?sdk_error, "sts returned an error assuming web identity role");
        CredentialsError::ProviderError(sdk_error.into())
    })?;
    sts::util::into_credentials(resp.credentials, "WebIdentityToken")
}

#[cfg(test)]
mod test {
    use crate::web_identity_token::{
        Builder, ENV_VAR_ROLE_ARN, ENV_VAR_SESSION_NAME, ENV_VAR_TOKEN_FILE,
    };

    use aws_sdk_sts::Region;
    use aws_types::os_shim_internal::{Env, Fs};

    use crate::provider_config::ProviderConfig;
    use crate::test_case::no_traffic_connector;
    use aws_types::credentials::CredentialsError;
    use std::collections::HashMap;

    #[tokio::test]
    async fn unloaded_provider() {
        // empty environment
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[]))
            .with_connector(no_traffic_connector())
            .with_region(Some(Region::from_static("us-east-1")));

        let provider = Builder::default().configure(&conf).build();
        let err = provider
            .credentials()
            .await
            .expect_err("should fail, provider not loaded");
        match err {
            CredentialsError::CredentialsNotLoaded => { /* ok */ }
            _ => panic!("incorrect error variant"),
        }
    }

    #[tokio::test]
    async fn missing_env_var() {
        let env = Env::from_slice(&[(ENV_VAR_TOKEN_FILE, "/token.jwt")]);
        let region = Some(Region::new("us-east-1"));
        let provider = Builder::default()
            .configure(
                &ProviderConfig::empty()
                    .with_region(region)
                    .with_env(env)
                    .with_connector(no_traffic_connector()),
            )
            .build();
        let err = provider
            .credentials()
            .await
            .expect_err("should fail, provider not loaded");
        assert!(
            format!("{}", err).contains("AWS_ROLE_ARN"),
            "`{}` did not contain expected string",
            err
        );
        match err {
            CredentialsError::InvalidConfiguration(_) => { /* ok */ }
            _ => panic!("incorrect error variant"),
        }
    }

    #[tokio::test]
    async fn fs_missing_file() {
        let env = Env::from_slice(&[
            (ENV_VAR_TOKEN_FILE, "/token.jwt"),
            (ENV_VAR_ROLE_ARN, "arn:aws:iam::123456789123:role/test-role"),
            (ENV_VAR_SESSION_NAME, "test-session"),
        ]);
        let fs = Fs::from_raw_map(HashMap::new());
        let provider = Builder::default()
            .configure(
                &ProviderConfig::empty()
                    .with_connector(no_traffic_connector())
                    .with_region(Some(Region::new("us-east-1")))
                    .with_env(env)
                    .with_fs(fs),
            )
            .build();
        let err = provider.credentials().await.expect_err("no JWT token");
        match err {
            CredentialsError::ProviderError(_) => { /* ok */ }
            _ => panic!("incorrect error variant"),
        }
    }
}
