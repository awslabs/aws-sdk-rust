/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP proxy and tunneled-origin support for integration tests.

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
use aws_smithy_runtime_api::client::dns::{DnsFuture, ResolveDns, ResolveDnsError};
use base64::Engine;
use http_1x::{Request, Response, StatusCode};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
use std::net::IpAddr;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use http_body_util::Full;
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use hyper::body::Bytes;
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use tokio::io::copy_bidirectional;
#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
use tokio::net::TcpStream;

/// One request observed by a test server.
#[derive(Clone, Debug)]
pub(crate) struct RecordedRequest {
    pub(crate) method: String,
    pub(crate) uri: String,
    pub(crate) headers: HashMap<String, String>,
}

impl RecordedRequest {
    fn from_request(request: &Request<Incoming>) -> Self {
        Self {
            method: request.method().to_string(),
            uri: request.uri().to_string(),
            headers: request
                .headers()
                .iter()
                .map(|(name, value)| {
                    (
                        name.to_string(),
                        value.to_str().unwrap_or_default().to_string(),
                    )
                })
                .collect(),
        }
    }
}

/// DNS resolver that maps one expected hostname to a test server.
#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
#[derive(Clone, Debug)]
pub(crate) struct RecordingDnsResolver {
    hostname: String,
    address: IpAddr,
    lookups: Arc<Mutex<Vec<String>>>,
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
impl RecordingDnsResolver {
    fn new(hostname: impl Into<String>, address: IpAddr) -> Self {
        Self {
            hostname: hostname.into(),
            address,
            lookups: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn lookups(&self) -> Vec<String> {
        self.lookups
            .lock()
            .expect("DNS lookup log is not poisoned")
            .clone()
    }
}

#[cfg(any(
    feature = "rustls-aws-lc",
    feature = "rustls-aws-lc-fips",
    feature = "rustls-ring",
    feature = "s2n-tls"
))]
impl ResolveDns for RecordingDnsResolver {
    fn resolve_dns<'a>(&'a self, name: &'a str) -> DnsFuture<'a> {
        self.lookups
            .lock()
            .expect("DNS lookup log is not poisoned")
            .push(name.to_string());
        if name == self.hostname {
            DnsFuture::ready(Ok(vec![self.address]))
        } else {
            DnsFuture::ready(Err(ResolveDnsError::new(std::io::Error::other(format!(
                "unexpected DNS lookup for {name}"
            )))))
        }
    }
}

/// HTTP/1 server used as either a forward proxy or a direct origin.
#[derive(Debug)]
pub(crate) struct MockHttpServer {
    connections: Arc<()>,
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl MockHttpServer {
    pub(crate) async fn new<F>(handler: F) -> Self
    where
        F: Fn(RecordedRequest) -> Response<String> + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let addr = listener.local_addr().expect("listener has an address");
        let (shutdown, mut shutdown_rx) = oneshot::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = requests.clone();
        let connections = Arc::new(());
        let server_connections = connections.clone();
        let handler = Arc::new(handler);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else {
                            break;
                        };
                        let handler = handler.clone();
                        let requests = server_requests.clone();
                        let connection = server_connections.clone();
                        tokio::spawn(async move {
                            let _connection = connection;
                            let service = service_fn(move |request: Request<Incoming>| {
                                let handler = handler.clone();
                                let requests = requests.clone();
                                async move {
                                    let request = RecordedRequest::from_request(&request);
                                    requests
                                        .lock()
                                        .expect("request log is not poisoned")
                                        .push(request.clone());
                                    Ok::<_, Infallible>(handler(request))
                                }
                            });
                            if let Err(error) = hyper::server::conn::http1::Builder::new()
                                .serve_connection(TokioIo::new(stream), service)
                                .await
                            {
                                tracing::debug!(%error, "test HTTP server connection ended");
                            }
                        });
                    }
                    _ = &mut shutdown_rx => break,
                }
            }
        });

        Self {
            connections,
            addr,
            shutdown: Some(shutdown),
            requests,
        }
    }

    pub(crate) async fn with_response(status: StatusCode, body: &str) -> Self {
        let body = body.to_string();
        Self::new(move |_| {
            Response::builder()
                .status(status)
                .body(body.clone())
                .expect("valid response")
        })
        .await
    }

    pub(crate) async fn with_auth_validation(user: &str, password: &str) -> Self {
        let expected = basic_authorization(user, password);
        Self::new(move |request| {
            if request.headers.get("proxy-authorization") == Some(&expected) {
                Response::builder()
                    .status(StatusCode::OK)
                    .body("authenticated".to_string())
                    .expect("valid response")
            } else {
                Response::builder()
                    .status(StatusCode::PROXY_AUTHENTICATION_REQUIRED)
                    .header("proxy-authenticate", "Basic realm=\"proxy\"")
                    .body("authentication required".to_string())
                    .expect("valid response")
            }
        })
        .await
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    #[cfg(any(
        feature = "rustls-aws-lc",
        feature = "rustls-aws-lc-fips",
        feature = "rustls-ring",
        feature = "s2n-tls"
    ))]
    pub(crate) fn dns_resolver(&self, hostname: impl Into<String>) -> RecordingDnsResolver {
        RecordingDnsResolver::new(hostname, self.addr.ip())
    }

    pub(crate) fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("request log is not poisoned")
            .clone()
    }

    pub(crate) fn connection_count(&self) -> usize {
        // The server and this handle each retain one reference.
        Arc::strong_count(&self.connections)
            .checked_sub(2)
            .expect("server references remain")
    }
}

impl Drop for MockHttpServer {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
#[derive(Debug)]
pub(crate) struct MockConnectProxy {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
impl MockConnectProxy {
    /// Starts a proxy that accepts CONNECT and relays bytes to one fixed origin.
    pub(crate) async fn relay_to(
        origin: SocketAddr,
        expected_authorization: Option<String>,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("CONNECT proxy should bind");
        let addr = listener.local_addr().expect("listener has an address");
        let (shutdown, mut shutdown_rx) = oneshot::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = requests.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else {
                            break;
                        };
                        let requests = server_requests.clone();
                        let expected_authorization = expected_authorization.clone();
                        tokio::spawn(async move {
                            let service = service_fn(move |mut request: Request<Incoming>| {
                                let requests = requests.clone();
                                let expected_authorization = expected_authorization.clone();
                                async move {
                                    let recorded = RecordedRequest::from_request(&request);
                                    requests
                                        .lock()
                                        .expect("request log is not poisoned")
                                        .push(recorded.clone());

                                    if recorded.method != "CONNECT" {
                                        return Ok::<_, Infallible>(empty_response(
                                            StatusCode::METHOD_NOT_ALLOWED,
                                        ));
                                    }
                                    if expected_authorization.as_ref()
                                        != recorded.headers.get("proxy-authorization")
                                    {
                                        return Ok(empty_response(
                                            StatusCode::PROXY_AUTHENTICATION_REQUIRED,
                                        ));
                                    }

                                    let upgraded = hyper::upgrade::on(&mut request);
                                    tokio::spawn(async move {
                                        let Ok(upgraded) = upgraded.await else {
                                            return;
                                        };
                                        let Ok(mut upstream) = TcpStream::connect(origin).await else {
                                            return;
                                        };
                                        let mut downstream = TokioIo::new(upgraded);
                                        let _ = copy_bidirectional(&mut downstream, &mut upstream).await;
                                    });

                                    Ok(empty_response(StatusCode::OK))
                                }
                            });
                            let connection = hyper::server::conn::http1::Builder::new()
                                .serve_connection(TokioIo::new(stream), service)
                                .with_upgrades();
                            if let Err(error) = connection.await {
                                tracing::debug!(%error, "CONNECT proxy connection ended");
                            }
                        });
                    }
                    _ = &mut shutdown_rx => break,
                }
            }
        });

        Self {
            addr,
            shutdown: Some(shutdown),
            requests,
        }
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("request log is not poisoned")
            .clone()
    }
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
impl Drop for MockConnectProxy {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
#[derive(Debug)]
pub(crate) struct MockTlsOrigin {
    addr: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
impl MockTlsOrigin {
    pub(crate) async fn new(body: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("TLS origin should bind");
        let addr = listener.local_addr().expect("listener has an address");
        let acceptor = super::tls::SERVER_IDENTITY
            .acceptor(&[b"http/1.1"])
            .expect("test TLS configuration should load");
        let (shutdown, mut shutdown_rx) = oneshot::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_requests = requests.clone();
        let body = body.to_string();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else {
                            break;
                        };
                        let acceptor = acceptor.clone();
                        let requests = server_requests.clone();
                        let body = body.clone();
                        tokio::spawn(async move {
                            let Ok(stream) = acceptor.accept(stream).await else {
                                return;
                            };
                            let service = service_fn(move |request: Request<Incoming>| {
                                let requests = requests.clone();
                                let body = body.clone();
                                async move {
                                    requests
                                        .lock()
                                        .expect("request log is not poisoned")
                                        .push(RecordedRequest::from_request(&request));
                                    Ok::<_, Infallible>(Response::new(Full::new(Bytes::from(body))))
                                }
                            });
                            if let Err(error) = hyper::server::conn::http1::Builder::new()
                                .serve_connection(TokioIo::new(stream), service)
                                .await
                            {
                                tracing::debug!(%error, "TLS origin connection ended");
                            }
                        });
                    }
                    _ = &mut shutdown_rx => break,
                }
            }
        });

        Self {
            addr,
            shutdown: Some(shutdown),
            requests,
        }
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub(crate) fn requests(&self) -> Vec<RecordedRequest> {
        self.requests
            .lock()
            .expect("request log is not poisoned")
            .clone()
    }
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
impl Drop for MockTlsOrigin {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

pub(crate) fn basic_authorization(user: &str, password: &str) -> String {
    format!(
        "Basic {}",
        base64::prelude::BASE64_STANDARD.encode(format!("{user}:{password}"))
    )
}

#[cfg(any(feature = "rustls-ring", feature = "s2n-tls"))]
fn empty_response(status: StatusCode) -> Response<Full<Bytes>> {
    Response::builder()
        .status(status)
        .body(Full::default())
        .expect("valid response")
}
