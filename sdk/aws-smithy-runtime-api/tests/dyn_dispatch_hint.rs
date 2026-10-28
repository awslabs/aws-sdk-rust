/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "client")]

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    AfterDeserializationInterceptorContextRef, BeforeDeserializationInterceptorContextMut,
    BeforeDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextMut,
    BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextMut,
    BeforeTransmitInterceptorContextRef, FinalizerInterceptorContextMut,
    FinalizerInterceptorContextRef,
};
use aws_smithy_runtime_api::client::interceptors::{dyn_dispatch_hint, Intercept, OverriddenHooks};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;

#[derive(Debug)]
struct SingleHook;

#[dyn_dispatch_hint]
impl Intercept for SingleHook {
    fn name(&self) -> &'static str {
        "SingleHook"
    }
    fn modify_before_signing(
        &self,
        _context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
}

#[test]
fn single_hook_sets_one_flag() {
    assert_eq!(
        SingleHook.overridden_hooks(),
        OverriddenHooks::MODIFY_BEFORE_SIGNING,
    );
}

#[derive(Debug)]
struct MultipleHooks;

#[dyn_dispatch_hint]
impl Intercept for MultipleHooks {
    fn name(&self) -> &'static str {
        "MultipleHooks"
    }
    fn read_before_execution(
        &self,
        _context: &BeforeSerializationInterceptorContextRef<'_>,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_execution(
        &self,
        _context: &FinalizerInterceptorContextRef<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
}

#[test]
fn multiple_hooks_or_flags_together() {
    let flags = MultipleHooks.overridden_hooks();
    let expected = OverriddenHooks::READ_BEFORE_EXECUTION | OverriddenHooks::READ_AFTER_EXECUTION;
    assert_eq!(
        flags, expected,
        "should contain exactly the two overridden hooks and nothing else"
    );
}

#[derive(Debug)]
struct NoHooks;

#[dyn_dispatch_hint]
impl Intercept for NoHooks {
    fn name(&self) -> &'static str {
        "NoHooks"
    }
}

#[test]
fn no_hooks_returns_none() {
    assert_eq!(NoHooks.overridden_hooks(), OverriddenHooks::none());
}

#[derive(Debug)]
struct NoMacro;

impl Intercept for NoMacro {
    fn name(&self) -> &'static str {
        "NoMacro"
    }
}

#[test]
fn without_macro_returns_all() {
    assert_eq!(NoMacro.overridden_hooks(), OverriddenHooks::all());
}

macro_rules! generate_hook {
    ($name:ident) => {
        fn $name(
            &self,
            _context: &mut BeforeTransmitInterceptorContextMut<'_>,
            _runtime_components: &RuntimeComponents,
            _cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            Ok(())
        }
    };
}

#[derive(Debug)]
struct MacroGeneratedHook;

#[dyn_dispatch_hint]
impl Intercept for MacroGeneratedHook {
    fn name(&self) -> &'static str {
        "MacroGeneratedHook"
    }
    generate_hook!(modify_before_signing);
}

/// A hook defined by a declarative macro is *not* detected, so it is never dispatched.
///
/// `#[dyn_dispatch_hint]` parses the impl block before declarative macros expand, so the body above
/// appears as an `ImplItem::Macro` rather than an `ImplItem::Fn`. The hook is implemented but gets
/// no bit in the mask, and `Interceptors` then skips it -- silently, with no warning.
///
/// This test pins that limitation rather than endorsing it. If the macro is ever taught to expand
/// declarative macros first, this assertion will fail; the fix is to expect
/// `OverriddenHooks::MODIFY_BEFORE_SIGNING` here and to delete the warning in the attribute's docs.
#[test]
fn macro_generated_hook_is_not_detected() {
    assert_eq!(
        MacroGeneratedHook.overridden_hooks(),
        OverriddenHooks::none(),
        "a macro-generated hook is invisible to the attribute; write hooks longhand"
    );
}

/// Every hook written longhand, with empty bodies. Exists only to assert the bitmask.
#[derive(Debug)]
struct AllHooks;

#[dyn_dispatch_hint]
impl Intercept for AllHooks {
    fn name(&self) -> &'static str {
        "AllHooks"
    }
    fn read_before_execution(
        &self,
        _c: &BeforeSerializationInterceptorContextRef<'_>,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_serialization(
        &self,
        _c: &mut BeforeSerializationInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_before_serialization(
        &self,
        _c: &BeforeSerializationInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_serialization(
        &self,
        _c: &BeforeTransmitInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_retry_loop(
        &self,
        _c: &mut BeforeTransmitInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_before_attempt(
        &self,
        _c: &BeforeTransmitInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_signing(
        &self,
        _c: &mut BeforeTransmitInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_before_signing(
        &self,
        _c: &BeforeTransmitInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_signing(
        &self,
        _c: &BeforeTransmitInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_transmit(
        &self,
        _c: &mut BeforeTransmitInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_before_transmit(
        &self,
        _c: &BeforeTransmitInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_transmit(
        &self,
        _c: &BeforeDeserializationInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_deserialization(
        &self,
        _c: &mut BeforeDeserializationInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_before_deserialization(
        &self,
        _c: &BeforeDeserializationInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_deserialization(
        &self,
        _c: &AfterDeserializationInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_attempt_completion(
        &self,
        _c: &mut FinalizerInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_attempt(
        &self,
        _c: &FinalizerInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn modify_before_completion(
        &self,
        _c: &mut FinalizerInterceptorContextMut<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
    fn read_after_execution(
        &self,
        _c: &FinalizerInterceptorContextRef<'_>,
        _rc: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        Ok(())
    }
}

/// Every one of the 19 hooks must be detected when written longhand.
///
/// If the attribute fails to notice a hook, its bit is missing, `Interceptors` skips it, and the
/// hook is silently never dispatched despite being implemented. Checking each flag by name covers
/// every hook at once, rather than relying on the handful spot-checked above, and reports which
/// hook was missed.
///
/// Note `OverriddenHooks::all()` is `u32::MAX` -- a permissive default, not the union of the real
/// hooks -- so this compares against the named constants instead.
///
/// This cannot catch a *permuted* mapping: two hooks swapping bits still leaves both set.
/// `one_overridden_hook_still_dispatches` in `aws-smithy-runtime` covers that case with a sparse
/// bitmask, where a wrong flag leaves its hook undispatched.
#[test]
fn every_longhand_hook_is_detected() {
    let mask = AllHooks.overridden_hooks();
    for (hook, flag) in [
        (
            "read_before_execution",
            OverriddenHooks::READ_BEFORE_EXECUTION,
        ),
        (
            "modify_before_serialization",
            OverriddenHooks::MODIFY_BEFORE_SERIALIZATION,
        ),
        (
            "read_before_serialization",
            OverriddenHooks::READ_BEFORE_SERIALIZATION,
        ),
        (
            "read_after_serialization",
            OverriddenHooks::READ_AFTER_SERIALIZATION,
        ),
        (
            "modify_before_retry_loop",
            OverriddenHooks::MODIFY_BEFORE_RETRY_LOOP,
        ),
        ("read_before_attempt", OverriddenHooks::READ_BEFORE_ATTEMPT),
        (
            "modify_before_signing",
            OverriddenHooks::MODIFY_BEFORE_SIGNING,
        ),
        ("read_before_signing", OverriddenHooks::READ_BEFORE_SIGNING),
        ("read_after_signing", OverriddenHooks::READ_AFTER_SIGNING),
        (
            "modify_before_transmit",
            OverriddenHooks::MODIFY_BEFORE_TRANSMIT,
        ),
        (
            "read_before_transmit",
            OverriddenHooks::READ_BEFORE_TRANSMIT,
        ),
        ("read_after_transmit", OverriddenHooks::READ_AFTER_TRANSMIT),
        (
            "modify_before_deserialization",
            OverriddenHooks::MODIFY_BEFORE_DESERIALIZATION,
        ),
        (
            "read_before_deserialization",
            OverriddenHooks::READ_BEFORE_DESERIALIZATION,
        ),
        (
            "read_after_deserialization",
            OverriddenHooks::READ_AFTER_DESERIALIZATION,
        ),
        (
            "modify_before_attempt_completion",
            OverriddenHooks::MODIFY_BEFORE_ATTEMPT_COMPLETION,
        ),
        ("read_after_attempt", OverriddenHooks::READ_AFTER_ATTEMPT),
        (
            "modify_before_completion",
            OverriddenHooks::MODIFY_BEFORE_COMPLETION,
        ),
        (
            "read_after_execution",
            OverriddenHooks::READ_AFTER_EXECUTION,
        ),
    ] {
        assert!(
            mask.contains(flag),
            "`{hook}` is implemented longhand but the attribute did not detect it, \
             so it would never be dispatched"
        );
    }
}
