/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Load timeout configuration properties from environment variables

use crate::parsing::parse_str_as_timeout;

use aws_smithy_types::timeout;
use aws_smithy_types::tristate::TriState;
use aws_types::os_shim_internal::Env;

use std::time::Duration;

// Currently unsupported timeouts
const ENV_VAR_CONNECT_TIMEOUT: &str = "AWS_CONNECT_TIMEOUT";
const ENV_VAR_TLS_NEGOTIATION_TIMEOUT: &str = "AWS_TLS_NEGOTIATION_TIMEOUT";
const ENV_VAR_READ_TIMEOUT: &str = "AWS_READ_TIMEOUT";

// Supported timeouts
const ENV_VAR_API_CALL_ATTEMPT_TIMEOUT: &str = "AWS_API_CALL_ATTEMPT_TIMEOUT";
const ENV_VAR_API_CALL_TIMEOUT: &str = "AWS_API_CALL_TIMEOUT";

/// Load a timeout_config from environment variables
///
/// This provider will check the values of the following variables in order to build a
/// [`timeout::Config`](aws_smithy_types::timeout::Config)
///
/// - `AWS_API_CALL_ATTEMPT_TIMEOUT`
/// - `AWS_API_CALL_TIMEOUT`
///
/// Timeout values represent the number of seconds before timing out and must be non-negative floats
/// or integers. NaN and infinity are also invalid.
#[derive(Debug, Default)]
pub struct EnvironmentVariableTimeoutConfigProvider {
    env: Env,
}

impl EnvironmentVariableTimeoutConfigProvider {
    /// Create a new [`EnvironmentVariableTimeoutConfigProvider`]
    pub fn new() -> Self {
        EnvironmentVariableTimeoutConfigProvider { env: Env::real() }
    }

    #[doc(hidden)]
    /// Create a timeout config provider from a given [`Env`]
    ///
    /// This method is used for tests that need to override environment variables.
    pub fn new_with_env(env: Env) -> Self {
        EnvironmentVariableTimeoutConfigProvider { env }
    }

    /// Attempt to create a new [`timeout::Config`](aws_smithy_types::timeout::Config) from environment variables
    pub fn timeout_config(&self) -> Result<timeout::Config, timeout::ConfigError> {
        // Warn users that set unsupported timeouts in their profile
        for timeout in [
            ENV_VAR_CONNECT_TIMEOUT,
            ENV_VAR_TLS_NEGOTIATION_TIMEOUT,
            ENV_VAR_READ_TIMEOUT,
        ] {
            warn_if_unsupported_timeout_is_set(&self.env, timeout);
        }

        let api_call_attempt_timeout =
            construct_timeout_from_env_var(&self.env, ENV_VAR_API_CALL_ATTEMPT_TIMEOUT)?;
        let api_call_timeout = construct_timeout_from_env_var(&self.env, ENV_VAR_API_CALL_TIMEOUT)?;

        let api_timeouts = timeout::Api::new()
            .with_call_timeout(api_call_timeout)
            .with_call_attempt_timeout(api_call_attempt_timeout);

        // Only API-related timeouts are currently supported
        Ok(timeout::Config::new().with_api_timeouts(api_timeouts))
    }
}

fn construct_timeout_from_env_var(
    env: &Env,
    var: &'static str,
) -> Result<TriState<Duration>, timeout::ConfigError> {
    match env.get(var).ok() {
        Some(timeout) => parse_str_as_timeout(&timeout, var.into(), "environment variable".into())
            .map(TriState::Set),
        None => Ok(TriState::Unset),
    }
}

fn warn_if_unsupported_timeout_is_set(env: &Env, var: &'static str) {
    if env.get(var).is_ok() {
        tracing::warn!(
                "Environment variable for '{}' timeout was set but that feature is currently unimplemented so the setting will be ignored. \
                To help us prioritize support for this feature, please upvote aws-sdk-rust#151 (https://github.com/awslabs/aws-sdk-rust/issues/151)",
            var
        )
    }
}

#[cfg(test)]
mod test {
    use super::{
        EnvironmentVariableTimeoutConfigProvider, ENV_VAR_API_CALL_ATTEMPT_TIMEOUT,
        ENV_VAR_API_CALL_TIMEOUT,
    };
    use aws_smithy_types::timeout;
    use aws_smithy_types::tristate::TriState;
    use aws_types::os_shim_internal::Env;
    use std::time::Duration;

    fn test_provider(vars: &[(&str, &str)]) -> EnvironmentVariableTimeoutConfigProvider {
        EnvironmentVariableTimeoutConfigProvider::new_with_env(Env::from_slice(vars))
    }

    #[test]
    fn no_defaults() {
        let built = test_provider(&[]).timeout_config().unwrap();

        assert_eq!(built.api.call_timeout(), TriState::Unset);
        assert_eq!(built.api.call_attempt_timeout(), TriState::Unset);
    }

    #[test]
    fn all_fields_can_be_set_at_once() {
        let expected_api_timeouts = timeout::Api::new()
            .with_call_attempt_timeout(TriState::Set(Duration::from_secs_f32(4.0)))
            // Some floats can't be represented as f32 so this duration will end up equalling the
            // duration from the env.
            .with_call_timeout(TriState::Set(Duration::from_secs_f32(900012350.0)));
        let expected_timeouts = timeout::Config::new().with_api_timeouts(expected_api_timeouts);

        assert_eq!(
            test_provider(&[
                (ENV_VAR_API_CALL_ATTEMPT_TIMEOUT, "04.000"),
                (ENV_VAR_API_CALL_TIMEOUT, "900012345.0")
            ])
            .timeout_config()
            .unwrap(),
            expected_timeouts
        );
    }
}
