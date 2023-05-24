/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::result::{ConnectorError, SdkError};
use aws_smithy_runtime_api::client::interceptors::context::{Error, Output};
use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, HttpResponse};

#[derive(Copy, Clone, Eq, PartialEq)]
enum OrchestrationPhase {
    Construction,
    Dispatch,
    ResponseHandling,
}

pub(super) struct Phase {
    phase: OrchestrationPhase,
    context: InterceptorContext,
}

impl Phase {
    pub(crate) fn construction(context: InterceptorContext) -> Self {
        Self::start(OrchestrationPhase::Construction, context)
    }
    pub(crate) fn dispatch(context: InterceptorContext) -> Self {
        Self::start(OrchestrationPhase::Dispatch, context)
    }
    pub(crate) fn response_handling(context: InterceptorContext) -> Self {
        Self::start(OrchestrationPhase::ResponseHandling, context)
    }

    fn start(phase: OrchestrationPhase, context: InterceptorContext) -> Self {
        match phase {
            OrchestrationPhase::Construction => {}
            OrchestrationPhase::Dispatch => {}
            OrchestrationPhase::ResponseHandling => debug_assert!(context.response().is_ok()),
        }
        Self { phase, context }
    }

    pub(crate) fn include_mut<E: Into<BoxError>>(
        mut self,
        c: impl FnOnce(&mut InterceptorContext) -> Result<(), E>,
    ) -> Result<Self, SdkError<Error, HttpResponse>> {
        match c(&mut self.context) {
            Ok(_) => Ok(self),
            Err(e) => Err(self.fail(e)),
        }
    }

    pub(crate) fn include<E: Into<BoxError>>(
        self,
        c: impl FnOnce(&InterceptorContext) -> Result<(), E>,
    ) -> Result<Self, SdkError<Error, HttpResponse>> {
        match c(&self.context) {
            Ok(_) => Ok(self),
            Err(e) => Err(self.fail(e)),
        }
    }

    pub(crate) fn fail(self, e: impl Into<BoxError>) -> SdkError<Error, HttpResponse> {
        self.into_sdk_error(e.into())
    }

    pub(crate) fn finalize(self) -> Result<Output, SdkError<Error, HttpResponse>> {
        debug_assert!(self.phase == OrchestrationPhase::ResponseHandling);
        let (_input, output_or_error, _request, response) = self.context.into_parts();
        match output_or_error {
            Some(output_or_error) => match output_or_error {
                Ok(output) => Ok(output),
                Err(error) => Err(SdkError::service_error(
                    error,
                    response.expect("response must be set by this point"),
                )),
            },
            None => unreachable!("phase can't get this far without bubbling up a failure"),
        }
    }

    fn into_sdk_error(self, e: BoxError) -> SdkError<Error, HttpResponse> {
        let e = match e.downcast::<ConnectorError>() {
            Ok(connector_error) => {
                debug_assert!(
                    self.phase == OrchestrationPhase::Dispatch,
                    "connector errors should only occur during the dispatch phase"
                );
                return SdkError::dispatch_failure(*connector_error);
            }
            Err(e) => e,
        };
        let (_input, output_or_error, _request, response) = self.context.into_parts();
        match self.phase {
            OrchestrationPhase::Construction => SdkError::construction_failure(e),
            OrchestrationPhase::Dispatch => {
                if let Some(response) = response {
                    SdkError::response_error(e, response)
                } else {
                    SdkError::dispatch_failure(ConnectorError::other(e, None))
                }
            }
            OrchestrationPhase::ResponseHandling => match (response, output_or_error) {
                (Some(response), Some(Err(error))) => SdkError::service_error(error, response),
                (Some(response), _) => SdkError::response_error(e, response),
                _ => unreachable!("response handling phase at least has a response"),
            },
        }
    }

    pub(crate) fn finish(self) -> InterceptorContext {
        self.context
    }
}
