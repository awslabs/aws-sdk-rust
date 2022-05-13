/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::SendOperationError;
use aws_smithy_http::middleware::{AsyncMapRequest, MapRequest};
use aws_smithy_http::operation;
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Debug)]
pub struct AsyncMapRequestLayer<M> {
    mapper: M,
}

impl<M: AsyncMapRequest + Clone> AsyncMapRequestLayer<M> {
    pub fn for_mapper(mapper: M) -> Self {
        AsyncMapRequestLayer { mapper }
    }
}

impl<S, M> Layer<S> for AsyncMapRequestLayer<M>
where
    M: Clone,
{
    type Service = AsyncMapRequestService<S, M>;

    fn layer(&self, inner: S) -> Self::Service {
        AsyncMapRequestService {
            inner,
            mapper: self.mapper.clone(),
        }
    }
}

/// Tower service for [`AsyncMapRequest`](aws_smithy_http::middleware::AsyncMapRequest)
#[derive(Clone)]
pub struct AsyncMapRequestService<S, M> {
    inner: S,
    mapper: M,
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

impl<S, M> Service<operation::Request> for AsyncMapRequestService<S, M>
where
    S: Service<operation::Request, Error = SendOperationError> + Clone + Send + 'static,
    M: AsyncMapRequest,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<Result<S::Response, S::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: operation::Request) -> Self::Future {
        let mut inner = self.inner.clone();
        let future = self.mapper.apply(req);
        Box::pin(async move {
            let mapped_request = future
                .await
                .map_err(|e| SendOperationError::RequestConstructionError(e.into()))?;
            inner.call(mapped_request).await
        })
    }
}

#[derive(Debug, Default)]
pub struct MapRequestLayer<M> {
    mapper: M,
}

impl<M: MapRequest + Clone> MapRequestLayer<M> {
    pub fn for_mapper(mapper: M) -> Self {
        MapRequestLayer { mapper }
    }
}

impl<S, M> Layer<S> for MapRequestLayer<M>
where
    M: Clone,
{
    type Service = MapRequestService<S, M>;

    fn layer(&self, inner: S) -> Self::Service {
        MapRequestService {
            inner,
            mapper: self.mapper.clone(),
        }
    }
}

#[pin_project(project = EnumProj)]
pub enum MapRequestFuture<F, E> {
    Inner(#[pin] F),
    Ready(Option<E>),
}

impl<O, F, E> Future for MapRequestFuture<F, E>
where
    F: Future<Output = Result<O, E>>,
{
    type Output = Result<O, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            EnumProj::Ready(e) => Poll::Ready(Err(e.take().unwrap())),
            EnumProj::Inner(f) => f.poll(cx),
        }
    }
}

#[derive(Clone)]
/// Tower service for [`MapRequest`](aws_smithy_http::middleware::MapRequest)
pub struct MapRequestService<S, M> {
    inner: S,
    mapper: M,
}

impl<S, M> Service<operation::Request> for MapRequestService<S, M>
where
    S: Service<operation::Request, Error = SendOperationError>,
    M: MapRequest,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = MapRequestFuture<S::Future, S::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: operation::Request) -> Self::Future {
        match self
            .mapper
            .apply(req)
            .map_err(|e| SendOperationError::RequestConstructionError(e.into()))
        {
            Err(e) => MapRequestFuture::Ready(Some(e)),
            Ok(req) => MapRequestFuture::Inner(self.inner.call(req)),
        }
    }
}
