/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Client configuration and request helpers shared by connection behavior tests.

use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
use aws_smithy_async::time::{SystemTimeSource, TimeSource};
use aws_smithy_http_client::pool::{Client as PoolClient, ConnectionPool};
use aws_smithy_http_client::proxy::ProxyConfig;
#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
use aws_smithy_http_client::tls;
use aws_smithy_http_client::{Builder, Connector};
use aws_smithy_runtime_api::client::dns::SharedDnsResolver;
use aws_smithy_runtime_api::client::http::{
    http_client_fn, HttpClient, HttpConnector, HttpConnectorSettings, SharedHttpClient,
    SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use http_body_util::BodyExt;
use std::time::Duration;

/// Default timeout for test waits and deadline assertions.
pub(crate) const WAIT: Duration = Duration::from_secs(5);

/// Settings applied uniformly by backend-neutral connection contracts.
#[derive(Clone, Debug, Default)]
pub(crate) struct BackendConfig {
    pub(crate) pool_idle_timeout: Option<Duration>,
    pub(crate) proxy_config: Option<ProxyConfig>,
    pub(crate) dns_resolver: Option<SharedDnsResolver>,
}

/// Hyper 1.x through `hyper_util::client::legacy::Client`.
///
/// "Legacy" is Hyper Util's module name and does not refer to smithy-rs's `hyper-014` feature.
#[derive(Clone, Copy, Debug)]
pub(crate) struct HyperUtilLegacyPool;

/// The partition-aware Smithy connection pool.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PartitionedConnectionPool;

/// Builds a cleartext client for one connection-pool implementation.
#[allow(dead_code)]
pub(crate) trait HttpClientBackend {
    fn build(&self, config: BackendConfig) -> SharedHttpClient;
}

impl HttpClientBackend for HyperUtilLegacyPool {
    fn build(&self, config: BackendConfig) -> SharedHttpClient {
        let BackendConfig {
            pool_idle_timeout,
            proxy_config,
            dns_resolver,
        } = config;
        if proxy_config.is_none() && dns_resolver.is_none() {
            let mut builder = Builder::new();
            if let Some(pool_idle_timeout) = pool_idle_timeout {
                builder = builder.pool_idle_timeout(pool_idle_timeout);
            }
            return builder.build_http();
        }

        http_client_fn(move |settings, _| {
            let mut builder = Connector::builder().connector_settings(settings.clone());
            if let Some(pool_idle_timeout) = pool_idle_timeout {
                builder = builder.pool_idle_timeout(Some(pool_idle_timeout));
            }
            if let Some(proxy_config) = &proxy_config {
                builder = builder.proxy_config(proxy_config.clone());
            }
            if let Some(resolver) = &dns_resolver {
                #[cfg(any(
                    feature = "rustls-aws-lc",
                    feature = "rustls-aws-lc-fips",
                    feature = "rustls-ring",
                    feature = "s2n-tls"
                ))]
                return SharedHttpConnector::new(
                    builder
                        .tls_provider(custom_dns_test_provider())
                        .build_with_resolver(resolver.clone()),
                );
                #[cfg(not(any(
                    feature = "rustls-aws-lc",
                    feature = "rustls-aws-lc-fips",
                    feature = "rustls-ring",
                    feature = "s2n-tls"
                )))]
                {
                    let _ = resolver;
                    panic!("the legacy custom-DNS test backend requires an enabled TLS provider");
                }
            }
            SharedHttpConnector::new(builder.build_http())
        })
    }
}

impl HttpClientBackend for PartitionedConnectionPool {
    fn build(&self, config: BackendConfig) -> SharedHttpClient {
        let mut builder = ConnectionPool::builder();
        if let Some(dns_resolver) = config.dns_resolver {
            builder = builder.dns_resolver(dns_resolver);
        }
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.idle_timeout(pool_idle_timeout);
        }
        if let Some(proxy_config) = config.proxy_config {
            builder = builder.proxy_config(proxy_config);
        }
        let pool = builder.build_http().expect("valid connection-pool config");
        SharedHttpClient::new(PoolClient::new(&pool).expect("anonymous partition exists"))
    }
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
fn custom_dns_test_provider() -> tls::Provider {
    #[cfg(feature = "rustls-aws-lc")]
    {
        tls::Provider::Rustls(tls::rustls_provider::CryptoMode::AwsLc)
    }
    #[cfg(all(not(feature = "rustls-aws-lc"), feature = "rustls-aws-lc-fips"))]
    {
        tls::Provider::Rustls(tls::rustls_provider::CryptoMode::AwsLcFips)
    }
    #[cfg(all(
        not(feature = "rustls-aws-lc"),
        not(feature = "rustls-aws-lc-fips"),
        feature = "rustls-ring"
    ))]
    {
        tls::Provider::Rustls(tls::rustls_provider::CryptoMode::Ring)
    }
    #[cfg(all(
        not(feature = "rustls-aws-lc"),
        not(feature = "rustls-aws-lc-fips"),
        not(feature = "rustls-ring"),
        feature = "s2n-tls"
    ))]
    {
        tls::Provider::S2nTls
    }
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
/// Builds a TLS client for one connection-pool implementation.
#[allow(dead_code)]
pub(crate) trait HttpsClientBackend {
    fn build_https(
        &self,
        config: BackendConfig,
        provider: tls::Provider,
        tls_context: tls::TlsContext,
    ) -> SharedHttpClient;
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
impl HttpsClientBackend for HyperUtilLegacyPool {
    fn build_https(
        &self,
        config: BackendConfig,
        provider: tls::Provider,
        tls_context: tls::TlsContext,
    ) -> SharedHttpClient {
        let BackendConfig {
            pool_idle_timeout,
            proxy_config,
            dns_resolver,
        } = config;
        if proxy_config.is_none() && dns_resolver.is_none() {
            let mut builder = Builder::new();
            if let Some(pool_idle_timeout) = pool_idle_timeout {
                builder = builder.pool_idle_timeout(pool_idle_timeout);
            }
            return builder
                .tls_provider(provider)
                .tls_context(tls_context)
                .build_https();
        };

        http_client_fn(move |settings, _| {
            let mut builder = Connector::builder()
                .connector_settings(settings.clone())
                .tls_provider(provider.clone())
                .tls_context(tls_context.clone());
            if let Some(proxy_config) = &proxy_config {
                builder = builder.proxy_config(proxy_config.clone());
            }
            if let Some(pool_idle_timeout) = pool_idle_timeout {
                builder = builder.pool_idle_timeout(Some(pool_idle_timeout));
            }
            match &dns_resolver {
                Some(resolver) => {
                    SharedHttpConnector::new(builder.build_with_resolver(resolver.clone()))
                }
                None => SharedHttpConnector::new(builder.build()),
            }
        })
    }
}

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
impl HttpsClientBackend for PartitionedConnectionPool {
    fn build_https(
        &self,
        config: BackendConfig,
        provider: tls::Provider,
        tls_context: tls::TlsContext,
    ) -> SharedHttpClient {
        let mut builder = ConnectionPool::builder();
        if let Some(dns_resolver) = config.dns_resolver {
            builder = builder.dns_resolver(dns_resolver);
        }
        if let Some(pool_idle_timeout) = config.pool_idle_timeout {
            builder = builder.idle_timeout(pool_idle_timeout);
        }
        if let Some(proxy_config) = config.proxy_config {
            builder = builder.proxy_config(proxy_config);
        }
        let pool = builder
            .tls_provider(provider)
            .tls_context(tls_context)
            .build_https()
            .expect("valid connection-pool config");
        SharedHttpClient::new(PoolClient::new(&pool).expect("anonymous partition exists"))
    }
}

pub(crate) fn runtime_components() -> RuntimeComponents {
    RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .with_sleep_impl(Some(SharedAsyncSleep::new(TokioSleep::new())))
        .build()
        .expect("valid runtime components")
}

pub(crate) fn connector_with_settings(
    client: &SharedHttpClient,
    settings: HttpConnectorSettings,
) -> SharedHttpConnector {
    client.http_connector(&settings, &runtime_components())
}

pub(crate) fn connector(client: &SharedHttpClient) -> SharedHttpConnector {
    connector_with_settings(client, HttpConnectorSettings::builder().build())
}

/// Builds a connector whose request telemetry uses `time_source`.
#[allow(dead_code)]
pub(crate) fn connector_with_time_source(
    client: &SharedHttpClient,
    time_source: impl TimeSource + 'static,
) -> SharedHttpConnector {
    let components = RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(time_source))
        .with_sleep_impl(Some(SharedAsyncSleep::new(TokioSleep::new())))
        .build()
        .expect("valid runtime components");
    client.http_connector(&HttpConnectorSettings::builder().build(), &components)
}

pub(crate) async fn send_request(
    connector: &SharedHttpConnector,
    request: HttpRequest,
) -> Result<HttpResponse, ConnectorError> {
    tokio::time::timeout(WAIT, connector.call(request))
        .await
        .expect("request should finish within the outer deadline")
}

pub(crate) async fn send_and_collect(
    connector: &SharedHttpConnector,
    request: HttpRequest,
) -> (u16, Vec<u8>) {
    let response = send_request(connector, request)
        .await
        .expect("request should succeed");
    collect_response(response).await
}

pub(crate) async fn get_and_collect(connector: &SharedHttpConnector, url: &str) -> (u16, Vec<u8>) {
    send_and_collect(
        connector,
        HttpRequest::get(url).expect("valid HTTP request"),
    )
    .await
}

pub(crate) async fn collect_response(response: HttpResponse) -> (u16, Vec<u8>) {
    let status = response.status().as_u16();
    let body = tokio::time::timeout(WAIT, response.into_body().collect())
        .await
        .expect("response body should finish within the outer deadline")
        .expect("response body should be readable")
        .to_bytes()
        .to_vec();
    (status, body)
}
