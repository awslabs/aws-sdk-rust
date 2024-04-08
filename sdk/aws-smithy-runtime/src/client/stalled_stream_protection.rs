/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::client::http::body::minimum_throughput::MinimumThroughputBody;
use aws_smithy_async::rt::sleep::SharedAsyncSleep;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeDeserializationInterceptorContextMut, BeforeTransmitInterceptorContextMut,
};
use aws_smithy_runtime_api::client::interceptors::Intercept;
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_runtime_api::client::stalled_stream_protection::StalledStreamProtectionConfig;
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use std::mem;

/// Adds stalled stream protection when sending requests and/or receiving responses.
#[derive(Debug)]
pub struct StalledStreamProtectionInterceptor {
    enable_for_request_body: bool,
    enable_for_response_body: bool,
}

/// Stalled stream protection can be enable for request bodies, response bodies,
/// or both.
pub enum StalledStreamProtectionInterceptorKind {
    /// Enable stalled stream protection for request bodies.
    RequestBody,
    /// Enable stalled stream protection for response bodies.
    ResponseBody,
    /// Enable stalled stream protection for both request and response bodies.
    RequestAndResponseBody,
}

impl StalledStreamProtectionInterceptor {
    /// Create a new stalled stream protection interceptor.
    pub fn new(kind: StalledStreamProtectionInterceptorKind) -> Self {
        use StalledStreamProtectionInterceptorKind::*;
        let (enable_for_request_body, enable_for_response_body) = match kind {
            RequestBody => (true, false),
            ResponseBody => (false, true),
            RequestAndResponseBody => (true, true),
        };

        Self {
            enable_for_request_body,
            enable_for_response_body,
        }
    }
}

impl Intercept for StalledStreamProtectionInterceptor {
    fn name(&self) -> &'static str {
        "StalledStreamProtectionInterceptor"
    }

    fn modify_before_transmit(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if self.enable_for_request_body {
            if let Some(cfg) = cfg.load::<StalledStreamProtectionConfig>() {
                if cfg.is_enabled() {
                    let (async_sleep, time_source) =
                        get_runtime_component_deps(runtime_components)?;
                    tracing::trace!("adding stalled stream protection to request body");
                    add_stalled_stream_protection_to_body(
                        context.request_mut().body_mut(),
                        cfg,
                        async_sleep,
                        time_source,
                    );
                }
            }
        }

        Ok(())
    }

    fn modify_before_deserialization(
        &self,
        context: &mut BeforeDeserializationInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        if self.enable_for_response_body {
            if let Some(cfg) = cfg.load::<StalledStreamProtectionConfig>() {
                if cfg.is_enabled() {
                    let (async_sleep, time_source) =
                        get_runtime_component_deps(runtime_components)?;
                    tracing::trace!("adding stalled stream protection to response body");
                    add_stalled_stream_protection_to_body(
                        context.response_mut().body_mut(),
                        cfg,
                        async_sleep,
                        time_source,
                    );
                }
            }
        }
        Ok(())
    }
}

fn get_runtime_component_deps(
    runtime_components: &RuntimeComponents,
) -> Result<(SharedAsyncSleep, SharedTimeSource), BoxError> {
    let async_sleep = runtime_components.sleep_impl().ok_or(
        "An async sleep implementation is required when stalled stream protection is enabled",
    )?;
    let time_source = runtime_components
        .time_source()
        .ok_or("A time source is required when stalled stream protection is enabled")?;
    Ok((async_sleep, time_source))
}

fn add_stalled_stream_protection_to_body(
    body: &mut SdkBody,
    cfg: &StalledStreamProtectionConfig,
    async_sleep: SharedAsyncSleep,
    time_source: SharedTimeSource,
) {
    let cfg = cfg.clone();
    let it = mem::replace(body, SdkBody::taken());
    let it = it.map_preserve_contents(move |body| {
        let cfg = cfg.clone();
        let async_sleep = async_sleep.clone();
        let time_source = time_source.clone();
        let mtb = MinimumThroughputBody::new(time_source, async_sleep, body, cfg.into());
        SdkBody::from_body_0_4(mtb)
    });
    let _ = mem::replace(body, it);
}
