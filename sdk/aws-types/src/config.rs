/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![deny(missing_docs)]

//! AWS Shared Config
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

use crate::credentials::SharedCredentialsProvider;
use crate::region::Region;

/// AWS Shared Configuration
pub struct Config {
    region: Option<Region>,
    credentials_provider: Option<SharedCredentialsProvider>,
}

/// Builder for AWS Shared Configuration
#[derive(Default)]
pub struct Builder {
    region: Option<Region>,
    credentials_provider: Option<SharedCredentialsProvider>,
}

impl Builder {
    /// Set the region for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::config::Config;
    /// use aws_types::region::Region;
    /// let config = Config::builder().region(Region::new("us-east-1")).build();
    /// ```
    pub fn region(mut self, region: impl Into<Option<Region>>) -> Self {
        self.set_region(region);
        self
    }

    /// Set the region for the builder
    ///
    /// # Examples
    /// ```rust
    /// fn region_override() -> Option<Region> {
    ///     // ...
    ///     # None
    /// }
    /// use aws_types::config::Config;
    /// use aws_types::region::Region;
    /// let mut builder = Config::builder();
    /// if let Some(region) = region_override() {
    ///     builder.set_region(region);
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_region(&mut self, region: impl Into<Option<Region>>) -> &mut Self {
        self.region = region.into();
        self
    }

    /// Set the credentials provider for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::credentials::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::config::Config;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_types::Credentials;
    ///   # Credentials::from_keys("test", "test", None)
    /// }
    ///
    /// let config = Config::builder()
    ///     .credentials_provider(SharedCredentialsProvider::new(make_provider()))
    ///     .build();
    /// ```
    pub fn credentials_provider(mut self, provider: SharedCredentialsProvider) -> Self {
        self.set_credentials_provider(Some(provider));
        self
    }

    /// Set the credentials provider for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::credentials::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::config::Config;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_types::Credentials;
    ///   # Credentials::from_keys("test", "test", None)
    /// }
    ///
    /// fn override_provider() -> bool {
    ///   // ...
    ///   # true
    /// }
    ///
    /// let mut builder = Config::builder();
    /// if override_provider() {
    ///     builder.set_credentials_provider(Some(SharedCredentialsProvider::new(make_provider())));
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_credentials_provider(
        &mut self,
        provider: Option<SharedCredentialsProvider>,
    ) -> &mut Self {
        self.credentials_provider = provider;
        self
    }

    /// Build a [`Config`](Config) from this builder
    pub fn build(self) -> Config {
        Config {
            region: self.region,
            credentials_provider: self.credentials_provider,
        }
    }
}

impl Config {
    /// Configured region
    pub fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Configured credentials provider
    pub fn credentials_provider(&self) -> Option<&SharedCredentialsProvider> {
        self.credentials_provider.as_ref()
    }

    /// Config builder
    pub fn builder() -> Builder {
        Builder::default()
    }
}
