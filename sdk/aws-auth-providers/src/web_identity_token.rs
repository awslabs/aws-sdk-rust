/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Load Credentials from Web Identity Tokens
//!
//! WebIdentity tokens can be loaded via environment variables, or via profiles:
//!
//! ## Via Environment Variables
//! WebIdentityTokenCredentialProvider will load the following environment variables:
//! - `AWS_WEB_IDENTITY_TOKEN_FILE`: **required**, location to find the token file containing a JWT token
//! - `AWS_ROLE_ARN`: **required**, role ARN to assume
//! - `AWS_IAM_ROLE_SESSION_NAME`: **optional**: Session name to use when assuming the role
//!
//! ## Via Shared Config Profiles
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

use aws_hyper::{DynConnector, StandardClient};
use aws_sdk_sts::Region;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::region::ProvideRegion;

use crate::{must_have_connector, sts_util};
use aws_auth::provider::{AsyncProvideCredentials, BoxFuture, CredentialsError, CredentialsResult};
use std::borrow::Cow;
use std::path::{Path, PathBuf};

const ENV_VAR_TOKEN_FILE: &str = "AWS_WEB_IDENTITY_TOKEN_FILE";
const ENV_VAR_ROLE_ARN: &str = "AWS_ROLE_ARN";
const ENV_VAR_SESSION_NAME: &str = "AWS_ROLE_SESSION_NAME";

/// Credential provider to load credentials from Web Identity  Tokens
///
/// See Module documentation for more details
pub struct WebIdentityTokenCredentialProvider {
    source: Source,
    fs: Fs,
    client: StandardClient,
    region: Option<Region>,
}

impl WebIdentityTokenCredentialProvider {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

enum Source {
    Env(Env),
    Static(WebIdentityTokenRole),
}

/// Hard-coded WebIdentityToken role
#[derive(Debug, Clone)]
pub struct WebIdentityTokenRole {
    pub web_identity_token_file: PathBuf,
    pub role_arn: String,
    pub session_name: String,
}

impl AsyncProvideCredentials for WebIdentityTokenCredentialProvider {
    fn provide_credentials<'a>(&'a self) -> BoxFuture<'a, CredentialsResult>
    where
        Self: 'a,
    {
        Box::pin(self.credentials())
    }
}

impl WebIdentityTokenCredentialProvider {
    fn source(&self) -> Result<Cow<WebIdentityTokenRole>, CredentialsError> {
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
                    .unwrap_or_else(|_| sts_util::default_session_name("web-identity-token"));
                Ok(Cow::Owned(WebIdentityTokenRole {
                    web_identity_token_file: token_file.into(),
                    role_arn,
                    session_name,
                }))
            }
            Source::Static(conf) => Ok(Cow::Borrowed(conf)),
        }
    }
    async fn credentials(&self) -> CredentialsResult {
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
        .await
    }
}

#[derive(Default)]
pub struct Builder {
    source: Option<Source>,
    fs: Fs,
    connector: Option<DynConnector>,
    region: Option<Region>,
}

impl Builder {
    pub fn fs(mut self, fs: Fs) -> Self {
        self.fs = fs;
        self
    }

    pub fn set_fs(&mut self, fs: Fs) -> &mut Self {
        self.fs = fs;
        self
    }

    pub fn env(mut self, env: Env) -> Self {
        self.source = Some(Source::Env(env));
        self
    }

    pub fn static_configuration(mut self, config: WebIdentityTokenRole) -> Self {
        self.source = Some(Source::Static(config));
        self
    }

    pub fn set_env(&mut self, env: Env) -> &mut Self {
        self.source = Some(Source::Env(env));
        self
    }

    pub fn connector(mut self, connector: DynConnector) -> Self {
        self.connector = Some(connector);
        self
    }

    pub fn set_connector(&mut self, connector: Option<DynConnector>) -> &mut Self {
        self.connector = connector;
        self
    }

    pub fn region(mut self, region: &dyn ProvideRegion) -> Self {
        self.region = region.region();
        self
    }

    pub fn set_region(&mut self, region: Option<Region>) -> &mut Self {
        self.region = region;
        self
    }

    pub fn build(self) -> WebIdentityTokenCredentialProvider {
        let connector = self.connector.unwrap_or_else(must_have_connector);
        let client = aws_hyper::Builder::<()>::new()
            .map_connector(|_| connector)
            .build();
        let source = self.source.unwrap_or_else(|| Source::Env(Env::default()));
        WebIdentityTokenCredentialProvider {
            source,
            fs: self.fs,
            client,
            region: self.region,
        }
    }
}

async fn load_credentials(
    fs: &Fs,
    client: &StandardClient,
    region: &dyn ProvideRegion,
    token_file: impl AsRef<Path>,
    role_arn: &str,
    session_name: &str,
) -> CredentialsResult {
    let token = fs
        .read_to_end(token_file)
        .await
        .map_err(|err| CredentialsError::ProviderError(err.into()))?;
    let token = String::from_utf8(token).map_err(|_utf_8_error| {
        CredentialsError::Unhandled("WebIdentityToken was not valid UTF-8".into())
    })?;
    let region = region.region().ok_or_else(|| {
        CredentialsError::InvalidConfiguration(
            "region is required for WebIdentityTokenProvider".into(),
        )
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
    sts_util::into_credentials(resp.credentials, "WebIdentityToken")
}

#[cfg(test)]
mod test {
    use crate::web_identity_token::{
        Builder, ENV_VAR_ROLE_ARN, ENV_VAR_SESSION_NAME, ENV_VAR_TOKEN_FILE,
    };
    use aws_auth::provider::CredentialsError;

    use aws_sdk_sts::Region;
    use aws_types::os_shim_internal::{Env, Fs};

    use std::collections::HashMap;

    #[tokio::test]
    async fn unloaded_provider() {
        // empty environment
        let env = Env::from_slice(&[]);
        let provider = Builder::default()
            .region(&Region::new("us-east-1"))
            .env(env)
            .build();
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
        let provider = Builder::default()
            .region(&Region::new("us-east-1"))
            .env(env)
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
        let fs = Fs::from_map(HashMap::new());
        let provider = Builder::default()
            .region(&Region::new("us-east-1"))
            .fs(fs)
            .env(env)
            .build();
        let err = provider.credentials().await.expect_err("no JWT token");
        match err {
            CredentialsError::ProviderError(_) => { /* ok */ }
            _ => panic!("incorrect error variant"),
        }
    }
}
