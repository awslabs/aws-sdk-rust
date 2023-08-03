/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![deny(missing_docs)]

//! AWS Shared Config
//!
//! This module contains an shared configuration representation that is agnostic from a specific service.

use aws_credential_types::cache::CredentialsCache;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::{SharedTimeSource, TimeSource};
use aws_smithy_client::http_connector::HttpConnector;
use aws_smithy_types::retry::RetryConfig;
use aws_smithy_types::timeout::TimeoutConfig;

use crate::app_name::AppName;
use crate::docs_for;
use crate::region::Region;

#[doc(hidden)]
/// Unified docstrings to keep crates in sync. Not intended for public use
pub mod unified_docs {
    #[macro_export]
    macro_rules! docs_for {
        (use_fips) => {
"When true, send this request to the FIPS-compliant regional endpoint.

If no FIPS-compliant endpoint can be determined, dispatching the request will return an error."
        };
        (use_dual_stack) => {
"When true, send this request to the dual-stack endpoint.

If no dual-stack endpoint is available the request MAY return an error.

**Note**: Some services do not offer dual-stack as a configurable parameter (e.g. Code Catalyst). For
these services, this setting has no effect"
        };

        (time_source) => { "The time source use to use for this client. This only needs to be required for creating deterministic tests or platforms where `SystemTime::now()` is not supported." };
    }
}

/// AWS Shared Configuration
#[derive(Debug, Clone)]
pub struct SdkConfig {
    app_name: Option<AppName>,
    credentials_cache: Option<CredentialsCache>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_url: Option<String>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<SharedAsyncSleep>,
    time_source: Option<SharedTimeSource>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
    use_fips: Option<bool>,
    use_dual_stack: Option<bool>,
}

/// Builder for AWS Shared Configuration
///
/// _Important:_ Using the `aws-config` crate to configure the SDK is preferred to invoking this
/// builder directly. Using this builder directly won't pull in any AWS recommended default
/// configuration values.
#[derive(Debug, Default)]
pub struct Builder {
    app_name: Option<AppName>,
    credentials_cache: Option<CredentialsCache>,
    credentials_provider: Option<SharedCredentialsProvider>,
    region: Option<Region>,
    endpoint_url: Option<String>,
    retry_config: Option<RetryConfig>,
    sleep_impl: Option<SharedAsyncSleep>,
    time_source: Option<SharedTimeSource>,
    timeout_config: Option<TimeoutConfig>,
    http_connector: Option<HttpConnector>,
    use_fips: Option<bool>,
    use_dual_stack: Option<bool>,
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

    /// Set the endpoint URL to use when making requests.
    /// # Examples
    /// ```
    /// use aws_types::SdkConfig;
    /// let config = SdkConfig::builder().endpoint_url("http://localhost:8080").build();
    /// ```
    pub fn endpoint_url(mut self, endpoint_url: impl Into<String>) -> Self {
        self.set_endpoint_url(Some(endpoint_url.into()));
        self
    }

    /// Set the endpoint URL to use when making requests.
    pub fn set_endpoint_url(&mut self, endpoint_url: Option<String>) -> &mut Self {
        self.endpoint_url = endpoint_url;
        self
    }

    /// Set the retry_config for the builder
    ///
    /// _Note:_ Retries require a sleep implementation in order to work. When enabling retry, make
    /// sure to set one with [Self::sleep_impl] or [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// let retry_config = RetryConfig::standard().with_max_attempts(5);
    /// let config = SdkConfig::builder().retry_config(retry_config).build();
    /// ```
    pub fn retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.set_retry_config(Some(retry_config));
        self
    }

    /// Set the retry_config for the builder
    ///
    /// _Note:_ Retries require a sleep implementation in order to work. When enabling retry, make
    /// sure to set one with [Self::sleep_impl] or [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::retry::RetryConfig;
    ///
    /// fn disable_retries(builder: &mut Builder) {
    ///     let retry_config = RetryConfig::standard().with_max_attempts(1);
    ///     builder.set_retry_config(Some(retry_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// disable_retries(&mut builder);
    /// ```
    pub fn set_retry_config(&mut self, retry_config: Option<RetryConfig>) -> &mut Self {
        self.retry_config = retry_config;
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// _Note:_ Timeouts require a sleep implementation in order to work.
    /// When enabling timeouts, be sure to set one with [Self::sleep_impl] or
    /// [Self::set_sleep_impl].
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::SdkConfig;
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// let timeout_config = TimeoutConfig::builder()
    ///     .operation_attempt_timeout(Duration::from_secs(2))
    ///     .operation_timeout(Duration::from_secs(5))
    ///     .build();
    /// let config = SdkConfig::builder()
    ///     .timeout_config(timeout_config)
    ///     .build();
    /// ```
    pub fn timeout_config(mut self, timeout_config: TimeoutConfig) -> Self {
        self.set_timeout_config(Some(timeout_config));
        self
    }

    /// Set the [`TimeoutConfig`] for the builder
    ///
    /// _Note:_ Timeouts require a sleep implementation in order to work.
    /// When enabling timeouts, be sure to set one with [Self::sleep_impl] or
    /// [Self::set_sleep_impl].
    ///
    /// # Examples
    /// ```rust
    /// # use std::time::Duration;
    /// use aws_types::sdk_config::{SdkConfig, Builder};
    /// use aws_smithy_types::timeout::TimeoutConfig;
    ///
    /// fn set_preferred_timeouts(builder: &mut Builder) {
    ///     let timeout_config = TimeoutConfig::builder()
    ///         .operation_attempt_timeout(Duration::from_secs(2))
    ///         .operation_timeout(Duration::from_secs(5))
    ///         .build();
    ///     builder.set_timeout_config(Some(timeout_config));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// set_preferred_timeouts(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_timeout_config(&mut self, timeout_config: Option<TimeoutConfig>) -> &mut Self {
        self.timeout_config = timeout_config;
        self
    }

    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// _Note:_ If you're using the Tokio runtime, a `TokioSleep` implementation is available in
    /// the `aws-smithy-async` crate.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep, Sleep};
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
    /// let sleep_impl = SharedAsyncSleep::new(ForeverSleep);
    /// let config = SdkConfig::builder().sleep_impl(sleep_impl).build();
    /// ```
    pub fn sleep_impl(mut self, sleep_impl: SharedAsyncSleep) -> Self {
        self.set_sleep_impl(Some(sleep_impl));
        self
    }

    /// Set the sleep implementation for the builder. The sleep implementation is used to create
    /// timeout futures.
    ///
    /// _Note:_ If you're using the Tokio runtime, a `TokioSleep` implementation is available in
    /// the `aws-smithy-async` crate.
    ///
    /// # Examples
    /// ```rust
    /// # use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep, Sleep};
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
    ///     let sleep_impl = SharedAsyncSleep::new(ForeverSleep);
    ///     builder.set_sleep_impl(Some(sleep_impl));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// set_never_ending_sleep_impl(&mut builder);
    /// let config = builder.build();
    /// ```
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<SharedAsyncSleep>) -> &mut Self {
        self.sleep_impl = sleep_impl;
        self
    }

    /// Set the [`CredentialsCache`] for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_credential_types::cache::CredentialsCache;
    /// use aws_types::SdkConfig;
    /// let config = SdkConfig::builder()
    ///     .credentials_cache(CredentialsCache::lazy())
    ///     .build();
    /// ```
    pub fn credentials_cache(mut self, cache: CredentialsCache) -> Self {
        self.set_credentials_cache(Some(cache));
        self
    }

    /// Set the [`CredentialsCache`] for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_credential_types::cache::CredentialsCache;
    /// use aws_types::SdkConfig;
    /// fn override_credentials_cache() -> bool {
    ///   // ...
    ///   # true
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// if override_credentials_cache() {
    ///     builder.set_credentials_cache(Some(CredentialsCache::lazy()));
    /// }
    /// let config = builder.build();
    /// ```
    pub fn set_credentials_cache(&mut self, cache: Option<CredentialsCache>) -> &mut Self {
        self.credentials_cache = cache;
        self
    }

    /// Set the credentials provider for the builder
    ///
    /// # Examples
    /// ```rust
    /// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::SdkConfig;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_credential_types::Credentials;
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
    /// use aws_credential_types::provider::{ProvideCredentials, SharedCredentialsProvider};
    /// use aws_types::SdkConfig;
    /// fn make_provider() -> impl ProvideCredentials {
    ///   // ...
    ///   # use aws_credential_types::Credentials;
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

    /// Sets the HTTP connector to use when making requests.
    ///
    /// ## Examples
    /// ```no_run
    /// # #[cfg(feature = "examples")]
    /// # fn example() {
    /// use std::time::Duration;
    /// use aws_smithy_client::{Client, hyper_ext};
    /// use aws_smithy_client::erase::DynConnector;
    /// use aws_smithy_client::http_connector::ConnectorSettings;
    /// use aws_types::SdkConfig;
    ///
    /// let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
    ///     .with_webpki_roots()
    ///     .https_only()
    ///     .enable_http1()
    ///     .enable_http2()
    ///     .build();
    /// let smithy_connector = hyper_ext::Adapter::builder()
    ///     // Optionally set things like timeouts as well
    ///     .connector_settings(
    ///         ConnectorSettings::builder()
    ///             .connect_timeout(Duration::from_secs(5))
    ///             .build()
    ///     )
    ///     .build(https_connector);
    /// let sdk_config = SdkConfig::builder()
    ///     .http_connector(smithy_connector)
    ///     .build();
    /// # }
    /// ```
    pub fn http_connector(mut self, http_connector: impl Into<HttpConnector>) -> Self {
        self.set_http_connector(Some(http_connector));
        self
    }

    /// Sets the HTTP connector to use when making requests.
    ///
    /// ## Examples
    /// ```no_run
    /// # #[cfg(feature = "examples")]
    /// # fn example() {
    /// use std::time::Duration;
    /// use aws_smithy_client::hyper_ext;
    /// use aws_smithy_client::http_connector::ConnectorSettings;
    /// use aws_types::sdk_config::{Builder, SdkConfig};
    ///
    /// fn override_http_connector(builder: &mut Builder) {
    ///     let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
    ///         .with_webpki_roots()
    ///         .https_only()
    ///         .enable_http1()
    ///         .enable_http2()
    ///         .build();
    ///     let smithy_connector = hyper_ext::Adapter::builder()
    ///         // Optionally set things like timeouts as well
    ///         .connector_settings(
    ///             ConnectorSettings::builder()
    ///                 .connect_timeout(Duration::from_secs(5))
    ///                 .build()
    ///         )
    ///         .build(https_connector);
    ///     builder.set_http_connector(Some(smithy_connector));
    /// }
    ///
    /// let mut builder = SdkConfig::builder();
    /// override_http_connector(&mut builder);
    /// let config = builder.build();
    /// # }
    /// ```
    pub fn set_http_connector(
        &mut self,
        http_connector: Option<impl Into<HttpConnector>>,
    ) -> &mut Self {
        self.http_connector = http_connector.map(|inner| inner.into());
        self
    }

    #[doc = docs_for!(use_fips)]
    pub fn use_fips(mut self, use_fips: bool) -> Self {
        self.set_use_fips(Some(use_fips));
        self
    }

    #[doc = docs_for!(use_fips)]
    pub fn set_use_fips(&mut self, use_fips: Option<bool>) -> &mut Self {
        self.use_fips = use_fips;
        self
    }

    #[doc = docs_for!(use_dual_stack)]
    pub fn use_dual_stack(mut self, use_dual_stack: bool) -> Self {
        self.set_use_dual_stack(Some(use_dual_stack));
        self
    }

    #[doc = docs_for!(use_dual_stack)]
    pub fn set_use_dual_stack(&mut self, use_dual_stack: Option<bool>) -> &mut Self {
        self.use_dual_stack = use_dual_stack;
        self
    }

    #[doc = docs_for!(time_source)]
    pub fn time_source(mut self, time_source: impl TimeSource + 'static) -> Self {
        self.set_time_source(Some(SharedTimeSource::new(time_source)));
        self
    }

    #[doc = docs_for!(time_source)]
    pub fn set_time_source(&mut self, time_source: Option<SharedTimeSource>) -> &mut Self {
        self.time_source = time_source;
        self
    }

    /// Build a [`SdkConfig`](SdkConfig) from this builder
    pub fn build(self) -> SdkConfig {
        SdkConfig {
            app_name: self.app_name,
            credentials_cache: self.credentials_cache,
            credentials_provider: self.credentials_provider,
            region: self.region,
            endpoint_url: self.endpoint_url,
            retry_config: self.retry_config,
            sleep_impl: self.sleep_impl,
            timeout_config: self.timeout_config,
            http_connector: self.http_connector,
            use_fips: self.use_fips,
            use_dual_stack: self.use_dual_stack,
            time_source: self.time_source,
        }
    }
}

impl SdkConfig {
    /// Configured region
    pub fn region(&self) -> Option<&Region> {
        self.region.as_ref()
    }

    /// Configured endpoint URL
    pub fn endpoint_url(&self) -> Option<&str> {
        self.endpoint_url.as_deref()
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
    pub fn sleep_impl(&self) -> Option<SharedAsyncSleep> {
        self.sleep_impl.clone()
    }

    /// Configured credentials cache
    pub fn credentials_cache(&self) -> Option<&CredentialsCache> {
        self.credentials_cache.as_ref()
    }

    /// Configured credentials provider
    pub fn credentials_provider(&self) -> Option<SharedCredentialsProvider> {
        self.credentials_provider.clone()
    }

    /// Configured time source
    pub fn time_source(&self) -> Option<SharedTimeSource> {
        self.time_source.clone()
    }

    /// Configured app name
    pub fn app_name(&self) -> Option<&AppName> {
        self.app_name.as_ref()
    }

    /// Configured HTTP Connector
    pub fn http_connector(&self) -> Option<&HttpConnector> {
        self.http_connector.as_ref()
    }

    /// Use FIPS endpoints
    pub fn use_fips(&self) -> Option<bool> {
        self.use_fips
    }

    /// Use dual-stack endpoint
    pub fn use_dual_stack(&self) -> Option<bool> {
        self.use_dual_stack
    }

    /// Config builder
    ///
    /// _Important:_ Using the `aws-config` crate to configure the SDK is preferred to invoking this
    /// builder directly. Using this builder directly won't pull in any AWS recommended default
    /// configuration values.
    pub fn builder() -> Builder {
        Builder::default()
    }
}
