/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

//! Type-erased variants of [`Client`] and friends.

// These types are technically public in that they're reachable from the public trait impls on
// DynMiddleware, but no-one should ever look at them or use them.
#[doc(hidden)]
pub mod boxclone;
use boxclone::*;

use crate::{bounds, retry, Client};
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::result::ConnectorError;
use std::fmt;
use tower::{Layer, Service, ServiceExt};

/// A [`Client`] whose connector and middleware types have been erased.
///
/// Mainly useful if you need to name `R` in a type-erased client. If you do not, you can instead
/// just use `Client` with no type parameters, which ends up being the same type.
pub type DynClient<R = retry::Standard> = Client<DynConnector, DynMiddleware<DynConnector>, R>;

impl<C, M, R> Client<C, M, R>
where
    C: bounds::SmithyConnector,
    M: bounds::SmithyMiddleware<C> + Send + Sync + 'static,
    R: retry::NewRequestPolicy,
{
    /// Erase the middleware type from the client type signature.
    ///
    /// This makes the final client type easier to name, at the cost of a marginal increase in
    /// runtime performance. See [`DynMiddleware`] for details.
    ///
    /// In practice, you'll use this method once you've constructed a client to your liking:
    ///
    /// ```no_run
    /// # #[cfg(feature = "https")]
    /// # fn not_main() {
    /// use aws_smithy_client::{Builder, Client};
    /// struct MyClient {
    ///     client: Client<aws_smithy_client::conns::Https>,
    /// }
    ///
    /// let client = Builder::new()
    ///     .https()
    ///     .middleware(tower::layer::util::Identity::new())
    ///     .build();
    /// let client = MyClient { client: client.into_dyn_middleware() };
    /// # client.client.check();
    /// # }
    pub fn into_dyn_middleware(self) -> Client<C, DynMiddleware<C>, R> {
        Client {
            connector: self.connector,
            middleware: DynMiddleware::new(self.middleware),
            retry_policy: self.retry_policy,
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
        }
    }
}

impl<C, M, R> Client<C, M, R>
where
    C: bounds::SmithyConnector,
    M: bounds::SmithyMiddleware<DynConnector> + Send + Sync + 'static,
    R: retry::NewRequestPolicy,
{
    /// Erase the connector type from the client type signature.
    ///
    /// This makes the final client type easier to name, at the cost of a marginal increase in
    /// runtime performance. See [`DynConnector`] for details.
    ///
    /// In practice, you'll use this method once you've constructed a client to your liking:
    ///
    /// ```no_run
    /// # #[cfg(feature = "https")]
    /// # fn not_main() {
    /// # type MyMiddleware = aws_smithy_client::DynMiddleware<aws_smithy_client::DynConnector>;
    /// use aws_smithy_client::{Builder, Client};
    /// struct MyClient {
    ///     client: Client<aws_smithy_client::DynConnector, MyMiddleware>,
    /// }
    ///
    /// let client = Builder::new()
    ///     .https()
    ///     .middleware(tower::layer::util::Identity::new())
    ///     .build();
    /// let client = MyClient { client: client.into_dyn_connector() };
    /// # client.client.check();
    /// # }
    pub fn into_dyn_connector(self) -> Client<DynConnector, M, R> {
        Client {
            connector: DynConnector::new(self.connector),
            middleware: self.middleware,
            retry_policy: self.retry_policy,
            timeout_config: self.timeout_config,
            sleep_impl: self.sleep_impl,
        }
    }

    /// Erase the connector and middleware types from the client type signature.
    ///
    /// This makes the final client type easier to name, at the cost of a marginal increase in
    /// runtime performance. See [`DynConnector`] and [`DynMiddleware`] for details.
    ///
    /// Note that if you're using the standard retry mechanism, [`retry::Standard`], `DynClient<R>`
    /// is equivalent to `Client` with no type arguments.
    ///
    /// In practice, you'll use this method once you've constructed a client to your liking:
    ///
    /// ```no_run
    /// # #[cfg(feature = "https")]
    /// # fn not_main() {
    /// use aws_smithy_client::{Builder, Client};
    /// struct MyClient {
    ///     client: aws_smithy_client::Client,
    /// }
    ///
    /// let client = Builder::new()
    ///     .https()
    ///     .middleware(tower::layer::util::Identity::new())
    ///     .build();
    /// let client = MyClient { client: client.into_dyn() };
    /// # client.client.check();
    /// # }
    pub fn into_dyn(self) -> DynClient<R> {
        self.into_dyn_connector().into_dyn_middleware()
    }
}

/// A Smithy connector that uses dynamic dispatch.
///
/// This type allows you to pay a small runtime cost to avoid having to name the exact connector
/// you're using anywhere you want to hold a [`Client`]. Specifically, this will use `Box` to
/// enable dynamic dispatch for every request that goes through the connector, which increases
/// memory pressure and suffers an additional vtable indirection for each request, but is unlikely
/// to matter in all but the highest-performance settings.
#[non_exhaustive]
#[derive(Clone)]
pub struct DynConnector(
    BoxCloneService<http::Request<SdkBody>, http::Response<SdkBody>, ConnectorError>,
);

impl fmt::Debug for DynConnector {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("DynConnector").finish()
    }
}

impl DynConnector {
    /// Construct a new dynamically-dispatched Smithy middleware.
    pub fn new<E, C>(connector: C) -> Self
    where
        C: bounds::SmithyConnector<Error = E> + Send + 'static,
        E: Into<ConnectorError>,
    {
        Self(BoxCloneService::new(connector.map_err(|e| e.into())))
    }
}

impl Service<http::Request<SdkBody>> for DynConnector {
    type Response = http::Response<SdkBody>;
    type Error = ConnectorError;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<SdkBody>) -> Self::Future {
        self.0.call(req)
    }
}

/// A Smithy middleware that uses dynamic dispatch.
///
/// This type allows you to pay a small runtime cost to avoid having to name the exact middleware
/// you're using anywhere you want to hold a [`Client`]. Specifically, this will use `Box` to
/// enable dynamic dispatch for every request that goes through the middleware, which increases
/// memory pressure and suffers an additional vtable indirection for each request, but is unlikely
/// to matter in all but the highest-performance settings.
#[non_exhaustive]
pub struct DynMiddleware<C>(
    ArcCloneLayer<
        aws_smithy_http_tower::dispatch::DispatchService<C>,
        aws_smithy_http::operation::Request,
        aws_smithy_http::operation::Response,
        aws_smithy_http_tower::SendOperationError,
    >,
);

impl<C> Clone for DynMiddleware<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C> fmt::Debug for DynMiddleware<C> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("DynMiddleware").finish()
    }
}

impl<C> DynMiddleware<C> {
    /// Construct a new dynamically-dispatched Smithy middleware.
    pub fn new<M: bounds::SmithyMiddleware<C> + Send + Sync + 'static>(middleware: M) -> Self {
        Self(ArcCloneLayer::new(middleware))
    }
}

impl<C> Layer<aws_smithy_http_tower::dispatch::DispatchService<C>> for DynMiddleware<C> {
    type Service = BoxCloneService<
        aws_smithy_http::operation::Request,
        aws_smithy_http::operation::Response,
        aws_smithy_http_tower::SendOperationError,
    >;

    fn layer(&self, inner: aws_smithy_http_tower::dispatch::DispatchService<C>) -> Self::Service {
        self.0.layer(inner)
    }
}
