/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(enableNewSmithyRuntime): Delete this file once test helpers on `CustomizableOperation` have been removed

use aws_smithy_runtime_api::client::interceptors::{
    BeforeTransmitInterceptorContextMut, BoxError, Interceptor,
};
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
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        (self.f)(context, cfg);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_http::body::SdkBody;
    use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
    use aws_smithy_runtime_api::client::orchestrator::ConfigBagAccessors;
    use aws_smithy_types::type_erasure::TypedBox;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn set_test_request_time() {
        let mut cfg = ConfigBag::base();
        let mut ctx = InterceptorContext::new(TypedBox::new("anything").erase());
        ctx.enter_serialization_phase();
        ctx.set_request(http::Request::builder().body(SdkBody::empty()).unwrap());
        let _ = ctx.take_input();
        ctx.enter_before_transmit_phase();
        let mut ctx = Into::into(&mut ctx);
        let request_time = UNIX_EPOCH + Duration::from_secs(1624036048);
        let interceptor = TestParamsSetterInterceptor::new(
            move |_: &mut BeforeTransmitInterceptorContextMut<'_>, cfg: &mut ConfigBag| {
                cfg.set_request_time(request_time);
            },
        );
        interceptor
            .modify_before_signing(&mut ctx, &mut cfg)
            .unwrap();
        assert_eq!(cfg.request_time().unwrap().now(), request_time);
    }
}
