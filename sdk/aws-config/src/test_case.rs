/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::default_provider::use_dual_stack::use_dual_stack_provider;
use crate::default_provider::use_fips::use_fips_provider;
use crate::provider_config::ProviderConfig;
use aws_credential_types::provider::{self, ProvideCredentials};
use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep, TokioSleep};
use aws_smithy_runtime::client::http::test_util::dvr::{
    NetworkTraffic, RecordingClient, ReplayingClient,
};
use aws_smithy_runtime::test_util::capture_test_logs::capture_test_logs;
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::sdk_config::SharedHttpClient;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

/// Test case credentials
///
/// Credentials for use in test cases. These implement Serialize/Deserialize and have a
/// non-hidden debug implementation.
#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Credentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    expiry: Option<u64>,
}

/// Convert real credentials to test credentials
///
/// Comparing equality on real credentials works, but it's a pain because the Debug implementation
/// hides the actual keys
impl From<&aws_credential_types::Credentials> for Credentials {
    fn from(credentials: &aws_credential_types::Credentials) -> Self {
        Self {
            access_key_id: credentials.access_key_id().into(),
            secret_access_key: credentials.secret_access_key().into(),
            session_token: credentials.session_token().map(ToString::to_string),
            expiry: credentials
                .expiry()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs()),
        }
    }
}

impl From<aws_credential_types::Credentials> for Credentials {
    fn from(credentials: aws_credential_types::Credentials) -> Self {
        (&credentials).into()
    }
}

/// Credentials test environment
///
/// A credentials test environment is a directory containing:
/// - an `fs` directory. This is loaded into the test as if it was mounted at `/`
/// - an `env.json` file containing environment variables
/// - an  `http-traffic.json` file containing an http traffic log from [`dvr`](aws_smithy_runtime::client::http::test_utils::dvr)
/// - a `test-case.json` file defining the expected output of the test
pub(crate) struct TestEnvironment {
    metadata: Metadata,
    base_dir: PathBuf,
    http_client: ReplayingClient,
    provider_config: ProviderConfig,
}

/// Connector which expects no traffic
pub(crate) fn no_traffic_client() -> SharedHttpClient {
    ReplayingClient::new(Vec::new()).into_shared()
}

#[derive(Debug)]
pub(crate) struct InstantSleep;
impl AsyncSleep for InstantSleep {
    fn sleep(&self, _duration: Duration) -> Sleep {
        Sleep::new(std::future::ready(()))
    }
}

#[derive(Deserialize, Debug)]
pub(crate) enum GenericTestResult<T> {
    Ok(T),
    ErrorContains(String),
}

impl<T> GenericTestResult<T>
where
    T: PartialEq + Debug,
{
    #[track_caller]
    pub(crate) fn assert_matches(&self, result: Result<impl Into<T>, impl Error>) {
        match (result, &self) {
            (Ok(actual), GenericTestResult::Ok(expected)) => {
                assert_eq!(expected, &actual.into(), "incorrect result was returned")
            }
            (Err(err), GenericTestResult::ErrorContains(substr)) => {
                let message = format!("{}", DisplayErrorContext(&err));
                assert!(
                    message.contains(substr),
                    "`{message}` did not contain `{substr}`"
                );
            }
            (Err(actual_error), GenericTestResult::Ok(expected_creds)) => panic!(
                "expected credentials ({:?}) but an error was returned: {}",
                expected_creds,
                DisplayErrorContext(&actual_error)
            ),
            (Ok(creds), GenericTestResult::ErrorContains(substr)) => panic!(
                "expected an error containing: `{}`, but a result was returned: {:?}",
                substr,
                creds.into()
            ),
        }
    }
}

type TestResult = GenericTestResult<Credentials>;

#[derive(Deserialize)]
pub(crate) struct Metadata {
    result: TestResult,
    docs: String,
    name: String,
}

impl TestEnvironment {
    pub(crate) async fn from_dir(dir: impl AsRef<Path>) -> Result<TestEnvironment, Box<dyn Error>> {
        let dir = dir.as_ref();
        let env = std::fs::read_to_string(dir.join("env.json"))
            .map_err(|e| format!("failed to load env: {}", e))?;
        let env: HashMap<String, String> =
            serde_json::from_str(&env).map_err(|e| format!("failed to parse env: {}", e))?;
        let env = Env::from(env);
        let fs = Fs::from_test_dir(dir.join("fs"), "/");
        let network_traffic = std::fs::read_to_string(dir.join("http-traffic.json"))
            .map_err(|e| format!("failed to load http traffic: {}", e))?;
        let network_traffic: NetworkTraffic = serde_json::from_str(&network_traffic)?;

        let metadata: Metadata = serde_json::from_str(
            &std::fs::read_to_string(dir.join("test-case.json"))
                .map_err(|e| format!("failed to load test case: {}", e))?,
        )?;
        let http_client = ReplayingClient::new(network_traffic.events().clone());
        let provider_config = ProviderConfig::empty()
            .with_fs(fs.clone())
            .with_env(env.clone())
            .with_http_client(http_client.clone())
            .with_sleep_impl(TokioSleep::new())
            .load_default_region()
            .await;

        let use_dual_stack = use_dual_stack_provider(&provider_config).await;
        let use_fips = use_fips_provider(&provider_config).await;
        let provider_config = provider_config
            .with_use_fips(use_fips)
            .with_use_dual_stack(use_dual_stack);

        Ok(TestEnvironment {
            base_dir: dir.into(),
            metadata,
            http_client,
            provider_config,
        })
    }

    pub(crate) fn with_provider_config<F>(mut self, provider_config_builder: F) -> Self
    where
        F: Fn(ProviderConfig) -> ProviderConfig,
    {
        self.provider_config = provider_config_builder(self.provider_config.clone());
        self
    }

    pub(crate) fn provider_config(&self) -> &ProviderConfig {
        &self.provider_config
    }

    #[allow(unused)]
    #[cfg(all(feature = "client-hyper", feature = "rustls"))]
    /// Record a test case from live (remote) HTTPS traffic
    ///
    /// The `default_connector()` from the crate will be used
    pub(crate) async fn execute_from_live_traffic<F, P>(
        &self,
        make_provider: impl Fn(ProviderConfig) -> F,
    ) where
        F: Future<Output = P>,
        P: ProvideCredentials,
    {
        // swap out the connector generated from `http-traffic.json` for a real connector:
        let live_connector = aws_smithy_runtime::client::http::hyper_014::default_connector(
            &Default::default(),
            self.provider_config.sleep_impl(),
        )
        .expect("feature gate on this function makes this always return Some");
        let live_client = RecordingClient::new(live_connector);
        let config = self
            .provider_config
            .clone()
            .with_http_client(live_client.clone());
        let provider = make_provider(config).await;
        let result = provider.provide_credentials().await;
        std::fs::write(
            self.base_dir.join("http-traffic-recorded.json"),
            serde_json::to_string(&live_client.network_traffic()).unwrap(),
        )
        .unwrap();
        self.check_results(result);
    }

    #[allow(dead_code)]
    /// Execute the test suite & record a new traffic log
    ///
    /// A connector will be created with the factory, then request traffic will be recorded.
    /// Response are generated from the existing http-traffic.json.
    pub(crate) async fn execute_and_update<F, P>(&self, make_provider: impl Fn(ProviderConfig) -> F)
    where
        F: Future<Output = P>,
        P: ProvideCredentials,
    {
        let recording_client = RecordingClient::new(self.http_client.clone());
        let config = self
            .provider_config
            .clone()
            .with_http_client(recording_client.clone());
        let provider = make_provider(config).await;
        let result = provider.provide_credentials().await;
        std::fs::write(
            self.base_dir.join("http-traffic-recorded.json"),
            serde_json::to_string(&recording_client.network_traffic()).unwrap(),
        )
        .unwrap();
        self.check_results(result);
    }

    fn log_info(&self) {
        eprintln!("test case: {}. {}", self.metadata.name, self.metadata.docs);
    }

    fn lines_with_secrets<'a>(&'a self, logs: &'a str) -> Vec<&'a str> {
        logs.lines().filter(|l| self.contains_secret(l)).collect()
    }

    fn contains_secret(&self, log_line: &str) -> bool {
        assert!(log_line.lines().count() <= 1);
        match &self.metadata.result {
            // NOTE: we aren't currently erroring if the session token is leaked, that is in the canonical request among other things
            TestResult::Ok(creds) => log_line.contains(&creds.secret_access_key),
            TestResult::ErrorContains(_) => false,
        }
    }

    /// Execute a test case. Failures lead to panics.
    pub(crate) async fn execute<F, P>(&self, make_provider: impl Fn(ProviderConfig) -> F)
    where
        F: Future<Output = P>,
        P: ProvideCredentials,
    {
        let (_guard, rx) = capture_test_logs();
        let provider = make_provider(self.provider_config.clone()).await;
        let result = provider.provide_credentials().await;
        tokio::time::pause();
        self.log_info();
        self.check_results(result);
        // todo: validate bodies
        match self
            .http_client
            .clone()
            .validate(
                &["CONTENT-TYPE", "x-aws-ec2-metadata-token"],
                |_expected, _actual| Ok(()),
            )
            .await
        {
            Ok(()) => {}
            Err(e) => panic!("{}", e),
        }
        let contents = rx.contents();
        let leaking_lines = self.lines_with_secrets(&contents);
        assert!(
            leaking_lines.is_empty(),
            "secret was exposed\n{:?}\nSee the following log lines:\n  {}",
            self.metadata.result,
            leaking_lines.join("\n  ")
        )
    }

    #[track_caller]
    fn check_results(&self, result: provider::Result) {
        self.metadata.result.assert_matches(result);
    }
}
