/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

use crate::SendOperationError;
use pin_project::pin_project;
use smithy_http::body::SdkBody;
use smithy_http::operation;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Layer, Service};

#[pin_project]
pub struct DispatchFuture<F> {
    #[pin]
    f: F,
}

/// Connects Operation driven middleware to an HTTP implementation.
///
/// It will also wrap the error type in OperationError to enable operation middleware
/// reporting specific errors
#[derive(Clone)]
pub struct DispatchService<S> {
    inner: S,
}

impl<F, T, E> Future for DispatchFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<BoxError>,
{
    type Output = Result<T, SendOperationError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.f
            .poll(cx)
            .map_err(|e| SendOperationError::RequestDispatchError(e.into()))
    }
}

impl<S> Service<operation::Request> for DispatchService<S>
where
    S: Service<http::Request<SdkBody>>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = SendOperationError;
    type Future = DispatchFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| SendOperationError::RequestDispatchError(e.into()))
    }

    fn call(&mut self, req: operation::Request) -> Self::Future {
        let (req, _property_bag) = req.into_parts();
        DispatchFuture {
            f: self.inner.call(req),
        }
    }
}

#[derive(Clone, Default)]
#[non_exhaustive]
pub struct DispatchLayer;

impl DispatchLayer {
    pub fn new() -> Self {
        DispatchLayer
    }
}

impl<S> Layer<S> for DispatchLayer
where
    S: Service<http::Request<SdkBody>>,
{
    type Service = DispatchService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        DispatchService { inner }
    }
}
