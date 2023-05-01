/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::endpoint::EndpointPrefix;
use aws_smithy_runtime_api::client::interceptors::InterceptorContext;
use aws_smithy_runtime_api::client::orchestrator::{
    BoxError, ConfigBagAccessors, HttpRequest, HttpResponse,
};
use aws_smithy_runtime_api::config_bag::ConfigBag;

pub(super) fn orchestrate_endpoint(
    ctx: &mut InterceptorContext<HttpRequest, HttpResponse>,
    cfg: &ConfigBag,
) -> Result<(), BoxError> {
    let params = cfg.endpoint_resolver_params();
    let endpoint_prefix = cfg.get::<EndpointPrefix>();
    let request = ctx.request_mut()?;

    let endpoint_resolver = cfg.endpoint_resolver();
    endpoint_resolver.resolve_and_apply_endpoint(params, endpoint_prefix, request)?;

    Ok(())
}
