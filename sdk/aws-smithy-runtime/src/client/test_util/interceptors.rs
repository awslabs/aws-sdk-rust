/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(enableNewSmithyRuntimeCleanup): Delete this file once test helpers on `CustomizableOperation` have been removed

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Interceptor;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;
use std::fmt;

pub struct TestParamsSetterInterceptor<F> {
    f: F,
}

impl<F> fmt::Debug for TestParamsSetterInterceptor<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestParamsSetterInterceptor")
    }
}

impl<F> TestParamsSetterInterceptor<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Interceptor for TestParamsSetterInterceptor<F>
where
    F: Fn(&mut BeforeTransmitInterceptorContextMut<'_>, &mut ConfigBag) + Send + Sync + 'static,
{
    fn modify_before_signing(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        (self.f)(context, cfg);

        Ok(())
    }
}
