/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
//! Test support: the recording interceptor, and the connectors and plugins the scenarios use.

use aws_smithy_runtime::client::auth::no_auth::NO_AUTH_SCHEME_ID;
use aws_smithy_runtime::client::orchestrator::endpoints::StaticUriEndpointResolver;
use aws_smithy_runtime::client::retries::strategy::NeverRetryStrategy;
use aws_smithy_runtime::client::test_util::{
    deserializer::CannedResponseDeserializer, serializer::CannedRequestSerializer,
};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::auth::static_resolver::StaticAuthSchemeOptionResolver;
use aws_smithy_runtime_api::client::auth::{
    AuthSchemeOptionResolverParams, SharedAuthSchemeOptionResolver,
};
use aws_smithy_runtime_api::client::endpoint::{EndpointResolverParams, SharedEndpointResolver};
use aws_smithy_runtime_api::client::http::{http_client_fn, HttpConnector, HttpConnectorFuture};
use aws_smithy_runtime_api::client::interceptors::context::{
    AfterDeserializationInterceptorContextRef, BeforeDeserializationInterceptorContextMut,
    BeforeDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextMut,
    BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextMut,
    BeforeTransmitInterceptorContextRef, FinalizerInterceptorContextMut,
    FinalizerInterceptorContextRef, InterceptorContext, Output,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, OrchestratorError};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::retries::{
    RequestAttempts, RetryStrategy, SharedRetryStrategy, ShouldAttempt,
};
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugin;
use aws_smithy_runtime_api::client::ser_de::{SharedRequestSerializer, SharedResponseDeserializer};
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
use aws_smithy_types::timeout::TimeoutConfig;
use http_1x::{Response, StatusCode};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// What a finalizer hook saw in `output_or_error` when it ran.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Observed {
    /// The slot was empty. The hook cannot tell whether anything went wrong, and the mutating
    /// finalizers have nothing to replace. This is the failure mode that a count-only matrix
    /// would miss.
    Nothing,
    Success,
    Timeout,
    ConnectorFailure,
    OtherError,
}

impl Observed {
    pub(super) fn of<O, E>(outcome: Option<Result<&O, &OrchestratorError<E>>>) -> Self {
        match outcome {
            None => Observed::Nothing,
            Some(Ok(_)) => Observed::Success,
            Some(Err(err)) if err.is_timeout_error() => Observed::Timeout,
            Some(Err(err)) if err.is_connector_error() => Observed::ConnectorFailure,
            Some(Err(_)) => Observed::OtherError,
        }
    }
}

// ---------------------------------------------------------------------------
// Recording
// ---------------------------------------------------------------------------

/// Shared recording state for one run.
#[derive(Clone, Debug, Default)]
pub(super) struct HookLog {
    pub(super) counts: Arc<Mutex<BTreeMap<&'static str, usize>>>,
    /// Invocation sequence, for the ordering assertion.
    pub(super) sequence: Arc<Mutex<Vec<&'static str>>>,
    /// What each finalizer hook observed, in invocation order.
    pub(super) observed: Arc<Mutex<Vec<(&'static str, Observed)>>>,
}

impl HookLog {
    pub(super) fn record(&self, hook: &'static str) {
        *self.counts.lock().unwrap().entry(hook).or_insert(0) += 1;
        self.sequence.lock().unwrap().push(hook);
    }

    pub(super) fn record_finalizer(&self, hook: &'static str, observed: Observed) {
        self.record(hook);
        self.observed.lock().unwrap().push((hook, observed));
    }

    pub(super) fn count(&self, hook: &str) -> usize {
        self.counts.lock().unwrap().get(hook).copied().unwrap_or(0)
    }
}

/// Generates a non-finalizer hook body that just records the call.
macro_rules! recording_hook {
    ($name:ident, $ctx:ident) => {
        fn $name(
            &self,
            _ctx: &$ctx<'_>,
            _components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            self.0.record(stringify!($name));
            Ok(())
        }
    };
    (mut $name:ident, $ctx:ident) => {
        fn $name(
            &self,
            _ctx: &mut $ctx<'_>,
            _components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            self.0.record(stringify!($name));
            Ok(())
        }
    };
    // Finalizers additionally record what they observed in `output_or_error`.
    (finalizer $name:ident, $ctx:ident) => {
        fn $name(
            &self,
            ctx: &$ctx<'_>,
            _components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            self.0
                .record_finalizer(stringify!($name), Observed::of(ctx.output_or_error()));
            Ok(())
        }
    };
    (finalizer mut $name:ident, $ctx:ident) => {
        fn $name(
            &self,
            ctx: &mut $ctx<'_>,
            _components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            self.0
                .record_finalizer(stringify!($name), Observed::of(ctx.output_or_error()));
            Ok(())
        }
    };
}

/// Recorder exercising the unconditional dispatch path.
///
/// `overridden_hooks()` is left to its default of `all()`, so every hook is dispatched without
/// consulting a bitmask.
#[derive(Clone, Debug, Default)]
pub(super) struct PlainRecorder(pub(super) HookLog);

impl Intercept for PlainRecorder {
    fn name(&self) -> &'static str {
        "PlainRecorder"
    }

    // `read_before_execution` is the one hook that takes no `RuntimeComponents`.
    fn read_before_execution(
        &self,
        _ctx: &BeforeSerializationInterceptorContextRef<'_>,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        self.0.record("read_before_execution");
        Ok(())
    }

    recording_hook!(
        mut modify_before_serialization,
        BeforeSerializationInterceptorContextMut
    );
    recording_hook!(
        read_before_serialization,
        BeforeSerializationInterceptorContextRef
    );
    recording_hook!(
        read_after_serialization,
        BeforeTransmitInterceptorContextRef
    );
    recording_hook!(
        mut modify_before_retry_loop,
        BeforeTransmitInterceptorContextMut
    );
    recording_hook!(read_before_attempt, BeforeTransmitInterceptorContextRef);
    recording_hook!(
        mut modify_before_signing,
        BeforeTransmitInterceptorContextMut
    );
    recording_hook!(read_before_signing, BeforeTransmitInterceptorContextRef);
    recording_hook!(read_after_signing, BeforeTransmitInterceptorContextRef);
    recording_hook!(
        mut modify_before_transmit,
        BeforeTransmitInterceptorContextMut
    );
    recording_hook!(read_before_transmit, BeforeTransmitInterceptorContextRef);
    recording_hook!(
        read_after_transmit,
        BeforeDeserializationInterceptorContextRef
    );
    recording_hook!(
        mut modify_before_deserialization,
        BeforeDeserializationInterceptorContextMut
    );
    recording_hook!(
        read_before_deserialization,
        BeforeDeserializationInterceptorContextRef
    );
    recording_hook!(
        read_after_deserialization,
        AfterDeserializationInterceptorContextRef
    );

    recording_hook!(finalizer mut modify_before_attempt_completion, FinalizerInterceptorContextMut);
    recording_hook!(finalizer read_after_attempt, FinalizerInterceptorContextRef);
    recording_hook!(finalizer mut modify_before_completion, FinalizerInterceptorContextMut);
    recording_hook!(finalizer read_after_execution, FinalizerInterceptorContextRef);
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Responds 200, except for the first `failures` calls which fail at the transport layer.
///
/// `failures: 0` always succeeds, `usize::MAX` always fails, and `1` makes `PerAttempt` and
/// `PerResponse` diverge on a retry.
#[derive(Clone, Debug)]
pub(super) struct ScriptedConnector {
    calls: Arc<AtomicUsize>,
    failures: usize,
}

impl ScriptedConnector {
    pub(super) fn always_ok() -> Self {
        Self::failing_first(0)
    }

    pub(super) fn always_failing() -> Self {
        Self::failing_first(usize::MAX)
    }

    /// Fails the first `n` calls, then succeeds -- `n = 1` makes `PerAttempt` and `PerResponse`
    /// diverge on a retry.
    pub(super) fn failing_first(n: usize) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            failures: n,
        }
    }
}

impl HttpConnector for ScriptedConnector {
    fn call(&self, _request: HttpRequest) -> HttpConnectorFuture {
        if self.calls.fetch_add(1, Ordering::SeqCst) < self.failures {
            return HttpConnectorFuture::ready(Err(ConnectorError::io("connector failed".into())));
        }
        HttpConnectorFuture::ready(Ok(http_1x::Response::builder()
            .status(200)
            .body(SdkBody::empty())
            .expect("valid response")
            .try_into()
            .unwrap()))
    }
}

/// Retries up to `max_attempts`, optionally asking for a delay so that an operation timeout can
/// land during backoff rather than mid-attempt.
///
/// `initial_delay` additionally stalls the *initial* request. The orchestrator awaits that delay
/// after the prologue but before `read_before_attempt`, which is how a timeout can be made to land
/// with no attempt ever started.
#[derive(Debug)]
pub(super) struct BoundedRetryStrategy {
    pub(super) max_attempts: u32,
    pub(super) delay: Option<Duration>,
    pub(super) initial_delay: Option<Duration>,
}

impl RetryStrategy for BoundedRetryStrategy {
    fn should_attempt_initial_request(
        &self,
        _components: &RuntimeComponents,
        _cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        Ok(match self.initial_delay {
            Some(delay) => ShouldAttempt::YesAfterDelay(delay),
            None => ShouldAttempt::Yes,
        })
    }

    fn should_attempt_retry(
        &self,
        _ctx: &InterceptorContext,
        _components: &RuntimeComponents,
        cfg: &ConfigBag,
    ) -> Result<ShouldAttempt, BoxError> {
        let attempts = cfg
            .load::<RequestAttempts>()
            .map(|a| a.attempts())
            .unwrap_or(1);
        if attempts >= self.max_attempts {
            return Ok(ShouldAttempt::No);
        }
        Ok(match self.delay {
            Some(delay) => ShouldAttempt::YesAfterDelay(delay),
            None => ShouldAttempt::Yes,
        })
    }
}

/// Supplies the serializer, deserializer, endpoint and auth that every run needs.
#[derive(Debug)]
pub(super) struct BaseRuntimePlugin(pub(super) RuntimeComponentsBuilder);

impl BaseRuntimePlugin {
    pub(super) fn new() -> Self {
        Self(
            RuntimeComponentsBuilder::for_tests()
                // Scenarios override these via `ScenarioPlugin` where they need something else.
                .with_retry_strategy(Some(SharedRetryStrategy::new(NeverRetryStrategy::new())))
                .with_http_client(Some(http_client_fn(|_, _| {
                    ScriptedConnector::always_ok().into_shared()
                })))
                .with_endpoint_resolver(Some(SharedEndpointResolver::new(
                    StaticUriEndpointResolver::http_localhost(8080),
                )))
                .with_auth_scheme_option_resolver(Some(SharedAuthSchemeOptionResolver::new(
                    StaticAuthSchemeOptionResolver::new(vec![NO_AUTH_SCHEME_ID]),
                ))),
        )
    }
}

impl RuntimePlugin for BaseRuntimePlugin {
    fn config(&self) -> Option<FrozenLayer> {
        let mut layer = Layer::new("BaseRuntimePlugin");
        layer.store_put(AuthSchemeOptionResolverParams::new("dontcare"));
        layer.store_put(EndpointResolverParams::new("dontcare"));
        layer.store_put(SharedRequestSerializer::new(
            CannedRequestSerializer::success(HttpRequest::empty()),
        ));
        layer.store_put(SharedResponseDeserializer::new(
            CannedResponseDeserializer::new(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(SdkBody::empty())
                    .map_err(|err| OrchestratorError::other(Box::new(err)))
                    .map(Output::erase),
            ),
        ));
        Some(layer.freeze())
    }

    fn runtime_components(
        &self,
        _: &RuntimeComponentsBuilder,
    ) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&self.0)
    }
}

/// Carries the per-scenario components and timeout config.
#[derive(Debug)]
pub(super) struct ScenarioPlugin {
    pub(super) components: RuntimeComponentsBuilder,
    pub(super) timeout: Option<TimeoutConfig>,
}

impl RuntimePlugin for ScenarioPlugin {
    fn config(&self) -> Option<FrozenLayer> {
        let timeout = self.timeout.clone()?;
        let mut layer = Layer::new("ScenarioPlugin");
        layer.store_put(timeout);
        Some(layer.freeze())
    }

    fn runtime_components(
        &self,
        _: &RuntimeComponentsBuilder,
    ) -> Cow<'_, RuntimeComponentsBuilder> {
        Cow::Borrowed(&self.components)
    }
}
