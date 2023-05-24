/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use self::auth::orchestrate_auth;
use crate::client::orchestrator::endpoints::orchestrate_endpoint;
use crate::client::orchestrator::http::read_body;
use crate::client::timeout::{MaybeTimeout, ProvideMaybeTimeoutConfig, TimeoutKind};
use aws_smithy_http::result::SdkError;
use aws_smithy_runtime_api::client::interceptors::context::phase::BeforeSerialization;
use aws_smithy_runtime_api::client::interceptors::context::{
    AttemptCheckpoint, Error, Input, Output,
};
use aws_smithy_runtime_api::client::interceptors::{InterceptorContext, Interceptors};
use aws_smithy_runtime_api::client::orchestrator::{BoxError, ConfigBagAccessors, HttpResponse};
use aws_smithy_runtime_api::client::retries::ShouldAttempt;
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins;
use aws_smithy_runtime_api::config_bag::ConfigBag;
use tracing::{debug_span, Instrument};

mod auth;
/// Defines types that implement a trait for endpoint resolution
pub mod endpoints;
mod http;

#[doc(hidden)]
#[macro_export]
macro_rules! handle_err {
    ([$checkpoint:expr] => $expr:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => {
                return Err($checkpoint.into_error(err.into()));
            }
        }
    };
    ($ctx:expr => $expr:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => {
                use aws_smithy_runtime_api::client::interceptors::context::phase::Phase;
                let (_input, output_or_error, _request, response, phase) = $ctx.into_parts();
                return Err(phase.convert_error(err.into(), output_or_error, response));
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! bail {
    ([$checkpoint:expr], $reason:expr) => {{
        return Err($checkpoint.into_error($reason.into()));
    }};
    ($ctx:expr, $reason:expr) => {{
        use aws_smithy_runtime_api::client::interceptors::context::phase::Phase;
        let reason: BoxError = $reason.into();
        let (_input, output_or_error, _request, response, phase) = $ctx.into_parts();
        return Err(phase.convert_error(reason, output_or_error, response));
    }};
}

#[tracing::instrument(skip_all)]
pub async fn invoke(
    input: Input,
    runtime_plugins: &RuntimePlugins,
) -> Result<Output, SdkError<Error, HttpResponse>> {
    let mut cfg = ConfigBag::base();
    let cfg = &mut cfg;

    let mut interceptors = Interceptors::new();
    let context = InterceptorContext::<()>::new(input);

    // Client configuration
    handle_err!(context => runtime_plugins.apply_client_configuration(cfg, &mut interceptors));
    handle_err!(context => interceptors.client_read_before_execution(&context, cfg));
    // Operation configuration
    handle_err!(context => runtime_plugins.apply_operation_configuration(cfg, &mut interceptors));
    handle_err!(context => interceptors.operation_read_before_execution(&context, cfg));

    let operation_timeout_config = cfg.maybe_timeout_config(TimeoutKind::Operation);
    invoke_post_config(cfg, context, interceptors)
        .maybe_timeout_with_config(operation_timeout_config)
        .await
}

async fn invoke_post_config(
    cfg: &mut ConfigBag,
    mut before_serialization: InterceptorContext<BeforeSerialization>,
    interceptors: Interceptors,
) -> Result<Output, SdkError<Error, HttpResponse>> {
    // Before serialization
    handle_err!(before_serialization => interceptors.read_before_serialization(&before_serialization, cfg));
    handle_err!(before_serialization => interceptors.modify_before_serialization(&mut before_serialization, cfg));

    // Serialization
    let mut serialization = before_serialization.into_serialization_phase();
    {
        let request_serializer = cfg.request_serializer();
        let request = handle_err!(serialization => request_serializer
            .serialize_input(serialization.take_input().expect("input set at this point")));
        serialization.set_request(request);
    }

    // Before transmit
    let mut before_transmit = serialization.into_before_transmit_phase();
    handle_err!(before_transmit => interceptors.read_after_serialization(&before_transmit, cfg));
    handle_err!(before_transmit => interceptors.modify_before_retry_loop(&mut before_transmit, cfg));

    {
        let retry_strategy = cfg.retry_strategy();
        match retry_strategy.should_attempt_initial_request(cfg) {
            // Yes, let's make a request
            Ok(ShouldAttempt::Yes) => {}
            // No, this request shouldn't be sent
            Ok(ShouldAttempt::No) => {
                bail!(before_transmit, "The retry strategy indicates that an initial request shouldn't be made, but it didn't specify why.");
            }
            // No, we shouldn't make a request because...
            Err(err) => bail!(before_transmit, err),
            Ok(ShouldAttempt::YesAfterDelay(_)) => {
                unreachable!("Delaying the initial request is currently unsupported. If this feature is important to you, please file an issue in GitHub.")
            }
        }
    }

    let mut checkpoint = AttemptCheckpoint::new(before_transmit);
    checkpoint = loop {
        if !checkpoint.rewind(cfg) {
            break checkpoint;
        }
        let attempt_timeout_config = cfg.maybe_timeout_config(TimeoutKind::OperationAttempt);

        checkpoint = make_an_attempt(checkpoint, cfg, &interceptors)
            .maybe_timeout_with_config(attempt_timeout_config)
            .await?;
        handle_err!([checkpoint] => interceptors.read_after_attempt(checkpoint.after_deser(), cfg));
        handle_err!([checkpoint] => interceptors.modify_before_attempt_completion(checkpoint.after_deser(), cfg));

        let retry_strategy = cfg.retry_strategy();
        match retry_strategy.should_attempt_retry(checkpoint.after_deser(), cfg) {
            // Yes, let's retry the request
            Ok(ShouldAttempt::Yes) => continue,
            // No, this request shouldn't be retried
            Ok(ShouldAttempt::No) => {}
            Ok(ShouldAttempt::YesAfterDelay(_delay)) => {
                // TODO(enableNewSmithyRuntime): implement retries with explicit delay
                todo!("implement retries with an explicit delay.")
            }
            // I couldn't determine if the request should be retried because an error occurred.
            Err(err) => bail!([checkpoint], err),
        }

        break checkpoint;
    };

    handle_err!([checkpoint] => interceptors.modify_before_completion(checkpoint.after_deser(), cfg));
    handle_err!([checkpoint] => interceptors.read_after_execution(checkpoint.after_deser(), cfg));

    checkpoint.finalize()
}

// Making an HTTP request can fail for several reasons, but we still need to
// call lifecycle events when that happens. Therefore, we define this
// `make_an_attempt` function to make error handling simpler.
#[tracing::instrument(skip_all)]
async fn make_an_attempt(
    mut checkpoint: AttemptCheckpoint,
    cfg: &mut ConfigBag,
    interceptors: &Interceptors,
) -> Result<AttemptCheckpoint, SdkError<Error, HttpResponse>> {
    handle_err!([checkpoint] => interceptors.read_before_attempt(checkpoint.before_transmit(), cfg));
    handle_err!([checkpoint] => orchestrate_endpoint(checkpoint.before_transmit(), cfg));
    handle_err!([checkpoint] => interceptors.modify_before_signing(checkpoint.before_transmit(), cfg));
    handle_err!([checkpoint] => interceptors.read_before_signing(checkpoint.before_transmit(), cfg));

    checkpoint = orchestrate_auth(checkpoint, cfg).await?;

    handle_err!([checkpoint] => interceptors.read_after_signing(checkpoint.before_transmit(), cfg));
    handle_err!([checkpoint] => interceptors.modify_before_transmit(checkpoint.before_transmit(), cfg));
    handle_err!([checkpoint] => interceptors.read_before_transmit(checkpoint.before_transmit(), cfg));

    // The connection consumes the request but we need to keep a copy of it
    // within the interceptor context, so we clone it here.
    checkpoint.transition_to_transmit();
    let call_result = handle_err!([checkpoint] => {
        let request = checkpoint.transmit().take_request();
        cfg.connection().call(request).await
    });
    checkpoint.transmit().set_response(call_result);
    checkpoint.transition_to_before_deserialization();

    handle_err!([checkpoint] => interceptors.read_after_transmit(checkpoint.before_deser(), cfg));
    handle_err!([checkpoint] => interceptors.modify_before_deserialization(checkpoint.before_deser(), cfg));
    handle_err!([checkpoint] => interceptors.read_before_deserialization(checkpoint.before_deser(), cfg));

    checkpoint.transition_to_deserialization();
    let output_or_error = handle_err!([checkpoint] => {
        let response = checkpoint.deser().response_mut();
        let response_deserializer = cfg.response_deserializer();
        match response_deserializer.deserialize_streaming(response) {
            Some(output_or_error) => Ok(output_or_error),
            None => read_body(response)
                .instrument(debug_span!("read_body"))
                .await
                .map(|_| response_deserializer.deserialize_nonstreaming(response)),
        }
    });

    checkpoint.deser().set_output_or_error(output_or_error);

    checkpoint.transition_to_after_deserialization();
    handle_err!([checkpoint] => interceptors.read_after_deserialization(checkpoint.after_deser(), cfg));

    Ok(checkpoint)
}
