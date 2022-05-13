/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![deny(missing_docs)]

//! AWS Shared Config
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

use std::sync::Arc;

use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout;

use crate::app_name::AppName;
use crate::credentials::SharedCredentialsProvider;
use crate::endpoint::ResolveAwsEndpoint;
use crate::region::Region;

/// AWS Shared Configuration
#[derive(Debug, Clone)]
pub struct SdkConfig {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<timeout::Config>,
    http_connector: Option<HttpConnector>,
}

/// Builder for AWS Shared Configuration
#[derive(Debug, Default)]
pub struct Builder {
    app_name: Option<AppName>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
    timeout_config: Option<timeout::Config>,
    http_connector: Option<HttpConnector>,
}

impl Builder {
    /// Set the region for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::SdkConfig;
    /// use aws_types::region::Region;
    /// let config = SdkConfig::builder().region(Region::new("us-east-1")).build();
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
    /// use aws_types::SdkConfig;
    /// use aws_types::region::Region;
    /// let mut builder = SdkConfig::builder();
    /// if let Some(region) = region_override() {
    ///     builder.set_region(region);
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_region(&mut self, region: impl Into<Option<Region>>) -> &mut Self {
        self.region = region.into();
        self
    }

    /// Set the endpoint resolver to use when making requests
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    /// use aws_types::SdkConfig;
    /// use aws_smithy_http::endpoint::Endpoint;
    /// use http::Uri;
    /// let config = SdkConfig::builder().endpoint_resolver(
    ///     Endpoint::immutable(Uri::from_static("http://localhost:8080"))
    /// ).build();
    /// ```
    pub fn endpoint_resolver(
        mut self,
        endpoint_resolver: impl ResolveAwsEndpoint + 'static,
    ) -> Self {
        self.set_endpoint_resolver(Some(Arc::new(endpoint_resolver)));
        self
    }

    /// Set the endpoint resolver to use when making requests
    ///
    /// # Examples
    /// ```
    /// use std::sync::Arc;
    /// use aws_types::SdkConfig;
    /// use aws_types::endpoint::ResolveAwsEndpoint;
    /// fn endpoint_resolver_override() -> Option<Arc<dyn ResolveAwsEndpoint>> {
    ///     // ...
    ///     # None
    /// }
    /// let mut config = SdkConfig::builder();
    /// config.set_endpoint_resolver(endpoint_resolver_override());
    /// config.build();
    /// ```
    pub fn set_endpoint_resolver(
        &mut self,
        endpoint_resolver: Option<Arc<dyn ResolveAwsEndpoint>>,
    ) -> &mut Self {
        self.endpoint_resolver = endpoint_resolver;
        self
    }

    /// Set the retry_config for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// let retry_config = RetryConfig::new().with_max_attempts(5);
    /// let config = SdkConfig::builder().retry_config(retry_config).build();
    /// ```
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.set_retry_config(Some(retry_config));
        self
    }

    /// Set the retry_config for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// fn disable_retries(builder: &mut Builder) {
    ///     let retry_config = RetryConfig::new().with_max_attempts(1);
    ///     builder.set_retry_config(Some(retry_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// disable_retries(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_retry_config(&mut self, retry_config: Option<RetryConfig>) -> &mut Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the [`timeout::Config`](aws_smithy_types::timeout::Config) for the builder
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::{timeout, tristate::TriState};
    ///
    /// let api_timeout_config = timeout::Api::new()
    ///     .with_call_attempt_timeout(TriState::Set(Duration::from_secs(2)))
    ///     .with_call_timeout(TriState::Set(Duration::from_secs(5)));
    /// let timeout_config = timeout::Config::new()
    ///     .with_api_timeouts(api_timeout_config);
    /// let config = SdkConfig::builder().timeout_config(timeout_config).build();
    /// ```
    pub fn timeout_config(mut self, timeout_config: timeout::Config) -> Self {
        self.set_timeout_config(Some(timeout_config));
        self
    }

    /// Set the [`timeout::Config`](aws_smithy_types::timeout::Config) for the builder
    ///
    /// # Examples
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::{timeout, tristate::TriState};
    ///
    /// fn set_preferred_timeouts(builder: &mut Builder) {
    ///     let api_timeout_config = timeout::Api::new()
    ///         .with_call_attempt_timeout(TriState::Set(Duration::from_secs(2)))
    ///         .with_call_timeout(TriState::Set(Duration::from_secs(5)));
    ///     let timeout_config = timeout::Config::new()
    ///         .with_api_timeouts(api_timeout_config);
    ///     builder.set_timeout_config(Some(timeout_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// set_preferred_timeouts(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_timeout_config(&mut self, timeout_config: Option<timeout::Config>) -> &mut Self {
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
    /// use aws_types::SdkConfig;
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
    /// let config = SdkConfig::builder().sleep_impl(sleep_impl).build();
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
    /// # use aws_types::sdk_config::{Builder, SdkConfig};
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
    /// let mut builder = SdkConfig::builder();
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
    /// use aws_types::SdkConfig;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_types::Credentials;
    ///   # Credentials::new("test", "test", None, None, "example")
    /// }
    ///
    /// let config = SdkConfig::builder()
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
    /// use aws_types::SdkConfig;
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
    /// let mut builder = SdkConfig::builder();
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

    /// Build a [`SdkConfig`](SdkConfig) from this builder
    pub fn build(self) -> SdkConfig {
        SdkConfig {
            app_name: self.app_name,
            credentials_provider: self.credentials_provider,
            region: self.region,
            endpoint_resolver: self.endpoint_resolver,
            retry_config: self.retry_config,
            sleep_impl: self.sleep_impl,
            timeout_config: self.timeout_config,
            http_connector: self.http_connector,
        }
    }
}

impl SdkConfig {
    /// Configured region
    pub fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Configured endpoint resolver
    pub fn endpoint_resolver(&self) -> Option<Arc<dyn ResolveAwsEndpoint>> {
        self.endpoint_resolver.clone()
    }

    /// Configured retry config
    pub fn retry_config(&self) -> Option<&RetryConfig> {
        self.retry_config.as_ref()
    }

    /// Configured timeout config
    pub fn timeout_config(&self) -> Option<&timeout::Config> {
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
