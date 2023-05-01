/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use super::phase::Phase;
use aws_smithy_http::result::SdkError;
use aws_smithy_runtime_api::client::interceptors::context::Error;
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors, HttpResponse};
use aws_smithy_runtime_api::config_bag::ConfigBag;

pub(super) async fn orchestrate_auth(
    dispatch_phase: Phase,
    cfg: &ConfigBag,
) -> Result<Phase, SdkError<Error, HttpResponse>> {
    dispatch_phase.include_mut(|ctx| {
        let params = cfg.auth_option_resolver_params();
        let auth_options = cfg.auth_option_resolver().resolve_auth_options(params)?;
        let identity_resolvers = cfg.identity_resolvers();

        for option in auth_options {
            let scheme_id = option.scheme_id();
            if let Some(auth_scheme) = cfg.http_auth_schemes().scheme(scheme_id) {
                let identity_resolver = auth_scheme.identity_resolver(identity_resolvers);
                let request_signer = auth_scheme.request_signer();

                let identity = identity_resolver.resolve_identity(cfg)?;
                let request = ctx.request_mut()?;
                request_signer.sign_request(request, &identity, cfg)?;
                return Result::<_, BoxError>::Ok(());
            }
        }

        Err("No auth scheme matched auth options. This is a bug. Please file an issue.".into())
    })
}
