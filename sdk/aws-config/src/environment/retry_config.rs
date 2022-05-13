/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::str::FromStr;

use aws_smithy_types::retry::{RetryConfigBuilder, RetryConfigErr, RetryMode};
use aws_types::os_shim_internal::Env;

const ENV_VAR_MAX_ATTEMPTS: &str = "AWS_MAX_ATTEMPTS";
const ENV_VAR_RETRY_MODE: &str = "AWS_RETRY_MODE";

/// Load a retry_config from environment variables
///
/// This provider will check the values of `AWS_RETRY_MODE` and `AWS_MAX_ATTEMPTS`
/// in order to build a retry config.
#[derive(Debug, Default)]
pub struct EnvironmentVariableRetryConfigProvider {
    env: Env,
}

impl EnvironmentVariableRetryConfigProvider {
    /// Create a new [`EnvironmentVariableRetryConfigProvider`]
    pub fn new() -> Self {
        EnvironmentVariableRetryConfigProvider { env: Env::real() }
    }

    #[doc(hidden)]
    /// Create an retry_config provider from a given `Env`
    ///
    /// This method is used for tests that need to override environment variables.
    pub fn new_with_env(env: Env) -> Self {
        EnvironmentVariableRetryConfigProvider { env }
    }

    /// Attempt to create a new `RetryConfig` from environment variables
    pub fn retry_config_builder(&self) -> Result<RetryConfigBuilder, RetryConfigErr> {
        let max_attempts = match self.env.get(ENV_VAR_MAX_ATTEMPTS).ok() {
            Some(max_attempts) => match max_attempts.parse::<u32>() {
                Ok(max_attempts) if max_attempts == 0 => {
                    return Err(RetryConfigErr::MaxAttemptsMustNotBeZero {
                        set_by: "environment variable".into(),
                    });
                }
                Ok(max_attempts) => Some(max_attempts),
                Err(source) => {
                    return Err(RetryConfigErr::FailedToParseMaxAttempts {
                        set_by: "environment variable".into(),
                        source,
                    });
                }
            },
            None => None,
        };

        let retry_mode = match self.env.get(ENV_VAR_RETRY_MODE) {
            Ok(retry_mode) => match RetryMode::from_str(&retry_mode) {
                Ok(retry_mode) => Some(retry_mode),
                Err(retry_mode_err) => {
                    return Err(RetryConfigErr::InvalidRetryMode {
                        set_by: "environment variable".into(),
                        source: retry_mode_err,
                    });
                }
            },
            Err(_) => None,
        };

        let mut retry_config_builder = RetryConfigBuilder::new();
        retry_config_builder
            .set_max_attempts(max_attempts)
            .set_mode(retry_mode);

        Ok(retry_config_builder)
    }
}

#[cfg(test)]
mod test {
    use aws_smithy_types::retry::{RetryConfig, RetryConfigErr, RetryMode};
    use aws_types::os_shim_internal::Env;

    use super::{EnvironmentVariableRetryConfigProvider, ENV_VAR_MAX_ATTEMPTS, ENV_VAR_RETRY_MODE};

    fn test_provider(vars: &[(&str, &str)]) -> EnvironmentVariableRetryConfigProvider {
        EnvironmentVariableRetryConfigProvider::new_with_env(Env::from_slice(vars))
    }

    #[test]
    fn defaults() {
        let built = test_provider(&[]).retry_config_builder().unwrap().build();

        assert_eq!(built.mode(), RetryMode::Standard);
        assert_eq!(built.max_attempts(), 3);
    }

    #[test]
    fn max_attempts_is_read_correctly() {
        assert_eq!(
            test_provider(&[(ENV_VAR_MAX_ATTEMPTS, "88")])
                .retry_config_builder()
                .unwrap()
                .build(),
            RetryConfig::new().with_max_attempts(88)
        );
    }

    #[test]
    fn max_attempts_errors_when_it_cant_be_parsed_as_an_integer() {
        assert!(matches!(
            test_provider(&[(ENV_VAR_MAX_ATTEMPTS, "not an integer")])
                .retry_config_builder()
                .unwrap_err(),
            RetryConfigErr::FailedToParseMaxAttempts { .. }
        ));
    }

    #[test]
    fn retry_mode_is_read_correctly() {
        assert_eq!(
            test_provider(&[(ENV_VAR_RETRY_MODE, "standard")])
                .retry_config_builder()
                .unwrap()
                .build(),
            RetryConfig::new().with_retry_mode(RetryMode::Standard)
        );
    }

    #[test]
    fn both_fields_can_be_set_at_once() {
        assert_eq!(
            test_provider(&[
                (ENV_VAR_RETRY_MODE, "standard"),
                (ENV_VAR_MAX_ATTEMPTS, "13")
            ])
            .retry_config_builder()
            .unwrap()
            .build(),
            RetryConfig::new()
                .with_max_attempts(13)
                .with_retry_mode(RetryMode::Standard)
        );
    }

    #[test]
    fn disallow_zero_max_attempts() {
        let err = test_provider(&[(ENV_VAR_MAX_ATTEMPTS, "0")])
            .retry_config_builder()
            .unwrap_err();
        assert!(matches!(
            err,
            RetryConfigErr::MaxAttemptsMustNotBeZero { .. }
        ));
    }
}
