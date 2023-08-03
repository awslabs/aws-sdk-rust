/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Implementation of [`SmithyConnector`](crate::bounds::SmithyConnector) for Hyper
//!
//! The module provides [`Adapter`] which enables using a [`hyper::Client`] as the connector for a Smithy
//! [`Client`](crate::Client). For most use cases, this shouldn't need to be used directly, but it is
//! available as an option.
//!
//! # Examples
//!
//! ### Construct a Smithy Client with Hyper and Rustls
//!
//! In the basic case, customers should not need to use this module. A default implementation of Hyper
//! with `rustls` will be constructed during client creation. However, if you are creating a Smithy
//! [`Client`](crate::Client), directly, use the `dyn_https_https()` method to match that default behavior:
//!
#![cfg_attr(
    not(all(feature = "rustls", feature = "client-hyper")),
    doc = "```no_run,ignore"
)]
#![cfg_attr(all(feature = "rustls", feature = "client-hyper"), doc = "```no_run")]
//! use aws_smithy_client::Client;
//!
//! let client = Client::builder()
//!     .dyn_https_connector(Default::default())
//!     .middleware(
//!         // Replace this with your middleware type
//!         tower::layer::util::Identity::default()
//!     )
//!     .build();
//! ```
//!
//! ### Use a Hyper client that uses WebPKI roots
//!
//! A use case for where you may want to use the [`Adapter`] is when settings Hyper client settings
//! that aren't otherwise exposed by the `Client` builder interface.
//!
#![cfg_attr(
    not(all(feature = "rustls", feature = "client-hyper")),
    doc = "```no_run,ignore"
)]
#![cfg_attr(
    all(
        feature = "rustls",
        feature = "client-hyper",
        feature = "hyper-webpki-doctest-only"
    ),
    doc = "```no_run"
)]
//! use std::time::Duration;
//! use aws_smithy_client::{Client, conns, hyper_ext};
//! use aws_smithy_client::erase::DynConnector;
//! use aws_smithy_client::http_connector::ConnectorSettings;
//!
//! let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
//!     .with_webpki_roots()
//!     .https_only()
//!     .enable_http1()
//!     .enable_http2()
//!     .build();
//! let smithy_connector = hyper_ext::Adapter::builder()
//!     // Optionally set things like timeouts as well
//!     .connector_settings(
//!         ConnectorSettings::builder()
//!             .connect_timeout(Duration::from_secs(5))
//!             .build()
//!     )
//!     .build(https_connector);
//!
//! // Once you have a Smithy connector, use it to construct a Smithy client:
//! let client = Client::builder()
//!     .connector(smithy_connector)
//!     .middleware(tower::layer::util::Identity::default())
//!     .build();
//! ```

use crate::http_connector::ConnectorSettings;
use crate::hyper_ext::timeout_middleware::{ConnectTimeout, HttpReadTimeout, HttpTimeoutError};
use crate::never::stream::EmptyStream;
use aws_smithy_async::future::timeout::TimedOutError;
use aws_smithy_async::rt::sleep::{default_async_sleep, SharedAsyncSleep};
use aws_smithy_http::body::SdkBody;

use aws_smithy_http::result::ConnectorError;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::retry::ErrorKind;
use http::{Extensions, Uri};
use hyper::client::connect::{
    capture_connection, CaptureConnection, Connected, Connection, HttpInfo,
};

use std::error::Error;
use std::fmt::Debug;

use crate::erase::boxclone::BoxFuture;
use aws_smithy_http::connection::{CaptureSmithyConnection, ConnectionMetadata};
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{BoxError, Service};

/// Adapter from a [`hyper::Client`](hyper::Client) to a connector usable by a Smithy [`Client`](crate::Client).
///
/// This adapter also enables TCP `CONNECT` and HTTP `READ` timeouts via [`Adapter::builder`]. For examples
/// see [the module documentation](crate::hyper_ext).
#[derive(Clone, Debug)]
pub struct Adapter<C> {
    client: HttpReadTimeout<hyper::Client<ConnectTimeout<C>, SdkBody>>,
}

/// Extract a smithy connection from a hyper CaptureConnection
fn extract_smithy_connection(capture_conn: &CaptureConnection) -> Option<ConnectionMetadata> {
    let capture_conn = capture_conn.clone();
    if let Some(conn) = capture_conn.clone().connection_metadata().as_ref() {
        let mut extensions = Extensions::new();
        conn.get_extras(&mut extensions);
        let http_info = extensions.get::<HttpInfo>();
        let smithy_connection = ConnectionMetadata::new(
            conn.is_proxied(),
            http_info.map(|info| info.remote_addr()),
            move || match capture_conn.connection_metadata().as_ref() {
                Some(conn) => conn.poison(),
                None => tracing::trace!("no connection existed to poison"),
            },
        );
        Some(smithy_connection)
    } else {
        None
    }
}

impl<C> Service<http::Request<SdkBody>> for Adapter<C>
where
    C: Clone + Send + Sync + 'static,
    C: Service<Uri>,
    C::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    C::Future: Unpin + Send + 'static,
    C::Error: Into<BoxError>,
{
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;

    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.client.poll_ready(cx).map_err(downcast_error)
    }

    fn call(&mut self, mut req: http::Request<SdkBody>) -> Self::Future {
        let capture_connection = capture_connection(&mut req);
        if let Some(capture_smithy_connection) = req.extensions().get::<CaptureSmithyConnection>() {
            capture_smithy_connection
                .set_connection_retriever(move || extract_smithy_connection(&capture_connection));
        }
        let fut = self.client.call(req);
        Box::pin(async move { Ok(fut.await.map_err(downcast_error)?.map(SdkBody::from)) })
    }
}

impl Adapter<()> {
    /// Builder for a Hyper Adapter
    ///
    /// Generally, end users should not need to construct an [`Adapter`] manually: a hyper adapter
    /// will be constructed automatically during client creation.
    pub fn builder() -> Builder {
        Builder::default()
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
    let err = match err.downcast::<hyper::Error>() {
        Ok(hyper_error) => return to_connector_error(*hyper_error),
        Err(box_error) => box_error,
    };

    // otherwise, we have no idea!
    ConnectorError::other(err, None)
}

/// Convert a [`hyper::Error`] into a [`ConnectorError`]
fn to_connector_error(err: hyper::Error) -> ConnectorError {
    if err.is_timeout() || find_source::<HttpTimeoutError>(&err).is_some() {
        ConnectorError::timeout(err.into())
    } else if err.is_user() {
        ConnectorError::user(err.into())
    } else if err.is_closed() || err.is_canceled() || find_source::<std::io::Error>(&err).is_some()
    {
        ConnectorError::io(err.into())
    }
    // We sometimes receive this from S3: hyper::Error(IncompleteMessage)
    else if err.is_incomplete_message() {
        ConnectorError::other(err.into(), Some(ErrorKind::TransientError))
    } else {
        tracing::warn!(err = %DisplayErrorContext(&err), "unrecognized error from Hyper. If this error should be retried, please file an issue.");
        ConnectorError::other(err.into(), None)
    }
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

/// Builder for [`hyper_ext::Adapter`](Adapter)
///
/// Unlike a Smithy client, the [`Service`] inside a [`hyper_ext::Adapter`](Adapter) is actually a service that
/// accepts a `Uri` and returns a TCP stream. One default implementation of this is provided,
/// that encrypts the stream with `rustls`.
///
/// # Examples
/// Construct a HyperAdapter with the default HTTP implementation (rustls). This can be useful when you want to share a Hyper connector
/// between multiple Smithy clients.
///
#[cfg_attr(
    not(all(feature = "rustls", feature = "client-hyper")),
    doc = "```no_run,ignore"
)]
#[cfg_attr(all(feature = "rustls", feature = "client-hyper"), doc = "```no_run")]
/// use tower::layer::util::Identity;
/// use aws_smithy_client::{conns, hyper_ext};
/// use aws_smithy_client::erase::DynConnector;
///
/// let hyper_connector = hyper_ext::Adapter::builder().build(conns::https());
/// // this client can then be used when constructing a Smithy Client
/// // Replace `Identity` with your middleware implementation:
/// let client = aws_smithy_client::Client::<DynConnector, Identity>::new(DynConnector::new(hyper_connector));
/// ```
#[derive(Default, Debug)]
pub struct Builder {
    connector_settings: Option<ConnectorSettings>,
    sleep_impl: Option<SharedAsyncSleep>,
    client_builder: Option<hyper::client::Builder>,
}

impl Builder {
    /// Create a HyperAdapter from this builder and a given connector
    pub fn build<C>(self, connector: C) -> Adapter<C>
    where
        C: Clone + Send + Sync + 'static,
        C: Service<Uri>,
        C::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
        C::Future: Unpin + Send + 'static,
        C::Error: Into<BoxError>,
    {
        let client_builder = self.client_builder.unwrap_or_default();
        let sleep_impl = self.sleep_impl.or_else(default_async_sleep);
        let (connect_timeout, read_timeout) = self
            .connector_settings
            .map(|c| (c.connect_timeout(), c.read_timeout()))
            .unwrap_or((None, None));

        // if we are using Hyper, Tokio must already be enabled so we can fallback to Tokio.
        let connector = match connect_timeout {
            Some(duration) => ConnectTimeout::new(
                connector,
                sleep_impl
                    .clone()
                    .expect("a sleep impl must be provided in order to have a connect timeout"),
                duration,
            ),
            None => ConnectTimeout::no_timeout(connector),
        };
        let base = client_builder.build(connector);
        let read_timeout = match read_timeout {
            Some(duration) => HttpReadTimeout::new(
                base,
                sleep_impl.expect("a sleep impl must be provided in order to have a read timeout"),
                duration,
            ),
            None => HttpReadTimeout::no_timeout(base),
        };
        Adapter {
            client: read_timeout,
        }
    }

    /// Set the async sleep implementation used for timeouts
    ///
    /// Calling this is only necessary for testing or to use something other than
    /// [`default_async_sleep`].
    pub fn sleep_impl(mut self, sleep_impl: SharedAsyncSleep) -> Self {
        self.sleep_impl = Some(sleep_impl);
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
    pub fn connector_settings(mut self, connector_settings: ConnectorSettings) -> Self {
        self.connector_settings = Some(connector_settings);
        self
    }

    /// Configure the HTTP settings for the `HyperAdapter`
    pub fn set_connector_settings(
        &mut self,
        connector_settings: Option<ConnectorSettings>,
    ) -> &mut Self {
        self.connector_settings = connector_settings;
        self
    }

    /// Override the Hyper client [`Builder`](hyper::client::Builder) used to construct this client.
    ///
    /// This enables changing settings like forcing HTTP2 and modifying other default client behavior.
    pub fn hyper_builder(mut self, hyper_builder: hyper::client::Builder) -> Self {
        self.client_builder = Some(hyper_builder);
        self
    }

    /// Override the Hyper client [`Builder`](hyper::client::Builder) used to construct this client.
    ///
    /// This enables changing settings like forcing HTTP2 and modifying other default client behavior.
    pub fn set_hyper_builder(
        &mut self,
        hyper_builder: Option<hyper::client::Builder>,
    ) -> &mut Self {
        self.client_builder = hyper_builder;
        self
    }
}

mod timeout_middleware {
    use std::error::Error;
    use std::fmt::Formatter;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::time::Duration;

    use http::Uri;
    use pin_project_lite::pin_project;
    use tower::BoxError;

    use aws_smithy_async::future::timeout::{TimedOutError, Timeout};
    use aws_smithy_async::rt::sleep::Sleep;
    use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep};

    #[derive(Debug)]
    pub(crate) struct HttpTimeoutError {
        kind: &'static str,
        duration: Duration,
    }

    impl std::fmt::Display for HttpTimeoutError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{} timeout occurred after {:?}",
                self.kind, self.duration
            )
        }
    }

    impl Error for HttpTimeoutError {
        // We implement the `source` function as returning a `TimedOutError` because when `downcast_error`
        // or `find_source` is called with an `HttpTimeoutError` (or another error wrapping an `HttpTimeoutError`)
        // this method will be checked to determine if it's a timeout-related error.
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&TimedOutError)
        }
    }

    /// Timeout wrapper that will timeout on the initial TCP connection
    ///
    /// # Stability
    /// This interface is unstable.
    #[derive(Clone, Debug)]
    pub(super) struct ConnectTimeout<I> {
        inner: I,
        timeout: Option<(SharedAsyncSleep, Duration)>,
    }

    impl<I> ConnectTimeout<I> {
        /// Create a new `ConnectTimeout` around `inner`.
        ///
        /// Typically, `I` will implement [`hyper::client::connect::Connect`].
        pub(crate) fn new(inner: I, sleep: SharedAsyncSleep, timeout: Duration) -> Self {
            Self {
                inner,
                timeout: Some((sleep, timeout)),
            }
        }

        pub(crate) fn no_timeout(inner: I) -> Self {
            Self {
                inner,
                timeout: None,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub(crate) struct HttpReadTimeout<I> {
        inner: I,
        timeout: Option<(SharedAsyncSleep, Duration)>,
    }

    impl<I> HttpReadTimeout<I> {
        /// Create a new `HttpReadTimeout` around `inner`.
        ///
        /// Typically, `I` will implement [`tower::Service<http::Request<SdkBody>>`].
        pub(crate) fn new(inner: I, sleep: SharedAsyncSleep, timeout: Duration) -> Self {
            Self {
                inner,
                timeout: Some((sleep, timeout)),
            }
        }

        pub(crate) fn no_timeout(inner: I) -> Self {
            Self {
                inner,
                timeout: None,
            }
        }
    }

    pin_project! {
        /// Timeout future for Tower services
        ///
        /// Timeout future to handle timing out, mapping errors, and the possibility of not timing out
        /// without incurring an additional allocation for each timeout layer.
        #[project = MaybeTimeoutFutureProj]
        pub enum MaybeTimeoutFuture<F> {
            Timeout {
                #[pin]
                timeout: Timeout<F, Sleep>,
                error_type: &'static str,
                duration: Duration,
            },
            NoTimeout {
                #[pin]
                future: F
            }
        }
    }

    impl<F, T, E> Future for MaybeTimeoutFuture<F>
    where
        F: Future<Output = Result<T, E>>,
        E: Into<BoxError>,
    {
        type Output = Result<T, BoxError>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let (timeout_future, kind, &mut duration) = match self.project() {
                MaybeTimeoutFutureProj::NoTimeout { future } => {
                    return future.poll(cx).map_err(|err| err.into());
                }
                MaybeTimeoutFutureProj::Timeout {
                    timeout,
                    error_type,
                    duration,
                } => (timeout, error_type, duration),
            };
            match timeout_future.poll(cx) {
                Poll::Ready(Ok(response)) => Poll::Ready(response.map_err(|err| err.into())),
                Poll::Ready(Err(_timeout)) => {
                    Poll::Ready(Err(HttpTimeoutError { kind, duration }.into()))
                }
                Poll::Pending => Poll::Pending,
            }
        }
    }

    impl<I> tower::Service<Uri> for ConnectTimeout<I>
    where
        I: tower::Service<Uri>,
        I::Error: Into<BoxError>,
    {
        type Response = I::Response;
        type Error = BoxError;
        type Future = MaybeTimeoutFuture<I::Future>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx).map_err(|err| err.into())
        }

        fn call(&mut self, req: Uri) -> Self::Future {
            match &self.timeout {
                Some((sleep, duration)) => {
                    let sleep = sleep.sleep(*duration);
                    MaybeTimeoutFuture::Timeout {
                        timeout: Timeout::new(self.inner.call(req), sleep),
                        error_type: "HTTP connect",
                        duration: *duration,
                    }
                }
                None => MaybeTimeoutFuture::NoTimeout {
                    future: self.inner.call(req),
                },
            }
        }
    }

    impl<I, B> tower::Service<http::Request<B>> for HttpReadTimeout<I>
    where
        I: tower::Service<http::Request<B>, Error = hyper::Error>,
    {
        type Response = I::Response;
        type Error = BoxError;
        type Future = MaybeTimeoutFuture<I::Future>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx).map_err(|err| err.into())
        }

        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match &self.timeout {
                Some((sleep, duration)) => {
                    let sleep = sleep.sleep(*duration);
                    MaybeTimeoutFuture::Timeout {
                        timeout: Timeout::new(self.inner.call(req), sleep),
                        error_type: "HTTP read",
                        duration: *duration,
                    }
                }
                None => MaybeTimeoutFuture::NoTimeout {
                    future: self.inner.call(req),
                },
            }
        }
    }

    #[cfg(test)]
    mod test {
        use crate::http_connector::ConnectorSettings;
        use crate::hyper_ext::Adapter;
        use crate::never::{NeverConnected, NeverReplies};
        use aws_smithy_async::assert_elapsed;
        use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
        use aws_smithy_http::body::SdkBody;
        use aws_smithy_types::error::display::DisplayErrorContext;
        use aws_smithy_types::timeout::TimeoutConfig;
        use std::time::Duration;
        use tower::Service;

        #[allow(unused)]
        fn connect_timeout_is_correct<T: Send + Sync + Clone + 'static>() {
            is_send_sync::<super::ConnectTimeout<T>>();
        }

        #[allow(unused)]
        fn is_send_sync<T: Send + Sync>() {}

        #[tokio::test]
        async fn http_connect_timeout_works() {
            let inner = NeverConnected::new();
            let connector_settings = ConnectorSettings::from_timeout_config(
                &TimeoutConfig::builder()
                    .connect_timeout(Duration::from_secs(1))
                    .build(),
            );
            let mut hyper = Adapter::builder()
                .connector_settings(connector_settings)
                .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
                .build(inner);
            let now = tokio::time::Instant::now();
            tokio::time::pause();
            let resp = hyper
                .call(
                    http::Request::builder()
                        .uri("http://foo.com")
                        .body(SdkBody::empty())
                        .unwrap(),
                )
                .await
                .unwrap_err();
            assert!(
                resp.is_timeout(),
                "expected resp.is_timeout() to be true but it was false, resp == {:?}",
                resp
            );
            let message = DisplayErrorContext(&resp).to_string();
            let expected =
                "timeout: error trying to connect: HTTP connect timeout occurred after 1s";
            assert!(
                message.contains(expected),
                "expected '{message}' to contain '{expected}'"
            );
            assert_elapsed!(now, Duration::from_secs(1));
        }

        #[tokio::test]
        async fn http_read_timeout_works() {
            let inner = NeverReplies::new();
            let connector_settings = ConnectorSettings::from_timeout_config(
                &TimeoutConfig::builder()
                    .connect_timeout(Duration::from_secs(1))
                    .read_timeout(Duration::from_secs(2))
                    .build(),
            );
            let mut hyper = Adapter::builder()
                .connector_settings(connector_settings)
                .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
                .build(inner);
            let now = tokio::time::Instant::now();
            tokio::time::pause();
            let resp = hyper
                .call(
                    http::Request::builder()
                        .uri("http://foo.com")
                        .body(SdkBody::empty())
                        .unwrap(),
                )
                .await
                .unwrap_err();
            assert!(
                resp.is_timeout(),
                "expected resp.is_timeout() to be true but it was false, resp == {:?}",
                resp
            );
            let message = format!("{}", DisplayErrorContext(&resp));
            let expected = "timeout: HTTP read timeout occurred after 2s";
            assert!(
                message.contains(expected),
                "expected '{message}' to contain '{expected}'"
            );
            assert_elapsed!(now, Duration::from_secs(2));
        }
    }
}

/// Make `EmptyStream` compatible with Hyper
impl Connection for EmptyStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[cfg(test)]
mod test {
    use crate::hyper_ext::Adapter;
    use aws_smithy_http::body::SdkBody;
    use http::Uri;
    use hyper::client::connect::{Connected, Connection};
    use std::io::{Error, ErrorKind};
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
    use tower::BoxError;

    #[tokio::test]
    async fn hyper_io_error() {
        let connector = TestConnection {
            inner: HangupStream,
        };
        let mut adapter = Adapter::builder().build(connector);
        use tower::Service;
        let err = adapter
            .call(
                http::Request::builder()
                    .uri("http://amazon.com")
                    .body(SdkBody::empty())
                    .unwrap(),
            )
            .await
            .expect_err("socket hangup");
        assert!(err.is_io(), "{:?}", err);
    }

    // ---- machinery to make a Hyper connector that responds with an IO Error
    #[derive(Clone)]
    struct HangupStream;

    impl Connection for HangupStream {
        fn connected(&self) -> Connected {
            Connected::new()
        }
    }

    impl AsyncRead for HangupStream {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(Err(Error::new(
                ErrorKind::ConnectionReset,
                "connection reset",
            )))
        }
    }

    impl AsyncWrite for HangupStream {
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
}
