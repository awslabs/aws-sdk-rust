/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Credentials from an AWS Console session vended by AWS Sign-In.

mod cache;
/// Utils related to [RFC 9449: OAuth 2.0 Demonstrating Proof of Possession (DPoP)](https://datatracker.ietf.org/doc/html/rfc9449)
mod dpop;
mod token;

use crate::login::cache::{load_cached_token, save_cached_token};
use crate::login::token::{LoginToken, LoginTokenError};
use crate::provider_config::ProviderConfig;
use aws_credential_types::credential_feature::AwsCredentialFeature;
use aws_credential_types::provider;
use aws_credential_types::provider::future;
use aws_credential_types::provider::ProvideCredentials;
use aws_sdk_signin::config::Builder as SignInClientConfigBuilder;
use aws_sdk_signin::operation::create_o_auth2_token::CreateOAuth2TokenError;
use aws_sdk_signin::types::{CreateOAuth2TokenRequestBody, OAuth2ErrorCode};
use aws_sdk_signin::Client as SignInClient;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_runtime::expiring_cache::ExpiringCache;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::SdkConfig;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

const REFRESH_BUFFER_TIME: Duration = Duration::from_secs(5 * 60 /* 5 minutes */);
const MIN_TIME_BETWEEN_REFRESH: Duration = Duration::from_secs(30);
pub(super) const PROVIDER_NAME: &str = "Login";

/// AWS credentials provider vended by AWS Sign-In. This provider allows users to acquire and refresh
/// AWS credentials that correspond to an AWS Console session.
///
/// See the [SDK developer guide](https://docs.aws.amazon.com/sdkref/latest/guide/access-login.html)
/// for more information on getting started with console sessions and the AWS CLI.
#[derive(Debug)]
pub struct LoginCredentialsProvider {
    inner: Arc<Inner>,
    token_cache: ExpiringCache<LoginToken, LoginTokenError>,
}

#[derive(Debug)]
struct Inner {
    fs: Fs,
    env: Env,
    session_arn: String,
    enabled_from_profile: bool,
    sdk_config: SdkConfig,
    time_source: SharedTimeSource,
    last_refresh_attempt: Mutex<Option<SystemTime>>,
}

impl LoginCredentialsProvider {
    /// Create a new [`Builder`] for the given login session ARN.
    ///
    /// The `session_arn` argument should take the form an Amazon Resource Name (ARN) like
    ///
    /// ```text
    /// arn:aws:iam::0123456789012:user/Admin
    /// ```
    pub fn builder(session_arn: impl Into<String>) -> Builder {
        Builder {
            session_arn: session_arn.into(),
            provider_config: None,
            enabled_from_profile: false,
        }
    }

    async fn resolve_token(&self) -> Result<LoginToken, LoginTokenError> {
        let token_cache = self.token_cache.clone();
        if let Some(token) = token_cache
            .yield_or_clear_if_expired(self.inner.time_source.now())
            .await
        {
            tracing::debug!("using cached Login token");
            return Ok(token);
        }

        let inner = self.inner.clone();
        let token = token_cache
            .get_or_load(|| async move {
                tracing::debug!("expiring cache asked for an updated Login token");
                let mut token =
                    load_cached_token(&inner.env, &inner.fs, &inner.session_arn).await?;

                tracing::debug!("loaded cached Login token");

                let now = inner.time_source.now();
                let expired = token.expires_at() <= now;
                let expires_soon = token.expires_at() - REFRESH_BUFFER_TIME <= now;
                let last_refresh = *inner.last_refresh_attempt.lock().unwrap();
                let min_time_passed = last_refresh
                    .map(|lr| {
                        now.duration_since(lr).expect("last_refresh is in the past")
                            >= MIN_TIME_BETWEEN_REFRESH
                    })
                    .unwrap_or(true);

                let refreshable = min_time_passed;

                tracing::debug!(
                    expired = ?expired,
                    expires_soon = ?expires_soon,
                    min_time_passed = ?min_time_passed,
                    refreshable = ?refreshable,
                    will_refresh = ?(expires_soon && refreshable),
                    "cached Login token refresh decision"
                );

                // Fail fast if the token has expired and we can't refresh it
                if expired && !refreshable {
                    tracing::debug!("cached Login token is expired and cannot be refreshed");
                    return Err(LoginTokenError::ExpiredToken);
                }

                // Refresh the token if it is going to expire soon
                if expires_soon && refreshable {
                    tracing::debug!("attempting to refresh Login token");
                    let refreshed_token = Self::refresh_cached_token(&inner, &token, now).await?;
                    token = refreshed_token;
                    *inner.last_refresh_attempt.lock().unwrap() = Some(now);
                }

                let expires_at = token.expires_at();
                Ok((token, expires_at))
            })
            .await?;

        Ok(token)
    }

    async fn refresh_cached_token(
        inner: &Inner,
        cached_token: &LoginToken,
        now: SystemTime,
    ) -> Result<LoginToken, LoginTokenError> {
        let dpop_auth_scheme = dpop::DPoPAuthScheme::new(&cached_token.dpop_key)?;
        let client_config = SignInClientConfigBuilder::from(&inner.sdk_config)
            .auth_scheme_resolver(dpop::DPoPAuthSchemeOptionResolver)
            .push_auth_scheme(dpop_auth_scheme)
            .build();

        let client = SignInClient::from_conf(client_config);

        let resp = client
            .create_o_auth2_token()
            .token_input(
                CreateOAuth2TokenRequestBody::builder()
                    .client_id(&cached_token.client_id)
                    .grant_type("refresh_token")
                    .refresh_token(cached_token.refresh_token.as_str())
                    .build()
                    .expect("valid CreateOAuth2TokenRequestBody"),
            )
            .send()
            .await
            .map_err(|err| {
                let service_err = err.into_service_error();
                let message = match &service_err {
                    CreateOAuth2TokenError::AccessDeniedException(e) => match e.error {
                        OAuth2ErrorCode::InsufficientPermissions => Some("Unable to refresh credentials due to insufficient permissions. You may be missing permission for the 'CreateOAuth2Token' action.".to_string()),
                        OAuth2ErrorCode::TokenExpired => Some("Your session has expired. Please reauthenticate.".to_string()),
                        OAuth2ErrorCode::UserCredentialsChanged => Some("Unable to refresh credentials because of a change in your password. Please reauthenticate with your new password.".to_string()),
                        _ => None,
                    }
                    _ => None,
                };

                LoginTokenError::RefreshFailed {
                    message,
                    source: service_err.into(),
                }
            })?;

        let token_output = resp.token_output.expect("valid token response");
        let new_token = LoginToken::from_refresh(cached_token, token_output, now);

        match save_cached_token(&inner.env, &inner.fs, &inner.session_arn, &new_token).await {
            Ok(_) => {}
            Err(e) => tracing::warn!("failed to save refreshed Login token: {e}"),
        }
        Ok(new_token)
    }

    async fn credentials(&self) -> provider::Result {
        let token = self.resolve_token().await?;

        let feat = match self.inner.enabled_from_profile {
            true => AwsCredentialFeature::CredentialsProfileLogin,
            false => AwsCredentialFeature::CredentialsProfile,
        };

        let mut creds = token.access_token;
        creds
            .get_property_mut_or_default::<Vec<AwsCredentialFeature>>()
            .push(feat);
        Ok(creds)
    }
}

impl ProvideCredentials for LoginCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> future::ProvideCredentials<'a>
    where
        Self: 'a,
    {
        future::ProvideCredentials::new(self.credentials())
    }
}

/// Builder for [`LoginCredentialsProvider`]
#[derive(Debug)]
pub struct Builder {
    session_arn: String,
    provider_config: Option<ProviderConfig>,
    enabled_from_profile: bool,
}

impl Builder {
    /// Override the configuration used for this provider
    pub fn configure(mut self, provider_config: &ProviderConfig) -> Self {
        self.provider_config = Some(provider_config.clone());
        self
    }

    /// Set whether this provider was enabled via a profile.
    /// Defaults to `false` (configured explicitly in user code).
    pub(crate) fn enabled_from_profile(mut self, enabled: bool) -> Self {
        self.enabled_from_profile = enabled;
        self
    }

    /// Construct a [`LoginCredentialsProvider`] from the builder
    pub fn build(self) -> LoginCredentialsProvider {
        let provider_config = self.provider_config.unwrap_or_default();
        let fs = provider_config.fs();
        let env = provider_config.env();
        let inner = Arc::new(Inner {
            fs,
            env,
            session_arn: self.session_arn,
            enabled_from_profile: self.enabled_from_profile,
            sdk_config: provider_config.client_config(),
            time_source: provider_config.time_source(),
            last_refresh_attempt: Mutex::new(None),
        });

        LoginCredentialsProvider {
            inner,
            token_cache: ExpiringCache::new(REFRESH_BUFFER_TIME),
        }
    }
}

#[cfg(test)]
mod test {
    //! Test suite for LoginCredentialsProvider
    //!
    //! This test module reads test cases from `test-data/login-provider-test-cases.json`
    //! and validates the behavior of the LoginCredentialsProvider against various scenarios
    //! from the SEP.
    use super::*;
    use crate::provider_config::ProviderConfig;
    use aws_credential_types::provider::ProvideCredentials;
    use aws_sdk_signin::config::RuntimeComponents;
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_async::test_util::instant_time_and_sleep;
    use aws_smithy_async::time::{SharedTimeSource, StaticTimeSource};
    use aws_smithy_runtime_api::client::{
        http::{
            HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings,
            SharedHttpConnector,
        },
        orchestrator::{HttpRequest, HttpResponse},
        result::ConnectorError,
    };
    use aws_smithy_types::body::SdkBody;
    use aws_types::os_shim_internal::{Env, Fs};
    use aws_types::region::Region;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::error::Error;
    use std::time::{Duration, UNIX_EPOCH};

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    struct LoginTestCase {
        documentation: String,
        config_contents: String,
        cache_contents: HashMap<String, serde_json::Value>,
        #[serde(default)]
        mock_api_calls: Vec<MockApiCall>,
        outcomes: Vec<Outcome>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct MockApiCall {
        #[serde(default)]
        response: Option<MockResponse>,
        #[serde(default)]
        response_code: Option<u16>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct MockResponse {
        token_output: TokenOutput,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct TokenOutput {
        access_token: AccessToken,
        refresh_token: String,
        expires_in: u64,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    struct AccessToken {
        access_key_id: String,
        secret_access_key: String,
        session_token: String,
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "result")]
    enum Outcome {
        #[serde(rename = "credentials")]
        Credentials {
            #[serde(rename = "accessKeyId")]
            access_key_id: String,
            #[serde(rename = "secretAccessKey")]
            secret_access_key: String,
            #[serde(rename = "sessionToken")]
            session_token: String,
            #[serde(rename = "accountId")]
            account_id: String,
            #[serde(default, rename = "expiresAt")]
            #[allow(dead_code)]
            expires_at: Option<String>,
        },
        #[serde(rename = "error")]
        Error,
        #[serde(rename = "cacheContents")]
        CacheContents(HashMap<String, serde_json::Value>),
    }

    impl LoginTestCase {
        async fn check(&self) {
            let session_arn = "arn:aws:sts::012345678910:assumed-role/Admin/admin";

            // Fixed time for testing: 2025-11-19T00:00:00Z
            let now = UNIX_EPOCH + Duration::from_secs(1763510400);
            let time_source = SharedTimeSource::new(StaticTimeSource::new(now));

            // Setup filesystem with cache and config contents
            let mut fs_map = HashMap::new();
            fs_map.insert(
                "/home/user/.aws/config".to_string(),
                self.config_contents.as_bytes().to_vec(),
            );
            for (filename, contents) in &self.cache_contents {
                let path = format!("/home/user/.aws/login/cache/{}", filename);
                // Add tokenType if missing (required by cache parser)
                let mut contents = contents.clone();
                if !contents.as_object().unwrap().contains_key("tokenType") {
                    contents.as_object_mut().unwrap().insert(
                        "tokenType".to_string(),
                        serde_json::Value::String("aws_sigv4".to_string()),
                    );
                }
                let json = serde_json::to_string(&contents).expect("valid json");
                fs_map.insert(path, json.into_bytes());
            }
            let fs = Fs::from_map(fs_map);

            let env = Env::from_slice(&[("HOME", "/home/user")]);

            // Setup mock HTTP client
            let http_client = if self.mock_api_calls.is_empty() {
                crate::test_case::no_traffic_client()
            } else {
                aws_smithy_runtime_api::client::http::SharedHttpClient::new(TestHttpClient::new(
                    &self.mock_api_calls,
                ))
            };

            let provider_config = ProviderConfig::empty()
                .with_env(env.clone())
                .with_fs(fs.clone())
                .with_http_client(http_client)
                .with_region(Some(Region::from_static("us-east-2")))
                .with_sleep_impl(TokioSleep::new())
                .with_time_source(time_source);

            let provider = LoginCredentialsProvider::builder(session_arn)
                .configure(&provider_config)
                .build();

            // Call provider once and validate result against all outcomes
            let result = dbg!(provider.provide_credentials().await);

            for outcome in &self.outcomes {
                match outcome {
                    Outcome::Credentials {
                        access_key_id,
                        secret_access_key,
                        session_token,
                        account_id,
                        expires_at: _,
                    } => {
                        let creds = result.as_ref().expect("credentials should succeed");
                        assert_eq!(access_key_id, creds.access_key_id());
                        assert_eq!(secret_access_key, creds.secret_access_key());
                        assert_eq!(session_token, creds.session_token().unwrap());
                        assert_eq!(account_id, creds.account_id().unwrap().as_str());
                    }
                    Outcome::Error => {
                        result.as_ref().expect_err("should fail");
                    }
                    Outcome::CacheContents(expected_cache) => {
                        // Verify cache was updated after provider call
                        for (filename, expected) in expected_cache {
                            let path = format!("/home/user/.aws/login/cache/{}", filename);
                            let actual = fs.read_to_end(&path).await.expect("cache file exists");
                            let actual: serde_json::Value =
                                serde_json::from_slice(&actual).expect("valid json");
                            // Compare only the fields that matter (ignore formatting differences)
                            assert_eq!(
                                expected.get("accessToken"),
                                actual.get("accessToken"),
                                "accessToken mismatch for {}",
                                filename
                            );
                            assert_eq!(
                                expected.get("refreshToken"),
                                actual.get("refreshToken"),
                                "refreshToken mismatch for {}",
                                filename
                            );
                        }
                    }
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    struct TestHttpClient {
        inner: SharedHttpConnector,
    }

    impl TestHttpClient {
        fn new(mock_calls: &[MockApiCall]) -> Self {
            Self {
                inner: SharedHttpConnector::new(TestHttpConnector {
                    mock_calls: mock_calls.to_vec(),
                }),
            }
        }
    }

    impl HttpClient for TestHttpClient {
        fn http_connector(
            &self,
            _settings: &HttpConnectorSettings,
            _components: &RuntimeComponents,
        ) -> SharedHttpConnector {
            self.inner.clone()
        }
    }

    #[derive(Debug, Clone)]
    struct TestHttpConnector {
        mock_calls: Vec<MockApiCall>,
    }

    impl HttpConnector for TestHttpConnector {
        fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
            if let Some(mock) = self.mock_calls.first() {
                if let Some(code) = mock.response_code {
                    return HttpConnectorFuture::ready(Ok(HttpResponse::new(
                        code.try_into().unwrap(),
                        SdkBody::from("{\"error\":\"refresh_failed\"}"),
                    )));
                }
                if let Some(resp) = &mock.response {
                    let body = format!(
                        r#"{{
                            "accessToken": {{
                                "accessKeyId": "{}",
                                "secretAccessKey": "{}",
                                "sessionToken": "{}"
                            }},
                            "expiresIn": {},
                            "refreshToken": "{}"
                        }}"#,
                        resp.token_output.access_token.access_key_id,
                        resp.token_output.access_token.secret_access_key,
                        resp.token_output.access_token.session_token,
                        resp.token_output.expires_in,
                        resp.token_output.refresh_token
                    );
                    return HttpConnectorFuture::ready(Ok(HttpResponse::new(
                        200.try_into().unwrap(),
                        SdkBody::from(body),
                    )));
                }
            }
            HttpConnectorFuture::ready(Ok(HttpResponse::new(
                500.try_into().unwrap(),
                SdkBody::from("{\"error\":\"no_mock\"}"),
            )))
        }
    }

    #[tokio::test]
    async fn run_login_tests() -> Result<(), Box<dyn Error>> {
        let test_cases = std::fs::read_to_string("test-data/login-provider-test-cases.json")?;
        let test_cases: Vec<LoginTestCase> = serde_json::from_str(&test_cases)?;

        for (idx, test) in test_cases.iter().enumerate() {
            println!("Running test {}: {}", idx, test.documentation);
            test.check().await;
        }
        Ok(())
    }

    /// How the first `CreateOAuth2Token` attempt fails before the retry succeeds
    #[derive(Clone, Copy, Debug)]
    enum FirstAttempt {
        ServerError,
        ConnectionReset,
        Timeout,
    }

    #[derive(Debug)]
    struct Attempt {
        uri: String,
        dpop: Vec<String>,
        body: Vec<u8>,
    }

    /// Fails the first refresh attempt, answers every later one with a refreshed token, and
    /// records what each attempt sent
    #[derive(Clone, Debug)]
    struct FlakySignIn {
        first_attempt: FirstAttempt,
        attempts: Arc<Mutex<Vec<Attempt>>>,
    }

    impl HttpClient for FlakySignIn {
        fn http_connector(
            &self,
            _settings: &HttpConnectorSettings,
            _components: &RuntimeComponents,
        ) -> SharedHttpConnector {
            SharedHttpConnector::new(self.clone())
        }
    }

    impl HttpConnector for FlakySignIn {
        fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
            let mut attempts = self.attempts.lock().unwrap();
            attempts.push(Attempt {
                uri: request.uri().to_string(),
                dpop: request
                    .headers()
                    .get_all("dpop")
                    .map(str::to_string)
                    .collect(),
                body: request.body().bytes().unwrap_or_default().to_vec(),
            });
            if attempts.len() == 1 {
                return HttpConnectorFuture::ready(match self.first_attempt {
                    FirstAttempt::ServerError => Ok(HttpResponse::new(
                        500.try_into().unwrap(),
                        SdkBody::from("{}"),
                    )),
                    FirstAttempt::ConnectionReset => Err(ConnectorError::io(
                        std::io::Error::from(std::io::ErrorKind::ConnectionReset).into(),
                    )),
                    FirstAttempt::Timeout => Err(ConnectorError::timeout("read timed out".into())),
                });
            }
            HttpConnectorFuture::ready(Ok(HttpResponse::new(
                200.try_into().unwrap(),
                SdkBody::from(
                    r#"{
                        "accessToken": {
                            "accessKeyId": "NEWREFRESHEDKEY",
                            "secretAccessKey": "newRefreshedSecretKey",
                            "sessionToken": "newRefreshedSessionToken"
                        },
                        "expiresIn": 900,
                        "refreshToken": "new_refresh_token"
                    }"#,
                ),
            )))
        }
    }

    /// A refresh attempt that fails with a retryable error is retried with a freshly signed
    /// DPoP proof. RFC 9449 section 11.1 lets the server reject a proof whose `jti` it has
    /// already seen, so replaying the first attempt's proof would turn a transient failure
    /// into a failed refresh.
    async fn check_retried_refresh(first_attempt: FirstAttempt) {
        let session_arn = "arn:aws:sts::012345678910:assumed-role/Admin/admin";
        let cache_path = "/home/user/.aws/login/cache/4b0ba8f99f075c0633e122fd73346ce203a3faf18ea0310eb2d29df1bab2e255.json";
        let cached_token = serde_json::json!({
            "accessToken": {
                "accessKeyId": "OLDEXPIREDKEY",
                "secretAccessKey": "oldExpiredSecretKey",
                "sessionToken": "oldExpiredSessionToken",
                "accountId": "012345678901",
                "expiresAt": "2020-01-01T00:00:00Z"
            },
            "tokenType": "aws_sigv4",
            "clientId": "arn:aws:signin:::devtools/same-device",
            "refreshToken": "valid_refresh_token",
            "dpopKey": "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEEIPt/u8InPLpQeQLJTvVX+sNDzni8vMDMt3Liu+nMBigfoAoGCCqGSM49\nAwEHoUQDQgAEILkGG7rNOnxiIJlMgimY1UPP8eDMFP0DAY6WGjngP4bvTAiUCQ/I\nffut2379uP+OBCm2ovGpBOJRgrl1RspUOQ==\n-----END EC PRIVATE KEY-----\n"
        });
        let fs = Fs::from_map(HashMap::from([(
            cache_path.to_string(),
            serde_json::to_vec(&cached_token).unwrap(),
        )]));
        let sign_in = FlakySignIn {
            first_attempt,
            attempts: Default::default(),
        };
        // 2025-11-19T00:00:00Z, the same fixed time as the SEP test cases
        let (time_source, sleep) =
            instant_time_and_sleep(UNIX_EPOCH + Duration::from_secs(1763510400));
        let provider_config = ProviderConfig::empty()
            .with_env(Env::from_slice(&[("HOME", "/home/user")]))
            .with_fs(fs.clone())
            .with_http_client(sign_in.clone())
            .with_region(Some(Region::from_static("us-east-2")))
            .with_sleep_impl(sleep)
            .with_time_source(time_source);
        let provider = LoginCredentialsProvider::builder(session_arn)
            .configure(&provider_config)
            .build();

        let creds = provider
            .provide_credentials()
            .await
            .expect("the retry refreshes the token");
        assert_eq!("NEWREFRESHEDKEY", creds.access_key_id());

        {
            let attempts = sign_in.attempts.lock().unwrap();
            assert_eq!(2, attempts.len(), "{first_attempt:?} is retried once");
            let proofs: Vec<_> = attempts
                .iter()
                .map(|attempt| {
                    // RFC 9449 section 4.3 rejects a request with more than one DPoP header
                    assert_eq!(1, attempt.dpop.len(), "one DPoP proof per attempt");
                    let (header, payload) = dpop::test_util::verify_proof(&attempt.dpop[0]);
                    assert_eq!(attempt.uri, payload["htu"]);
                    (header, payload)
                })
                .collect();
            // The refresh token is bound to the cached key (RFC 9449 section 5). Coordinates come
            // from `openssl ec -pubout`, not from `p256`.
            assert_eq!(
                serde_json::json!({
                    "kty": "EC",
                    "crv": "P-256",
                    "x": "ILkGG7rNOnxiIJlMgimY1UPP8eDMFP0DAY6WGjngP4Y",
                    "y": "70wIlAkPyH37rdt-_bj_jgQptqLxqQTiUYK5dUbKVDk",
                }),
                proofs[0].0["jwk"]
            );
            assert_eq!(proofs[0].0, proofs[1].0, "both attempts prove the same key");
            assert_ne!(
                proofs[0].1["jti"], proofs[1].1["jti"],
                "a retried attempt must not replay the first attempt's proof"
            );
            assert_eq!(
                attempts[0].body, attempts[1].body,
                "only the proof changes between attempts"
            );
        }

        let cached: serde_json::Value =
            serde_json::from_slice(&fs.read_to_end(cache_path).await.unwrap()).unwrap();
        assert_eq!("NEWREFRESHEDKEY", cached["accessToken"]["accessKeyId"]);
        assert_eq!("new_refresh_token", cached["refreshToken"]);
        assert_eq!(
            cached_token["dpopKey"], cached["dpopKey"],
            "the refreshed session keeps the key it is bound to"
        );

        // The refreshed token is served from memory: one successful refresh, not one per call
        provider.provide_credentials().await.unwrap();
        assert_eq!(2, sign_in.attempts.lock().unwrap().len());
    }

    #[tokio::test]
    async fn retried_refresh_after_server_error_signs_a_fresh_proof() {
        check_retried_refresh(FirstAttempt::ServerError).await;
    }

    #[tokio::test]
    async fn retried_refresh_after_connection_reset_signs_a_fresh_proof() {
        check_retried_refresh(FirstAttempt::ConnectionReset).await;
    }

    #[tokio::test]
    async fn retried_refresh_after_timeout_signs_a_fresh_proof() {
        check_retried_refresh(FirstAttempt::Timeout).await;
    }
}
