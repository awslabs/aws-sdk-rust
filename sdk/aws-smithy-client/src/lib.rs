/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */
//! A Hyper-based Smithy service client.
#![warn(
    missing_debug_implementations,
    missing_docs,
    rustdoc::all,
    rust_2018_idioms
)]

pub mod bounds;
pub mod erase;
pub mod retry;

// https://github.com/rust-lang/rust/issues/72081
#[allow(rustdoc::private_doc_tests)]
mod builder;
pub use builder::Builder;

#[cfg(feature = "test-util")]
pub mod dvr;
#[cfg(feature = "test-util")]
pub mod test_connection;

#[cfg(feature = "hyper")]
pub mod hyper_ext;

// The types in this module are only used to write the bounds in [`Client::check`]. Customers will
// not need them. But the module and its types must be public so that we can call `check` from
// doc-tests.
#[doc(hidden)]
pub mod static_tests;

#[cfg(feature = "hyper")]
pub mod never;
pub mod timeout;
pub use timeout::TimeoutLayer;

/// Type aliases for standard connection types.
#[cfg(feature = "hyper")]
#[allow(missing_docs)]
pub mod conns {

    #[cfg(feature = "rustls")]
    pub type Https = hyper_rustls::HttpsConnector<hyper::client::HttpConnector>;

    // Creating a `with_native_roots` HTTP client takes 300ms on OS X. Cache this so that we
    // don't need to repeatedly incur that cost.
    #[cfg(feature = "rustls")]
    lazy_static::lazy_static! {
        static ref HTTPS_NATIVE_ROOTS: Https = {
            hyper_rustls::HttpsConnector::with_native_roots()
        };
    }

    #[cfg(feature = "rustls")]
    pub fn https() -> Https {
        HTTPS_NATIVE_ROOTS.clone()
    }

    #[cfg(feature = "native-tls")]
    pub fn native_tls() -> NativeTls {
        hyper_tls::HttpsConnector::new()
    }

    #[cfg(feature = "native-tls")]
    pub type NativeTls = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

    #[cfg(feature = "rustls")]
    pub type Rustls =
        crate::hyper_ext::Adapter<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>;
}

use std::error::Error;
use std::sync::Arc;
use tower::{Layer, Service, ServiceBuilder, ServiceExt};

use crate::timeout::generate_timeout_service_params_from_timeout_config;
use aws_smithy_async::rt::sleep::{default_async_sleep, AsyncSleep};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::operation::Operation;
use aws_smithy_http::response::ParseHttpResponse;
pub use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http::retry::ClassifyResponse;
use aws_smithy_http_tower::dispatch::DispatchLayer;
use aws_smithy_http_tower::parse_response::ParseResponseLayer;
use aws_smithy_types::retry::ProvideErrorKind;
use aws_smithy_types::timeout::TimeoutConfig;

/// Smithy service client.
///
/// The service client is customizable in a number of ways (see [`Builder`]), but most customers
/// can stick with the standard constructor provided by [`Client::new`]. It takes only a single
/// argument, which is the middleware that fills out the [`http::Request`] for each higher-level
/// operation so that it can ultimately be sent to the remote host. The middleware is responsible
/// for filling in any request parameters that aren't specified by the Smithy protocol definition,
/// such as those used for routing (like the URL), authentication, and authorization.
///
/// The middleware takes the form of a [`tower::Layer`] that wraps the actual connection for each
/// request. The [`tower::Service`] that the middleware produces must accept requests of the type
/// [`aws_smithy_http::operation::Request`] and return responses of the type
/// [`http::Response<SdkBody>`], most likely by modifying the provided request in place, passing it
/// to the inner service, and then ultimately returning the inner service's response.
///
/// With the `hyper` feature enabled, you can construct a `Client` directly from a
/// [`hyper::Client`] using [`hyper_ext::Adapter::builder`]. You can also enable the `rustls` or `native-tls`
/// features to construct a Client against a standard HTTPS endpoint using [`Builder::rustls`] and
/// `Builder::native_tls` respectively.
#[derive(Debug)]
pub struct Client<
    Connector = erase::DynConnector,
    Middleware = erase::DynMiddleware<Connector>,
    RetryPolicy = retry::Standard,
> {
    connector: Connector,
    middleware: Middleware,
    retry_policy: RetryPolicy,
    timeout_config: TimeoutConfig,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
}

// Quick-create for people who just want "the default".
impl<C, M> Client<C, M>
where
    M: Default,
{
    /// Create a Smithy client that the given connector, a middleware default, the [standard
    /// retry policy](crate::retry::Standard), and the [`default_async_sleep`] sleep implementation.
    pub fn new(connector: C) -> Self {
        let mut client = Builder::new()
            .connector(connector)
            .middleware(M::default())
            .build();

        client.set_sleep_impl(default_async_sleep());

        client
    }
}

impl<C, M> Client<C, M> {
    /// Set the standard retry policy's configuration.
    pub fn set_retry_config(&mut self, config: retry::Config) {
        self.retry_policy.with_config(config);
    }

    /// Adjust a standard retry client with the given policy configuration.
    pub fn with_retry_config(mut self, config: retry::Config) -> Self {
        self.set_retry_config(config);
        self
    }

    /// Set the client's timeout configuration.
    pub fn set_timeout_config(&mut self, config: TimeoutConfig) {
        self.timeout_config = config;
    }

    /// Set the client's timeout configuration.
    pub fn with_timeout_config(mut self, config: TimeoutConfig) -> Self {
        self.set_timeout_config(config);
        self
    }

    /// Set the [`AsyncSleep`] function that the client will use to create things like timeout futures.
    pub fn set_sleep_impl(&mut self, sleep_impl: Option<Arc<dyn AsyncSleep>>) {
        self.sleep_impl = sleep_impl;
    }

    /// Set the [`AsyncSleep`] function that the client will use to create things like timeout futures.
    pub fn with_sleep_impl(mut self, sleep_impl: Arc<dyn AsyncSleep>) -> Self {
        self.set_sleep_impl(Some(sleep_impl));
        self
    }
}

fn check_send_sync<T: Send + Sync>(t: T) -> T {
    t
}

impl<C, M, R> Client<C, M, R>
where
    C: bounds::SmithyConnector,
    M: bounds::SmithyMiddleware<C>,
    R: retry::NewRequestPolicy,
{
    /// Dispatch this request to the network
    ///
    /// For ergonomics, this does not include the raw response for successful responses. To
    /// access the raw response use `call_raw`.
    pub async fn call<O, T, E, Retry>(&self, input: Operation<O, Retry>) -> Result<T, SdkError<E>>
    where
        O: Send + Sync,
        Retry: Send + Sync,
        R::Policy: bounds::SmithyRetryPolicy<O, T, E, Retry>,
        bounds::Parsed<<M as bounds::SmithyMiddleware<C>>::Service, O, Retry>:
            Service<Operation<O, Retry>, Response = SdkSuccess<T>, Error = SdkError<E>> + Clone,
    {
        self.call_raw(input).await.map(|res| res.parsed)
    }

    /// Dispatch this request to the network
    ///
    /// The returned result contains the raw HTTP response which can be useful for debugging or
    /// implementing unsupported features.
    pub async fn call_raw<O, T, E, Retry>(
        &self,
        input: Operation<O, Retry>,
    ) -> Result<SdkSuccess<T>, SdkError<E>>
    where
        O: Send + Sync,
        Retry: Send + Sync,
        R::Policy: bounds::SmithyRetryPolicy<O, T, E, Retry>,
        // This bound is not _technically_ inferred by all the previous bounds, but in practice it
        // is because _we_ know that there is only implementation of Service for Parsed
        // (ParsedResponseService), and it will apply as long as the bounds on C, M, and R hold,
        // and will produce (as expected) Response = SdkSuccess<T>, Error = SdkError<E>. But Rust
        // doesn't know that -- there _could_ theoretically be other implementations of Service for
        // Parsed that don't return those same types. So, we must give the bound.
        bounds::Parsed<<M as bounds::SmithyMiddleware<C>>::Service, O, Retry>:
            Service<Operation<O, Retry>, Response = SdkSuccess<T>, Error = SdkError<E>> + Clone,
    {
        let connector = self.connector.clone();

        let timeout_servic_params = generate_timeout_service_params_from_timeout_config(
            &self.timeout_config,
            self.sleep_impl.clone(),
        );

        let svc = ServiceBuilder::new()
            .layer(TimeoutLayer::new(timeout_servic_params.api_call))
            .retry(self.retry_policy.new_request_policy())
            .layer(TimeoutLayer::new(timeout_servic_params.api_call_attempt))
            .layer(ParseResponseLayer::<O, Retry>::new())
            // These layers can be considered as occurring in order. That is, first invoke the
            // customer-provided middleware, then dispatch dispatch over the wire.
            .layer(&self.middleware)
            .layer(DispatchLayer::new())
            .service(connector);

        check_send_sync(svc).ready().await?.call(input).await
    }

    /// Statically check the validity of a `Client` without a request to send.
    ///
    /// This will make sure that all the bounds hold that would be required by `call` and
    /// `call_raw` (modulo those that relate to the specific `Operation` type). Comes in handy to
    /// ensure (statically) that all the various constructors actually produce "useful" types.
    #[doc(hidden)]
    pub fn check(&self)
    where
        R::Policy: tower::retry::Policy<
                static_tests::ValidTestOperation,
                SdkSuccess<()>,
                SdkError<static_tests::TestOperationError>,
            > + Clone,
    {
        let _ = |o: static_tests::ValidTestOperation| {
            let _ = self.call_raw(o);
        };
    }
}
