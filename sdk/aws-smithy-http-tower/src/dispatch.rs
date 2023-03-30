/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::SendOperationError;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::connection::CaptureSmithyConnection;
use aws_smithy_http::operation;
use aws_smithy_http::result::ConnectorError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug_span, trace, Instrument};

/// Connects Operation driven middleware to an HTTP implementation.
///
/// It will also wrap the error type in OperationError to enable operation middleware
/// reporting specific errors
#[derive(Clone)]
pub struct DispatchService<S> {
    inner: S,
}

type BoxedResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

impl<S> Service<operation::Request> for DispatchService<S>
where
    S: Service<http::Request<SdkBody>, Response = http::Response<SdkBody>> + Clone + Send + 'static,
    S::Error: Into<ConnectorError>,
    S::Future: Send + 'static,
{
    type Response = operation::Response;
    type Error = SendOperationError;
    type Future = BoxedResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|e| SendOperationError::RequestDispatchError(e.into()))
    }

    fn call(&mut self, req: operation::Request) -> Self::Future {
        let (mut req, property_bag) = req.into_parts();
        // copy the smithy connection
        if let Some(smithy_conn) = property_bag.acquire().get::<CaptureSmithyConnection>() {
            req.extensions_mut().insert(smithy_conn.clone());
        } else {
            println!("nothing to copy!");
        }
        let mut inner = self.inner.clone();
        let future = async move {
            trace!(request = ?req, "dispatching request");
            inner
                .call(req)
                .await
                .map(|resp| operation::Response::from_parts(resp, property_bag))
                .map_err(|e| SendOperationError::RequestDispatchError(e.into()))
        }
        .instrument(debug_span!("dispatch"));
        Box::pin(future)
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
