/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::box_error::BoxError;
use crate::client::interceptors::context::{
    AfterDeserializationInterceptorContextRef, BeforeDeserializationInterceptorContextMut,
    BeforeDeserializationInterceptorContextRef, BeforeSerializationInterceptorContextMut,
    BeforeSerializationInterceptorContextRef, BeforeTransmitInterceptorContextMut,
    BeforeTransmitInterceptorContextRef, FinalizerInterceptorContextMut,
    FinalizerInterceptorContextRef, InterceptorContext,
};
use crate::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::error::display::DisplayErrorContext;
use context::{Error, Input, Output};
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

pub mod context;
pub mod error;

pub use error::InterceptorError;

macro_rules! interceptor_trait_fn {
    ($name:ident, $phase:ident, $docs:tt) => {
        #[doc = $docs]
        fn $name(
            &self,
            context: &$phase<'_>,
            runtime_components: &RuntimeComponents,
            cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            let _ctx = context;
            let _rc = runtime_components;
            let _cfg = cfg;
            Ok(())
        }
    };
    (mut $name:ident, $phase:ident, $docs:tt) => {
        #[doc = $docs]
        fn $name(
            &self,
            context: &mut $phase<'_>,
            runtime_components: &RuntimeComponents,
            cfg: &mut ConfigBag,
        ) -> Result<(), BoxError> {
            let _ctx = context;
            let _rc = runtime_components;
            let _cfg = cfg;
            Ok(())
        }
    };
}

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
pub trait Interceptor: fmt::Debug {
    /// A hook called at the start of an execution, before the SDK
    /// does anything else.
    ///
    /// **When:** This will **ALWAYS** be called once per execution. The duration
    /// between invocation of this hook and `after_execution` is very close
    /// to full duration of the execution.
    ///
    /// **Available Information:** The [InterceptorContext::input()] is
    /// **ALWAYS** available. Other information **WILL NOT** be available.
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `before_execution` invoked.
    /// Other hooks will then be skipped and execution will jump to
    /// `modify_before_completion` with the raised error as the
    /// [InterceptorContext::output_or_error()]. If multiple
    /// `before_execution` methods raise errors, the latest
    /// will be used and earlier ones will be logged and dropped.
    fn read_before_execution(
        &self,
        context: &BeforeSerializationInterceptorContextRef<'_>,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let _ctx = context;
        let _cfg = cfg;
        Ok(())
    }

    interceptor_trait_fn!(
        mut modify_before_serialization,
        BeforeSerializationInterceptorContextMut,
        "
        A hook called before the input message is marshalled into a
        transport message.
        This method has the ability to modify and return a new
        request message of the same type.

        **When:** This will **ALWAYS** be called once per execution, except when a
        failure occurs earlier in the request pipeline.

        **Available Information:** The [InterceptorContext::input()] is
        **ALWAYS** available. This request may have been modified by earlier
        `modify_before_serialization` hooks, and may be modified further by
        later hooks. Other information **WILL NOT** be available.

        **Error Behavior:** If errors are raised by this hook,
        execution will jump to `modify_before_completion` with the raised
        error as the [InterceptorContext::output_or_error()].

        **Return Constraints:** The input message returned by this hook
        MUST be the same type of input message passed into this hook.
        If not, an error will immediately be raised.
        "
    );

    interceptor_trait_fn!(
        read_before_serialization,
        BeforeSerializationInterceptorContextRef,
        "
        A hook called before the input message is marshalled
        into a transport
        message.

        **When:** This will **ALWAYS** be called once per execution, except when a
        failure occurs earlier in the request pipeline. The
        duration between invocation of this hook and `after_serialization` is
        very close to the amount of time spent marshalling the request.

        **Available Information:** The [InterceptorContext::input()] is
        **ALWAYS** available. Other information **WILL NOT** be available.

        **Error Behavior:** If errors are raised by this hook,
        execution will jump to `modify_before_completion` with the raised
        error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        read_after_serialization,
        BeforeTransmitInterceptorContextRef,
        "
        /// A hook called after the input message is marshalled into
        /// a transport message.
        ///
        /// **When:** This will **ALWAYS** be called once per execution, except when a
        /// failure occurs earlier in the request pipeline. The duration
        /// between invocation of this hook and `before_serialization` is very
        /// close to the amount of time spent marshalling the request.
        ///
        /// **Available Information:** The [InterceptorContext::input()]
        /// and [InterceptorContext::request()] are **ALWAYS** available.
        /// Other information **WILL NOT** be available.
        ///
        /// **Error Behavior:** If errors are raised by this hook,
        /// execution will jump to `modify_before_completion` with the raised
        /// error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        mut modify_before_retry_loop,
        BeforeTransmitInterceptorContextMut,
        "
        A hook called before the retry loop is entered. This method
        has the ability to modify and return a new transport request
        message of the same type, except when a failure occurs earlier in the request pipeline.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        Other information **WILL NOT** be available.

        **Error Behavior:** If errors are raised by this hook,
        execution will jump to `modify_before_completion` with the raised
        error as the [InterceptorContext::output_or_error()].

        **Return Constraints:** The transport request message returned by this
        hook MUST be the same type of request message passed into this hook
        If not, an error will immediately be raised.
        "
    );

    interceptor_trait_fn!(
        read_before_attempt,
        BeforeTransmitInterceptorContextRef,
        "
        A hook called before each attempt at sending the transmission
        request message to the service.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method will be
        called multiple times in the event of retries.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** Errors raised by this hook will be stored
        until all interceptors have had their `before_attempt` invoked.
        Other hooks will then be skipped and execution will jump to
        `modify_before_attempt_completion` with the raised error as the
        [InterceptorContext::output_or_error()]. If multiple
        `before_attempt` methods raise errors, the latest will be used
        and earlier ones will be logged and dropped.
        "
    );

    interceptor_trait_fn!(
        mut modify_before_signing,
        BeforeTransmitInterceptorContextMut,
        "
        A hook called before the transport request message is signed.
        This method has the ability to modify and return a new transport
        request message of the same type.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        The `http::Request` may have been modified by earlier
        `modify_before_signing` hooks, and may be modified further by later
        hooks. Other information **WILL NOT** be available. In the event of
        retries, the `InterceptorContext` will not include changes made
        in previous attempts
        (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].

        **Return Constraints:** The transport request message returned by this
        hook MUST be the same type of request message passed into this hook

        If not, an error will immediately be raised.
        "
    );

    interceptor_trait_fn!(
        read_before_signing,
        BeforeTransmitInterceptorContextRef,
        "
        A hook called before the transport request message is signed.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries. The duration between
        invocation of this hook and `after_signing` is very close to
        the amount of time spent signing the request.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        read_after_signing,
        BeforeTransmitInterceptorContextRef,
        "
        A hook called after the transport request message is signed.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries. The duration between
        invocation of this hook and `before_signing` is very close to
        the amount of time spent signing the request.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        mut modify_before_transmit,
        BeforeTransmitInterceptorContextMut,
        "
        /// A hook called before the transport request message is sent to the
        /// service. This method has the ability to modify and return
        /// a new transport request message of the same type.
        ///
        /// **When:** This will **ALWAYS** be called once per attempt, except when a
        /// failure occurs earlier in the request pipeline. This method may be
        /// called multiple times in the event of retries.
        ///
        /// **Available Information:** The [InterceptorContext::input()]
        /// and [InterceptorContext::request()] are **ALWAYS** available.
        /// The `http::Request` may have been modified by earlier
        /// `modify_before_transmit` hooks, and may be modified further by later
        /// hooks. Other information **WILL NOT** be available.
        /// In the event of retries, the `InterceptorContext` will not include
        /// changes made in previous attempts (e.g. by request signers or
        other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].

        **Return Constraints:** The transport request message returned by this
        hook MUST be the same type of request message passed into this hook

        If not, an error will immediately be raised.
        "
    );

    interceptor_trait_fn!(
        read_before_transmit,
        BeforeTransmitInterceptorContextRef,
        "
        A hook called before the transport request message is sent to the
        service.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries. The duration between
        invocation of this hook and `after_transmit` is very close to
        the amount of time spent communicating with the service.
        Depending on the protocol, the duration may not include the
        time spent reading the response data.

        **Available Information:** The [InterceptorContext::input()]
        and [InterceptorContext::request()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).


        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        read_after_transmit,
        BeforeDeserializationInterceptorContextRef,
        "
        A hook called after the transport request message is sent to the
        service and a transport response message is received.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries. The duration between
        invocation of this hook and `before_transmit` is very close to
        the amount of time spent communicating with the service.
        Depending on the protocol, the duration may not include the time
        spent reading the response data.

        **Available Information:** The [InterceptorContext::input()],
        [InterceptorContext::request()] and
        [InterceptorContext::response()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        mut modify_before_deserialization,
        BeforeDeserializationInterceptorContextMut,
        "
        A hook called before the transport response message is unmarshalled.
        This method has the ability to modify and return a new transport
        response message of the same type.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries.

        **Available Information:** The [InterceptorContext::input()],
        [InterceptorContext::request()] and
        [InterceptorContext::response()] are **ALWAYS** available.
        The transmit_response may have been modified by earlier
        `modify_before_deserialization` hooks, and may be modified further by
        later hooks. Other information **WILL NOT** be available. In the event of
        retries, the `InterceptorContext` will not include changes made in
        previous attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the
        [InterceptorContext::output_or_error()].

        **Return Constraints:** The transport response message returned by this
        hook MUST be the same type of response message passed into
        this hook. If not, an error will immediately be raised.
        "
    );

    interceptor_trait_fn!(
        read_before_deserialization,
        BeforeDeserializationInterceptorContextRef,
        "
        A hook called before the transport response message is unmarshalled

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. This method may be
        called multiple times in the event of retries. The duration between
        invocation of this hook and `after_deserialization` is very close
        to the amount of time spent unmarshalling the service response.
        Depending on the protocol and operation, the duration may include
        the time spent downloading the response data.

        **Available Information:** The [InterceptorContext::input()],
        [InterceptorContext::request()] and
        [InterceptorContext::response()] are **ALWAYS** available.
        Other information **WILL NOT** be available. In the event of retries,
        the `InterceptorContext` will not include changes made in previous
        attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion`
        with the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    interceptor_trait_fn!(
        read_after_deserialization,
        AfterDeserializationInterceptorContextRef,
        "
        A hook called after the transport response message is unmarshalled.

        **When:** This will **ALWAYS** be called once per attempt, except when a
        failure occurs earlier in the request pipeline. The duration
        between invocation of this hook and `before_deserialization` is
        very close to the amount of time spent unmarshalling the
        service response. Depending on the protocol and operation,
        the duration may include the time spent downloading
        the response data.

        **Available Information:** The [InterceptorContext::input()],
        [InterceptorContext::request()],
        [InterceptorContext::response()] and
        [InterceptorContext::output_or_error()] are **ALWAYS** available. In the event
        of retries, the `InterceptorContext` will not include changes made
        in previous attempts (e.g. by request signers or other interceptors).

        **Error Behavior:** If errors are raised by this
        hook, execution will jump to `modify_before_attempt_completion` with
        the raised error as the [InterceptorContext::output_or_error()].
        "
    );

    /// A hook called when an attempt is completed. This method has the
    /// ability to modify and return a new output message or error
    /// matching the currently-executing operation.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, except when a
    /// failure occurs before `before_attempt`. This method may
    /// be called multiple times in the event of retries.
    ///
    /// **Available Information:** The [InterceptorContext::input()],
    /// [InterceptorContext::request()],
    /// [InterceptorContext::response()] and
    /// [InterceptorContext::output_or_error()] are **ALWAYS** available. In the event
    /// of retries, the `InterceptorContext` will not include changes made
    /// in previous attempts (e.g. by request signers or other interceptors).
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook, execution will jump to `after_attempt` with
    /// the raised error as the [InterceptorContext::output_or_error()].
    ///
    /// **Return Constraints:** Any output message returned by this
    /// hook MUST match the operation being invoked. Any error type can be
    /// returned, replacing the response currently in the context.
    fn modify_before_attempt_completion(
        &self,
        context: &mut FinalizerInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let _ctx = context;
        let _rc = runtime_components;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an attempt is completed.
    ///
    /// **When:** This will **ALWAYS** be called once per attempt, as long as
    /// `before_attempt` has been executed.
    ///
    /// **Available Information:** The [InterceptorContext::input()],
    /// [InterceptorContext::request()] and
    /// [InterceptorContext::output_or_error()] are **ALWAYS** available.
    /// The [InterceptorContext::response()] is available if a
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
    /// raised error as the [InterceptorContext::output_or_error()].
    fn read_after_attempt(
        &self,
        context: &FinalizerInterceptorContextRef<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let _ctx = context;
        let _rc = runtime_components;
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
    /// **Available Information:** The [InterceptorContext::input()]
    /// and [InterceptorContext::output_or_error()] are **ALWAYS** available. The
    /// [InterceptorContext::request()]
    /// and [InterceptorContext::response()] are available if the
    /// execution proceeded far enough for them to be generated.
    ///
    /// **Error Behavior:** If errors are raised by this
    /// hook , execution will jump to `after_attempt` with
    /// the raised error as the [InterceptorContext::output_or_error()].
    ///
    /// **Return Constraints:** Any output message returned by this
    /// hook MUST match the operation being invoked. Any error type can be
    /// returned , replacing the response currently in the context.
    fn modify_before_completion(
        &self,
        context: &mut FinalizerInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let _ctx = context;
        let _rc = runtime_components;
        let _cfg = cfg;
        Ok(())
    }

    /// A hook called when an execution is completed.
    ///
    /// **When:** This will **ALWAYS** be called once per execution. The duration
    /// between invocation of this hook and `before_execution` is very
    /// close to the full duration of the execution.
    ///
    /// **Available Information:** The [InterceptorContext::input()]
    /// and [InterceptorContext::output_or_error()] are **ALWAYS** available. The
    /// [InterceptorContext::request()] and
    /// [InterceptorContext::response()] are available if the
    /// execution proceeded far enough for them to be generated.
    ///
    /// **Error Behavior:** Errors raised by this hook will be stored
    /// until all interceptors have had their `after_execution` invoked.
    /// The error will then be treated as the
    /// [InterceptorContext::output_or_error()] to the customer. If multiple
    /// `after_execution` methods raise errors , the latest will be
    /// used and earlier ones will be logged and dropped.
    fn read_after_execution(
        &self,
        context: &FinalizerInterceptorContextRef<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let _ctx = context;
        let _rc = runtime_components;
        let _cfg = cfg;
        Ok(())
    }
}

/// Interceptor wrapper that may be shared
#[derive(Clone)]
pub struct SharedInterceptor {
    interceptor: Arc<dyn Interceptor + Send + Sync>,
    check_enabled: Arc<dyn Fn(&ConfigBag) -> bool + Send + Sync>,
}

impl fmt::Debug for SharedInterceptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedInterceptor")
            .field("interceptor", &self.interceptor)
            .finish()
    }
}

impl SharedInterceptor {
    /// Create a new `SharedInterceptor` from `Interceptor`
    pub fn new<T: Interceptor + Send + Sync + 'static>(interceptor: T) -> Self {
        Self {
            interceptor: Arc::new(interceptor),
            check_enabled: Arc::new(|conf: &ConfigBag| {
                conf.load::<DisableInterceptor<T>>().is_none()
            }),
        }
    }

    fn enabled(&self, conf: &ConfigBag) -> bool {
        (self.check_enabled)(conf)
    }
}

impl AsRef<dyn Interceptor> for SharedInterceptor {
    fn as_ref(&self) -> &(dyn Interceptor + 'static) {
        self.interceptor.as_ref()
    }
}

impl Deref for SharedInterceptor {
    type Target = Arc<dyn Interceptor + Send + Sync>;
    fn deref(&self) -> &Self::Target {
        &self.interceptor
    }
}

/// A interceptor wrapper to conditionally enable the interceptor based on [`DisableInterceptor`]
struct ConditionallyEnabledInterceptor(SharedInterceptor);
impl ConditionallyEnabledInterceptor {
    fn if_enabled(&self, cfg: &ConfigBag) -> Option<&dyn Interceptor> {
        if self.0.enabled(cfg) {
            Some(self.0.as_ref())
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Interceptors<I> {
    interceptors: I,
}

macro_rules! interceptor_impl_fn {
    (mut $interceptor:ident) => {
        pub fn $interceptor(
            self,
            ctx: &mut InterceptorContext,
            runtime_components: &RuntimeComponents,
            cfg: &mut ConfigBag,
        ) -> Result<(), InterceptorError> {
            tracing::trace!(concat!(
                "running `",
                stringify!($interceptor),
                "` interceptors"
            ));
            let mut result: Result<(), BoxError> = Ok(());
            let mut ctx = ctx.into();
            for interceptor in self.into_iter() {
                if let Some(interceptor) = interceptor.if_enabled(cfg) {
                    if let Err(new_error) =
                        interceptor.$interceptor(&mut ctx, runtime_components, cfg)
                    {
                        if let Err(last_error) = result {
                            tracing::debug!("{}", DisplayErrorContext(&*last_error));
                        }
                        result = Err(new_error);
                    }
                }
            }
            result.map_err(InterceptorError::$interceptor)
        }
    };
    (ref $interceptor:ident) => {
        pub fn $interceptor(
            self,
            ctx: &InterceptorContext,
            runtime_components: &RuntimeComponents,
            cfg: &mut ConfigBag,
        ) -> Result<(), InterceptorError> {
            let mut result: Result<(), BoxError> = Ok(());
            let ctx = ctx.into();
            for interceptor in self.into_iter() {
                if let Some(interceptor) = interceptor.if_enabled(cfg) {
                    if let Err(new_error) = interceptor.$interceptor(&ctx, runtime_components, cfg)
                    {
                        if let Err(last_error) = result {
                            tracing::debug!("{}", DisplayErrorContext(&*last_error));
                        }
                        result = Err(new_error);
                    }
                }
            }
            result.map_err(InterceptorError::$interceptor)
        }
    };
}

/// Generalized interceptor disabling interface
///
/// RuntimePlugins can disable interceptors by inserting [`DisableInterceptor<T>`](DisableInterceptor) into the config bag
#[must_use]
#[derive(Debug)]
pub struct DisableInterceptor<T> {
    _t: PhantomData<T>,
    #[allow(unused)]
    cause: &'static str,
}

impl<T> Storable for DisableInterceptor<T>
where
    T: fmt::Debug + Send + Sync + 'static,
{
    type Storer = StoreReplace<Self>;
}

/// Disable an interceptor with a given cause
pub fn disable_interceptor<T: Interceptor>(cause: &'static str) -> DisableInterceptor<T> {
    DisableInterceptor {
        _t: PhantomData::default(),
        cause,
    }
}

impl<I> Interceptors<I>
where
    I: Iterator<Item = SharedInterceptor>,
{
    pub fn new(interceptors: I) -> Self {
        Self { interceptors }
    }

    fn into_iter(self) -> impl Iterator<Item = ConditionallyEnabledInterceptor> {
        self.interceptors.map(ConditionallyEnabledInterceptor)
    }

    pub fn read_before_execution(
        self,
        operation: bool,
        ctx: &InterceptorContext<Input, Output, Error>,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        tracing::trace!(
            "running {} `read_before_execution` interceptors",
            if operation { "operation" } else { "client" }
        );
        let mut result: Result<(), BoxError> = Ok(());
        let ctx: BeforeSerializationInterceptorContextRef<'_> = ctx.into();
        for interceptor in self.into_iter() {
            if let Some(interceptor) = interceptor.if_enabled(cfg) {
                if let Err(new_error) = interceptor.read_before_execution(&ctx, cfg) {
                    if let Err(last_error) = result {
                        tracing::debug!("{}", DisplayErrorContext(&*last_error));
                    }
                    result = Err(new_error);
                }
            }
        }
        result.map_err(InterceptorError::read_before_execution)
    }

    interceptor_impl_fn!(mut modify_before_serialization);
    interceptor_impl_fn!(ref read_before_serialization);
    interceptor_impl_fn!(ref read_after_serialization);
    interceptor_impl_fn!(mut modify_before_retry_loop);
    interceptor_impl_fn!(ref read_before_attempt);
    interceptor_impl_fn!(mut modify_before_signing);
    interceptor_impl_fn!(ref read_before_signing);
    interceptor_impl_fn!(ref read_after_signing);
    interceptor_impl_fn!(mut modify_before_transmit);
    interceptor_impl_fn!(ref read_before_transmit);
    interceptor_impl_fn!(ref read_after_transmit);
    interceptor_impl_fn!(mut modify_before_deserialization);
    interceptor_impl_fn!(ref read_before_deserialization);
    interceptor_impl_fn!(ref read_after_deserialization);

    pub fn modify_before_attempt_completion(
        self,
        ctx: &mut InterceptorContext<Input, Output, Error>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        tracing::trace!("running `modify_before_attempt_completion` interceptors");
        let mut result: Result<(), BoxError> = Ok(());
        let mut ctx: FinalizerInterceptorContextMut<'_> = ctx.into();
        for interceptor in self.into_iter() {
            if let Some(interceptor) = interceptor.if_enabled(cfg) {
                if let Err(new_error) =
                    interceptor.modify_before_attempt_completion(&mut ctx, runtime_components, cfg)
                {
                    if let Err(last_error) = result {
                        tracing::debug!("{}", DisplayErrorContext(&*last_error));
                    }
                    result = Err(new_error);
                }
            }
        }
        result.map_err(InterceptorError::modify_before_attempt_completion)
    }

    pub fn read_after_attempt(
        self,
        ctx: &InterceptorContext<Input, Output, Error>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        tracing::trace!("running `read_after_attempt` interceptors");
        let mut result: Result<(), BoxError> = Ok(());
        let ctx: FinalizerInterceptorContextRef<'_> = ctx.into();
        for interceptor in self.into_iter() {
            if let Some(interceptor) = interceptor.if_enabled(cfg) {
                if let Err(new_error) =
                    interceptor.read_after_attempt(&ctx, runtime_components, cfg)
                {
                    if let Err(last_error) = result {
                        tracing::debug!("{}", DisplayErrorContext(&*last_error));
                    }
                    result = Err(new_error);
                }
            }
        }
        result.map_err(InterceptorError::read_after_attempt)
    }

    pub fn modify_before_completion(
        self,
        ctx: &mut InterceptorContext<Input, Output, Error>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        tracing::trace!("running `modify_before_completion` interceptors");
        let mut result: Result<(), BoxError> = Ok(());
        let mut ctx: FinalizerInterceptorContextMut<'_> = ctx.into();
        for interceptor in self.into_iter() {
            if let Some(interceptor) = interceptor.if_enabled(cfg) {
                if let Err(new_error) =
                    interceptor.modify_before_completion(&mut ctx, runtime_components, cfg)
                {
                    if let Err(last_error) = result {
                        tracing::debug!("{}", DisplayErrorContext(&*last_error));
                    }
                    result = Err(new_error);
                }
            }
        }
        result.map_err(InterceptorError::modify_before_completion)
    }

    pub fn read_after_execution(
        self,
        ctx: &InterceptorContext<Input, Output, Error>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), InterceptorError> {
        tracing::trace!("running `read_after_execution` interceptors");
        let mut result: Result<(), BoxError> = Ok(());
        let ctx: FinalizerInterceptorContextRef<'_> = ctx.into();
        for interceptor in self.into_iter() {
            if let Some(interceptor) = interceptor.if_enabled(cfg) {
                if let Err(new_error) =
                    interceptor.read_after_execution(&ctx, runtime_components, cfg)
                {
                    if let Err(last_error) = result {
                        tracing::debug!("{}", DisplayErrorContext(&*last_error));
                    }
                    result = Err(new_error);
                }
            }
        }
        result.map_err(InterceptorError::read_after_execution)
    }
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use crate::client::interceptors::context::Input;
    use crate::client::interceptors::{
        disable_interceptor, BeforeTransmitInterceptorContextRef, BoxError, Interceptor,
        InterceptorContext, Interceptors, SharedInterceptor,
    };
    use crate::client::runtime_components::{RuntimeComponents, RuntimeComponentsBuilder};
    use aws_smithy_types::config_bag::ConfigBag;

    #[derive(Debug)]
    struct TestInterceptor;
    impl Interceptor for TestInterceptor {}

    #[test]
    fn test_disable_interceptors() {
        #[derive(Debug)]
        struct PanicInterceptor;
        impl Interceptor for PanicInterceptor {
            fn read_before_transmit(
                &self,
                _context: &BeforeTransmitInterceptorContextRef<'_>,
                _rc: &RuntimeComponents,
                _cfg: &mut ConfigBag,
            ) -> Result<(), BoxError> {
                Err("boom".into())
            }
        }
        let rc = RuntimeComponentsBuilder::for_tests()
            .with_interceptor(SharedInterceptor::new(PanicInterceptor))
            .with_interceptor(SharedInterceptor::new(TestInterceptor))
            .build()
            .unwrap();

        let mut cfg = ConfigBag::base();
        let interceptors = Interceptors::new(rc.interceptors());
        assert_eq!(
            interceptors
                .into_iter()
                .filter(|i| i.if_enabled(&cfg).is_some())
                .count(),
            2
        );

        Interceptors::new(rc.interceptors())
            .read_before_transmit(&InterceptorContext::new(Input::new(5)), &rc, &mut cfg)
            .expect_err("interceptor returns error");
        cfg.interceptor_state()
            .store_put(disable_interceptor::<PanicInterceptor>("test"));
        assert_eq!(
            Interceptors::new(rc.interceptors())
                .into_iter()
                .filter(|i| i.if_enabled(&cfg).is_some())
                .count(),
            1
        );
        // shouldn't error because interceptors won't run
        Interceptors::new(rc.interceptors())
            .read_before_transmit(&InterceptorContext::new(Input::new(5)), &rc, &mut cfg)
            .expect("interceptor is now disabled");
    }
}
