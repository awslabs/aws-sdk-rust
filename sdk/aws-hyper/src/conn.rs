/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::BoxError;
use http::Request;
use hyper::client::ResponseFuture;
use hyper::Response;
use smithy_http::body::SdkBody;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

#[derive(Clone)]
pub struct Standard(Connector);

impl Standard {
    /// An https connection
    ///
    /// If the `rustls` feature is enabled, this will use `rustls`.
    /// If the ONLY the `native-tls` feature is enabled, this will use `native-tls`.
    /// If both features are enabled, this will use `rustls`
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    pub fn https() -> Self {
        #[cfg(feature = "rustls")]
        {
            Self::rustls()
        }

        // If we are compiling this function & rustls is not enabled, then native-tls MUST be enabled
        #[cfg(not(feature = "rustls"))]
        {
            Self::native_tls()
        }
    }

    #[cfg(feature = "rustls")]
    pub fn rustls() -> Self {
        let https = hyper_rustls::HttpsConnector::with_native_roots();
        let client = hyper::Client::builder().build::<_, SdkBody>(https);
        Self(Connector::RustlsHttps(client))
    }

    #[cfg(feature = "native-tls")]
    pub fn native_tls() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, SdkBody>(https);
        Self(Connector::NativeHttps(client))
    }

    /// A connection based on the provided `impl HttpService`
    ///
    /// Generally, [`Standard::https()`](Standard::https) should be used. This constructor is intended to support
    /// using things like [`TestConnection`](crate::test_connection::TestConnection) or alternative
    /// http implementations.
    pub fn new(connector: impl HttpService + 'static) -> Self {
        Self(Connector::Dyn(Box::new(connector)))
    }
}

/// An Http connection type for most use cases
///
/// This supports three options:
/// 1. HTTPS
/// 2. A `TestConnection`
/// 3. Any implementation of the `HttpService` trait
///
/// This is designed to be used with [`aws_hyper::Client`](crate::Client) as a connector.
#[derive(Clone)]
enum Connector {
    /// An Https Connection
    ///
    /// This is the correct connection for use cases talking to real AWS services.
    #[cfg(feature = "native-tls")]
    NativeHttps(hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>, SdkBody>),

    /// An Https Connection
    ///
    /// This is the correct connection for use cases talking to real AWS services.
    #[cfg(feature = "rustls")]
    RustlsHttps(hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, SdkBody>),

    /// A generic escape hatch
    ///
    /// This enables using any implementation of the HttpService trait. This allows using a totally
    /// separate HTTP stack or your own custom `TestConnection`.
    Dyn(Box<dyn HttpService>),
}

impl Clone for Box<dyn HttpService> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait HttpService: Send + Sync {
    /// Return whether this service is ready to accept a request
    ///
    /// See [`Service::poll_ready`](tower::Service::poll_ready)
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>>;

    /// Call this service and return a response
    ///
    /// See [`Service::call`](tower::Service::call)
    fn call(
        &mut self,
        req: http::Request<SdkBody>,
    ) -> Pin<Box<dyn Future<Output = Result<http::Response<SdkBody>, BoxError>> + Send>>;

    /// Return a Boxed-clone of this service
    ///
    /// `aws_hyper::Client` will clone the inner service for each request so this should be a cheap
    /// clone operation.
    fn clone_box(&self) -> Box<dyn HttpService>;
}

/// Reverse implementation: If you have a correctly shaped tower service, it _is_ an `HttpService`
///
/// This is to facilitate ease of use for people using `Standard::Dyn`
impl<S> HttpService for S
where
    S: Service<http::Request<SdkBody>, Response = http::Response<SdkBody>>
        + Send
        + Sync
        + Clone
        + 'static,
    S::Error: Into<BoxError> + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), BoxError>> {
        Service::poll_ready(self, cx).map_err(|err| err.into())
    }

    fn call(
        &mut self,
        req: Request<SdkBody>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<SdkBody>, BoxError>> + Send>> {
        let fut = Service::call(self, req);
        let fut = async move {
            fut.await
                .map(|res| res.map(SdkBody::from))
                .map_err(|err| err.into())
        };
        Box::pin(fut)
    }

    fn clone_box(&self) -> Box<dyn HttpService> {
        Box::new(self.clone())
    }
}

impl tower::Service<http::Request<SdkBody>> for Standard {
    type Response = http::Response<SdkBody>;
    type Error = BoxError;
    type Future = StandardFuture;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match &mut self.0 {
            #[cfg(feature = "native-tls")]
            Connector::NativeHttps(https) => {
                Service::poll_ready(https, cx).map_err(|err| err.into())
            }
            #[cfg(feature = "rustls")]
            Connector::RustlsHttps(https) => {
                Service::poll_ready(https, cx).map_err(|err| err.into())
            }
            Connector::Dyn(conn) => conn.poll_ready(cx),
        }
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        match &mut self.0 {
            #[cfg(feature = "native-tls")]
            Connector::NativeHttps(https) => StandardFuture::Https(Service::call(https, req)),
            #[cfg(feature = "rustls")]
            Connector::RustlsHttps(https) => StandardFuture::Https(Service::call(https, req)),
            Connector::Dyn(conn) => StandardFuture::Dyn(conn.call(req)),
        }
    }
}

/// Future returned by `Standard` when used as a tower::Service
#[pin_project::pin_project(project = FutProj)]
pub enum StandardFuture {
    Https(#[pin] ResponseFuture),
    Dyn(#[pin] Pin<Box<dyn Future<Output = Result<http::Response<SdkBody>, BoxError>> + Send>>),
}

impl Future for StandardFuture {
    type Output = Result<http::Response<SdkBody>, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            FutProj::Https(fut) => fut
                .poll(cx)
                .map(|resp| resp.map(|res| res.map(SdkBody::from)))
                .map_err(|err| err.into()),
            FutProj::Dyn(dyn_fut) => dyn_fut.poll(cx),
        }
    }
}
