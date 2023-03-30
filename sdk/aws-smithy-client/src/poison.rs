/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Connection Poisoning
//!
//! The client supports behavior where on transient errors (e.g. timeouts, 503, etc.) it will ensure
//! that the offending connection is not reused. This happens to ensure that in the case where the
//! connection itself is broken (e.g. connected to a bad host) we don't reuse it for other requests.
//!
//! This relies on a series of mechanisms:
//! 1. [`CaptureSmithyConnection`] is a container which exists in the operation property bag. It is
//! inserted by this layer before the request is sent.
//! 2. The [`DispatchLayer`](aws_smithy_http_tower::dispatch::DispatchLayer) copies the field from operation extensions HTTP request extensions.
//! 3. The HTTP layer (e.g. Hyper) sets [`ConnectionMetadata`](aws_smithy_http::connection::ConnectionMetadata)
//! when it is available.
//! 4. When the response comes back, if indicated, this layer invokes
//! [`ConnectionMetadata::poison`](aws_smithy_http::connection::ConnectionMetadata::poison).
//!
//! ### Why isn't this integrated into `retry.rs`?
//! If the request has a streaming body, we won't attempt to retry because [`Operation::try_clone()`] will
//! return `None`. Therefore, we need to handle this inside of the retry loop.

use std::future::Future;

use aws_smithy_http::operation::Operation;
use aws_smithy_http::result::{SdkError, SdkSuccess};
use aws_smithy_http::retry::ClassifyRetry;

use aws_smithy_http::connection::CaptureSmithyConnection;
use aws_smithy_types::retry::{ErrorKind, ReconnectMode, RetryKind};
use pin_project_lite::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// PoisonLayer that poisons connections depending on the error kind
pub(crate) struct PoisonLayer<S> {
    inner: PhantomData<S>,
    mode: ReconnectMode,
}

impl<S> PoisonLayer<S> {
    pub(crate) fn new(mode: ReconnectMode) -> Self {
        Self {
            inner: Default::default(),
            mode,
        }
    }
}

impl<S> Clone for PoisonLayer<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Default::default(),
            mode: self.mode,
        }
    }
}

impl<S> tower::Layer<S> for PoisonLayer<S> {
    type Service = PoisonService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PoisonService {
            inner,
            mode: self.mode,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PoisonService<S> {
    inner: S,
    mode: ReconnectMode,
}

impl<H, R, S, O, E> tower::Service<Operation<H, R>> for PoisonService<S>
where
    R: ClassifyRetry<SdkSuccess<O>, SdkError<E>>,
    S: tower::Service<Operation<H, R>, Response = SdkSuccess<O>, Error = SdkError<E>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = PoisonServiceFuture<S::Future, R>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Operation<H, R>) -> Self::Future {
        let classifier = req.retry_classifier().clone();
        let capture_smithy_connection = CaptureSmithyConnection::new();
        req.properties_mut()
            .insert(capture_smithy_connection.clone());
        PoisonServiceFuture {
            inner: self.inner.call(req),
            conn: capture_smithy_connection,
            mode: self.mode,
            classifier,
        }
    }
}

pin_project! {
    pub struct PoisonServiceFuture<F, R> {
        #[pin]
        inner: F,
        classifier: R,
        conn: CaptureSmithyConnection,
        mode: ReconnectMode
    }
}

impl<F, R, T, E> Future for PoisonServiceFuture<F, R>
where
    F: Future<Output = Result<SdkSuccess<T>, SdkError<E>>>,
    R: ClassifyRetry<SdkSuccess<T>, SdkError<E>>,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(resp) => {
                let retry_kind = this.classifier.classify_retry(resp.as_ref());
                if this.mode == &ReconnectMode::ReconnectOnTransientError
                    && retry_kind == RetryKind::Error(ErrorKind::TransientError)
                {
                    if let Some(smithy_conn) = this.conn.get() {
                        tracing::info!("poisoning connection: {:?}", smithy_conn);
                        smithy_conn.poison();
                    } else {
                        tracing::trace!("No smithy connection found! The underlying HTTP connection never set a connection.");
                    }
                }
                Poll::Ready(resp)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
