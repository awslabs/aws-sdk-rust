/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! SSO Credentials Provider
//!
//! This credentials provider enables loading credentials from `~/.aws/sso/cache`. For more information,
//! see [Using AWS SSO Credentials](https://docs.aws.amazon.com/toolkit-for-vscode/latest/userguide/sso-credentials.html)
//!
//! This provider is included automatically when profiles are loaded.

use crate::fs_util::{home_dir, Os};
use crate::json_credentials::{json_parse_loop, InvalidJsonCredentials};
use crate::provider_config::ProviderConfig;

use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_credential_types::Credentials;
use aws_sdk_sso::middleware::DefaultMiddleware as SsoMiddleware;
use aws_sdk_sso::operation::get_role_credentials::GetRoleCredentialsInput;
use aws_sdk_sso::types::RoleCredentials;
use aws_smithy_client::erase::DynConnector;
use aws_smithy_json::deserialize::Token;
use aws_smithy_types::date_time::Format;
use aws_smithy_types::DateTime;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::region::Region;

use std::convert::TryInto;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;

use ring::digest;
use zeroize::Zeroizing;

impl crate::provider_config::ProviderConfig {
    pub(crate) fn sso_client(
        &self,
    ) -> aws_smithy_client::Client<aws_smithy_client::erase::DynConnector, SsoMiddleware> {
        use crate::connector::expect_connector;

        let mut client_builder = aws_smithy_client::Client::builder()
            .connector(expect_connector(self.connector(&Default::default())))
            .middleware(SsoMiddleware::default());
        client_builder.set_sleep_impl(self.sleep());
        client_builder.build()
    }
}

/// SSO Credentials Provider
///
/// _Note: This provider is part of the default credentials chain and is integrated with the profile-file provider._
///
/// This credentials provider will use cached SSO tokens stored in `~/.aws/sso/cache/<hash>.json`.
/// `<hash>` is computed based on the configured [`start_url`](Builder::start_url).
#[derive(Debug)]
pub struct SsoCredentialsProvider {
    fs: Fs,
    env: Env,
    sso_config: SsoConfig,
    client: aws_smithy_client::Client<DynConnector, SsoMiddleware>,
}

impl SsoCredentialsProvider {
    /// Creates a builder for [`SsoCredentialsProvider`]
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub(crate) fn new(provider_config: &ProviderConfig, sso_config: SsoConfig) -> Self {
        let fs = provider_config.fs();
        let env = provider_config.env();

        SsoCredentialsProvider {
            fs,
            env,
            client: provider_config.sso_client(),
            sso_config,
        }
    }

    async fn credentials(&self) -> provider::Result {
        load_sso_credentials(&self.sso_config, &self.client, &self.env, &self.fs).await
    }
}

impl ProvideCredentials for SsoCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

/// Builder for [`SsoCredentialsProvider`]
#[derive(Default, Debug, Clone)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    account_id: Option<String>,
    role_name: Option<String>,
    start_url: Option<String>,
    region: Option<Region>,
}

impl Builder {
    /// Create a new builder for [`SsoCredentialsProvider`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the configuration used for this provider
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    /// Set the account id used for SSO
    pub fn account_id(mut self, account_id: impl Into<String>) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    /// Set the role name used for SSO
    pub fn role_name(mut self, role_name: impl Into<String>) -> Self {
        self.role_name = Some(role_name.into());
        self
    }

    /// Set the start URL used for SSO
    pub fn start_url(mut self, start_url: impl Into<String>) -> Self {
        self.start_url = Some(start_url.into());
        self
    }

    /// Set the region used for SSO
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Construct an SsoCredentialsProvider from the builder
    ///
    /// # Panics
    /// This method will panic if the any of the following required fields are unset:
    /// - [`start_url`](Self::start_url)
    /// - [`role_name`](Self::role_name)
    /// - [`account_id`](Self::account_id)
    /// - [`region`](Self::region)
    pub fn build(self) -> SsoCredentialsProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let sso_config = SsoConfig {
            account_id: self.account_id.expect("account_id must be set"),
            role_name: self.role_name.expect("role_name must be set"),
            start_url: self.start_url.expect("start_url must be set"),
            region: self.region.expect("region must be set"),
        };
        SsoCredentialsProvider::new(&provider_config, sso_config)
    }
}

#[derive(Debug)]
pub(crate) enum LoadTokenError {
    InvalidCredentials(InvalidJsonCredentials),
    NoHomeDirectory,
    IoError { err: io::Error, path: PathBuf },
}

impl Display for LoadTokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadTokenError::InvalidCredentials(err) => {
                write!(f, "SSO Token was invalid (expected JSON): {}", err)
            }
            LoadTokenError::NoHomeDirectory => write!(f, "Could not resolve a home directory"),
            LoadTokenError::IoError { err, path } => {
                write!(f, "failed to read `{}`: {}", path.display(), err)
            }
        }
    }
}

impl Error for LoadTokenError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LoadTokenError::InvalidCredentials(err) => Some(err as _),
            LoadTokenError::NoHomeDirectory => None,
            LoadTokenError::IoError { err, .. } => Some(err as _),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SsoConfig {
    pub(crate) account_id: String,
    pub(crate) role_name: String,
    pub(crate) start_url: String,
    pub(crate) region: Region,
}

async fn load_sso_credentials(
    sso_config: &SsoConfig,
    sso: &aws_smithy_client::Client<DynConnector, SsoMiddleware>,
    env: &Env,
    fs: &Fs,
) -> provider::Result {
    let token = load_token(&sso_config.start_url, env, fs)
        .await
        .map_err(CredentialsError::provider_error)?;
    let config = aws_sdk_sso::Config::builder()
        .region(sso_config.region.clone())
        .build();
    let operation = GetRoleCredentialsInput::builder()
        .role_name(&sso_config.role_name)
        .access_token(&*token.access_token)
        .account_id(&sso_config.account_id)
        .build()
        .map_err(|err| {
            CredentialsError::unhandled(format!("could not construct SSO token input: {}", err))
        })?
        .make_operation(&config)
        .await
        .map_err(CredentialsError::unhandled)?;
    let resp = sso
        .call(operation)
        .await
        .map_err(CredentialsError::provider_error)?;
    let credentials: RoleCredentials = resp
        .role_credentials
        .ok_or_else(|| CredentialsError::unhandled("SSO did not return credentials"))?;
    let akid = credentials
        .access_key_id
        .ok_or_else(|| CredentialsError::unhandled("no access key id in response"))?;
    let secret_key = credentials
        .secret_access_key
        .ok_or_else(|| CredentialsError::unhandled("no secret key in response"))?;
    let expiration = DateTime::from_millis(credentials.expiration)
        .try_into()
        .map_err(|err| {
            CredentialsError::unhandled(format!(
                "expiration could not be converted into a system time: {}",
                err
            ))
        })?;
    Ok(Credentials::new(
        akid,
        secret_key,
        credentials.session_token,
        Some(expiration),
        "SSO",
    ))
}

/// Load the token for `start_url` from `~/.aws/sso/cache/<hashofstarturl>.json`
async fn load_token(start_url: &str, env: &Env, fs: &Fs) -> Result<SsoToken, LoadTokenError> {
    let home = home_dir(env, Os::real()).ok_or(LoadTokenError::NoHomeDirectory)?;
    let path = sso_token_path(start_url, &home);
    let data =
        Zeroizing::new(
            fs.read_to_end(&path)
                .await
                .map_err(|err| LoadTokenError::IoError {
                    err,
                    path: path.to_path_buf(),
                })?,
        );
    let token = parse_token_json(&data).map_err(LoadTokenError::InvalidCredentials)?;
    Ok(token)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SsoToken {
    access_token: Zeroizing<String>,
    expires_at: DateTime,
    region: Option<Region>,
}

/// Parse SSO token JSON from input
fn parse_token_json(input: &[u8]) -> Result<SsoToken, InvalidJsonCredentials> {
    /*
      Example:
      {
        "accessToken": "base64string",
        "expiresAt": "2019-11-14T04:05:45Z",
        "region": "us-west-2",
        "startUrl": "https://d-abc123.awsapps.com/start"
    }*/
    let mut acccess_token = None;
    let mut expires_at = None;
    let mut region = None;
    let mut start_url = None;
    json_parse_loop(input, |key, value| {
        match (key, value) {
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("accessToken") => {
                acccess_token = Some(value.to_unescaped()?.to_string())
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("expiresAt") => {
                expires_at = Some(value.to_unescaped()?)
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("region") => {
                region = Some(value.to_unescaped()?.to_string())
            }
            (key, Token::ValueString { value, .. }) if key.eq_ignore_ascii_case("startUrl") => {
                start_url = Some(value.to_unescaped()?.to_string())
            }
            _other => {} // ignored
        };
        Ok(())
    })?;
    let access_token =
        Zeroizing::new(acccess_token.ok_or(InvalidJsonCredentials::MissingField("accessToken"))?);
    let expires_at = expires_at.ok_or(InvalidJsonCredentials::MissingField("expiresAt"))?;
    let expires_at = DateTime::from_str(expires_at.as_ref(), Format::DateTime).map_err(|e| {
        InvalidJsonCredentials::InvalidField {
            field: "expiresAt",
            err: e.into(),
        }
    })?;
    let region = region.map(Region::new);
    Ok(SsoToken {
        access_token,
        expires_at,
        region,
    })
}

/// Determine the SSO token path for a given start_url
fn sso_token_path(start_url: &str, home: &str) -> PathBuf {
    // hex::encode returns a lowercase string
    let mut out = PathBuf::with_capacity(home.len() + "/.aws/sso/cache".len() + ".json".len() + 40);
    out.push(home);
    out.push(".aws/sso/cache");
    out.push(&hex::encode(digest::digest(
        &digest::SHA1_FOR_LEGACY_USE_ONLY,
        start_url.as_bytes(),
    )));
    out.set_extension("json");
    out
}

#[cfg(test)]
mod test {
    use crate::json_credentials::InvalidJsonCredentials;
    use crate::sso::{load_token, parse_token_json, sso_token_path, LoadTokenError, SsoToken};
    use aws_smithy_types::DateTime;
    use aws_types::os_shim_internal::{Env, Fs};
    use aws_types::region::Region;
    use zeroize::Zeroizing;

    #[test]
    fn deserialize_valid_tokens() {
        let token = br#"
        {
            "accessToken": "base64string",
            "expiresAt": "2009-02-13T23:31:30Z",
            "region": "us-west-2",
            "startUrl": "https://d-abc123.awsapps.com/start"
        }"#;
        assert_eq!(
            parse_token_json(token).expect("valid"),
            SsoToken {
                access_token: Zeroizing::new("base64string".into()),
                expires_at: DateTime::from_secs(1234567890),
                region: Some(Region::from_static("us-west-2"))
            }
        );

        let no_region = br#"{
            "accessToken": "base64string",
            "expiresAt": "2009-02-13T23:31:30Z"
        }"#;
        assert_eq!(
            parse_token_json(no_region).expect("valid"),
            SsoToken {
                access_token: Zeroizing::new("base64string".into()),
                expires_at: DateTime::from_secs(1234567890),
                region: None
            }
        );
    }

    #[test]
    fn invalid_timestamp() {
        let token = br#"
        {
            "accessToken": "base64string",
            "expiresAt": "notatimestamp",
            "region": "us-west-2",
            "startUrl": "https://d-abc123.awsapps.com/start"
        }"#;
        let err = parse_token_json(token).expect_err("invalid timestamp");
        assert!(
            format!("{}", err).contains("Invalid field in response: `expiresAt`."),
            "{}",
            err
        );
    }

    #[test]
    fn missing_fields() {
        let token = br#"
        {
            "expiresAt": "notatimestamp",
            "region": "us-west-2",
            "startUrl": "https://d-abc123.awsapps.com/start"
        }"#;
        let err = parse_token_json(token).expect_err("missing akid");
        assert!(
            matches!(err, InvalidJsonCredentials::MissingField("accessToken")),
            "incorrect error: {:?}",
            err
        );

        let token = br#"
        {
            "accessToken": "akid",
            "region": "us-west-2",
            "startUrl": "https://d-abc123.awsapps.com/start"
        }"#;
        let err = parse_token_json(token).expect_err("missing expiry");
        assert!(
            matches!(err, InvalidJsonCredentials::MissingField("expiresAt")),
            "incorrect error: {:?}",
            err
        );
    }

    #[test]
    fn determine_correct_cache_filenames() {
        assert_eq!(
            sso_token_path("https://d-92671207e4.awsapps.com/start", "/home/me").as_os_str(),
            "/home/me/.aws/sso/cache/13f9d35043871d073ab260e020f0ffde092cb14b.json"
        );
        assert_eq!(
            sso_token_path("https://d-92671207e4.awsapps.com/start", "/home/me/").as_os_str(),
            "/home/me/.aws/sso/cache/13f9d35043871d073ab260e020f0ffde092cb14b.json"
        );
    }

    #[tokio::test]
    async fn gracefully_handle_missing_files() {
        let err = load_token(
            "asdf",
            &Env::from_slice(&[("HOME", "/home")]),
            &Fs::from_slice(&[]),
        )
        .await
        .expect_err("should fail, file is missing");
        assert!(
            matches!(err, LoadTokenError::IoError { .. }),
            "should be io error, got {}",
            err
        );
    }
}
