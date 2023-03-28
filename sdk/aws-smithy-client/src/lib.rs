/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! A Hyper-based Smithy service client.
//!
//! | Feature           | Description |
//! |-------------------|-------------|
//! | `event-stream`    | Provides Sender/Receiver implementations for Event Stream codegen. |
//! | `rt-tokio`        | Run async code with the `tokio` runtime |
//! | `test-util`       | Include various testing utils |
//! | `native-tls`      | Use `native-tls` as the HTTP client's TLS implementation |
//! | `rustls`          | Use `rustls` as the HTTP client's TLS implementation |
//! | `client-hyper`    | Use `hyper` to handle HTTP requests |

#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rustdoc::all,
    rust_2018_idioms
)]

pub mod bounds;
pub mod erase;
pub mod http_connector;
pub mod never;
pub mod retry;
pub mod timeout;

// https://github.com/rust-lang/rust/issues/72081
#[allow(rustdoc::private_doc_tests)]
mod builder;
pub use builder::Builder;

#[cfg(feature = "test-util")]
pub mod dvr;
#[cfg(feature = "test-util")]
pub mod test_connection;

#[cfg(feature = "client-hyper")]
pub mod conns;
#[cfg(feature = "client-hyper")]
pub mod hyper_ext;

// The types in this module are only used to write the bounds in [`Client::check`]. Customers will
// not need them. But the module and its types must be public so that we can call `check` from
// doc-tests.
#[doc(hidden)]
pub mod static_tests;

use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_http::operation::Operation;
use aws_smithy_http::response::ParseHttpResponse;
pub use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http_tower::dispatch::DispatchLayer;
use aws_smithy_http_tower::parse_response::ParseResponseLayer;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::retry::ProvideErrorKind;
use aws_smithy_types::timeout::OperationTimeoutConfig;
use std::sync::Arc;
use timeout::ClientTimeoutParams;
pub use timeout::TimeoutLayer;
use tower::{Service, ServiceBuilder, ServiceExt};
use tracing::{debug_span, field, field::display, Instrument};

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
/// request. The [`tower::Service`](Service) that the middleware produces must accept requests of the type
/// [`aws_smithy_http::operation::Request`] and return responses of the type
/// [`http::Response<SdkBody>`], most likely by modifying the provided request in place, passing it
/// to the inner service, and then ultimately returning the inner service's response.
///
/// With the `hyper` feature enabled, you can construct a `Client` directly from a
/// [`hyper::Client`] using [`hyper_ext::Adapter::builder`]. You can also enable the `rustls` or `native-tls`
/// features to construct a Client against a standard HTTPS endpoint using [`Builder::rustls_connector`] and
/// `Builder::native_tls_connector` respectively.
#[derive(Debug)]
pub struct Client<
    Connector = erase::DynConnector,
    Middleware = erase::DynMiddleware<Connector>,
    RetryPolicy = retry::Standard,
> {
    connector: Connector,
    middleware: Middleware,
    retry_policy: RetryPolicy,
    operation_timeout_config: OperationTimeoutConfig,
    sleep_impl: Option<Arc<dyn AsyncSleep>>,
}

impl Client<(), (), ()> {
    /// Returns a client builder
    pub fn builder() -> Builder {
        Builder::new()
    }
}

// Quick-create for people who just want "the default".
impl<C, M> Client<C, M>
where
    M: Default,
{
    /// Create a Smithy client from the given `connector`, a middleware default, the
    /// [standard retry policy](retry::Standard), and the
    /// [`default_async_sleep`](aws_smithy_async::rt::sleep::default_async_sleep) sleep implementation.
    pub fn new(connector: C) -> Self {
        Builder::new()
            .connector(connector)
            .middleware(M::default())
            .build()
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
    pub async fn call<O, T, E, Retry>(&self, op: Operation<O, Retry>) -> Result<T, SdkError<E>>
    where
        O: Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        Retry: Send + Sync,
        R::Policy: bounds::SmithyRetryPolicy<O, T, E, Retry>,
        bounds::Parsed<<M as bounds::SmithyMiddleware<C>>::Service, O, Retry>:
            Service<Operation<O, Retry>, Response = SdkSuccess<T>, Error = SdkError<E>> + Clone,
    {
        self.call_raw(op).await.map(|res| res.parsed)
    }

    /// Dispatch this request to the network
    ///
    /// The returned result contains the raw HTTP response which can be useful for debugging or
    /// implementing unsupported features.
    pub async fn call_raw<O, T, E, Retry>(
        &self,
        op: Operation<O, Retry>,
    ) -> Result<SdkSuccess<T>, SdkError<E>>
    where
        O: Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
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

        let timeout_params =
            ClientTimeoutParams::new(&self.operation_timeout_config, self.sleep_impl.clone());

        let svc = ServiceBuilder::new()
            .layer(TimeoutLayer::new(timeout_params.operation_timeout))
            .retry(
                self.retry_policy
                    .new_request_policy(self.sleep_impl.clone()),
            )
            .layer(TimeoutLayer::new(timeout_params.operation_attempt_timeout))
            .layer(ParseResponseLayer::<O, Retry>::new())
            // These layers can be considered as occurring in order. That is, first invoke the
            // customer-provided middleware, then dispatch dispatch over the wire.
            .layer(&self.middleware)
            .layer(DispatchLayer::new())
            .service(connector);

        // send_operation records the full request-response lifecycle.
        // NOTE: For operations that stream output, only the setup is captured in this span.
        let span = debug_span!(
            "send_operation",
            operation = field::Empty,
            service = field::Empty,
            status = field::Empty,
            message = field::Empty
        );
        let (mut req, parts) = op.into_request_response();
        if let Some(metadata) = &parts.metadata {
            span.record("operation", &metadata.name());
            span.record("service", &metadata.service());
            // This will clone two `Cow::<&'static str>::Borrow`s in the vast majority of cases
            req.properties_mut().insert(metadata.clone());
        }
        let op = Operation::from_parts(req, parts);

        let result = async move { check_send_sync(svc).ready().await?.call(op).await }
            .instrument(span.clone())
            .await;
        match &result {
            Ok(_) => {
                span.record("status", &"ok");
            }
            Err(err) => {
                span.record(
                    "status",
                    &match err {
                        SdkError::ConstructionFailure(_) => "construction_failure",
                        SdkError::DispatchFailure(_) => "dispatch_failure",
                        SdkError::ResponseError(_) => "response_error",
                        SdkError::ServiceError(_) => "service_error",
                        SdkError::TimeoutError(_) => "timeout_error",
                        _ => "error",
                    },
                )
                .record("message", &display(DisplayErrorContext(err)));
            }
        }
        result
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
