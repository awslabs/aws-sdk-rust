/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// TODO(msrvUpgrade): This can be removed once we upgrade the MSRV to Rust 1.69
#![allow(unknown_lints)]

use self::auth::orchestrate_auth;
use crate::client::orchestrator::endpoints::orchestrate_endpoint;
use crate::client::orchestrator::http::read_body;
use crate::client::timeout::{MaybeTimeout, ProvideMaybeTimeoutConfig, TimeoutKind};
use aws_smithy_async::rt::sleep::AsyncSleep;
use aws_smithy_http::body::SdkBody;
use aws_smithy_http::byte_stream::ByteStream;
use aws_smithy_http::result::SdkError;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::config_bag_accessors::ConfigBagAccessors;
use aws_smithy_runtime_api::client::interceptors::context::{
    Error, Input, InterceptorContext, Output, RewindResult,
};
use aws_smithy_runtime_api::client::interceptors::Interceptors;
use aws_smithy_runtime_api::client::orchestrator::{
    HttpResponse, LoadedRequestBody, OrchestratorError, RequestSerializer,
};
use aws_smithy_runtime_api::client::request_attempts::RequestAttempts;
use aws_smithy_runtime_api::client::retries::ShouldAttempt;
use aws_smithy_runtime_api::client::runtime_plugin::RuntimePlugins;
use aws_smithy_types::config_bag::ConfigBag;
use std::mem;
use tracing::{debug, debug_span, instrument, Instrument};

mod auth;
/// Defines types that implement a trait for endpoint resolution
pub mod endpoints;
mod http;
pub mod interceptors;

macro_rules! halt {
    ([$ctx:ident] => $err:expr) => {{
        debug!("encountered orchestrator error; halting");
        $ctx.fail($err.into());
        return;
    }};
}

macro_rules! halt_on_err {
    ([$ctx:ident] => $expr:expr) => {
        match $expr {
            Ok(ok) => ok,
            Err(err) => halt!([$ctx] => err),
        }
    };
}

macro_rules! continue_on_err {
    ([$ctx:ident] => $expr:expr) => {
        if let Err(err) = $expr {
            debug!("encountered orchestrator error; continuing");
            $ctx.fail(err.into());
        }
    };
}

pub async fn invoke(
    input: Input,
    runtime_plugins: &RuntimePlugins,
) -> Result<Output, SdkError<Error, HttpResponse>> {
    invoke_with_stop_point(input, runtime_plugins, StopPoint::None)
        .await?
        .finalize()
}

/// Allows for returning early at different points during orchestration.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopPoint {
    /// Don't stop orchestration early
    None,

    /// Stop the orchestrator before transmitting the request
    BeforeTransmit,
}

#[tracing::instrument(skip_all, name = "invoke")]
pub async fn invoke_with_stop_point(
    input: Input,
    runtime_plugins: &RuntimePlugins,
    stop_point: StopPoint,
) -> Result<InterceptorContext, SdkError<Error, HttpResponse>> {
    let mut cfg = ConfigBag::base();
    let cfg = &mut cfg;

    let mut interceptors = Interceptors::new();
    let mut ctx = InterceptorContext::new(input);

    if let Err(err) = apply_configuration(&mut ctx, cfg, &mut interceptors, runtime_plugins) {
        return Err(SdkError::construction_failure(err));
    }
    let operation_timeout_config = cfg.maybe_timeout_config(TimeoutKind::Operation);
    async {
        // If running the pre-execution interceptors failed, then we skip running the op and run the
        // final interceptors instead.
        if !ctx.is_failed() {
            try_op(&mut ctx, cfg, &interceptors, stop_point).await;
        }
        finally_op(&mut ctx, cfg, &interceptors).await;
        Ok(ctx)
    }
    .maybe_timeout_with_config(operation_timeout_config)
    .await
}

/// Apply configuration is responsible for apply runtime plugins to the config bag, as well as running
/// `read_before_execution` interceptors. If a failure occurs due to config construction, `invoke`
/// will raise it to the user. If an interceptor fails, then `invoke`
#[instrument(skip_all)]
fn apply_configuration(
    ctx: &mut InterceptorContext,
    cfg: &mut ConfigBag,
    interceptors: &mut Interceptors,
    runtime_plugins: &RuntimePlugins,
) -> Result<(), BoxError> {
    runtime_plugins.apply_client_configuration(cfg, interceptors.client_interceptors_mut())?;
    continue_on_err!([ctx] => interceptors.client_read_before_execution(ctx, cfg));
    runtime_plugins
        .apply_operation_configuration(cfg, interceptors.operation_interceptors_mut())?;
    continue_on_err!([ctx] => interceptors.operation_read_before_execution(ctx, cfg));

    Ok(())
}

#[instrument(skip_all)]
async fn try_op(
    ctx: &mut InterceptorContext,
    cfg: &mut ConfigBag,
    interceptors: &Interceptors,
    stop_point: StopPoint,
) {
    // Before serialization
    halt_on_err!([ctx] => interceptors.read_before_serialization(ctx, cfg));
    halt_on_err!([ctx] => interceptors.modify_before_serialization(ctx, cfg));

    // Serialization
    ctx.enter_serialization_phase();
    {
        let request_serializer = cfg.request_serializer();
        let input = ctx.take_input().expect("input set at this point");
        let request = halt_on_err!([ctx] => request_serializer.serialize_input(input, cfg).map_err(OrchestratorError::other));
        ctx.set_request(request);
    }

    // Load the request body into memory if configured to do so
    if let LoadedRequestBody::Requested = cfg.loaded_request_body() {
        let mut body = SdkBody::taken();
        mem::swap(&mut body, ctx.request_mut().expect("set above").body_mut());
        let loaded_body = halt_on_err!([ctx] => ByteStream::new(body).collect().await).into_bytes();
        *ctx.request_mut().as_mut().expect("set above").body_mut() =
            SdkBody::from(loaded_body.clone());
        cfg.interceptor_state()
            .set_loaded_request_body(LoadedRequestBody::Loaded(loaded_body));
    }

    // Before transmit
    ctx.enter_before_transmit_phase();
    halt_on_err!([ctx] => interceptors.read_after_serialization(ctx, cfg));
    halt_on_err!([ctx] => interceptors.modify_before_retry_loop(ctx, cfg));

    let retry_strategy = cfg.retry_strategy();
    // If we got a retry strategy from the bag, ask it what to do.
    // Otherwise, assume we should attempt the initial request.
    let should_attempt = retry_strategy
        .map(|rs| rs.should_attempt_initial_request(cfg))
        .unwrap_or(Ok(ShouldAttempt::Yes));
    match should_attempt {
        // Yes, let's make a request
        Ok(ShouldAttempt::Yes) => debug!("retry strategy has OK'd initial request"),
        // No, this request shouldn't be sent
        Ok(ShouldAttempt::No) => {
            let err: BoxError = "the retry strategy indicates that an initial request shouldn't be made, but it didn't specify why".into();
            halt!([ctx] => OrchestratorError::other(err));
        }
        // No, we shouldn't make a request because...
        Err(err) => halt!([ctx] => OrchestratorError::other(err)),
        Ok(ShouldAttempt::YesAfterDelay(_)) => {
            unreachable!("Delaying the initial request is currently unsupported. If this feature is important to you, please file an issue in GitHub.")
        }
    }

    // Save a request checkpoint before we make the request. This will allow us to "rewind"
    // the request in the case of retry attempts.
    ctx.save_checkpoint();
    for i in 1usize.. {
        debug!("beginning attempt #{i}");
        // Break from the loop if we can't rewind the request's state. This will always succeed the
        // first time, but will fail on subsequent iterations if the request body wasn't retryable.
        if let RewindResult::Impossible = ctx.rewind(cfg) {
            debug!("request cannot be retried since the request body cannot be cloned");
            break;
        }
        // Track which attempt we're currently on.
        cfg.interceptor_state()
            .store_put::<RequestAttempts>(i.into());
        let attempt_timeout_config = cfg.maybe_timeout_config(TimeoutKind::OperationAttempt);
        let maybe_timeout = async {
            try_attempt(ctx, cfg, interceptors, stop_point).await;
            finally_attempt(ctx, cfg, interceptors).await;
            Result::<_, SdkError<Error, HttpResponse>>::Ok(())
        }
        .maybe_timeout_with_config(attempt_timeout_config)
        .await
        .map_err(|err| OrchestratorError::timeout(err.into_source().unwrap()));

        // We continue when encountering a timeout error. The retry classifier will decide what to do with it.
        continue_on_err!([ctx] => maybe_timeout);

        let retry_strategy = cfg.retry_strategy();

        // If we got a retry strategy from the bag, ask it what to do.
        // If no strategy was set, we won't retry.
        let should_attempt = match retry_strategy {
            Some(retry_strategy) => halt_on_err!(
                [ctx] => retry_strategy.should_attempt_retry(ctx, cfg).map_err(OrchestratorError::other)
            ),
            None => ShouldAttempt::No,
        };
        match should_attempt {
            // Yes, let's retry the request
            ShouldAttempt::Yes => continue,
            // No, this request shouldn't be retried
            ShouldAttempt::No => {
                debug!("this error is not retryable, exiting attempt loop");
                break;
            }
            ShouldAttempt::YesAfterDelay(delay) => {
                let sleep_impl = halt_on_err!([ctx] => cfg.sleep_impl().ok_or(OrchestratorError::other(
                    "the retry strategy requested a delay before sending the next request, but no 'async sleep' implementation was set"
                )));
                sleep_impl.sleep(delay).await;
                continue;
            }
        }
    }
}

#[instrument(skip_all)]
async fn try_attempt(
    ctx: &mut InterceptorContext,
    cfg: &mut ConfigBag,
    interceptors: &Interceptors,
    stop_point: StopPoint,
) {
    halt_on_err!([ctx] => interceptors.read_before_attempt(ctx, cfg));
    halt_on_err!([ctx] => orchestrate_endpoint(ctx, cfg).map_err(OrchestratorError::other));
    halt_on_err!([ctx] => interceptors.modify_before_signing(ctx, cfg));
    halt_on_err!([ctx] => interceptors.read_before_signing(ctx, cfg));

    halt_on_err!([ctx] => orchestrate_auth(ctx, cfg).await.map_err(OrchestratorError::other));

    halt_on_err!([ctx] => interceptors.read_after_signing(ctx, cfg));
    halt_on_err!([ctx] => interceptors.modify_before_transmit(ctx, cfg));
    halt_on_err!([ctx] => interceptors.read_before_transmit(ctx, cfg));

    // Return early if a stop point is set for before transmit
    if let StopPoint::BeforeTransmit = stop_point {
        return;
    }

    // The connection consumes the request but we need to keep a copy of it
    // within the interceptor context, so we clone it here.
    ctx.enter_transmit_phase();
    let call_result = halt_on_err!([ctx] => {
        let request = ctx.take_request().expect("set during serialization");
        cfg.connection().call(request).await.map_err(|err| {
            match err.downcast() {
                Ok(connector_error) => OrchestratorError::connector(*connector_error),
                Err(box_err) => OrchestratorError::other(box_err)
            }
        })
    });
    ctx.set_response(call_result);
    ctx.enter_before_deserialization_phase();

    halt_on_err!([ctx] => interceptors.read_after_transmit(ctx, cfg));
    halt_on_err!([ctx] => interceptors.modify_before_deserialization(ctx, cfg));
    halt_on_err!([ctx] => interceptors.read_before_deserialization(ctx, cfg));

    ctx.enter_deserialization_phase();
    let output_or_error = async {
        let response = ctx.response_mut().expect("set during transmit");
        let response_deserializer = cfg.response_deserializer();
        match response_deserializer.deserialize_streaming(response) {
            Some(output_or_error) => output_or_error,
            None => read_body(response)
                .instrument(debug_span!("read_body"))
                .await
                .map_err(OrchestratorError::response)
                .and_then(|_| response_deserializer.deserialize_nonstreaming(response)),
        }
    }
    .await;
    ctx.set_output_or_error(output_or_error);

    ctx.enter_after_deserialization_phase();
    halt_on_err!([ctx] => interceptors.read_after_deserialization(ctx, cfg));
}

#[instrument(skip_all)]
async fn finally_attempt(
    ctx: &mut InterceptorContext,
    cfg: &mut ConfigBag,
    interceptors: &Interceptors,
) {
    continue_on_err!([ctx] => interceptors.modify_before_attempt_completion(ctx, cfg));
    continue_on_err!([ctx] => interceptors.read_after_attempt(ctx, cfg));
}

#[instrument(skip_all)]
async fn finally_op(
    ctx: &mut InterceptorContext,
    cfg: &mut ConfigBag,
    interceptors: &Interceptors,
) {
    continue_on_err!([ctx] => interceptors.modify_before_completion(ctx, cfg));
    continue_on_err!([ctx] => interceptors.read_after_execution(ctx, cfg));
}

#[cfg(all(test, feature = "test-util", feature = "anonymous-auth"))]
mod tests {
    use super::*;
    use crate::client::auth::no_auth::NoAuthRuntimePlugin;
    use crate::client::orchestrator::endpoints::{
        StaticUriEndpointResolver, StaticUriEndpointResolverParams,
    };
    use crate::client::retries::strategy::NeverRetryStrategy;
    use crate::client::test_util::{
        connector::OkConnector, deserializer::CannedResponseDeserializer,
        serializer::CannedRequestSerializer,
    };
    use ::http::{Request, Response, StatusCode};
    use aws_smithy_runtime_api::client::interceptors::context::{
        AfterDeserializationInterceptorContextRef, BeforeDeserializationInterceptorContextMut,
        BeforeDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextMut,
        BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextMut,
        BeforeTransmitInterceptorContextRef, FinalizerInterceptorContextMut,
        FinalizerInterceptorContextRef,
    };
    use aws_smithy_runtime_api::client::interceptors::{
        Interceptor, InterceptorRegistrar, SharedInterceptor,
    };
    use aws_smithy_runtime_api::client::orchestrator::{
        DynConnection, DynEndpointResolver, DynResponseDeserializer, SharedRequestSerializer,
    };
    use aws_smithy_runtime_api::client::retries::DynRetryStrategy;
    use aws_smithy_runtime_api::client::runtime_plugin::{RuntimePlugin, RuntimePlugins};
    use aws_smithy_types::config_bag::{ConfigBag, FrozenLayer, Layer};
    use aws_smithy_types::type_erasure::{TypeErasedBox, TypedBox};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tracing_test::traced_test;

    fn new_request_serializer() -> CannedRequestSerializer {
        CannedRequestSerializer::success(
            Request::builder()
                .body(SdkBody::empty())
                .expect("request is valid"),
        )
    }

    fn new_response_deserializer() -> CannedResponseDeserializer {
        CannedResponseDeserializer::new(
            Response::builder()
                .status(StatusCode::OK)
                .body(SdkBody::empty())
                .map_err(|err| OrchestratorError::other(Box::new(err)))
                .map(|res| Output::new(Box::new(res))),
        )
    }

    #[derive(Debug)]
    struct TestOperationRuntimePlugin;

    impl RuntimePlugin for TestOperationRuntimePlugin {
        fn config(&self) -> Option<FrozenLayer> {
            let mut cfg = Layer::new("test operation");
            cfg.set_request_serializer(SharedRequestSerializer::new(new_request_serializer()));
            cfg.set_response_deserializer(
                DynResponseDeserializer::new(new_response_deserializer()),
            );
            cfg.set_retry_strategy(DynRetryStrategy::new(NeverRetryStrategy::new()));
            cfg.set_endpoint_resolver(DynEndpointResolver::new(
                StaticUriEndpointResolver::http_localhost(8080),
            ));
            cfg.set_endpoint_resolver_params(StaticUriEndpointResolverParams::new().into());
            cfg.set_connection(DynConnection::new(OkConnector::new()));

            Some(cfg.freeze())
        }
    }

    macro_rules! interceptor_error_handling_test {
        ($interceptor:ident, $ctx:ty, $expected:expr) => {
            #[derive(Debug)]
            struct FailingInterceptorA;
            impl Interceptor for FailingInterceptorA {
                fn $interceptor(&self, _ctx: $ctx, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
                    tracing::debug!("FailingInterceptorA called!");
                    Err("FailingInterceptorA".into())
                }
            }

            #[derive(Debug)]
            struct FailingInterceptorB;
            impl Interceptor for FailingInterceptorB {
                fn $interceptor(&self, _ctx: $ctx, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
                    tracing::debug!("FailingInterceptorB called!");
                    Err("FailingInterceptorB".into())
                }
            }

            #[derive(Debug)]
            struct FailingInterceptorC;
            impl Interceptor for FailingInterceptorC {
                fn $interceptor(&self, _ctx: $ctx, _cfg: &mut ConfigBag) -> Result<(), BoxError> {
                    tracing::debug!("FailingInterceptorC called!");
                    Err("FailingInterceptorC".into())
                }
            }

            #[derive(Debug)]
            struct FailingInterceptorsClientRuntimePlugin;

            impl RuntimePlugin for FailingInterceptorsClientRuntimePlugin {
                fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
                    interceptors.register(SharedInterceptor::new(FailingInterceptorA));
                }
            }

            #[derive(Debug)]
            struct FailingInterceptorsOperationRuntimePlugin;

            impl RuntimePlugin for FailingInterceptorsOperationRuntimePlugin {
                fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
                    interceptors.register(SharedInterceptor::new(FailingInterceptorB));
                    interceptors.register(SharedInterceptor::new(FailingInterceptorC));
                }
            }

            let input = TypeErasedBox::new(Box::new(()));
            let runtime_plugins = RuntimePlugins::new()
                .with_client_plugin(FailingInterceptorsClientRuntimePlugin)
                .with_operation_plugin(TestOperationRuntimePlugin)
                .with_operation_plugin(NoAuthRuntimePlugin::new())
                .with_operation_plugin(FailingInterceptorsOperationRuntimePlugin);
            let actual = invoke(input, &runtime_plugins)
                .await
                .expect_err("should error");
            let actual = format!("{:?}", actual);
            assert_eq!($expected, format!("{:?}", actual));

            assert!(logs_contain("FailingInterceptorA called!"));
            assert!(logs_contain("FailingInterceptorB called!"));
            assert!(logs_contain("FailingInterceptorC called!"));
        };
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_execution_error_handling() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ReadBeforeExecution, source: Some(\"FailingInterceptorC\") } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_execution,
            &BeforeSerializationInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_serialization_error_handling() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ModifyBeforeSerialization, source: Some(\"FailingInterceptorC\") } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_serialization,
            &mut BeforeSerializationInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_serialization_error_handling() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ReadBeforeSerialization, source: Some(\"FailingInterceptorC\") } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_serialization,
            &BeforeSerializationInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_serialization_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ReadAfterSerialization, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_serialization,
            &BeforeTransmitInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_retry_loop_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeRetryLoop, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_retry_loop,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_attempt_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ReadBeforeAttempt, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_attempt,
            &BeforeTransmitInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_signing_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeSigning, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_signing,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_signing_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ReadBeforeSigning, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_signing,
            &BeforeTransmitInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_signing_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ReadAfterSigning, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_signing,
            &BeforeTransmitInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_transmit_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeTransmit, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_transmit,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_transmit_error_handling() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ReadBeforeTransmit, source: Some(\"FailingInterceptorC\") }, connection: Unknown } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_transmit,
            &BeforeTransmitInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_transmit_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterTransmit, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_transmit,
            &BeforeDeserializationInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_deserialization_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeDeserialization, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_deserialization,
            &mut BeforeDeserializationInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_deserialization_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadBeforeDeserialization, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            read_before_deserialization,
            &BeforeDeserializationInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_deserialization_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterDeserialization, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_deserialization,
            &AfterDeserializationInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_attempt_completion_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_attempt_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterAttempt, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_attempt,
            &FinalizerInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_completion_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_execution_error_handling() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterExecution, source: Some(\"FailingInterceptorC\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_handling_test!(
            read_after_execution,
            &FinalizerInterceptorContextRef<'_>,
            expected
        );
    }

    macro_rules! interceptor_error_redirection_test {
        ($origin_interceptor:ident, $origin_ctx:ty, $destination_interceptor:ident, $destination_ctx:ty, $expected:expr) => {
            #[derive(Debug)]
            struct OriginInterceptor;
            impl Interceptor for OriginInterceptor {
                fn $origin_interceptor(
                    &self,
                    _ctx: $origin_ctx,
                    _cfg: &mut ConfigBag,
                ) -> Result<(), BoxError> {
                    tracing::debug!("OriginInterceptor called!");
                    Err("OriginInterceptor".into())
                }
            }

            #[derive(Debug)]
            struct DestinationInterceptor;
            impl Interceptor for DestinationInterceptor {
                fn $destination_interceptor(
                    &self,
                    _ctx: $destination_ctx,
                    _cfg: &mut ConfigBag,
                ) -> Result<(), BoxError> {
                    tracing::debug!("DestinationInterceptor called!");
                    Err("DestinationInterceptor".into())
                }
            }

            #[derive(Debug)]
            struct InterceptorsTestOperationRuntimePlugin;

            impl RuntimePlugin for InterceptorsTestOperationRuntimePlugin {
                fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
                    interceptors.register(SharedInterceptor::new(OriginInterceptor));
                    interceptors.register(SharedInterceptor::new(DestinationInterceptor));
                }
            }

            let input = TypeErasedBox::new(Box::new(()));
            let runtime_plugins = RuntimePlugins::new()
                .with_operation_plugin(TestOperationRuntimePlugin)
                .with_operation_plugin(AnonymousAuthRuntimePlugin::new())
                .with_operation_plugin(InterceptorsTestOperationRuntimePlugin);
            let actual = invoke(input, &runtime_plugins)
                .await
                .expect_err("should error");
            let actual = format!("{:?}", actual);
            assert_eq!($expected, format!("{:?}", actual));

            assert!(logs_contain("OriginInterceptor called!"));
            assert!(logs_contain("DestinationInterceptor called!"));
        };
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_execution_error_causes_jump_to_modify_before_completion() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"DestinationInterceptor\") } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_execution,
            &BeforeSerializationInterceptorContextRef<'_>,
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_serialization_error_causes_jump_to_modify_before_completion() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"DestinationInterceptor\") } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_serialization,
            &mut BeforeSerializationInterceptorContextMut<'_>,
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_serialization_error_causes_jump_to_modify_before_completion() {
        let expected = r#""ConstructionFailure(ConstructionFailure { source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"DestinationInterceptor\") } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_serialization,
            &BeforeSerializationInterceptorContextRef<'_>,
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_serialization_error_causes_jump_to_modify_before_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            read_after_serialization,
            &BeforeTransmitInterceptorContextRef<'_>,
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_retry_loop_error_causes_jump_to_modify_before_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_retry_loop,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_attempt_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_attempt,
            &BeforeTransmitInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_signing_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_signing,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_signing_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_signing,
            &BeforeTransmitInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_signing_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            read_after_signing,
            &BeforeTransmitInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_transmit_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_transmit,
            &mut BeforeTransmitInterceptorContextMut<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_transmit_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""DispatchFailure(DispatchFailure { source: ConnectorError { kind: Other(None), source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, connection: Unknown } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_transmit,
            &BeforeTransmitInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_transmit_error_causes_jump_to_modify_before_attempt_completion() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            read_after_transmit,
            &BeforeDeserializationInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_deserialization_error_causes_jump_to_modify_before_attempt_completion(
    ) {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_deserialization,
            &mut BeforeDeserializationInterceptorContextMut<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_before_deserialization_error_causes_jump_to_modify_before_attempt_completion(
    ) {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(None), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            read_before_deserialization,
            &BeforeDeserializationInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_read_after_deserialization_error_causes_jump_to_modify_before_attempt_completion()
    {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ModifyBeforeAttemptCompletion, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            read_after_deserialization,
            &AfterDeserializationInterceptorContextRef<'_>,
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            expected
        );
    }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_attempt_completion_error_causes_jump_to_read_after_attempt() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterAttempt, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_attempt_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            read_after_attempt,
            &FinalizerInterceptorContextRef<'_>,
            expected
        );
    }

    // #[tokio::test]
    // #[traced_test]
    // async fn test_read_after_attempt_error_causes_jump_to_modify_before_attempt_completion() {
    //     todo!("I'm confused by the behavior described in the spec")
    // }

    #[tokio::test]
    #[traced_test]
    async fn test_modify_before_completion_error_causes_jump_to_read_after_execution() {
        let expected = r#""ResponseError(ResponseError { source: InterceptorError { kind: ReadAfterExecution, source: Some(\"DestinationInterceptor\") }, raw: Response { status: 200, version: HTTP/1.1, headers: {}, body: SdkBody { inner: Once(Some(b\"\")), retryable: true } } })""#.to_string();
        interceptor_error_redirection_test!(
            modify_before_completion,
            &mut FinalizerInterceptorContextMut<'_>,
            read_after_execution,
            &FinalizerInterceptorContextRef<'_>,
            expected
        );
    }

    #[tokio::test]
    async fn test_stop_points() {
        let runtime_plugins = || {
            RuntimePlugins::new()
                .with_operation_plugin(TestOperationRuntimePlugin)
                .with_operation_plugin(AnonymousAuthRuntimePlugin::new())
        };

        // StopPoint::None should result in a response getting set since orchestration doesn't stop
        let context = invoke_with_stop_point(
            TypedBox::new(()).erase(),
            &runtime_plugins(),
            StopPoint::None,
        )
        .await
        .expect("success");
        assert!(context.response().is_some());

        // StopPoint::BeforeTransmit will exit right before sending the request, so there should be no response
        let context = invoke_with_stop_point(
            TypedBox::new(()).erase(),
            &runtime_plugins(),
            StopPoint::BeforeTransmit,
        )
        .await
        .expect("success");
        assert!(context.response().is_none());
    }

    /// The "finally" interceptors should run upon error when the StopPoint is set to BeforeTransmit
    #[tokio::test]
    async fn test_stop_points_error_handling() {
        #[derive(Debug, Default)]
        struct Inner {
            modify_before_retry_loop_called: AtomicBool,
            modify_before_completion_called: AtomicBool,
            read_after_execution_called: AtomicBool,
        }
        #[derive(Clone, Debug, Default)]
        struct TestInterceptor {
            inner: Arc<Inner>,
        }

        impl Interceptor for TestInterceptor {
            fn modify_before_retry_loop(
                &self,
                _context: &mut BeforeTransmitInterceptorContextMut<'_>,
                _cfg: &mut ConfigBag,
            ) -> Result<(), BoxError> {
                self.inner
                    .modify_before_retry_loop_called
                    .store(true, Ordering::Relaxed);
                Err("test error".into())
            }

            fn modify_before_completion(
                &self,
                _context: &mut FinalizerInterceptorContextMut<'_>,
                _cfg: &mut ConfigBag,
            ) -> Result<(), BoxError> {
                self.inner
                    .modify_before_completion_called
                    .store(true, Ordering::Relaxed);
                Ok(())
            }

            fn read_after_execution(
                &self,
                _context: &FinalizerInterceptorContextRef<'_>,
                _cfg: &mut ConfigBag,
            ) -> Result<(), BoxError> {
                self.inner
                    .read_after_execution_called
                    .store(true, Ordering::Relaxed);
                Ok(())
            }
        }

        #[derive(Debug)]
        struct TestInterceptorRuntimePlugin {
            interceptor: TestInterceptor,
        }
        impl RuntimePlugin for TestInterceptorRuntimePlugin {
            fn interceptors(&self, interceptors: &mut InterceptorRegistrar) {
                interceptors.register(SharedInterceptor::new(self.interceptor.clone()));
            }
        }

        let interceptor = TestInterceptor::default();
        let runtime_plugins = || {
            RuntimePlugins::new()
                .with_operation_plugin(TestOperationRuntimePlugin)
                .with_operation_plugin(AnonymousAuthRuntimePlugin::new())
                .with_operation_plugin(TestInterceptorRuntimePlugin {
                    interceptor: interceptor.clone(),
                })
        };

        // StopPoint::BeforeTransmit will exit right before sending the request, so there should be no response
        let context = invoke_with_stop_point(
            TypedBox::new(()).erase(),
            &runtime_plugins(),
            StopPoint::BeforeTransmit,
        )
        .await
        .expect("success");
        assert!(context.response().is_none());

        assert!(interceptor
            .inner
            .modify_before_retry_loop_called
            .load(Ordering::Relaxed));
        assert!(interceptor
            .inner
            .modify_before_completion_called
            .load(Ordering::Relaxed));
        assert!(interceptor
            .inner
            .read_after_execution_called
            .load(Ordering::Relaxed));
    }
}
