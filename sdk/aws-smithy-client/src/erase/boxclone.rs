/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

// This is an adaptation of tower::util::{BoxLayer, BoxService} that includes Clone and doesn't
// include Sync.

use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use tower::layer::{layer_fn, Layer};
use tower::Service;

pub(super) struct ArcCloneLayer<In, T, U, E> {
    inner: Arc<dyn Layer<In, Service = BoxCloneService<T, U, E>> + Send + Sync>,
}

impl<In, T, U, E> Clone for ArcCloneLayer<In, T, U, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<In, T, U, E> ArcCloneLayer<In, T, U, E> {
    /// Create a new [`BoxLayer`].
    pub fn new<L>(inner_layer: L) -> Self
    where
        L: Layer<In> + Send + Sync + 'static,
        L::Service: Service<T, Response = U, Error = E> + Clone + Send + Sync + 'static,
        <L::Service as Service<T>>::Future: Send + 'static,
    {
        let layer = layer_fn(move |inner: In| {
            let out = inner_layer.layer(inner);
            BoxCloneService::new(out)
        });

        Self {
            inner: Arc::new(layer),
        }
    }
}

impl<In, T, U, E> Layer<In> for ArcCloneLayer<In, T, U, E> {
    type Service = BoxCloneService<T, U, E>;

    fn layer(&self, inner: In) -> Self::Service {
        self.inner.layer(inner)
    }
}

impl<In, T, U, E> fmt::Debug for ArcCloneLayer<In, T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("ArcCloneLayer").finish()
    }
}

trait CloneService<T>: Service<T> {
    fn clone_box(
        &self,
    ) -> Box<
        dyn CloneService<T, Response = Self::Response, Error = Self::Error, Future = Self::Future>
            + Send
            + Sync
            + 'static,
    >;
}

impl<T, Request> CloneService<Request> for T
where
    T: Service<Request> + Clone + Send + Sync + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<
        dyn CloneService<
                Request,
                Response = Self::Response,
                Error = Self::Error,
                Future = Self::Future,
            >
            + 'static
            + Send
            + Sync,
    > {
        Box::new(self.clone())
    }
}

pub type BoxFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;
pub struct BoxCloneService<T, U, E> {
    inner: Box<
        dyn CloneService<T, Response = U, Error = E, Future = BoxFuture<U, E>>
            + Send
            + Sync
            + 'static,
    >,
}

#[derive(Debug, Clone)]
struct Boxed<S> {
    inner: S,
}

impl<T, U, E> BoxCloneService<T, U, E> {
    #[allow(missing_docs)]
    pub fn new<S>(inner: S) -> Self
    where
        S: Service<T, Response = U, Error = E> + Send + Sync + 'static + Clone,
        S::Future: Send + 'static,
    {
        let inner = Box::new(Boxed { inner });
        BoxCloneService { inner }
    }
}

impl<T, U, E> Clone for BoxCloneService<T, U, E>
where
    T: 'static,
    U: 'static,
    E: 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_box(),
        }
    }
}

impl<T, U, E> Service<T> for BoxCloneService<T, U, E> {
    type Response = U;
    type Error = E;
    type Future = BoxFuture<U, E>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: T) -> BoxFuture<U, E> {
        self.inner.call(request)
    }
}

impl<T, U, E> fmt::Debug for BoxCloneService<T, U, E> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BoxCloneService").finish()
    }
}

impl<S, Request> Service<Request> for Boxed<S>
where
    S: Service<Request> + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        Box::pin(self.inner.call(request))
    }
}
