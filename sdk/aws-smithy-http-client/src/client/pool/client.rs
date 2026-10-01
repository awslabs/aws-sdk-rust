/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Partition-bound handles for Smithy HTTP operations.

use super::partition::PartitionId;
use super::registry::PartitionState;
use super::ConnectionPool;
use crate::client::timeout::{self, TimeoutKind};
use crate::client::{downcast_error, ConnectorCallTimer};
use crate::sync::Arc;
use aws_smithy_async::rt::sleep::{default_async_sleep, SharedAsyncSleep};
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_runtime_api::client::connector_metadata::ConnectorMetadata;
use aws_smithy_runtime_api::client::http::telemetry::CaptureHttpAttemptTelemetry;
use aws_smithy_runtime_api::client::http::{
    HttpClient, HttpConnector, HttpConnectorFuture, HttpConnectorSettings, SharedHttpConnector,
};
use aws_smithy_runtime_api::client::orchestrator::{HttpRequest, HttpResponse};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_runtime_api::client::runtime_components::{
    RuntimeComponents, RuntimeComponentsBuilder,
};
use aws_smithy_types::config_bag::ConfigBag;
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use std::time::Duration;

/// Smithy HTTP client bound to one connection-pool partition.
///
/// Construction resolves the partition once. Every request uses that partition
/// for establishment placement and local reuse, while the configured reuse
/// scope may permit dispatch through a peer partition's connection. Cloning a
/// client shares the pool, resolved partition state, and reusable connections.
#[derive(Clone)]
pub struct Client {
    /// Shared connection pool and immutable policy.
    pool: ConnectionPool,
    /// Resolved runtime and network placement for this client.
    partition: Arc<PartitionState>,
}

impl Client {
    /// Creates a client for a pool built without explicit partitions.
    ///
    /// # Errors
    ///
    /// Returns [`ClientBuildError`] when the pool contains only explicit
    /// partitions.
    pub fn new(pool: &ConnectionPool) -> Result<Self, ClientBuildError> {
        Self::from_partition(pool, PartitionId::ANONYMOUS)
    }

    /// Creates a client bound to `id`.
    ///
    /// The identity may name a declared partition or the anonymous partition
    /// retained by a pool built without declarations.
    ///
    /// # Errors
    ///
    /// Returns [`ClientBuildError`] when the pool does not contain `id`.
    pub fn from_partition(
        pool: &ConnectionPool,
        id: PartitionId,
    ) -> Result<Self, ClientBuildError> {
        let partition = pool
            .inner
            .registry
            .partition(id)
            .ok_or_else(|| ClientBuildError::invalid_partition(id))?;
        Ok(Self {
            pool: pool.clone(),
            partition,
        })
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("partition", &self.partition.id())
            .field("pool", &self.pool)
            .finish_non_exhaustive()
    }
}

impl HttpClient for Client {
    fn validate_base_client_config(
        &self,
        _: &RuntimeComponentsBuilder,
        _: &ConfigBag,
    ) -> Result<(), BoxError> {
        self.pool
            .inner
            .transport
            .initialize_for_partition(&self.partition)
    }

    fn http_connector(
        &self,
        settings: &HttpConnectorSettings,
        components: &RuntimeComponents,
    ) -> SharedHttpConnector {
        let connect_timeout = settings.connect_timeout();
        let read_timeout = settings.read_timeout();
        let sleep = components.sleep_impl().or_else(default_async_sleep);
        let time_source = components.time_source().unwrap_or_default();

        SharedHttpConnector::new(PoolConnector {
            pool: self.pool.clone(),
            partition: self.partition.clone(),
            connect_timeout,
            read_timeout,
            sleep,
            time_source,
        })
    }

    fn connector_metadata(&self) -> Option<ConnectorMetadata> {
        Some(ConnectorMetadata::new("hyper", Some(Cow::Borrowed("1.x"))))
    }
}

/// Error returned when a [`Client`] cannot be resolved from a pool.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientBuildError {
    /// Construction failure represented without exposing the internal enum.
    kind: ClientBuildErrorKind,
}

impl ClientBuildError {
    /// Creates an unresolved-partition error.
    fn invalid_partition(partition: PartitionId) -> Self {
        Self {
            kind: ClientBuildErrorKind::InvalidPartition(partition),
        }
    }

    /// Returns the unresolved partition identity, when applicable.
    pub fn partition(&self) -> Option<PartitionId> {
        match &self.kind {
            ClientBuildErrorKind::InvalidPartition(partition) => Some(*partition),
        }
    }
}

impl fmt::Display for ClientBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ClientBuildErrorKind::InvalidPartition(partition) => {
                write!(
                    f,
                    "connection pool does not contain partition {partition:?}"
                )
            }
        }
    }
}

impl Error for ClientBuildError {}

/// Internal categories represented by [`ClientBuildError`].
#[derive(Clone, Debug, Eq, PartialEq)]
enum ClientBuildErrorKind {
    /// The pool retained no partition with this identity.
    InvalidPartition(PartitionId),
}

/// Per-operation Smithy adapter over one resolved pool partition.
///
/// [`HttpClient::http_connector`] combines the retained pool and partition
/// with that operation's timeout settings. Each call validates timeout timer
/// availability, converts the Smithy request, and routes it through the shared
/// pool. The connect timeout covers a newly started transport operation; the
/// read timeout covers pool acquisition and dispatch through response headers.
/// The adapter neither creates another pool nor resolves the partition again.
struct PoolConnector {
    /// Shared pool used for acquisition and dispatch.
    pool: ConnectionPool,
    /// Partition selected when the client was constructed.
    partition: Arc<PartitionState>,
    /// Maximum duration of the transport connection operation.
    ///
    /// Connector readiness completes before this timeout starts.
    connect_timeout: Option<Duration>,
    /// Maximum duration through response headers.
    read_timeout: Option<Duration>,
    /// Runtime timer used by operation timeouts.
    sleep: Option<SharedAsyncSleep>,
    /// Operation runtime clock used for request-attempt telemetry.
    time_source: SharedTimeSource,
}

impl fmt::Debug for PoolConnector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolConnector")
            .field("partition", &self.partition.id())
            .field("connect_timeout", &self.connect_timeout)
            .field("read_timeout", &self.read_timeout)
            .finish_non_exhaustive()
    }
}

/// Converts and dispatches one request through the partitioned pool.
async fn send_pool_request(
    pool: &ConnectionPool,
    partition: Arc<PartitionState>,
    request: HttpRequest,
    connect_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    sleep: Option<SharedAsyncSleep>,
    attempt_telemetry: Option<super::dispatch::AttemptTelemetryInput>,
) -> Result<HttpResponse, ConnectorError> {
    if (connect_timeout.is_some() || read_timeout.is_some()) && sleep.is_none() {
        return Err(ConnectorError::user(MissingAsyncSleep.into()));
    }

    let request = request
        .try_into_http1x()
        .map_err(|error| ConnectorError::user(error.into()))?;
    let options = super::dispatch::RequestOptions::new(
        connect_timeout
            .zip(sleep.clone())
            .map(|(duration, sleep)| super::establish::TransportTimeout::new(duration, sleep)),
        attempt_telemetry,
    );
    let send = pool.send_request(partition, request, options);
    let response =
        timeout::maybe_timeout_future(send, read_timeout, sleep.as_ref(), TimeoutKind::Read)
            .await
            .map_err(downcast_error)?;
    HttpResponse::try_from(response).map_err(|error| ConnectorError::other(error.into(), None))
}

impl HttpConnector for PoolConnector {
    fn call(&self, request: HttpRequest) -> HttpConnectorFuture {
        let attempt_capture = request.extension::<CaptureHttpAttemptTelemetry>().cloned();
        let connector_call_timer = ConnectorCallTimer::start(&request, &self.time_source);
        let attempt_telemetry = attempt_capture.as_ref().map(|capture| {
            super::dispatch::AttemptTelemetryInput::new(capture.clone(), self.time_source.clone())
        });
        let pool = self.pool.clone();
        let partition = self.partition.clone();
        let connect_timeout = self.connect_timeout;
        let read_timeout = self.read_timeout;
        let sleep = self.sleep.clone();
        HttpConnectorFuture::new(async move {
            let result = send_pool_request(
                &pool,
                partition,
                request,
                connect_timeout,
                read_timeout,
                sleep,
                attempt_telemetry,
            )
            .await;
            if let Some(timer) = connector_call_timer {
                timer.finish();
            }
            result
        })
    }
}

/// Operation timeouts were configured without a runtime timer.
#[derive(Debug)]
struct MissingAsyncSleep;

impl fmt::Display for MissingAsyncSleep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an async sleep implementation is required for HTTP connect or read timeouts")
    }
}

impl Error for MissingAsyncSleep {}

#[cfg(all(test, feature = "rt-tokio", not(smithy_http_client_loom)))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn configured_timeout_without_sleep_returns_a_user_error() {
        let pool = ConnectionPool::builder()
            .idle_timeout(None)
            .build_http()
            .unwrap();
        let client = Client::new(&pool).unwrap();

        for (connect_timeout, read_timeout) in [
            (Some(Duration::from_secs(1)), None),
            (None, Some(Duration::from_secs(1))),
        ] {
            let connector = PoolConnector {
                pool: client.pool.clone(),
                partition: client.partition.clone(),
                connect_timeout,
                read_timeout,
                sleep: None,
                time_source: SharedTimeSource::default(),
            };

            let error = connector
                .call(HttpRequest::get("http://example.com/").unwrap())
                .await
                .expect_err("a timeout without sleep unexpectedly started a request");
            assert!(error.is_user(), "unexpected connector error: {error:?}");
            assert_eq!(
                "an async sleep implementation is required for HTTP connect or read timeouts",
                std::error::Error::source(&error).unwrap().to_string()
            );
        }
    }
}
