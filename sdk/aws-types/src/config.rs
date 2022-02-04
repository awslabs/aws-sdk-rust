/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

#![deny(missing_docs)]

//! AWS Shared Config
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

use std::sync::Arc;

use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;

use crate::app_name::AppName;
use crate::credentials::SharedCredentialsProvider;
use crate::region::Region;

/// AWS Shared Configuration
#[derive(Debug, Clone)]
pub struct Config {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
}

/// Builder for AWS Shared Configuration
#[derive(Debug, Default)]
pub struct Builder {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
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

    /// Set the retry_config for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::config::Config;
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// let retry_config = RetryConfig::new().with_max_attempts(5);
    /// let config = Config::builder().retry_config(retry_config).build();
    /// ```
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.set_retry_config(Some(retry_config));
        self
    }

    /// Set the retry_config for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::config::{Config, Builder};
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// fn disable_retries(builder: &mut Builder) {
    ///     let retry_config = RetryConfig::new().with_max_attempts(1);
    ///     builder.set_retry_config(Some(retry_config));
    /// }
    ///
    /// let mut builder = Config::builder();
    /// disable_retries(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_retry_config(&mut self, retry_config: Option<RetryConfig>) -> &mut Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::config::Config;
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// let timeout_config = TimeoutConfig::new()
    ///     .with_api_call_attempt_timeout(Some(Duration::from_secs(1)));
    /// let config = Config::builder().timeout_config(timeout_config).build();
    /// ```
    pub fn timeout_config(mut self, timeout_config: TimeoutConfig) -> Self {
        self.set_timeout_config(Some(timeout_config));
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// # Examples
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::config::{Config, Builder};
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// fn set_preferred_timeouts(builder: &mut Builder) {
    ///     let timeout_config = TimeoutConfig::new()
    ///         .with_api_call_attempt_timeout(Some(Duration::from_secs(2)))
    ///         .with_api_call_timeout(Some(Duration::from_secs(5)));
    ///     builder.set_timeout_config(Some(timeout_config));
    /// }
    ///
    /// let mut builder = Config::builder();
    /// set_preferred_timeouts(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_timeout_config(&mut self, timeout_config: Option<TimeoutConfig>) -> &mut Self {
        self.timeout_config = timeout_config;
        self
    }

    #[doc(hidden)]
    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
    /// use aws_types::config::Config;
    ///
    /// ##[derive(Debug)]
    /// pub struct ForeverSleep;
    ///
    /// impl AsyncSleep for ForeverSleep {
    ///     fn sleep(&self, duration: std::time::Duration) -> Sleep {
    ///         Sleep::new(std::future::pending())
    ///     }
    /// }
    ///
    /// let sleep_impl = Arc::new(ForeverSleep);
    /// let config = Config::builder().sleep_impl(sleep_impl).build();
    /// ```
    pub fn sleep_impl(mut self, sleep_impl: Arc<dyn AsyncSleep>) -> Self {
        self.set_sleep_impl(Some(sleep_impl));
        self
    }

    #[doc(hidden)]
    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// # Examples
    /// ```rust
    /// # use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
    /// # use aws_types::config::{Builder, Config};
    /// #[derive(Debug)]
    /// pub struct ForeverSleep;
    ///
    /// impl AsyncSleep for ForeverSleep {
    ///     fn sleep(&self, duration: std::time::Duration) -> Sleep {
    ///         Sleep::new(std::future::pending())
    ///     }
    /// }
    ///
    /// fn set_never_ending_sleep_impl(builder: &mut Builder) {
    ///     let sleep_impl = std::sync::Arc::new(ForeverSleep);
    ///     builder.set_sleep_impl(Some(sleep_impl));
    /// }
    ///
    /// let mut builder = Config::builder();
    /// set_never_ending_sleep_impl(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<Arc<dyn AsyncSleep>>) -> &mut Self {
        self.sleep_impl = sleep_impl;
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
    ///   # Credentials::new("test", "test", None, None, "example")
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
    ///   # Credentials::new("test", "test", None, None, "example")
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

    /// Sets the name of the app that is using the client.
    ///
    /// This _optional_ name is used to identify the application in the user agent that
    /// gets sent along with requests.
    pub fn app_name(mut self, app_name: AppName) -> Self {
        self.set_app_name(Some(app_name));
        self
    }

    /// Sets the name of the app that is using the client.
    ///
    /// This _optional_ name is used to identify the application in the user agent that
    /// gets sent along with requests.
    pub fn set_app_name(&mut self, app_name: Option<AppName>) -> &mut Self {
        self.app_name = app_name;
        self
    }

    /// Sets the HTTP connector that clients will use to make HTTP requests.
    pub fn http_connector(mut self, http_connector: HttpConnector) -> Self {
        self.set_http_connector(Some(http_connector));
        self
    }

    /// Sets the HTTP connector that clients will use to make HTTP requests.
    pub fn set_http_connector(&mut self, http_connector: Option<HttpConnector>) -> &mut Self {
        self.http_connector = http_connector;
        self
    }

    /// Build a [`Config`](Config) from this builder
    pub fn build(self) -> Config {
        Config {
            app_name: self.app_name,
            credentials_provider: self.credentials_provider,
            region: self.region,
            retry_config: self.retry_config,
            sleep_impl: self.sleep_impl,
            timeout_config: self.timeout_config,
            http_connector: self.http_connector,
        }
    }
}

impl Config {
    /// Configured region
    pub fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Configured retry config
    pub fn retry_config(&self) -> Option<&RetryConfig> {
        self.retry_config.as_ref()
    }

    /// Configured timeout config
    pub fn timeout_config(&self) -> Option<&TimeoutConfig> {
        self.timeout_config.as_ref()
    }

    #[doc(hidden)]
    /// Configured sleep implementation
    pub fn sleep_impl(&self) -> Option<Arc<dyn AsyncSleep>> {
        self.sleep_impl.clone()
    }

    /// Configured credentials provider
    pub fn credentials_provider(&self) -> Option<&SharedCredentialsProvider> {
        self.credentials_provider.as_ref()
    }

    /// Configured app name
    pub fn app_name(&self) -> Option<&AppName> {
        self.app_name.as_ref()
    }

    /// Configured HTTP Connector
    pub fn http_connector(&self) -> Option<&HttpConnector> {
        self.http_connector.as_ref()
    }

    /// Config builder
    pub fn builder() -> Builder {
        Builder::default()
    }
}
