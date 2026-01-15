/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{MockResponse, Rule, RuleMode};
use aws_smithy_http_client::test_util::infallible_client_fn;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::http::SharedHttpClient;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextMut, Error,
    FinalizerInterceptorContextMut, Input, Output,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::orchestrator::{HttpResponse, OrchestratorError};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// Store active rule in config bag
#[derive(Debug, Clone)]
struct ActiveRule(Rule);

impl Storable for ActiveRule {
    type Storer = StoreReplace<ActiveRule>;
}

// Store the response ID in the config bag so that we can find the proper response
#[derive(Debug, Clone)]
struct ResponseId(usize);

impl Storable for ResponseId {
    type Storer = StoreReplace<ResponseId>;
}

/// Interceptor which produces mock responses based on a list of rules
pub struct MockResponseInterceptor {
    rules: Arc<Mutex<VecDeque<Rule>>>,
    rule_mode: RuleMode,
    must_match: bool,
    active_responses: Arc<Mutex<HashMap<usize, MockResponse<Output, Error>>>>,
    /// Monotonically increasing identifier that identifies a given response
    /// so that we can store it on the request path and correctly load it back
    /// on the response path.
    current_response_id: Arc<AtomicUsize>,
}

impl fmt::Debug for MockResponseInterceptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} rules", self.rules.lock().unwrap().len())
    }
}

impl Default for MockResponseInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

impl MockResponseInterceptor {
    /// Create a new [MockResponseInterceptor]
    ///
    /// This is normally created and registered on a client through the [`mock_client`](crate::mock_client) macro.
    pub fn new() -> Self {
        Self {
            rules: Default::default(),
            rule_mode: RuleMode::MatchAny,
            must_match: true,
            active_responses: Default::default(),
            current_response_id: Default::default(),
        }
    }
    /// Add a rule to the Interceptor
    ///
    /// Rules are matched in order—this rule will only apply if all previous rules do not match.
    pub fn with_rule(self, rule: &Rule) -> Self {
        self.rules.lock().unwrap().push_back(rule.clone());
        self
    }

    /// Set the RuleMode to use when evaluating rules.
    ///
    /// See `RuleMode` enum for modes and how they are applied.
    pub fn rule_mode(mut self, rule_mode: RuleMode) -> Self {
        self.rule_mode = rule_mode;
        self
    }

    /// Allow passthrough for unmatched requests.
    ///
    /// By default, if a request doesn't match any rule, the interceptor will panic.
    /// This method allows unmatched requests to pass through.
    pub fn allow_passthrough(mut self) -> Self {
        self.must_match = false;
        self
    }
}

impl Intercept for MockResponseInterceptor {
    fn name(&self) -> &'static str {
        "MockResponseInterceptor"
    }

    fn read_before_serialization(
        &self,
        context: &BeforeSerializationInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let mut rules = self.rules.lock().unwrap();
        let input = context.inner().input().expect("input set");

        // Find a matching rule and get its response
        let mut matching_rule = None;
        let mut matching_response = None;

        match self.rule_mode {
            RuleMode::Sequential => {
                // Sequential mode requires rules match in-order
                let i = 0;
                while i < rules.len() && matching_response.is_none() {
                    let rule = &rules[i];

                    // Check if the rule is already exhausted or if it's a simple rule used once
                    //
                    // In `aws-smithy-mocks-experimental` all rules were infinite sequences
                    // but were only usable once in sequential mode. We retain that here for
                    // backwards compatibility.
                    if rule.is_exhausted() || (rule.is_simple() && rule.num_calls() > 0) {
                        // Rule is exhausted, remove it and try the next one
                        rules.remove(i);
                        continue; // Don't increment i since we removed an element
                    }

                    // Check if the rule matches
                    if !(rule.matcher)(input) {
                        // Rule doesn't match, this is an error in sequential mode
                        panic!("In order matching was enforced but rule did not match {input:?}");
                    }

                    // Rule matches and is not exhausted, get the response
                    if let Some(response) = rule.next_response(input) {
                        matching_rule = Some(rule.clone());
                        matching_response = Some(response);
                    } else {
                        // Rule is exhausted, remove it and try the next one
                        rules.remove(i);
                        continue; // Don't increment i since we removed an element
                    }

                    // We found a matching rule and got a response, so we're done
                    break;
                }
            }
            RuleMode::MatchAny => {
                // Find any matching rule with a response
                for rule in rules.iter() {
                    // Skip exhausted rules
                    if rule.is_exhausted() {
                        continue;
                    }

                    if (rule.matcher)(input) {
                        if let Some(response) = rule.next_response(input) {
                            matching_rule = Some(rule.clone());
                            matching_response = Some(response);
                            break;
                        }
                    }
                }
            }
        };

        match (matching_rule, matching_response) {
            (Some(rule), Some(response)) => {
                // Store the rule in the config bag
                cfg.interceptor_state().store_put(ActiveRule(rule));

                // we have to store the response on the interceptor, because going
                // through interceptor context requires the type to impl Clone.  to
                // find the right response for this request we generate a new monotonically
                // increasing identifier, store that on request context, and then map from
                // the response identifier to the response payload on the global interceptor
                // state.
                let response_id = self.current_response_id.fetch_add(1, Ordering::SeqCst);
                cfg.interceptor_state().store_put(ResponseId(response_id));
                let mut active_responses = self.active_responses.lock().unwrap();
                active_responses.insert(response_id, response);
            }
            _ => {
                // No matching rule or no response
                if self.must_match {
                    panic!(
                        "must_match was enabled but no rules matched or all rules were exhausted for {input:?}"
                    );
                }
            }
        }

        Ok(())
    }

    fn modify_before_transmit(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if let Some(response_id) = cfg.load::<ResponseId>() {
            let mut state = self.active_responses.lock().unwrap();
            let mut active_response = state.remove(&response_id.0);
            if active_response.is_none() {
                // in the case of retries we try to get the next response if it has been consumed
                if let Some(active_rule) = cfg.load::<ActiveRule>() {
                    // During retries, input is not available in modify_before_transmit.
                    // For HTTP status responses that don't use the input, we can use a dummy input.
                    let dummy_input = Input::doesnt_matter();
                    let next_resp = active_rule.0.next_response(&dummy_input);
                    active_response = next_resp;
                }
            }

            if let Some(resp) = active_response {
                match resp {
                    // place the http response into the extensions and let the HTTP client return it
                    MockResponse::Http(http_resp) => {
                        context
                            .request_mut()
                            .add_extension(MockHttpResponse(Arc::new(http_resp)));
                    }
                    _ => {
                        // put it back for modeled output/errors
                        state.insert(response_id.0, resp);
                    }
                }
            }
        }

        Ok(())
    }

    fn modify_before_attempt_completion(
        &self,
        context: &mut FinalizerInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        // Handle modeled responses
        if let Some(response_id) = cfg.load::<ResponseId>() {
            let mut state = self.active_responses.lock().unwrap();
            let active_response = state.remove(&response_id.0);
            if let Some(resp) = active_response {
                match resp {
                    MockResponse::Output(output) => {
                        context.inner_mut().set_output_or_error(Ok(output));
                    }
                    MockResponse::Error(error) => {
                        context
                            .inner_mut()
                            .set_output_or_error(Err(OrchestratorError::operation(error)));
                    }
                    MockResponse::Http(_) => {
                        // HTTP responses are handled by the mock HTTP client
                    }
                }
            }
        }

        Ok(())
    }
}

/// Extension for storing mock HTTP responses in request extensions
#[derive(Clone)]
struct MockHttpResponse(Arc<HttpResponse>);

/// Create a mock HTTP client that works with the interceptor using existing utilities
pub fn create_mock_http_client() -> SharedHttpClient {
    infallible_client_fn(|mut req| {
        // Try to get the mock HTTP response generator from the extensions
        if let Some(mock_response) = req.extensions_mut().remove::<MockHttpResponse>() {
            let http_resp =
                Arc::try_unwrap(mock_response.0).expect("mock HTTP response has single reference");
            return http_resp.try_into_http1x().unwrap();
        }

        // Default dummy response if no mock response is defined
        http::Response::builder()
            .status(418)
            .body(SdkBody::from("Mock HTTP client dummy response"))
            .unwrap()
    })
}

#[cfg(test)]
mod tests {
    use aws_smithy_async::rt::sleep::{SharedAsyncSleep, TokioSleep};
    use aws_smithy_runtime::client::orchestrator::operation::Operation;
    use aws_smithy_runtime::client::retries::classifiers::HttpStatusCodeClassifier;
    use aws_smithy_runtime_api::client::orchestrator::{
        HttpRequest, HttpResponse, OrchestratorError,
    };
    use aws_smithy_runtime_api::client::result::SdkError;
    use aws_smithy_runtime_api::http::StatusCode;
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::retry::RetryConfig;
    use aws_smithy_types::timeout::TimeoutConfig;

    use crate::{create_mock_http_client, MockResponseInterceptor, RuleBuilder, RuleMode};
    use std::time::Duration;

    // Simple test input and output types
    #[derive(Debug)]
    struct TestInput {
        bucket: String,
        key: String,
    }
    impl TestInput {
        fn new(bucket: &str, key: &str) -> Self {
            Self {
                bucket: bucket.to_string(),
                key: key.to_string(),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestOutput {
        content: String,
    }

    impl TestOutput {
        fn new(content: &str) -> Self {
            Self {
                content: content.to_string(),
            }
        }
    }

    #[derive(Debug)]
    struct TestError {
        message: String,
    }

    impl TestError {
        fn new(message: &str) -> Self {
            Self {
                message: message.to_string(),
            }
        }
    }

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }

    impl std::error::Error for TestError {}

    // Helper function to create a RuleBuilder with proper type hints
    fn create_rule_builder() -> RuleBuilder<TestInput, TestOutput, TestError> {
        RuleBuilder::new_from_mock(
            || TestInput {
                bucket: "".to_string(),
                key: "".to_string(),
            },
            || {
                let fut: std::future::Ready<Result<TestOutput, SdkError<TestError, HttpResponse>>> =
                    std::future::ready(Ok(TestOutput {
                        content: "".to_string(),
                    }));
                fut
            },
        )
    }

    // Helper function to create an Operation with common configuration
    fn create_test_operation(
        interceptor: MockResponseInterceptor,
        enable_retries: bool,
    ) -> Operation<TestInput, TestOutput, TestError> {
        let builder = Operation::builder()
            .service_name("test")
            .operation_name("test")
            .http_client(create_mock_http_client())
            .endpoint_url("http://localhost:1234")
            .no_auth()
            .sleep_impl(SharedAsyncSleep::new(TokioSleep::new()))
            .timeout_config(TimeoutConfig::disabled())
            .interceptor(interceptor)
            .serializer(|input: TestInput| {
                let mut request = HttpRequest::new(SdkBody::empty());
                request
                    .set_uri(format!("/{}/{}", input.bucket, input.key))
                    .expect("valid URI");
                Ok(request)
            })
            .deserializer::<TestOutput, TestError>(|response| {
                if response.status().is_success() {
                    let body = std::str::from_utf8(response.body().bytes().unwrap())
                        .unwrap_or("empty body")
                        .to_string();
                    Ok(TestOutput { content: body })
                } else {
                    Err(OrchestratorError::operation(TestError {
                        message: format!("Error: {}", response.status()),
                    }))
                }
            });

        if enable_retries {
            let retry_config = RetryConfig::standard()
                .with_max_attempts(5)
                .with_initial_backoff(Duration::from_millis(1))
                .with_max_backoff(Duration::from_millis(5));

            builder
                .retry_classifier(HttpStatusCodeClassifier::default())
                .standard_retry(&retry_config)
                .build()
        } else {
            builder.no_retry().build()
        }
    }

    #[tokio::test]
    async fn test_retry_sequence() {
        // Create a rule with repeated error responses followed by success
        let rule = create_rule_builder()
            .match_requests(|input| input.bucket == "test-bucket" && input.key == "test-key")
            .sequence()
            .http_status(503, None)
            .times(2)
            .output(|| TestOutput::new("success after retries"))
            .build();

        // Create an interceptor with the rule
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, true);

        // Make a single request - it should automatically retry through the sequence
        let result = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;

        // Should succeed with the final output after retries
        assert!(
            result.is_ok(),
            "Expected success but got error: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            TestOutput {
                content: "success after retries".to_string()
            }
        );

        // Verify the rule was used the expected number of times (all 4 responses: 2 errors + 1 success)
        assert_eq!(rule.num_calls(), 3);
    }

    #[tokio::test]
    async fn test_compute_output() {
        // Create a rule that computes its responses based off of input data
        let rule = create_rule_builder()
            .match_requests(|input| input.bucket == "test-bucket" && input.key == "test-key")
            .then_compute_output(|input| TestOutput {
                content: format!("{}.{}", input.bucket, input.key),
            });

        // Create an interceptor with the rule
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, true);

        let result = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;

        // Should succeed with the output derived from input
        assert!(
            result.is_ok(),
            "Expected success but got error: {:?}",
            result.err()
        );
        assert_eq!(
            result.unwrap(),
            TestOutput {
                content: "test-bucket.test-key".to_string()
            }
        );

        // Verify the rule was used once, no retries
        assert_eq!(rule.num_calls(), 1);
    }

    #[should_panic(
        expected = "must_match was enabled but no rules matched or all rules were exhausted for"
    )]
    #[tokio::test]
    async fn test_exhausted_rules_sequential() {
        // Create a rule with a single response
        let rule = create_rule_builder().then_output(|| TestOutput::new("only response"));

        // Create an interceptor with the rule
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, false);

        // First call should succeed
        let result1 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result1.is_ok());

        // Second call should panic because the rules are exhausted
        let _result2 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
    }

    #[tokio::test]
    async fn test_rule_mode_match_any() {
        // Create two rules with different matchers
        let rule1 = create_rule_builder()
            .match_requests(|input| input.bucket == "bucket1")
            .then_output(|| TestOutput::new("response1"));

        let rule2 = create_rule_builder()
            .match_requests(|input| input.bucket == "bucket2")
            .then_output(|| TestOutput::new("response2"));

        // Create an interceptor with both rules in MatchAny mode
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::MatchAny)
            .with_rule(&rule1)
            .with_rule(&rule2);

        let operation = create_test_operation(interceptor, false);

        // Call with bucket1 should match rule1
        let result1 = operation
            .invoke(TestInput::new("bucket1", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("response1"));

        // Call with bucket2 should match rule2
        let result2 = operation
            .invoke(TestInput::new("bucket2", "test-key"))
            .await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), TestOutput::new("response2"));

        // Verify the rules were used the expected number of times
        assert_eq!(rule1.num_calls(), 1);
        assert_eq!(rule2.num_calls(), 1);

        // Calling with bucket1 again should match rule1 a second time
        let result1 = operation
            .invoke(TestInput::new("bucket1", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("response1"));
        assert_eq!(rule1.num_calls(), 2);
    }

    #[tokio::test]
    async fn test_mixed_response_types() {
        // Create a rule with all three types of responses
        let rule = create_rule_builder()
            .sequence()
            .output(|| TestOutput::new("first output"))
            .error(|| TestError::new("expected error"))
            .http_response(|| {
                HttpResponse::new(
                    StatusCode::try_from(200).unwrap(),
                    SdkBody::from("http response"),
                )
            })
            .build();

        // Create an interceptor with the rule
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, false);

        // First call should return the modeled output
        let result1 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("first output"));

        // Second call should return the modeled error
        let result2 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result2.is_err());
        let sdk_err = result2.unwrap_err();
        let err = sdk_err.as_service_error().expect("expected service error");
        assert_eq!(err.to_string(), "expected error");

        // Third call should return the HTTP response
        let result3 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), TestOutput::new("http response"));

        // Verify the rule was used the expected number of times
        assert_eq!(rule.num_calls(), 3);
    }
    #[tokio::test]
    async fn test_exhausted_sequence_match_any() {
        // Create a rule with a sequence that will be exhausted
        let rule = create_rule_builder()
            .match_requests(|input| input.bucket == "bucket-1")
            .sequence()
            .output(|| TestOutput::new("response 1"))
            .output(|| TestOutput::new("response 2"))
            .build();

        // Create another rule to use after the first one is exhausted
        let fallback_rule =
            create_rule_builder().then_output(|| TestOutput::new("fallback response"));

        // Create an interceptor with both rules
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::MatchAny)
            .with_rule(&rule)
            .with_rule(&fallback_rule);

        let operation = create_test_operation(interceptor, false);

        // First two calls should use the first rule
        let result1 = operation
            .invoke(TestInput::new("bucket-1", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("response 1"));

        // second should use our fallback rule
        let result2 = operation
            .invoke(TestInput::new("other-bucket", "test-key"))
            .await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), TestOutput::new("fallback response"));

        // Third call should use the first rule again and exhaust it
        let result3 = operation
            .invoke(TestInput::new("bucket-1", "test-key"))
            .await;
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), TestOutput::new("response 2"));

        // first rule is exhausted so the matcher shouldn't matter and we should hit our fallback rule
        let result4 = operation
            .invoke(TestInput::new("bucket-1", "test-key"))
            .await;
        assert!(result4.is_ok());
        assert_eq!(result4.unwrap(), TestOutput::new("fallback response"));

        // Verify the rules were used the expected number of times
        assert_eq!(rule.num_calls(), 2);
        assert_eq!(fallback_rule.num_calls(), 2);
    }

    #[tokio::test]
    async fn test_exhausted_sequence_sequential() {
        // Create a rule with a sequence that will be exhausted
        let rule = create_rule_builder()
            .sequence()
            .output(|| TestOutput::new("response 1"))
            .output(|| TestOutput::new("response 2"))
            .build();

        // Create another rule to use after the first one is exhausted
        let fallback_rule =
            create_rule_builder().then_output(|| TestOutput::new("fallback response"));

        // Create an interceptor with both rules
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule)
            .with_rule(&fallback_rule);

        let operation = create_test_operation(interceptor, false);

        // First two calls should use the first rule
        let result1 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("response 1"));

        let result2 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), TestOutput::new("response 2"));

        // Third call should use the fallback rule
        let result3 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), TestOutput::new("fallback response"));

        // Verify the rules were used the expected number of times
        assert_eq!(rule.num_calls(), 2);
        assert_eq!(fallback_rule.num_calls(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_usage() {
        use std::sync::Arc;
        use tokio::task;

        // Create a rule with multiple responses
        let rule = Arc::new(
            create_rule_builder()
                .sequence()
                .output(|| TestOutput::new("response 1"))
                .output(|| TestOutput::new("response 2"))
                .output(|| TestOutput::new("response 3"))
                .build(),
        );

        // Create an interceptor with the rule
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = Arc::new(create_test_operation(interceptor, false));

        // Spawn multiple tasks that use the operation concurrently
        let mut handles = vec![];
        for i in 0..3 {
            let op = operation.clone();
            let handle = task::spawn(async move {
                let result = op
                    .invoke(TestInput::new(&format!("bucket-{i}"), "test-key"))
                    .await;
                result.unwrap()
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Sort the results to make the test deterministic
        results.sort_by(|a, b| a.content.cmp(&b.content));

        // Verify we got all three responses
        assert_eq!(results.len(), 3);
        assert_eq!(results[0], TestOutput::new("response 1"));
        assert_eq!(results[1], TestOutput::new("response 2"));
        assert_eq!(results[2], TestOutput::new("response 3"));

        // Verify the rule was used the expected number of times
        assert_eq!(rule.num_calls(), 3);
    }

    #[tokio::test]
    async fn test_sequential_rule_removal() {
        // Create a rule that matches only when key != "correct-key"
        let rule1 = create_rule_builder()
            .match_requests(|input| input.bucket == "test-bucket" && input.key != "correct-key")
            .then_http_response(|| {
                HttpResponse::new(
                    StatusCode::try_from(404).unwrap(),
                    SdkBody::from("not found"),
                )
            });

        // Create a rule that matches only when key == "correct-key"
        let rule2 = create_rule_builder()
            .match_requests(|input| input.bucket == "test-bucket" && input.key == "correct-key")
            .then_output(|| TestOutput::new("success"));

        // Create an interceptor with both rules in Sequential mode
        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule1)
            .with_rule(&rule2);

        let operation = create_test_operation(interceptor, true);

        // First call with key="foo" should match rule1
        let result1 = operation.invoke(TestInput::new("test-bucket", "foo")).await;
        assert!(result1.is_err());
        assert_eq!(rule1.num_calls(), 1);

        // Second call with key="correct-key" should match rule2
        // But this will fail if rule1 is not removed after being used
        let result2 = operation
            .invoke(TestInput::new("test-bucket", "correct-key"))
            .await;

        // This should succeed, rule1 doesn't match but should have been removed
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), TestOutput::new("success"));
        assert_eq!(rule2.num_calls(), 1);
    }

    #[tokio::test]
    async fn test_simple_rule_in_match_any_mode() {
        let rule = create_rule_builder().then_output(|| TestOutput::new("simple response"));

        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::MatchAny)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, false);

        for i in 0..5 {
            let result = operation
                .invoke(TestInput::new("test-bucket", "test-key"))
                .await;
            assert!(result.is_ok(), "Call {i} should succeed");
            assert_eq!(result.unwrap(), TestOutput::new("simple response"));
        }
        assert_eq!(rule.num_calls(), 5);
        assert!(!rule.is_exhausted());
    }

    #[tokio::test]
    async fn test_simple_rule_in_sequential_mode() {
        let rule1 = create_rule_builder().then_output(|| TestOutput::new("first response"));
        let rule2 = create_rule_builder().then_output(|| TestOutput::new("second response"));

        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule1)
            .with_rule(&rule2);

        let operation = create_test_operation(interceptor, false);

        let result1 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("first response"));

        // Second call should use rule2 (rule1 should be removed after one use in Sequential mode)
        let result2 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), TestOutput::new("second response"));

        assert_eq!(rule1.num_calls(), 1);
        assert_eq!(rule2.num_calls(), 1);
    }

    #[tokio::test]
    async fn test_repeatedly_method() {
        let rule = create_rule_builder()
            .sequence()
            .output(|| TestOutput::new("first response"))
            .output(|| TestOutput::new("repeated response"))
            .repeatedly()
            .build();

        let interceptor = MockResponseInterceptor::new()
            .rule_mode(RuleMode::Sequential)
            .with_rule(&rule);

        let operation = create_test_operation(interceptor, false);

        let result1 = operation
            .invoke(TestInput::new("test-bucket", "test-key"))
            .await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), TestOutput::new("first response"));

        // all subsequent calls should return "repeated response"
        for i in 0..10 {
            let result = operation
                .invoke(TestInput::new("test-bucket", "test-key"))
                .await;
            assert!(result.is_ok(), "Call {i} should succeed");
            assert_eq!(result.unwrap(), TestOutput::new("repeated response"));
        }
        assert_eq!(rule.num_calls(), 11);
        assert!(!rule.is_exhausted());
    }

    #[should_panic(expected = "times(n) called before adding a response to the sequence")]
    #[test]
    fn test_times_validation() {
        // This should panic because times() is called before adding any responses
        let _rule = create_rule_builder()
            .sequence()
            .times(3)
            .output(|| TestOutput::new("response"))
            .build();
    }

    #[should_panic(expected = "repeatedly() called before adding a response to the sequence")]
    #[test]
    fn test_repeatedly_validation() {
        // This should panic because repeatedly() is called before adding any responses
        let _rule = create_rule_builder().sequence().repeatedly().build();
    }

    #[test]
    fn test_total_responses_overflow() {
        // Create a rule with a large number of repetitions to test overflow handling
        let rule = create_rule_builder()
            .sequence()
            .output(|| TestOutput::new("response"))
            .times(usize::MAX / 2)
            .output(|| TestOutput::new("another response"))
            .repeatedly()
            .build();
        assert_eq!(rule.max_responses, usize::MAX);
    }

    #[tokio::test]
    async fn test_compute_response_conditional() {
        use crate::MockResponse;

        let rule = create_rule_builder().then_compute_response(|input| {
            if input.key == "error-key" {
                MockResponse::Error(TestError::new("conditional error"))
            } else {
                MockResponse::Output(TestOutput::new(&format!("response for {}", input.key)))
            }
        });

        let interceptor = MockResponseInterceptor::new().with_rule(&rule);
        let operation = create_test_operation(interceptor, false);

        // Test success case
        let result = operation
            .invoke(TestInput::new("test-bucket", "success-key"))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), TestOutput::new("response for success-key"));

        // Test error case
        let result = operation
            .invoke(TestInput::new("test-bucket", "error-key"))
            .await;
        assert!(result.is_err());
        assert_eq!(rule.num_calls(), 2);
    }
}
