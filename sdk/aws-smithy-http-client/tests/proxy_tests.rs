/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Proxy behavior contracts shared by the legacy and partitioned connection pools.
//!
//! Each contract is backend-neutral and has an explicit runner for every client
//! implementation that must preserve the behavior.

#![cfg(feature = "default-client")]

mod common {
    #[allow(dead_code)]
    pub(crate) mod client;
    #[cfg(all(
        feature = "wire-mock",
        any(feature = "rustls-ring", feature = "s2n-tls")
    ))]
    #[allow(dead_code)]
    pub(crate) mod h2;
    pub(crate) mod proxy;
    #[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
    pub(crate) mod tls;
}

use aws_smithy_http_client::proxy::ProxyConfig;
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use aws_smithy_http_client::tls;
use aws_smithy_runtime_api::box_error::BoxError;
#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
use aws_smithy_runtime_api::client::dns::{
    DnsFuture, ResolveDns, ResolveDnsError, SharedDnsResolver,
};
use aws_smithy_runtime_api::client::http::{HttpConnector, SharedHttpClient, SharedHttpConnector};
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use common::client::HttpsClientBackend;
use common::client::{
    self as test_client, BackendConfig, HttpClientBackend, HyperUtilLegacyPool,
    PartitionedConnectionPool,
};
#[cfg(all(
    feature = "wire-mock",
    any(feature = "rustls-ring", feature = "s2n-tls")
))]
use common::h2::{
    H2ConnectionPlan, H2ConnectionScript, H2Event, H2Response, H2StreamScript, H2TestServer,
};
use common::proxy::{basic_authorization, MockHttpServer};
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use common::proxy::{MockConnectProxy, MockTlsOrigin};
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use common::tls as test_tls;
use http_1x::{Response, StatusCode};
use http_body_util::BodyExt;
use std::future::Future;
use std::time::Duration;

struct TestClient {
    _client: SharedHttpClient,
    connector: SharedHttpConnector,
}

fn http_client(backend: &dyn HttpClientBackend, config: BackendConfig) -> TestClient {
    let client = backend.build(config);
    let connector = test_client::connector(&client);
    TestClient {
        _client: client,
        connector,
    }
}

fn proxy_backend_config(proxy_config: ProxyConfig) -> BackendConfig {
    BackendConfig {
        proxy_config: Some(proxy_config),
        ..Default::default()
    }
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
fn https_client(
    backend: &dyn HttpsClientBackend,
    config: BackendConfig,
    provider: tls::Provider,
    tls_context: tls::TlsContext,
) -> TestClient {
    let client = backend.build_https(config, provider, tls_context);
    let connector = test_client::connector(&client);
    TestClient {
        _client: client,
        connector,
    }
}

async fn send_request(
    client: &TestClient,
    request: HttpRequest,
) -> Result<(StatusCode, String), BoxError> {
    let response = client.connector.call(request).await?;
    let status = StatusCode::from_u16(response.status().as_u16())?;
    let body = response.into_body().collect().await?.to_bytes();
    Ok((status, String::from_utf8(body.to_vec())?))
}

async fn get(client: &TestClient, uri: &str) -> Result<(StatusCode, String), BoxError> {
    send_request(client, HttpRequest::get(uri)?).await
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
#[derive(Clone, Debug)]
struct FailingDns;

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
impl ResolveDns for FailingDns {
    fn resolve_dns<'a>(&'a self, name: &'a str) -> DnsFuture<'a> {
        DnsFuture::ready(Err(ResolveDnsError::new(std::io::Error::other(format!(
            "intentional DNS failure for {name}"
        )))))
    }
}

#[cfg(all(
    feature = "wire-mock",
    any(feature = "rustls-ring", feature = "s2n-tls")
))]
async fn h2_forward_proxy_applies_configured_authentication(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let server = H2TestServer::builder()
        .connections(H2ConnectionPlan::queue([H2ConnectionScript::new().route(
            "/protected",
            H2StreamScript::respond(H2Response::ok("authenticated over H2")),
        )]))
        .start()
        .await
        .expect("H2 forward proxy should start");
    let config = ProxyConfig::http(server.url("/"))
        .expect("valid proxy URI")
        .with_basic_auth("h2user", "h2pass");
    let client = https_client(
        backend,
        proxy_backend_config(config),
        provider,
        test_tls::SERVER_IDENTITY.client_context(),
    );

    assert_eq!(
        (StatusCode::OK, "authenticated over H2".to_string()),
        get(&client, "http://origin.test/protected")
            .await
            .expect("H2 forward-proxy request should succeed")
    );
    let expected = basic_authorization("h2user", "h2pass");
    let authorization = server.events().into_iter().find_map(|event| match event {
        H2Event::StreamAccepted {
            path,
            proxy_authorization,
            ..
        } if path == "/protected" => proxy_authorization,
        _ => None,
    });
    assert_eq!(Some(expected), authorization);

    drop(client);
    server
        .shutdown()
        .await
        .expect("clean H2 forward-proxy shutdown");
}

#[cfg(all(feature = "wire-mock", feature = "rustls-ring"))]
#[tokio::test]
async fn test_h2_forward_proxy_auth_with_rustls_and_hyper_util_legacy_pool() {
    h2_forward_proxy_applies_configured_authentication(
        &HyperUtilLegacyPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(all(feature = "wire-mock", feature = "rustls-ring"))]
#[tokio::test]
async fn test_h2_forward_proxy_auth_with_rustls_and_partitioned_connection_pool() {
    h2_forward_proxy_applies_configured_authentication(
        &PartitionedConnectionPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(all(feature = "wire-mock", feature = "s2n-tls"))]
#[tokio::test]
async fn test_h2_forward_proxy_auth_with_s2n_tls_and_hyper_util_legacy_pool() {
    h2_forward_proxy_applies_configured_authentication(&HyperUtilLegacyPool, tls::Provider::S2nTls)
        .await;
}

#[cfg(all(feature = "wire-mock", feature = "s2n-tls"))]
#[tokio::test]
async fn test_h2_forward_proxy_auth_with_s2n_tls_and_partitioned_connection_pool() {
    h2_forward_proxy_applies_configured_authentication(
        &PartitionedConnectionPool,
        tls::Provider::S2nTls,
    )
    .await;
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn https_proxy_dns_failure_remains_retryable_io(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let client = https_client(
        backend,
        BackendConfig {
            proxy_config: Some(
                ProxyConfig::all("http://proxy.invalid:8080").expect("valid proxy URI"),
            ),
            dns_resolver: Some(SharedDnsResolver::new(FailingDns)),
            ..Default::default()
        },
        provider,
        tls::TlsContext::default(),
    );

    let error = test_client::send_request(
        &client.connector,
        HttpRequest::get("https://origin.test/protected").expect("valid request"),
    )
    .await
    .expect_err("proxy DNS failure must fail the request");
    assert!(
        error.is_io(),
        "HTTPS proxy DNS failure should remain retryable I/O, got {error:?}"
    );
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_proxy_dns_failure_with_rustls_and_hyper_util_legacy_pool() {
    https_proxy_dns_failure_remains_retryable_io(
        &HyperUtilLegacyPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_proxy_dns_failure_with_rustls_and_partitioned_connection_pool() {
    https_proxy_dns_failure_remains_retryable_io(
        &PartitionedConnectionPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_proxy_dns_failure_with_s2n_tls_and_hyper_util_legacy_pool() {
    https_proxy_dns_failure_remains_retryable_io(&HyperUtilLegacyPool, tls::Provider::S2nTls).await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_proxy_dns_failure_with_s2n_tls_and_partitioned_connection_pool() {
    https_proxy_dns_failure_remains_retryable_io(&PartitionedConnectionPool, tls::Provider::S2nTls)
        .await;
}

#[allow(clippy::await_holding_lock)]
async fn with_env_vars<F, Fut, R>(vars: &[(&str, &str)], test: F) -> R
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = R>,
{
    static ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = ENV_MUTEX.lock().expect("environment lock is not poisoned");
    let previous: Vec<_> = vars
        .iter()
        .map(|(name, _)| (*name, std::env::var(name)))
        .collect();

    for (name, value) in vars {
        std::env::set_var(name, value);
    }
    let result = test().await;
    for (name, value) in previous {
        match value {
            Ok(value) => std::env::set_var(name, value),
            Err(_) => std::env::remove_var(name),
        }
    }
    result
}

async fn http_forward_proxy_uses_absolute_form(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::new(|request| {
        assert_eq!("GET", request.method);
        assert_eq!("http://api.example.com/v1/data", request.uri);
        assert_eq!(
            Some(&"api.example.com".to_string()),
            request.headers.get("host")
        );
        Response::builder()
            .status(StatusCode::OK)
            .body("proxied".to_string())
            .expect("valid response")
    })
    .await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        (StatusCode::OK, "proxied".to_string()),
        get(&client, "http://api.example.com/v1/data")
            .await
            .expect("proxy request succeeds")
    );
    assert_eq!(1, proxy.requests().len());
}

#[tokio::test]
async fn test_http_forward_proxy_uses_absolute_form_with_hyper_util_legacy_pool() {
    http_forward_proxy_uses_absolute_form(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_http_forward_proxy_uses_absolute_form_with_partitioned_connection_pool() {
    http_forward_proxy_uses_absolute_form(&PartitionedConnectionPool).await;
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
async fn custom_dns_resolver_is_used_for_proxy_connections(backend: &dyn HttpClientBackend) {
    const PROXY_HOST: &str = "proxy.test";

    let proxy = MockHttpServer::with_response(StatusCode::OK, "proxied through custom DNS").await;
    let resolver = proxy.dns_resolver(PROXY_HOST);
    let proxy_config = ProxyConfig::http(format!("http://{PROXY_HOST}:{}", proxy.addr().port()))
        .expect("valid proxy URI");
    let client = http_client(
        backend,
        BackendConfig {
            dns_resolver: Some(SharedDnsResolver::new(resolver.clone())),
            ..proxy_backend_config(proxy_config)
        },
    );

    assert_eq!(
        (StatusCode::OK, "proxied through custom DNS".to_string()),
        get(&client, "http://origin.test/custom-dns-proxy")
            .await
            .expect("proxy request succeeds")
    );
    assert_eq!(vec![PROXY_HOST.to_string()], resolver.lookups());
    assert_eq!(
        "http://origin.test/custom-dns-proxy",
        proxy.requests()[0].uri
    );
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
#[tokio::test]
async fn test_custom_dns_resolver_is_used_for_proxy_connections_with_hyper_util_legacy_pool() {
    custom_dns_resolver_is_used_for_proxy_connections(&HyperUtilLegacyPool).await;
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
#[tokio::test]
async fn test_custom_dns_resolver_is_used_for_proxy_connections_with_partitioned_connection_pool() {
    custom_dns_resolver_is_used_for_proxy_connections(&PartitionedConnectionPool).await;
}

async fn configured_proxy_authentication_is_applied(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_auth_validation("testuser", "testpass").await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("testuser", "testpass");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        (StatusCode::OK, "authenticated".to_string()),
        get(&client, "http://service.test/protected")
            .await
            .expect("authenticated proxy request succeeds")
    );
    assert_eq!(
        Some(&basic_authorization("testuser", "testpass")),
        proxy.requests()[0].headers.get("proxy-authorization")
    );
}

#[tokio::test]
async fn test_configured_proxy_authentication_with_hyper_util_legacy_pool() {
    configured_proxy_authentication_is_applied(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_configured_proxy_authentication_with_partitioned_connection_pool() {
    configured_proxy_authentication_is_applied(&PartitionedConnectionPool).await;
}

async fn caller_proxy_authorization_is_preserved(backend: &dyn HttpClientBackend) {
    let caller_authorization = basic_authorization("caller", "credentials");
    let expected = caller_authorization.clone();
    let proxy = MockHttpServer::new(move |request| {
        assert_eq!(Some(&expected), request.headers.get("proxy-authorization"));
        Response::builder()
            .status(StatusCode::OK)
            .body("caller authorization".to_string())
            .expect("valid response")
    })
    .await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("configured", "credentials");
    let client = http_client(backend, proxy_backend_config(config));
    let mut request = HttpRequest::get("http://service.test/caller-auth").expect("valid request");
    request
        .headers_mut()
        .insert("proxy-authorization", caller_authorization);

    assert_eq!(
        (StatusCode::OK, "caller authorization".to_string()),
        send_request(&client, request)
            .await
            .expect("caller authorization request succeeds")
    );
}

#[tokio::test]
async fn test_caller_proxy_authorization_is_preserved_with_hyper_util_legacy_pool() {
    caller_proxy_authorization_is_preserved(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_caller_proxy_authorization_is_preserved_with_partitioned_connection_pool() {
    caller_proxy_authorization_is_preserved(&PartitionedConnectionPool).await;
}

async fn proxy_url_authentication_is_applied(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_auth_validation("urluser", "urlpass").await;
    let config =
        ProxyConfig::http(format!("http://urluser:urlpass@{}", proxy.addr())).expect("valid proxy");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        StatusCode::OK,
        get(&client, "http://service.test/url-auth")
            .await
            .expect("URL-authenticated request succeeds")
            .0
    );
    assert_eq!(
        Some(&basic_authorization("urluser", "urlpass")),
        proxy.requests()[0].headers.get("proxy-authorization")
    );
}

#[tokio::test]
async fn test_proxy_url_authentication_with_hyper_util_legacy_pool() {
    proxy_url_authentication_is_applied(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_proxy_url_authentication_with_partitioned_connection_pool() {
    proxy_url_authentication_is_applied(&PartitionedConnectionPool).await;
}

async fn proxy_url_authentication_precedes_configured_authentication(
    backend: &dyn HttpClientBackend,
) {
    let proxy = MockHttpServer::with_auth_validation("urluser", "urlpass").await;
    let config = ProxyConfig::http(format!("http://urluser:urlpass@{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("configured", "credentials");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        StatusCode::OK,
        get(&client, "http://service.test/auth-precedence")
            .await
            .expect("proxy request succeeds")
            .0
    );
    assert_eq!(
        Some(&basic_authorization("urluser", "urlpass")),
        proxy.requests()[0].headers.get("proxy-authorization")
    );
}

#[tokio::test]
async fn test_proxy_url_authentication_precedence_with_hyper_util_legacy_pool() {
    proxy_url_authentication_precedes_configured_authentication(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_proxy_url_authentication_precedence_with_partitioned_connection_pool() {
    proxy_url_authentication_precedes_configured_authentication(&PartitionedConnectionPool).await;
}

async fn environment_proxy_is_used(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "environment proxy").await;
    let proxy_uri = format!("http://{}", proxy.addr());
    with_env_vars(
        &[
            ("HTTP_PROXY", &proxy_uri),
            ("NO_PROXY", "localhost,127.0.0.1"),
        ],
        || async {
            let client = http_client(backend, proxy_backend_config(ProxyConfig::from_env()));
            assert_eq!(
                (StatusCode::OK, "environment proxy".to_string()),
                get(&client, "http://service.test/environment")
                    .await
                    .expect("environment proxy request succeeds")
            );
        },
    )
    .await;
    assert_eq!("http://service.test/environment", proxy.requests()[0].uri);
}

#[tokio::test]
async fn test_environment_proxy_with_hyper_util_legacy_pool() {
    environment_proxy_is_used(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_environment_proxy_with_partitioned_connection_pool() {
    environment_proxy_is_used(&PartitionedConnectionPool).await;
}

async fn no_proxy_bypasses_proxy(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "unexpected proxy").await;
    let origin = MockHttpServer::with_response(StatusCode::OK, "direct").await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .no_proxy("127.0.0.1");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        (StatusCode::OK, "direct".to_string()),
        get(&client, &format!("http://{}/bypass", origin.addr()))
            .await
            .expect("direct request succeeds")
    );
    assert!(proxy.requests().is_empty());
    assert_eq!("/bypass", origin.requests()[0].uri);
}

#[tokio::test]
async fn test_no_proxy_bypass_with_hyper_util_legacy_pool() {
    no_proxy_bypasses_proxy(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_no_proxy_bypass_with_partitioned_connection_pool() {
    no_proxy_bypasses_proxy(&PartitionedConnectionPool).await;
}

async fn disabled_proxy_uses_direct_origin_form(backend: &dyn HttpClientBackend) {
    let origin = MockHttpServer::with_response(StatusCode::OK, "direct").await;
    let client = http_client(backend, proxy_backend_config(ProxyConfig::disabled()));

    assert_eq!(
        (StatusCode::OK, "direct".to_string()),
        get(&client, &format!("http://{}/direct", origin.addr()))
            .await
            .expect("direct request succeeds")
    );
    assert_eq!("/direct", origin.requests()[0].uri);
}

#[tokio::test]
async fn test_disabled_proxy_uses_direct_origin_form_with_hyper_util_legacy_pool() {
    disabled_proxy_uses_direct_origin_form(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_disabled_proxy_uses_direct_origin_form_with_partitioned_connection_pool() {
    disabled_proxy_uses_direct_origin_form(&PartitionedConnectionPool).await;
}

async fn https_only_proxy_bypasses_http(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "unexpected proxy").await;
    let origin = MockHttpServer::with_response(StatusCode::OK, "direct HTTP").await;
    let config =
        ProxyConfig::https(format!("http://{}", proxy.addr())).expect("valid proxy configuration");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        (StatusCode::OK, "direct HTTP".to_string()),
        get(&client, &format!("http://{}/http-only", origin.addr()))
            .await
            .expect("direct HTTP request succeeds")
    );
    assert!(proxy.requests().is_empty());
}

#[tokio::test]
async fn test_https_only_proxy_bypasses_http_with_hyper_util_legacy_pool() {
    https_only_proxy_bypasses_http(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_https_only_proxy_bypasses_http_with_partitioned_connection_pool() {
    https_only_proxy_bypasses_http(&PartitionedConnectionPool).await;
}

async fn all_traffic_proxy_forwards_http(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "all traffic").await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        (StatusCode::OK, "all traffic".to_string()),
        get(&client, "http://service.test/all")
            .await
            .expect("all-traffic proxy request succeeds")
    );
    assert_eq!("http://service.test/all", proxy.requests()[0].uri);
}

#[tokio::test]
async fn test_all_traffic_proxy_forwards_http_with_hyper_util_legacy_pool() {
    all_traffic_proxy_forwards_http(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_all_traffic_proxy_forwards_http_with_partitioned_connection_pool() {
    all_traffic_proxy_forwards_http(&PartitionedConnectionPool).await;
}

async fn unreachable_proxy_fails(backend: &dyn HttpClientBackend) {
    let config = ProxyConfig::http("http://127.0.0.1:1").expect("valid proxy");
    let client = http_client(backend, proxy_backend_config(config));
    assert!(
        get(&client, "http://service.test/unreachable")
            .await
            .is_err(),
        "an unreachable proxy must fail the request"
    );
}

#[tokio::test]
async fn test_unreachable_proxy_fails_with_hyper_util_legacy_pool() {
    unreachable_proxy_fails(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_unreachable_proxy_fails_with_partitioned_connection_pool() {
    unreachable_proxy_fails(&PartitionedConnectionPool).await;
}

async fn incorrect_proxy_authentication_returns_407(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_auth_validation("correct", "password").await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("wrong", "credentials");
    let client = http_client(backend, proxy_backend_config(config));

    assert_eq!(
        StatusCode::PROXY_AUTHENTICATION_REQUIRED,
        get(&client, "http://service.test/denied")
            .await
            .expect("proxy response is returned")
            .0
    );
    assert_eq!(
        Some(&basic_authorization("wrong", "credentials")),
        proxy.requests()[0].headers.get("proxy-authorization")
    );
}

#[tokio::test]
async fn test_incorrect_proxy_authentication_returns_407_with_hyper_util_legacy_pool() {
    incorrect_proxy_authentication_returns_407(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_incorrect_proxy_authentication_returns_407_with_partitioned_connection_pool() {
    incorrect_proxy_authentication_returns_407(&PartitionedConnectionPool).await;
}

async fn disabled_proxy_overrides_environment(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "unexpected proxy").await;
    let origin = MockHttpServer::with_response(StatusCode::OK, "direct").await;
    let proxy_uri = format!("http://{}", proxy.addr());
    with_env_vars(&[("HTTP_PROXY", &proxy_uri)], || async {
        let client = http_client(backend, proxy_backend_config(ProxyConfig::disabled()));
        assert_eq!(
            (StatusCode::OK, "direct".to_string()),
            get(&client, &format!("http://{}/disabled", origin.addr()))
                .await
                .expect("direct request succeeds")
        );
    })
    .await;
    assert!(proxy.requests().is_empty());
    assert_eq!("/disabled", origin.requests()[0].uri);
}

#[tokio::test]
async fn test_disabled_proxy_overrides_environment_with_hyper_util_legacy_pool() {
    disabled_proxy_overrides_environment(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_disabled_proxy_overrides_environment_with_partitioned_connection_pool() {
    disabled_proxy_overrides_environment(&PartitionedConnectionPool).await;
}

async fn idle_proxy_connection_is_evicted(backend: &dyn HttpClientBackend) {
    let proxy = MockHttpServer::with_response(StatusCode::OK, "proxied").await;
    let config = ProxyConfig::http(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = http_client(
        backend,
        BackendConfig {
            pool_idle_timeout: Some(Duration::from_millis(100)),
            ..proxy_backend_config(config)
        },
    );

    assert_eq!(
        StatusCode::OK,
        get(&client, "http://service.test/idle")
            .await
            .expect("proxy request succeeds")
            .0
    );
    assert_eq!(1, proxy.connection_count());

    tokio::time::timeout(Duration::from_secs(2), async {
        while proxy.connection_count() != 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("idle proxy connection should close");
}

#[tokio::test]
async fn test_idle_proxy_connection_is_evicted_with_hyper_util_legacy_pool() {
    idle_proxy_connection_is_evicted(&HyperUtilLegacyPool).await;
}

#[tokio::test]
async fn test_idle_proxy_connection_is_evicted_with_partitioned_connection_pool() {
    idle_proxy_connection_is_evicted(&PartitionedConnectionPool).await;
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn https_connect_uses_authority_form_and_authentication(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let expected = basic_authorization("connectuser", "connectpass");
    let expected_for_handler = expected.clone();
    let proxy = MockHttpServer::new(move |request| {
        assert_eq!("CONNECT", request.method);
        assert_eq!("secure.example.com:443", request.uri);
        assert_eq!(
            Some(&expected_for_handler),
            request.headers.get("proxy-authorization")
        );
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("tunnel rejected".to_string())
            .expect("valid response")
    })
    .await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("connectuser", "connectpass");
    let client = https_client(
        backend,
        proxy_backend_config(config),
        provider,
        tls::TlsContext::default(),
    );

    assert!(
        get(&client, "https://secure.example.com/private")
            .await
            .is_err(),
        "a rejected CONNECT request must fail"
    );
    assert_eq!(1, proxy.requests().len());
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_connect_form_and_auth_with_rustls_and_hyper_util_legacy_pool() {
    https_connect_uses_authority_form_and_authentication(
        &HyperUtilLegacyPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_connect_form_and_auth_with_rustls_and_partitioned_connection_pool() {
    https_connect_uses_authority_form_and_authentication(
        &PartitionedConnectionPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_connect_form_and_auth_with_s2n_tls_and_hyper_util_legacy_pool() {
    https_connect_uses_authority_form_and_authentication(
        &HyperUtilLegacyPool,
        tls::Provider::S2nTls,
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_connect_form_and_auth_with_s2n_tls_and_partitioned_connection_pool() {
    https_connect_uses_authority_form_and_authentication(
        &PartitionedConnectionPool,
        tls::Provider::S2nTls,
    )
    .await;
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn https_connect_without_authentication_is_rejected(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let proxy = MockHttpServer::new(|request| {
        assert_eq!("CONNECT", request.method);
        assert_eq!("secure.example.com:443", request.uri);
        assert!(!request.headers.contains_key("proxy-authorization"));
        Response::builder()
            .status(StatusCode::PROXY_AUTHENTICATION_REQUIRED)
            .body("authentication required".to_string())
            .expect("valid response")
    })
    .await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = https_client(
        backend,
        proxy_backend_config(config),
        provider,
        tls::TlsContext::default(),
    );

    assert!(
        get(&client, "https://secure.example.com/private")
            .await
            .is_err(),
        "a 407 CONNECT response must fail"
    );
    assert_eq!(1, proxy.requests().len());
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_connect_without_auth_with_rustls_and_hyper_util_legacy_pool() {
    https_connect_without_authentication_is_rejected(
        &HyperUtilLegacyPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_connect_without_auth_with_rustls_and_partitioned_connection_pool() {
    https_connect_without_authentication_is_rejected(
        &PartitionedConnectionPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_connect_without_auth_with_s2n_tls_and_hyper_util_legacy_pool() {
    https_connect_without_authentication_is_rejected(&HyperUtilLegacyPool, tls::Provider::S2nTls)
        .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_connect_without_auth_with_s2n_tls_and_partitioned_connection_pool() {
    https_connect_without_authentication_is_rejected(
        &PartitionedConnectionPool,
        tls::Provider::S2nTls,
    )
    .await;
}

/// The partitioned pool reports a proxy's 407 answer to CONNECT as a terminal error, not
/// retryable I/O. The legacy client retries every connect failure, so this contract has no
/// legacy runner.
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn pool_https_connect_407_is_terminal(provider: tls::Provider) {
    let proxy = MockHttpServer::new(|request| {
        assert_eq!("CONNECT", request.method);
        Response::builder()
            .status(StatusCode::PROXY_AUTHENTICATION_REQUIRED)
            .body("authentication required".to_string())
            .expect("valid response")
    })
    .await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = https_client(
        &PartitionedConnectionPool,
        proxy_backend_config(config),
        provider,
        tls::TlsContext::default(),
    );

    let error = test_client::send_request(
        &client.connector,
        HttpRequest::get("https://secure.example.com/private").expect("valid request"),
    )
    .await
    .expect_err("a 407 CONNECT response must fail");
    assert!(
        error.is_other(),
        "expected ConnectorError::other, got {error:?}"
    );
    assert!(!error.is_io(), "a proxy's 407 must not be retryable I/O");
    assert_eq!(1, proxy.requests().len());
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_connect_407_is_terminal_with_rustls_and_partitioned_connection_pool() {
    pool_https_connect_407_is_terminal(tls::Provider::rustls(
        tls::rustls_provider::CryptoMode::Ring,
    ))
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_connect_407_is_terminal_with_s2n_tls_and_partitioned_connection_pool() {
    pool_https_connect_407_is_terminal(tls::Provider::S2nTls).await;
}

/// An origin that closes the tunneled connection during the TLS handshake fails the pool
/// request with retryable I/O.
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn pool_tunneled_handshake_eof_is_io(provider: tls::Provider) {
    let origin = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("origin should bind");
    let origin_addr = origin.local_addr().expect("origin has an address");
    let closing_origin = tokio::spawn(async move {
        loop {
            let (stream, _) = origin.accept().await.expect("origin accepts");
            drop(stream);
        }
    });
    let proxy = MockConnectProxy::relay_to(origin_addr, None).await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr())).expect("valid proxy");
    let client = https_client(
        &PartitionedConnectionPool,
        proxy_backend_config(config),
        provider,
        test_tls::SERVER_IDENTITY.client_context(),
    );

    let error = test_client::send_request(
        &client.connector,
        HttpRequest::get(format!("https://localhost:{}/", origin_addr.port()))
            .expect("valid request"),
    )
    .await
    .expect_err("an origin that closes during the handshake fails the request");
    closing_origin.abort();
    assert!(error.is_io(), "expected ConnectorError::io, got {error:?}");
    assert_eq!(1, proxy.requests().len());
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_tunneled_handshake_eof_is_io_with_rustls_and_partitioned_connection_pool() {
    pool_tunneled_handshake_eof_is_io(tls::Provider::rustls(
        tls::rustls_provider::CryptoMode::Ring,
    ))
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_tunneled_handshake_eof_is_io_with_s2n_tls_and_partitioned_connection_pool() {
    pool_tunneled_handshake_eof_is_io(tls::Provider::S2nTls).await;
}

/// An `https://` proxy that closes the connection during the TLS handshake to the proxy fails the
/// pool request with retryable I/O, as a handshake failure to the origin does.
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn pool_https_proxy_handshake_eof_is_io(provider: tls::Provider) {
    let proxy = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("proxy should bind");
    let proxy_addr = proxy.local_addr().expect("proxy has an address");
    let closing_proxy = tokio::spawn(async move {
        loop {
            let (stream, _) = proxy.accept().await.expect("proxy accepts");
            drop(stream);
        }
    });
    let config = ProxyConfig::all(format!("https://{proxy_addr}")).expect("valid proxy");
    let client = https_client(
        &PartitionedConnectionPool,
        proxy_backend_config(config),
        provider,
        tls::TlsContext::default(),
    );

    let error = test_client::send_request(
        &client.connector,
        HttpRequest::get("https://secure.example.com/private").expect("valid request"),
    )
    .await
    .expect_err("a proxy that closes during the handshake fails the request");
    closing_proxy.abort();
    assert!(error.is_io(), "expected ConnectorError::io, got {error:?}");
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_https_proxy_handshake_eof_is_io_with_rustls_and_partitioned_connection_pool() {
    pool_https_proxy_handshake_eof_is_io(tls::Provider::rustls(
        tls::rustls_provider::CryptoMode::Ring,
    ))
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_https_proxy_handshake_eof_is_io_with_s2n_tls_and_partitioned_connection_pool() {
    pool_https_proxy_handshake_eof_is_io(tls::Provider::S2nTls).await;
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
async fn tunneled_https_request_uses_origin_form(
    backend: &dyn HttpsClientBackend,
    provider: tls::Provider,
) {
    let origin = MockTlsOrigin::new("tunneled response").await;
    let authorization = basic_authorization("connectuser", "connectpass");
    let proxy = MockConnectProxy::relay_to(origin.addr(), Some(authorization.clone())).await;
    let config = ProxyConfig::all(format!("http://{}", proxy.addr()))
        .expect("valid proxy")
        .with_basic_auth("connectuser", "connectpass");
    let client = https_client(
        backend,
        proxy_backend_config(config),
        provider,
        test_tls::SERVER_IDENTITY.client_context(),
    );
    let target = format!("https://localhost:{}/inside?value=1", origin.addr().port());

    assert_eq!(
        (StatusCode::OK, "tunneled response".to_string()),
        get(&client, &target)
            .await
            .expect("tunneled HTTPS request succeeds")
    );

    let proxy_requests = proxy.requests();
    assert_eq!(1, proxy_requests.len());
    assert_eq!("CONNECT", proxy_requests[0].method);
    assert_eq!(
        format!("localhost:{}", origin.addr().port()),
        proxy_requests[0].uri
    );
    assert_eq!(
        Some(&authorization),
        proxy_requests[0].headers.get("proxy-authorization")
    );

    let origin_requests = origin.requests();
    assert_eq!(1, origin_requests.len());
    assert_eq!("GET", origin_requests[0].method);
    assert_eq!("/inside?value=1", origin_requests[0].uri);
    assert!(
        !origin_requests[0]
            .headers
            .contains_key("proxy-authorization"),
        "proxy credentials must not cross the CONNECT tunnel"
    );
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_tunneled_https_origin_form_with_rustls_and_hyper_util_legacy_pool() {
    tunneled_https_request_uses_origin_form(
        &HyperUtilLegacyPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "rustls-ring")]
#[tokio::test]
async fn test_tunneled_https_origin_form_with_rustls_and_partitioned_connection_pool() {
    tunneled_https_request_uses_origin_form(
        &PartitionedConnectionPool,
        tls::Provider::rustls(tls::rustls_provider::CryptoMode::Ring),
    )
    .await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_tunneled_https_origin_form_with_s2n_tls_and_hyper_util_legacy_pool() {
    tunneled_https_request_uses_origin_form(&HyperUtilLegacyPool, tls::Provider::S2nTls).await;
}

#[cfg(feature = "s2n-tls")]
#[tokio::test]
async fn test_tunneled_https_origin_form_with_s2n_tls_and_partitioned_connection_pool() {
    tunneled_https_request_uses_origin_form(&PartitionedConnectionPool, tls::Provider::S2nTls)
        .await;
}
