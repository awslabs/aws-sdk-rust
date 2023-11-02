/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP clients and connectors
//!
//! # What is a connector?
//!
//! When we talk about connectors, we are referring to the [`HttpConnector`] trait, and implementations of
//! that trait. This trait simply takes a HTTP request, and returns a future with the response for that
//! request.
//!
//! This is slightly different from what a connector is in other libraries such as
//! [`hyper`]. In hyper 0.x, the connector is a [`tower`] `Service` that takes a `Uri` and returns
//! a future with something that implements `AsyncRead + AsyncWrite`.
//!
//! The [`HttpConnector`] is designed to be a layer on top of
//! whole HTTP libraries, such as hyper, which allows Smithy clients to be agnostic to the underlying HTTP
//! transport layer. This also makes it easy to write tests with a fake HTTP connector, and several
//! such test connector implementations are available in [`aws-smithy-runtime`]
//! with the `test-util` feature enabled.
//!
//! # Responsibilities of a connector
//!
//! A connector primarily makes HTTP requests, but is also the place where connect and read timeouts are
//! implemented. The `HyperConnector` in [`aws-smithy-runtime`] is an example where timeouts are implemented
//! as part of the connector.
//!
//! Connectors are also responsible for DNS lookup, TLS, connection reuse, pooling, and eviction.
//! The Smithy clients have no knowledge of such concepts.
//!
//! # The [`HttpClient`] trait
//!
//! Connectors allow us to make requests, but we need a layer on top of connectors so that we can handle
//! varying connector settings. For example, say we configure some default HTTP connect/read timeouts on
//! Client, and then configure some override connect/read timeouts for a specific operation. These timeouts
//! ultimately are part of the connector, so the same connector can't be reused for the two different sets
//! of timeouts. Thus, the [`HttpClient`] implementation is responsible for managing multiple connectors
//! with varying config. Some example configs that can impact which connector is used:
//!
//! - HTTP protocol versions
//! - TLS settings
//! - Timeouts
//!
//! Some of these aren't implemented yet, but they will appear in the [`HttpConnectorSettings`] struct
//! once they are.
//!
//! [`hyper`]: https://crates.io/crates/hyper
//! [`tower`]: https://crates.io/crates/tower
//! [`aws-smithy-runtime`]: https://crates.io/crates/aws-smithy-runtime

use crate::client::orchestrator::{HttpRequest, HttpResponse};
use crate::client::runtime_components::RuntimeComponents;
use crate::impl_shared_conversions;
use aws_smithy_async::future::now_or_later::NowOrLater;
use aws_smithy_http::result::ConnectorError;
use pin_project_lite::pin_project;
use std::fmt;
use std::future::Future as StdFuture;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

type BoxFuture = aws_smithy_async::future::BoxFuture<HttpResponse, ConnectorError>;

pin_project! {
    /// Future for [`HttpConnector::call`].
    pub struct HttpConnectorFuture {
        #[pin]
        inner: NowOrLater<Result<HttpResponse, ConnectorError>, BoxFuture>,
    }
}

impl HttpConnectorFuture {
    /// Create a new `HttpConnectorFuture` with the given future.
    pub fn new<F>(future: F) -> Self
    where
        F: StdFuture<Output = Result<HttpResponse, ConnectorError>> + Send + 'static,
    {
        Self {
            inner: NowOrLater::new(Box::pin(future)),
        }
    }

    /// Create a new `HttpConnectorFuture` with the given boxed future.
    ///
    /// Use this if you already have a boxed future to avoid double boxing it.
    pub fn new_boxed(
        future: Pin<Box<dyn StdFuture<Output = Result<HttpResponse, ConnectorError>> + Send>>,
    ) -> Self {
        Self {
            inner: NowOrLater::new(future),
        }
    }

    /// Create a `HttpConnectorFuture` that is immediately ready with the given result.
    pub fn ready(result: Result<HttpResponse, ConnectorError>) -> Self {
        Self {
            inner: NowOrLater::ready(result),
        }
    }
}

impl StdFuture for HttpConnectorFuture {
    type Output = Result<HttpResponse, ConnectorError>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.inner.poll(cx)
    }
}

/// Trait with a `call` function that asynchronously converts a request into a response.
///
/// Ordinarily, a connector would use an underlying HTTP library such as [hyper](https://crates.io/crates/hyper),
/// and any associated HTTPS implementation alongside it to service requests.
///
/// However, it can also be useful to create fake/mock connectors implementing this trait
/// for testing.
pub trait HttpConnector: Send + Sync + fmt::Debug {
    /// Asynchronously converts a request into a response.
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture;
}

/// A shared [`HttpConnector`] implementation.
#[derive(Clone, Debug)]
pub struct SharedHttpConnector(Arc<dyn HttpConnector>);

impl SharedHttpConnector {
    /// Returns a new [`SharedHttpConnector`].
    pub fn new(connection: impl HttpConnector + 'static) -> Self {
        Self(Arc::new(connection))
    }
}

impl HttpConnector for SharedHttpConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        (*self.0).call(request)
    }
}

impl_shared_conversions!(convert SharedHttpConnector from HttpConnector using SharedHttpConnector::new);

/// Returns a [`SharedHttpClient`] that calls the given `connector` function to select a HTTP connector.
pub fn http_client_fn<F>(connector: F) -> SharedHttpClient
where
    F: Fn(&HttpConnectorSettings, &RuntimeComponents) -> SharedHttpConnector
        + Send
        + Sync
        + 'static,
{
    struct ConnectorFn<T>(T);
    impl<T> fmt::Debug for ConnectorFn<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("ConnectorFn")
        }
    }
    impl<T> HttpClient for ConnectorFn<T>
    where
        T: (Fn(&HttpConnectorSettings, &RuntimeComponents) -> SharedHttpConnector) + Send + Sync,
    {
        fn http_connector(
            &self,
            settings: &HttpConnectorSettings,
            components: &RuntimeComponents,
        ) -> SharedHttpConnector {
            (self.0)(settings, components)
        }
    }

    SharedHttpClient::new(ConnectorFn(connector))
}

/// HTTP client abstraction.
///
/// A HTTP client implementation must apply connect/read timeout settings,
/// and must maintain a connection pool.
pub trait HttpClient: Send + Sync + fmt::Debug {
    /// Returns a HTTP connector based on the requested connector settings.
    ///
    /// The settings include connector timeouts, which should be incorporated
    /// into the connector. The `HttpClient` is responsible for caching
    /// the connector across requests.
    ///
    /// In the future, the settings may have additional parameters added,
    /// such as HTTP version, or TLS certificate paths.
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        components: &RuntimeComponents,
    ) -> SharedHttpConnector;
}

/// Shared HTTP client for use across multiple clients and requests.
#[derive(Clone, Debug)]
pub struct SharedHttpClient {
    selector: Arc<dyn HttpClient>,
}

impl SharedHttpClient {
    /// Creates a new `SharedHttpClient`
    pub fn new(selector: impl HttpClient + 'static) -> Self {
        Self {
            selector: Arc::new(selector),
        }
    }
}

impl HttpClient for SharedHttpClient {
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        self.selector.http_connector(settings, components)
    }
}

impl_shared_conversions!(convert SharedHttpClient from HttpClient using SharedHttpClient::new);

/// Builder for [`HttpConnectorSettings`].
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct HttpConnectorSettingsBuilder {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl HttpConnectorSettingsBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = Some(connect_timeout);
        self
    }

    /// Sets the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn set_connect_timeout(&mut self, connect_timeout: Option<Duration>) -> &mut Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(mut self, read_timeout: Duration) -> Self {
        self.read_timeout = Some(read_timeout);
        self
    }

    /// Sets the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn set_read_timeout(&mut self, read_timeout: Option<Duration>) -> &mut Self {
        self.read_timeout = read_timeout;
        self
    }

    /// Builds the [`HttpConnectorSettings`].
    pub fn build(self) -> HttpConnectorSettings {
        HttpConnectorSettings {
            connect_timeout: self.connect_timeout,
            read_timeout: self.read_timeout,
        }
    }
}

/// Settings for HTTP Connectors
#[non_exhaustive]
#[derive(Clone, Default, Debug)]
pub struct HttpConnectorSettings {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl HttpConnectorSettings {
    /// Returns a builder for `HttpConnectorSettings`.
    pub fn builder() -> HttpConnectorSettingsBuilder {
        Default::default()
    }

    /// Returns the connect timeout that should be used.
    ///
    /// The connect timeout is a limit on the amount of time it takes to initiate a socket connection.
    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// Returns the read timeout that should be used.
    ///
    /// The read timeout is the limit on the amount of time it takes to read the first byte of a response
    /// from the time the request is initiated.
    pub fn read_timeout(&self) -> Option<Duration> {
        self.read_timeout
    }
}
