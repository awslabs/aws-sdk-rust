/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! IMDSv2 Credentials Provider
//!
//! # Important
//! This credential provider will NOT fallback to IMDSv1. Ensure that IMDSv2 is enabled on your instances.

use super::client::error::ImdsError;
use crate::imds;
use crate::imds::client::LazyClient;
use crate::json_credentials::{parse_json_credentials, JsonCredentials, RefreshableCredentials};
use crate::provider_config::ProviderConfig;
use aws_credential_types::provider::{self, error::CredentialsError, future, ProvideCredentials};
use aws_credential_types::time_source::TimeSource;
use aws_credential_types::Credentials;
use aws_types::os_shim_internal::Env;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

const CREDENTIAL_EXPIRATION_INTERVAL: Duration = Duration::from_secs(15 * 60);

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

/// IMDSv2 Credentials Provider
///
/// _Note: This credentials provider will NOT fallback to the IMDSv1 flow._
#[derive(Debug)]
pub struct ImdsCredentialsProvider {
    client: LazyClient,
    env: Env,
    profile: Option<String>,
    time_source: TimeSource,
    last_retrieved_credentials: Arc<RwLock<Option<Credentials>>>,
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
        let env = provider_config.env();
        let client = self
            .imds_override
            .map(LazyClient::from_ready_client)
            .unwrap_or_else(|| {
                imds::Client::builder()
                    .configure(&provider_config)
                    .build_lazy()
            });
        ImdsCredentialsProvider {
            client,
            env,
            profile: self.profile_override,
            time_source: provider_config.time_source(),
            last_retrieved_credentials: Arc::new(RwLock::new(self.last_retrieved_credentials)),
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

    /// Retrieve the instance profile from IMDS
    async fn get_profile_uncached(&self) -> Result<String, CredentialsError> {
        match self
            .client()
            .await?
            .get("/latest/meta-data/iam/security-credentials/")
            .await
        {
            Ok(profile) => Ok(profile),
            Err(ImdsError::ErrorResponse(context))
                if context.response().status().as_u16() == 404 =>
            {
                tracing::warn!(
                    "received 404 from IMDS when loading profile information. \
                    Hint: This instance may not have an IAM role associated."
                );
                Err(CredentialsError::not_loaded("received 404 from IMDS"))
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
        let rng = fastrand::Rng::with_seed(
            self.time_source
                .now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("now should be after UNIX EPOCH")
                .as_secs(),
        );
        // calculate credentials' refresh offset with jitter
        let refresh_offset =
            CREDENTIAL_EXPIRATION_INTERVAL + Duration::from_secs(rng.u64(120..=600));
        let new_expiry = self.time_source.now() + refresh_offset;

        if new_expiry < expiration {
            return expiration;
        }

        tracing::warn!(
            "Attempting credential expiration extension due to a credential service availability issue. \
            A refresh of these credentials will be attempted again within the next {:.2} minutes.",
            refresh_offset.as_secs_f64() / 60.0,
        );

        new_expiry
    }

    async fn retrieve_credentials(&self) -> provider::Result {
        if self.imds_disabled() {
            tracing::debug!("IMDS disabled because $AWS_EC2_METADATA_DISABLED was set to `true`");
            return Err(CredentialsError::not_loaded(
                "IMDS disabled by $AWS_ECS_METADATA_DISABLED",
            ));
        }
        tracing::debug!("loading credentials from IMDS");
        let profile: Cow<'_, str> = match &self.profile {
            Some(profile) => profile.into(),
            None => self.get_profile_uncached().await?.into(),
        };
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
            Ok(JsonCredentials::RefreshableCredentials(RefreshableCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                expiration,
                ..
            })) => {
                let expiration = self.maybe_extend_expiration(expiration);
                let creds = Credentials::new(
                    access_key_id,
                    secret_access_key,
                    Some(session_token.to_string()),
                    expiration.into(),
                    "IMDSv2",
                );
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
    use std::time::{Duration, UNIX_EPOCH};

    use crate::imds::client::test::{
        imds_request, imds_response, make_client, token_request, token_response,
    };
    use crate::imds::credentials::ImdsCredentialsProvider;
    use crate::provider_config::ProviderConfig;
    use aws_credential_types::provider::ProvideCredentials;
    use aws_credential_types::time_source::{TestingTimeSource, TimeSource};
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_client::erase::DynConnector;
    use aws_smithy_client::test_connection::TestConnection;
    use tracing_test::traced_test;

    const TOKEN_A: &str = "token_a";

    #[tokio::test]
    async fn profile_is_not_cached() {
        let connection = TestConnection::new(vec![
                (
                    token_request("http://169.254.169.254", 21600),
                    token_response(21600, TOKEN_A),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                    imds_response(r#"profile-name"#),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/profile-name", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                    imds_response(r#"different-profile"#),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/different-profile", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST2\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
            ]);
        let client = ImdsCredentialsProvider::builder()
            .imds_client(make_client(&connection).await)
            .build();
        let creds1 = client.provide_credentials().await.expect("valid creds");
        let creds2 = client.provide_credentials().await.expect("valid creds");
        assert_eq!(creds1.access_key_id(), "ASIARTEST");
        assert_eq!(creds2.access_key_id(), "ASIARTEST2");
        connection.assert_requests_match(&[]);
    }

    #[tokio::test]
    #[traced_test]
    async fn expired_credentials_should_be_extended() {
        let connection = TestConnection::new(vec![
                (
                    token_request("http://169.254.169.254", 21600),
                    token_response(21600, TOKEN_A),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                    imds_response(r#"profile-name"#),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/profile-name", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
            ]);

        // set to 2021-09-21T17:41:25Z that renders fetched credentials already expired (2021-09-21T04:16:53Z)
        let time_of_request_to_fetch_credentials = UNIX_EPOCH + Duration::from_secs(1632246085);
        let time_source = TimeSource::testing(&TestingTimeSource::new(
            time_of_request_to_fetch_credentials,
        ));

        tokio::time::pause();

        let provider_config = ProviderConfig::no_configuration()
            .with_http_connector(DynConnector::new(connection.clone()))
            .with_time_source(time_source)
            .with_sleep(TokioSleep::new());
        let client = crate::imds::Client::builder()
            .configure(&provider_config)
            .build()
            .await
            .expect("valid client");
        let provider = ImdsCredentialsProvider::builder()
            .configure(&provider_config)
            .imds_client(client)
            .build();
        let creds = provider.provide_credentials().await.expect("valid creds");
        assert!(creds.expiry().unwrap() > time_of_request_to_fetch_credentials);
        connection.assert_requests_match(&[]);

        // We should inform customers that expired credentials are being used for stability.
        assert!(logs_contain("Attempting credential expiration extension"));
    }

    #[tokio::test]
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    async fn read_timeout_during_credentials_refresh_should_yield_last_retrieved_credentials() {
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint(http::Uri::from_static("http://240.0.0.0"))
            .build()
            .await
            .expect("valid client");
        let expected = aws_credential_types::Credentials::for_tests();
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
            // seed fallback credentials for testing
            .last_retrieved_credentials(expected.clone())
            .build();
        let actual = provider.provide_credentials().await;
        assert_eq!(actual.unwrap(), expected);
    }

    #[tokio::test]
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    async fn read_timeout_during_credentials_refresh_should_error_without_last_retrieved_credentials(
    ) {
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint(http::Uri::from_static("http://240.0.0.0"))
            .build()
            .await
            .expect("valid client");
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
            // no fallback credentials provided
            .build();
        let actual = provider.provide_credentials().await;
        assert!(matches!(
            actual,
            Err(aws_credential_types::provider::error::CredentialsError::CredentialsNotLoaded(_))
        ));
    }

    #[tokio::test]
    #[cfg(any(feature = "rustls", feature = "native-tls"))]
    async fn external_timeout_during_credentials_refresh_should_yield_last_retrieved_credentials() {
        use aws_sdk_sso::config::AsyncSleep;
        let client = crate::imds::Client::builder()
            // 240.* can never be resolved
            .endpoint(http::Uri::from_static("http://240.0.0.0"))
            .build()
            .await
            .expect("valid client");
        let expected = aws_credential_types::Credentials::for_tests();
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(client)
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
            Ok(_) => assert!(false, "provide_credentials completed before timeout future"),
            Err(_err) => match provider.fallback_on_interrupt() {
                Some(actual) => assert_eq!(actual, expected),
                None => assert!(
                    false,
                    "provide_credentials timed out and no credentials returned from fallback_on_interrupt"
                ),
            },
        };
    }

    #[tokio::test]
    async fn fallback_credentials_should_be_used_when_imds_returns_500_during_credentials_refresh()
    {
        let connection = TestConnection::new(vec![
                // The next three request/response pairs will correspond to the first call to `provide_credentials`.
                // During the call, it populates last_retrieved_credentials.
                (
                    token_request("http://169.254.169.254", 21600),
                    token_response(21600, TOKEN_A),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                    imds_response(r#"profile-name"#),
                ),
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/profile-name", TOKEN_A),
                    imds_response("{\n  \"Code\" : \"Success\",\n  \"LastUpdated\" : \"2021-09-20T21:42:26Z\",\n  \"Type\" : \"AWS-HMAC\",\n  \"AccessKeyId\" : \"ASIARTEST\",\n  \"SecretAccessKey\" : \"testsecret\",\n  \"Token\" : \"testtoken\",\n  \"Expiration\" : \"2021-09-21T04:16:53Z\"\n}"),
                ),
                // The following request/response pair corresponds to the second call to `provide_credentials`.
                // During the call, IMDS returns response code 500.
                (
                    imds_request("http://169.254.169.254/latest/meta-data/iam/security-credentials/", TOKEN_A),
                    http::Response::builder().status(500).body("").unwrap(),
                ),
            ]);
        let provider = ImdsCredentialsProvider::builder()
            .imds_client(make_client(&connection).await)
            .build();
        let creds1 = provider.provide_credentials().await.expect("valid creds");
        assert_eq!(creds1.access_key_id(), "ASIARTEST");
        // `creds1` should be returned as fallback credentials and assigned to `creds2`
        let creds2 = provider.provide_credentials().await.expect("valid creds");
        assert_eq!(creds1, creds2);
        connection.assert_requests_match(&[]);
    }
}
