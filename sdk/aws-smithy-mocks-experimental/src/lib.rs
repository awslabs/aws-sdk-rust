/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
/* End of automatically managed default lints */
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeDeserializationInterceptorContextMut, BeforeSerializationInterceptorContextMut, Error,
    FinalizerInterceptorContextMut, Input, Output,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::orchestrator::{HttpResponse, OrchestratorError};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::http::{Response, StatusCode};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};

// why do we need a macro for this?
// We want customers to be able to provide an ergonomic way to say the method they're looking for,
// `Client::list_buckets`, e.g. But there isn't enough information on that type to recover everything.
// This macro commits a small amount of crimes to recover that type information so we can construct
// a rule that can intercept these operations.

/// `mock!` macro that produces a [`RuleBuilder`] from a client invocation
///
/// See the `examples` folder of this crate for fully worked examples.
///
/// # Examples
/// **Mock and return a success response**:
/// ```rust
/// use aws_sdk_s3::operation::get_object::GetObjectOutput;
/// use aws_sdk_s3::Client;
/// use aws_smithy_types::byte_stream::ByteStream;
/// use aws_smithy_mocks_experimental::mock;
/// let get_object_happy_path = mock!(Client::get_object)
///   .match_requests(|req|req.bucket() == Some("test-bucket") && req.key() == Some("test-key"))
///   .then_output(||GetObjectOutput::builder().body(ByteStream::from_static(b"12345-abcde")).build());
/// ```
///
/// **Mock and return an error**:
/// ```rust
/// use aws_sdk_s3::operation::get_object::GetObjectError;
/// use aws_sdk_s3::types::error::NoSuchKey;
/// use aws_sdk_s3::Client;
/// use aws_smithy_mocks_experimental::mock;
/// let get_object_error_path = mock!(Client::get_object)
///   .then_error(||GetObjectError::NoSuchKey(NoSuchKey::builder().build()));
/// ```
#[macro_export]
macro_rules! mock {
    ($operation: expr) => {
        #[allow(unreachable_code)]
        {
            $crate::RuleBuilder::new(
                // We don't actually want to run this code, so we put it in a closure. The closure
                // has the types we want which makes this whole thing type-safe (and the IDE can even
                // figure out the right input/output types in inference!)
                // The code generated here is:
                // `Client::list_buckets(todo!())`
                || $operation(todo!()).as_input().clone().build().unwrap(),
                || $operation(todo!()).send(),
            )
        }
    };
}

type MatchFn = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
type OutputFn = Arc<dyn Fn() -> Result<Output, OrchestratorError<Error>> + Send + Sync>;

impl Debug for MockResponseInterceptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} rules", self.rules.lock().unwrap().len())
    }
}

#[derive(Clone)]
enum MockOutput {
    HttpResponse(Arc<dyn Fn() -> Result<HttpResponse, BoxError> + Send + Sync>),
    ModeledResponse(OutputFn),
}

/// Interceptor which produces mock responses based on a list of rules
pub struct MockResponseInterceptor {
    rules: Arc<Mutex<VecDeque<Rule>>>,
    enforce_order: bool,
    must_match: bool,
}

impl Default for MockResponseInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RuleBuilder<I, O, E> {
    _ty: PhantomData<(I, O, E)>,
    input_filter: MatchFn,
}

impl<I, O, E> RuleBuilder<I, O, E>
where
    I: Send + Sync + Debug + 'static,
    O: Send + Sync + Debug + 'static,
    E: Send + Sync + Debug + std::error::Error + 'static,
{
    /// Creates a new [`RuleBuilder`]. This is normally constructed with the [`mock!`] macro
    pub fn new<F, R>(_input_hint: impl Fn() -> I, _output_hint: impl Fn() -> F) -> Self
    where
        F: Future<Output = Result<O, SdkError<E, R>>>,
    {
        Self {
            _ty: Default::default(),
            input_filter: Arc::new(|i: &Input| i.downcast_ref::<I>().is_some()),
        }
    }

    /// Add an additional filter to constrain which inputs match this rule.
    ///
    /// For examples, see the examples directory of this repository.
    pub fn match_requests(mut self, filter: impl Fn(&I) -> bool + Send + Sync + 'static) -> Self {
        self.input_filter = Arc::new(move |i: &Input| match i.downcast_ref::<I>() {
            Some(typed_input) => filter(typed_input),
            _ => false,
        });
        self
    }

    /// If the rule matches, then return a specific HTTP response.
    ///
    /// This is the recommended way of testing error behavior.
    pub fn then_http_response(
        self,
        response: impl Fn() -> HttpResponse + Send + Sync + 'static,
    ) -> Rule {
        Rule::new(
            self.input_filter,
            MockOutput::HttpResponse(Arc::new(move || Ok(response()))),
        )
    }

    /// If a rule matches, then return a specific output
    pub fn then_output(self, output: impl Fn() -> O + Send + Sync + 'static) -> Rule {
        Rule::new(
            self.input_filter,
            MockOutput::ModeledResponse(Arc::new(move || Ok(Output::erase(output())))),
        )
    }

    /// If a rule matches, then return a specific error
    ///
    /// Although this _basically_ works, using `then_http_response` is strongly recommended to
    /// create a higher fidelity mock. Error handling is quite complex in practice and returning errors
    /// directly often will not perfectly capture the way the error is actually returned to the SDK.
    pub fn then_error(self, output: impl Fn() -> E + Send + Sync + 'static) -> Rule {
        Rule::new(
            self.input_filter,
            MockOutput::ModeledResponse(Arc::new(move || {
                Err(OrchestratorError::operation(Error::erase(output())))
            })),
        )
    }
}

#[derive(Clone)]
pub struct Rule {
    matcher: MatchFn,
    output: MockOutput,
    used_count: Arc<AtomicUsize>,
}

impl Debug for Rule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule")
    }
}

impl Rule {
    fn new(matcher: MatchFn, output: MockOutput) -> Self {
        Self {
            matcher,
            output,
            used_count: Default::default(),
        }
    }
    fn record_usage(&self) {
        self.used_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Returns the number of times this rule has been hit.
    pub fn num_calls(&self) -> usize {
        self.used_count.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
struct ActiveRule(Rule);
impl Storable for ActiveRule {
    type Storer = StoreReplace<ActiveRule>;
}

impl MockResponseInterceptor {
    pub fn new() -> Self {
        Self {
            rules: Default::default(),
            enforce_order: false,
            must_match: true,
        }
    }
    /// Add a rule to the Interceptor
    ///
    /// Rules are matched in order—this rule will only apply if all previous rules do not match.
    pub fn with_rule(self, rule: &Rule) -> Self {
        self.rules.lock().unwrap().push_back(rule.clone());
        self
    }

    /// Require that rules are matched in order.
    ///
    /// If a rule matches out of order, the interceptor will panic.
    pub fn enforce_order(mut self) -> Self {
        self.enforce_order = true;
        self
    }

    pub fn allow_passthrough(mut self) -> Self {
        self.must_match = false;
        self
    }
}

impl Intercept for MockResponseInterceptor {
    fn name(&self) -> &'static str {
        "test"
    }

    fn modify_before_serialization(
        &self,
        context: &mut BeforeSerializationInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let mut rules = self.rules.lock().unwrap();
        let rule = match self.enforce_order {
            true => {
                let rule = rules
                    .pop_front()
                    .expect("no more rules but a new request was received");
                if !(rule.matcher)(context.input()) {
                    panic!(
                        "In order matching was enforced but the next rule did not match {:?}",
                        context.input()
                    );
                }
                Some(rule)
            }
            false => rules
                .iter()
                .find(|rule| (rule.matcher)(context.input()))
                .cloned(),
        };
        match rule {
            Some(rule) => {
                cfg.interceptor_state().store_put(ActiveRule(rule.clone()));
            }
            None => {
                if self.must_match {
                    panic!(
                        "must_match was enabled but no rules matches {:?}",
                        context.input()
                    );
                }
            }
        }
        Ok(())
    }

    fn modify_before_deserialization(
        &self,
        context: &mut BeforeDeserializationInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if let Some(rule) = cfg.load::<ActiveRule>() {
            let rule = &rule.0;
            let result = match &rule.output {
                MockOutput::HttpResponse(output_fn) => output_fn(),
                _ => return Ok(()),
            };
            rule.record_usage();

            match result {
                Ok(http_response) => *context.response_mut() = http_response,
                Err(e) => context
                    .inner_mut()
                    .set_output_or_error(Err(OrchestratorError::response(e))),
            }
        }
        Ok(())
    }

    fn modify_before_attempt_completion(
        &self,
        context: &mut FinalizerInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if let Some(rule) = _cfg.load::<ActiveRule>() {
            let rule = &rule.0;
            let result = match &rule.output {
                MockOutput::ModeledResponse(output_fn) => output_fn(),
                _ => return Ok(()),
            };

            rule.record_usage();
            if result.is_err() {
                // the orchestrator will panic of no response is present
                context.inner_mut().set_response(Response::new(
                    StatusCode::try_from(500).unwrap(),
                    SdkBody::from("stubbed error response"),
                ))
            }
            context.inner_mut().set_output_or_error(result);
        }
        Ok(())
    }
}
