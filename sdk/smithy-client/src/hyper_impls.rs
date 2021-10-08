/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use std::sync::Arc;

use http::Uri;
use hyper::client::connect::Connection;

use tokio::io::{AsyncRead, AsyncWrite};
use tower::Service;

use smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
use smithy_http::body::SdkBody;
pub use smithy_http::result::{SdkError, SdkSuccess};

use crate::hyper_impls::timeout_middleware::{ConnectTimeout, HttpReadTimeout};
use crate::{timeout, BoxError, Builder as ClientBuilder};

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
    type Error = BoxError;

    #[allow(clippy::type_complexity)]
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        let fut = self.0.call(req);
        Box::pin(async move { Ok(fut.await?.map(SdkBody::from)) })
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
    /// [`smithy_async::rt::sleep::default_async_sleep`].
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

    use smithy_async::future;
    use smithy_async::future::timeout::Timeout;
    use smithy_async::rt::sleep::AsyncSleep;
    use smithy_async::rt::sleep::Sleep;

    use crate::BoxError;

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
                timeout: Timeout<F, Sleep>
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
            let timeout_future = match self.project() {
                MaybeTimeoutFutureProj::NoTimeout { future } => {
                    return future.poll(cx).map_err(|err| err.into())
                }
                MaybeTimeoutFutureProj::Timeout { timeout } => timeout,
            };
            match timeout_future.poll(cx) {
                Poll::Ready(Ok(response)) => Poll::Ready(response.map_err(|err| err.into())),
                Poll::Ready(Err(timeout)) => Poll::Ready(Err(timeout.into())),
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
        I: tower::Service<http::Request<B>>,
        I::Error: Into<BoxError>,
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
        use smithy_async::rt::sleep::TokioSleep;
        use smithy_http::body::SdkBody;
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
            assert_eq!(format!("{}", resp), "error trying to connect: timed out");
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
            let _resp = hyper
                .call(
                    http::Request::builder()
                        .uri("http://foo.com")
                        .body(SdkBody::empty())
                        .unwrap(),
                )
                .await
                .expect_err("timeout");
            assert_elapsed!(now, Duration::from_secs(2));
        }
    }
}
