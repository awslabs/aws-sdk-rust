/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::environment::parse_bool;
use crate::provider_config::ProviderConfig;
use crate::standard_property::StandardProperty;
use aws_smithy_types::error::display::DisplayErrorContext;

mod env {
    pub(super) const ENDPOINT_URL: &str = "AWS_ENDPOINT_URL";
    pub(super) const IGNORE_CONFIGURED_ENDPOINT_URLS: &str = "AWS_IGNORE_CONFIGURED_ENDPOINT_URLS";
}

mod profile_key {
    pub(super) const ENDPOINT_URL: &str = "endpoint_url";
    pub(super) const IGNORE_CONFIGURED_ENDPOINT_URLS: &str = "ignore_configured_endpoint_urls";
}

/// Load the value for "endpoint_url"
///
/// This checks the following sources:
/// 1. The environment variable `AWS_ENDPOINT_URL=http://localhost:1234'
/// 2. The profile key `endpoint_url=http://localhost:1234`
pub async fn use_endpoint_url_provider(provider_config: &ProviderConfig) -> Option<String> {
    let ignore: bool = StandardProperty::new()
        .env(env::IGNORE_CONFIGURED_ENDPOINT_URLS)
        .profile(profile_key::IGNORE_CONFIGURED_ENDPOINT_URLS)
        .validate(provider_config, parse_bool)
        .await
        .map_err(
            |err| tracing::warn!(err = %DisplayErrorContext(&err), "invalid value for ignore_configured_endpoint_urls setting"),
        )
        .unwrap_or(Some(false))
        .unwrap_or_default();
    if ignore {
        return None;
    }
    StandardProperty::new()
        .env(env::ENDPOINT_URL)
        .profile(profile_key::ENDPOINT_URL)
        .load(provider_config)
        .await
        .map(|(v, _ctx)| return v.as_ref().to_string())
}

#[cfg(test)]
mod test {
    use crate::default_provider::endpoint_url::use_endpoint_url_provider;
    use crate::profile::profile_file::{ProfileFileKind, ProfileFiles};
    use crate::provider_config::ProviderConfig;
    use aws_types::os_shim_internal::{Env, Fs};
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn defaults_to_none() {
        let conf = ProviderConfig::empty();
        assert_eq!(use_endpoint_url_provider(&conf).await, None);
    }

    #[tokio::test]
    #[traced_test]
    async fn environment_priority() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[(
                "AWS_ENDPOINT_URL",
                "http://localhost:1",
            )]))
            .with_profile_config(
                Some(
                    ProfileFiles::builder()
                        .with_file(ProfileFileKind::Config, "conf")
                        .build(),
                ),
                None,
            )
            .with_fs(Fs::from_slice(&[(
                "conf",
                "[default]\nendpoint_url = http://localhost:2",
            )]));
        assert_eq!(
            use_endpoint_url_provider(&conf).await.unwrap(),
            "http://localhost:1".to_string()
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn log_error_on_invalid_ignore() {
        let conf = ProviderConfig::empty().with_env(Env::from_slice(&[(
            "AWS_IGNORE_CONFIGURED_ENDPOINT_URLS",
            "not-a-boolean",
        )]));
        assert_eq!(use_endpoint_url_provider(&conf).await, None);
        assert!(logs_contain(
            "invalid value for ignore_configured_endpoint_urls setting"
        ));
        assert!(logs_contain("AWS_IGNORE_CONFIGURED_ENDPOINT_URLS"));
    }

    #[tokio::test]
    #[traced_test]
    async fn ignore_if_specified_explicitly() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[
                ("AWS_ENDPOINT_URL", "http://localhost:1"),
                ("AWS_IGNORE_CONFIGURED_ENDPOINT_URLS", "true"),
            ]))
            .with_profile_config(
                Some(
                    ProfileFiles::builder()
                        .with_file(ProfileFileKind::Config, "conf")
                        .build(),
                ),
                None,
            )
            .with_fs(Fs::from_slice(&[(
                "conf",
                "[default]\nendpoint_url = http://localhost:2",
            )]));
        assert_eq!(use_endpoint_url_provider(&conf).await, None);
    }
}
