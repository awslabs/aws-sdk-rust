/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Interceptor context.
//!
//! Interceptors have access to varying pieces of context during the course of an operation.
//!
//! An operation is composed of multiple phases. The initial phase is [`phase::BeforeSerialization`], which
//! has the original input as context. The next phase is [`phase::BeforeTransmit`], which has the serialized
//! request as context. Depending on which hook is being called with the dispatch context,
//! the serialized request may or may not be signed (which should be apparent from the hook name).
//! Following the [`phase::BeforeTransmit`] phase is the [`phase::BeforeDeserialization`] phase, which has
//! the raw response available as context. Finally, the [`phase::AfterDeserialization`] phase
//! has both the raw and parsed response available.
//!
//! To summarize:
//! 1. [`phase::BeforeSerialization`]: Only has the operation input.
//! 2. [`phase::BeforeTransmit`]: Only has the serialized request.
//! 3. [`phase::BeforeDeserialization`]: Has the raw response.
//! 3. [`phase::AfterDeserialization`]: Has the raw response and the parsed response.
//!
//! When implementing hooks, if information from a previous phase is required, then implement
//! an earlier hook to examine that context, and save off any necessary information into the
//! [`crate::config_bag::ConfigBag`] for later hooks to examine.  Interior mutability is **NOT**
//! recommended for storing request-specific information in your interceptor implementation.
//! Use the [`crate::config_bag::ConfigBag`] instead.

use crate::client::interceptors::BoxError;
use crate::client::orchestrator::{HttpRequest, HttpResponse};
use crate::config_bag::ConfigBag;
use crate::type_erasure::{TypeErasedBox, TypeErasedError};
use aws_smithy_http::result::SdkError;

pub type Input = TypeErasedBox;
pub type Output = TypeErasedBox;
pub type Error = TypeErasedError;
pub type OutputOrError = Result<Output, Error>;

type Request = HttpRequest;
type Response = HttpResponse;

/// Operation phases.
pub mod phase {
    use crate::client::interceptors::context::{Error, Output};
    use crate::client::interceptors::BoxError;
    use crate::client::orchestrator::HttpResponse;
    use aws_smithy_http::result::{ConnectorError, SdkError};

    macro_rules! impl_phase {
        ($phase:ty, $convert_err:ident) => {
            impl Phase for $phase {
                fn convert_error(
                    &self,
                    error: BoxError,
                    output_or_error: Option<Result<Output, Error>>,
                    response: Option<HttpResponse>,
                ) -> SdkError<Error, HttpResponse> {
                    $convert_err(error, output_or_error, response)
                }
            }
        };
    }

    #[doc(hidden)]
    pub trait Phase {
        fn convert_error(
            &self,
            error: BoxError,
            output_or_error: Option<Result<Output, Error>>,
            response: Option<HttpResponse>,
        ) -> SdkError<Error, HttpResponse>;
    }

    fn convert_construction_failure(
        error: BoxError,
        _: Option<Result<Output, Error>>,
        _: Option<HttpResponse>,
    ) -> SdkError<Error, HttpResponse> {
        SdkError::construction_failure(error)
    }

    fn convert_dispatch_error(
        error: BoxError,
        _: Option<Result<Output, Error>>,
        response: Option<HttpResponse>,
    ) -> SdkError<Error, HttpResponse> {
        let error = match error.downcast::<ConnectorError>() {
            Ok(connector_error) => {
                return SdkError::dispatch_failure(*connector_error);
            }
            Err(e) => e,
        };
        if let Some(response) = response {
            SdkError::response_error(error, response)
        } else {
            SdkError::dispatch_failure(ConnectorError::other(error, None))
        }
    }

    fn convert_response_handling_error(
        error: BoxError,
        output_or_error: Option<Result<Output, Error>>,
        response: Option<HttpResponse>,
    ) -> SdkError<Error, HttpResponse> {
        match (response, output_or_error) {
            (Some(response), Some(Err(error))) => SdkError::service_error(error, response),
            (Some(response), _) => SdkError::response_error(error, response),
            _ => unreachable!("phase has a response"),
        }
    }

    /// Represents the phase of an operation prior to serialization.
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct BeforeSerialization;
    impl_phase!(BeforeSerialization, convert_construction_failure);

    #[doc(hidden)] // This one isn't exposed in the interceptors, but is used internally
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct Serialization;
    impl_phase!(Serialization, convert_construction_failure);

    /// Represents the phase of an operation prior to transmitting a request over the network.
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct BeforeTransmit;
    impl_phase!(BeforeTransmit, convert_dispatch_error);

    #[doc(hidden)] // This one isn't exposed in the interceptors, but is used internally
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct Transmit;
    impl_phase!(Transmit, convert_dispatch_error);

    /// Represents the phase of an operation after receiving a response, but before parsing that response.
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct BeforeDeserialization;
    impl_phase!(BeforeDeserialization, convert_response_handling_error);

    #[doc(hidden)] // This one isn't exposed in the interceptors, but is used internally
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct Deserialization;
    impl_phase!(Deserialization, convert_response_handling_error);

    /// Represents the phase of an operation after parsing a response.
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub struct AfterDeserialization;
    impl_phase!(AfterDeserialization, convert_response_handling_error);
}

/// A container for the data currently available to an interceptor.
///
/// Different context is available based on which phase the operation is currently in. For example,
/// context in the [`phase::BeforeSerialization`] phase won't have a `request` yet since the input hasn't been
/// serialized at that point. But once it gets into the [`phase::BeforeTransmit`] phase, the `request` will be set.
pub struct InterceptorContext<Phase, I = Input, O = Output, E = Error> {
    input: Option<I>,
    output_or_error: Option<Result<O, E>>,
    request: Option<Request>,
    response: Option<Response>,
    phase: Phase,
}

//
// All phases
//
impl InterceptorContext<(), Input, Output, Error> {
    /// Creates a new interceptor context in the [`phase::BeforeSerialization`] phase.
    pub fn new(
        input: Input,
    ) -> InterceptorContext<phase::BeforeSerialization, Input, Output, Error> {
        InterceptorContext {
            input: Some(input),
            output_or_error: None,
            request: None,
            response: None,
            phase: Default::default(),
        }
    }
}
impl<Phase: phase::Phase, I, O, E> InterceptorContext<Phase, I, O, E> {
    /// Decomposes the context into its constituent parts.
    #[doc(hidden)]
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        Option<I>,
        Option<Result<O, E>>,
        Option<Request>,
        Option<Response>,
        Phase,
    ) {
        (
            self.input,
            self.output_or_error,
            self.request,
            self.response,
            self.phase,
        )
    }
}

//
// BeforeSerialization phase methods
//
impl<I, O, E> InterceptorContext<phase::BeforeSerialization, I, O, E> {
    /// Retrieve the input for the operation being invoked.
    pub fn input(&self) -> &I {
        self.input
            .as_ref()
            .expect("input is present in phase::BeforeSerialization")
    }

    /// Retrieve the input for the operation being invoked.
    pub fn input_mut(&mut self) -> &mut I {
        self.input
            .as_mut()
            .expect("input is present in phase::BeforeSerialization")
    }

    /// Advance to the next phase.
    #[doc(hidden)]
    pub fn into_serialization_phase(self) -> InterceptorContext<phase::Serialization, I, O, E> {
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: phase::Serialization::default(),
        }
    }
}

//
// Serialization phase methods
//
impl<I, O, E> InterceptorContext<phase::Serialization, I, O, E> {
    /// Takes ownership of the input.
    pub fn take_input(&mut self) -> Option<I> {
        self.input.take()
    }

    pub fn set_request(&mut self, request: Request) {
        debug_assert!(
            self.request.is_none(),
            "called set_request but a request was already set"
        );
        self.request = Some(request);
    }

    /// Advance to the next phase.
    #[doc(hidden)]
    pub fn into_before_transmit_phase(self) -> InterceptorContext<phase::BeforeTransmit, I, O, E> {
        debug_assert!(
            self.input.is_none(),
            "input must be taken before going into phase::BeforeTransmit"
        );
        debug_assert!(
            self.request.is_some(),
            "request must be set before going into phase::BeforeTransmit"
        );
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: Default::default(),
        }
    }
}

//
// BeforeTransmit phase methods
//
impl<I, O, E> InterceptorContext<phase::BeforeTransmit, I, O, E> {
    /// Creates a new interceptor context in the [`phase::BeforeTransmit`] phase.
    pub fn new(
        input: Option<Input>,
        request: HttpRequest,
    ) -> InterceptorContext<phase::BeforeTransmit, Input, Output, Error> {
        InterceptorContext {
            input,
            output_or_error: None,
            request: Some(request),
            response: None,
            phase: Default::default(),
        }
    }

    /// Retrieve the transmittable request for the operation being invoked.
    /// This will only be available once request marshalling has completed.
    pub fn request(&self) -> &Request {
        self.request
            .as_ref()
            .expect("request populated in phase::BeforeTransmit")
    }

    /// Retrieve the transmittable request for the operation being invoked.
    /// This will only be available once request marshalling has completed.
    pub fn request_mut(&mut self) -> &mut Request {
        self.request
            .as_mut()
            .expect("request populated in phase::BeforeTransmit")
    }

    #[doc(hidden)]
    pub fn into_transmit_phase(self) -> InterceptorContext<phase::Transmit, I, O, E> {
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: Default::default(),
        }
    }
}

//
// Transmit phase methods
//
impl<I, O, E> InterceptorContext<phase::Transmit, I, O, E> {
    /// Takes ownership of the request.
    #[doc(hidden)]
    pub fn take_request(&mut self) -> Request {
        debug_assert!(self.request.is_some());
        self.request
            .take()
            .expect("take request once during transmit")
    }

    #[doc(hidden)]
    pub fn set_response(&mut self, response: Response) {
        debug_assert!(
            self.response.is_none(),
            "called set_response but a response was already set"
        );
        self.response = Some(response);
    }

    #[doc(hidden)]
    pub fn into_before_deserialization_phase(
        self,
    ) -> InterceptorContext<phase::BeforeDeserialization, I, O, E> {
        debug_assert!(
            self.request.is_none(),
            "request must be taken before going into phase::BeforeDeserialization"
        );
        debug_assert!(
            self.response.is_some(),
            "response must be set to before going into phase::BeforeDeserialization"
        );
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: Default::default(),
        }
    }
}

impl<I, O, E> InterceptorContext<phase::BeforeDeserialization, I, O, E> {
    /// Returns the response.
    pub fn response(&self) -> &Response {
        self.response
            .as_ref()
            .expect("response set in phase::BeforeDeserialization")
    }

    /// Returns a mutable reference to the response.
    pub fn response_mut(&mut self) -> &mut Response {
        self.response
            .as_mut()
            .expect("response set in phase::BeforeDeserialization")
    }

    #[doc(hidden)]
    pub fn into_deserialization_phase(self) -> InterceptorContext<phase::Deserialization, I, O, E> {
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: Default::default(),
        }
    }
}

impl<I, O, E> InterceptorContext<phase::Deserialization, I, O, E> {
    /// Returns the response.
    pub fn response(&self) -> &Response {
        self.response
            .as_ref()
            .expect("response set in phase::Deserialization")
    }

    /// Returns a mutable reference to the response.
    pub fn response_mut(&mut self) -> &mut Response {
        self.response
            .as_mut()
            .expect("response set in phase::Deserialization")
    }

    #[doc(hidden)]
    pub fn set_output_or_error(&mut self, output: Result<O, E>) {
        debug_assert!(self.output_or_error.is_none());
        self.output_or_error = Some(output);
    }

    #[doc(hidden)]
    pub fn into_after_deserialization_phase(
        self,
    ) -> InterceptorContext<phase::AfterDeserialization, I, O, E> {
        debug_assert!(
            self.output_or_error.is_some(),
            "output must be set to before going into phase::AfterDeserialization"
        );
        InterceptorContext {
            input: self.input,
            output_or_error: self.output_or_error,
            request: self.request,
            response: self.response,
            phase: Default::default(),
        }
    }
}

impl<I, O, E> InterceptorContext<phase::AfterDeserialization, I, O, E> {
    /// Returns the response.
    pub fn response(&self) -> &Response {
        self.response
            .as_ref()
            .expect("response set in phase::BeforeDeserialization")
    }

    /// Returns a mutable reference to the response.
    pub fn response_mut(&mut self) -> &mut Response {
        self.response
            .as_mut()
            .expect("response set in phase::BeforeDeserialization")
    }

    /// Returns the deserialized output or error.
    pub fn output_or_error(&self) -> Result<&O, &E> {
        self.output_or_error
            .as_ref()
            .expect("output set in phase::AfterDeserialization")
            .as_ref()
    }

    /// Returns the mutable reference to the deserialized output or error.
    pub fn output_or_error_mut(&mut self) -> &mut Result<O, E> {
        self.output_or_error
            .as_mut()
            .expect("output set in phase::AfterDeserialization")
    }

    #[doc(hidden)]
    pub fn finalize(self) -> Result<O, SdkError<E, HttpResponse>> {
        self.output_or_error
            .expect("output already populated in the response handling phase")
            .map_err(|error| {
                SdkError::service_error(
                    error,
                    self.response
                        .expect("raw response already populated in the response handling phase"),
                )
            })
    }
}

// This isn't great since it relies on a lot of runtime checking, but the
// compiler doesn't exactly make it easy to handle phase changes in a `loop`.
#[doc(hidden)]
pub struct AttemptCheckpoint {
    tainted: bool,
    checkpointed_request: Option<HttpRequest>,
    before_transmit: Option<InterceptorContext<phase::BeforeTransmit>>,
    transmit: Option<InterceptorContext<phase::Transmit>>,
    before_deserialization: Option<InterceptorContext<phase::BeforeDeserialization>>,
    deserialization: Option<InterceptorContext<phase::Deserialization>>,
    after_deserialization: Option<InterceptorContext<phase::AfterDeserialization>>,
}

impl AttemptCheckpoint {
    pub fn new(before_transmit: InterceptorContext<phase::BeforeTransmit>) -> Self {
        Self {
            tainted: false,
            checkpointed_request: Self::try_clone(before_transmit.request()),
            before_transmit: Some(before_transmit),
            transmit: None,
            before_deserialization: None,
            deserialization: None,
            after_deserialization: None,
        }
    }

    pub fn before_transmit(&mut self) -> &mut InterceptorContext<phase::BeforeTransmit> {
        self.tainted = true;
        self.before_transmit
            .as_mut()
            .expect("must be in the before transmit phase")
    }

    pub fn transmit(&mut self) -> &mut InterceptorContext<phase::Transmit> {
        self.transmit
            .as_mut()
            .expect("must be in the transmit phase")
    }

    pub fn before_deser(&mut self) -> &mut InterceptorContext<phase::BeforeDeserialization> {
        self.before_deserialization
            .as_mut()
            .expect("must be in the before deserialization phase")
    }

    pub fn deser(&mut self) -> &mut InterceptorContext<phase::Deserialization> {
        self.deserialization
            .as_mut()
            .expect("must be in the deserialization phase")
    }

    pub fn after_deser(&mut self) -> &mut InterceptorContext<phase::AfterDeserialization> {
        self.after_deserialization
            .as_mut()
            .expect("must be in the after deserialization phase")
    }

    pub fn transition_to_transmit(&mut self) {
        self.transmit = Some(
            self.before_transmit
                .take()
                .expect("must be in the before transmit phase")
                .into_transmit_phase(),
        );
    }

    pub fn transition_to_deserialization(&mut self) {
        self.deserialization = Some(
            self.before_deserialization
                .take()
                .expect("must be in the before deserialization phase")
                .into_deserialization_phase(),
        )
    }

    pub fn transition_to_before_deserialization(&mut self) {
        self.before_deserialization = Some(
            self.transmit
                .take()
                .expect("must be in the transmit phase")
                .into_before_deserialization_phase(),
        )
    }

    pub fn transition_to_after_deserialization(&mut self) {
        self.after_deserialization = Some(
            self.deserialization
                .take()
                .expect("must be in the deserialization phase")
                .into_after_deserialization_phase(),
        )
    }

    // Returns false if rewinding isn't possible
    pub fn rewind(&mut self, _cfg: &mut ConfigBag) -> bool {
        // If before transmit was never touched, then we don't need to rewind
        if !self.tainted {
            return true;
        }
        // If checkpointed_request was never set, then this is not a retryable request
        if self.checkpointed_request.is_none() {
            return false;
        }
        // Otherwise, rewind back to the beginning of BeforeTransmit
        // TODO(enableNewSmithyRuntime): Also rewind the ConfigBag
        fn into_input<P: phase::Phase + 'static>(context: InterceptorContext<P>) -> Option<Input> {
            context.into_parts().0
        }
        // Take the input from the current phase
        let input = None
            .or(self.before_transmit.take().map(into_input))
            .or(self.transmit.take().map(into_input))
            .or(self.before_deserialization.take().map(into_input))
            .or(self.deserialization.take().map(into_input))
            .or(self.after_deserialization.take().map(into_input))
            .expect("at least one phase must be in progress");
        let fresh_request =
            Self::try_clone(self.checkpointed_request.as_ref().expect("checked above"))
                .expect("cloneable request");
        self.before_transmit = Some(InterceptorContext::<phase::BeforeTransmit>::new(
            input,
            fresh_request,
        ));
        true
    }

    pub fn into_error(self, reason: BoxError) -> SdkError<Error, HttpResponse> {
        fn err<P: phase::Phase + 'static>(
            context: InterceptorContext<P>,
        ) -> Box<dyn FnOnce(BoxError) -> SdkError<Error, HttpResponse>> {
            Box::new(move |reason| {
                let (_input, output_or_error, _request, response, phase) = context.into_parts();
                phase.convert_error(reason, output_or_error, response)
            })
        }
        // Convert the current phase into an error
        (None
            .or(self.before_transmit.map(err))
            .or(self.transmit.map(err))
            .or(self.before_deserialization.map(err))
            .or(self.deserialization.map(err))
            .or(self.after_deserialization.map(err))
            .expect("at least one phase must be in progress"))(reason)
    }

    pub fn finalize(self) -> Result<Output, SdkError<Error, HttpResponse>> {
        self.after_deserialization
            .expect("must be in the after deserialization phase")
            .finalize()
    }

    pub fn try_clone(request: &HttpRequest) -> Option<HttpRequest> {
        let cloned_body = request.body().try_clone()?;
        let mut cloned_request = ::http::Request::builder()
            .uri(request.uri().clone())
            .method(request.method());
        *cloned_request
            .headers_mut()
            .expect("builder has not been modified, headers must be valid") =
            request.headers().clone();
        Some(
            cloned_request
                .body(cloned_body)
                .expect("a clone of a valid request should be a valid request"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_erasure::TypedBox;
    use aws_smithy_http::body::SdkBody;
    use http::header::{AUTHORIZATION, CONTENT_LENGTH};
    use http::{HeaderValue, Uri};

    #[test]
    fn test_success_transitions() {
        let input = TypedBox::new("input".to_string()).erase();
        let output = TypedBox::new("output".to_string()).erase();

        let mut context = InterceptorContext::<()>::new(input);
        assert_eq!("input", context.input().downcast_ref::<String>().unwrap());
        context.input_mut();

        let mut context = context.into_serialization_phase();
        let _ = context.take_input();
        context.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());

        let mut checkpoint = AttemptCheckpoint::new(context.into_before_transmit_phase());
        checkpoint.before_transmit().request();
        checkpoint.before_transmit().request_mut();

        checkpoint.transition_to_transmit();
        let _ = checkpoint.transmit().take_request();
        checkpoint
            .transmit()
            .set_response(http::Response::builder().body(SdkBody::empty()).unwrap());

        checkpoint.transition_to_before_deserialization();
        checkpoint.before_deser().response();
        checkpoint.before_deser().response_mut();

        checkpoint.transition_to_deserialization();
        checkpoint.deser().response();
        checkpoint.deser().response_mut();
        checkpoint.deser().set_output_or_error(Ok(output));

        checkpoint.transition_to_after_deserialization();
        checkpoint.after_deser().response();
        checkpoint.after_deser().response_mut();
        let _ = checkpoint.after_deser().output_or_error();
        let _ = checkpoint.after_deser().output_or_error_mut();

        let output = checkpoint.finalize().expect("success");
        assert_eq!("output", output.downcast_ref::<String>().unwrap());
    }

    #[test]
    fn test_rewind_for_retry() {
        use std::fmt;
        #[derive(Debug)]
        struct Error;
        impl fmt::Display for Error {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("dontcare")
            }
        }
        impl std::error::Error for Error {}

        let mut cfg = ConfigBag::base();
        let input = TypedBox::new("input".to_string()).erase();
        let output = TypedBox::new("output".to_string()).erase();
        let error = TypedBox::new(Error).erase_error();

        let context = InterceptorContext::<()>::new(input);
        assert_eq!("input", context.input().downcast_ref::<String>().unwrap());

        let mut context = context.into_serialization_phase();
        let _ = context.take_input();
        context.set_request(
            http::Request::builder()
                .header("test", "the-original-unmutated-request")
                .body(SdkBody::empty())
                .unwrap(),
        );

        let mut checkpoint = AttemptCheckpoint::new(context.into_before_transmit_phase());

        // Modify the test header post-checkpoint to simulate modifying the request for signing or a mutating interceptor
        checkpoint
            .before_transmit()
            .request_mut()
            .headers_mut()
            .remove("test");
        checkpoint
            .before_transmit()
            .request_mut()
            .headers_mut()
            .insert(
                "test",
                HeaderValue::from_static("request-modified-after-signing"),
            );

        checkpoint.transition_to_transmit();
        let request = checkpoint.transmit().take_request();
        assert_eq!(
            "request-modified-after-signing",
            request.headers().get("test").unwrap()
        );
        checkpoint
            .transmit()
            .set_response(http::Response::builder().body(SdkBody::empty()).unwrap());

        checkpoint.transition_to_before_deserialization();
        checkpoint.transition_to_deserialization();
        checkpoint.deser().set_output_or_error(Err(error));

        assert!(checkpoint.rewind(&mut cfg));

        // Now after rewinding, the test header should be its original value
        assert_eq!(
            "the-original-unmutated-request",
            checkpoint
                .before_transmit()
                .request()
                .headers()
                .get("test")
                .unwrap()
        );

        checkpoint.transition_to_transmit();
        let _ = checkpoint.transmit().take_request();
        checkpoint
            .transmit()
            .set_response(http::Response::builder().body(SdkBody::empty()).unwrap());

        checkpoint.transition_to_before_deserialization();
        checkpoint.transition_to_deserialization();
        checkpoint.deser().set_output_or_error(Ok(output));

        checkpoint.transition_to_after_deserialization();

        let output = checkpoint.finalize().expect("success");
        assert_eq!("output", output.downcast_ref::<String>().unwrap());
    }

    #[test]
    fn try_clone_clones_all_data() {
        let request = ::http::Request::builder()
            .uri(Uri::from_static("http://www.amazon.com"))
            .method("POST")
            .header(CONTENT_LENGTH, 456)
            .header(AUTHORIZATION, "Token: hello")
            .body(SdkBody::from("hello world!"))
            .expect("valid request");
        let cloned = AttemptCheckpoint::try_clone(&request).expect("request is cloneable");

        assert_eq!(&Uri::from_static("http://www.amazon.com"), cloned.uri());
        assert_eq!("POST", cloned.method());
        assert_eq!(2, cloned.headers().len());
        assert_eq!("Token: hello", cloned.headers().get(AUTHORIZATION).unwrap(),);
        assert_eq!("456", cloned.headers().get(CONTENT_LENGTH).unwrap());
        assert_eq!("hello world!".as_bytes(), cloned.body().bytes().unwrap());
    }
}
