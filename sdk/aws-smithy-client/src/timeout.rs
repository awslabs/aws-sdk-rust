/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Timeout Configuration
//!
//! While timeout configuration is unstable, this module is in aws-smithy-client.
//!
//! As timeout and HTTP configuration stabilizes, this will move to aws-types and become a part of
//! HttpSettings.
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::SdkError;
use aws_smithy_async::future::timeout::Timeout;
use aws_smithy_async::rt::sleep::{AsyncSleep, Sleep};
use aws_smithy_http::operation::Operation;
use aws_smithy_types::timeout::TimeoutConfig;
use pin_project_lite::pin_project;
use tower::Layer;

/// Timeout Configuration
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct Settings {
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    _tls_negotiation_timeout: Option<Duration>,
}

impl Settings {
    /// Create a new timeout configuration with no timeouts set
    pub fn new() -> Self {
        Default::default()
    }

    /// The configured TCP-connect timeout
    pub fn connect(&self) -> Option<Duration> {
        self.connect_timeout
    }

    /// The configured HTTP-read timeout
    pub fn read(&self) -> Option<Duration> {
        self.read_timeout
    }

    /// Sets the connect timeout
    pub fn with_connect_timeout(self, connect_timeout: Duration) -> Self {
        Self {
            connect_timeout: Some(connect_timeout),
            ..self
        }
    }

    /// Sets the read timeout
    pub fn with_read_timeout(self, read_timeout: Duration) -> Self {
        Self {
            read_timeout: Some(read_timeout),
            ..self
        }
    }
}

#[derive(Debug)]
struct RequestTimeoutError {
    kind: &'static str,
    duration: Duration,
}

impl RequestTimeoutError {
    pub fn new_boxed(kind: &'static str, duration: Duration) -> Box<Self> {
        Box::new(Self { kind, duration })
    }
}

impl std::fmt::Display for RequestTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} timeout occurred after {:?}",
            self.kind, self.duration
        )
    }
}

impl std::error::Error for RequestTimeoutError {}

#[derive(Clone, Debug)]
/// A struct containing everything needed to create a new [`TimeoutService`]
pub struct TimeoutServiceParams {
    /// The duration of timeouts created from these params
    duration: Duration,
    /// The kind of timeouts created from these params
    kind: &'static str,
    /// The AsyncSleep impl that will be used to create time-limited futures
    async_sleep: Arc<dyn AsyncSleep>,
}

#[derive(Clone, Debug, Default)]
/// A struct of structs containing everything needed to create new [`TimeoutService`]s
pub struct ClientTimeoutParams {
    /// Params used to create a new API call [`TimeoutService`]
    pub(crate) api_call: Option<TimeoutServiceParams>,
    /// Params used to create a new API call attempt [`TimeoutService`]
    pub(crate) api_call_attempt: Option<TimeoutServiceParams>,
}

/// Convert a [`TimeoutConfig`] into an [`ClientTimeoutParams`] in order to create the set of
/// [`TimeoutService`]s needed by a [`crate::Client`]
pub fn generate_timeout_service_params_from_timeout_config(
    timeout_config: &TimeoutConfig,
    async_sleep: Option<Arc<dyn AsyncSleep>>,
) -> ClientTimeoutParams {
    if let Some(async_sleep) = async_sleep {
        ClientTimeoutParams {
            api_call: timeout_config
                .api_call_timeout()
                .map(|duration| TimeoutServiceParams {
                    duration,
                    kind: "API call (all attempts including retries)",
                    async_sleep: async_sleep.clone(),
                }),
            api_call_attempt: timeout_config.api_call_attempt_timeout().map(|duration| {
                TimeoutServiceParams {
                    duration,
                    kind: "API call (single attempt)",
                    async_sleep: async_sleep.clone(),
                }
            }),
        }
    } else {
        Default::default()
    }
}

/// A service that wraps another service, adding the ability to set a timeout for requests
/// handled by the inner service.
#[derive(Clone, Debug)]
pub struct TimeoutService<S> {
    inner: S,
    params: Option<TimeoutServiceParams>,
}

impl<S> TimeoutService<S> {
    /// Create a new `TimeoutService` that will timeout after the duration specified in `params` elapses
    pub fn new(inner: S, params: Option<TimeoutServiceParams>) -> Self {
        Self { inner, params }
    }

    /// Create a new `TimeoutService` that will never timeout
    pub fn no_timeout(inner: S) -> Self {
        Self {
            inner,
            params: None,
        }
    }
}

/// A layer that wraps services in a timeout service
#[non_exhaustive]
#[derive(Debug)]
pub struct TimeoutLayer(Option<TimeoutServiceParams>);

impl TimeoutLayer {
    /// Create a new `TimeoutLayer`
    pub fn new(params: Option<TimeoutServiceParams>) -> Self {
        TimeoutLayer(params)
    }
}

impl<S> Layer<S> for TimeoutLayer {
    type Service = TimeoutService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimeoutService {
            inner,
            params: self.0.clone(),
        }
    }
}

pin_project! {
    #[non_exhaustive]
    #[must_use = "futures do nothing unless you `.await` or poll them"]
    // This allow is needed because otherwise Clippy will get mad we didn't document the
    // generated TimeoutServiceFutureProj
    #[allow(missing_docs)]
    #[project = TimeoutServiceFutureProj]
    /// A future generated by a [`TimeoutService`] that may or may not have a timeout depending on
    /// whether or not one was set. Because `TimeoutService` can be used at multiple levels of the
    /// service stack, a `kind` can be set so that when a timeout occurs, you can know which kind of
    /// timeout it was.
    pub enum TimeoutServiceFuture<F> {
        /// A wrapper around an inner future that will output an [`SdkError`] if it runs longer than
        /// the given duration
        Timeout {
            #[pin]
            future: Timeout<F, Sleep>,
            kind: &'static str,
            duration: Duration,
        },
        /// A thin wrapper around an inner future that will never time out
        NoTimeout {
            #[pin]
            future: F
        }
    }
}

impl<F> TimeoutServiceFuture<F> {
    /// Given a `future`, an implementor of `AsyncSleep`, a `kind` for this timeout, and a `duration`,
    /// wrap the `future` inside a [`Timeout`] future and create a new [`TimeoutServiceFuture`] that
    /// will output an [`SdkError`] if `future` doesn't complete before `duration` has elapsed.
    pub fn new(future: F, params: &TimeoutServiceParams) -> Self {
        Self::Timeout {
            future: Timeout::new(future, params.async_sleep.sleep(params.duration)),
            kind: params.kind,
            duration: params.duration,
        }
    }

    /// Create a [`TimeoutServiceFuture`] that will never time out.
    pub fn no_timeout(future: F) -> Self {
        Self::NoTimeout { future }
    }
}

impl<InnerFuture, T, E> Future for TimeoutServiceFuture<InnerFuture>
where
    InnerFuture: Future<Output = Result<T, SdkError<E>>>,
{
    type Output = Result<T, SdkError<E>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let (future, kind, duration) = match self.project() {
            TimeoutServiceFutureProj::NoTimeout { future } => return future.poll(cx),
            TimeoutServiceFutureProj::Timeout {
                future,
                kind,
                duration,
            } => (future, kind, duration),
        };
        match future.poll(cx) {
            Poll::Ready(Ok(response)) => Poll::Ready(response),
            Poll::Ready(Err(_timeout)) => Poll::Ready(Err(SdkError::TimeoutError(
                RequestTimeoutError::new_boxed(kind, *duration),
            ))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<H, R, InnerService, E> tower::Service<Operation<H, R>> for TimeoutService<InnerService>
where
    InnerService: tower::Service<Operation<H, R>, Error = SdkError<E>>,
{
    type Response = InnerService::Response;
    type Error = aws_smithy_http::result::SdkError<E>;
    type Future = TimeoutServiceFuture<InnerService::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Operation<H, R>) -> Self::Future {
        let future = self.inner.call(req);

        if let Some(params) = &self.params {
            Self::Future::new(future, params)
        } else {
            Self::Future::no_timeout(future)
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use std::time::Duration;

    use crate::never::NeverService;
    use crate::timeout::generate_timeout_service_params_from_timeout_config;
    use crate::{SdkError, TimeoutLayer};
    use aws_smithy_async::assert_elapsed;
    use aws_smithy_async::rt::sleep::{AsyncSleep, TokioSleep};
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_http::operation::{Operation, Request};
    use aws_smithy_types::timeout::TimeoutConfig;
    use tower::{Service, ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn test_timeout_service_ends_request_that_never_completes() {
        let req = Request::new(http::Request::new(SdkBody::empty()));
        let op = Operation::new(req, ());
        let never_service: NeverService<_, (), _> = NeverService::new();
        let timeout_config =
            TimeoutConfig::new().with_api_call_timeout(Some(Duration::from_secs_f32(0.25)));
        let sleep_impl: Option<Arc<dyn AsyncSleep>> = Some(Arc::new(TokioSleep::new()));
        let timeout_service_params =
            generate_timeout_service_params_from_timeout_config(&timeout_config, sleep_impl);
        let mut svc = ServiceBuilder::new()
            .layer(TimeoutLayer::new(timeout_service_params.api_call))
            .service(never_service);

        let now = tokio::time::Instant::now();
        tokio::time::pause();

        let err: SdkError<Box<dyn std::error::Error + 'static>> =
            svc.ready().await.unwrap().call(op).await.unwrap_err();

        assert_eq!(format!("{:?}", err), "TimeoutError(RequestTimeoutError { kind: \"API call (all attempts including retries)\", duration: 250ms })");
        assert_elapsed!(now, Duration::from_secs_f32(0.25));
    }
}
