/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_types::app_name::AppName;
use aws_types::os_shim_internal::Env;

/// Load an app name from the `AWS_SDK_UA_APP_ID` environment variable.
#[derive(Debug, Default)]
pub struct EnvironmentVariableAppNameProvider {
    env: Env,
}

impl EnvironmentVariableAppNameProvider {
    /// Create a new `EnvironmentVariableAppNameProvider`
    pub fn new() -> Self {
        Self { env: Env::real() }
    }

    #[doc(hidden)]
    /// Create an region provider from a given `Env`
    ///
    /// This method is used for tests that need to override environment variables.
    pub fn new_with_env(env: Env) -> Self {
        Self { env }
    }

    /// Attempts to create an `AppName` from the `AWS_SDK_UA_APP_ID` environment variable.
    pub fn app_name(&self) -> Option<AppName> {
        if let Ok(name) = self.env.get("AWS_SDK_UA_APP_ID") {
            match AppName::new(name) {
                Ok(name) => Some(name),
                Err(err) => {
                    tracing::warn!(err = %err, "`AWS_SDK_UA_APP_ID` environment variable value was invalid");
                    None
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::environment::EnvironmentVariableAppNameProvider;
    use aws_types::app_name::AppName;
    use aws_types::os_shim_internal::Env;
    use std::collections::HashMap;

    #[test]
    fn env_var_not_set() {
        let provider = EnvironmentVariableAppNameProvider::new_with_env(Env::from(HashMap::new()));
        assert_eq!(None, provider.app_name());
    }

    #[test]
    fn env_var_set() {
        let provider = EnvironmentVariableAppNameProvider::new_with_env(Env::from(
            vec![("AWS_SDK_UA_APP_ID".to_string(), "something".to_string())]
                .into_iter()
                .collect::<HashMap<String, String>>(),
        ));
        assert_eq!(
            Some(AppName::new("something").unwrap()),
            provider.app_name()
        );
    }
}
