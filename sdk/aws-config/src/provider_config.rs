/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Configuration Options for Credential Providers

use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
use aws_smithy_client::erase::DynConnector;
use aws_types::os_shim_internal::{Env, Fs, TimeSource};
use aws_types::{
    http_connector::{HttpConnector, HttpSettings},
    region::Region,
};

use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use crate::connector::default_connector;
use http::Uri;
use hyper::client::connect::Connection;
use tokio::io::{AsyncRead, AsyncWrite};

/// Configuration options for Credential Providers
///
/// Most credential providers builders offer a `configure` method which applies general provider configuration
/// options.
///
/// To use a region from the default region provider chain use [`ProviderConfig::with_default_region`].
/// Otherwise, use [`ProviderConfig::without_region`]. Note that some credentials providers require a region
/// to be explicitly set.
#[derive(Clone)]
pub struct ProviderConfig {
    env: Env,
    fs: Fs,
    time_source: TimeSource,
    connector: HttpConnector,
    sleep: Option<Arc<dyn AsyncSleep>>,
    region: Option<Region>,
}

impl Debug for ProviderConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderConfig")
            .field("env", &self.env)
            .field("fs", &self.fs)
            .field("sleep", &self.sleep)
            .field("region", &self.region)
            .finish()
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        let connector = HttpConnector::ConnectorFn(Arc::new(
            |settings: &HttpSettings, sleep: Option<Arc<dyn AsyncSleep>>| {
                default_connector(settings, sleep)
            },
        ));

        Self {
            env: Env::default(),
            fs: Fs::default(),
            time_source: TimeSource::default(),
            connector,
            sleep: default_async_sleep(),
            region: None,
        }
    }
}

#[cfg(test)]
impl ProviderConfig {
    /// ProviderConfig with all configuration removed
    ///
    /// Unlike [`ProviderConfig::empty`] where `env` and `fs` will use their non-mocked implementations,
    /// this method will use an empty mock environment and an empty mock file system.
    pub fn no_configuration() -> Self {
        use aws_types::os_shim_internal::ManualTimeSource;
        use std::collections::HashMap;
        use std::time::UNIX_EPOCH;
        Self {
            env: Env::from_slice(&[]),
            fs: Fs::from_raw_map(HashMap::new()),
            time_source: TimeSource::manual(&ManualTimeSource::new(UNIX_EPOCH)),
            connector: HttpConnector::Prebuilt(None),
            sleep: None,
            region: None,
        }
    }
}

impl ProviderConfig {
    /// Create a default provider config with the region unset.
    ///
    /// Using this option means that you may need to set a region manually.
    ///
    /// This constructor will use a default value for the HTTPS connector and Sleep implementation
    /// when they are enabled as crate features which is usually the correct option. To construct
    /// a `ProviderConfig` without these fields set, use [`ProviderConfig::empty`].
    ///
    ///
    /// # Examples
    /// ```no_run
    /// # #[cfg(any(feature = "rustls", feature = "native-tls"))]
    /// # fn example() {
    /// use aws_config::provider_config::ProviderConfig;
    /// use aws_sdk_sts::Region;
    /// use aws_config::web_identity_token::WebIdentityTokenCredentialsProvider;
    /// let conf = ProviderConfig::without_region().with_region(Some(Region::new("us-east-1")));
    ///
    /// let credential_provider = WebIdentityTokenCredentialsProvider::builder().configure(&conf).build();
    /// # }
    /// ```
    pub fn without_region() -> Self {
        Self::default()
    }

    /// Constructs a ProviderConfig with no fields set
    pub fn empty() -> Self {
        ProviderConfig {
            env: Env::default(),
            fs: Fs::default(),
            time_source: TimeSource::default(),
            connector: HttpConnector::Prebuilt(None),
            sleep: None,
            region: None,
        }
    }

    /// Create a default provider config with the region region automatically loaded from the default chain.
    ///
    /// # Examples
    /// ```no_run
    /// # async fn test() {
    /// use aws_config::provider_config::ProviderConfig;
    /// use aws_sdk_sts::Region;
    /// use aws_config::web_identity_token::WebIdentityTokenCredentialsProvider;
    /// let conf = ProviderConfig::with_default_region().await;
    /// let credential_provider = WebIdentityTokenCredentialsProvider::builder().configure(&conf).build();
    /// }
    /// ```
    pub async fn with_default_region() -> Self {
        Self::without_region().load_default_region().await
    }

    // When all crate features are disabled, these accessors are unused

    #[allow(dead_code)]
    pub(crate) fn env(&self) -> Env {
        self.env.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn fs(&self) -> Fs {
        self.fs.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn time_source(&self) -> TimeSource {
        self.time_source.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn default_connector(&self) -> Option<DynConnector> {
        self.connector
            .connector(&HttpSettings::default(), self.sleep.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn connector(&self, settings: &HttpSettings) -> Option<DynConnector> {
        self.connector.connector(settings, self.sleep.clone())
    }

    #[allow(dead_code)]
    pub(crate) fn sleep(&self) -> Option<Arc<dyn AsyncSleep>> {
        self.sleep.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn region(&self) -> Option<Region> {
        self.region.clone()
    }

    /// Override the region for the configuration
    pub fn with_region(mut self, region: Option<Region>) -> Self {
        self.region = region;
        self
    }

    /// Use the [default region chain](crate::default_provider::region) to set the
    /// region for this configuration
    ///
    /// Note: the `env` and `fs` already set on this provider will be used when loading the default region.
    pub async fn load_default_region(self) -> Self {
        use crate::default_provider::region::DefaultRegionChain;
        let provider_chain = DefaultRegionChain::builder().configure(&self).build();
        self.with_region(provider_chain.region().await)
    }

    // these setters are doc(hidden) because they only exist for tests

    #[doc(hidden)]
    pub fn with_fs(self, fs: Fs) -> Self {
        ProviderConfig { fs, ..self }
    }

    #[doc(hidden)]
    pub fn with_env(self, env: Env) -> Self {
        ProviderConfig { env, ..self }
    }

    #[doc(hidden)]
    pub fn with_time_source(self, time_source: TimeSource) -> Self {
        ProviderConfig {
            time_source,
            ..self
        }
    }

    /// Override the HTTPS connector for this configuration
    ///
    /// **Warning**: Use of this method will prevent you from taking advantage of the HTTP connect timeouts.
    /// Consider [`ProviderConfig::with_tcp_connector`].
    ///
    /// # Stability
    /// This method is expected to change to support HTTP configuration.
    pub fn with_http_connector(self, connector: DynConnector) -> Self {
        ProviderConfig {
            connector: HttpConnector::Prebuilt(Some(connector)),
            ..self
        }
    }

    /// Override the TCP connector for this configuration
    ///
    /// This connector MUST provide an HTTPS encrypted connection.
    ///
    /// # Stability
    /// This method may change to support HTTP configuration.
    pub fn with_tcp_connector<C>(self, connector: C) -> Self
    where
        C: Clone + Send + Sync + 'static,
        C: tower::Service<Uri>,
        C::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
        C::Future: Unpin + Send + 'static,
        C::Error: Into<Box<dyn Error + Send + Sync + 'static>>,
    {
        let connector_fn = move |settings: &HttpSettings, sleep: Option<Arc<dyn AsyncSleep>>| {
            let mut builder =
                aws_smithy_client::hyper_ext::Adapter::builder().timeout(&settings.timeout_config);
            if let Some(sleep) = sleep {
                builder = builder.sleep_impl(sleep);
            };
            Some(DynConnector::new(builder.build(connector.clone())))
        };
        ProviderConfig {
            connector: HttpConnector::ConnectorFn(Arc::new(connector_fn)),
            ..self
        }
    }

    /// Override the sleep implementation for this configuration
    pub fn with_sleep(self, sleep: impl AsyncSleep + 'static) -> Self {
        ProviderConfig {
            sleep: Some(Arc::new(sleep)),
            ..self
        }
    }
}
