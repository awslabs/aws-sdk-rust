/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::SendOperationError;
use aws_smithy_http::middleware::load_response;
use aws_smithy_http::operation;
use aws_smithy_http::operation::Operation;
use aws_smithy_http::response::ParseHttpResponse;
use aws_smithy_http::result::SdkError;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::field::display;
use tracing::{debug_span, field, info_span, Instrument};

/// `ParseResponseService` dispatches [`Operation`](aws_smithy_http::operation::Operation)s and parses them.
///
/// `ParseResponseService` is intended to wrap a `DispatchService` which will handle the interface between
/// services that operate on [`operation::Request`](operation::Request) and services that operate
/// on [`http::Request`](http::Request).
#[derive(Clone)]
pub struct ParseResponseService<S, O, R> {
    inner: S,
    _output_type: PhantomData<(O, R)>,
}

#[derive(Default)]
pub struct ParseResponseLayer<O, R> {
    _output_type: PhantomData<(O, R)>,
}

/// `ParseResponseLayer` dispatches [`Operation`](aws_smithy_http::operation::Operation)s and parses them.
impl<O, R> ParseResponseLayer<O, R> {
    pub fn new() -> Self {
        ParseResponseLayer {
            _output_type: Default::default(),
        }
    }
}

impl<S, O, R> Layer<S> for ParseResponseLayer<O, R>
where
    S: Service<operation::Request, Response = operation::Response>,
{
    type Service = ParseResponseService<S, O, R>;

    fn layer(&self, inner: S) -> Self::Service {
        ParseResponseService {
            inner,
            _output_type: Default::default(),
        }
    }
}

type BoxedResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

/// ParseResponseService
///
/// Generic Parameter Listing:
/// `S`: The inner service
/// `O`: The type of the response parser whose output type is `Result<T, E>`
/// `T`: The happy path return of the response parser
/// `E`: The error path return of the response parser
/// `R`: The type of the retry policy
impl<InnerService, ResponseHandler, SuccessResponse, FailureResponse, RetryPolicy>
    tower::Service<operation::Operation<ResponseHandler, RetryPolicy>>
    for ParseResponseService<InnerService, ResponseHandler, RetryPolicy>
where
    InnerService:
        Service<operation::Request, Response = operation::Response, Error = SendOperationError>,
    InnerService::Future: Send + 'static,
    ResponseHandler: ParseHttpResponse<Output = Result<SuccessResponse, FailureResponse>>
        + Send
        + Sync
        + 'static,
    FailureResponse: std::error::Error,
{
    type Response = aws_smithy_http::result::SdkSuccess<SuccessResponse>;
    type Error = aws_smithy_http::result::SdkError<FailureResponse>;
    type Future = BoxedResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: Operation<ResponseHandler, RetryPolicy>) -> Self::Future {
        let (req, parts) = req.into_request_response();
        let handler = parts.response_handler;
        // send_operation records the full request-response lifecycle.
        // NOTE: For operations that stream output, only the setup is captured in this span.
        let span = info_span!(
            "send_operation",
            operation = field::Empty,
            service = field::Empty,
            status = field::Empty,
            message = field::Empty
        );
        let inner_span = span.clone();
        if let Some(metadata) = parts.metadata {
            span.record("operation", &metadata.name());
            span.record("service", &metadata.service());
        }
        let resp = self.inner.call(req);
        let fut = async move {
            let resp = match resp.await {
                Err(e) => Err(e.into()),
                Ok(resp) => {
                    // load_response contains reading the body as far as is required & parsing the response
                    let response_span = debug_span!("load_response");
                    load_response(resp, &handler)
                        .instrument(response_span)
                        .await
                }
            };
            match &resp {
                Ok(_) => inner_span.record("status", &"ok"),
                Err(SdkError::ServiceError { err, .. }) => inner_span
                    .record("status", &"service_err")
                    .record("message", &display(&err)),
                Err(SdkError::ResponseError { err, .. }) => inner_span
                    .record("status", &"response_err")
                    .record("message", &display(&err)),
                Err(SdkError::DispatchFailure(err)) => inner_span
                    .record("status", &"dispatch_failure")
                    .record("message", &display(err)),
                Err(SdkError::ConstructionFailure(err)) => inner_span
                    .record("status", &"construction_failure")
                    .record("message", &display(err)),
                Err(SdkError::TimeoutError(err)) => inner_span
                    .record("status", &"timeout_error")
                    .record("message", &display(err)),
            };
            resp
        }
        .instrument(span);
        Box::pin(fut)
    }
}
