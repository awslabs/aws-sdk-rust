/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::environment::parse_bool;
use crate::provider_config::ProviderConfig;
use crate::standard_property::StandardProperty;
use aws_smithy_types::error::display::DisplayErrorContext;

mod env {
    pub(super) const USE_DUAL_STACK: &str = "AWS_USE_DUALSTACK_ENDPOINT";
}

mod profile_key {
    pub(super) const USE_DUAL_STACK: &str = "use_dualstack_endpoint";
}

pub(crate) async fn use_dual_stack_provider(provider_config: &ProviderConfig) -> Option<bool> {
    StandardProperty::new()
        .env(env::USE_DUAL_STACK)
        .profile(profile_key::USE_DUAL_STACK)
        .validate(provider_config, parse_bool)
        .await
        .map_err(
            |err| tracing::warn!(err = %DisplayErrorContext(&err), "invalid value for dual-stack setting"),
        )
        .unwrap_or(None)
}

#[cfg(test)]
mod test {
    use crate::default_provider::use_dual_stack::use_dual_stack_provider;
    use crate::profile::profile_file::{ProfileFileKind, ProfileFiles};
    use crate::provider_config::ProviderConfig;
    use aws_types::os_shim_internal::{Env, Fs};
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn log_error_on_invalid_value() {
        let conf = ProviderConfig::empty().with_env(Env::from_slice(&[(
            "AWS_USE_DUALSTACK_ENDPOINT",
            "not-a-boolean",
        )]));
        assert_eq!(use_dual_stack_provider(&conf).await, None);
        assert!(logs_contain("invalid value for dual-stack setting"));
        assert!(logs_contain("AWS_USE_DUALSTACK_ENDPOINT"));
    }

    #[tokio::test]
    #[traced_test]
    async fn environment_priority() {
        let conf = ProviderConfig::empty()
            .with_env(Env::from_slice(&[("AWS_USE_DUALSTACK_ENDPOINT", "TRUE")]))
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
                "[default]\nuse_dualstack_endpoint = false",
            )]));
        assert_eq!(use_dual_stack_provider(&conf).await, Some(true));
    }

    #[tokio::test]
    #[traced_test]
    async fn profile_works() {
        let conf = ProviderConfig::empty()
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
                "[default]\nuse_dualstack_endpoint = false",
            )]));
        assert_eq!(use_dual_stack_provider(&conf).await, Some(false));
    }
}
