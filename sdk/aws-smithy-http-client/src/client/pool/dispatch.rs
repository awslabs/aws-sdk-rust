/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Request routing and protocol dispatch.
//!
//! A [`super::Client`] supplies its partition and one request. This module then
//! performs the request journey in four phases:
//!
//! 1. validate protocol-sensitive request semantics;
//! 2. resolve the origin cell and its owner runtime;
//! 3. acquire the next authority that can make progress; and
//! 4. submit through the selected HTTP protocol.
//!
//! Acquisition does not necessarily acquire a connection. A request may
//! receive an exclusive HTTP/1 sender, an activation on an existing HTTP/2
//! generation, or authority to establish a new transport. A waiter event is
//! the lower-level transition that supplies one of those outcomes.
//!
//! Protocol dispatch may return the original request only when Hyper did not
//! accept it. The same acquisition remains authoritative while the dispatcher
//! selects a replacement; this is not an SDK request retry. Child modules own
//! protocol wire preparation and accepted-response lifetime.

mod h1;
mod h2;

use self::h1::H1DispatchOutcome;
use self::h2::H2DispatchOutcome;
use super::admission::ProtocolRequirement;
use super::cell::h1::H1Selection;
use super::cell::h2::H2Activation;
use super::cell::{AcquisitionOutcome, AcquisitionStep, OriginCell, WaiterId};
use super::establish::{self, TransportTimeout};
use super::partition::DriverSpawner;
use super::registry::PartitionState;
use super::{ConnectionPool, PoolInner};
use crate::sync::Arc;
use aws_smithy_async::time::SharedTimeSource;
use aws_smithy_runtime_api::client::connection::ConnectionMetadata;
use aws_smithy_runtime_api::client::http::telemetry::{
    CaptureHttpAttemptTelemetry, ConnectionAcquisitionTelemetry, ConnectionUsage,
};
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_types::body::SdkBody;
use http_1x::{header, Method, Request, Response, Uri, Version};
use std::future::poll_fn;
use std::time::SystemTime;

/// Replacement selections allowed after the initial HTTP/2 dispatch attempt.
const MAX_H2_REPLACEMENTS: usize = 2;

/// Operation settings known before the request's origin cell is resolved.
///
/// This value is separate from [`AcquisitionContext`], which also contains
/// cell and runtime state discovered by dispatch.
#[derive(Clone, Debug, Default)]
pub(super) struct RequestOptions {
    /// Transport connection timeout configured for this operation.
    connect_timeout: Option<TransportTimeout>,
    /// Inputs needed to start request-attempt observation at acquisition.
    attempt_telemetry: Option<AttemptTelemetryInput>,
}

impl RequestOptions {
    /// Creates operation settings for one pool request.
    pub(super) fn new(
        connect_timeout: Option<TransportTimeout>,
        attempt_telemetry: Option<AttemptTelemetryInput>,
    ) -> Self {
        Self {
            connect_timeout,
            attempt_telemetry,
        }
    }
}

/// Capture and operation runtime clock paired before pool dispatch.
#[derive(Clone, Debug)]
pub(super) struct AttemptTelemetryInput {
    capture: CaptureHttpAttemptTelemetry,
    time_source: SharedTimeSource,
}

impl AttemptTelemetryInput {
    /// Pairs the request capture with the clock that measures its attempt.
    pub(super) fn new(capture: CaptureHttpAttemptTelemetry, time_source: SharedTimeSource) -> Self {
        Self {
            capture,
            time_source,
        }
    }
}

/// Stable state shared by acquisition and protocol dispatch for one request.
///
/// This context owns ordinary shared configuration and identities. Linear
/// acquisition values, such as an establishment permit or an exclusive
/// protocol handle, remain separate arguments so their transfer is explicit.
#[derive(Clone)]
pub(super) struct AcquisitionContext {
    /// Shared pool policy and transport construction.
    pub(super) pool: Arc<PoolInner>,
    /// Partition from which the request was issued.
    pub(super) partition: Arc<PartitionState>,
    /// Partition-local state for the request's canonical origin.
    pub(super) cell: Arc<OriginCell>,
    /// Absolute request URI retained across protocol wire-form changes.
    pub(super) absolute_uri: Uri,
    /// Runtime that owns establishment, drivers, and pending return work.
    pub(super) owner_spawner: DriverSpawner,
    /// Transport connection timeout configured for this operation.
    pub(super) connect_timeout: Option<TransportTimeout>,
    /// Request-attempt observation retained until Hyper accepts a connection.
    pub(super) attempt_telemetry: Option<AttemptTelemetry>,
}

/// Request-scoped capture and acquisition start time.
///
/// This value exists only when the request carries a telemetry capture, so an
/// unobserved request does not read the clock.
#[derive(Clone)]
pub(super) struct AttemptTelemetry {
    /// Shared attempt capture installed on the Smithy request.
    capture: CaptureHttpAttemptTelemetry,
    /// Time at which this request entered pool acquisition.
    acquisition_started_at: SystemTime,
    /// Operation runtime clock used for this request attempt.
    time_source: SharedTimeSource,
}

impl AttemptTelemetry {
    /// Starts connection acquisition using the operation runtime clock.
    fn start(input: AttemptTelemetryInput) -> Self {
        let AttemptTelemetryInput {
            capture,
            time_source,
        } = input;
        let acquisition_started_at = time_source.now();
        Self {
            capture,
            acquisition_started_at,
            time_source,
        }
    }

    /// Freezes one selected connection when the protocol accepts the request.
    fn freeze_selection(
        &self,
        reused: bool,
        metadata: Option<&ConnectionMetadata>,
    ) -> FrozenConnectionSelection {
        FrozenConnectionSelection {
            capture: self.capture.clone(),
            acquisition_started_at: self.acquisition_started_at,
            acquisition_completed_at: self.time_source.now(),
            usage: if reused {
                ConnectionUsage::Reused
            } else {
                ConnectionUsage::Fresh
            },
            metadata: metadata
                .expect("attempt telemetry requires selected connection metadata")
                .clone(),
        }
    }
}

/// Selected connection facts held until the request cannot return to acquisition.
pub(super) struct FrozenConnectionSelection {
    capture: CaptureHttpAttemptTelemetry,
    acquisition_started_at: SystemTime,
    acquisition_completed_at: SystemTime,
    usage: ConnectionUsage,
    metadata: ConnectionMetadata,
}

impl FrozenConnectionSelection {
    /// Commits the selected connection and any valid acquisition interval.
    pub(super) fn record(self) {
        let acquisition = ConnectionAcquisitionTelemetry::from_interval(
            self.acquisition_started_at,
            self.acquisition_completed_at,
            self.usage,
        );
        self.capture
            .record_connection_selection(acquisition, self.metadata);
    }
}

impl AcquisitionContext {
    fn captures_connection_selection(&self) -> bool {
        self.attempt_telemetry.is_some()
    }

    /// Freezes the selected connection when Hyper may retain the request.
    ///
    /// Protocol dispatch records the resulting telemetry only after the request
    /// can no longer be returned to acquisition.
    pub(super) fn freeze_connection_selection(
        &self,
        reused: bool,
        metadata: Option<&ConnectionMetadata>,
    ) -> Option<FrozenConnectionSelection> {
        self.attempt_telemetry
            .as_ref()
            .map(|telemetry| telemetry.freeze_selection(reused, metadata))
    }
}

/// Resolves request-scoped pool state and runs acquisition through dispatch.
pub(super) async fn send(
    pool: &ConnectionPool,
    partition: Arc<PartitionState>,
    request: Request<SdkBody>,
    options: RequestOptions,
) -> Result<Response<SdkBody>, ConnectorError> {
    validate_request_before_acquisition(&request)
        .map_err(|error| ConnectorError::user(error.into()))?;

    let RequestOptions {
        connect_timeout,
        attempt_telemetry,
    } = options;
    let attempt_telemetry = attempt_telemetry.map(AttemptTelemetry::start);
    let absolute_uri = request.uri().clone();
    let cell = pool
        .inner
        .registry
        .resolve_cell(&partition, &absolute_uri)
        .map_err(|error| ConnectorError::user(error.into()))?;
    tracing::trace!(
        request_partition = ?partition.id(),
        origin_scheme = %cell.id().origin().scheme(),
        origin_host = cell.id().origin().host(),
        origin_port = ?cell.id().origin().port(),
        "request resolved to connection-pool cell"
    );

    let owner_spawner = partition
        .owner_spawner()
        .map_err(|error| ConnectorError::user(error.into()))?;
    partition.ensure_maintenance_started(&owner_spawner);

    let context = AcquisitionContext {
        pool: pool.inner.clone(),
        partition,
        cell,
        absolute_uri,
        owner_spawner,
        connect_timeout,
        attempt_telemetry,
    };

    acquire_and_dispatch(context, request).await
}

/// Request authority ready for one protocol-specific dispatch attempt.
enum DispatchTarget {
    /// Exclusive HTTP/1 sender ownership.
    H1(H1Selection),
    /// Prospective request reservation on one exact HTTP/2 generation.
    H2(H2Activation),
}

/// Acquires protocol dispatch authority and submits the request to Hyper.
///
/// A protocol dispatcher may return the original request only when Hyper did
/// not accept it. After one HTTP/2 target reaches dispatch, at most two stale
/// replacements may return the request for another selection.
async fn acquire_and_dispatch(
    context: AcquisitionContext,
    mut request: Request<SdkBody>,
) -> Result<Response<SdkBody>, ConnectorError> {
    let requirement = protocol_requirement(&request);
    let mut h2_replacements = H2ReplacementBudget::default();

    loop {
        match acquire_for_dispatch(&context, requirement).await? {
            DispatchTarget::H1(selection) => {
                match h1::dispatch(&context, request, selection).await? {
                    H1DispatchOutcome::Response(response) => return Ok(response),
                    H1DispatchOutcome::NotAccepted(returned) => request = returned,
                }
            }
            DispatchTarget::H2(activation) => {
                match h2::dispatch(&context, request, activation).await? {
                    H2DispatchOutcome::Response(response) => return Ok(response),
                    H2DispatchOutcome::NotAccepted(unaccepted) => {
                        let (returned, error) = unaccepted.into_parts();
                        if !h2_replacements.admit_replacement() {
                            return Err(error);
                        }
                        request = returned;
                    }
                }
            }
        }
    }
}

/// Acquires the next protocol value that can dispatch this request.
///
/// Local reusable values complete immediately under the cell lock. On a miss,
/// one waiter remains registered while reuse and establishment race to supply
/// dispatch authority. `poll_waiter` performs one synchronous state poll; no
/// cell lock is retained while this future is pending.
async fn acquire_for_dispatch(
    context: &AcquisitionContext,
    requirement: ProtocolRequirement,
) -> Result<DispatchTarget, ConnectorError> {
    'acquire: loop {
        if requirement.accepts_h2() {
            if let Some(activation) = OriginCell::select_h2(&context.cell) {
                tracing::trace!(
                    connection_id = %activation.connection().id(),
                    request_partition = ?context.partition.id(),
                    connection_partition = ?activation.connection().owner_partition(),
                    origin_scheme = %context.cell.id().origin().scheme(),
                    origin_host = context.cell.id().origin().host(),
                    origin_port = ?context.cell.id().origin().port(),
                    "HTTP/2 pool hit; activating local generation"
                );
                return Ok(DispatchTarget::H2(activation));
            }
        }
        if requirement.accepts_h1() {
            if let Some(selection) = OriginCell::select_h1(&context.cell) {
                tracing::trace!(
                    connection_id = %selection.connection_id(),
                    request_partition = ?context.partition.id(),
                    connection_partition = ?selection.connection().owner_partition(),
                    origin_scheme = %selection.connection().info().origin().scheme(),
                    origin_host = selection.connection().info().origin().host(),
                    origin_port = ?selection.connection().info().origin().port(),
                    "HTTP/1 pool hit; reusing idle connection"
                );
                return Ok(DispatchTarget::H1(selection));
            }
        }

        let waiter = OriginCell::register_waiter(&context.cell, requirement);
        tracing::trace!(
            request_partition = ?context.partition.id(),
            origin_scheme = %context.cell.id().origin().scheme(),
            origin_host = context.cell.id().origin().host(),
            origin_port = ?context.cell.id().origin().port(),
            protocol_requirement = ?requirement,
            "connection acquisition queued"
        );
        let mut waiter_guard = WaiterCancellationGuard::new(context.cell.clone(), waiter);
        loop {
            match poll_fn(|cx| context.cell.poll_waiter(waiter, cx)).await {
                AcquisitionStep::Resolved(AcquisitionOutcome::H1(selection)) => {
                    waiter_guard.disarm();
                    return Ok(DispatchTarget::H1(selection));
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::H2(activation)) => {
                    waiter_guard.disarm();
                    OriginCell::offer_local_h2(&context.cell);
                    OriginCell::offer_peer_h2(&context.cell);
                    tracing::trace!(
                        connection_id = %activation.connection().id(),
                        request_partition = ?context.partition.id(),
                        connection_partition = ?activation.connection().owner_partition(),
                        origin_scheme = %context.cell.id().origin().scheme(),
                        origin_host = context.cell.id().origin().host(),
                        origin_port = ?context.cell.id().origin().port(),
                        "HTTP/2 acquisition completed"
                    );
                    return Ok(DispatchTarget::H2(activation));
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::Failed(error)) => {
                    waiter_guard.disarm();
                    return Err(error);
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::RetryAcquisition) => {
                    waiter_guard.disarm();
                    continue 'acquire;
                }
                AcquisitionStep::StartEstablishment(permit) => {
                    tracing::trace!(
                        request_partition = ?context.partition.id(),
                        origin_scheme = %context.cell.id().origin().scheme(),
                        origin_host = context.cell.id().origin().host(),
                        origin_port = ?context.cell.id().origin().port(),
                        "connection establishment starting"
                    );
                    let attempt =
                        establish::establish(context.clone(), waiter, permit, requirement);
                    let completion =
                        EstablishmentCompletionGuard::new(context.cell.clone(), waiter);
                    context.owner_spawner.spawn(Box::pin(async move {
                        let mut completion = completion;
                        if !completion.start() {
                            drop(attempt);
                            completion.disarm();
                            return;
                        }
                        match attempt.await {
                            establish::EstablishmentOutcome::Complete(result) => {
                                completion.complete(result);
                            }
                            establish::EstablishmentOutcome::WaiterCompletionTransferred => {
                                completion.disarm();
                            }
                        }
                    }));
                }
            }
        }
    }
}
/// Per-request bound on replacement HTTP/2 selections.
#[derive(Default)]
struct H2ReplacementBudget {
    /// Replacement selections admitted after the initial selection.
    used: usize,
}

impl H2ReplacementBudget {
    /// Returns whether one more replacement selection may proceed.
    fn admit_replacement(&mut self) -> bool {
        if self.used >= MAX_H2_REPLACEMENTS {
            return false;
        }
        self.used += 1;
        true
    }
}

/// Guarantees a terminal result for one submitted establishment attempt.
///
/// The guard enters the owner-runtime future before submission. Runtime task
/// drop reports failure to the waiter. A result that wins before the submitted
/// future starts disarms the guard without polling the connector.
struct EstablishmentCompletionGuard {
    cell: Arc<OriginCell>,
    waiter: WaiterId,
    active: bool,
}

impl EstablishmentCompletionGuard {
    fn new(cell: Arc<OriginCell>, waiter: WaiterId) -> Self {
        Self {
            cell,
            waiter,
            active: true,
        }
    }

    fn start(&self) -> bool {
        self.cell.start_establishment(self.waiter)
    }

    fn complete(mut self, result: AcquisitionOutcome) {
        self.active = false;
        self.cell.complete_establishment(self.waiter, result);
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for EstablishmentCompletionGuard {
    fn drop(&mut self) {
        if self.active {
            let error = ConnectorError::io(EstablishmentTaskDropped.into());
            self.cell
                .complete_establishment(self.waiter, AcquisitionOutcome::Failed(error));
            tracing::debug!(
                request_partition = ?self.cell.id().partition(),
                connection_partition = ?self.cell.id().partition(),
                origin_scheme = %self.cell.id().origin().scheme(),
                origin_host = self.cell.id().origin().host(),
                origin_port = ?self.cell.id().origin().port(),
                "connection establishment task dropped"
            );
        }
    }
}

/// The owner runtime discarded an establishment future before completion.
#[derive(Debug)]
struct EstablishmentTaskDropped;

impl std::fmt::Display for EstablishmentTaskDropped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the owner-runtime connection establishment task was dropped")
    }
}

impl std::error::Error for EstablishmentTaskDropped {}

/// Cancels a request's waiter until it consumes a terminal acquisition outcome.
struct WaiterCancellationGuard {
    cell: Arc<OriginCell>,
    waiter: WaiterId,
    active: bool,
}

impl WaiterCancellationGuard {
    fn new(cell: Arc<OriginCell>, waiter: WaiterId) -> Self {
        Self {
            cell,
            waiter,
            active: true,
        }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for WaiterCancellationGuard {
    fn drop(&mut self) {
        if self.active {
            OriginCell::cancel_waiter(&self.cell, self.waiter);
        }
    }
}

/// Rejects request forms the pool cannot route before acquiring connection state.
fn validate_request_before_acquisition(
    request: &Request<SdkBody>,
) -> Result<(), RequestDispatchError> {
    if request.version() == Version::HTTP_2 && request.method() == Method::CONNECT {
        return Err(RequestDispatchError::ExtendedConnectUnsupported);
    }
    if request.version() == Version::HTTP_2 && has_upgrade_semantics(request) {
        return Err(RequestDispatchError::UpgradeOverHttp2);
    }
    match request.version() {
        Version::HTTP_11 | Version::HTTP_2 => Ok(()),
        Version::HTTP_10 if request.method() == Method::CONNECT => {
            Err(RequestDispatchError::ConnectOverHttp10)
        }
        Version::HTTP_10 => Ok(()),
        version => Err(RequestDispatchError::UnsupportedVersion(version)),
    }
}

/// Returns the protocol capability needed to preserve the request's wire semantics.
fn protocol_requirement(request: &Request<SdkBody>) -> ProtocolRequirement {
    if request.version() == Version::HTTP_2 {
        ProtocolRequirement::H2Required
    } else if request.version() == Version::HTTP_10
        || request.method() == Method::CONNECT
        || has_upgrade_semantics(request)
    {
        ProtocolRequirement::H1Required
    } else {
        ProtocolRequirement::H1Compatible
    }
}

/// Returns whether the request asks HTTP/1 to switch protocols.
fn has_upgrade_semantics(request: &Request<SdkBody>) -> bool {
    request.headers().contains_key(header::UPGRADE)
        || request
            .headers()
            .get_all(header::CONNECTION)
            .iter()
            .any(|value| {
                value.to_str().is_ok_and(|value| {
                    value
                        .split(',')
                        .any(|token| token.trim().eq_ignore_ascii_case("upgrade"))
                })
            })
}

/// Marker for a `Host` header synthesized during an HTTP/1 dispatch attempt.
#[derive(Clone, Copy, Debug)]
struct H1HostHeaderInserted;

/// A request rejected before Hyper accepts it for dispatch.
#[derive(Debug)]
enum RequestDispatchError {
    /// The absolute URI did not contain the authority required for `Host`.
    MissingAuthority,
    /// The derived `Host` value was not a valid HTTP header value.
    InvalidHostHeader(http_1x::header::InvalidHeaderValue),
    /// HTTP/1.0 cannot represent a CONNECT request accepted by this client.
    ConnectOverHttp10,
    /// The request selected an HTTP version this client cannot dispatch.
    UnsupportedVersion(Version),
    /// Extended CONNECT requires HTTP/2 stream-lifecycle support.
    ExtendedConnectUnsupported,
    /// HTTP/1 upgrade semantics cannot be represented by this HTTP/2 path.
    UpgradeOverHttp2,
}

impl std::fmt::Display for RequestDispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAuthority => f.write_str("request URI must contain an authority"),
            Self::InvalidHostHeader(_) => {
                f.write_str("request URI authority is not a valid Host header")
            }
            Self::ConnectOverHttp10 => f.write_str("CONNECT is not supported over HTTP/1.0"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported HTTP version: {version:?}")
            }
            Self::ExtendedConnectUnsupported => {
                f.write_str("extended CONNECT over HTTP/2 is not supported")
            }
            Self::UpgradeOverHttp2 => {
                f.write_str("HTTP/1 protocol upgrade semantics cannot use HTTP/2")
            }
        }
    }
}

impl std::error::Error for RequestDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidHostHeader(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;

    fn request(method: Method, version: Version) -> Request<SdkBody> {
        Request::builder()
            .method(method)
            .uri("https://example.com/resource")
            .version(version)
            .body(SdkBody::empty())
            .unwrap()
    }

    #[test]
    fn request_semantics_select_protocol_capability() {
        assert_eq!(
            ProtocolRequirement::H1Compatible,
            protocol_requirement(&request(Method::GET, Version::HTTP_11))
        );
        assert_eq!(
            ProtocolRequirement::H2Required,
            protocol_requirement(&request(Method::GET, Version::HTTP_2))
        );
        assert_eq!(
            ProtocolRequirement::H1Required,
            protocol_requirement(&request(Method::GET, Version::HTTP_10))
        );
        assert_eq!(
            ProtocolRequirement::H1Required,
            protocol_requirement(&request(Method::CONNECT, Version::HTTP_11))
        );

        let mut upgrade = request(Method::GET, Version::HTTP_11);
        upgrade
            .headers_mut()
            .insert(header::CONNECTION, "keep-alive, Upgrade".parse().unwrap());
        assert_eq!(
            ProtocolRequirement::H1Required,
            protocol_requirement(&upgrade)
        );
    }

    #[test]
    fn unsupported_h2_tunnel_and_upgrade_forms_are_rejected() {
        assert!(matches!(
            validate_request_before_acquisition(&request(Method::CONNECT, Version::HTTP_2)),
            Err(RequestDispatchError::ExtendedConnectUnsupported)
        ));

        let mut upgrade = request(Method::GET, Version::HTTP_2);
        upgrade
            .headers_mut()
            .insert(header::UPGRADE, "websocket".parse().unwrap());
        assert!(matches!(
            validate_request_before_acquisition(&upgrade),
            Err(RequestDispatchError::UpgradeOverHttp2)
        ));
    }

    #[test]
    fn h2_replacement_is_bounded_after_two_selections() {
        let mut budget = H2ReplacementBudget::default();
        for _ in 0..MAX_H2_REPLACEMENTS {
            assert!(budget.admit_replacement());
        }
        assert!(!budget.admit_replacement());
    }
}
