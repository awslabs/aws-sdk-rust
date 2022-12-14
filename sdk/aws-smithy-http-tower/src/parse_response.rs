/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::SendOperationError;
use aws_smithy_http::middleware::load_response;
use aws_smithy_http::operation;
use aws_smithy_http::operation::Operation;
use aws_smithy_http::response::ParseHttpResponse;
use aws_smithy_http::result::{SdkError, SdkSuccess};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};
use tracing::{debug_span, Instrument};

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
    Service<Operation<ResponseHandler, RetryPolicy>>
    for ParseResponseService<InnerService, ResponseHandler, RetryPolicy>
where
    InnerService:
        Service<operation::Request, Response = operation::Response, Error = SendOperationError>,
    InnerService::Future: Send + 'static,
    ResponseHandler: ParseHttpResponse<Output = Result<SuccessResponse, FailureResponse>>
        + Send
        + Sync
        + 'static,
    FailureResponse: std::error::Error + 'static,
{
    type Response = SdkSuccess<SuccessResponse>;
    type Error = SdkError<FailureResponse>;
    type Future = BoxedResultFuture<Self::Response, Self::Error>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(|err| err.into())
    }

    fn call(&mut self, req: Operation<ResponseHandler, RetryPolicy>) -> Self::Future {
        let (req, parts) = req.into_request_response();
        let handler = parts.response_handler;
        let resp = self.inner.call(req);
        Box::pin(async move {
            match resp.await {
                Err(e) => Err(e.into()),
                Ok(resp) => {
                    load_response(resp, &handler)
                        // load_response contains reading the body as far as is required & parsing the response
                        .instrument(debug_span!("load_response"))
                        .await
                }
            }
        })
    }
}
