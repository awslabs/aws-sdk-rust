/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
use aws_smithy_runtime_api::client::retries::classifiers::{
    ClassifyRetry, RetryAction, RetryClassifierPriority, RetryReason,
};
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ErrorKind;
use std::borrow::Cow;
use std::error::Error as StdError;
use std::marker::PhantomData;

/// Parse the AWS-specific `x-amz-retry-after` response header (milliseconds) into
/// a [`Duration`](std::time::Duration). Returns `None` when the header is absent
/// or does not parse as an integer, in which case the SDK falls back to
/// exponential backoff.
fn retry_after_from_ctx(ctx: &InterceptorContext) -> Option<std::time::Duration> {
    ctx.response()
        .and_then(|res| res.headers().get("x-amz-retry-after"))
        .and_then(|header| header.parse::<u64>().ok())
        .map(std::time::Duration::from_millis)
}

/// AWS error codes that represent throttling errors.
pub const THROTTLING_ERRORS: &[&str] = &[
    "Throttling",
    "ThrottlingException",
    "ThrottledException",
    "RequestThrottledException",
    "TooManyRequestsException",
    "ProvisionedThroughputExceededException",
    "TransactionInProgressException",
    "RequestLimitExceeded",
    "BandwidthLimitExceeded",
    "LimitExceededException",
    "RequestThrottled",
    "SlowDown",
    "PriorRequestNotComplete",
    "EC2ThrottledException",
];

/// AWS error codes that represent transient errors.
pub const TRANSIENT_ERRORS: &[&str] = &["RequestTimeout", "RequestTimeoutException"];

/// A retry classifier for determining if the response sent by an AWS service requires a retry.
#[derive(Debug)]
pub struct AwsErrorCodeClassifier<E> {
    throttling_errors: Cow<'static, [&'static str]>,
    transient_errors: Cow<'static, [&'static str]>,
    _inner: PhantomData<E>,
}

impl<E> Default for AwsErrorCodeClassifier<E> {
    fn default() -> Self {
        Self {
            throttling_errors: THROTTLING_ERRORS.into(),
            transient_errors: TRANSIENT_ERRORS.into(),
            _inner: PhantomData,
        }
    }
}

/// Builder for [`AwsErrorCodeClassifier`]
#[derive(Debug)]
pub struct AwsErrorCodeClassifierBuilder<E> {
    throttling_errors: Option<Cow<'static, [&'static str]>>,
    transient_errors: Option<Cow<'static, [&'static str]>>,
    _inner: PhantomData<E>,
}

impl<E> AwsErrorCodeClassifierBuilder<E> {
    /// Set `transient_errors` for the builder
    pub fn transient_errors(
        mut self,
        transient_errors: impl Into<Cow<'static, [&'static str]>>,
    ) -> Self {
        self.transient_errors = Some(transient_errors.into());
        self
    }

    /// Build a new [`AwsErrorCodeClassifier`]
    pub fn build(self) -> AwsErrorCodeClassifier<E> {
        AwsErrorCodeClassifier {
            throttling_errors: self.throttling_errors.unwrap_or(THROTTLING_ERRORS.into()),
            transient_errors: self.transient_errors.unwrap_or(TRANSIENT_ERRORS.into()),
            _inner: self._inner,
        }
    }
}

impl<E> AwsErrorCodeClassifier<E> {
    /// Create a new [`AwsErrorCodeClassifier`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a builder that can create a new [`AwsErrorCodeClassifier`]
    pub fn builder() -> AwsErrorCodeClassifierBuilder<E> {
        AwsErrorCodeClassifierBuilder {
            throttling_errors: None,
            transient_errors: None,
            _inner: PhantomData,
        }
    }
}

impl<E> ClassifyRetry for AwsErrorCodeClassifier<E>
where
    E: StdError + ProvideErrorMetadata + Send + Sync + 'static,
{
    fn classify_retry(&self, ctx: &InterceptorContext) -> RetryAction {
        // Check for a result
        let output_or_error = ctx.output_or_error();
        // Check for an error
        let error = match output_or_error {
            Some(Ok(_)) | None => return RetryAction::NoActionIndicated,
            Some(Err(err)) => err,
        };

        let retry_after = retry_after_from_ctx(ctx);

        let error_code = OrchestratorError::as_operation_error(error)
            .and_then(|err| err.downcast_ref::<E>())
            .and_then(|err| err.code());

        if let Some(error_code) = error_code {
            if self.throttling_errors.contains(&error_code) {
                return RetryAction::RetryIndicated(RetryReason::RetryableError {
                    kind: ErrorKind::ThrottlingError,
                    retry_after,
                });
            }
            if self.transient_errors.contains(&error_code) {
                return RetryAction::RetryIndicated(RetryReason::RetryableError {
                    kind: ErrorKind::TransientError,
                    retry_after,
                });
            }
        };

        RetryAction::NoActionIndicated
    }

    fn classify_retry_v2(&self, ctx: &InterceptorContext, previous: &RetryAction) -> RetryAction {
        // First, run our own error-code-based classification. When it
        // recognizes an AWS error code it already attaches x-amz-retry-after,
        // so that result stands unchanged.
        let own = self.classify_retry(ctx);
        if own != RetryAction::NoActionIndicated {
            return own;
        }

        // No modeled error code matched. Per the Retry Behavior 2.1 spec,
        // x-amz-retry-after MUST be incorporated into the backoff for ANY
        // retryable error -- including transient errors classified purely by
        // HTTP status (e.g. a bare 500) by HttpStatusCodeClassifier, which
        // cannot read this AWS-specific header. If an earlier-running
        // classifier already marked the response retryable but without an
        // explicit delay, tuck the server-directed delay into that verdict,
        // preserving the upstream error kind. The header alone does not make a
        // response retryable, so we only refine an already-retryable `previous`;
        // an unparsable header leaves retry_after as None and the request falls
        // back to exponential backoff.
        if let RetryAction::RetryIndicated(RetryReason::RetryableError {
            kind,
            retry_after: None,
        }) = previous
        {
            if let Some(retry_after) = retry_after_from_ctx(ctx) {
                return RetryAction::RetryIndicated(RetryReason::RetryableError {
                    kind: *kind,
                    retry_after: Some(retry_after),
                });
            }
        }

        RetryAction::NoActionIndicated
    }

    fn name(&self) -> &'static str {
        "AWS Error Code"
    }

    fn priority(&self) -> RetryClassifierPriority {
        RetryClassifierPriority::run_before(
            RetryClassifierPriority::modeled_as_retryable_classifier(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::retries::classifiers::AwsErrorCodeClassifier;
    use aws_smithy_runtime_api::client::interceptors::context::InterceptorContext;
    use aws_smithy_runtime_api::client::interceptors::context::{Error, Input};
    use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
    use aws_smithy_runtime_api::client::retries::classifiers::{ClassifyRetry, RetryAction};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::error::metadata::ProvideErrorMetadata;
    use aws_smithy_types::error::ErrorMetadata;
    use aws_smithy_types::retry::ErrorKind;
    use std::fmt;
    use std::time::Duration;

    #[derive(Debug)]
    struct CodedError {
        metadata: ErrorMetadata,
    }

    impl CodedError {
        fn new(code: &'static str) -> Self {
            Self {
                metadata: ErrorMetadata::builder().code(code).build(),
            }
        }
    }

    impl fmt::Display for CodedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Coded Error")
        }
    }

    impl std::error::Error for CodedError {}

    impl ProvideErrorMetadata for CodedError {
        fn meta(&self) -> &ErrorMetadata {
            &self.metadata
        }
    }

    #[test]
    fn classify_by_error_code() {
        let policy = AwsErrorCodeClassifier::<CodedError>::new();
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
            CodedError::new("Throttling"),
        ))));

        assert_eq!(policy.classify_retry(&ctx), RetryAction::throttling_error());

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
            CodedError::new("RequestTimeout"),
        ))));
        assert_eq!(policy.classify_retry(&ctx), RetryAction::transient_error())
    }

    #[test]
    fn classify_generic() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let err = ErrorMetadata::builder().code("SlowDown").build();
        let test_response = http_1x::Response::new("OK").map(SdkBody::from);

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(test_response.try_into().unwrap());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));

        assert_eq!(policy.classify_retry(&ctx), RetryAction::throttling_error());
    }

    #[test]
    fn test_retry_after_header() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let err = ErrorMetadata::builder().code("SlowDown").build();
        let res = http_1x::Response::builder()
            .header("x-amz-retry-after", "5000")
            .body("retry later")
            .unwrap()
            .map(SdkBody::from);
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(res.try_into().unwrap());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));

        assert_eq!(
            policy.classify_retry(&ctx),
            RetryAction::retryable_error_with_explicit_delay(
                ErrorKind::ThrottlingError,
                Duration::from_secs(5)
            )
        );
    }

    #[test]
    fn test_invalid_retry_after_header_is_ignored() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let err = ErrorMetadata::builder().code("SlowDown").build();
        let res = http_1x::Response::builder()
            .header("x-amz-retry-after", "invalid")
            .body("retry later")
            .unwrap()
            .map(SdkBody::from);
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(res.try_into().unwrap());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));

        // Invalid header ignored — retryable_error() has retry_after: None,
        // so the retry strategy will fall back to exponential backoff.
        assert_eq!(
            policy.classify_retry(&ctx),
            RetryAction::retryable_error(ErrorKind::ThrottlingError)
        );
    }

    /// Build a context whose response optionally carries an `x-amz-retry-after`
    /// header (whose value is a count of **milliseconds**) and whose error
    /// optionally carries an error code (`None` models a bare response with no
    /// identifiable AWS error code, e.g. an unparsable 5xx).
    fn ctx_with(
        retry_after_header_millis: Option<&str>,
        error_code: Option<&'static str>,
    ) -> InterceptorContext {
        let mut builder = http_1x::Response::builder();
        if let Some(millis) = retry_after_header_millis {
            builder = builder.header("x-amz-retry-after", millis);
        }
        let res = builder.body("nope").unwrap().map(SdkBody::from);

        let err = match error_code {
            Some(code) => ErrorMetadata::builder().code(code).build(),
            None => ErrorMetadata::builder().build(),
        };

        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        ctx.set_response(res.try_into().unwrap());
        ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(err))));
        ctx
    }

    // The headline scenario: a bare 5xx with no identifiable AWS error code that
    // an earlier classifier (`HttpStatusCodeClassifier`) marked transient.
    // `classify_retry_v2` attaches the server-directed delay from
    // `x-amz-retry-after`, which that earlier classifier cannot read. The header
    // is a count of milliseconds, so 5000ms == 5s. The clamping
    // to `[t_i, 5 + t_i]` happens later in the backoff strategy, so the
    // classifier stores the raw parsed duration.
    #[test]
    fn classify_retry_v2_attaches_server_delay_on_bare_5xx() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        // No error code -> this classifier's own classification is `NoActionIndicated`.
        let ctx = ctx_with(Some("5000"), None);

        let previous = RetryAction::transient_error();
        assert_eq!(
            policy.classify_retry_v2(&ctx, &previous),
            RetryAction::retryable_error_with_explicit_delay(
                ErrorKind::TransientError,
                Duration::from_secs(5)
            ),
        );
    }

    // Same refinement occurs when there is an error code that this classifier
    // simply doesn't recognize ("InternalError" is not in its throttling/transient
    // lists), so its own classification is still `NoActionIndicated`.
    #[test]
    fn classify_retry_v2_attaches_server_delay_on_unrecognized_error_code() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let ctx = ctx_with(Some("5000"), Some("InternalError"));

        let previous = RetryAction::transient_error();
        assert_eq!(
            policy.classify_retry_v2(&ctx, &previous),
            RetryAction::retryable_error_with_explicit_delay(
                ErrorKind::TransientError,
                Duration::from_secs(5)
            ),
        );
    }

    // The `kind` of the previous (already-retryable) verdict must be preserved
    // when the server-directed delay is tucked in.
    #[test]
    fn classify_retry_v2_preserves_previous_kind() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let ctx = ctx_with(Some("2500"), None);

        let previous = RetryAction::retryable_error(ErrorKind::ServerError);
        assert_eq!(
            policy.classify_retry_v2(&ctx, &previous),
            RetryAction::retryable_error_with_explicit_delay(
                ErrorKind::ServerError,
                Duration::from_millis(2500)
            ),
        );
    }

    // When this classifier recognizes the error code itself, its own verdict
    // (which already attaches `x-amz-retry-after`) stands unchanged, regardless
    // of what an earlier classifier decided. `SlowDown` is a
    // throttling error (not transient).
    #[test]
    fn classify_retry_v2_prefers_own_error_code_verdict() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let ctx = ctx_with(Some("5000"), Some("SlowDown"));

        // Even with a `previous` transient verdict, the modeled throttling code wins.
        let previous = RetryAction::transient_error();
        assert_eq!(
            policy.classify_retry_v2(&ctx, &previous),
            RetryAction::retryable_error_with_explicit_delay(
                ErrorKind::ThrottlingError,
                Duration::from_secs(5)
            ),
        );
    }

    // The header alone must not make a response retryable: if no earlier
    // classifier marked the response retryable, `classify_retry_v2` indicates no
    // action even when `x-amz-retry-after` is present.
    #[test]
    fn classify_retry_v2_ignores_header_without_retryable_previous() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let ctx = ctx_with(Some("5000"), None);

        assert_eq!(
            policy.classify_retry_v2(&ctx, &RetryAction::NoActionIndicated),
            RetryAction::NoActionIndicated,
        );
    }

    // When the previous verdict already carries an explicit delay, the
    // refinement branch (which only matches `retry_after: None`) is skipped and
    // this classifier abstains by returning `NoActionIndicated`. Abstaining is
    // precisely how it avoids overriding the existing delay: in
    // `run_classifiers_on_ctx`, a `NoActionIndicated` result is a no-op that
    // leaves the accumulated verdict (here, the `Some(1s)` delay) untouched.
    // That end-to-end preservation is exercised in the aws-smithy-runtime
    // `run_classifiers_on_ctx` test.
    #[test]
    fn classify_retry_v2_abstains_when_previous_has_explicit_delay() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();
        let ctx = ctx_with(Some("5000"), None);

        let previous = RetryAction::retryable_error_with_explicit_delay(
            ErrorKind::TransientError,
            Duration::from_secs(1),
        );
        assert_eq!(
            policy.classify_retry_v2(&ctx, &previous),
            RetryAction::NoActionIndicated,
        );
    }

    // Invalid values in `x-amz-retry-after` must be ignored (fall
    // back to exponential backoff). With no header, or an unparsable one, an
    // already-retryable `previous` is left as-is (this classifier returns
    // `NoActionIndicated`).
    #[test]
    fn classify_retry_v2_falls_back_without_valid_header() {
        let policy = AwsErrorCodeClassifier::<ErrorMetadata>::new();

        let previous = RetryAction::transient_error();
        // Absent header.
        assert_eq!(
            policy.classify_retry_v2(&ctx_with(None, None), &previous),
            RetryAction::NoActionIndicated,
        );
        // Unparsable header.
        assert_eq!(
            policy.classify_retry_v2(&ctx_with(Some("invalid"), None), &previous),
            RetryAction::NoActionIndicated,
        );
    }
}
