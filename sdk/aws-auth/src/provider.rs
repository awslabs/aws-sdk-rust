/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::{Credentials, CredentialsError, ProvideCredentials};
use std::collections::HashMap;
use std::env::VarError;

/// Load Credentials from Environment Variables
pub struct EnvironmentVariableCredentialsProvider {
    env: Box<dyn Fn(&str) -> Result<String, VarError> + Send + Sync>,
}

impl EnvironmentVariableCredentialsProvider {
    pub fn new() -> Self {
        EnvironmentVariableCredentialsProvider { env: Box::new(var) }
    }

    /// Create a EnvironmentVariable provider from a HashMap for testing
    pub fn for_map(env: HashMap<String, String>) -> Self {
        EnvironmentVariableCredentialsProvider {
            env: Box::new(move |key: &str| {
                env.get(key)
                    .ok_or(VarError::NotPresent)
                    .map(|k| k.to_string())
            }),
        }
    }
}

impl Default for EnvironmentVariableCredentialsProvider {
    fn default() -> Self {
        Self::new()
    }
}

fn var(key: &str) -> Result<String, VarError> {
    std::env::var(key)
}

const ENV_PROVIDER: &str = "EnvironmentVariable";

impl ProvideCredentials for EnvironmentVariableCredentialsProvider {
    fn provide_credentials(&self) -> Result<Credentials, CredentialsError> {
        let access_key = (self.env)("AWS_ACCESS_KEY_ID").map_err(to_cred_error)?;
        let secret_key = (self.env)("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| (self.env)("SECRET_ACCESS_KEY"))
            .map_err(to_cred_error)?;
        let session_token = (self.env)("AWS_SESSION_TOKEN").ok();
        Ok(Credentials {
            access_key_id: access_key,
            secret_access_key: secret_key,
            session_token,
            expires_after: None,
            provider_name: ENV_PROVIDER,
        })
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
    use crate::provider::EnvironmentVariableCredentialsProvider;
    use crate::{CredentialsError, ProvideCredentials};
    use std::collections::HashMap;

    #[test]
    fn valid_no_token() {
        let mut env = HashMap::new();
        env.insert("AWS_ACCESS_KEY_ID".to_owned(), "access".to_owned());
        env.insert("AWS_SECRET_ACCESS_KEY".to_owned(), "secret".to_owned());

        let provider = EnvironmentVariableCredentialsProvider::for_map(env);
        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token, None);
        assert_eq!(creds.access_key_id, "access");
        assert_eq!(creds.secret_access_key, "secret");
    }

    #[test]
    fn valid_with_token() {
        let mut env = HashMap::new();
        env.insert("AWS_ACCESS_KEY_ID".to_owned(), "access".to_owned());
        env.insert("AWS_SECRET_ACCESS_KEY".to_owned(), "secret".to_owned());
        env.insert("AWS_SESSION_TOKEN".to_owned(), "token".to_owned());

        let provider = EnvironmentVariableCredentialsProvider::for_map(env);
        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token.unwrap(), "token");
        assert_eq!(creds.access_key_id, "access");
        assert_eq!(creds.secret_access_key, "secret");
    }

    #[test]
    fn secret_key_fallback() {
        let mut env = HashMap::new();
        env.insert("AWS_ACCESS_KEY_ID".to_owned(), "access".to_owned());
        env.insert("SECRET_ACCESS_KEY".to_owned(), "secret".to_owned());
        env.insert("AWS_SESSION_TOKEN".to_owned(), "token".to_owned());

        let provider = EnvironmentVariableCredentialsProvider::for_map(env);
        let creds = provider.provide_credentials().expect("valid credentials");
        assert_eq!(creds.session_token.unwrap(), "token");
        assert_eq!(creds.access_key_id, "access");
        assert_eq!(creds.secret_access_key, "secret");
    }

    #[test]
    fn missing() {
        let env = HashMap::new();
        let provider = EnvironmentVariableCredentialsProvider::for_map(env);
        let err = provider
            .provide_credentials()
            .expect_err("no credentials defined");
        match err {
            CredentialsError::Unhandled(_) => panic!("wrong error type"),
            _ => (),
        };
    }

    #[test]
    fn real_environment() {
        let provider = EnvironmentVariableCredentialsProvider::new();
        // we don't know what's in the env, just make sure it doesn't crash.
        let _ = provider.provide_credentials();
    }
}
