/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod config_bag;

pub use crate::config_bag::ConfigBag;
use aws_smithy_http::body::SdkBody;
use aws_smithy_runtime_api::interceptors::{InterceptorContext, Interceptors};
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
    // 1
    // // TODO(runtime-plugins) initialize runtime plugins (see section 3.11 "Runtime Plugins and Their Configuration" of SRA)
    // let cfg = cfg.clone().apply_plugins();

    interceptors.read_before_execution(&ctx)?;
    interceptors.modify_before_serialization(&mut ctx)?;
    interceptors.read_before_serialization(&ctx)?;

    let request_serializer = cfg
        .get::<Box<dyn RequestSerializer<In, Req>>>()
        .ok_or("missing serializer")?;
    let req = request_serializer.serialize_request(ctx.modeled_request_mut(), cfg)?;
    ctx.set_tx_request(req);

    interceptors.read_after_serialization(&ctx)?;
    interceptors.modify_before_retry_loop(&mut ctx)?;

    loop {
        make_an_attempt(&mut ctx, cfg, interceptors).await?;
        interceptors.read_after_attempt(&ctx)?;
        interceptors.modify_before_attempt_completion(&mut ctx)?;

        let retry_strategy = cfg
            .get::<Box<dyn RetryStrategy<Result<T, BoxError>>>>()
            .ok_or("missing retry strategy")?;
        let mod_res = ctx
            .modeled_response()
            .expect("it's set during 'make_an_attempt'");
        if retry_strategy.should_retry(mod_res, cfg)? {
            continue;
        }

        interceptors.modify_before_completion(&mut ctx)?;
        let trace_probe = cfg
            .get::<Box<dyn TraceProbe>>()
            .ok_or("missing trace probes")?;
        trace_probe.dispatch_events(cfg);
        interceptors.read_after_execution(&ctx)?;

        break;
    }

    let (modeled_response, _) = ctx.into_responses()?;
    modeled_response
}

pub fn try_clone_http_request(req: &http::Request<SdkBody>) -> Option<http::Request<SdkBody>> {
    let cloned_body = req.body().try_clone()?;
    let mut cloned_request = http::Request::builder()
        .uri(req.uri().clone())
        .method(req.method());
    *cloned_request
        .headers_mut()
        .expect("builder has not been modified, headers must be valid") = req.headers().clone();
    let req = cloned_request
        .body(cloned_body)
        .expect("a clone of a valid request should be a valid request");

    Some(req)
}

pub fn try_clone_http_response(res: &http::Response<SdkBody>) -> Option<http::Response<SdkBody>> {
    let cloned_body = res.body().try_clone()?;
    let mut cloned_response = http::Response::builder()
        .version(res.version())
        .status(res.status());
    *cloned_response
        .headers_mut()
        .expect("builder has not been modified, headers must be valid") = res.headers().clone();
    let res = cloned_response
        .body(cloned_body)
        .expect("a clone of a valid response should be a valid request");

    Some(res)
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
    interceptors.read_before_attempt(ctx)?;

    let tx_req_mut = ctx.tx_request_mut().expect("tx_request has been set");
    let endpoint_orchestrator = cfg
        .get::<Box<dyn EndpointOrchestrator<Req>>>()
        .ok_or("missing endpoint orchestrator")?;
    endpoint_orchestrator.resolve_and_apply_endpoint(tx_req_mut, cfg)?;

    interceptors.modify_before_signing(ctx)?;
    interceptors.read_before_signing(ctx)?;

    let tx_req_mut = ctx.tx_request_mut().expect("tx_request has been set");
    let auth_orchestrator = cfg
        .get::<Box<dyn AuthOrchestrator<Req>>>()
        .ok_or("missing auth orchestrator")?;
    auth_orchestrator.auth_request(tx_req_mut, cfg)?;

    interceptors.read_after_signing(ctx)?;
    interceptors.modify_before_transmit(ctx)?;
    interceptors.read_before_transmit(ctx)?;

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

    interceptors.read_after_transmit(ctx)?;
    interceptors.modify_before_deserialization(ctx)?;
    interceptors.read_before_deserialization(ctx)?;
    let tx_res = ctx.tx_response_mut().expect("tx_response has been set");
    let response_deserializer = cfg
        .get::<Box<dyn ResponseDeserializer<Res, Result<T, BoxError>>>>()
        .ok_or("missing response deserializer")?;
    let res = response_deserializer.deserialize_response(tx_res, cfg)?;
    ctx.set_modeled_response(res);

    interceptors.read_after_deserialization(ctx)?;

    Ok(())
}
