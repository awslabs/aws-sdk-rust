/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! IMDSv2 Credentials Provider
//!
//! # Important
//! This credential provider will NOT fallback to IMDSv1. Ensure that IMDSv2 is enabled on your instances.

use crate::imds;
use crate::imds::client::{ImdsError, LazyClient};
use crate::provider_config::ProviderConfig;
use aws_types::credentials::{future, CredentialsError, ProvideCredentials};
use aws_types::os_shim_internal::Env;
use aws_types::{credentials, Credentials};
use smithy_client::SdkError;
use smithy_json::deserialize::token::skip_value;
use smithy_json::deserialize::{json_token_iter, EscapeError, Token};
use smithy_types::instant::Format;
use smithy_types::Instant;
use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::time::SystemTime;
use tokio::sync::OnceCell;

/// IMDSv2 Credentials Provider
///
/// **Note**: This credentials provider will NOT fallback to the IMDSv1 flow.
#[derive(Debug)]
pub struct ImdsCredentialsProvider {
    client: LazyClient,
    env: Env,
    profile: OnceCell<String>,
}

/// Builder for [`ImdsCredentialsProvider`]
#[derive(Default, Debug)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    profile_override: Option<String>,
    imds_override: Option<imds::Client>,
}

impl Builder {
    /// Override the configuration used for this provider
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    /// Override the [instance profile](instance-profile) used for this provider.
    ///
    /// When retrieving IMDS credentials, a call must first be made to
    /// `<IMDS_BASE_URL>/latest/meta-data/iam/security-credentials`. This returns the instance
    /// profile used. By setting this parameter, the initial call to retrieve the profile is skipped
    /// and the provided value is used instead.
    ///
    /// [instance-profile]: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/iam-roles-for-amazon-ec2.html#ec2-instance-profile
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profile_override = Some(profile.into());
        self
    }

    /// Override the IMDS client used for this provider
    ///
    /// The IMDS client will be loaded and configured via `~/.aws/config` and environment variables,
    /// however, if necessary the entire client may be provided directly.
    ///
    /// For more information about IMDS client configuration loading see [`imds::Client`]
    pub fn imds_client(mut self, client: imds::Client) -> Self {
        if self.provider_config.is_some() {
            tracing::warn!("provider config override by a full client override");
        }
        self.imds_override = Some(client);
        self
    }

    /// Create an [`ImdsCredentialsProvider`] from this builder.
    pub fn build(self) -> ImdsCredentialsProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let env = provider_config.env();
        let client = self
            .imds_override
            .map(LazyClient::from_ready_client)
            .unwrap_or_else(|| {
                imds::Client::builder()
                    .configure(&provider_config)
                    .build_lazy()
            });
        let profile = OnceCell::new_with(self.profile_override);
        ImdsCredentialsProvider {
            client,
            env,
            profile,
        }
    }
}

mod codes {
    pub(super) const ASSUME_ROLE_UNAUTHORIZED_ACCESS: &str = "AssumeRoleUnauthorizedAccess";
}

impl ProvideCredentials for ImdsCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

impl ImdsCredentialsProvider {
    /// Builder for [`ImdsCredentialsProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    fn imds_disabled(&self) -> bool {
        match self.env.get(super::env::EC2_METADATA_DISABLED) {
            Ok(value) => value.eq_ignore_ascii_case("true"),
            _ => false,
        }
    }

    /// Load an inner IMDS client from the OnceCell
    async fn client(&self) -> Result<&imds::Client, CredentialsError> {
        self.client.client().await.map_err(|build_error| {
            CredentialsError::InvalidConfiguration(format!("{}", build_error).into())
        })
    }

    /// Retrieve the instance profile directly. This method should only be used as an argument to
    /// `OnceCell::get_or_try_init`
    async fn get_profile_uncached(&self) -> Result<String, CredentialsError> {
        match self
            .client()
            .await?
            .get("/latest/meta-data/iam/security-credentials")
            .await
        {
            Ok(profile) => Ok(profile),
            Err(SdkError::ServiceError {
                err: ImdsError::ErrorResponse { code: 404, .. },
                ..
            }) => {
                tracing::info!(
                    "received 404 from IMDS when loading profile information. \
                Hint: This instance may not have an IAM role associated."
                );
                Err(CredentialsError::CredentialsNotLoaded)
            }
            Err(other) => Err(CredentialsError::ProviderError(other.into())),
        }
    }

    async fn credentials(&self) -> credentials::Result {
        if self.imds_disabled() {
            tracing::debug!("IMDS disabled because $AWS_EC2_METADATA_DISABLED was set to `true`");
            return Err(CredentialsError::CredentialsNotLoaded);
        }
        let get_profile = self.get_profile_uncached();
        let profile = self.profile.get_or_try_init(|| get_profile).await?;
        let credentials = self
            .client()
            .await?
            .get(&format!(
                "/latest/meta-data/iam/security-credentials/{}",
                profile
            ))
            .await
            .map_err(|e| CredentialsError::ProviderError(e.into()))?;
        match parse_imds_credentials(&credentials) {
            Ok(ImdsCredentialsResponse::Success {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
                ..
            }) => Ok(Credentials::new(
                access_key_id,
                secret_access_key,
                session_token.map(|tok| tok.to_string()),
                Some(expiration),
                "IMDSv2",
            )),
            Ok(ImdsCredentialsResponse::Error { code, message })
                if code == codes::ASSUME_ROLE_UNAUTHORIZED_ACCESS =>
            {
                Err(CredentialsError::InvalidConfiguration(
                    format!(
                        "Incorrect IMDS/IAM configuration: [{}] {}. \
                        Hint: Does this role have a trust relationship with EC2?",
                        code, message
                    )
                    .into(),
                ))
            }
            Ok(ImdsCredentialsResponse::Error { code, message }) => {
                Err(CredentialsError::ProviderError(
                    format!(
                        "Error retrieving credentials from IMDS: {} {}",
                        code, message
                    )
                    .into(),
                ))
            }
            // got bad data from IMDS, should not occur during normal operation:
            Err(invalid) => Err(CredentialsError::Unhandled(invalid.into())),
        }
    }
}

/// Internal data mapping for IMDS response
#[derive(PartialEq, Eq, Debug)]
enum ImdsCredentialsResponse<'a> {
    Success {
        access_key_id: Cow<'a, str>,
        secret_access_key: Cow<'a, str>,
        session_token: Option<Cow<'a, str>>,
        expiration: SystemTime,
        r#type: Cow<'a, str>,
    },
    Error {
        code: Cow<'a, str>,
        message: Cow<'a, str>,
    },
}

impl From<EscapeError> for InvalidImdsResponse {
    fn from(err: EscapeError) -> Self {
        InvalidImdsResponse::JsonError(err.into())
    }
}

impl From<smithy_json::deserialize::Error> for InvalidImdsResponse {
    fn from(err: smithy_json::deserialize::Error) -> Self {
        InvalidImdsResponse::JsonError(err.into())
    }
}

#[derive(Debug)]
enum InvalidImdsResponse {
    JsonError(Box<dyn Error + Send + Sync>),
    Custom(Cow<'static, str>),
    MissingField(&'static str),
}

impl Display for InvalidImdsResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidImdsResponse::Custom(msg) => write!(f, "{}", msg),
            InvalidImdsResponse::MissingField(field) => write!(
                f,
                "Expected field `{}` in IMDS response but it was missing",
                field
            ),
            InvalidImdsResponse::JsonError(json) => {
                write!(f, "invalid JSON in IMDS response: {}", json)
            }
        }
    }
}

impl Error for InvalidImdsResponse {}

/// Deserialize an IMDS response from a string
///
/// There are two levels of error here: the top level distinguishes between a successfully parsed
/// response from IMDS vs. something invalid / unexpected. The inner error distinguishes between
/// a successful request to IMDS that includes credentials vs. an error with a code.
fn parse_imds_credentials(
    credentials_response: &str,
) -> Result<ImdsCredentialsResponse, InvalidImdsResponse> {
    let mut tokens = json_token_iter(credentials_response.as_bytes()).peekable();
    let mut code = None;
    let mut tpe = None;
    let mut access_key_id = None;
    let mut secret_access_key = None;
    let mut session_token = None;
    let mut expiration = None;
    let mut message = None;
    if !matches!(tokens.next().transpose()?, Some(Token::StartObject { .. })) {
        return Err(InvalidImdsResponse::JsonError(
            "expected a JSON document starting with `{`".into(),
        ));
    }
    loop {
        match tokens.next().transpose()? {
            Some(Token::EndObject { .. }) => break,
            Some(Token::ObjectKey { key, .. }) => {
                if let Some(Ok(Token::ValueString { value, .. })) = tokens.peek() {
                    match key.as_escaped_str() {
                        /*
                         "Code": "Success",
                         "Type": "AWS-HMAC",
                         "AccessKeyId" : "accessKey",
                         "SecretAccessKey" : "secret",
                         "Token" : "token",
                         "Expiration" : "....",
                         "LastUpdated" : "2009-11-23T0:00:00Z"
                        */
                        "Code" => code = Some(value.to_unescaped()?),
                        "Type" => tpe = Some(value.to_unescaped()?),
                        "AccessKeyId" => access_key_id = Some(value.to_unescaped()?),
                        "SecretAccessKey" => secret_access_key = Some(value.to_unescaped()?),
                        "Token" => session_token = Some(value.to_unescaped()?),
                        "Expiration" => expiration = Some(value.to_unescaped()?),

                        // Error case handling: message will be set
                        "Message" => message = Some(value.to_unescaped()?),
                        _ => {}
                    }
                }
                skip_value(&mut tokens)?;
            }
            other => {
                return Err(InvalidImdsResponse::Custom(
                    format!("expected object key, found: {:?}", other,).into(),
                ));
            }
        }
    }
    if tokens.next().is_some() {
        return Err(InvalidImdsResponse::Custom(
            "found more JSON tokens after completing parsing".into(),
        ));
    }
    match code {
        // IMDS does not appear to reply with an `Code` missing, but documentation indicates it
        // may be possible
        None | Some(Cow::Borrowed("Success")) => {
            let tpe = tpe.ok_or(InvalidImdsResponse::MissingField("Type"))?;
            let access_key_id =
                access_key_id.ok_or(InvalidImdsResponse::MissingField("AccessKeyId"))?;
            let secret_access_key =
                secret_access_key.ok_or(InvalidImdsResponse::MissingField("SecretAccessKey"))?;
            let expiration = expiration.ok_or(InvalidImdsResponse::MissingField("Expiration"))?;
            let expiration = Instant::from_str(expiration.as_ref(), Format::DateTime)
                .map_err(|err| {
                    InvalidImdsResponse::Custom(format!("invalid date: {}", err).into())
                })?
                .to_system_time()
                .ok_or_else(|| {
                    InvalidImdsResponse::Custom("invalid expiration (prior to unix epoch)".into())
                })?;
            Ok(ImdsCredentialsResponse::Success {
                access_key_id,
                secret_access_key,
                r#type: tpe,
                session_token,
                expiration,
            })
        }
        Some(other) => Ok(ImdsCredentialsResponse::Error {
            code: other,
            message: message.unwrap_or_else(|| "no message".into()),
        }),
    }
}

#[cfg(test)]
mod test {
    use crate::imds::credentials::{
        parse_imds_credentials, ImdsCredentialsResponse, InvalidImdsResponse,
    };
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn imds_success_response() {
        let response = r#"
        {
          "Code" : "Success",
          "LastUpdated" : "2021-09-17T20:57:08Z",
          "Type" : "AWS-HMAC",
          "AccessKeyId" : "ASIARTEST",
          "SecretAccessKey" : "xjtest",
          "Token" : "IQote///test",
          "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_imds_credentials(response).expect("valid JSON");
        assert_eq!(
            parsed,
            ImdsCredentialsResponse::Success {
                r#type: "AWS-HMAC".into(),
                access_key_id: "ASIARTEST".into(),
                secret_access_key: "xjtest".into(),
                session_token: Some("IQote///test".into()),
                expiration: UNIX_EPOCH + Duration::from_secs(1631935916),
            }
        )
    }

    #[test]
    fn imds_invalid_json() {
        let error = parse_imds_credentials("404: not found").expect_err("no json");
        match error {
            InvalidImdsResponse::JsonError(_) => {} // ok.
            err => panic!("incorrect error: {:?}", err),
        }
    }

    #[test]
    fn imds_not_json_object() {
        let error = parse_imds_credentials("[1,2,3]").expect_err("no json");
        match error {
            InvalidImdsResponse::JsonError(_) => {} // ok.
            _ => panic!("incorrect error"),
        }
    }

    #[test]
    fn imds_missing_code() {
        let resp = r#"{
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "AccessKeyId" : "ASIARTEST",
            "SecretAccessKey" : "xjtest",
            "Token" : "IQote///test",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_imds_credentials(resp).expect("code not required");
        assert_eq!(
            parsed,
            ImdsCredentialsResponse::Success {
                r#type: "AWS-HMAC".into(),
                access_key_id: "ASIARTEST".into(),
                secret_access_key: "xjtest".into(),
                session_token: Some("IQote///test".into()),
                expiration: UNIX_EPOCH + Duration::from_secs(1631935916),
            }
        )
    }

    #[test]
    fn imds_optional_session_token() {
        let resp = r#"{
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "AccessKeyId" : "ASIARTEST",
            "SecretAccessKey" : "xjtest",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        let parsed = parse_imds_credentials(resp).expect("code not required");
        assert_eq!(
            parsed,
            ImdsCredentialsResponse::Success {
                r#type: "AWS-HMAC".into(),
                access_key_id: "ASIARTEST".into(),
                secret_access_key: "xjtest".into(),
                session_token: None,
                expiration: UNIX_EPOCH + Duration::from_secs(1631935916),
            }
        )
    }

    #[test]
    fn imds_missing_akid() {
        let resp = r#"{
            "Code": "Success",
            "LastUpdated" : "2021-09-17T20:57:08Z",
            "Type" : "AWS-HMAC",
            "SecretAccessKey" : "xjtest",
            "Token" : "IQote///test",
            "Expiration" : "2021-09-18T03:31:56Z"
        }"#;
        match parse_imds_credentials(resp).expect_err("no code") {
            InvalidImdsResponse::MissingField("AccessKeyId") => {} // ok
            resp => panic!("incorrect imds response: {:?}", resp),
        }
    }

    #[test]
    fn imds_error_response() {
        let response = r#"{
          "Code" : "AssumeRoleUnauthorizedAccess",
          "Message" : "EC2 cannot assume the role integration-test.",
          "LastUpdated" : "2021-09-17T20:46:56Z"
        }"#;
        let parsed = parse_imds_credentials(response).expect("valid JSON");
        assert_eq!(
            parsed,
            ImdsCredentialsResponse::Error {
                code: "AssumeRoleUnauthorizedAccess".into(),
                message: "EC2 cannot assume the role integration-test.".into(),
            }
        );
    }
}
