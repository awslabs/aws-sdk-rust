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
use crate::json_credentials::{parse_json_credentials, JsonCredentials};
use crate::provider_config::ProviderConfig;
use aws_smithy_client::SdkError;
use aws_types::credentials::{future, CredentialsError, ProvideCredentials};
use aws_types::os_shim_internal::Env;
use aws_types::{credentials, Credentials};

use tokio::sync::OnceCell;

/// IMDSv2 Credentials Provider
///
/// _Note: This credentials provider will NOT fallback to the IMDSv1 flow._
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
            // need to format the build error since we don't own it and it can't be cloned
            CredentialsError::invalid_configuration(format!("{}", build_error))
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
            Err(ImdsError::ErrorResponse { response, .. }) if response.status().as_u16() == 404 => {
                tracing::info!(
                    "received 404 from IMDS when loading profile information. \
                Hint: This instance may not have an IAM role associated."
                );
                Err(CredentialsError::not_loaded("received 404 from IMDS"))
            }
            Err(ImdsError::FailedToLoadToken(SdkError::DispatchFailure(err))) => Err(
                CredentialsError::not_loaded(format!("could not communicate with imds: {}", err)),
            ),
            Err(other) => Err(CredentialsError::provider_error(other)),
        }
    }

    async fn credentials(&self) -> credentials::Result {
        if self.imds_disabled() {
            tracing::debug!("IMDS disabled because $AWS_EC2_METADATA_DISABLED was set to `true`");
            return Err(CredentialsError::not_loaded(
                "IMDS disabled by $AWS_ECS_METADATA_DISABLED",
            ));
        }
        tracing::debug!("loading credentials from IMDS");
        let get_profile = self.get_profile_uncached();
        let profile = self.profile.get_or_try_init(|| get_profile).await?;
        tracing::debug!(profile = %profile, "loaded profile");
        let credentials = self
            .client()
            .await?
            .get(&format!(
                "/latest/meta-data/iam/security-credentials/{}",
                profile
            ))
            .await
            .map_err(CredentialsError::provider_error)?;
        match parse_json_credentials(&credentials) {
            Ok(JsonCredentials::RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
                ..
            }) => Ok(Credentials::new(
                access_key_id,
                secret_access_key,
                Some(session_token.to_string()),
                expiration.into(),
                "IMDSv2",
            )),
            Ok(JsonCredentials::Error { code, message })
                if code == codes::ASSUME_ROLE_UNAUTHORIZED_ACCESS =>
            {
                Err(CredentialsError::invalid_configuration(format!(
                    "Incorrect IMDS/IAM configuration: [{}] {}. \
                        Hint: Does this role have a trust relationship with EC2?",
                    code, message
                )))
            }
            Ok(JsonCredentials::Error { code, message }) => {
                Err(CredentialsError::provider_error(format!(
                    "Error retrieving credentials from IMDS: {} {}",
                    code, message
                )))
            }
            // got bad data from IMDS, should not occur during normal operation:
            Err(invalid) => Err(CredentialsError::unhandled(invalid)),
        }
    }
}
