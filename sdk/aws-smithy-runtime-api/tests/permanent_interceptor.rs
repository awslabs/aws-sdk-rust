/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![cfg(feature = "client")]

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::{
    disable_interceptor, Intercept, SharedInterceptor,
};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Layer};

#[derive(Debug)]
struct TestInterceptor;

impl Intercept for TestInterceptor {
    fn name(&self) -> &'static str {
        "TestInterceptor"
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
fn permanent_interceptor_is_always_enabled() {
    let interceptor = SharedInterceptor::permanent(TestInterceptor);
    let cfg = ConfigBag::base();
    assert!(interceptor.enabled(&cfg));
}

#[test]
fn new_interceptor_can_be_disabled() {
    let interceptor = SharedInterceptor::new(TestInterceptor);
    let mut cfg = ConfigBag::base();
    let mut layer = Layer::new("test");
    layer.store_put(disable_interceptor::<TestInterceptor>("test"));
    cfg.push_shared_layer(layer.freeze());
    assert!(!interceptor.enabled(&cfg));
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "attempted to disable permanent interceptor")]
fn permanent_interceptor_panics_on_disable_in_debug() {
    let interceptor = SharedInterceptor::permanent(TestInterceptor);
    let mut cfg = ConfigBag::base();
    let mut layer = Layer::new("test");
    layer.store_put(disable_interceptor::<TestInterceptor>("test"));
    cfg.push_shared_layer(layer.freeze());
    interceptor.enabled(&cfg);
}
