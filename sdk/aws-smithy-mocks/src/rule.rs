/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::context::{Error, Input, Output};
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_runtime_api::http::StatusCode;
use aws_smithy_types::body::SdkBody;
use std::fmt;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// A mock response that can be returned by a rule.
///
/// This enum represents the different types of responses that can be returned by a mock rule:
/// - `Output`: A successful modeled response
/// - `Error`: A modeled error
/// - `Http`: An HTTP response
///
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MockResponse<O, E> {
    /// A successful modeled response.
    Output(O),
    /// A modeled error.
    Error(E),
    /// An HTTP response.
    Http(HttpResponse),
}

/// A function that matches requests.
type MatchFn = Arc<dyn Fn(&Input) -> bool + Send + Sync>;
type ServeFn = Arc<dyn Fn(usize, &Input) -> Option<MockResponse<Output, Error>> + Send + Sync>;

/// A rule for matching requests and providing mock responses.
///
/// Rules are created using the `mock!` macro or the `RuleBuilder`.
///
#[derive(Clone)]
pub struct Rule {
    /// Function that determines if this rule matches a request.
    pub(crate) matcher: MatchFn,

    /// Handler function that generates responses.
    response_handler: ServeFn,

    /// Number of times this rule has been called.
    call_count: Arc<AtomicUsize>,

    /// Maximum number of responses this rule will provide.
    pub(crate) max_responses: usize,

    /// Flag indicating this is a "simple" rule which changes how it is interpreted
    /// depending on the RuleMode.
    ///
    /// See [smithy-rs#4135](https://github.com/smithy-lang/smithy-rs/issues/4135)
    is_simple: bool,
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule")
    }
}

impl Rule {
    /// Creates a new rule with the given matcher, response handler, and max responses.
    #[allow(clippy::type_complexity)]
    pub(crate) fn new<O, E>(
        matcher: MatchFn,
        response_handler: Arc<dyn Fn(usize, &Input) -> Option<MockResponse<O, E>> + Send + Sync>,
        max_responses: usize,
        is_simple: bool,
    ) -> Self
    where
        O: fmt::Debug + Send + Sync + 'static,
        E: fmt::Debug + Send + Sync + std::error::Error + 'static,
    {
        Rule {
            matcher,
            response_handler: Arc::new(move |idx: usize, input: &Input| {
                if idx < max_responses {
                    response_handler(idx, input).map(|resp| match resp {
                        MockResponse::Output(o) => MockResponse::Output(Output::erase(o)),
                        MockResponse::Error(e) => MockResponse::Error(Error::erase(e)),
                        MockResponse::Http(http_resp) => MockResponse::Http(http_resp),
                    })
                } else {
                    None
                }
            }),
            call_count: Arc::new(AtomicUsize::new(0)),
            max_responses,
            is_simple,
        }
    }

    /// Test if this is a "simple" rule (non-sequenced)
    pub(crate) fn is_simple(&self) -> bool {
        self.is_simple
    }

    /// Gets the next response.
    pub(crate) fn next_response(&self, input: &Input) -> Option<MockResponse<Output, Error>> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        (self.response_handler)(idx, input)
    }

    /// Returns the number of times this rule has been called.
    pub fn num_calls(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }

    /// Checks if this rule is exhausted (has provided all its responses).
    pub fn is_exhausted(&self) -> bool {
        self.num_calls() >= self.max_responses
    }
}

/// RuleMode describes how rules will be interpreted.
/// - In RuleMode::MatchAny, the first matching rule will be applied, and the rules will remain unchanged.
/// - In RuleMode::Sequential, the first matching rule will be applied, and that rule will be removed from the list of rules **once it is exhausted**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleMode {
    /// Match rules in the order they were added. The first matching rule will be applied and the
    /// rules will remain unchanged
    Sequential,
    /// The first matching rule will be applied, and that rule will be removed from the list of rules
    /// **once it is exhausted**. Each rule can have multiple responses, and all responses in a rule
    /// will be consumed before moving to the next rule.
    MatchAny,
}

/// A builder for creating rules.
///
/// This builder provides a fluent API for creating rules with different response types.
///
pub struct RuleBuilder<I, O, E> {
    /// Function that determines if this rule matches a request.
    pub(crate) input_filter: MatchFn,

    /// Phantom data for the input type.
    pub(crate) _ty: std::marker::PhantomData<(I, O, E)>,
}

impl<I, O, E> RuleBuilder<I, O, E>
where
    I: fmt::Debug + Send + Sync + 'static,
    O: fmt::Debug + Send + Sync + 'static,
    E: fmt::Debug + Send + Sync + std::error::Error + 'static,
{
    /// Creates a new [`RuleBuilder`]
    #[doc(hidden)]
    pub fn new() -> Self {
        RuleBuilder {
            input_filter: Arc::new(|i: &Input| i.downcast_ref::<I>().is_some()),
            _ty: std::marker::PhantomData,
        }
    }

    /// Creates a new [`RuleBuilder`]. This is normally constructed with the [`mock!`](crate::mock!) macro
    #[doc(hidden)]
    pub fn new_from_mock<F, R>(_input_hint: impl Fn() -> I, _output_hint: impl Fn() -> F) -> Self
    where
        F: Future<Output = Result<O, SdkError<E, R>>>,
    {
        Self {
            input_filter: Arc::new(|i: &Input| i.downcast_ref::<I>().is_some()),
            _ty: Default::default(),
        }
    }

    /// Sets the function that determines if this rule matches a request.
    pub fn match_requests<F>(mut self, filter: F) -> Self
    where
        F: Fn(&I) -> bool + Send + Sync + 'static,
    {
        self.input_filter = Arc::new(move |i: &Input| match i.downcast_ref::<I>() {
            Some(typed_input) => filter(typed_input),
            _ => false,
        });
        self
    }

    /// Start building a response sequence
    ///
    /// A sequence allows a single rule to generate multiple responses which can
    /// be used to test retry behavior.
    ///
    /// # Examples
    ///
    /// With repetition using `times()`:
    ///
    /// ```rust,ignore
    /// let rule = mock!(Client::get_object)
    ///     .sequence()
    ///     .http_status(503, None)
    ///     .times(2)                                        // First two calls return 503
    ///     .output(|| GetObjectOutput::builder().build())   // Third call succeeds
    ///     .build();
    /// ```
    pub fn sequence(self) -> ResponseSequenceBuilder<I, O, E> {
        ResponseSequenceBuilder::new(self.input_filter)
    }

    /// Creates a rule that returns a modeled output.
    pub fn then_output<F>(self, output_fn: F) -> Rule
    where
        F: Fn() -> O + Send + Sync + 'static,
    {
        self.sequence().output(output_fn).build_simple()
    }

    /// Creates a rule that returns a modeled error.
    pub fn then_error<F>(self, error_fn: F) -> Rule
    where
        F: Fn() -> E + Send + Sync + 'static,
    {
        self.sequence().error(error_fn).build_simple()
    }

    /// Creates a rule that returns an HTTP response.
    pub fn then_http_response<F>(self, response_fn: F) -> Rule
    where
        F: Fn() -> HttpResponse + Send + Sync + 'static,
    {
        self.sequence().http_response(response_fn).build_simple()
    }

    /// Creates a rule that computes an output based on the input.
    ///
    /// This allows generating responses based on the input request.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let rule = mock!(Client::get_object)
    ///     .compute_output(|req| {
    ///         GetObjectOutput::builder()
    ///             .body(ByteStream::from_static(format!("content for {}", req.key().unwrap_or("unknown")).as_bytes()))
    ///             .build()
    ///     })
    ///     .build();
    /// ```
    pub fn then_compute_output<F>(self, compute_fn: F) -> Rule
    where
        F: Fn(&I) -> O + Send + Sync + 'static,
    {
        self.sequence().compute_output(compute_fn).build_simple()
    }

    /// Creates a rule that computes an arbitrary response based on the input.
    ///
    /// This allows generating any type of response (output, error, or HTTP) based on the input request.
    /// Unlike `then_compute_output`, this method can return errors or HTTP responses conditionally.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let rule = mock!(Client::get_object)
    ///     .then_compute_response(|req| {
    ///         if req.key() == Some("error") {
    ///             MockResponse::Error(GetObjectError::NoSuchKey(NoSuchKey::builder().build()))
    ///         } else {
    ///             MockResponse::Output(GetObjectOutput::builder()
    ///                 .body(ByteStream::from_static(b"content"))
    ///                 .build())
    ///         }
    ///     })
    ///     .build();
    /// ```
    pub fn then_compute_response<F>(self, compute_fn: F) -> Rule
    where
        F: Fn(&I) -> MockResponse<O, E> + Send + Sync + 'static,
    {
        self.sequence().compute_response(compute_fn).build_simple()
    }
}

type SequenceGeneratorFn<O, E> = Arc<dyn Fn(&Input) -> MockResponse<O, E> + Send + Sync>;

/// A builder for creating response sequences
pub struct ResponseSequenceBuilder<I, O, E> {
    /// The response generators in the sequence
    generators: Vec<(SequenceGeneratorFn<O, E>, usize)>,

    /// Function that determines if this rule matches a request
    input_filter: MatchFn,

    /// flag indicating this is a "simple" rule
    is_simple: bool,

    /// Marker for the input, output, and error types
    _marker: std::marker::PhantomData<I>,
}

/// Final sequence builder state  - can only `build()`
pub struct FinalizedResponseSequenceBuilder<I, O, E> {
    inner: ResponseSequenceBuilder<I, O, E>,
}

impl<I, O, E> ResponseSequenceBuilder<I, O, E>
where
    I: fmt::Debug + Send + Sync + 'static,
    O: fmt::Debug + Send + Sync + 'static,
    E: fmt::Debug + Send + Sync + std::error::Error + 'static,
{
    /// Create a new response sequence builder
    pub(crate) fn new(input_filter: MatchFn) -> Self {
        Self {
            generators: Vec::new(),
            input_filter,
            is_simple: false,
            _marker: std::marker::PhantomData,
        }
    }

    /// Add a modeled output response to the sequence
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let rule = mock!(Client::get_object)
    ///     .sequence()
    ///     .output(|| GetObjectOutput::builder().build())
    ///     .build();
    /// ```
    pub fn output<F>(mut self, output_fn: F) -> Self
    where
        F: Fn() -> O + Send + Sync + 'static,
    {
        let generator = Arc::new(move |_input: &Input| MockResponse::Output(output_fn()));
        self.generators.push((generator, 1));
        self
    }

    /// Add a modeled error response to the sequence
    pub fn error<F>(mut self, error_fn: F) -> Self
    where
        F: Fn() -> E + Send + Sync + 'static,
    {
        let generator = Arc::new(move |_input: &Input| MockResponse::Error(error_fn()));
        self.generators.push((generator, 1));
        self
    }

    /// Add an HTTP status code response to the sequence
    pub fn http_status(mut self, status: u16, body: Option<String>) -> Self {
        let status_code = StatusCode::try_from(status).unwrap();

        let generator: SequenceGeneratorFn<O, E> = match body {
            Some(body) => Arc::new(move |_input: &Input| {
                MockResponse::Http(HttpResponse::new(status_code, SdkBody::from(body.clone())))
            }),
            None => Arc::new(move |_input: &Input| {
                MockResponse::Http(HttpResponse::new(status_code, SdkBody::empty()))
            }),
        };

        self.generators.push((generator, 1));
        self
    }

    /// Add an HTTP response to the sequence
    pub fn http_response<F>(mut self, response_fn: F) -> Self
    where
        F: Fn() -> HttpResponse + Send + Sync + 'static,
    {
        let generator = Arc::new(move |_input: &Input| MockResponse::Http(response_fn()));
        self.generators.push((generator, 1));
        self
    }

    /// Add a computed output response to the sequence.  Note that this is not `pub`
    /// because creating computed output rules off of sequenced rules doesn't work,
    /// as we can't preserve the input across retries.  So we only expose `compute_output`
    /// on unsequenced rules above.
    fn compute_output<F>(mut self, compute_fn: F) -> Self
    where
        F: Fn(&I) -> O + Send + Sync + 'static,
    {
        let generator = Arc::new(move |input: &Input| {
            if let Some(typed_input) = input.downcast_ref::<I>() {
                MockResponse::Output(compute_fn(typed_input))
            } else {
                panic!("Input type mismatch in compute_output")
            }
        });
        self.generators.push((generator, 1));
        self
    }

    /// Add a computed response to the sequence. Not `pub` for same reason as `compute_output`.
    fn compute_response<F>(mut self, compute_fn: F) -> Self
    where
        F: Fn(&I) -> MockResponse<O, E> + Send + Sync + 'static,
    {
        let generator = Arc::new(move |input: &Input| {
            if let Some(typed_input) = input.downcast_ref::<I>() {
                compute_fn(typed_input)
            } else {
                panic!("Input type mismatch in compute_response")
            }
        });
        self.generators.push((generator, 1));
        self
    }

    /// Repeat the last added response multiple times.
    ///
    /// This method sets the number of times the last response in the sequence will be used.
    /// For example, if you add a response and then call `times(3)`, that response will be
    /// returned for the next 3 calls to the rule.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a rule that returns 503 twice, then succeeds
    /// let rule = mock!(Client::get_object)
    ///     .sequence()
    ///     .http_status(503, None)
    ///     .times(2)                                        // First two calls return 503
    ///     .output(|| GetObjectOutput::builder().build())   // Third call succeeds
    ///     .build();
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Called with a count of 0
    /// - Called before adding any responses to the sequence
    pub fn times(mut self, count: usize) -> Self {
        if self.generators.is_empty() {
            panic!("times(n) called before adding a response to the sequence");
        }
        match count {
            0 => panic!("repeat count must be greater than zero"),
            1 => {
                return self;
            }
            _ => {}
        }

        // update the repeat count of the last generator
        if let Some(last_generator) = self.generators.last_mut() {
            last_generator.1 = count;
        }
        self
    }
    /// Make the last response in the sequence repeat indefinitely.
    ///
    /// This method causes the last response added to the sequence to be repeated
    /// forever, making the rule never exhaust. After calling `repeatedly()`,
    /// no more responses can be added to the sequence.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // Create a rule that returns an error once, then succeeds forever
    /// let rule = mock!(Client::get_object)
    ///     .sequence()
    ///     .error(|| GetObjectError::NoSuchKey(NoSuchKey::builder().build()))
    ///     .output(|| GetObjectOutput::builder().build())
    ///     .repeatedly()
    ///     .build();
    ///
    /// // First call will return NoSuchKey error
    /// // All subsequent calls will return success
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if called before adding any responses to the sequence.
    pub fn repeatedly(self) -> FinalizedResponseSequenceBuilder<I, O, E> {
        if self.generators.is_empty() {
            panic!("repeatedly() called before adding a response to the sequence");
        }
        let inner = self.times(usize::MAX);
        FinalizedResponseSequenceBuilder { inner }
    }

    /// Build this a "simple" rule (internal detail)
    pub(crate) fn build_simple(mut self) -> Rule {
        self.is_simple = true;
        self.repeatedly().build()
    }

    /// Build the rule with this response sequence
    pub fn build(self) -> Rule {
        let generators = self.generators;
        let is_simple = self.is_simple;

        // calculate total responses (sum of all repetitions)
        let total_responses: usize = generators
            .iter()
            .map(|(_, count)| *count)
            .fold(0, |acc, count| acc.saturating_add(count));

        Rule::new(
            self.input_filter,
            Arc::new(move |idx, input| {
                // find which generator to use
                let mut current_idx = idx;
                for (generator, repeat_count) in &generators {
                    if current_idx < *repeat_count {
                        return Some(generator(input));
                    }
                    current_idx -= repeat_count;
                }
                None
            }),
            total_responses,
            is_simple,
        )
    }
}

impl<I, O, E> FinalizedResponseSequenceBuilder<I, O, E>
where
    I: fmt::Debug + Send + Sync + 'static,
    O: fmt::Debug + Send + Sync + 'static,
    E: fmt::Debug + Send + Sync + std::error::Error + 'static,
{
    /// Build the rule with this response sequence
    pub fn build(self) -> Rule {
        self.inner.build()
    }
}
