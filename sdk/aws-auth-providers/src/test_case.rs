/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![cfg(test)]

use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use aws_auth::provider::{AsyncProvideCredentials, CredentialsResult};
use aws_hyper::DynConnector;
use aws_types::os_shim_internal::{Env, Fs};
use serde::Deserialize;
use smithy_client::dvr::{NetworkTraffic, RecordingConnection, ReplayingConnection};

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
impl From<&aws_auth::Credentials> for Credentials {
    fn from(credentials: &aws_auth::Credentials) -> Self {
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

/// Credentials test environment
///
/// A credentials test environment is a directory containing:
/// - an `fs` directory. This is loaded into the test as if it was mounted at `/`
/// - an `env.json` file containing environment variables
/// - an  `http-traffic.json` file containing an http traffic log from [`dvr`](smithy_client::dvr)
/// - a `test-case.json` file defining the expected output of the test
pub struct TestEnvironment {
    env: Env,
    fs: Fs,
    network_traffic: NetworkTraffic,
    metadata: Metadata,
    base_dir: PathBuf,
}

#[derive(Deserialize)]
enum TestResult {
    Ok(Credentials),
    ErrorContains(String),
}

#[derive(Deserialize)]
pub struct Metadata {
    result: TestResult,
    docs: String,
    name: String,
}

impl TestEnvironment {
    pub fn from_dir(dir: impl AsRef<Path>) -> Result<TestEnvironment, Box<dyn Error>> {
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
        Ok(TestEnvironment {
            base_dir: dir.into(),
            env,
            fs,
            network_traffic,
            metadata,
        })
    }

    /// Execute the test suite & record a new traffic log
    ///
    /// A connector will be created with the factory, then request traffic will be recorded.
    /// Response are generated from the existing http-traffic.json.
    pub async fn execute_and_update<P>(&self, make_provider: impl Fn(Fs, Env, DynConnector) -> P)
    where
        P: AsyncProvideCredentials,
    {
        let connector = RecordingConnection::new(ReplayingConnection::new(
            self.network_traffic.events().clone(),
        ));
        let provider = make_provider(
            self.fs.clone(),
            self.env.clone(),
            DynConnector::new(connector.clone()),
        );
        let result = provider.provide_credentials().await;
        std::fs::write(
            self.base_dir.join("http-traffic-recorded.json"),
            serde_json::to_string(&connector.network_traffic()).unwrap(),
        )
        .unwrap();
        self.check_results(&result);
    }

    fn log_info(&self) {
        eprintln!("test case: {}. {}", self.metadata.name, self.metadata.docs);
    }

    /// Execute a test case. Failures lead to panics.
    pub async fn execute<P>(&self, make_provider: impl Fn(Fs, Env, DynConnector) -> P)
    where
        P: AsyncProvideCredentials,
    {
        let connector = ReplayingConnection::new(self.network_traffic.events().clone());
        let provider = make_provider(
            self.fs.clone(),
            self.env.clone(),
            DynConnector::new(connector.clone()),
        );
        let result = provider.provide_credentials().await;
        self.log_info();
        self.check_results(&result);
        // todo: validate bodies
        match connector.validate(&["CONTENT-TYPE", "HOST"], |_expected, _actual| Ok(())) {
            Ok(()) => {}
            Err(e) => panic!("{}", e),
        }
    }

    fn check_results(&self, result: &CredentialsResult) {
        match (&result, &self.metadata.result) {
            (Ok(actual), TestResult::Ok(expected)) => {
                assert_eq!(
                    expected,
                    &Credentials::from(actual),
                    "incorrect credentials were returned"
                )
            }
            (Err(err), TestResult::ErrorContains(substr)) => {
                assert!(
                    format!("{}", err).contains(substr),
                    "`{}` did not contain `{}`",
                    err,
                    substr
                )
            }
            (Err(actual_error), TestResult::Ok(expected_creds)) => panic!(
                "expected credentials ({:?}) but an error was returned: {}",
                expected_creds, actual_error
            ),
            (Ok(creds), TestResult::ErrorContains(substr)) => panic!(
                "expected an error containing: `{}`, but credentials were returned: {:?}",
                substr, creds
            ),
        }
    }
}
