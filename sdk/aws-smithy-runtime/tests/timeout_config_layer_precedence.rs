/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Locks the config-layer contract that a per-service default timeout relies on.
//!
//! The AWS SDK bakes a per-service default `read_timeout` into the generated
//! `ServiceRuntimePlugin`'s config layer. That only produces correct behavior because of three
//! properties of machinery outside this design:
//!
//! 1. `ServiceRuntimePlugin` declares [`Order::Defaults`], while the layer carrying caller config
//!    is a `StaticRuntimePlugin` at the default [`Order::Overrides`]. So the service layer is
//!    pushed first and therefore loses to caller config.
//! 2. `MergeTimeoutConfig` merges *per field* across every layer, so a service default for
//!    `read_timeout` does not erase a caller's `connect_timeout` or vice versa.
//! 3. `CanDisable` distinguishes "unset" from "explicitly disabled", so a caller can turn the
//!    default off rather than only replace it.
//!
//! None of that is this crate's own behavior, which is why it is worth pinning: a change to any of
//! the three would break per-service defaults with an absent or unoverridable timeout rather than a
//! compile error.

#![cfg(feature = "client")]

use aws_smithy_runtime::client::defaults::{default_timeout_config_plugin_v2, DefaultPluginParams};
use aws_smithy_runtime_api::client::behavior_version::BehaviorVersion;
use aws_smithy_runtime_api::client::runtime_plugin::{
    Order, RuntimePlugin, RuntimePlugins, SharedRuntimePlugin, StaticRuntimePlugin,
};
use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
use aws_smithy_types::timeout::{MergeTimeoutConfig, TimeoutConfig};
use std::time::Duration;

/// Matches `DEFAULT_CONNECT_TIMEOUT` in `aws_smithy_runtime::client::defaults`, which is private.
const CONNECT: Duration = Duration::from_millis(3100);
/// Stands in for the 5-minute general default from the exemption artifact.
const SERVICE_READ: Duration = Duration::from_secs(300);

/// Stands in for the generated `ServiceRuntimePlugin` carrying a per-service default.
///
/// `read_timeout: None` models a fully-exempt service, which must emit **no entry at all** rather
/// than an empty `TimeoutConfig` — see `exempt_service_must_emit_no_entry`.
#[derive(Debug)]
struct ServicePlugin {
    read_timeout: Option<Duration>,
}

impl RuntimePlugin for ServicePlugin {
    fn order(&self) -> Order {
        // Mirrors ServiceRuntimePluginGenerator.kt, which emits `Order::Defaults`.
        Order::Defaults
    }

    fn config(&self) -> Option<FrozenLayer> {
        let mut layer = Layer::new("AmazonTestService");
        if let Some(d) = self.read_timeout {
            layer.store_put(TimeoutConfig::builder().read_timeout(d).build());
        }
        Some(layer.freeze())
    }
}

fn layer_with(name: &'static str, tc: TimeoutConfig) -> FrozenLayer {
    let mut layer = Layer::new(name);
    layer.store_put(tc);
    layer.freeze()
}

/// Resolves `TimeoutConfig` the way the orchestrator does, with the plugin registration order the
/// generated `base_client_runtime_plugins` uses.
///
/// Note that `ServiceRuntimePlugin` is registered *after* the caller-config plugin but still ends
/// up at lower priority, because `insert_plugin!` sorts by `Order` rather than by registration.
/// Getting that backwards is the whole hazard this file guards.
fn resolve(
    service_default: Option<Duration>,
    caller_client_config: Option<TimeoutConfig>,
    caller_operation_override: Option<TimeoutConfig>,
) -> TimeoutConfig {
    let defaults: Vec<SharedRuntimePlugin> = default_timeout_config_plugin_v2(
        &DefaultPluginParams::new()
            .with_behavior_version(BehaviorVersion::latest())
            .with_retry_partition_name("dontcare"),
    )
    .into_iter()
    .collect();

    let mut plugins = RuntimePlugins::new()
        .with_client_plugins(defaults)
        .with_client_plugin({
            let mut p = StaticRuntimePlugin::new();
            if let Some(tc) = caller_client_config {
                p = p.with_config(layer_with("user_config", tc));
            }
            p
        })
        .with_client_plugin(ServicePlugin {
            read_timeout: service_default,
        });

    if let Some(tc) = caller_operation_override {
        plugins = plugins.with_operation_plugin(
            StaticRuntimePlugin::new().with_config(layer_with("config_override", tc)),
        );
    }

    let mut cfg = ConfigBag::base();
    plugins.apply_client_configuration(&mut cfg).unwrap();
    plugins.apply_operation_configuration(&mut cfg).unwrap();
    cfg.load::<MergeTimeoutConfig>()
}

#[test]
fn service_default_applies_when_caller_sets_nothing() {
    let tc = resolve(Some(SERVICE_READ), None, None);
    assert_eq!(tc.read_timeout(), Some(SERVICE_READ));
    assert_eq!(
        tc.connect_timeout(),
        Some(CONNECT),
        "a service read default must not displace the connect default"
    );
}

#[test]
fn exempt_service_must_emit_no_entry() {
    let tc = resolve(None, None, None);
    assert_eq!(
        tc.read_timeout(),
        None,
        "a service with no default must end up with no read timeout"
    );
    assert_eq!(
        tc.connect_timeout(),
        Some(CONNECT),
        "omitting the entry must leave the connect default intact"
    );
}

#[test]
fn caller_client_config_beats_service_default() {
    let caller = TimeoutConfig::builder()
        .read_timeout(Duration::from_secs(10))
        .build();
    let tc = resolve(Some(SERVICE_READ), Some(caller), None);
    assert_eq!(tc.read_timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn caller_operation_override_beats_service_default() {
    let override_ = TimeoutConfig::builder()
        .read_timeout(Duration::from_secs(7))
        .build();
    let tc = resolve(Some(SERVICE_READ), None, Some(override_));
    assert_eq!(
        tc.read_timeout(),
        Some(Duration::from_secs(7)),
        "per-operation config_override must win"
    );
}

#[test]
fn caller_disabling_only_the_read_timeout_beats_service_default() {
    let mut b = TimeoutConfig::builder();
    b.set_read_timeout(None);
    let tc = resolve(Some(SERVICE_READ), Some(b.build()), None);
    assert_eq!(
        tc.read_timeout(),
        None,
        "set_read_timeout(None) must suppress the service default"
    );
}

/// The lossless way to opt out of a per-service default: ask for a longer one.
///
/// Prefer this over `set_read_timeout(None)` in customer-facing guidance, because disabling has
/// the side effect documented in `disabling_a_timeout_also_truncates_the_merge_chain`.
#[test]
fn caller_can_opt_out_losslessly_with_a_longer_read_timeout() {
    let caller = TimeoutConfig::builder()
        .read_timeout(Duration::from_secs(3600))
        .build();
    let tc = resolve(Some(SERVICE_READ), Some(caller), None);
    assert_eq!(tc.read_timeout(), Some(Duration::from_secs(3600)));
    assert_eq!(
        tc.connect_timeout(),
        Some(CONNECT),
        "replacing the read default must leave the connect default intact"
    );
}

#[test]
fn caller_disabling_all_timeouts_beats_service_default() {
    let tc = resolve(Some(SERVICE_READ), Some(TimeoutConfig::disabled()), None);
    assert_eq!(tc.read_timeout(), None);
    assert_eq!(tc.connect_timeout(), None);
}

#[test]
fn unrelated_caller_setting_does_not_suppress_the_service_default() {
    let caller = TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(1))
        .build();
    let tc = resolve(Some(SERVICE_READ), Some(caller), None);
    assert_eq!(tc.connect_timeout(), Some(Duration::from_secs(1)));
    assert_eq!(
        tc.read_timeout(),
        Some(SERVICE_READ),
        "setting connect_timeout must not consume the read default"
    );
}

/// A per-operation value reinstates a timeout the client disabled.
///
/// `Disabled` blocks layers *below* it, not above, so "disabled on the client" is not absolute.
/// That is the intended precedence — per-operation config is more specific than client config —
/// and it means `config_override` can re-enable a timeout for one call.
#[test]
fn operation_override_reinstates_a_client_level_disable() {
    let mut disabled = TimeoutConfig::builder();
    disabled.set_read_timeout(None);
    let override_ = TimeoutConfig::builder()
        .read_timeout(Duration::from_secs(7))
        .build();
    let tc = resolve(Some(SERVICE_READ), Some(disabled.build()), Some(override_));
    assert_eq!(
        tc.read_timeout(),
        Some(Duration::from_secs(7)),
        "a per-operation value must beat a client-level disable"
    );
    assert_eq!(tc.connect_timeout(), Some(CONNECT));
}

/// Setting any other timeout alongside a disable avoids the truncation below.
///
/// This is the bound on that sharp edge: `has_timeouts()` becomes true as soon as *one* field is
/// `Set`, so the merge chain is absorbed normally and only the explicitly disabled field is lost.
#[test]
fn disabling_read_alongside_another_timeout_preserves_other_defaults() {
    let mut caller = TimeoutConfig::builder().connect_timeout(Duration::from_secs(1));
    caller.set_read_timeout(None);
    let tc = resolve(Some(SERVICE_READ), Some(caller.build()), None);
    assert_eq!(tc.read_timeout(), None, "the disable still applies");
    assert_eq!(
        tc.connect_timeout(),
        Some(Duration::from_secs(1)),
        "and nothing else is collaterally lost"
    );
}

/// Documents a pre-existing sharp edge, not desired behavior.
///
/// `merge_iter` stops absorbing lower-priority layers once the accumulated value reports
/// `has_timeouts() == false`. That predicate tests each field for `Set`, and `Disabled` is not
/// `Set`, so a config whose only non-`Unset` field is *disabled* truncates the chain too. Both
/// variants below already drop the 3.1s connect default today, with no service layer in play, so
/// this design does not introduce it.
///
/// It fires only when the caller sets nothing else — see
/// `disabling_read_alongside_another_timeout_preserves_other_defaults`. The consequence for this
/// design is that a fully-exempt service must emit no layer entry rather than an empty
/// `TimeoutConfig`. Asserting today's behavior means a future fix trips this test deliberately.
#[test]
fn disabling_a_timeout_also_truncates_the_merge_chain() {
    // Variant 1: an empty TimeoutConfig.
    let empty = resolve(
        Some(SERVICE_READ),
        Some(TimeoutConfig::builder().build()),
        None,
    );
    assert_eq!(empty.read_timeout(), None, "pre-existing: chain truncated");
    assert_eq!(
        empty.connect_timeout(),
        None,
        "pre-existing: an empty TimeoutConfig drops the connect default"
    );

    // Variant 2: only the read timeout explicitly disabled. `Disabled` is not `Set`, so
    // `has_timeouts()` is still false and the connect default is lost too.
    let mut b = TimeoutConfig::builder();
    b.set_read_timeout(None);
    let read_disabled = resolve(Some(SERVICE_READ), Some(b.build()), None);
    assert_eq!(read_disabled.read_timeout(), None);
    assert_eq!(
        read_disabled.connect_timeout(),
        None,
        "pre-existing: disabling only the read timeout also drops the connect default"
    );

    // The same thing happens with no per-service default, which is what makes it pre-existing.
    let mut b = TimeoutConfig::builder();
    b.set_read_timeout(None);
    let without_service_default = resolve(None, Some(b.build()), None);
    assert_eq!(
        without_service_default.connect_timeout(),
        None,
        "pre-existing independently of this design"
    );
}
