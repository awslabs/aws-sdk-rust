/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_http::connection::{CaptureSmithyConnection, ConnectionMetadata};
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::interceptors::context::{
    BeforeDeserializationInterceptorContextMut, BeforeTransmitInterceptorContextMut,
};
use aws_smithy_runtime_api::client::interceptors::Interceptor;
use aws_smithy_runtime_api::client::retries::{ClassifyRetry, RetryReason};
use aws_smithy_runtime_api::client::runtime_components::RuntimeComponents;
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreReplace};
use aws_smithy_types::retry::{ErrorKind, ReconnectMode, RetryConfig};
use std::fmt;
use tracing::{debug, error};

/// A interceptor for poisoning connections in response to certain events.
///
/// This interceptor, when paired with a compatible connection, allows the connection to be
/// poisoned in reaction to certain events *(like receiving a transient error.)* This allows users
/// to avoid sending requests to a server that isn't responding. This can increase the load on a
/// server, because more connections will be made overall.
///
/// **In order for this interceptor to work,** the configured connection must interact with the
/// "connection retriever" stored in an HTTP request's `extensions` map. For an example of this,
/// see [aws_smithy_client::hyper_ext::Adapter](https://github.com/awslabs/smithy-rs/blob/47b3d23ff3cabd67e797af616101f5a4ea6be5e8/rust-runtime/aws-smithy-client/src/hyper_ext.rs#L155).
/// When a connection is made available to the retriever, this interceptor will call a `.poison`
/// method on it, signalling that the connection should be dropped. It is up to the connection
/// implementer to handle this.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct ConnectionPoisoningInterceptor {}

impl ConnectionPoisoningInterceptor {
    /// Create a new `ConnectionPoisoningInterceptor`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Interceptor for ConnectionPoisoningInterceptor {
    fn name(&self) -> &'static str {
        "ConnectionPoisoningInterceptor"
    }

    fn modify_before_transmit(
        &self,
        context: &mut BeforeTransmitInterceptorContextMut<'_>,
        _runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let capture_smithy_connection = CaptureSmithyConnectionWrapper::new();
        context
            .request_mut()
            .extensions_mut()
            .insert(capture_smithy_connection.clone_inner());
        cfg.interceptor_state().store_put(capture_smithy_connection);

        Ok(())
    }

    fn modify_before_deserialization(
        &self,
        context: &mut BeforeDeserializationInterceptorContextMut<'_>,
        runtime_components: &RuntimeComponents,
        cfg: &mut ConfigBag,
    ) -> Result<(), BoxError> {
        let reconnect_mode = cfg
            .load::<RetryConfig>()
            .map(RetryConfig::reconnect_mode)
            .unwrap_or(ReconnectMode::ReconnectOnTransientError);
        let captured_connection = cfg.load::<CaptureSmithyConnectionWrapper>().cloned();
        let retry_classifiers = runtime_components
            .retry_classifiers()
            .ok_or("retry classifiers are required for connection poisoning to work")?;

        let error_is_transient = retry_classifiers
            .classify_retry(context.inner_mut())
            .map(|reason| reason == RetryReason::Error(ErrorKind::TransientError))
            .unwrap_or_default();
        let connection_poisoning_is_enabled =
            reconnect_mode == ReconnectMode::ReconnectOnTransientError;

        if error_is_transient && connection_poisoning_is_enabled {
            debug!("received a transient error, poisoning the connection...");

            if let Some(captured_connection) = captured_connection.and_then(|conn| conn.get()) {
                captured_connection.poison();
                debug!("the connection was poisoned")
            } else {
                error!(
                    "unable to poison the connection because no connection was found! The underlying HTTP connector never set a connection."
                );
            }
        }

        Ok(())
    }
}

// TODO(enableNewSmithyRuntimeCleanup): A storable wrapper won't be needed anymore once we absorb aws_smithy_http into the new runtime crate.
/// A wrapper around CaptureSmithyConnection that implements `Storable` so that it can be added to the `ConfigBag`.
#[derive(Clone, Default)]
pub struct CaptureSmithyConnectionWrapper {
    inner: CaptureSmithyConnection,
}

impl CaptureSmithyConnectionWrapper {
    /// Creates a new `CaptureSmithyConnectionWrapper`.
    pub fn new() -> Self {
        Self {
            inner: CaptureSmithyConnection::new(),
        }
    }

    /// Returns a reference to the inner `CaptureSmithyConnection`.
    pub fn clone_inner(&self) -> CaptureSmithyConnection {
        self.inner.clone()
    }

    /// Returns the captured connection metadata, if any.
    pub fn get(&self) -> Option<ConnectionMetadata> {
        self.inner.get()
    }

    /// Sets the connection retriever function.
    pub fn set_connection_retriever<F>(&self, f: F)
    where
        F: Fn() -> Option<ConnectionMetadata> + Send + Sync + 'static,
    {
        self.inner.set_connection_retriever(f)
    }
}

impl Storable for CaptureSmithyConnectionWrapper {
    type Storer = StoreReplace<Self>;
}

impl fmt::Debug for CaptureSmithyConnectionWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CaptureSmithyConnectionWrapper")
    }
}
