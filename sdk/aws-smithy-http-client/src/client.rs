/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

mod timeout;
/// TLS connector(s)
pub mod tls;

use crate::cfg::cfg_tls;
use crate::tls::TlsContext;
use aws_smithy_async::future::timeout::TimedOutError;
use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep, SharedAsyncSleep};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::connection::CaptureSmithyConnection;
use aws_smithy_runtime_api::client::connection::ConnectionMetadata;
use aws_smithy_runtime_api::client::connector_metadata::ConnectorMetadata;
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpClient,
    SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::retry::ErrorKind;
use client::connect::Connection;
use h2::Reason;
use http_1x::{Extensions, Uri};
use hyper::rt::{Read, Write};
use hyper_util::client::legacy as client;
use hyper_util::client::legacy::connect::dns::GaiResolver;
use hyper_util::client::legacy::connect::{
    capture_connection, CaptureConnection, Connect, HttpConnector as HyperHttpConnector, HttpInfo,
};
use hyper_util::rt::TokioExecutor;
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::RwLock;
use std::time::Duration;

/// Given `HttpConnectorSettings` and an `SharedAsyncSleep`, create a `SharedHttpConnector` from defaults depending on what cargo features are activated.
pub fn default_connector(
    settings: &HttpConnectorSettings,
    sleep: Option<SharedAsyncSleep>,
) -> Option<SharedHttpConnector> {
    #[cfg(feature = "rustls-aws-lc")]
    {
        tracing::trace!(settings = ?settings, sleep = ?sleep, "creating a new default connector");
        let mut conn_builder = Connector::builder().connector_settings(settings.clone());

        if let Some(sleep) = sleep {
            conn_builder = conn_builder.sleep_impl(sleep);
        }

        let conn = conn_builder
            .tls_provider(tls::Provider::Rustls(
                tls::rustls_provider::CryptoMode::AwsLc,
            ))
            .build();
        Some(SharedHttpConnector::new(conn))
    }
    #[cfg(not(feature = "rustls-aws-lc"))]
    {
        tracing::trace!(settings = ?settings, sleep = ?sleep, "no default connector available");
        None
    }
}

/// [`HttpConnector`] used to make HTTP requests.
///
/// This connector also implements socket connect and read timeouts.
///
/// This shouldn't be used directly in most cases.
/// See the docs on [`Builder`] for examples of how to customize the HTTP client.
#[derive(Debug)]
pub struct Connector {
    adapter: Box<dyn HttpConnector>,
}

impl Connector {
    /// Builder for an HTTP connector.
    pub fn builder() -> ConnectorBuilder {
        ConnectorBuilder {
            enable_tcp_nodelay: true,
            ..Default::default()
        }
    }
}

impl HttpConnector for Connector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        self.adapter.call(request)
    }
}

/// Builder for [`Connector`].
#[derive(Default, Debug)]
pub struct ConnectorBuilder<Tls = TlsUnset> {
    connector_settings: Option<HttpConnectorSettings>,
    sleep_impl: Option<SharedAsyncSleep>,
    client_builder: Option<hyper_util::client::legacy::Builder>,
    enable_tcp_nodelay: bool,
    interface: Option<String>,
    #[allow(unused)]
    tls: Tls,
}

/// Initial builder state, `TlsProvider` choice required
#[derive(Default)]
#[non_exhaustive]
pub struct TlsUnset {}

/// TLS implementation selected
pub struct TlsProviderSelected {
    #[allow(unused)]
    provider: tls::Provider,
    #[allow(unused)]
    context: TlsContext,
}

impl ConnectorBuilder<TlsUnset> {
    /// Set the TLS implementation to use for this connector
    pub fn tls_provider(self, provider: tls::Provider) -> ConnectorBuilder<TlsProviderSelected> {
        ConnectorBuilder {
            connector_settings: self.connector_settings,
            sleep_impl: self.sleep_impl,
            client_builder: self.client_builder,
            enable_tcp_nodelay: self.enable_tcp_nodelay,
            interface: self.interface,
            tls: TlsProviderSelected {
                provider,
                context: TlsContext::default(),
            },
        }
    }

    /// Build an HTTP connector sans TLS
    #[doc(hidden)]
    pub fn build_http(self) -> Connector {
        let base = self.base_connector();
        self.wrap_connector(base)
    }
}

impl<Any> ConnectorBuilder<Any> {
    /// Create a [`Connector`] from this builder and a given connector.
    pub(crate) fn wrap_connector<C>(self, tcp_connector: C) -> Connector
    where
        C: Send + Sync + 'static,
        C: Clone,
        C: tower::Service<Uri>,
        C::Response: Read + Write + Connection + Send + Sync + Unpin,
        C: Connect,
        C::Future: Unpin + Send + 'static,
        C::Error: Into<BoxError>,
    {
        let client_builder =
            self.client_builder
                .unwrap_or(hyper_util::client::legacy::Builder::new(
                    TokioExecutor::new(),
                ));
        let sleep_impl = self.sleep_impl.or_else(default_async_sleep);
        let (connect_timeout, read_timeout) = self
            .connector_settings
            .map(|c| (c.connect_timeout(), c.read_timeout()))
            .unwrap_or((None, None));

        let connector = match connect_timeout {
            Some(duration) => timeout::ConnectTimeout::new(
                tcp_connector,
                sleep_impl
                    .clone()
                    .expect("a sleep impl must be provided in order to have a connect timeout"),
                duration,
            ),
            None => timeout::ConnectTimeout::no_timeout(tcp_connector),
        };
        let base = client_builder.build(connector);
        let read_timeout = match read_timeout {
            Some(duration) => timeout::HttpReadTimeout::new(
                base,
                sleep_impl.expect("a sleep impl must be provided in order to have a read timeout"),
                duration,
            ),
            None => timeout::HttpReadTimeout::no_timeout(base),
        };
        Connector {
            adapter: Box::new(Adapter {
                client: read_timeout,
            }),
        }
    }

    /// Get the base TCP connector by mapping our config to the underlying `HttpConnector` from hyper
    /// (which is a base TCP connector with no TLS or any wrapping)
    fn base_connector(&self) -> HyperHttpConnector {
        self.base_connector_with_resolver(GaiResolver::new())
    }

    /// Get the base TCP connector by mapping our config to the underlying `HttpConnector` from hyper
    /// using the given resolver `R`
    fn base_connector_with_resolver<R>(&self, resolver: R) -> HyperHttpConnector<R> {
        let mut conn = HyperHttpConnector::new_with_resolver(resolver);
        conn.set_nodelay(self.enable_tcp_nodelay);
        #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
        if let Some(interface) = &self.interface {
            conn.set_interface(interface);
        }
        conn
    }

    /// Set the async sleep implementation used for timeouts
    ///
    /// Calling this is only necessary for testing or to use something other than
    /// [`default_async_sleep`].
    pub fn sleep_impl(mut self, sleep_impl: impl AsyncSleep + 'static) -> Self {
        self.sleep_impl = Some(sleep_impl.into_shared());
        self
    }

    /// Set the async sleep implementation used for timeouts
    ///
    /// Calling this is only necessary for testing or to use something other than
    /// [`default_async_sleep`].
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<SharedAsyncSleep>) -> &mut Self {
        self.sleep_impl = sleep_impl;
        self
    }

    /// Configure the HTTP settings for the `HyperAdapter`
    pub fn connector_settings(mut self, connector_settings: HttpConnectorSettings) -> Self {
        self.connector_settings = Some(connector_settings);
        self
    }

    /// Configure the HTTP settings for the `HyperAdapter`
    pub fn set_connector_settings(
        &mut self,
        connector_settings: Option<HttpConnectorSettings>,
    ) -> &mut Self {
        self.connector_settings = connector_settings;
        self
    }

    /// Configure `SO_NODELAY` for all sockets to the supplied value `nodelay`
    pub fn enable_tcp_nodelay(mut self, nodelay: bool) -> Self {
        self.enable_tcp_nodelay = nodelay;
        self
    }

    /// Configure `SO_NODELAY` for all sockets to the supplied value `nodelay`
    pub fn set_enable_tcp_nodelay(&mut self, nodelay: bool) -> &mut Self {
        self.enable_tcp_nodelay = nodelay;
        self
    }

    /// Sets the value for the `SO_BINDTODEVICE` option on this socket.
    ///
    /// If a socket is bound to an interface, only packets received from that particular
    /// interface are processed by the socket. Note that this only works for some socket
    /// types (e.g. `AF_INET` sockets).
    ///
    /// On Linux it can be used to specify a [VRF], but the binary needs to either have
    /// `CAP_NET_RAW` capability set or be run as root.
    ///
    /// This function is only available on Android, Fuchsia, and Linux.
    ///
    /// [VRF]: https://www.kernel.org/doc/Documentation/networking/vrf.txt
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    pub fn set_interface<S: Into<String>>(&mut self, interface: S) -> &mut Self {
        self.interface = Some(interface.into());
        self
    }

    /// Override the Hyper client [`Builder`](hyper_util::client::legacy::Builder) used to construct this client.
    ///
    /// This enables changing settings like forcing HTTP2 and modifying other default client behavior.
    pub(crate) fn hyper_builder(
        mut self,
        hyper_builder: hyper_util::client::legacy::Builder,
    ) -> Self {
        self.set_hyper_builder(Some(hyper_builder));
        self
    }

    /// Override the Hyper client [`Builder`](hyper_util::client::legacy::Builder) used to construct this client.
    ///
    /// This enables changing settings like forcing HTTP2 and modifying other default client behavior.
    pub(crate) fn set_hyper_builder(
        &mut self,
        hyper_builder: Option<hyper_util::client::legacy::Builder>,
    ) -> &mut Self {
        self.client_builder = hyper_builder;
        self
    }
}

/// Adapter to use a Hyper 1.0-based Client as an `HttpConnector`
///
/// This adapter also enables TCP `CONNECT` and HTTP `READ` timeouts via [`Connector::builder`].
struct Adapter<C> {
    client: timeout::HttpReadTimeout<
        hyper_util::client::legacy::Client<timeout::ConnectTimeout<C>, SdkBody>,
    >,
}

impl<C> fmt::Debug for Adapter<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Adapter")
            .field("client", &"** hyper client **")
            .finish()
    }
}

/// Extract a smithy connection from a hyper CaptureConnection
fn extract_smithy_connection(capture_conn: &CaptureConnection) -> Option<ConnectionMetadata> {
    let capture_conn = capture_conn.clone();
    if let Some(conn) = capture_conn.clone().connection_metadata().as_ref() {
        let mut extensions = Extensions::new();
        conn.get_extras(&mut extensions);
        let http_info = extensions.get::<HttpInfo>();
        let mut builder = ConnectionMetadata::builder()
            .proxied(conn.is_proxied())
            .poison_fn(move || match capture_conn.connection_metadata().as_ref() {
                Some(conn) => conn.poison(),
                None => tracing::trace!("no connection existed to poison"),
            });

        builder
            .set_local_addr(http_info.map(|info| info.local_addr()))
            .set_remote_addr(http_info.map(|info| info.remote_addr()));

        let smithy_connection = builder.build();

        Some(smithy_connection)
    } else {
        None
    }
}

impl<C> HttpConnector for Adapter<C>
where
    C: Clone + Send + Sync + 'static,
    C: tower::Service<Uri>,
    C::Response: Connection + Read + Write + Unpin + 'static,
    timeout::ConnectTimeout<C>: Connect,
    C::Future: Unpin + Send + 'static,
    C::Error: Into<BoxError>,
{
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let mut request = match request.try_into_http1x() {
            Ok(request) => request,
            Err(err) => {
                return HttpConnectorFuture::ready(Err(ConnectorError::user(err.into())));
            }
        };
        let capture_connection = capture_connection(&mut request);
        if let Some(capture_smithy_connection) =
            request.extensions().get::<CaptureSmithyConnection>()
        {
            capture_smithy_connection
                .set_connection_retriever(move || extract_smithy_connection(&capture_connection));
        }
        let mut client = self.client.clone();
        use tower::Service;
        let fut = client.call(request);
        HttpConnectorFuture::new(async move {
            let response = fut
                .await
                .map_err(downcast_error)?
                .map(SdkBody::from_body_1_x);
            match HttpResponse::try_from(response) {
                Ok(response) => Ok(response),
                Err(err) => Err(ConnectorError::other(err.into(), None)),
            }
        })
    }
}

/// Downcast errors coming out of hyper into an appropriate `ConnectorError`
fn downcast_error(err: BoxError) -> ConnectorError {
    // is a `TimedOutError` (from aws_smithy_async::timeout) in the chain? if it is, this is a timeout
    if find_source::<TimedOutError>(err.as_ref()).is_some() {
        return ConnectorError::timeout(err);
    }
    // is the top of chain error actually already a `ConnectorError`? return that directly
    let err = match err.downcast::<ConnectorError>() {
        Ok(connector_error) => return *connector_error,
        Err(box_error) => box_error,
    };
    // generally, the top of chain will probably be a hyper error. Go through a set of hyper specific
    // error classifications
    let err = match find_source::<hyper::Error>(err.as_ref()) {
        Some(hyper_error) => return to_connector_error(hyper_error)(err),
        None => err,
    };

    // otherwise, we have no idea!
    ConnectorError::other(err, None)
}

/// Convert a [`hyper::Error`] into a [`ConnectorError`]
fn to_connector_error(err: &hyper::Error) -> fn(BoxError) -> ConnectorError {
    if err.is_timeout() || find_source::<timeout::HttpTimeoutError>(err).is_some() {
        return ConnectorError::timeout;
    }
    if err.is_user() {
        return ConnectorError::user;
    }
    if err.is_closed() || err.is_canceled() || find_source::<std::io::Error>(err).is_some() {
        return ConnectorError::io;
    }
    // We sometimes receive this from S3: hyper::Error(IncompleteMessage)
    if err.is_incomplete_message() {
        return |err: BoxError| ConnectorError::other(err, Some(ErrorKind::TransientError));
    }

    if let Some(h2_err) = find_source::<h2::Error>(err) {
        if h2_err.is_go_away()
            || (h2_err.is_reset() && h2_err.reason() == Some(Reason::REFUSED_STREAM))
        {
            return ConnectorError::io;
        }
    }

    tracing::warn!(err = %DisplayErrorContext(&err), "unrecognized error from Hyper. If this error should be retried, please file an issue.");
    |err: BoxError| ConnectorError::other(err, None)
}

fn find_source<'a, E: Error + 'static>(err: &'a (dyn Error + 'static)) -> Option<&'a E> {
    let mut next = Some(err);
    while let Some(err) = next {
        if let Some(matching_err) = err.downcast_ref::<E>() {
            return Some(matching_err);
        }
        next = err.source();
    }
    None
}

// TODO(https://github.com/awslabs/aws-sdk-rust/issues/1090): CacheKey must also include ptr equality to any
// runtime components that are used—sleep_impl as a base (unless we prohibit overriding sleep impl)
// If we decide to put a DnsResolver in RuntimeComponents, then we'll need to handle that as well.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct CacheKey {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
}

impl From<&HttpConnectorSettings> for CacheKey {
    fn from(value: &HttpConnectorSettings) -> Self {
        Self {
            connect_timeout: value.connect_timeout(),
            read_timeout: value.read_timeout(),
        }
    }
}

struct HyperClient<F> {
    connector_cache: RwLock<HashMap<CacheKey, SharedHttpConnector>>,
    client_builder: hyper_util::client::legacy::Builder,
    connector_fn: F,
}

impl<F> fmt::Debug for HyperClient<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HyperClient")
            .field("connector_cache", &self.connector_cache)
            .field("client_builder", &self.client_builder)
            .finish()
    }
}

impl<F> HttpClient for HyperClient<F>
where
    F: Fn(
            hyper_util::client::legacy::Builder,
            Option<&HttpConnectorSettings>,
            Option<&RuntimeComponents>,
        ) -> Connector
        + Send
        + Sync
        + 'static,
{
    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        let key = CacheKey::from(settings);
        let mut connector = self.connector_cache.read().unwrap().get(&key).cloned();
        if connector.is_none() {
            let mut cache = self.connector_cache.write().unwrap();
            // Short-circuit if another thread already wrote a connector to the cache for this key
            if !cache.contains_key(&key) {
                let start = components.time_source().map(|ts| ts.now());
                let connector = (self.connector_fn)(
                    self.client_builder.clone(),
                    Some(settings),
                    Some(components),
                );
                let end = components.time_source().map(|ts| ts.now());
                if let (Some(start), Some(end)) = (start, end) {
                    if let Ok(elapsed) = end.duration_since(start) {
                        tracing::debug!("new connector created in {:?}", elapsed);
                    }
                }
                let connector = SharedHttpConnector::new(connector);
                cache.insert(key.clone(), connector);
            }
            connector = cache.get(&key).cloned();
        }

        connector.expect("cache populated above")
    }

    fn validate_base_client_config(
        &self,
        _: &RuntimeComponentsBuilder,
        _: &ConfigBag,
    ) -> Result<(), BoxError> {
        // Initialize the TCP connector at this point so that native certs load
        // at client initialization time instead of upon first request. We do it
        // here rather than at construction so that it won't run if this is not
        // the selected HTTP client for the base config (for example, if this was
        // the default HTTP client, and it was overridden by a later plugin).
        let _ = (self.connector_fn)(self.client_builder.clone(), None, None);
        Ok(())
    }

    fn connector_metadata(&self) -> Option<ConnectorMetadata> {
        Some(ConnectorMetadata::new("hyper", Some(Cow::Borrowed("1.x"))))
    }
}

/// Builder for a hyper-backed [`HttpClient`] implementation.
///
/// This builder can be used to customize the underlying TCP connector used, as well as
/// hyper client configuration.
///
/// # Examples
///
/// Construct a Hyper client with the RusTLS TLS implementation.
/// This can be useful when you want to share a Hyper connector between multiple
/// generated Smithy clients.
#[derive(Clone, Default, Debug)]
pub struct Builder<Tls = TlsUnset> {
    client_builder: Option<hyper_util::client::legacy::Builder>,
    #[allow(unused)]
    tls_provider: Tls,
}

cfg_tls! {
    mod dns;
    use aws_smithy_runtime_api::client::dns::ResolveDns;

    impl ConnectorBuilder<TlsProviderSelected> {
        /// Build a [`Connector`] that will use the default DNS resolver implementation.
        pub fn build(self) -> Connector {
            let http_connector = self.base_connector();
            self.build_https(http_connector)
        }

        /// Configure the TLS context
        pub fn tls_context(mut self, ctx: TlsContext) -> Self {
            self.tls.context = ctx;
            self
        }

        /// Configure the TLS context
        pub fn set_tls_context(&mut self, ctx: TlsContext) -> &mut Self {
            self.tls.context = ctx;
            self
        }

        /// Build a [`Connector`] that will use the given DNS resolver implementation.
        pub fn build_with_resolver<R: ResolveDns + Clone + 'static>(self, resolver: R) -> Connector {
            use crate::client::dns::HyperUtilResolver;
            let http_connector = self.base_connector_with_resolver(HyperUtilResolver { resolver });
            self.build_https(http_connector)
        }

        fn build_https<R>(self, http_connector: HyperHttpConnector<R>) -> Connector
        where
            R: Clone + Send + Sync + 'static,
            R: tower::Service<hyper_util::client::legacy::connect::dns::Name>,
            R::Response: Iterator<Item = std::net::SocketAddr>,
            R::Future: Send,
            R::Error: Into<Box<dyn Error + Send + Sync>>,
        {
            match &self.tls.provider {
                // TODO(hyper1) - fix cfg_rustls! to allow matching on patterns so we can re-use it and not duplicate these cfg matches everywhere
                #[cfg(any(
                    feature = "rustls-aws-lc",
                    feature = "rustls-aws-lc-fips",
                    feature = "rustls-ring"
                ))]
                tls::Provider::Rustls(crypto_mode) => {
                    let https_connector = tls::rustls_provider::build_connector::wrap_connector(
                        http_connector,
                        crypto_mode.clone(),
                        &self.tls.context,
                    );
                    self.wrap_connector(https_connector)
                },
                #[cfg(feature = "s2n-tls")]
                tls::Provider::S2nTls  => {
                    let https_connector = tls::s2n_tls_provider::build_connector::wrap_connector(http_connector, &self.tls.context);
                    self.wrap_connector(https_connector)
                }
            }
        }
    }

    impl Builder<TlsProviderSelected> {
        /// Create an HTTPS client with the selected TLS provider.
        ///
        /// The trusted certificates will be loaded later when this becomes the selected
        /// HTTP client for a Smithy client.
        pub fn build_https(self) -> SharedHttpClient {
            build_with_conn_fn(
                self.client_builder,
                move |client_builder, settings, runtime_components| {
                    let builder = new_conn_builder(client_builder, settings, runtime_components)
                        .tls_provider(self.tls_provider.provider.clone())
                        .tls_context(self.tls_provider.context.clone());
                    builder.build()
                },
            )
        }

        /// Create an HTTPS client using a custom DNS resolver
        pub fn build_with_resolver(
            self,
            resolver: impl ResolveDns + Clone + 'static,
        ) -> SharedHttpClient {
            build_with_conn_fn(
                self.client_builder,
                move |client_builder, settings, runtime_components| {
                    let builder = new_conn_builder(client_builder, settings, runtime_components)
                        .tls_provider(self.tls_provider.provider.clone())
                        .tls_context(self.tls_provider.context.clone());
                    builder.build_with_resolver(resolver.clone())
                },
            )
        }

        /// Configure the TLS context
        pub fn tls_context(mut self, ctx: TlsContext) -> Self {
            self.tls_provider.context = ctx;
            self
        }
    }
}

impl Builder<TlsUnset> {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a new HTTP client without TLS enabled
    #[doc(hidden)]
    pub fn build_http(self) -> SharedHttpClient {
        build_with_conn_fn(
            self.client_builder,
            move |client_builder, settings, runtime_components| {
                let builder = new_conn_builder(client_builder, settings, runtime_components);
                builder.build_http()
            },
        )
    }

    /// Set the TLS implementation to use
    pub fn tls_provider(self, provider: tls::Provider) -> Builder<TlsProviderSelected> {
        Builder {
            client_builder: self.client_builder,
            tls_provider: TlsProviderSelected {
                provider,
                context: TlsContext::default(),
            },
        }
    }
}

pub(crate) fn build_with_conn_fn<F>(
    client_builder: Option<hyper_util::client::legacy::Builder>,
    connector_fn: F,
) -> SharedHttpClient
where
    F: Fn(
            hyper_util::client::legacy::Builder,
            Option<&HttpConnectorSettings>,
            Option<&RuntimeComponents>,
        ) -> Connector
        + Send
        + Sync
        + 'static,
{
    SharedHttpClient::new(HyperClient {
        connector_cache: RwLock::new(HashMap::new()),
        client_builder: client_builder
            .unwrap_or_else(|| hyper_util::client::legacy::Builder::new(TokioExecutor::new())),
        connector_fn,
    })
}

#[allow(dead_code)]
pub(crate) fn build_with_tcp_conn_fn<C, F>(
    client_builder: Option<hyper_util::client::legacy::Builder>,
    tcp_connector_fn: F,
) -> SharedHttpClient
where
    F: Fn() -> C + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    C: tower::Service<Uri>,
    C::Response: Connection + Read + Write + Send + Sync + Unpin + 'static,
    C::Future: Unpin + Send + 'static,
    C::Error: Into<BoxError>,
    C: Connect,
{
    build_with_conn_fn(
        client_builder,
        move |client_builder, settings, runtime_components| {
            let builder = new_conn_builder(client_builder, settings, runtime_components);
            builder.wrap_connector(tcp_connector_fn())
        },
    )
}

fn new_conn_builder(
    client_builder: hyper_util::client::legacy::Builder,
    settings: Option<&HttpConnectorSettings>,
    runtime_components: Option<&RuntimeComponents>,
) -> ConnectorBuilder {
    let mut builder = Connector::builder().hyper_builder(client_builder);
    builder.set_connector_settings(settings.cloned());
    if let Some(components) = runtime_components {
        builder.set_sleep_impl(components.sleep_impl());
    }
    builder
}

#[cfg(test)]
mod test {
    use std::io::{Error, ErrorKind};
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use aws_smithy_async::assert_elapsed;
    use aws_smithy_async::rt::sleep::TokioSleep;
    use aws_smithy_async::time::SystemTimeSource;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use http_1x::Uri;
    use hyper::rt::ReadBufCursor;
    use hyper_util::client::legacy::connect::Connected;

    use crate::client::timeout::test::NeverConnects;

    use super::*;

    #[tokio::test]
    async fn connector_selection() {
        // Create a client that increments a count every time it creates a new Connector
        let creation_count = Arc::new(AtomicU32::new(0));
        let http_client = build_with_tcp_conn_fn(None, {
            let count = creation_count.clone();
            move || {
                count.fetch_add(1, Ordering::Relaxed);
                NeverConnects
            }
        });

        // This configuration should result in 4 separate connectors with different timeout settings
        let settings = [
            HttpConnectorSettings::builder()
                .connect_timeout(Duration::from_secs(3))
                .build(),
            HttpConnectorSettings::builder()
                .read_timeout(Duration::from_secs(3))
                .build(),
            HttpConnectorSettings::builder()
                .connect_timeout(Duration::from_secs(3))
                .read_timeout(Duration::from_secs(3))
                .build(),
            HttpConnectorSettings::builder()
                .connect_timeout(Duration::from_secs(5))
                .read_timeout(Duration::from_secs(3))
                .build(),
        ];

        // Kick off thousands of parallel tasks that will try to create a connector
        let components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(SystemTimeSource::new()))
            .build()
            .unwrap();
        let mut handles = Vec::new();
        for setting in &settings {
            for _ in 0..1000 {
                let client = http_client.clone();
                handles.push(tokio::spawn({
                    let setting = setting.clone();
                    let components = components.clone();
                    async move {
                        let _ = client.http_connector(&setting, &components);
                    }
                }));
            }
        }
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify only 4 connectors were created amidst the chaos
        assert_eq!(4, creation_count.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn hyper_io_error() {
        let connector = TestConnection {
            inner: HangupStream,
        };
        let adapter = Connector::builder().wrap_connector(connector).adapter;
        let err = adapter
            .call(HttpRequest::get("https://socket-hangup.com").unwrap())
            .await
            .expect_err("socket hangup");
        assert!(err.is_io(), "unexpected error type: {:?}", err);
    }

    // ---- machinery to make a Hyper connector that responds with an IO Error
    #[derive(Clone)]
    struct HangupStream;

    impl Connection for HangupStream {
        fn connected(&self) -> Connected {
            Connected::new()
        }
    }

    impl Read for HangupStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: ReadBufCursor<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Err(Error::new(
                ErrorKind::ConnectionReset,
                "connection reset",
            )))
        }
    }

    impl Write for HangupStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<Result<usize, Error>> {
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            Poll::Pending
        }

        fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
            Poll::Pending
        }
    }

    #[derive(Clone)]
    struct TestConnection<T> {
        inner: T,
    }

    impl<T> tower::Service<Uri> for TestConnection<T>
    where
        T: Clone + Connection,
    {
        type Response = T;
        type Error = BoxError;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: Uri) -> Self::Future {
            std::future::ready(Ok(self.inner.clone()))
        }
    }

    #[tokio::test]
    async fn http_connect_timeout_works() {
        let tcp_connector = NeverConnects::default();
        let connector_settings = HttpConnectorSettings::builder()
            .connect_timeout(Duration::from_secs(1))
            .build();
        let hyper = Connector::builder()
            .connector_settings(connector_settings)
            .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
            .wrap_connector(tcp_connector)
            .adapter;
        let now = tokio::time::Instant::now();
        tokio::time::pause();
        let resp = hyper
            .call(HttpRequest::get("https://static-uri.com").unwrap())
            .await
            .unwrap_err();
        assert!(
            resp.is_timeout(),
            "expected resp.is_timeout() to be true but it was false, resp == {:?}",
            resp
        );
        let message = DisplayErrorContext(&resp).to_string();
        let expected = "timeout: client error (Connect): HTTP connect timeout occurred after 1s";
        assert!(
            message.contains(expected),
            "expected '{message}' to contain '{expected}'"
        );
        assert_elapsed!(now, Duration::from_secs(1));
    }

    #[tokio::test]
    async fn http_read_timeout_works() {
        let tcp_connector = crate::client::timeout::test::NeverReplies;
        let connector_settings = HttpConnectorSettings::builder()
            .connect_timeout(Duration::from_secs(1))
            .read_timeout(Duration::from_secs(2))
            .build();
        let hyper = Connector::builder()
            .connector_settings(connector_settings)
            .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
            .wrap_connector(tcp_connector)
            .adapter;
        let now = tokio::time::Instant::now();
        tokio::time::pause();
        let err = hyper
            .call(HttpRequest::get("https://fake-uri.com").unwrap())
            .await
            .unwrap_err();
        assert!(
            err.is_timeout(),
            "expected err.is_timeout() to be true but it was false, err == {err:?}",
        );
        let message = format!("{}", DisplayErrorContext(&err));
        let expected = "timeout: HTTP read timeout occurred after 2s";
        assert!(
            message.contains(expected),
            "expected '{message}' to contain '{expected}'"
        );
        assert_elapsed!(now, Duration::from_secs(2));
    }

    #[cfg(feature = "s2n-tls")]
    #[tokio::test]
    async fn s2n_tls_provider() {
        // Create an HttpConnector with the s2n-tls provider.
        let client = Builder::new()
            .tls_provider(tls::Provider::S2nTls)
            .build_https();
        let connector_settings = HttpConnectorSettings::builder().build();

        // HyperClient::http_connector invokes TimeSource::now to determine how long it takes to
        // create new HttpConnectors. As such, a real time source must be provided.
        let runtime_components = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(SystemTimeSource::new()))
            .build()
            .unwrap();

        let connector = client.http_connector(&connector_settings, &runtime_components);

        // Ensure that s2n-tls is used as the underlying TLS provider when selected.
        //
        // s2n-tls-hyper will error when given an invalid scheme. Ensure that this error is produced
        // from s2n-tls-hyper, and not another TLS provider.
        let error = connector
            .call(HttpRequest::get("notascheme://amazon.com").unwrap())
            .await
            .unwrap_err();
        let error = error.into_source();
        let s2n_error = error
            .source()
            .unwrap()
            .downcast_ref::<s2n_tls_hyper::error::Error>()
            .unwrap();
        assert!(matches!(
            s2n_error,
            s2n_tls_hyper::error::Error::InvalidScheme
        ));
    }
}
