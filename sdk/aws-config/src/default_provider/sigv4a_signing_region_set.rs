/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::provider_config::ProviderConfig;
use aws_runtime::env_config::EnvConfigValue;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_types::region::SigningRegionSet;
use std::fmt;

mod env {
    pub(super) const SIGV4A_SIGNING_REGION_SET: &str = "AWS_SIGV4A_SIGNING_REGION_SET";
}

mod profile_key {
    pub(super) const SIGV4A_SIGNING_REGION_SET: &str = "sigv4a_signing_region_set";
}

/// Load the value for the SigV4a signing region set.
///
/// Checks `AWS_SIGV4A_SIGNING_REGION_SET` env var, then `sigv4a_signing_region_set` profile key.
/// The value is a comma-delimited list of region names.
pub(crate) async fn sigv4a_signing_region_set_provider(
    provider_config: &ProviderConfig,
) -> Option<SigningRegionSet> {
    let env = provider_config.env();
    let profiles = provider_config.profile().await;

    EnvConfigValue::new()
        .env(env::SIGV4A_SIGNING_REGION_SET)
        .profile(profile_key::SIGV4A_SIGNING_REGION_SET)
        .validate(&env, profiles, parse_signing_region_set)
        .map_err(|err| {
            tracing::warn!(
                err = %DisplayErrorContext(&err),
                "invalid value for sigv4a signing region set"
            )
        })
        .unwrap_or(None)
}

fn parse_signing_region_set(csv: &str) -> Result<SigningRegionSet, InvalidSigningRegionSet> {
    let region_set: SigningRegionSet = csv
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if region_set.as_ref().is_empty() {
        return Err(InvalidSigningRegionSet {
            value: format!("Empty value in `{csv}`."),
        });
    }
    Ok(region_set)
}

#[derive(Debug)]
struct InvalidSigningRegionSet {
    value: String,
}

impl fmt::Display for InvalidSigningRegionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Not a valid signing region set: {}", self.value)
    }
}

impl std::error::Error for InvalidSigningRegionSet {}

#[cfg(test)]
mod test {
    use super::env;
    use crate::{
        default_provider::sigv4a_signing_region_set::sigv4a_signing_region_set_provider,
        provider_config::ProviderConfig,
    };
    use aws_types::os_shim_internal::{Env, Fs};
    use aws_types::region::SigningRegionSet;
    use tracing_test::traced_test;

    #[tokio::test]
    async fn load_from_env_var() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[(env::SIGV4A_SIGNING_REGION_SET, "*")]));
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert_eq!(result, Some(SigningRegionSet::from("*")));
    }

    #[tokio::test]
    async fn load_from_env_var_multi_region() {
        let conf = ProviderConfig::empty().with_env(Env::from_slice(&[(
            env::SIGV4A_SIGNING_REGION_SET,
            "us-east-1, eu-west-1",
        )]));
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_ref(), "us-east-1,eu-west-1");
    }

    #[tokio::test]
    async fn load_from_profile() {
        let conf = ProviderConfig::empty()
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(Fs::from_slice(&[(
                "conf",
                "[default]\nsigv4a_signing_region_set = *",
            )]));
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert_eq!(result, Some(SigningRegionSet::from("*")));
    }

    #[tokio::test]
    async fn env_var_wins_over_profile() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[(
                env::SIGV4A_SIGNING_REGION_SET,
                "us-east-1",
            )]))
            .with_profile_config(
                Some(
                    #[allow(deprecated)]
                    crate::profile::profile_file::ProfileFiles::builder()
                        .with_file(
                            #[allow(deprecated)]
                            crate::profile::profile_file::ProfileFileKind::Config,
                            "conf",
                        )
                        .build(),
                ),
                None,
            )
            .with_fs(Fs::from_slice(&[(
                "conf",
                "[default]\nsigv4a_signing_region_set = *",
            )]));
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert_eq!(result, Some(SigningRegionSet::from("us-east-1")));
    }

    #[tokio::test]
    async fn returns_none_when_not_configured() {
        let conf = ProviderConfig::empty();
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[traced_test]
    async fn log_error_on_empty_value() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[(env::SIGV4A_SIGNING_REGION_SET, "")]));
        let result = sigv4a_signing_region_set_provider(&conf).await;
        assert_eq!(result, None);
        assert!(logs_contain("invalid value for sigv4a signing region set"));
    }
}
