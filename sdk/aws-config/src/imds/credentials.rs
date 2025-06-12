/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! IMDSv2 Credentials Provider
//!
//! # Important
//! This credential provider will NOT fallback to IMDSv1. Ensure that IMDSv2 is enabled on your instances.

use super::client::error::ImdsError;
use crate::environment::parse_bool;
use crate::imds::{self, Client};
use crate::json_credentials::{parse_json_credentials, JsonCredentials, RefreshableCredentials};
use crate::provider_config::ProviderConfig;
use aws_credential_types::attributes::AccountId;
use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_credential_types::Credentials;
use aws_runtime::env_config::EnvConfigValue;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::origin::Origin;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

const CREDENTIAL_EXPIRATION_INTERVAL: Duration = Duration::from_secs(10 * 60);
const WARNING_FOR_EXTENDING_CREDENTIALS_EXPIRY: &str =
    "Attempting credential expiration extension due to a credential service availability issue. \
    A refresh of these credentials will be attempted again within the next";

#[derive(Debug)]
struct ImdsCommunicationError {
    source: Box<dyn StdError + Send + Sync + 'static>,
}

impl fmt::Display for ImdsCommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not communicate with IMDS")
    }
}

impl StdError for ImdsCommunicationError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref())
    }
}

// Enum representing the type of IMDS endpoint that the credentials provider should access
// when retrieving the IMDS profile name or credentials.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum ApiVersion {
    #[default]
    Unknown,
    Extended,
    Legacy,
}

// A state maintained by the IMDS credentials provider to manage the retrieval of the IMDS profile name or credentials.
#[derive(Clone, Debug, Default)]
struct ProviderState {
    api_version: ApiVersion,
    resolved_profile: Option<String>,
}

/// IMDSv2 Credentials Provider
///
/// _Note: This credentials provider will NOT fallback to the IMDSv1 flow._
#[derive(Debug)]
pub struct ImdsCredentialsProvider {
    client: Client,
    provider_config: ProviderConfig,
    profile: Option<String>,
    time_source: SharedTimeSource,
    last_retrieved_credentials: Arc<RwLock<Option<Credentials>>>,
    provider_state: Arc<RwLock<ProviderState>>,
}

/// Builder for [`ImdsCredentialsProvider`]
#[derive(Default, Debug)]
pub struct Builder {
    provider_config: Option<ProviderConfig>,
    profile_override: Option<String>,
    imds_override: Option<imds::Client>,
    last_retrieved_credentials: Option<Credentials>,
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
    /// `<IMDS_BASE_URL>/latest/meta-data/iam/security-credentials/`. This returns the instance
    /// profile used. By setting this parameter, retrieving the profile is skipped
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
        self.imds_override = Some(client);
        self
    }

    #[allow(dead_code)]
    #[cfg(test)]
    fn last_retrieved_credentials(mut self, credentials: Credentials) -> Self {
        self.last_retrieved_credentials = Some(credentials);
        self
    }

    /// Create an [`ImdsCredentialsProvider`] from this builder.
    pub fn build(self) -> ImdsCredentialsProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let client = self
            .imds_override
            .unwrap_or_else(|| imds::Client::builder().configure(&provider_config).build());
        ImdsCredentialsProvider {
            client,
            profile: self.profile_override,
            time_source: provider_config.time_source(),
            provider_config,
            last_retrieved_credentials: Arc::new(RwLock::new(self.last_retrieved_credentials)),
            provider_state: Arc::new(RwLock::new(ProviderState::default())),
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

    fn fallback_on_interrupt(&self) -> Option<Credentials> {
        self.last_retrieved_credentials.read().unwrap().clone()
    }
}

impl ImdsCredentialsProvider {
    /// Builder for [`ImdsCredentialsProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    // Retrieve the value for "disable ec2 metadata". If the value is `true`, the method also returns
    // the source that set it to `true`.
    //
    // This checks the following sources:
    // 1. The environment variable `AWS_EC2_METADATA_DISABLED=true/false`
    // 2. The profile key `disable_ec2_metadata=true/false`
    async fn imds_disabled(&self) -> (bool, Origin) {
        EnvConfigValue::new()
            .env(super::env::EC2_METADATA_DISABLED)
            .profile(super::profile_key::EC2_METADATA_DISABLED)
            .validate_and_return_origin(
                &self.provider_config.env(),
                self.provider_config.profile().await,
                parse_bool,
            )
            .map_err(
                |err| tracing::warn!(err = %DisplayErrorContext(&err), "invalid value for `disable ec2 metadata` setting"),
            )
            .map(|(disabled, origin)| (disabled.unwrap_or_default(), origin))
            .unwrap_or_default()
    }

    // Return a configured instance profile name. If the profile name is blank, the method returns
    // a `CredentialsError`.
    //
    // This checks the following sources:
    // 1. The profile name configured via [`Builder::profile`]
    // 2. The environment variable `AWS_EC2_INSTANCE_PROFILE_NAME`
    // 3. The profile key `ec2_instance_profile_name`
    async fn configured_instance_profile_name(
        &self,
    ) -> Result<Option<Cow<'_, str>>, CredentialsError> {
        let configured = match &self.profile {
            Some(profile) => Some(profile.into()),
            None => EnvConfigValue::new()
                .env(super::env::EC2_INSTANCE_PROFILE_NAME)
                .profile(super::profile_key::EC2_INSTANCE_PROFILE_NAME)
                .validate(
                    &self.provider_config.env(),
                    self.provider_config.profile().await,
                    |s| Ok::<String, std::convert::Infallible>(s.to_owned()),
                )
                .expect("validator is infallible")
                .map(Cow::Owned),
        };

        match configured {
            Some(configured) if configured.trim().is_empty() => Err(CredentialsError::not_loaded(
                "blank profile name is not supported",
            )),
            otherwise => Ok(otherwise),
        }
    }

    fn uri_base(&self) -> &str {
        let api_version = &self
            .provider_state
            .read()
            .expect("write critical section does not cause panic")
            .api_version;
        use ApiVersion::*;
        match api_version {
            Legacy => "/latest/meta-data/iam/security-credentials/",
            _ => "/latest/meta-data/iam/security-credentials-extended/",
        }
    }

    // Retrieve the instance profile from IMDS
    //
    // Starting with `ApiVersion::Unknown`, the method first attempts to retrive it using the extended API.
    // If the call is successful, it updates `ProviderState` to remember that the extended API is functional and moves on.
    // Otherwise, it updates `ProviderState` to the legacy mode and tries again.
    // In the end, if the legacy API does not work either, the method gives up and returns a `CredentialsError`.
    async fn resolve_profile_name(&self) -> Result<Cow<'_, str>, CredentialsError> {
        if let Some(profile) = self.configured_instance_profile_name().await? {
            return Ok(profile);
        }

        if let Some(profile) = &self
            .provider_state
            .read()
            .expect("write critical section does not cause panic")
            .resolved_profile
        {
            return Ok(profile.clone().into());
        }

        match self.client.get(self.uri_base()).await {
            Ok(profile) => {
                let state = &mut self
                    .provider_state
                    .write()
                    .expect("write critical section does not cause panic");
                state.resolved_profile = Some(profile.clone().into());
                if state.api_version == ApiVersion::Unknown {
                    state.api_version = ApiVersion::Extended;
                }
                Ok(Cow::Owned(profile.into()))
            }
            Err(ImdsError::ErrorResponse(context))
                if context.response().status().as_u16() == 404 =>
            {
                tracing::warn!(
                    "received 404 from IMDS when loading profile information. \
                    Hint: This instance may not have an IAM role associated."
                );

                {
                    let state = &mut self
                        .provider_state
                        .write()
                        .expect("write critical section does not cause panic");
                    if state.api_version == ApiVersion::Unknown {
                        tracing::debug!("retrieving an IMDS profile name failed using the extended API, switching to the legacy API and trying again");
                        state.api_version = ApiVersion::Legacy;
                    } else {
                        return Err(CredentialsError::not_loaded("received 404 from IMDS"));
                    }
                }

                Box::pin(self.resolve_profile_name()).await
            }
            Err(ImdsError::FailedToLoadToken(context)) if context.is_dispatch_failure() => {
                Err(CredentialsError::not_loaded(ImdsCommunicationError {
                    source: context.into_source().into(),
                }))
            }
            Err(other) => Err(CredentialsError::provider_error(other)),
        }
    }

    // Extend the cached expiration time if necessary
    //
    // This allows continued use of the credentials even when IMDS returns expired ones.
    fn maybe_extend_expiration(&self, expiration: SystemTime) -> SystemTime {
        let now = self.time_source.now();
        // If credentials from IMDS are not stale, use them as they are.
        if now < expiration {
            return expiration;
        }

        let mut rng = fastrand::Rng::with_seed(
            now.duration_since(SystemTime::UNIX_EPOCH)
                .expect("now should be after UNIX EPOCH")
                .as_secs(),
        );
        // Calculate credentials' refresh offset with jitter, which should be less than 15 minutes
        // the smallest amount of time credentials are valid for.
        // Setting it to something longer than that may have the risk of the credentials expiring
        // before the next refresh.
        let refresh_offset = CREDENTIAL_EXPIRATION_INTERVAL + Duration::from_secs(rng.u64(0..=300));
        let new_expiry = now + refresh_offset;

        tracing::warn!(
            "{WARNING_FOR_EXTENDING_CREDENTIALS_EXPIRY} {:.2} minutes.",
            refresh_offset.as_secs_f64() / 60.0,
        );

        new_expiry
    }

    async fn retrieve_credentials(&self) -> provider::Result {
        if let (true, origin) = self.imds_disabled().await {
            let err = format!("IMDS disabled by {origin} set to `true`",);
            tracing::debug!(err);
            return Err(CredentialsError::not_loaded(err));
        }

        tracing::debug!("loading credentials from IMDS");

        let profile = self.resolve_profile_name().await?;
        tracing::debug!(profile = %profile, "loaded profile");

        let credentials = match self
            .client
            .get(format!("{uri_base}{profile}", uri_base = self.uri_base()))
            .await
        {
            Ok(credentials) => {
                let state = &mut self.provider_state.write().expect("write critical section does not cause panic");
                if state.api_version == ApiVersion::Unknown {
                    state.api_version = ApiVersion::Extended;
                }
                Ok(credentials)
            }
            Err(ImdsError::ErrorResponse(raw)) if raw.response().status().as_u16() == 404 => {
                {
                    let state = &mut self.provider_state.write().expect("write critical section does not cause panic");
                    if state.api_version == ApiVersion::Unknown {
                        tracing::debug!("retrieving credentials failed using the extended API, switching to the legacy API and trying again");
                        state.api_version = ApiVersion::Legacy;
                    } else if self.profile.is_none() {
                        tracing::debug!("retrieving credentials failed using {:?}, clearing cached profile and trying again", state.api_version);
                        state.resolved_profile = None;
                    } else {
                        return Err(CredentialsError::provider_error(ImdsError::ErrorResponse(
                            raw,
                        )));
                    }
                }
                return Box::pin(self.retrieve_credentials()).await;
            }
            otherwise => otherwise,
        }
        .map_err(CredentialsError::provider_error)?;

        match parse_json_credentials(credentials.as_ref()) {
            Ok(JsonCredentials::RefreshableCredentials(RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                account_id,
                expiration,
                ..
            })) => {
                let expiration = self.maybe_extend_expiration(expiration);
                let mut builder = Credentials::builder()
                    .access_key_id(access_key_id)
                    .secret_access_key(secret_access_key)
                    .session_token(session_token)
                    .expiry(expiration)
                    .provider_name("IMDSv2");
                builder.set_account_id(account_id.map(AccountId::from));
                let creds = builder.build();
                *self.last_retrieved_credentials.write().unwrap() = Some(creds.clone());
                Ok(creds)
            }
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

    async fn credentials(&self) -> provider::Result {
        match self.retrieve_credentials().await {
            creds @ Ok(_) => creds,
            // Any failure while retrieving credentials MUST NOT impede use of existing credentials.
            err => match &*self.last_retrieved_credentials.read().unwrap() {
                Some(creds) => Ok(creds.clone()),
                _ => err,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::imds::client::test::{
        imds_request, imds_response, imds_response_404, make_imds_client, token_request,
        token_response,
    };
    use crate::provider_config::ProviderConfig;
    use aws_credential_types::provider::ProvideCredentials;
    use aws_smithy_async::test_util::instant_time_and_sleep;
    use aws_smithy_http_client::test_util::{ReplayEvent, StaticReplayClient};
    use aws_smithy_types::body::SdkBody;
    use std::convert::identity as IdentityFn;
    use std::future::Future;
    use std::pin::Pin;
    use std::time::{Duration, UNIX_EPOCH};
    use tracing_test::traced_test;

    const TOKEN_A: &str = "token_a";

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn warn_on_invalid_value_for_disable_ec2_metadata() {
        let provider_config =
            ProviderConfig::empty().with_env(aws_types::os_shim_internal::Env::from_slice(&[(
                imds::env::EC2_METADATA_DISABLED,
                "not-a-boolean",
            )]));
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        assert!(!provider.imds_disabled().await.0);
        assert!(logs_contain(
            "invalid value for `disable ec2 metadata` setting"
        ));
        assert!(logs_contain(imds::env::EC2_METADATA_DISABLED));
    }

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn environment_priority_on_disable_ec2_metadata() {
        let provider_config = ProviderConfig::empty()
            .with_env(aws_types::os_shim_internal::Env::from_slice(&[(
                imds::env::EC2_METADATA_DISABLED,
                "TRUE",
            )]))
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(aws_types::os_shim_internal::Fs::from_slice(&[(
                "conf",
                "[default]\ndisable_ec2_metadata = false",
            )]));
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        assert_eq!(
            (true, Origin::shared_environment_variable()),
            provider.imds_disabled().await
        );
    }

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn disable_ec2_metadata_via_profile_file() {
        let provider_config = ProviderConfig::empty()
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(aws_types::os_shim_internal::Fs::from_slice(&[(
                "conf",
                "[default]\ndisable_ec2_metadata = true",
            )]));
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        assert_eq!(
            (true, Origin::shared_profile_file()),
            provider.imds_disabled().await
        );
    }

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn creds_provider_configuration_priority_on_ec2_instance_profile_name() {
        let provider_config = ProviderConfig::empty()
            .with_env(aws_types::os_shim_internal::Env::from_slice(&[(
                imds::env::EC2_INSTANCE_PROFILE_NAME,
                "profile-via-env",
            )]))
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(aws_types::os_shim_internal::Fs::from_slice(&[(
                "conf",
                "[default]\nec2_instance_profile_name = profile-via-profile-file",
            )]));

        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .profile("profile-via-creds-provider")
            .configure(&provider_config)
            .imds_client(client.clone())
            .build();
        assert_eq!(
            Some(Cow::Borrowed("profile-via-creds-provider")),
            provider.configured_instance_profile_name().await.unwrap()
        );

        // negative test with a blank profile name
        let provider = ImdsCredentialsProvider::builder()
            .profile("")
            .configure(&provider_config)
            .imds_client(client)
            .build();
        let err = provider
            .configured_instance_profile_name()
            .await
            .err()
            .unwrap();
        assert!(format!("{}", DisplayErrorContext(&err))
            .contains("blank profile name is not supported"));
    }

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn environment_priority_on_ec2_instance_profile_name() {
        let provider_config = ProviderConfig::empty()
            .with_env(aws_types::os_shim_internal::Env::from_slice(&[(
                imds::env::EC2_INSTANCE_PROFILE_NAME,
                "profile-via-env",
            )]))
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(aws_types::os_shim_internal::Fs::from_slice(&[(
                "conf",
                "[default]\nec2_instance_profile_name = profile-via-profile-file",
            )]));
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        assert_eq!(
            Some(Cow::Borrowed("profile-via-env")),
            provider.configured_instance_profile_name().await.unwrap()
        );
    }

    #[tokio::test]
    #[traced_test]
    #[cfg(feature = "default-https-client")]
    async fn ec2_instance_profile_name_via_profile_file() {
        let provider_config = ProviderConfig::empty()
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(aws_types::os_shim_internal::Fs::from_slice(&[(
                "conf",
                "[default]\nec2_instance_profile_name = profile-via-profile-file",
            )]));
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        assert_eq!(
            Some(Cow::Borrowed("profile-via-profile-file")),
            provider.configured_instance_profile_name().await.unwrap()
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn credentials_not_stale_should_be_used_as_they_are() {
        let http_client = StaticReplayClient::new(vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response(r#"profile-name"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/profile-name", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ]);

        // set to 2021-09-21T04:16:50Z that makes returned credentials' expiry (2021-09-21T04:16:53Z)
        // not stale
        let time_of_request_to_fetch_credentials = UNIX_EPOCH + Duration::from_secs(1632197810);
        let (time_source, sleep) = instant_time_and_sleep(time_of_request_to_fetch_credentials);

        let provider_config = ProviderConfig::no_configuration()
            .with_http_client(http_client.clone())
            .with_sleep_impl(sleep)
            .with_time_source(time_source);
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        let creds = provider.provide_credentials().await.expect("valid creds");
        // The expiry should be equal to what is originally set (==2021-09-21T04:16:53Z).
        assert_eq!(
            creds.expiry(),
            UNIX_EPOCH.checked_add(Duration::from_secs(1632197813))
        );
        assert!(creds.account_id().is_none());
        http_client.assert_requests_match(&[]);

        // There should not be logs indicating credentials are extended for stability.
        assert!(!logs_contain(WARNING_FOR_EXTENDING_CREDENTIALS_EXPIRY));
    }

    #[tokio::test]
    #[traced_test]
    async fn expired_credentials_should_be_extended() {
        let http_client = StaticReplayClient::new(vec![
                ReplayEvent::new(
                    token_request("http://169.254.169.254", 21600),
                    token_response(21600, TOKEN_A),
                ),
                ReplayEvent::new(
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                    imds_response(r#"profile-name"#),
                ),
                ReplayEvent::new(
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/profile-name", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
            ]);

        // set to 2021-09-21T17:41:25Z that renders fetched credentials already expired (2021-09-21T04:16:53Z)
        let time_of_request_to_fetch_credentials = UNIX_EPOCH + Duration::from_secs(1632246085);
        let (time_source, sleep) = instant_time_and_sleep(time_of_request_to_fetch_credentials);

        let provider_config = ProviderConfig::no_configuration()
            .with_http_client(http_client.clone())
            .with_sleep_impl(sleep)
            .with_time_source(time_source);
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        let creds = provider.provide_credentials().await.expect("valid creds");
        assert!(creds.expiry().unwrap() > time_of_request_to_fetch_credentials);
        http_client.assert_requests_match(&[]);

        // We should inform customers that expired credentials are being used for stability.
        assert!(logs_contain(WARNING_FOR_EXTENDING_CREDENTIALS_EXPIRY));
    }

    #[tokio::test]
    #[cfg(feature = "default-https-client")]
    async fn read_timeout_during_credentials_refresh_should_yield_last_retrieved_credentials() {
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint("http://240.0.0.0")
            .unwrap()
            .build();
        let expected = aws_credential_types::Credentials::for_tests();
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
            // seed fallback credentials for testing
            .last_retrieved_credentials(expected.clone())
            .build();
        let actual = provider.provide_credentials().await.unwrap();
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    #[cfg(feature = "default-https-client")]
    async fn read_timeout_during_credentials_refresh_should_error_without_last_retrieved_credentials(
    ) {
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint("http://240.0.0.0")
            .unwrap()
            .build();
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
            // no fallback credentials provided
            .build();
        let actual = provider.provide_credentials().await.err().unwrap();
        assert!(
            matches!(actual, CredentialsError::CredentialsNotLoaded(_)),
            "\nexpected: Err(CredentialsError::CredentialsNotLoaded(_))\nactual: {actual:?}"
        );
    }

    #[tokio::test]
    #[cfg(feature = "default-https-client")]
    async fn external_timeout_during_credentials_refresh_should_yield_last_retrieved_credentials() {
        use aws_smithy_async::rt::sleep::AsyncSleep;
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint("http://240.0.0.0")
            .unwrap()
            .build();
        let expected = aws_credential_types::Credentials::for_tests();
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
            .configure(&ProviderConfig::no_configuration())
            // seed fallback credentials for testing
            .last_retrieved_credentials(expected.clone())
            .build();
        let sleeper = aws_smithy_async::rt::sleep::TokioSleep::new();
        let timeout = aws_smithy_async::future::timeout::Timeout::new(
            provider.provide_credentials(),
            // make sure `sleeper.sleep` will be timed out first by setting a shorter duration than connect timeout
            sleeper.sleep(std::time::Duration::from_millis(100)),
        );
        match timeout.await {
            Ok(_) => panic!("provide_credentials completed before timeout future"),
            Err(_err) => match provider.fallback_on_interrupt() {
                Some(actual) => assert_eq!(expected, actual),
                None => panic!(
                    "provide_credentials timed out and no credentials returned from fallback_on_interrupt"
                ),
            },
        };
    }

    #[tokio::test]
    async fn fallback_credentials_should_be_used_when_imds_returns_500_during_credentials_refresh()
    {
        let http_client = StaticReplayClient::new(vec![
                // The next three request/response pairs will correspond to the first call to `provide_credentials`.
                // During the call, it populates last_retrieved_credentials.
                ReplayEvent::new(
                    token_request("http://169.254.169.254", 21600),
                    token_response(21600, TOKEN_A),
                ),
                ReplayEvent::new(
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                    imds_response(r#"profile-name"#),
                ),
                ReplayEvent::new(
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/profile-name", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
                // The following request/response pair corresponds to the second call to `provide_credentials`.
                // During the call, IMDS returns response code 500.
                ReplayEvent::new(
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/profile-name", TOKEN_A),
                    http::Response::builder().status(500).body(SdkBody::empty()).unwrap(),
                ),
            ]);
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(make_imds_client(&http_client))
            .configure(&ProviderConfig::no_configuration())
            .build();
        let creds1 = provider.provide_credentials().await.expect("valid creds");
        assert_eq!("ASIARTEST", creds1.access_key_id());
        // `creds1` should be returned as fallback credentials
        assert_eq!(
            creds1,
            provider.provide_credentials().await.expect("valid creds")
        );
        http_client.assert_requests_match(&[]);
    }

    async fn run_test<F>(
        events: Vec<ReplayEvent>,
        update_builder: fn(Builder) -> Builder,
        runner: F,
    ) where
        F: Fn(ImdsCredentialsProvider) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
    {
        let http_client = StaticReplayClient::new(events);
        let builder = ImdsCredentialsProvider::builder()
            .imds_client(make_imds_client(&http_client))
            .configure(&ProviderConfig::no_configuration());
        let provider = update_builder(builder).build();
        runner(provider).await;
        http_client.assert_requests_match(&[]);
    }

    async fn assert(provider: ImdsCredentialsProvider, expected: &[(Option<&str>, Option<&str>)]) {
        for (expected_access_key_id, expected_account_id) in expected {
            let creds = provider.provide_credentials().await.expect("valid creds");
            assert_eq!(expected_access_key_id, &Some(creds.access_key_id()),);
            assert_eq!(
                expected_account_id,
                &creds.account_id().map(|id| id.as_str())
            );
        }
    }

    #[tokio::test]
    async fn returns_valid_credentials_with_account_id() {
        let extended_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            // A profile is not cached, so we should expect a network call to obtain one.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response(r#"my-profile-0001"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0001", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"123456789101\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            // For the second call to `provide_credentials`, we shouldn't expect a network call to obtain a profile since it's been resolved and cached.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0001", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"123456789101\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(extended_api_events, IdentityFn, |provider| {
            Box::pin(assert(
                provider,
                &[
                    (Some("ASIARTEST"), Some("123456789101")),
                    (Some("ASIARTEST"), Some("123456789101")),
                ],
            ))
        })
        .await;

        let legacy_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            // Obtaining a profile from IMDS using the extended API results in 404.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response_404(),
            ),
            // Should be retried using the legacy API.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                imds_response(r#"my-profile-0009"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0009", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0009", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(legacy_api_events, IdentityFn, |provider| {
            Box::pin(assert(
                provider,
                &[(Some("ASIARTEST"), None), (Some("ASIARTEST"), None)],
            ))
        })
        .await;
    }

    #[tokio::test]
    async fn should_return_credentials_when_profile_is_configured_by_user() {
        let extended_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0002", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"234567891011\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0002", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"234567891011\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(
            extended_api_events,
            |b| b.profile("my-profile-0002"),
            |provider| {
                Box::pin(assert(
                    provider,
                    &[
                        (Some("ASIARTEST"), Some("234567891011")),
                        (Some("ASIARTEST"), Some("234567891011")),
                    ],
                ))
            },
        )
        .await;

        let legacy_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            // Obtaining a credentials using the extended API results in 404.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0010", TOKEN_A),
                imds_response_404(),
            ),
            // Obtain credentials using the legacy API with the configured profile.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0010", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0010", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(
            legacy_api_events,
            |b| b.profile("my-profile-0010"),
            |provider| {
                Box::pin(assert(
                    provider,
                    &[(Some("ASIARTEST"), None), (Some("ASIARTEST"), None)],
                ))
            },
        )
        .await;
    }

    #[tokio::test]
    async fn should_return_valid_credentials_when_profile_is_unstable() {
        let extended_api_events = vec![
            // First call to `provide_credentials` succeeds with the extended API.
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response(r#"my-profile-0003"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0003", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"345678910112\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),

            // Credentials retrieval failed due to unstable profile.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0003", TOKEN_A),
                imds_response_404(),
            ),
            // Start over and retrieve a new profile with the extended API.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response(r#"my-profile-0003-b"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0003-b", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"AccountId\" : \"314253647589\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(extended_api_events, IdentityFn, |provider| {
            Box::pin(assert(
                provider,
                &[
                    (Some("ASIARTEST"), Some("345678910112")),
                    (Some("ASIARTEST"), Some("314253647589")),
                ],
            ))
        })
        .await;

        let legacy_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response_404()
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                imds_response(r#"my-profile-0011"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0011", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            // Credentials retrieval failed due to unstable profile.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0011", TOKEN_A),
                imds_response_404()
            ),
            // Start over and retrieve a new profile with the legacy API.
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                imds_response(r#"my-profile-0011-b"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0011-b", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(legacy_api_events, IdentityFn, |provider| {
            Box::pin(assert(
                provider,
                &[(Some("ASIARTEST"), None), (Some("ASIARTEST"), None)],
            ))
        })
        .await;
    }

    #[tokio::test]
    async fn should_error_when_imds_remains_unstable_with_profile_configured_by_user() {
        // This negative test exercises the same code path for both the extended and legacy APIs.
        // A single set of events is sufficient for testing both.
        let events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0004", TOKEN_A),
                imds_response_404(),
            ),
            // Try obtaining credentials again with the legacy API
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/my-profile-0004", TOKEN_A),
                imds_response_404(),
            ),
        ];
        run_test(
            events,
            |b| b.profile("my-profile-0004"),
            |provider| {
                Box::pin(async move {
                    let err = provider.provide_credentials().await.err().unwrap();
                    matches!(err, CredentialsError::CredentialsNotLoaded(_));
                })
            },
        )
        .await;
    }

    #[tokio::test]
    async fn returns_valid_credentials_without_account_id_using_extended_api() {
        let extended_api_events = vec![
            ReplayEvent::new(
                token_request("http://169.254.169.254", 21600),
                token_response(21600, TOKEN_A),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/", TOKEN_A),
                imds_response(r#"my-profile-0005"#),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0005", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
            ReplayEvent::new(
                imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials-extended/my-profile-0005", TOKEN_A),
                imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
            ),
        ];
        run_test(extended_api_events, IdentityFn, |provider| {
            Box::pin(assert(
                provider,
                &[(Some("ASIARTEST"), None), (Some("ASIARTEST"), None)],
            ))
        })
        .await;
    }
}
