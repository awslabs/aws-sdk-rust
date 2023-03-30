/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

pub mod context;
pub mod error;

use crate::config_bag::ConfigBag;
pub use context::InterceptorContext;
pub use error::InterceptorError;

/// An interceptor allows injecting code into the SDK ’s request execution pipeline.
///
/// ## Terminology:
/// - An execution is one end-to-end invocation against an SDK client.
/// - An attempt is an attempt at performing an execution. By default executions are retried multiple
///   times based on the client ’s retry strategy.
/// - A hook is a single method on the interceptor, allowing injection of code into a specific part
///   of the SDK ’s request execution pipeline. Hooks are either "read" hooks, which make it possible
///   to read in-flight request or response messages, or "read/write" hooks, which make it possible
///   to modify in-flight request or output messages.
pub trait Interceptor<ModReq, TxReq, TxRes, ModRes> {
    /// A hook called at the start of an execution, before the SDK
    /// does anything else.
    ///
    /// **When:** This will **ALWAYS** be called once per execution. The duration
    /// between invocation of this hook and `after_execution` is very close
    /// to full duration of the execution.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()] is
    /// **ALWAYS** available. Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `before_execution` invoked.
    /// Other hooks will then be skipped and execution will jump to
    /// `modify_before_completion` with the raised error as the
    /// [InterceptorContext::modeled_response()]. If multiple
    /// `before_execution` methods raise errors, the latest
    /// will be used and earlier ones will be logged and dropped.
    fn read_before_execution(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the input message is marshalled into a
    /// transport message.
    /// This method has the ability to modify and return a new
    /// request message of the same type.
    ///
    /// **When:** This will **ALWAYS** be called once per execution, except when a
    /// failure occurs earlier in the request pipeline.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()] is
    /// **ALWAYS** available. This request may have been modified by earlier
    /// `modify_before_serialization` hooks, and may be modified further by
    /// later hooks. Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** If errors are raised by this hook,
    ///
    /// execution will jump to `modify_before_completion` with the raised
    /// error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** The input message returned by this hook
    /// MUST be the same type of input message passed into this hook.
    /// If not, an error will immediately be raised.
    fn modify_before_serialization(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the input message is marshalled
    /// into a transport
    /// message.
    ///
    /// **When:** This will **ALWAYS** be called once per execution, except when a
    /// failure occurs earlier in the request pipeline. The
    /// duration between invocation of this hook and `after_serialization` is
    /// very close to the amount of time spent marshalling the request.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()] is
    /// **ALWAYS** available. Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** If errors are raised by this hook,
    /// execution will jump to `modify_before_completion` with the raised
    /// error as the [InterceptorContext::modeled_response()].
    fn read_before_serialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called after the input message is marshalled into
    /// a transport message.
    ///
    /// **When:** This will **ALWAYS** be called once per execution, except when a
    /// failure occurs earlier in the request pipeline. The duration
    /// between invocation of this hook and `before_serialization` is very
    /// close to the amount of time spent marshalling the request.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** If errors are raised by this hook,
    /// execution will jump to `modify_before_completion` with the raised
    /// error as the [InterceptorContext::modeled_response()].
    fn read_after_serialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the retry loop is entered. This method
    /// has the ability to modify and return a new transport request
    /// message of the same type, except when a failure occurs earlier in the request pipeline.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** If errors are raised by this hook,
    /// execution will jump to `modify_before_completion` with the raised
    /// error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** The transport request message returned by this
    /// hook MUST be the same type of request message passed into this hook
    /// If not, an error will immediately be raised.
    fn modify_before_retry_loop(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before each attempt at sending the transmission
    /// request message to the service.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method will be
    /// called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `before_attempt` invoked.
    /// Other hooks will then be skipped and execution will jump to
    /// `modify_before_attempt_completion` with the raised error as the
    /// [InterceptorContext::modeled_response()]. If multiple
    /// `before_attempt` methods raise errors, the latest will be used
    /// and earlier ones will be logged and dropped.
    fn read_before_attempt(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the transport request message is signed.
    /// This method has the ability to modify and return a new transport
    /// request message of the same type.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// The `http::Request` may have been modified by earlier
    /// `modify_before_signing` hooks, and may be modified further by later
    /// hooks. Other information **WILL NOT** be available. In the event of
    /// retries, the `InterceptorContext` will not include changes made
    /// in previous attempts
    /// (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** The transport request message returned by this
    /// hook MUST be the same type of request message passed into this hook
    ///
    /// If not, an error will immediately be raised.
    fn modify_before_signing(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;

        Ok(())
    }

    /// A hook called before the transport request message is signed.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries. The duration between
    /// invocation of this hook and `after_signing` is very close to
    /// the amount of time spent signing the request.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    fn read_before_signing(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called after the transport request message is signed.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries. The duration between
    /// invocation of this hook and `before_signing` is very close to
    /// the amount of time spent signing the request.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    fn read_after_signing(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the transport request message is sent to the
    /// service. This method has the ability to modify and return
    /// a new transport request message of the same type.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// The `http::Request` may have been modified by earlier
    /// `modify_before_transmit` hooks, and may be modified further by later
    /// hooks. Other information **WILL NOT** be available.
    /// In the event of retries, the `InterceptorContext` will not include
    /// changes made in previous attempts (e.g. by request signers or
    /// other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** The transport request message returned by this
    /// hook MUST be the same type of request message passed into this hook
    ///
    /// If not, an error will immediately be raised.
    fn modify_before_transmit(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the transport request message is sent to the
    /// service.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries. The duration between
    /// invocation of this hook and `after_transmit` is very close to
    /// the amount of time spent communicating with the service.
    /// Depending on the protocol, the duration may not include the
    /// time spent reading the response data.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::tx_request()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    fn read_before_transmit(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called after the transport request message is sent to the
    /// service and a transport response message is received.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries. The duration between
    /// invocation of this hook and `before_transmit` is very close to
    /// the amount of time spent communicating with the service.
    /// Depending on the protocol, the duration may not include the time
    /// spent reading the response data.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()] and
    /// [InterceptorContext::tx_response()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    fn read_after_transmit(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the transport response message is unmarshalled.
    /// This method has the ability to modify and return a new transport
    /// response message of the same type.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()] and
    /// [InterceptorContext::tx_response()] are **ALWAYS** available.
    /// The transmit_response may have been modified by earlier
    /// `modify_before_deserialization` hooks, and may be modified further by
    /// later hooks. Other information **WILL NOT** be available. In the event of
    /// retries, the `InterceptorContext` will not include changes made in
    /// previous attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the
    /// [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** The transport response message returned by this
    /// hook MUST be the same type of response message passed into
    /// this hook. If not, an error will immediately be raised.
    fn modify_before_deserialization(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called before the transport response message is unmarshalled
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. This method may be
    /// called multiple times in the event of retries. The duration between
    /// invocation of this hook and `after_deserialization` is very close
    /// to the amount of time spent unmarshalling the service response.
    /// Depending on the protocol and operation, the duration may include
    /// the time spent downloading the response data.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()] and
    /// [InterceptorContext::tx_response()] are **ALWAYS** available.
    /// Other information **WILL NOT** be available. In the event of retries,
    /// the `InterceptorContext` will not include changes made in previous
    /// attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion`
    /// with the raised error as the [InterceptorContext::modeled_response()].
    fn read_before_deserialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called after the transport response message is unmarshalled.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs earlier in the request pipeline. The duration
    /// between invocation of this hook and `before_deserialization` is
    /// very close to the amount of time spent unmarshalling the
    /// service response. Depending on the protocol and operation,
    /// the duration may include the time spent downloading
    /// the response data.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()],
    /// [InterceptorContext::tx_response()] and
    /// [InterceptorContext::modeled_response()] are **ALWAYS** available. In the event
    /// of retries, the `InterceptorContext` will not include changes made
    /// in previous attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `modify_before_attempt_completion` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    fn read_after_deserialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an attempt is completed. This method has the
    /// ability to modify and return a new output message or error
    /// matching the currently-executing operation.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs before `before_attempt`. This method may
    /// be called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()],
    /// [InterceptorContext::tx_response()] and
    /// [InterceptorContext::modeled_response()] are **ALWAYS** available. In the event
    /// of retries, the `InterceptorContext` will not include changes made
    /// in previous attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `after_attempt` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** Any output message returned by this
    /// hook MUST match the operation being invoked. Any error type can be
    /// returned, replacing the response currently in the context.
    fn modify_before_attempt_completion(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an attempt is completed.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, as long as
    /// `before_attempt` has been executed.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()],
    /// [InterceptorContext::tx_request()] and
    /// [InterceptorContext::modeled_response()] are **ALWAYS** available.
    /// The [InterceptorContext::tx_response()] is available if a
    /// response was received by the service for this attempt.
    /// In the event of retries, the `InterceptorContext` will not include
    /// changes made in previous attempts (e.g. by request signers or other
    /// interceptors).
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `after_attempt` invoked.
    /// If multiple `after_execution` methods raise errors, the latest
    /// will be used and earlier ones will be logged and dropped. If the
    /// retry strategy determines that the execution is retryable,
    /// execution will then jump to `before_attempt`. Otherwise,
    /// execution will jump to `modify_before_attempt_completion` with the
    /// raised error as the [InterceptorContext::modeled_response()].
    fn read_after_attempt(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an execution is completed.
    /// This method has the ability to modify and return a new
    /// output message or error matching the currently - executing
    /// operation.
    ///
    /// **When:** This will **ALWAYS** be called once per execution.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::modeled_response()] are **ALWAYS** available. The
    /// [InterceptorContext::tx_request()]
    /// and [InterceptorContext::tx_response()] are available if the
    /// execution proceeded far enough for them to be generated.
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook , execution will jump to `after_attempt` with
    /// the raised error as the [InterceptorContext::modeled_response()].
    ///
    /// **Return Constraints:** Any output message returned by this
    /// hook MUST match the operation being invoked. Any error type can be
    /// returned , replacing the response currently in the context.
    fn modify_before_completion(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an execution is completed.
    ///
    /// **When:** This will **ALWAYS** be called once per execution. The duration
    /// between invocation of this hook and `before_execution` is very
    /// close to the full duration of the execution.
    ///
    /// **Available Information:** The [InterceptorContext::modeled_request()]
    /// and [InterceptorContext::modeled_response()] are **ALWAYS** available. The
    /// [InterceptorContext::tx_request()] and
    /// [InterceptorContext::tx_response()] are available if the
    /// execution proceeded far enough for them to be generated.
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `after_execution` invoked.
    /// The error will then be treated as the
    /// [InterceptorContext::modeled_response()] to the customer. If multiple
    /// `after_execution` methods raise errors , the latest will be
    /// used and earlier ones will be logged and dropped.
    fn read_after_execution(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }
}

pub struct Interceptors<ModReq, TxReq, TxRes, ModRes> {
    client_interceptors: Vec<Box<dyn Interceptor<ModReq, TxReq, TxRes, ModRes>>>,
    operation_interceptors: Vec<Box<dyn Interceptor<ModReq, TxReq, TxRes, ModRes>>>,
}

impl<ModReq, TxReq, TxRes, ModRes> Default for Interceptors<ModReq, TxReq, TxRes, ModRes> {
    fn default() -> Self {
        Self {
            client_interceptors: Vec::new(),
            operation_interceptors: Vec::new(),
        }
    }
}

impl<ModReq, TxReq, TxRes, ModRes> Interceptors<ModReq, TxReq, TxRes, ModRes> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_client_interceptor(
        &mut self,
        interceptor: impl Interceptor<ModReq, TxReq, TxRes, ModRes> + 'static,
    ) -> &mut Self {
        self.client_interceptors.push(Box::new(interceptor));
        self
    }

    pub fn with_operation_interceptor(
        &mut self,
        interceptor: impl Interceptor<ModReq, TxReq, TxRes, ModRes> + 'static,
    ) -> &mut Self {
        self.operation_interceptors.push(Box::new(interceptor));
        self
    }

    fn all_interceptors_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Box<dyn Interceptor<ModReq, TxReq, TxRes, ModRes>>> {
        self.client_interceptors
            .iter_mut()
            .chain(self.operation_interceptors.iter_mut())
    }

    pub fn client_read_before_execution(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.client_interceptors.iter_mut() {
            interceptor.read_before_execution(context, cfg)?;
        }
        Ok(())
    }

    pub fn operation_read_before_execution(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.operation_interceptors.iter_mut() {
            interceptor.read_before_execution(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_serialization(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_serialization(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_before_serialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_before_serialization(context, cfg)?;
        }
        Ok(())
    }

    pub fn read_after_serialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_serialization(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_retry_loop(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_retry_loop(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_before_attempt(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_before_attempt(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_signing(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_signing(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_before_signing(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_before_signing(context, cfg)?;
        }
        Ok(())
    }

    pub fn read_after_signing(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_signing(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_transmit(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_transmit(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_before_transmit(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_before_transmit(context, cfg)?;
        }
        Ok(())
    }

    pub fn read_after_transmit(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_transmit(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_deserialization(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_deserialization(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_before_deserialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_before_deserialization(context, cfg)?;
        }
        Ok(())
    }

    pub fn read_after_deserialization(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_deserialization(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_attempt_completion(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_attempt_completion(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_after_attempt(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_attempt(context, cfg)?;
        }
        Ok(())
    }

    pub fn modify_before_completion(
        &mut self,
        context: &mut InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.modify_before_completion(context, cfg)?;
        }

        Ok(())
    }

    pub fn read_after_execution(
        &mut self,
        context: &InterceptorContext<ModReq, TxReq, TxRes, ModRes>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        for interceptor in self.all_interceptors_mut() {
            interceptor.read_after_execution(context, cfg)?;
        }
        Ok(())
    }
}
