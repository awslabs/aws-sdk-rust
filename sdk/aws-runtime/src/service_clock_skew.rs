/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::auth::HttpSignatureType;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeDeserializationInterceptorContextMut, BeforeTransmitInterceptorContextMut,
    BeforeTransmitInterceptorContextRef, InterceptorContext,
};
use aws_smithy_runtime_api::client::interceptors::{dyn_dispatch_hint, Intercept};
use aws_smithy_runtime_api::client::orchestrator::{HttpResponse, OrchestratorError};
use aws_smithy_runtime_api::client::retries::classifiers::{
    ClassifyRetry, RetryAction, RetryClassifierPriority,
};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::date_time::Format;
use aws_smithy_types::error::metadata::ProvideErrorMetadata;
use aws_smithy_types::retry::ErrorKind;
use aws_smithy_types::DateTime;
use std::error::Error as StdError;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// Below this absolute skew, a possible clock-skew error is treated as a genuine signature
// error rather than retried as a skew error.
const SKEW_DETECTION_THRESHOLD: Duration = Duration::from_secs(4 * 60);
// If a request's round trip exceeds this, the skew measurement is discarded as unreliable.
const MAX_TRUSTED_REQUEST_DURATION: Duration = Duration::from_secs(15 * 60);

// Error codes that indicate a possible clock-skew signing problem.
pub(crate) const CLOCK_SKEW_ERROR_CODES: &[&str] = &[
    "InvalidSignatureException",
    "SignatureDoesNotMatch",
    "AuthFailure",
    "RequestTimeTooSkewed",
    "AccessDeniedException",
];

// A signed clock skew, in milliseconds. Positive means the service clock is ahead of the
// client clock. `Default` is zero.
//
// Why i64 milliseconds: skew is directional (the client can be behind or ahead of the
// service), so the unsigned `std::time::Duration` cannot represent it. Milliseconds are ample
// precision (the HTTP `Date` header is second-resolution and SigV4 signing is second-granular),
// and i64 spans far more range than any real clock drift.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ClockSkew(i64);

impl ClockSkew {
    // The absolute magnitude of the skew.
    fn abs(self) -> Duration {
        Duration::from_millis(self.0.unsigned_abs())
    }

    // Returns `t` adjusted by this (signed) skew, saturating if the shift is unrepresentable.
    pub(crate) fn apply(self, t: SystemTime) -> SystemTime {
        if self.0 >= 0 {
            t.checked_add(Duration::from_millis(self.0 as u64))
        } else {
            t.checked_sub(Duration::from_millis(self.0.unsigned_abs()))
        }
        .unwrap_or(t)
    }
}

// Operation-scoped skew, seeded from the client skew at operation start and updated on each
// response. Read by the signer to sign at `now() + AttemptSkew`.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AttemptSkew(pub(crate) ClockSkew);
impl Storable for AttemptSkew {
    type Storer = StoreReplace<Self>;
}

// The timestamp to sign with: `now` shifted by the operation's `AttemptSkew`.
//
// Presigned requests are never shifted. A presigned URL is a function of its inputs, and
// presigning stops before transmit, so it never reads a response: it can neither contribute to the
// skew nor be corrected by one.
pub(crate) fn signing_time(
    now: SystemTime,
    signature_type: HttpSignatureType,
    cfg: &ConfigBag,
) -> SystemTime {
    if signature_type == HttpSignatureType::HttpRequestQueryParams {
        return now;
    }
    cfg.load::<AttemptSkew>()
        .map_or(now, |skew| skew.0.apply(now))
}

// The client's raw local time when the request was sent (no skew applied), used to compute
// the midpoint for the candidate skew.
#[derive(Clone, Copy, Debug)]
struct TimeRequestSent(SystemTime);
impl Storable for TimeRequestSent {
    type Storer = StoreReplace<Self>;
}

// The surviving candidate skew for a response, attached as a response extension so the retry
// classifier can read it (a classifier has no config bag access).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ResponseClockSkew(pub(crate) ClockSkew);

/// When present and `true`, clock skew correction is disabled.
#[derive(Clone, Copy, Debug)]
pub struct DisableClockSkewCorrection(bool);
impl DisableClockSkewCorrection {
    /// Returns whether clock skew correction is disabled.
    pub fn is_disabled(&self) -> bool {
        self.0
    }
}
impl From<bool> for DisableClockSkewCorrection {
    fn from(disable: bool) -> Self {
        Self(disable)
    }
}
impl Storable for DisableClockSkewCorrection {
    type Storer = StoreReplace<Self>;
}

fn disabled(cfg: &ConfigBag) -> bool {
    cfg.load::<DisableClockSkewCorrection>()
        .map(|d| d.0)
        .unwrap_or(false)
}

fn server_time(response: &HttpResponse) -> Option<SystemTime> {
    let date = response.headers().get("date")?;
    let date_time = DateTime::from_str(date, Format::HttpDate).ok()?;
    SystemTime::try_from(date_time).ok()
}

// `serverTime - midpoint`, as a signed skew. The midpoint assumes the server generated its
// timestamp halfway through the round trip (the NTP clock offset calculation, RFC 5905 §8).
fn signed_skew(server: SystemTime, midpoint: SystemTime) -> ClockSkew {
    match server.duration_since(midpoint) {
        Ok(d) => ClockSkew(d.as_millis() as i64),
        Err(e) => ClockSkew(-(e.duration().as_millis() as i64)),
    }
}

/// Interceptor that tracks the clock skew between the client and service, corrects the
/// signing timestamp on retries, and records the skew for retry classification.
///
/// Holds the client-level clock skew, which persists across operations on this client.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct ServiceClockSkewInterceptor {
    client_skew: Arc<Mutex<ClockSkew>>,
}

impl ServiceClockSkewInterceptor {
    /// Creates a new `ServiceClockSkewInterceptor`.
    pub fn new() -> Self {
        Self::default()
    }

    // Seeds the client-level skew (for conformance tests that start from a non-zero skew).
    #[cfg(test)]
    fn with_client_skew(client_skew: ClockSkew) -> Self {
        Self {
            client_skew: Arc::new(Mutex::new(client_skew)),
        }
    }

    // Reads the current client-level skew (to assert `expectedClientSkew`).
    #[cfg(test)]
    fn client_skew(&self) -> ClockSkew {
        *self.client_skew.lock().unwrap()
    }
}

#[dyn_dispatch_hint]
impl Intercept for ServiceClockSkewInterceptor {
    fn name(&self) -> &'static str {
        "ServiceClockSkewInterceptor"
    }

    fn modify_before_retry_loop(
        &self,
        _ctx: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if disabled(cfg) {
            return Ok(());
        }
        // Seed the attempt skew from the persisted client skew so the first attempt of this
        // operation signs with any offset learned by previous operations.
        let seed = *self.client_skew.lock().unwrap();
        cfg.interceptor_state().store_put(AttemptSkew(seed));
        Ok(())
    }

    fn read_before_transmit(
        &self,
        _ctx: &BeforeTransmitInterceptorContextRef<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if disabled(cfg) {
            return Ok(());
        }
        let now = runtime_components
            .time_source()
            .ok_or("a time source is required (clock skew)")?
            .now();
        cfg.interceptor_state().store_put(TimeRequestSent(now));
        Ok(())
    }

    fn modify_before_deserialization(
        &self,
        ctx: &mut BeforeDeserializationInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if disabled(cfg) {
            return Ok(());
        }
        let time_received = runtime_components
            .time_source()
            .ok_or("a time source is required (clock skew)")?
            .now();
        let Some(time_sent) = cfg.load::<TimeRequestSent>().map(|t| t.0) else {
            tracing::debug!("no recorded request send time; skipping clock skew measurement");
            return Ok(());
        };
        // Cached response (RFC 7234 §5.1): the `Date` is stale, so don't trust it.
        if ctx.response().headers().get("age").is_some() {
            tracing::debug!("response came from a cache; skipping clock skew measurement");
            return Ok(());
        }
        let Some(server) = server_time(ctx.response()) else {
            tracing::debug!("no usable `Date` response header; skipping clock skew measurement");
            return Ok(());
        };
        let elapsed = time_received.duration_since(time_sent).unwrap_or_default();
        if elapsed > MAX_TRUSTED_REQUEST_DURATION {
            tracing::debug!(
                ?elapsed,
                "request too slow to measure clock skew reliably; skipping"
            );
            return Ok(());
        }
        let midpoint = time_sent + elapsed / 2;
        let candidate = signed_skew(server, midpoint);
        // Record unconditionally: every response refreshes the skew, so a stale value is
        // corrected without any special-case healing logic.
        cfg.interceptor_state().store_put(AttemptSkew(candidate));
        *self.client_skew.lock().unwrap() = candidate;
        tracing::trace!(skew_ms = candidate.0, "recorded clock skew");
        // Hand the surviving candidate to the retry classifier via the response.
        ctx.response_mut()
            .add_extension(ResponseClockSkew(candidate));
        Ok(())
    }
}

/// Retry classifier for clock-skew errors.
///
/// A response is retryable when its error code is a known clock-skew code and the skew
/// measured from the response exceeds the detection threshold.
#[derive(Debug)]
pub struct ServiceClockSkewClassifier<E> {
    _inner: PhantomData<E>,
}

impl<E> ServiceClockSkewClassifier<E> {
    /// Creates a new `ServiceClockSkewClassifier`.
    pub fn new() -> Self {
        Self {
            _inner: PhantomData,
        }
    }
}

impl<E> Default for ServiceClockSkewClassifier<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> ClassifyRetry for ServiceClockSkewClassifier<E>
where
    E: StdError + ProvideErrorMetadata + Send + Sync + 'static,
{
    // Only `classify_retry` is implemented; the default `classify_retry_v2` delegates to it. This
    // classifier makes a self-contained determination from the response alone (the attached skew
    // and the error code), so it has no use for the verdict accumulated by earlier classifiers
    // that `classify_retry_v2` exposes.
    fn classify_retry(&self, ctx: &InterceptorContext) -> RetryAction {
        // Only a surviving candidate (one that passed the Age/duration/Date checks) is
        // attached; its absence means no skew could be computed for this response.
        let Some(skew) = ctx
            .response()
            .and_then(|r| r.extension::<ResponseClockSkew>())
        else {
            return RetryAction::NoActionIndicated;
        };
        if skew.0.abs() <= SKEW_DETECTION_THRESHOLD {
            return RetryAction::NoActionIndicated;
        }
        let error_code = match ctx.output_or_error() {
            Some(Err(err)) => OrchestratorError::as_operation_error(err)
                .and_then(|err| err.downcast_ref::<E>())
                .and_then(|err| err.code()),
            _ => return RetryAction::NoActionIndicated,
        };
        match error_code {
            // Non-throttling, so it draws the standard retry cost and counts toward max attempts.
            Some(code) if CLOCK_SKEW_ERROR_CODES.contains(&code) => {
                RetryAction::retryable_error(ErrorKind::ServerError)
            }
            _ => RetryAction::NoActionIndicated,
        }
    }

    fn name(&self) -> &'static str {
        "ServiceClockSkew"
    }

    fn priority(&self) -> RetryClassifierPriority {
        RetryClassifierPriority::run_after(RetryClassifierPriority::http_status_code_classifier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_async::test_util::ManualTimeSource;
    use aws_smithy_runtime_api::client::interceptors::context::{Error, Input, Output};
    use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::error::ErrorMetadata;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::fmt;

    #[derive(Debug)]
    struct CodedError {
        metadata: ErrorMetadata,
    }

    impl CodedError {
        fn new(code: &str) -> Self {
            Self {
                metadata: ErrorMetadata::builder().code(code).build(),
            }
        }
    }

    impl fmt::Display for CodedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "coded error")
        }
    }

    impl std::error::Error for CodedError {}

    impl ProvideErrorMetadata for CodedError {
        fn meta(&self) -> &ErrorMetadata {
            &self.metadata
        }
    }

    #[test]
    fn apply_adjusts_by_sign() {
        let base = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        assert_eq!(ClockSkew(0).apply(base), base);
        assert_eq!(ClockSkew(5000).apply(base), base + Duration::from_secs(5));
        assert_eq!(ClockSkew(-5000).apply(base), base - Duration::from_secs(5));
    }

    #[test]
    fn signed_skew_tracks_direction() {
        let midpoint = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        // Server ahead of the client.
        assert_eq!(
            signed_skew(midpoint + Duration::from_secs(10), midpoint),
            ClockSkew(10_000)
        );
        // Client ahead of the server.
        assert_eq!(
            signed_skew(midpoint - Duration::from_secs(10), midpoint),
            ClockSkew(-10_000)
        );
    }

    #[test]
    fn presigning_is_not_shifted_by_skew() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1000);
        let mut cfg = ConfigBag::base();

        assert_eq!(
            signing_time(now, HttpSignatureType::HttpRequestHeaders, &cfg),
            now,
            "nothing to apply before a skew is recorded",
        );

        cfg.interceptor_state().store_put(AttemptSkew(TEN_MIN));
        assert_eq!(
            signing_time(now, HttpSignatureType::HttpRequestHeaders, &cfg),
            now + Duration::from_secs(10 * 60),
        );
        assert_eq!(
            signing_time(now, HttpSignatureType::HttpRequestQueryParams, &cfg),
            now,
            "a presigned URL must not move with the client's skew",
        );
    }

    // Build a failed-response context with an optional attached skew and error code.
    fn ctx(code: Option<&'static str>, skew: Option<ClockSkew>) -> InterceptorContext {
        let mut ctx = InterceptorContext::new(Input::doesnt_matter());
        let http = http_1x::Response::builder()
            .status(403)
            .body(SdkBody::empty())
            .unwrap();
        let mut resp: HttpResponse = http.try_into().unwrap();
        if let Some(s) = skew {
            resp.add_extension(ResponseClockSkew(s));
        }
        ctx.set_response(resp);
        match code {
            Some(c) => ctx.set_output_or_error(Err(OrchestratorError::operation(Error::erase(
                CodedError::new(c),
            )))),
            None => ctx.set_output_or_error(Ok(Output::erase("ok"))),
        }
        ctx
    }

    const TEN_MIN: ClockSkew = ClockSkew(10 * 60 * 1000);
    const ONE_MIN: ClockSkew = ClockSkew(60 * 1000);

    #[test]
    fn no_attached_skew_is_not_retried() {
        let classifier = ServiceClockSkewClassifier::<CodedError>::new();
        assert_eq!(
            classifier.classify_retry(&ctx(Some("InvalidSignatureException"), None)),
            RetryAction::NoActionIndicated
        );
    }

    #[test]
    fn skew_below_threshold_is_not_retried() {
        let classifier = ServiceClockSkewClassifier::<CodedError>::new();
        assert_eq!(
            classifier.classify_retry(&ctx(Some("InvalidSignatureException"), Some(ONE_MIN))),
            RetryAction::NoActionIndicated
        );
    }

    #[test]
    fn skew_above_threshold_with_known_code_is_retried() {
        let classifier = ServiceClockSkewClassifier::<CodedError>::new();
        assert_eq!(
            classifier.classify_retry(&ctx(Some("RequestTimeTooSkewed"), Some(TEN_MIN))),
            RetryAction::retryable_error(ErrorKind::ServerError)
        );
    }

    #[test]
    fn skew_above_threshold_with_unknown_code_is_not_retried() {
        let classifier = ServiceClockSkewClassifier::<CodedError>::new();
        assert_eq!(
            classifier.classify_retry(&ctx(Some("SomeOtherError"), Some(TEN_MIN))),
            RetryAction::NoActionIndicated
        );
    }

    #[test]
    fn success_is_not_retried() {
        let classifier = ServiceClockSkewClassifier::<CodedError>::new();
        assert_eq!(
            classifier.classify_retry(&ctx(None, Some(TEN_MIN))),
            RetryAction::NoActionIndicated
        );
    }

    // ---- Data-driven conformance suite ----
    //
    // Drives the real interceptor (skew measurement, discards, recording), the real classifier
    // (code set + detection threshold), and the signing offset (`AttemptSkew::apply`, exactly what
    // the signer runs) against the shared clock-skew test cases. The attempt/retry sequencing is
    // modeled here (the JSON lists the attempts that occur); the orchestrator wiring itself is
    // covered by codegen tests. Time is injected via a settable `ManualTimeSource`.

    #[derive(Deserialize)]
    struct Suite {
        tests: Vec<TestCase>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestCase {
        description: String,
        operations: Vec<Operation>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Operation {
        initial_client_skew: i64,
        #[serde(default)]
        max_attempts: Option<u32>,
        attempts: Vec<Attempt>,
        expected_client_skew: i64,
        expected_outcome: String,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Attempt {
        client_time_at_send: String,
        client_time_at_receive: String,
        expected_signing_time: String,
        response: ResponseSpec,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ResponseSpec {
        status_code: u16,
        #[serde(default)]
        headers: HashMap<String, String>,
        #[serde(default)]
        error_code: Option<String>,
    }

    const SUITE: &str = include_str!("../test-data/clock-skew-test-cases.json");

    fn parse_time(s: &str) -> SystemTime {
        SystemTime::try_from(DateTime::from_str(s, Format::DateTime).expect("valid timestamp"))
            .expect("representable time")
    }

    // The suite expresses skews in whole seconds.
    fn skew_secs(secs: i64) -> ClockSkew {
        ClockSkew(secs * 1000)
    }

    fn build_response(spec: &ResponseSpec) -> HttpResponse {
        let mut builder = http_1x::Response::builder().status(spec.status_code);
        for (name, value) in &spec.headers {
            builder = builder.header(name.as_str(), value.as_str());
        }
        builder.body(SdkBody::empty()).unwrap().try_into().unwrap()
    }

    fn response_output_or_error(spec: &ResponseSpec) -> Result<Output, OrchestratorError<Error>> {
        match &spec.error_code {
            Some(code) => Err(OrchestratorError::operation(Error::erase(CodedError::new(
                code,
            )))),
            None => Ok(Output::erase("ok")),
        }
    }

    // A context advanced to the `BeforeTransmit` phase with a placeholder request.
    fn before_transmit_context() -> InterceptorContext {
        let mut context = InterceptorContext::new(Input::doesnt_matter());
        context.enter_serialization_phase();
        context.set_request(HttpRequest::empty());
        let _ = context.take_input();
        context.enter_before_transmit_phase();
        context
    }

    fn run_operation(interceptor: &ServiceClockSkewInterceptor, op: &Operation, desc: &str) {
        let time = ManualTimeSource::new(SystemTime::UNIX_EPOCH);
        let rc = RuntimeComponentsBuilder::for_tests()
            .with_time_source(Some(time.clone()))
            .build()
            .unwrap();
        let mut cfg = ConfigBag::base();

        // Once per operation: seed AttemptSkew from the persisted ClientSkew.
        {
            let mut seed = before_transmit_context();
            let mut seed_ctx = (&mut seed).into();
            interceptor
                .modify_before_retry_loop(&mut seed_ctx, &rc, &mut cfg)
                .unwrap();
        }

        for (j, attempt) in op.attempts.iter().enumerate() {
            let send = parse_time(&attempt.client_time_at_send);
            let receive = parse_time(&attempt.client_time_at_receive);

            // The signer signs at now() + AttemptSkew.
            time.set_time(send);
            let attempt_skew = cfg.load::<AttemptSkew>().map(|s| s.0).unwrap_or_default();
            assert_eq!(
                attempt_skew.apply(send),
                parse_time(&attempt.expected_signing_time),
                "{desc}: attempt {j} signing time",
            );

            let mut context = before_transmit_context();
            {
                let ref_ctx = (&context).into();
                interceptor
                    .read_before_transmit(&ref_ctx, &rc, &mut cfg)
                    .unwrap();
            }

            context.enter_transmit_phase();
            let _ = context.take_request();
            context.set_response(build_response(&attempt.response));
            context.enter_before_deserialization_phase();
            time.set_time(receive);
            {
                let mut mut_ctx = (&mut context).into();
                interceptor
                    .modify_before_deserialization(&mut mut_ctx, &rc, &mut cfg)
                    .unwrap();
            }

            context.enter_deserialization_phase();
            context.set_output_or_error(response_output_or_error(&attempt.response));
            let action = ServiceClockSkewClassifier::<CodedError>::new().classify_retry(&context);

            let is_last = j + 1 == op.attempts.len();
            if !is_last {
                // A retry followed, so the classifier must have indicated one.
                assert!(action.should_retry(), "{desc}: attempt {j} should retry");
            } else if op.expected_outcome == "error"
                && op
                    .max_attempts
                    .is_none_or(|m| (op.attempts.len() as u32) < m)
            {
                // No further attempt despite remaining budget => the classifier declined to retry.
                assert!(
                    !action.should_retry(),
                    "{desc}: final attempt should not be retried as clock skew",
                );
            }
        }

        let final_status = op
            .attempts
            .last()
            .expect("at least one attempt")
            .response
            .status_code;
        let outcome = if (200..300).contains(&final_status) {
            "success"
        } else {
            "error"
        };
        assert_eq!(outcome, op.expected_outcome, "{desc}: outcome");
    }

    #[test]
    fn clock_skew_conformance() {
        let suite: Suite = serde_json::from_str(SUITE).expect("valid clock skew suite json");
        assert_eq!(suite.tests.len(), 10, "expected the full clock skew suite");
        for case in &suite.tests {
            // ClientSkew starts at the first operation's initial value and persists across the
            // operations of a case (a single client).
            let interceptor = ServiceClockSkewInterceptor::with_client_skew(skew_secs(
                case.operations[0].initial_client_skew,
            ));
            for (i, op) in case.operations.iter().enumerate() {
                assert_eq!(
                    interceptor.client_skew(),
                    skew_secs(op.initial_client_skew),
                    "{}: operation {i} initial ClientSkew",
                    case.description,
                );
                run_operation(&interceptor, op, &case.description);
                assert_eq!(
                    interceptor.client_skew(),
                    skew_secs(op.expected_client_skew),
                    "{}: operation {i} expected ClientSkew",
                    case.description,
                );
            }
        }
    }
}
