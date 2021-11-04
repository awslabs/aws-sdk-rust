/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::sync::Arc;

use http::Uri;
use hyper::client::connect::Connection;

use tokio::io::{AsyncRead, AsyncWrite};
use tower::{BoxError, Service};

use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
pub use aws_smithy_http::result::{SdkError, SdkSuccess};
use std::error::Error;

use crate::hyper_impls::timeout_middleware::{ConnectTimeout, HttpReadTimeout, TimeoutError};
use crate::{timeout, Builder as ClientBuilder};
use aws_smithy_async::future::timeout::TimedOutError;
use aws_smithy_types::retry::ErrorKind;

/// Adapter from a [`hyper::Client`] to a connector usable by a [`Client`](crate::Client).
///
/// This adapter also enables TCP connect and HTTP read timeouts via [`HyperAdapter::builder`]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HyperAdapter<C>(HttpReadTimeout<hyper::Client<ConnectTimeout<C>, SdkBody>>);

impl<C> Service<http::Request<SdkBody>> for HyperAdapter<C>
where
    C: Clone + Send + Sync + 'static,
    C: tower::Service<Uri>,
    C::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    C::Future: Unpin + Send + 'static,
    C::Error: Into<BoxError>,
{
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;

    #[allow(clippy::type_complexity)]
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx).map_err(downcast_error)
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        let fut = self.0.call(req);
        Box::pin(async move { Ok(fut.await.map_err(downcast_error)?.map(SdkBody::from)) })
    }
}

impl HyperAdapter<()> {
    /// Builder for a Hyper Adapter
    ///
    /// Generally, end users should not need to construct a HyperAdapter manually: a hyper adapter
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
    if err.is_timeout() || find_source::<TimeoutError>(&err).is_some() {
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
        tracing::warn!(err = ?err, "unrecognized error from Hyper. If this error should be retried, please file an issue.");
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

#[derive(Default, Debug)]
/// Builder for [`HyperAdapter`]
pub struct Builder {
    timeout: timeout::Settings,
    sleep: Option<Arc<dyn AsyncSleep>>,
    client_builder: hyper::client::Builder,
}

impl Builder {
    /// Create a HyperAdapter from this builder and a given connector
    pub fn build<C>(self, connector: C) -> HyperAdapter<C>
    where
        C: Clone + Send + Sync + 'static,
        C: tower::Service<Uri>,
        C::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
        C::Future: Unpin + Send + 'static,
        C::Error: Into<BoxError>,
    {
        // if we are using Hyper, Tokio must already be enabled so we can fallback to Tokio.
        let sleep = self.sleep.or_else(default_async_sleep);
        let connector = match self.timeout.connect() {
            Some(duration) => ConnectTimeout::new(
                connector,
                sleep
                    .clone()
                    .expect("a sleep impl must be provided to use timeouts"),
                duration,
            ),
            None => ConnectTimeout::no_timeout(connector),
        };
        let base = self.client_builder.build(connector);
        let http_timeout = match self.timeout.read() {
            Some(duration) => HttpReadTimeout::new(
                base,
                sleep
                    .clone()
                    .expect("a sleep impl must be provided to use timeouts"),
                duration,
            ),
            None => HttpReadTimeout::no_timeout(base),
        };
        HyperAdapter(http_timeout)
    }

    /// Set the async sleep implementation used for timeouts
    ///
    /// Calling this is only necessary for testing or to use something other than
    /// [`aws_smithy_async::rt::sleep::default_async_sleep`].
    pub fn sleep_impl(self, sleep_impl: impl AsyncSleep + 'static) -> Self {
        Self {
            sleep: Some(Arc::new(sleep_impl)),
            ..self
        }
    }

    /// Configure the timeout for the HyperAdapter
    ///
    /// When unset, the underlying adaptor will not use any timeouts.
    pub fn timeout(self, timeout_config: &timeout::Settings) -> Self {
        Self {
            timeout: timeout_config.clone(),
            ..self
        }
    }

    /// Override the Hyper client [`Builder`](hyper::client::Builder) used to construct this client.
    ///
    /// This enables changing settings like forcing HTTP2 and modifying other default client behavior.
    pub fn hyper_builder(self, hyper_builder: hyper::client::Builder) -> Self {
        Self {
            client_builder: hyper_builder,
            ..self
        }
    }
}

#[cfg(any(feature = "rustls", feature = "native_tls"))]
impl<M> crate::Client<crate::erase::DynConnector, M>
where
    M: Default,
    M: crate::bounds::SmithyMiddleware<crate::erase::DynConnector> + Send + Sync + 'static,
{
    /// Create a Smithy client that uses HTTPS and the [standard retry
    /// policy](crate::retry::Standard) over the default middleware implementation.
    ///
    /// For convenience, this constructor type-erases the concrete TLS connector backend used using
    /// dynamic dispatch. This comes at a slight runtime performance cost. See
    /// [`DynConnector`](crate::erase::DynConnector) for details. To avoid that overhead, use
    /// [`Builder::rustls`](ClientBuilder::rustls) or `Builder::native_tls` instead.
    pub fn https() -> Self {
        #[cfg(feature = "rustls")]
        let with_https = |b: ClientBuilder<_>| b.rustls();
        // If we are compiling this function & rustls is not enabled, then native-tls MUST be enabled
        #[cfg(not(feature = "rustls"))]
        let with_https = |b: ClientBuilder<_>| b.native_tls();

        with_https(ClientBuilder::new())
            .middleware(M::default())
            .build()
            .into_dyn_connector()
    }
}

#[cfg(feature = "rustls")]
impl<M, R> ClientBuilder<(), M, R> {
    /// Connect to the service over HTTPS using Rustls.
    pub fn rustls(self) -> ClientBuilder<HyperAdapter<crate::conns::Https>, M, R> {
        self.connector(HyperAdapter::builder().build(crate::conns::https()))
    }

    /// Connect to the service over HTTPS using Rustls.
    ///
    /// This is exactly equivalent to [`Builder::rustls`](ClientBuilder::rustls). If you instead wish to use `native_tls`,
    /// use `Builder::native_tls`.
    pub fn https(self) -> ClientBuilder<HyperAdapter<crate::conns::Https>, M, R> {
        self.rustls()
    }
}
#[cfg(feature = "native-tls")]
impl<M, R> ClientBuilder<(), M, R> {
    /// Connect to the service over HTTPS using the native TLS library on your platform.
    pub fn native_tls(
        self,
    ) -> ClientBuilder<HyperAdapter<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>, M, R>
    {
        self.connector(HyperAdapter::builder().build(crate::conns::native_tls()))
    }
}

mod timeout_middleware {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};
    use std::time::Duration;

    use http::Uri;

    use pin_project_lite::pin_project;

    use aws_smithy_async::future;
    use aws_smithy_async::future::timeout::{TimedOutError, Timeout};
    use aws_smithy_async::rt::sleep::AsyncSleep;
    use aws_smithy_async::rt::sleep::Sleep;
    use std::error::Error;
    use std::fmt::Formatter;
    use tower::BoxError;

    #[derive(Debug)]
    pub(crate) struct TimeoutError {
        operation: &'static str,
        duration: Duration,
        cause: TimedOutError,
    }

    impl std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "timed out after {:?}", self.duration)
        }
    }

    impl Error for TimeoutError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.cause)
        }
    }

    /// Timeout wrapper that will timeout on the initial TCP connection
    ///
    /// # Stability
    /// This interface is unstable.
    #[derive(Clone, Debug)]
    pub(super) struct ConnectTimeout<I> {
        inner: I,
        timeout: Option<(Arc<dyn AsyncSleep>, Duration)>,
    }

    impl<I> ConnectTimeout<I> {
        /// Create a new `ConnectTimeout` around `inner`.
        ///
        /// Typically, `I` will implement [`hyper::client::connect::Connect`].
        pub fn new(inner: I, sleep: Arc<dyn AsyncSleep>, timeout: Duration) -> Self {
            Self {
                inner,
                timeout: Some((sleep, timeout)),
            }
        }

        pub fn no_timeout(inner: I) -> Self {
            Self {
                inner,
                timeout: None,
            }
        }
    }

    #[derive(Clone, Debug)]
    pub struct HttpReadTimeout<I> {
        inner: I,
        timeout: Option<(Arc<dyn AsyncSleep>, Duration)>,
    }

    impl<I> HttpReadTimeout<I> {
        /// Create a new `HttpReadTimeout` around `inner`.
        ///
        /// Typically, `I` will implement [`tower::Service<http::Request<SdkBody>>`].
        pub fn new(inner: I, sleep: Arc<dyn AsyncSleep>, timeout: Duration) -> Self {
            Self {
                inner,
                timeout: Some((sleep, timeout)),
            }
        }

        pub fn no_timeout(inner: I) -> Self {
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
            let (timeout_future, timeout_type, dur) = match self.project() {
                MaybeTimeoutFutureProj::NoTimeout { future } => {
                    return future.poll(cx).map_err(|err| err.into())
                }
                MaybeTimeoutFutureProj::Timeout {
                    timeout,
                    error_type,
                    duration,
                } => (timeout, error_type, duration),
            };
            match timeout_future.poll(cx) {
                Poll::Ready(Ok(response)) => Poll::Ready(response.map_err(|err| err.into())),
                Poll::Ready(Err(_timeout)) => Poll::Ready(Err(TimeoutError {
                    operation: timeout_type,
                    duration: *dur,
                    cause: TimedOutError,
                }
                .into())),
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
                        timeout: future::timeout::Timeout::new(self.inner.call(req), sleep),
                        error_type: "connect",
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
                        timeout: future::timeout::Timeout::new(self.inner.call(req), sleep),

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
        use crate::hyper_impls::HyperAdapter;
        use crate::never::{NeverConnected, NeverReplies};
        use crate::timeout;
        use aws_smithy_async::rt::sleep::TokioSleep;
        use aws_smithy_http::body::SdkBody;
        use std::time::Duration;
        use tower::Service;

        macro_rules! assert_elapsed {
            ($start:expr, $dur:expr) => {{
                let elapsed = $start.elapsed();
                // type ascription improves compiler error when wrong type is passed
                let lower: std::time::Duration = $dur;

                // Handles ms rounding
                assert!(
                    elapsed >= lower && elapsed <= lower + std::time::Duration::from_millis(5),
                    "actual = {:?}, expected = {:?}",
                    elapsed,
                    lower
                );
            }};
        }

        #[allow(unused)]
        fn connect_timeout_is_correct<T: Send + Sync + Clone + 'static>() {
            is_send_sync::<super::ConnectTimeout<T>>();
        }

        #[allow(unused)]
        fn is_send_sync<T: Send + Sync>() {}

        #[tokio::test]
        async fn connect_timeout_works() {
            let inner = NeverConnected::new();
            let timeout = timeout::Settings::new().with_connect_timeout(Duration::from_secs(1));
            let mut hyper = HyperAdapter::builder()
                .timeout(&timeout)
                .sleep_impl(TokioSleep::new())
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
                .expect_err("timeout");
            assert!(resp.is_timeout(), "{:?}", resp);
            assert_eq!(
                format!("{}", resp),
                "timeout: error trying to connect: timed out after 1s"
            );
            assert_elapsed!(now, Duration::from_secs(1));
        }

        #[tokio::test]
        async fn http_timeout_works() {
            let inner = NeverReplies::new();
            let timeout = timeout::Settings::new()
                .with_connect_timeout(Duration::from_secs(1))
                .with_read_timeout(Duration::from_secs(2));
            let mut hyper = HyperAdapter::builder()
                .timeout(&timeout)
                .sleep_impl(TokioSleep::new())
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
                .expect_err("timeout");
            assert!(resp.is_timeout(), "{:?}", resp);
            assert_elapsed!(now, Duration::from_secs(2));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::hyper_impls::HyperAdapter;
    use http::Uri;
    use hyper::client::connect::{Connected, Connection};

    use aws_smithy_http::body::SdkBody;
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
        let mut adapter = HyperAdapter::builder().build(connector);
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
            Poll::Ready(Err(std::io::Error::new(
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
        T: Clone + hyper::client::connect::Connection,
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
