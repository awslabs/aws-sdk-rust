/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */
//! The scenario table, the runner, and the assertions.

use super::support::{
    BaseRuntimePlugin, BoundedRetryStrategy, HookLog, Observed, PlainRecorder, ScenarioPlugin,
    ScriptedConnector,
};
use aws_smithy_http_client::test_util::NeverClient;
use aws_smithy_runtime::client::auth::no_auth::NoAuthRuntimePluginV2;
use aws_smithy_runtime::client::defaults::{default_sleep_impl_plugin, default_time_source_plugin};
use aws_smithy_runtime::client::orchestrator::{invoke_with_stop_point, StopPoint};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::http::http_client_fn;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeTransmitInterceptorContextRef, Input,
};
use aws_smithy_runtime_api::client::interceptors::{
    dyn_dispatch_hint, known_hooks, Intercept, SharedInterceptor,
};
use aws_smithy_runtime_api::client::retries::SharedRetryStrategy;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins;
use aws_smithy_runtime_api::shared::IntoShared;
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::timeout::TimeoutConfig;
use std::time::Duration;

/// The hooks this matrix covers, in lifecycle order.
///
/// Asserted against `known_hooks!()` below, so adding a hook to the runtime forces a matrix update
/// rather than silently leaving the new hook uncovered. Spelled out rather than taken from
/// `known_hooks!()` directly so that a *reordering* is caught too, not only an addition.
const HOOKS: &[&str] = &[
    "read_before_execution",
    "modify_before_serialization",
    "read_before_serialization",
    "read_after_serialization",
    "modify_before_retry_loop",
    "read_before_attempt",
    "modify_before_signing",
    "read_before_signing",
    "read_after_signing",
    "modify_before_transmit",
    "read_before_transmit",
    "read_after_transmit",
    "modify_before_deserialization",
    "read_before_deserialization",
    "read_after_deserialization",
    "modify_before_attempt_completion",
    "read_after_attempt",
    "modify_before_completion",
    "read_after_execution",
];

/// Expected invocation count for a hook in a given scenario.
#[derive(Copy, Clone, Debug, PartialEq)]
enum Expect {
    /// Never invoked.
    Never,
    /// Exactly once per execution.
    Once,
    /// Once per attempt actually started, measured by the `read_before_attempt` count.
    ///
    /// A literal would not do: a consumer detects a missing invocation with an equality against the
    /// attempt count, so a fix with the wrong cardinality would silently corrupt its metrics.
    PerAttempt,
    /// Once per attempt that received a response, else zero.
    PerResponse,
}
use Expect::*;

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Connector {
    Ok,
    Fail,
    FailThenOk,
    /// Never responds, so a timeout is the only way the attempt ends.
    Never,
}

struct Scenario {
    name: &'static str,
    connector: Connector,
    timeout: Option<TimeoutConfig>,
    retry: Option<BoundedRetryStrategy>,
    /// Abandon the future instead of letting the operation finish, to pin caller-initiated
    /// cancellation behaviour.
    caller_abandons: bool,
    /// Attempts expected to start, i.e. the `read_before_attempt` count.
    attempts: usize,
    /// Attempts expected to receive a response.
    responses: usize,
    /// Expected count for each hook in [`HOOKS`] order.
    expected: [Expect; 19],
    /// Expected `Observed` for every finalizer invocation, in invocation order.
    expected_observations: Vec<(&'static str, Observed)>,
}

/// Builds an expectation row from the per-stage groups, so the scenarios read as a table.
fn row(
    prologue: Expect,
    attempt: Expect,
    deserialization: Expect,
    attempt_finalizers: Expect,
    execution_finalizers: Expect,
) -> [Expect; 19] {
    [
        // prologue: read_before_execution .. modify_before_retry_loop
        prologue,
        prologue,
        prologue,
        prologue,
        prologue,
        // attempt: read_before_attempt .. read_before_transmit
        attempt,
        attempt,
        attempt,
        attempt,
        attempt,
        attempt,
        // deserialization: read_after_transmit .. read_after_deserialization
        deserialization,
        deserialization,
        deserialization,
        deserialization,
        // finalizers
        attempt_finalizers,
        attempt_finalizers,
        execution_finalizers,
        execution_finalizers,
    ]
}

fn attempt_timeout(millis: u64) -> Option<TimeoutConfig> {
    Some(
        TimeoutConfig::builder()
            .operation_attempt_timeout(Duration::from_millis(millis))
            .build(),
    )
}

fn operation_timeout(millis: u64) -> Option<TimeoutConfig> {
    Some(
        TimeoutConfig::builder()
            .operation_timeout(Duration::from_millis(millis))
            .build(),
    )
}

/// Every finalizer in one attempt plus the execution, all observing the same thing.
fn observations(attempt: Observed, execution: Observed) -> Vec<(&'static str, Observed)> {
    vec![
        ("modify_before_attempt_completion", attempt),
        ("read_after_attempt", attempt),
        ("modify_before_completion", execution),
        ("read_after_execution", execution),
    ]
}

fn scenarios() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "success",
            connector: Connector::Ok,
            timeout: None,
            retry: None,
            caller_abandons: false,
            attempts: 1,
            responses: 1,
            expected: row(Once, PerAttempt, PerResponse, PerAttempt, Once),
            expected_observations: observations(Observed::Success, Observed::Success),
        },
        Scenario {
            name: "retry then success",
            connector: Connector::FailThenOk,
            timeout: None,
            retry: Some(BoundedRetryStrategy {
                max_attempts: 2,
                delay: None,
                initial_delay: None,
            }),
            caller_abandons: false,
            attempts: 2,
            responses: 1,
            expected: row(Once, PerAttempt, PerResponse, PerAttempt, Once),
            // The first attempt genuinely failed; the second succeeded.
            expected_observations: vec![
                (
                    "modify_before_attempt_completion",
                    Observed::ConnectorFailure,
                ),
                ("read_after_attempt", Observed::ConnectorFailure),
                ("modify_before_attempt_completion", Observed::Success),
                ("read_after_attempt", Observed::Success),
                ("modify_before_completion", Observed::Success),
                ("read_after_execution", Observed::Success),
            ],
        },
        Scenario {
            name: "transport error",
            connector: Connector::Fail,
            timeout: None,
            retry: None,
            caller_abandons: false,
            attempts: 1,
            responses: 0,
            expected: row(Once, PerAttempt, Never, PerAttempt, Once),
            expected_observations: observations(
                Observed::ConnectorFailure,
                Observed::ConnectorFailure,
            ),
        },
        Scenario {
            name: "attempt timeout",
            connector: Connector::Never,
            timeout: attempt_timeout(200),
            retry: None,
            caller_abandons: false,
            attempts: 1,
            responses: 0,
            expected: row(Once, PerAttempt, Never, PerAttempt, Once),
            expected_observations: observations(Observed::Timeout, Observed::Timeout),
        },
        Scenario {
            name: "operation timeout mid-attempt",
            connector: Connector::Never,
            timeout: operation_timeout(200),
            retry: None,
            caller_abandons: false,
            attempts: 1,
            responses: 0,
            expected: row(Once, PerAttempt, Never, PerAttempt, Once),
            expected_observations: observations(Observed::Timeout, Observed::Timeout),
        },
        Scenario {
            name: "operation timeout during retry backoff",
            connector: Connector::Fail,
            timeout: operation_timeout(300),
            retry: Some(BoundedRetryStrategy {
                max_attempts: 10,
                delay: Some(Duration::from_secs(2)),
                initial_delay: None,
            }),
            caller_abandons: false,
            attempts: 1,
            responses: 0,
            expected: row(Once, PerAttempt, Never, PerAttempt, Once),
            // The attempt itself failed on the connector before the operation budget expired, so
            // the attempt pair sees the real error and only the execution pair sees the timeout.
            expected_observations: observations(Observed::ConnectorFailure, Observed::Timeout),
        },
        Scenario {
            // The negative control for the in-flight marker: the operation budget expires while
            // the initial request is still stalled, so no attempt ever starts. The attempt-level
            // finalizers must NOT fire -- `read_after_attempt` is owed only once
            // `read_before_attempt` has run -- while the execution-level pair still must. A fix
            // that settled the orphaned attempt unconditionally instead of consulting the marker
            // would fire them here and be caught.
            name: "operation timeout before any attempt starts",
            connector: Connector::Never,
            timeout: operation_timeout(200),
            retry: Some(BoundedRetryStrategy {
                max_attempts: 1,
                delay: None,
                initial_delay: Some(Duration::from_secs(2)),
            }),
            caller_abandons: false,
            attempts: 0,
            responses: 0,
            expected: row(Once, Never, Never, Never, Once),
            expected_observations: vec![
                ("modify_before_completion", Observed::Timeout),
                ("read_after_execution", Observed::Timeout),
            ],
        },
        Scenario {
            // A dropped future cannot run anything: `finally_attempt` and `finally_op` live inside
            // the `async move` block the caller just dropped, and Rust has no async drop. These
            // zeros are structural rather than a gap left to fix later -- the only way to change
            // them is to run the finalizers from `Drop`, which this design rejected. If this row
            // fails, that architectural choice changed.
            name: "caller abandons the future",
            connector: Connector::Never,
            timeout: None,
            retry: None,
            caller_abandons: true,
            attempts: 1,
            responses: 0,
            expected: row(Once, PerAttempt, Never, Never, Never),
            expected_observations: vec![],
        },
    ]
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// Installs one connector for the whole run; cloning shares its call counter.
fn with_connector(
    components: RuntimeComponentsBuilder,
    connector: ScriptedConnector,
) -> RuntimeComponentsBuilder {
    components.with_http_client(Some(http_client_fn(move |_, _| {
        connector.clone().into_shared()
    })))
}

async fn run(scenario: &Scenario) -> HookLog {
    let log = HookLog::default();
    let interceptor = SharedInterceptor::new(PlainRecorder(log.clone()));

    let mut components = RuntimeComponentsBuilder::new("matrix").with_interceptor(interceptor);
    components = match scenario.connector {
        Connector::Never => components.with_http_client(Some(NeverClient::new())),
        Connector::Ok => with_connector(components, ScriptedConnector::always_ok()),
        Connector::Fail => with_connector(components, ScriptedConnector::always_failing()),
        Connector::FailThenOk => with_connector(components, ScriptedConnector::failing_first(1)),
    };
    if let Some(retry) = &scenario.retry {
        components =
            components.with_retry_strategy(Some(SharedRetryStrategy::new(BoundedRetryStrategy {
                max_attempts: retry.max_attempts,
                delay: retry.delay,
                initial_delay: retry.initial_delay,
            })));
    }

    let plugins = RuntimePlugins::new()
        .with_client_plugin(BaseRuntimePlugin::new())
        // Registered as operation plugins so they override the panicking fake sleep that
        // `RuntimeComponentsBuilder::for_tests()` installs.
        .with_operation_plugin(default_sleep_impl_plugin().expect("sleep impl available"))
        .with_operation_plugin(default_time_source_plugin().expect("time source available"))
        .with_operation_plugin(NoAuthRuntimePluginV2::new())
        .with_operation_plugin(ScenarioPlugin {
            components,
            timeout: scenario.timeout.clone(),
        });

    let operation = invoke_with_stop_point(
        "test",
        "test",
        Input::doesnt_matter(),
        &plugins,
        StopPoint::None,
    );

    if scenario.caller_abandons {
        // Drop the future mid-flight.
        let _ = tokio::time::timeout(Duration::from_millis(150), operation).await;
    } else {
        let _ = operation.await;
    }

    log
}

#[tokio::test]
async fn hook_expectation_matrix() {
    // Adding a hook to the runtime, or reordering the list, must force this matrix to be updated.
    assert_eq!(
        HOOKS,
        known_hooks!(),
        "hook list drifted from KNOWN_HOOKS; add the new hook to HOOKS and to every scenario row"
    );

    let position = |hook: &str| {
        HOOKS
            .iter()
            .position(|h| *h == hook)
            .unwrap_or_else(|| panic!("recorded unknown hook {hook}"))
    };
    let attempt_restart = position("read_before_attempt");

    let mut failures: Vec<String> = Vec::new();

    for scenario in &scenarios() {
        assert_eq!(
            scenario.expected.len(),
            HOOKS.len(),
            "{}: expectation row width does not match hook count",
            scenario.name
        );

        {
            let log = run(scenario).await;
            let label = scenario.name;

            // The attempt count anchors `PerAttempt`, so verify it first -- otherwise a wrong
            // attempt count would silently satisfy the per-attempt rows.
            let started = log.count("read_before_attempt");
            if started != scenario.attempts {
                failures.push(format!(
                    "{label}: {started} attempts started, expected {}",
                    scenario.attempts
                ));
            }

            // 1. Counts.
            for (index, hook) in HOOKS.iter().enumerate() {
                let expected = match scenario.expected[index] {
                    Never => 0,
                    Once => 1,
                    PerAttempt => scenario.attempts,
                    PerResponse => scenario.responses,
                };
                let actual = log.count(hook);
                if actual != expected {
                    failures.push(format!(
                        "{label}: {hook} invoked {actual} times, expected {expected} ({:?})",
                        scenario.expected[index]
                    ));
                }
            }

            // 2. Error fidelity: a finalizer that fires but sees nothing cannot do its job.
            let observed = log.observed.lock().unwrap().clone();
            if observed != scenario.expected_observations {
                failures.push(format!(
                    "{label}: finalizers observed {observed:?}, expected {:?}",
                    scenario.expected_observations
                ));
            }

            // 3. Lifecycle ordering.
            for window in log.sequence.lock().unwrap().windows(2) {
                let (from, to) = (position(window[0]), position(window[1]));
                if !(to > from || to == attempt_restart) {
                    failures.push(format!(
                        "{label}: {} ran before {}, which is out of lifecycle order",
                        window[0], window[1]
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} hook matrix mismatch(es):\n  {}",
        failures.len(),
        failures.join("\n  ")
    );
}

/// An interceptor that overrides exactly one hook must still have that hook dispatched.
///
/// Dispatch is gated on `overridden_hooks()`, and the matrix above cannot exercise that gate: its
/// recorder implements all 19 hooks, so the generated bitmask is all-ones and every flag check
/// passes regardless of which flag it consults. A sparse bitmask is the only way to catch a hook
/// wired to the wrong `OverriddenHooks` constant -- verified by mispairing `read_before_signing`
/// with `READ_AFTER_SIGNING`, which leaves this test's hook uninvoked while the matrix stays green.
#[tokio::test]
async fn one_overridden_hook_still_dispatches() {
    #[derive(Clone, Debug, Default)]
    struct SingleHookRecorder(HookLog);

    #[dyn_dispatch_hint]
    impl Intercept for SingleHookRecorder {
        fn name(&self) -> &'static str {
            "SingleHookRecorder"
        }

        fn read_before_signing(
            &self,
            _ctx: &BeforeTransmitInterceptorContextRef<'_>,
            _components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            self.0.record("read_before_signing");
            Ok(())
        }
    }

    let log = HookLog::default();
    let components = RuntimeComponentsBuilder::new("single_hook")
        .with_interceptor(SharedInterceptor::new(SingleHookRecorder(log.clone())));
    let plugins = RuntimePlugins::new()
        .with_client_plugin(BaseRuntimePlugin::new())
        .with_operation_plugin(default_sleep_impl_plugin().expect("sleep impl available"))
        .with_operation_plugin(default_time_source_plugin().expect("time source available"))
        .with_operation_plugin(NoAuthRuntimePluginV2::new())
        .with_operation_plugin(ScenarioPlugin {
            components,
            timeout: None,
        });

    let _ = invoke_with_stop_point(
        "test",
        "test",
        Input::doesnt_matter(),
        &plugins,
        StopPoint::None,
    )
    .await;

    assert_eq!(
        log.count("read_before_signing"),
        1,
        "the single overridden hook must be dispatched"
    );
    // Nothing else was overridden, so nothing else may run.
    assert_eq!(
        log.counts.lock().unwrap().len(),
        1,
        "only the overridden hook may be dispatched, saw {:?}",
        log.counts.lock().unwrap()
    );
}
