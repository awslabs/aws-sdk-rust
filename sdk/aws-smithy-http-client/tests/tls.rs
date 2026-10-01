/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(any(feature = "__rustls", feature = "s2n-tls",))]

mod common {
    #[allow(dead_code)]
    pub(crate) mod client;
    pub(crate) mod tls;
}

use aws_smithy_async::time::SystemTimeSource;
#[cfg(all(feature = "rustls-aws-lc", feature = "rt-tokio"))]
use aws_smithy_http_client::pool::{Client as PoolClient, ConnectionPool};
use aws_smithy_http_client::tls;
#[cfg(any(feature = "rustls-aws-lc", feature = "s2n-tls"))]
use aws_smithy_http_client::tls::{ServerName, TlsContext};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::http::{HttpClient, HttpConnector, HttpConnectorSettings};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
use aws_smithy_types::byte_stream::ByteStream;
#[cfg(feature = "rt-tokio")]
use common::client::PartitionedConnectionPool;
use common::client::{BackendConfig, HttpsClientBackend, HyperUtilLegacyPool};
use common::tls as test_tls;
use http_1x::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing::{debug, error};

/// HTTPS echo server with configurable ALPN and observable connection lifetime.
struct TlsEchoServer {
    task: JoinHandle<()>,
    addr: SocketAddr,
    active_connections: Arc<()>,
}

impl TlsEchoServer {
    /// Starts a server advertising HTTP/2 and HTTP/1 protocols.
    async fn start() -> Result<Self, BoxError> {
        Self::start_with_alpn(&[b"h2", b"http/1.1", b"http/1.0"]).await
    }

    /// Starts a server advertising the given ALPN protocols in order.
    async fn start_with_alpn(alpn_protocols: &[&[u8]]) -> Result<Self, BoxError> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        debug!("Starting to serve on https://{}", addr);

        let tls_acceptor = test_tls::SERVER_IDENTITY.acceptor(alpn_protocols)?;
        let service = service_fn(echo);
        let active_connections = Arc::new(());
        let listener_connections = active_connections.clone();

        let task = tokio::spawn(async move {
            loop {
                let (tcp_stream, remote_addr) = listener.accept().await.unwrap();
                debug!("accepted connection from: {}", remote_addr);

                let tls_acceptor = tls_acceptor.clone();
                let connection = listener_connections.clone();
                tokio::spawn(async move {
                    let _connection = connection;
                    let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                        Ok(tls_stream) => tls_stream,
                        Err(err) => {
                            error!("failed to perform tls handshake: {err:#}");
                            return;
                        }
                    };
                    if let Err(err) = Builder::new(TokioExecutor::new())
                        .serve_connection(TokioIo::new(tls_stream), service)
                        .await
                    {
                        error!("failed to serve connection: {err:#}");
                    }
                });
            }
        });

        Ok(Self {
            task,
            addr,
            active_connections,
        })
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Returns the number of accepted connections that remain open.
    fn active_connection_count(&self) -> usize {
        // The server value and listener task each hold one baseline reference.
        Arc::strong_count(&self.active_connections)
            .checked_sub(2)
            .expect("active connection reference count includes both owners")
    }
}

impl Drop for TlsEchoServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Handles the request paths exercised by the TLS contracts.
async fn echo(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let mut response = Response::new(Full::default());
    match (req.method(), req.uri().path()) {
        // default route.
        (&Method::GET, "/") => {
            *response.body_mut() = Full::from("Hello TLS!");
        }
        // echo service route.
        (&Method::POST, "/echo") => {
            *response.body_mut() = Full::from(req.into_body().collect().await?.to_bytes());
        }
        // Catch-all 404.
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Ok(response)
}

async fn native_ca_rejects_test_certificate(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
    expected_error: &str,
) {
    let client = backend.build_https(
        BackendConfig::default(),
        provider,
        tls::TlsContext::default(),
    );
    let error = run_tls_test(&client)
        .await
        .expect_err("the native trust store must reject the test certificate");
    let error = format!("{error:?}");
    assert!(
        error.contains(expected_error),
        "expected TLS error containing {expected_error:?}, got {error}"
    );
}

async fn custom_ca_accepts_test_certificate(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let client = backend.build_https(
        BackendConfig::default(),
        provider,
        test_tls::SERVER_IDENTITY.client_context(),
    );
    run_tls_test(&client).await.unwrap();
}

#[cfg(feature = "rustls-aws-lc")]
fn rustls_aws_lc() -> tls::Provider {
    tls::Provider::Rustls(tls::rustls_provider::CryptoMode::AwsLc)
}

#[cfg(feature = "rustls-aws-lc")]
#[tokio::test]
async fn test_rustls_aws_lc_native_ca_with_hyper_util_legacy_pool() {
    native_ca_rejects_test_certificate(
        &HyperUtilLegacyPool,
        rustls_aws_lc(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-aws-lc")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_aws_lc_native_ca_with_partitioned_connection_pool() {
    native_ca_rejects_test_certificate(
        &PartitionedConnectionPool,
        rustls_aws_lc(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-aws-lc")]
#[tokio::test]
async fn test_rustls_aws_lc_custom_ca_with_hyper_util_legacy_pool() {
    custom_ca_accepts_test_certificate(&HyperUtilLegacyPool, rustls_aws_lc()).await;
}

#[cfg(feature = "rustls-aws-lc")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_aws_lc_custom_ca_with_partitioned_connection_pool() {
    custom_ca_accepts_test_certificate(&PartitionedConnectionPool, rustls_aws_lc()).await;
}

#[cfg(all(feature = "rustls-aws-lc", feature = "rt-tokio"))]
#[tokio::test]
async fn partitioned_pool_falls_back_to_h1_after_alpn() {
    let server = TlsEchoServer::start_with_alpn(&[b"http/1.1"])
        .await
        .unwrap();
    let pool = ConnectionPool::builder()
        .tls_provider(tls::Provider::Rustls(
            tls::rustls_provider::CryptoMode::AwsLc,
        ))
        .tls_context(test_tls::SERVER_IDENTITY.client_context())
        .build_https()
        .expect("valid HTTPS pool");
    let client = PoolClient::new(&pool).expect("anonymous partition exists");
    let connector_settings = HttpConnectorSettings::builder().build();
    let runtime_components = RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .build()
        .unwrap();
    let connector = client.http_connector(&connector_settings, &runtime_components);
    let endpoint = format!("https://localhost:{}/", server.addr().port());

    for _ in 0..2 {
        let mut response = connector
            .call(HttpRequest::get(&endpoint).unwrap())
            .await
            .expect("HTTP/1 request over TLS should succeed");
        let body = ByteStream::new(response.take_body())
            .collect()
            .await
            .expect("response body should be readable")
            .into_bytes();
        assert_eq!(b"Hello TLS!", &body[..]);
    }
    assert_eq!(1, server.active_connection_count());
}

#[cfg(all(feature = "rustls-aws-lc", feature = "rt-tokio"))]
#[tokio::test]
async fn partitioned_pool_narrows_alpn_for_h1_required_request() {
    let server = TlsEchoServer::start_with_alpn(&[b"h2", b"http/1.1"])
        .await
        .unwrap();
    let pool = ConnectionPool::builder()
        .tls_provider(tls::Provider::Rustls(
            tls::rustls_provider::CryptoMode::AwsLc,
        ))
        .tls_context(test_tls::SERVER_IDENTITY.client_context())
        .build_https()
        .expect("valid HTTPS pool");
    let client = PoolClient::new(&pool).expect("anonymous partition exists");
    let connector_settings = HttpConnectorSettings::builder().build();
    let runtime_components = RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .build()
        .unwrap();
    let connector = client.http_connector(&connector_settings, &runtime_components);
    let endpoint = format!("https://localhost:{}/", server.addr().port());
    let mut request = HttpRequest::new(ByteStream::default().into_inner());
    request
        .set_method("CONNECT")
        .expect("CONNECT is a valid HTTP method");
    request
        .set_uri(endpoint.as_str())
        .expect("valid server URI");

    let response = connector
        .call(request)
        .await
        .expect("HTTP/1-required request should narrow the ALPN offer");

    assert_eq!(StatusCode::NOT_FOUND.as_u16(), response.status().as_u16());
    assert_eq!(1, server.active_connection_count());
}

#[cfg(feature = "rustls-aws-lc")]
async fn custom_ca_connection_obeys_idle_timeout(backend: &dyn HttpsClientBackend) {
    const TIMEOUT: Duration = Duration::from_secs(10);
    let client = backend.build_https(
        BackendConfig {
            pool_idle_timeout: Some(TIMEOUT),
            ..Default::default()
        },
        rustls_aws_lc(),
        test_tls::SERVER_IDENTITY.client_context(),
    );
    run_tls_test_with_idle_timeout(&client, Some(TIMEOUT))
        .await
        .unwrap();
}

#[cfg(feature = "rustls-aws-lc")]
#[tokio::test(start_paused = false)]
// can't have paused clock due to <https://github.com/hyperium/hyper/issues/3950>
async fn test_rustls_aws_lc_custom_ca_idle_timeout_with_hyper_util_legacy_pool() {
    custom_ca_connection_obeys_idle_timeout(&HyperUtilLegacyPool).await;
}

#[cfg(feature = "rustls-aws-lc")]
#[cfg(feature = "rt-tokio")]
#[tokio::test(start_paused = false)]
// can't have paused clock due to <https://github.com/hyperium/hyper/issues/3950>
async fn test_rustls_aws_lc_custom_ca_idle_timeout_with_partitioned_connection_pool() {
    custom_ca_connection_obeys_idle_timeout(&PartitionedConnectionPool).await;
}

#[cfg(feature = "rustls-aws-lc-fips")]
fn rustls_aws_lc_fips() -> tls::Provider {
    tls::Provider::Rustls(tls::rustls_provider::CryptoMode::AwsLcFips)
}

#[cfg(feature = "rustls-aws-lc-fips")]
#[tokio::test]
async fn test_rustls_aws_lc_fips_native_ca_with_hyper_util_legacy_pool() {
    native_ca_rejects_test_certificate(
        &HyperUtilLegacyPool,
        rustls_aws_lc_fips(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-aws-lc-fips")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_aws_lc_fips_native_ca_with_partitioned_connection_pool() {
    native_ca_rejects_test_certificate(
        &PartitionedConnectionPool,
        rustls_aws_lc_fips(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-aws-lc-fips")]
#[tokio::test]
async fn test_rustls_aws_lc_fips_custom_ca_with_hyper_util_legacy_pool() {
    custom_ca_accepts_test_certificate(&HyperUtilLegacyPool, rustls_aws_lc_fips()).await;
}

#[cfg(feature = "rustls-aws-lc-fips")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_aws_lc_fips_custom_ca_with_partitioned_connection_pool() {
    custom_ca_accepts_test_certificate(&PartitionedConnectionPool, rustls_aws_lc_fips()).await;
}

#[cfg(feature = "rustls-ring")]
fn rustls_ring() -> tls::Provider {
    tls::Provider::Rustls(tls::rustls_provider::CryptoMode::Ring)
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_rustls_ring_native_ca_with_hyper_util_legacy_pool() {
    native_ca_rejects_test_certificate(
        &HyperUtilLegacyPool,
        rustls_ring(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_ring_native_ca_with_partitioned_connection_pool() {
    native_ca_rejects_test_certificate(
        &PartitionedConnectionPool,
        rustls_ring(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_rustls_ring_custom_ca_with_hyper_util_legacy_pool() {
    custom_ca_accepts_test_certificate(&HyperUtilLegacyPool, rustls_ring()).await;
}

#[cfg(feature = "rustls-ring")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_ring_custom_ca_with_partitioned_connection_pool() {
    custom_ca_accepts_test_certificate(&PartitionedConnectionPool, rustls_ring()).await;
}

#[cfg(all(aws_sdk_unstable, feature = "rustls-ring"))]
fn rustls_custom_provider() -> tls::Provider {
    tls::Provider::Rustls(tls::rustls_provider::CryptoMode::Custom(
        rustls::crypto::ring::default_provider(),
    ))
}

#[cfg(all(aws_sdk_unstable, feature = "rustls-ring"))]
#[tokio::test]
async fn test_rustls_custom_provider_native_ca_with_hyper_util_legacy_pool() {
    native_ca_rejects_test_certificate(
        &HyperUtilLegacyPool,
        rustls_custom_provider(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(all(aws_sdk_unstable, feature = "rustls-ring"))]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_custom_provider_native_ca_with_partitioned_connection_pool() {
    native_ca_rejects_test_certificate(
        &PartitionedConnectionPool,
        rustls_custom_provider(),
        "InvalidCertificate(UnknownIssuer)",
    )
    .await;
}

#[cfg(all(aws_sdk_unstable, feature = "rustls-ring"))]
#[tokio::test]
async fn test_rustls_custom_provider_custom_ca_with_hyper_util_legacy_pool() {
    custom_ca_accepts_test_certificate(&HyperUtilLegacyPool, rustls_custom_provider()).await;
}

#[cfg(all(aws_sdk_unstable, feature = "rustls-ring"))]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_rustls_custom_provider_custom_ca_with_partitioned_connection_pool() {
    custom_ca_accepts_test_certificate(&PartitionedConnectionPool, rustls_custom_provider()).await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_s2n_native_ca_with_hyper_util_legacy_pool() {
    native_ca_rejects_test_certificate(
        &HyperUtilLegacyPool,
        tls::Provider::S2nTls,
        "Certificate is untrusted",
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_s2n_native_ca_with_partitioned_connection_pool() {
    native_ca_rejects_test_certificate(
        &PartitionedConnectionPool,
        tls::Provider::S2nTls,
        "Certificate is untrusted",
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_s2n_tls_custom_ca_with_hyper_util_legacy_pool() {
    custom_ca_accepts_test_certificate(&HyperUtilLegacyPool, tls::Provider::S2nTls).await;
}

#[cfg(feature = "s2n-tls")]
#[cfg(feature = "rt-tokio")]
#[tokio::test]
async fn test_s2n_tls_custom_ca_with_partitioned_connection_pool() {
    custom_ca_accepts_test_certificate(&PartitionedConnectionPool, tls::Provider::S2nTls).await;
}

async fn run_tls_test(client: &dyn HttpClient) -> Result<(), BoxError> {
    run_tls_test_with_idle_timeout(client, None).await
}

async fn run_tls_test_with_idle_timeout(
    client: &dyn HttpClient,
    pool_timeout: Option<Duration>,
) -> Result<(), BoxError> {
    let server = TlsEchoServer::start().await?;
    let start = tokio::time::Instant::now();
    assert_eq!(server.active_connection_count(), 0);
    let endpoint = format!("https://localhost:{}/", server.addr().port());

    let connector_settings = HttpConnectorSettings::builder().build();
    let runtime_components = RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .build()
        .unwrap();
    let connector = client.http_connector(&connector_settings, &runtime_components);
    let mut response = connector.call(HttpRequest::get(endpoint).unwrap()).await?;

    let sdk_body = response.take_body();
    let body_stream = ByteStream::new(sdk_body);
    let resp_bytes = body_stream.collect().await?.into_bytes();
    assert_eq!(b"Hello TLS!", &resp_bytes[..]);

    if let Some(pool_timeout) = pool_timeout {
        assert_eq!(server.active_connection_count(), 1);
        tokio::time::sleep_until(start + pool_timeout - Duration::from_secs(1)).await;
        assert_eq!(server.active_connection_count(), 1);
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(server.active_connection_count(), 0);
    }
    Ok(())
}

/// Like `run_tls_test` but connects via 127.0.0.1 instead of localhost.
/// The test cert's SANs only include "localhost" and "sdktest.com", so
/// connecting by IP will fail hostname verification unless additional
/// server names are configured.
#[cfg(any(feature = "rustls-aws-lc", feature = "s2n-tls"))]
async fn run_tls_test_to_ip(client: &dyn HttpClient) -> Result<(), BoxError> {
    let server = TlsEchoServer::start().await?;
    let endpoint = format!("https://127.0.0.1:{}/", server.addr().port());

    let connector_settings = HttpConnectorSettings::builder().build();
    let runtime_components = RuntimeComponentsBuilder::for_tests()
        .with_time_source(Some(SystemTimeSource::new()))
        .build()
        .unwrap();
    let connector = client.http_connector(&connector_settings, &runtime_components);
    let mut response = connector.call(HttpRequest::get(endpoint).unwrap()).await?;

    let sdk_body = response.take_body();
    let body_stream = ByteStream::new(sdk_body);
    let resp_bytes = body_stream.collect().await?.into_bytes();
    assert_eq!(b"Hello TLS!", &resp_bytes[..]);
    Ok(())
}

#[cfg(any(feature = "rustls-aws-lc", feature = "s2n-tls"))]
mod additional_server_names {
    //! Certificate-name contracts for each TLS provider and client backend.

    use super::*;

    fn client_context(additional_server_names: &[&str]) -> TlsContext {
        let additional_server_names = additional_server_names
            .iter()
            .map(|name| {
                ServerName::try_from(name.to_string()).expect("additional server name is valid")
            })
            .collect();
        test_tls::SERVER_IDENTITY
            .client_context_builder()
            .with_additional_server_names(additional_server_names)
            .build()
            .expect("additional server names produce a valid client context")
    }

    async fn assert_ip_rejected(
        backend: &dyn HttpsClientBackend,
        provider: tls::Provider,
        tls_context: TlsContext,
        expected_error: &str,
    ) {
        let client = backend.build_https(BackendConfig::default(), provider, tls_context);
        let error = run_tls_test_to_ip(&client)
            .await
            .expect_err("the certificate must not validate for the request IP");
        let error = format!("{error:?}");
        assert!(
            error.contains(expected_error),
            "expected TLS error containing {expected_error:?}, got {error}"
        );
    }

    async fn assert_ip_accepted(backend: &dyn HttpsClientBackend, provider: tls::Provider) {
        let client = backend.build_https(
            BackendConfig::default(),
            provider,
            client_context(&["localhost"]),
        );
        run_tls_test_to_ip(&client).await.unwrap();
    }

    async fn assert_primary_name_accepted(
        backend: &dyn HttpsClientBackend,
        provider: tls::Provider,
    ) {
        let client = backend.build_https(
            BackendConfig::default(),
            provider,
            client_context(&["sdktest.com"]),
        );
        run_tls_test(&client).await.unwrap();
    }

    #[cfg(feature = "rustls-aws-lc")]
    mod rustls_aws_lc {
        use super::*;
        use crate::rustls_aws_lc as provider;

        const CERTIFICATE_ERROR: &str = "InvalidCertificate";

        async fn missing_name_is_rejected(backend: &dyn HttpsClientBackend) {
            assert_ip_rejected(
                backend,
                provider(),
                test_tls::SERVER_IDENTITY.client_context(),
                CERTIFICATE_ERROR,
            )
            .await;
        }

        async fn wrong_name_is_rejected(backend: &dyn HttpsClientBackend) {
            assert_ip_rejected(
                backend,
                provider(),
                client_context(&["wrong.example.com"]),
                CERTIFICATE_ERROR,
            )
            .await;
        }

        async fn matching_name_is_accepted(backend: &dyn HttpsClientBackend) {
            assert_ip_accepted(backend, provider()).await;
        }

        async fn primary_name_is_preserved(backend: &dyn HttpsClientBackend) {
            assert_primary_name_accepted(backend, provider()).await;
        }

        mod hyper_util_legacy_pool {
            use super::*;

            #[tokio::test]
            async fn test_missing_name_is_rejected() {
                missing_name_is_rejected(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_wrong_name_is_rejected() {
                wrong_name_is_rejected(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_matching_name_is_accepted() {
                matching_name_is_accepted(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_primary_name_is_preserved() {
                primary_name_is_preserved(&HyperUtilLegacyPool).await;
            }
        }

        #[cfg(feature = "rt-tokio")]
        mod partitioned_connection_pool {
            use super::*;

            #[tokio::test]
            async fn test_missing_name_is_rejected() {
                missing_name_is_rejected(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_wrong_name_is_rejected() {
                wrong_name_is_rejected(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_matching_name_is_accepted() {
                matching_name_is_accepted(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_primary_name_is_preserved() {
                primary_name_is_preserved(&PartitionedConnectionPool).await;
            }
        }
    }

    #[cfg(feature = "s2n-tls")]
    mod s2n_tls {
        use super::*;

        const CERTIFICATE_ERROR: &str = "Certificate is not valid for the supplied hostname";

        async fn missing_name_is_rejected(backend: &dyn HttpsClientBackend) {
            assert_ip_rejected(
                backend,
                tls::Provider::S2nTls,
                test_tls::SERVER_IDENTITY.client_context(),
                CERTIFICATE_ERROR,
            )
            .await;
        }

        async fn wrong_name_is_rejected(backend: &dyn HttpsClientBackend) {
            assert_ip_rejected(
                backend,
                tls::Provider::S2nTls,
                client_context(&["wrong.example.com"]),
                CERTIFICATE_ERROR,
            )
            .await;
        }

        async fn matching_name_is_accepted(backend: &dyn HttpsClientBackend) {
            assert_ip_accepted(backend, tls::Provider::S2nTls).await;
        }

        async fn primary_name_is_preserved(backend: &dyn HttpsClientBackend) {
            assert_primary_name_accepted(backend, tls::Provider::S2nTls).await;
        }

        mod hyper_util_legacy_pool {
            use super::*;

            #[tokio::test]
            async fn test_missing_name_is_rejected() {
                missing_name_is_rejected(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_wrong_name_is_rejected() {
                wrong_name_is_rejected(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_matching_name_is_accepted() {
                matching_name_is_accepted(&HyperUtilLegacyPool).await;
            }

            #[tokio::test]
            async fn test_primary_name_is_preserved() {
                primary_name_is_preserved(&HyperUtilLegacyPool).await;
            }
        }

        #[cfg(feature = "rt-tokio")]
        mod partitioned_connection_pool {
            use super::*;

            #[tokio::test]
            async fn test_missing_name_is_rejected() {
                missing_name_is_rejected(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_wrong_name_is_rejected() {
                wrong_name_is_rejected(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_matching_name_is_accepted() {
                matching_name_is_accepted(&PartitionedConnectionPool).await;
            }

            #[tokio::test]
            async fn test_primary_name_is_preserved() {
                primary_name_is_preserved(&PartitionedConnectionPool).await;
            }
        }
    }
}
