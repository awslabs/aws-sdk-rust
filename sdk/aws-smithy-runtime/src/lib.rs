/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

#![warn(
    // missing_docs,
    // rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

use aws_smithy_runtime_api::config_bag::ConfigBag;
use aws_smithy_runtime_api::interceptors::{InterceptorContext, Interceptors};
use aws_smithy_runtime_api::runtime_plugin::RuntimePlugins;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxFallibleFut<T> = Pin<Box<dyn Future<Output = Result<T, BoxError>>>>;

pub trait TraceProbe: Send + Sync + Debug {
    fn dispatch_events(&self, cfg: &ConfigBag) -> BoxFallibleFut<()>;
}

pub trait RequestSerializer<In, TxReq>: Send + Sync + Debug {
    fn serialize_request(&self, req: &mut In, cfg: &ConfigBag) -> Result<TxReq, BoxError>;
}

pub trait ResponseDeserializer<TxRes, Out>: Send + Sync + Debug {
    fn deserialize_response(&self, res: &mut TxRes, cfg: &ConfigBag) -> Result<Out, BoxError>;
}

pub trait Connection<TxReq, TxRes>: Send + Sync + Debug {
    fn call(&self, req: &mut TxReq, cfg: &ConfigBag) -> BoxFallibleFut<TxRes>;
}

pub trait RetryStrategy<Out>: Send + Sync + Debug {
    fn should_retry(&self, res: &Out, cfg: &ConfigBag) -> Result<bool, BoxError>;
}

pub trait AuthOrchestrator<Req>: Send + Sync + Debug {
    fn auth_request(&self, req: &mut Req, cfg: &ConfigBag) -> Result<(), BoxError>;
}

pub trait EndpointOrchestrator<Req>: Send + Sync + Debug {
    fn resolve_and_apply_endpoint(&self, req: &mut Req, cfg: &ConfigBag) -> Result<(), BoxError>;
    // TODO(jdisanti) The EP Orc and Auth Orc need to share info on auth schemes but I'm not sure how that should happen
    fn resolve_auth_schemes(&self) -> Result<Vec<String>, BoxError>;
}

/// `In`: The input message e.g. `ListObjectsRequest`
/// `Req`: The transport request message e.g. `http::Request<SmithyBody>`
/// `Res`: The transport response message e.g. `http::Response<SmithyBody>`
/// `Out`: The output message. A `Result` containing either:
///     - The 'success' output message e.g. `ListObjectsResponse`
///     - The 'failure' output message e.g. `NoSuchBucketException`
pub async fn invoke<In, Req, Res, T>(
    input: In,
    interceptors: &mut Interceptors<In, Req, Res, Result<T, BoxError>>,
    runtime_plugins: &RuntimePlugins,
    cfg: &mut ConfigBag,
) -> Result<T, BoxError>
where
    // The input must be Clone in case of retries
    In: Clone + 'static,
    Req: 'static,
    Res: 'static,
    T: 'static,
{
    let mut ctx: InterceptorContext<In, Req, Res, Result<T, BoxError>> =
        InterceptorContext::new(input);

    runtime_plugins.apply_client_configuration(cfg)?;
    interceptors.client_read_before_execution(&ctx, cfg)?;

    runtime_plugins.apply_operation_configuration(cfg)?;
    interceptors.operation_read_before_execution(&ctx, cfg)?;

    interceptors.read_before_serialization(&ctx, cfg)?;
    interceptors.modify_before_serialization(&mut ctx, cfg)?;

    let request_serializer = cfg
        .get::<Box<dyn RequestSerializer<In, Req>>>()
        .ok_or("missing serializer")?;
    let req = request_serializer.serialize_request(ctx.modeled_request_mut(), cfg)?;
    ctx.set_tx_request(req);

    interceptors.read_after_serialization(&ctx, cfg)?;
    interceptors.modify_before_retry_loop(&mut ctx, cfg)?;

    loop {
        make_an_attempt(&mut ctx, cfg, interceptors).await?;
        interceptors.read_after_attempt(&ctx, cfg)?;
        interceptors.modify_before_attempt_completion(&mut ctx, cfg)?;

        let retry_strategy = cfg
            .get::<Box<dyn RetryStrategy<Result<T, BoxError>>>>()
            .ok_or("missing retry strategy")?;
        let mod_res = ctx
            .modeled_response()
            .expect("it's set during 'make_an_attempt'");
        if retry_strategy.should_retry(mod_res, cfg)? {
            continue;
        }

        interceptors.modify_before_completion(&mut ctx, cfg)?;
        let trace_probe = cfg
            .get::<Box<dyn TraceProbe>>()
            .ok_or("missing trace probes")?;
        trace_probe.dispatch_events(cfg);
        interceptors.read_after_execution(&ctx, cfg)?;

        break;
    }

    let (modeled_response, _) = ctx.into_responses()?;
    modeled_response
}

// Making an HTTP request can fail for several reasons, but we still need to
// call lifecycle events when that happens. Therefore, we define this
// `make_an_attempt` function to make error handling simpler.
async fn make_an_attempt<In, Req, Res, T>(
    ctx: &mut InterceptorContext<In, Req, Res, Result<T, BoxError>>,
    cfg: &mut ConfigBag,
    interceptors: &mut Interceptors<In, Req, Res, Result<T, BoxError>>,
) -> Result<(), BoxError>
where
    In: Clone + 'static,
    Req: 'static,
    Res: 'static,
    T: 'static,
{
    interceptors.read_before_attempt(ctx, cfg)?;

    let tx_req_mut = ctx.tx_request_mut().expect("tx_request has been set");
    let endpoint_orchestrator = cfg
        .get::<Box<dyn EndpointOrchestrator<Req>>>()
        .ok_or("missing endpoint orchestrator")?;
    endpoint_orchestrator.resolve_and_apply_endpoint(tx_req_mut, cfg)?;

    interceptors.modify_before_signing(ctx, cfg)?;
    interceptors.read_before_signing(ctx, cfg)?;

    let tx_req_mut = ctx.tx_request_mut().expect("tx_request has been set");
    let auth_orchestrator = cfg
        .get::<Box<dyn AuthOrchestrator<Req>>>()
        .ok_or("missing auth orchestrator")?;
    auth_orchestrator.auth_request(tx_req_mut, cfg)?;

    interceptors.read_after_signing(ctx, cfg)?;
    interceptors.modify_before_transmit(ctx, cfg)?;
    interceptors.read_before_transmit(ctx, cfg)?;

    // The connection consumes the request but we need to keep a copy of it
    // within the interceptor context, so we clone it here.
    let res = {
        let tx_req = ctx.tx_request_mut().expect("tx_request has been set");
        let connection = cfg
            .get::<Box<dyn Connection<Req, Res>>>()
            .ok_or("missing connector")?;
        connection.call(tx_req, cfg).await?
    };
    ctx.set_tx_response(res);

    interceptors.read_after_transmit(ctx, cfg)?;
    interceptors.modify_before_deserialization(ctx, cfg)?;
    interceptors.read_before_deserialization(ctx, cfg)?;
    let tx_res = ctx.tx_response_mut().expect("tx_response has been set");
    let response_deserializer = cfg
        .get::<Box<dyn ResponseDeserializer<Res, Result<T, BoxError>>>>()
        .ok_or("missing response deserializer")?;
    let res = response_deserializer.deserialize_response(tx_res, cfg)?;
    ctx.set_modeled_response(res);

    interceptors.read_after_deserialization(ctx, cfg)?;

    Ok(())
}
