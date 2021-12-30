/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Load a region from an AWS profile

use crate::meta::region::{future, ProvideRegion};
use crate::provider_config::ProviderConfig;
use aws_types::os_shim_internal::{Env, Fs};
use aws_types::region::Region;

/// Load a region from a profile file
///
/// This provider will attempt to load AWS shared configuration, then read the `region` property
/// from the active profile.
///
/// # Examples
///
/// **Loads "us-west-2" as the region**
/// ```ini
/// [default]
/// region = us-west-2
/// ```
///
/// **Loads `us-east-1` as the region _if and only if_ the `AWS_PROFILE` environment variable is set
/// to `other`.**
///
/// ```ini
/// [profile other]
/// region = us-east-1
/// ```
///
/// This provider is part of the [default region provider chain](crate::default_provider::region).
#[derive(Debug, Default)]
pub struct ProfileFileRegionProvider {
    fs: Fs,
    env: Env,
    profile_override: Option<String>,
}

/// Builder for [ProfileFileRegionProvider]
#[derive(Default)]
pub struct Builder {
    config: Option<ProviderConfig>,
    profile_override: Option<String>,
}

impl Builder {
    /// Override the configuration for this provider
    pub fn configure(mut self, config: &ProviderConfig) -> Self {
        self.config = Some(config.clone());
        self
    }

    /// Override the profile name used by the [ProfileFileRegionProvider]
    pub fn profile_name(mut self, profile_name: impl Into<String>) -> Self {
        self.profile_override = Some(profile_name.into());
        self
    }

    /// Build a [ProfileFileRegionProvider] from this builder
    pub fn build(self) -> ProfileFileRegionProvider {
        let conf = self.config.unwrap_or_default();
        ProfileFileRegionProvider {
            env: conf.env(),
            fs: conf.fs(),
            profile_override: self.profile_override,
        }
    }
}

impl ProfileFileRegionProvider {
    /// Create a new [ProfileFileRegionProvider]
    ///
    /// To override the selected profile, set the `AWS_PROFILE` environment variable or use the [`Builder`].
    pub fn new() -> Self {
        Self {
            fs: Fs::real(),
            env: Env::real(),
            profile_override: None,
        }
    }

    /// [`Builder`] to construct a [`ProfileFileRegionProvider`]
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn region(&self) -> Option<Region> {
        let profile = super::parser::load(&self.fs, &self.env)
            .await
            .map_err(|err| tracing::warn!(err = %err, "failed to parse profile"))
            .ok()?;
        let selected_profile = self
            .profile_override
            .as_deref()
            .unwrap_or_else(|| profile.selected_profile());
        let selected_profile = profile.get_profile(selected_profile)?;
        selected_profile
            .get("region")
            .map(|region| Region::new(region.to_owned()))
    }
}

impl ProvideRegion for ProfileFileRegionProvider {
    fn region(&self) -> future::ProvideRegion {
        future::ProvideRegion::new(self.region())
    }
}

#[cfg(test)]
mod test {
    use crate::profile::ProfileFileRegionProvider;
    use crate::provider_config::ProviderConfig;
    use crate::test_case::no_traffic_connector;
    use aws_sdk_sts::Region;
    use aws_types::os_shim_internal::{Env, Fs};
    use futures_util::FutureExt;
    use tracing_test::traced_test;

    fn provider_config(dir_name: &str) -> ProviderConfig {
        let fs = Fs::from_test_dir(format!("test-data/profile-provider/{}/fs", dir_name), "/");
        let env = Env::from_slice(&[("HOME", "/home")]);
        ProviderConfig::empty()
            .with_fs(fs)
            .with_env(env)
            .with_http_connector(no_traffic_connector())
    }

    #[traced_test]
    #[test]
    fn load_region() {
        let provider = ProfileFileRegionProvider::builder()
            .configure(&provider_config("region_override"))
            .build();
        assert_eq!(
            provider.region().now_or_never().unwrap(),
            Some(Region::from_static("us-east-1"))
        );
    }

    #[test]
    fn load_region_env_profile_override() {
        let conf = provider_config("region_override").with_env(Env::from_slice(&[
            ("HOME", "/home"),
            ("AWS_PROFILE", "base"),
        ]));
        let provider = ProfileFileRegionProvider::builder()
            .configure(&conf)
            .build();
        assert_eq!(
            provider.region().now_or_never().unwrap(),
            Some(Region::from_static("us-east-1"))
        );
    }

    #[test]
    fn load_region_nonexistent_profile() {
        let conf = provider_config("region_override").with_env(Env::from_slice(&[
            ("HOME", "/home"),
            ("AWS_PROFILE", "doesnotexist"),
        ]));
        let provider = ProfileFileRegionProvider::builder()
            .configure(&conf)
            .build();
        assert_eq!(provider.region().now_or_never().unwrap(), None);
    }

    #[test]
    fn load_region_explicit_override() {
        let conf = provider_config("region_override");
        let provider = ProfileFileRegionProvider::builder()
            .configure(&conf)
            .profile_name("base")
            .build();
        assert_eq!(
            provider.region().now_or_never().unwrap(),
            Some(Region::from_static("us-east-1"))
        );
    }
}
