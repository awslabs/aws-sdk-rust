/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::provider::{CredentialsError, ProvideCredentials};
use crate::Credentials;
use aws_types::os_shim_internal::Env;
use std::env::VarError;

/// Load Credentials from Environment Variables
pub struct EnvironmentVariableCredentialsProvider {
    env: Env,
}

impl EnvironmentVariableCredentialsProvider {
    pub fn new() -> Self {
        EnvironmentVariableCredentialsProvider { env: Env::real() }
    }
}

impl Default for EnvironmentVariableCredentialsProvider {
    fn default() -> Self {
        Self::new()
    }
}

const ENV_PROVIDER: &str = "EnvironmentVariable";

impl ProvideCredentials for EnvironmentVariableCredentialsProvider {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError> {
        let access_key = self.env.get("AWS_ACCESS_KEY_ID").map_err(to_cred_error)?;
        let secret_key = self
            .env
            .get("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| self.env.get("SECRET_ACCESS_KEY"))
            .map_err(to_cred_error)?;
        let session_token = self.env.get("AWS_SESSION_TOKEN").ok();
        Ok(Credentials::new(
            access_key,
            secret_key,
            session_token,
            None,
            ENV_PROVIDER,
        ))
    }
}

fn to_cred_error(err: VarError) -> CredentialsError {
    match err {
        VarError::NotPresent => CredentialsError::CredentialsNotLoaded,
        e @ VarError::NotUnicode(_) => CredentialsError::Unhandled(Box::new(e)),
    }
}

#[cfg(test)]
mod test {
    use super::EnvironmentVariableCredentialsProvider;
    use crate::provider::{CredentialsError, ProvideCredentials};
    use aws_types::os_shim_internal::Env;

    fn make_provider(vars: &[(&str, &str)]) -> EnvironmentVariableCredentialsProvider {
        EnvironmentVariableCredentialsProvider {
            env: Env::from_slice(vars),
        }
    }

    #[test]
    fn valid_no_token() {
        let provider = make_provider(&[
            ("AWS_ACCESS_KEY_ID", "access"),
            ("AWS_SECRET_ACCESS_KEY", "secret"),
        ]);
        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token(), None);
        assert_eq!(creds.access_key_id(), "access");
        assert_eq!(creds.secret_access_key(), "secret");
    }

    #[test]
    fn valid_with_token() {
        let provider = make_provider(&[
            ("AWS_ACCESS_KEY_ID", "access"),
            ("AWS_SECRET_ACCESS_KEY", "secret"),
            ("AWS_SESSION_TOKEN", "token"),
        ]);

        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token().unwrap(), "token");
        assert_eq!(creds.access_key_id(), "access");
        assert_eq!(creds.secret_access_key(), "secret");
    }

    #[test]
    fn secret_key_fallback() {
        let provider = make_provider(&[
            ("AWS_ACCESS_KEY_ID", "access"),
            ("SECRET_ACCESS_KEY", "secret"),
            ("AWS_SESSION_TOKEN", "token"),
        ]);

        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token().unwrap(), "token");
        assert_eq!(creds.access_key_id(), "access");
        assert_eq!(creds.secret_access_key(), "secret");
    }

    #[test]
    fn missing() {
        let provider = make_provider(&[]);
        let err = provider
            .provide_credentials()
            .expect_err("no credentials defined");
        if let CredentialsError::Unhandled(_) = err {
            panic!("wrong error type")
        };
    }

    #[test]
    fn real_environment() {
        let provider = EnvironmentVariableCredentialsProvider::new();
        // we don't know what's in the env, just make sure it doesn't crash.
        let _ = provider.provide_credentials();
    }
}
