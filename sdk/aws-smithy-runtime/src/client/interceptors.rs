/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::BeforeTransmitInterceptorContextMut;
use aws_smithy_runtime_api::client::interceptors::Interceptor;
use aws_smithy_runtime_api::client::orchestrator::HttpRequest;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::ConfigBag;
use std::error::Error as StdError;
use std::fmt;
use std::marker::PhantomData;

pub struct MapRequestInterceptor<F, E> {
    f: F,
    _phantom: PhantomData<E>,
}

impl<F, E> fmt::Debug for MapRequestInterceptor<F, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MapRequestInterceptor")
    }
}

impl<F, E> MapRequestInterceptor<F, E> {
    pub fn new(f: F) -> Self {
        Self {
            f,
            _phantom: PhantomData,
        }
    }
}

impl<F, E> Interceptor for MapRequestInterceptor<F, E>
where
    F: Fn(HttpRequest) -> Result<HttpRequest, E> + Send + Sync + 'static,
    E: StdError + Send + Sync + 'static,
{
    fn modify_before_signing(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let mut request = HttpRequest::new(SdkBody::taken());
        std::mem::swap(&mut request, context.request_mut());
        let mut mapped = (self.f)(request)?;
        std::mem::swap(&mut mapped, context.request_mut());

        Ok(())
    }
}

pub struct MutateRequestInterceptor<F> {
    f: F,
}

impl<F> fmt::Debug for MutateRequestInterceptor<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MutateRequestInterceptor")
    }
}

impl<F> MutateRequestInterceptor<F> {
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> Interceptor for MutateRequestInterceptor<F>
where
    F: Fn(&mut HttpRequest) + Send + Sync + 'static,
{
    fn modify_before_signing(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        _cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let request = context.request_mut();
        (self.f)(request);

        Ok(())
    }
}
