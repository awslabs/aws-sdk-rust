/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Load timeout configuration properties from environment variables

use aws_smithy_types::timeout::{parse_str_as_timeout, TimeoutConfig, TimeoutConfigError};
use aws_types::os_shim_internal::Env;
use std::time::Duration;

const ENV_VAR_CONNECT_TIMEOUT: &str = "AWS_CONNECT_TIMEOUT";
const ENV_VAR_TLS_NEGOTIATION_TIMEOUT: &str = "AWS_TLS_NEGOTIATION_TIMEOUT";
const ENV_VAR_READ_TIMEOUT: &str = "AWS_READ_TIMEOUT";
const ENV_VAR_API_CALL_ATTEMPT_TIMEOUT: &str = "AWS_API_CALL_ATTEMPT_TIMEOUT";
const ENV_VAR_API_CALL_TIMEOUT: &str = "AWS_API_CALL_TIMEOUT";

/// Load a timeout_config from environment variables
///
/// This provider will check the values of the following variables in order to build a `TimeoutConfig`
///
/// - `AWS_CONNECT_TIMEOUT`
/// - `AWS_TLS_NEGOTIATION_TIMEOUT`
/// - `AWS_READ_TIMEOUT`
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

    /// Attempt to create a new [`TimeoutConfig`] from environment variables
    pub fn timeout_config(&self) -> Result<TimeoutConfig, TimeoutConfigError> {
        let connect_timeout = construct_timeout_from_env_var(&self.env, ENV_VAR_CONNECT_TIMEOUT)?;
        let tls_negotiation_timeout =
            construct_timeout_from_env_var(&self.env, ENV_VAR_TLS_NEGOTIATION_TIMEOUT)?;
        let read_timeout = construct_timeout_from_env_var(&self.env, ENV_VAR_READ_TIMEOUT)?;
        let api_call_attempt_timeout =
            construct_timeout_from_env_var(&self.env, ENV_VAR_API_CALL_ATTEMPT_TIMEOUT)?;
        let api_call_timeout = construct_timeout_from_env_var(&self.env, ENV_VAR_API_CALL_TIMEOUT)?;

        Ok(TimeoutConfig::new()
            .with_connect_timeout(connect_timeout)
            .with_tls_negotiation_timeout(tls_negotiation_timeout)
            .with_read_timeout(read_timeout)
            .with_api_call_attempt_timeout(api_call_attempt_timeout)
            .with_api_call_timeout(api_call_timeout))
    }
}

fn construct_timeout_from_env_var(
    env: &Env,
    var: &'static str,
) -> Result<Option<Duration>, TimeoutConfigError> {
    match env.get(var).ok() {
        Some(timeout) => {
            parse_str_as_timeout(&timeout, var.into(), "environment variable".into()).map(Some)
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod test {
    use super::{
        EnvironmentVariableTimeoutConfigProvider, ENV_VAR_API_CALL_ATTEMPT_TIMEOUT,
        ENV_VAR_API_CALL_TIMEOUT, ENV_VAR_CONNECT_TIMEOUT, ENV_VAR_READ_TIMEOUT,
        ENV_VAR_TLS_NEGOTIATION_TIMEOUT,
    };
    use aws_smithy_types::timeout::TimeoutConfig;
    use aws_types::os_shim_internal::Env;
    use std::time::Duration;

    fn test_provider(vars: &[(&str, &str)]) -> EnvironmentVariableTimeoutConfigProvider {
        EnvironmentVariableTimeoutConfigProvider::new_with_env(Env::from_slice(vars))
    }

    #[test]
    fn no_defaults() {
        let built = test_provider(&[]).timeout_config().unwrap();

        assert_eq!(built.read_timeout(), None);
        assert_eq!(built.connect_timeout(), None);
        assert_eq!(built.tls_negotiation_timeout(), None);
        assert_eq!(built.api_call_attempt_timeout(), None);
        assert_eq!(built.api_call_timeout(), None);
    }

    #[test]
    fn all_fields_can_be_set_at_once() {
        assert_eq!(
            test_provider(&[
                (ENV_VAR_READ_TIMEOUT, "1.0"),
                (ENV_VAR_CONNECT_TIMEOUT, "2"),
                (ENV_VAR_TLS_NEGOTIATION_TIMEOUT, "3.0000"),
                (ENV_VAR_API_CALL_ATTEMPT_TIMEOUT, "04.000"),
                (ENV_VAR_API_CALL_TIMEOUT, "900012345.0")
            ])
            .timeout_config()
            .unwrap(),
            TimeoutConfig::new()
                .with_read_timeout(Some(Duration::from_secs_f32(1.0)))
                .with_connect_timeout(Some(Duration::from_secs_f32(2.0)))
                .with_tls_negotiation_timeout(Some(Duration::from_secs_f32(3.0)))
                .with_api_call_attempt_timeout(Some(Duration::from_secs_f32(4.0)))
                // Some floats can't be represented as f32 so this duration will be equal to the
                // duration from the env.
                .with_api_call_timeout(Some(Duration::from_secs_f32(900012350.0)))
        );
    }
}
