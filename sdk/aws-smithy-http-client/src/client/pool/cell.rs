/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! State local to one partition and origin.
//!
//! An [`OriginCell`] owns its acquisition order and protocol records. A
//! bounded cell publishes aggregate demand to [`OriginAdmission`] after
//! releasing its lock, and admission returns capacity through the same
//! unlocked boundary.
//!
//! [`AcquisitionQueue`] owns every live acquisition attempt, including the
//! bounded FIFO, delivery crossings, establishment launch, cancellation, and
//! terminal results. Its module documents the complete transition model.
//! [`CellState`] couples that model to HTTP/1 sender ownership, HTTP/2 flights,
//! connection generations, activation gates, and cross-cell reuse. A local
//! HTTP/1 return either satisfies one acquisition or becomes idle, never both.
//! An HTTP/2 activation becomes visible only with a prospective request claim
//! and one recorded activation turn.
//!
//! Permits, senders, admission updates, and task wakes are detached from
//! mutable state before their fallback or callback runs. This keeps drop and
//! wake paths outside the cell lock.

pub(super) mod h1;
pub(super) mod h2;
mod waiters;

#[cfg(test)]
use self::h1::H1CloseHandle;
#[cfg(all(test, not(smithy_http_client_loom)))]
use self::h1::H1DriverGuard;
#[cfg(test)]
use self::h1::H1Sender;
use self::h1::{H1CellState, H1Selection, OwnedH1Sender};
#[cfg(test)]
use self::waiters::AcquisitionProbe;
pub(in crate::client::pool) use self::waiters::WaiterId;
use self::waiters::{AcquisitionQueue, CellCommitError, CellCommitOutcome, DeliveryReservation};
use super::admission::{
    AdmissionAction, CapacityLease, DeliveryGuard, DemandSnapshot, H1MatchId, H1SupplyStatus,
    H2SupplyStatus, OriginAdmission, ProtocolRequirement, SupplyRevision,
};
use super::connection::CloseReason;
#[cfg(test)]
use super::connection::ConnectionInfo;
#[cfg(test)]
use super::connection::ConnectionState;
use super::maintenance::PartitionMaintenance;
use super::origin::OriginKey;
use super::partition::{EligibilityGroup, PartitionId};
use super::stats::{CellConnectionStats, PartitionConnectionStats};
use crate::sync::{Arc, Mutex};
#[cfg(test)]
use aws_smithy_runtime_api::client::connection::ConnectionId;
use aws_smithy_runtime_api::client::result::ConnectorError;
use std::task::{Context, Poll};
use std::time::SystemTime;

/// Stable identity of an [`OriginCell`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(in crate::client::pool) struct CellId {
    /// Partition that owns this cell's connection work.
    partition: PartitionId,
    /// Canonical origin whose requests share this cell.
    origin: OriginKey,
}

impl CellId {
    pub(super) fn new(partition: PartitionId, origin: OriginKey) -> Self {
        Self { partition, origin }
    }

    pub(in crate::client::pool) fn partition(&self) -> PartitionId {
        self.partition
    }

    pub(in crate::client::pool) fn origin(&self) -> &OriginKey {
        &self.origin
    }
}

/// Stable state shared by requests for one partition and canonical origin.
pub(in crate::client::pool) struct OriginCell {
    /// Complete identity used by partition and admission indexes.
    id: CellId,
    /// Partitions whose connections may satisfy this cell's demand.
    eligibility_group: EligibilityGroup,
    /// Origin-wide authority present only when capacity is bounded.
    admission: Option<Arc<OriginAdmission>>,
    /// Partition-owned idle deadline scheduler.
    maintenance: Option<Arc<PartitionMaintenance>>,
    /// Connection lifetimes that may outlive records under `state`.
    connection_stats: Arc<CellConnectionStats>,
    /// Waiter order and protocol records protected by one cell-local lock.
    state: Mutex<CellState>,
}

/// Mutable state protected by one partition-origin cell lock.
///
/// Waiter outcomes and protocol state share this lock. HTTP/1 returns,
/// HTTP/2 activation gates, flight participants, and cancellation therefore
/// commit against one acquisition order.
#[derive(Debug)]
struct CellState {
    /// Cell-local acquisition order and delivered results.
    acquisitions: AcquisitionQueue,
    /// Complete HTTP/1 sender and peer-reservation state.
    h1: H1CellState,
    /// Complete HTTP/2 acquisition state for this cell.
    h2: h2::H2CellState,
    /// Last admission-facing HTTP/1 status and its monotonic revision.
    h1_supply_revision: SupplyRevision<H1SupplyStatus>,
    /// Last admission-facing HTTP/2 status and its monotonic revision.
    h2_supply_revision: SupplyRevision<H2SupplyStatus>,
}

impl Default for CellState {
    fn default() -> Self {
        Self {
            acquisitions: AcquisitionQueue::default(),
            h1: H1CellState::default(),
            h2: h2::H2CellState::default(),
            h1_supply_revision: SupplyRevision::new(
                0,
                H1SupplyStatus {
                    has_returnable_connection: false,
                    peer_use_blocked: false,
                },
            ),
            h2_supply_revision: SupplyRevision::new(0, H2SupplyStatus::Unavailable),
        }
    }
}

impl CellState {
    /// Checks the coupled waiter and protocol state machines.
    fn assert_consistent(&self) {
        #[cfg(any(debug_assertions, test))]
        {
            if std::thread::panicking() {
                return;
            }
            self.acquisitions.assert_consistent();
            self.h1.assert_consistent();
            self.h2.assert_consistent();
            self.h2.assert_pending_waiters(&self.acquisitions);
        }
    }

    /// Derives the complete HTTP/1 status admission needs from cell-owned state.
    fn h1_supply_status(&self) -> H1SupplyStatus {
        let local_h1_demand = self.acquisitions.has_h1_compatible_waiter();
        H1SupplyStatus {
            has_returnable_connection: self.h1.has_returnable(),
            peer_use_blocked: !self.h1.peer_reservation_available()
                || self.h1.blocks_peer_selection(local_h1_demand)
                || self.acquisitions.has_prior_h1_waiter(),
        }
    }

    /// Returns an HTTP/1 revision only when admission's view must change.
    ///
    /// The first submission advances to revision one even for the initial
    /// unavailable status because it also drives retained H1 matching.
    fn take_h1_supply_update(&mut self) -> Option<SupplyRevision<H1SupplyStatus>> {
        let current = self.h1_supply_status();
        if self.h1_supply_revision.revision != 0 && self.h1_supply_revision.status == current {
            return None;
        }
        Some(self.advance_h1_supply_revision(current))
    }

    /// Returns the current HTTP/1 revision for an exact crossing fallback.
    fn current_h1_supply_revision(&mut self) -> SupplyRevision<H1SupplyStatus> {
        let current = self.h1_supply_status();
        if self.h1_supply_revision.revision != 0 && self.h1_supply_revision.status == current {
            return self.h1_supply_revision;
        }
        self.advance_h1_supply_revision(current)
    }

    fn advance_h1_supply_revision(
        &mut self,
        status: H1SupplyStatus,
    ) -> SupplyRevision<H1SupplyStatus> {
        let revision = self
            .h1_supply_revision
            .revision
            .checked_add(1)
            .expect("HTTP/1 supply revision exhausted");
        self.h1_supply_revision = SupplyRevision::new(revision, status);
        self.h1_supply_revision
    }

    /// Returns an HTTP/2 revision only when admission's view must change.
    ///
    /// Initial unavailability is suppressed because absence already represents
    /// that state in admission.
    fn take_h2_supply_update(&mut self) -> Option<SupplyRevision<H2SupplyStatus>> {
        let current = self.h2_supply_status();
        if self.h2_supply_revision.status == current {
            return None;
        }
        Some(self.advance_h2_supply_revision(current))
    }

    /// Returns the current HTTP/2 revision for an exact reclaim fallback.
    fn current_h2_supply_revision(&mut self) -> SupplyRevision<H2SupplyStatus> {
        let current = self.h2_supply_status();
        if self.h2_supply_revision.status == current {
            return self.h2_supply_revision;
        }
        self.advance_h2_supply_revision(current)
    }

    /// Derives the exact generation admission may route or reclaim.
    fn h2_supply_status(&self) -> H2SupplyStatus {
        match self.h2.peer_routable_generation() {
            Some(generation) => H2SupplyStatus::Accepting {
                generation,
                idle: self.h2.is_idle(generation),
            },
            None => H2SupplyStatus::Unavailable,
        }
    }

    fn advance_h2_supply_revision(
        &mut self,
        status: H2SupplyStatus,
    ) -> SupplyRevision<H2SupplyStatus> {
        let revision = self
            .h2_supply_revision
            .revision
            .checked_add(1)
            .expect("HTTP/2 supply revision exhausted");
        self.h2_supply_revision = SupplyRevision::new(revision, status);
        self.h2_supply_revision
    }

    /// Atomically decides whether a peer may reserve this cell's connection.
    fn commit_h1_reservation(
        &mut self,
        match_id: H1MatchId,
    ) -> H1ReservationDecision<OwnedH1Sender> {
        let local_h1_demand = self.acquisitions.has_h1_compatible_waiter();
        if self.h1.blocks_peer_selection(local_h1_demand)
            || self.acquisitions.has_prior_h1_waiter()
            || !self.h1.peer_reservation_available()
        {
            let report = self.current_h1_supply_revision();
            self.assert_consistent();
            return H1ReservationDecision::Rejected(report);
        }

        if let Some(owner) = self.h1.take_idle_candidate() {
            assert!(
                self.h1.reserve_resolving(match_id),
                "available HTTP/1 connection could not reserve its peer match"
            );
            self.assert_consistent();
            return H1ReservationDecision::Candidate(owner);
        }
        if self.h1.has_returnable() {
            assert!(
                self.h1.reserve_waiting(match_id),
                "available HTTP/1 connection could not reserve its peer match"
            );
            self.assert_consistent();
            return H1ReservationDecision::Installed;
        }

        let report = self.current_h1_supply_revision();
        self.assert_consistent();
        H1ReservationDecision::Rejected(report)
    }

    /// Extracts an idle sender without reserving a future sender return.
    fn try_take_idle_h1(&mut self, match_id: H1MatchId) -> H1IdleProbeDecision<OwnedH1Sender> {
        let local_h1_demand = self.acquisitions.has_h1_compatible_waiter();
        if self.h1.blocks_peer_selection(local_h1_demand)
            || self.acquisitions.has_prior_h1_waiter()
            || !self.h1.peer_reservation_available()
        {
            let revision = self.current_h1_supply_revision();
            self.assert_consistent();
            return H1IdleProbeDecision::Unavailable(revision);
        }

        let Some(owner) = self.h1.take_idle_candidate() else {
            let revision = self.current_h1_supply_revision();
            self.assert_consistent();
            return H1IdleProbeDecision::Unavailable(revision);
        };
        assert!(
            self.h1.reserve_resolving(match_id),
            "idle HTTP/1 connection could not reserve its peer match"
        );
        self.assert_consistent();
        H1IdleProbeDecision::Candidate(owner)
    }

    /// Clears a peer reservation and returns the cell's complete H1 status.
    fn cancel_h1_reservation(&mut self, match_id: H1MatchId) -> SupplyRevision<H1SupplyStatus> {
        self.h1.release_peer_reservation(match_id);
        let local_h1_demand = self.acquisitions.has_h1_compatible_waiter();
        self.h1.clear_unused_turn(local_h1_demand);
        self.assert_consistent();
        self.current_h1_supply_revision()
    }

    /// Clears a reservation before its provisional sender follows ordinary return.
    fn reject_h1_match(&mut self, match_id: H1MatchId) {
        self.h1.release_peer_reservation(match_id);
        self.assert_consistent();
    }

    /// Revalidates one retained match and sender residence as one transition.
    fn commit_h1_match(&mut self, match_id: H1MatchId, owner: &OwnedH1Sender) -> bool {
        let committed =
            self.h1.names_peer_reservation(match_id) && self.h1.commit_return_to_waiter(owner);
        self.assert_consistent();
        committed
    }

    /// Completes or rejects a reservation after its external action resolves.
    fn complete_h1_match(
        &mut self,
        match_id: H1MatchId,
        transferred: bool,
    ) -> SupplyRevision<H1SupplyStatus> {
        if transferred {
            let local_h1_demand = self.acquisitions.has_h1_compatible_waiter();
            self.h1.complete_peer_transfer(match_id, local_h1_demand);
        } else {
            self.h1.release_peer_reservation(match_id);
        }
        self.assert_consistent();
        self.current_h1_supply_revision()
    }

    /// Removes active HTTP/2-compatible demand already served by visible H2 state.
    fn publishable_demand_updates(
        &self,
        updates: [Option<DemandSnapshot>; 2],
    ) -> [Option<DemandSnapshot>; 2] {
        let h2_visible = self.h2.has_visible_h2();
        updates.map(|snapshot| {
            snapshot
                .filter(|snapshot| !h2_visible || !snapshot.is_active() || !snapshot.accepts_h2())
        })
    }
}

/// Supplier-cell decision for one HTTP/1 reservation attempt.
pub(in crate::client::pool) enum H1ReservationDecision<C> {
    /// A future reusable sender return will satisfy the retained match.
    Installed,
    /// An idle sender was extracted immediately.
    Candidate(C),
    /// The supplier cell could not reserve sender ownership for the match.
    Rejected(SupplyRevision<H1SupplyStatus>),
}

/// Supplier-cell result of probing for an immediately idle HTTP/1 sender.
pub(in crate::client::pool) enum H1IdleProbeDecision<C> {
    /// An idle sender was extracted immediately.
    Candidate(C),
    /// The supplier remains live but has no sender available for immediate use.
    Unavailable(SupplyRevision<H1SupplyStatus>),
}

impl<C> H1IdleProbeDecision<C> {
    /// Maps only the detached sender while preserving the supplier decision.
    fn map_candidate<D>(self, map: impl FnOnce(C) -> D) -> H1IdleProbeDecision<D> {
        match self {
            Self::Candidate(candidate) => H1IdleProbeDecision::Candidate(map(candidate)),
            Self::Unavailable(revision) => H1IdleProbeDecision::Unavailable(revision),
        }
    }
}

impl<C> H1ReservationDecision<C> {
    /// Maps only the detached sender while preserving the supplier decision.
    fn map_candidate<D>(self, map: impl FnOnce(C) -> D) -> H1ReservationDecision<D> {
        match self {
            Self::Installed => H1ReservationDecision::Installed,
            Self::Candidate(candidate) => H1ReservationDecision::Candidate(map(candidate)),
            Self::Rejected(revision) => H1ReservationDecision::Rejected(revision),
        }
    }
}

/// A terminal result that may satisfy an acquisition waiter.
#[derive(Debug)]
pub(super) enum AcquisitionOutcome {
    /// An exclusive HTTP/1 sender selected from an installed cell-owned record.
    H1(H1Selection),
    /// A prospective request lease against one HTTP/2 generation.
    H2(h2::H2Activation),
    /// Establishment failed before producing a dispatchable connection.
    Failed(ConnectorError),
    /// An HTTP/2 generation closed before serving a transferred attempt.
    RetryAcquisition,
}

/// One event observed while driving an acquisition attempt.
#[derive(Debug)]
pub(super) enum AcquisitionStep {
    /// Capacity is available and one establishment attempt may start.
    StartEstablishment(EstablishmentPermit),
    /// A returned sender or establishment produced the terminal result.
    Resolved(AcquisitionOutcome),
}

/// Optional bounded-origin capacity for one establishment attempt.
///
/// Unbounded origins carry no lease. For bounded origins, dropping this value
/// before connection installation returns the permit to admission.
#[derive(Debug)]
pub(super) struct EstablishmentPermit {
    /// Bounded-origin slot transferred into a connection on success.
    lease: Option<CapacityLease>,
}

impl EstablishmentPermit {
    /// Creates a permit for an origin without a connection bound.
    fn unbounded() -> Self {
        Self { lease: None }
    }

    /// Takes ownership of one bounded-origin capacity lease.
    pub(in crate::client::pool) fn bounded(lease: CapacityLease) -> Self {
        Self { lease: Some(lease) }
    }

    /// Transfers bounded capacity into an installed connection, when present.
    pub(super) fn into_lease(self) -> Option<CapacityLease> {
        self.lease
    }
}

impl OriginCell {
    /// Creates a cell before its one-time publication in a partition registry.
    pub(super) fn new(
        partition: PartitionId,
        origin: OriginKey,
        eligibility_group: EligibilityGroup,
        admission: Option<Arc<OriginAdmission>>,
        maintenance: Option<Arc<PartitionMaintenance>>,
    ) -> Self {
        Self {
            id: CellId::new(partition, origin),
            eligibility_group,
            admission,
            maintenance,
            connection_stats: Arc::new(CellConnectionStats::default()),
            state: Mutex::new(CellState::default()),
        }
    }

    /// Returns the next idle deadline from immutable pool policy.
    pub(in crate::client::pool) fn idle_deadline(&self) -> Option<SystemTime> {
        self.maintenance
            .as_ref()
            .and_then(|maintenance| maintenance.idle_deadline())
    }

    /// Publishes a newly idle deadline to partition maintenance.
    fn notify_maintenance(&self, deadline: Option<SystemTime>) {
        if let Some(maintenance) = &self.maintenance {
            maintenance.notify_deadline(deadline);
        }
    }

    pub(in crate::client::pool) fn id(&self) -> &CellId {
        &self.id
    }

    /// Returns shared lifetime counts for establishments and connections.
    pub(in crate::client::pool) fn connection_stats(&self) -> Arc<CellConnectionStats> {
        self.connection_stats.clone()
    }

    /// Returns connection state for this exact partition-origin cell.
    pub(in crate::client::pool) fn connection_stats_snapshot(&self) -> PartitionConnectionStats {
        let (pending, h1_idle, h1_active, h2_accepting, h2_active_requests) = {
            let state = self.state.lock();
            let (h1_idle, h1_active) = state.h1.connection_counts();
            let (h2_accepting, h2_active_requests) = state.h2.connection_counts();
            (
                state.acquisitions.pending_count(),
                h1_idle,
                h1_active,
                h2_accepting,
                h2_active_requests,
            )
        };
        self.connection_stats.snapshot(
            pending,
            h1_idle,
            h1_active,
            h2_accepting,
            h2_active_requests,
        )
    }

    #[cfg(test)]
    pub(in crate::client::pool) fn eligibility_group(&self) -> &EligibilityGroup {
        &self.eligibility_group
    }

    #[cfg(test)]
    pub(in crate::client::pool) fn admission(&self) -> Option<&Arc<OriginAdmission>> {
        self.admission.as_ref()
    }

    /// Closes H1 records and H2 generations whose idle deadline elapsed.
    pub(super) fn expire_idle(cell: &Arc<Self>, now: SystemTime) {
        let (expired_h1, expired_h2) = {
            let state = cell.state.lock();
            (state.h1.expired_idle(now), state.h2.expired(now))
        };
        for id in expired_h1 {
            if Self::close_h1(cell, id, CloseReason::IdleTimeout) {
                tracing::trace!(
                    connection_id = %id,
                    connection_partition = ?cell.id.partition(),
                    origin_scheme = %cell.id.origin().scheme(),
                    origin_host = cell.id.origin().host(),
                    origin_port = ?cell.id.origin().port(),
                    "HTTP/1 idle connection expired"
                );
            }
        }
        if let Some(generation) = expired_h2 {
            if Self::close_h2(cell, generation, CloseReason::IdleTimeout) {
                tracing::trace!(
                    connection_partition = ?cell.id.partition(),
                    origin_scheme = %cell.id.origin().scheme(),
                    origin_host = cell.id.origin().host(),
                    origin_port = ?cell.id.origin().port(),
                    h2_generation = ?generation,
                    "HTTP/2 idle generation expired"
                );
            }
        }
    }

    /// Returns this cell's nearest reusable connection deadline.
    pub(super) fn nearest_idle_deadline(&self) -> Option<SystemTime> {
        let state = self.state.lock();
        [
            state.h1.nearest_idle_deadline(),
            state.h2.nearest_idle_deadline(),
        ]
        .into_iter()
        .flatten()
        .min()
    }

    /// Logically closes every retained connection for pool shutdown.
    pub(super) fn close_all(cell: &Arc<Self>, reason: CloseReason) {
        let (h1, h2) = {
            let state = cell.state.lock();
            (state.h1.connection_ids(), state.h2.generation_ids())
        };
        for id in h1 {
            Self::close_h1(cell, id, reason);
        }
        for generation in h2 {
            Self::close_h2(cell, generation, reason);
        }
    }

    /// Applies one resolved acquisition delivery after admission unlock.
    ///
    /// Connection-cell H1 revalidation has already completed, so reserving
    /// the requesting cell cannot be followed by another fallible connection-
    /// cell transition.
    ///
    /// # Panics
    ///
    /// Panics if a waiter reserved by this function disappears or enters a
    /// state that cannot accept the committed acquisition step.
    pub(in crate::client::pool) fn receive_delivery(
        cell: &Arc<Self>,
        delivery: DeliveryGuard,
    ) -> Option<AdmissionAction> {
        let reservation = {
            let mut state = cell.state.lock();
            state
                .acquisitions
                .reserve_delivery_waiter(delivery.demand(), &cell.eligibility_group)
        };

        let DeliveryReservation::Reserved { waiter, successor } = reservation else {
            return delivery.refuse(None);
        };

        let (step, mut settlement) = delivery.into_step(successor);
        let (commit, suppress_successor) = {
            let mut state = cell.state.lock();
            let commit = match step {
                AcquisitionStep::StartEstablishment(permit) => {
                    state.acquisitions.commit_capacity(waiter, permit)
                }
                AcquisitionStep::Resolved(outcome) => {
                    state.acquisitions.commit_borrowed_h1(waiter, outcome)
                }
            };
            (commit, state.h2.has_visible_h2())
        };
        if suppress_successor {
            settlement.suppress_h2_successor();
        }

        let (next, waker, error) = match commit {
            CellCommitOutcome::Committed { waker } => (settlement.accept(), waker, None),
            CellCommitOutcome::Refused { returned, waker } => {
                (settlement.refuse(returned), waker, None)
            }
            CellCommitOutcome::Invalid {
                returned,
                waker,
                error,
            } => (settlement.refuse(returned), waker, Some(error)),
        };
        if let Some(waker) = waker {
            waker.wake();
        }
        if let Some(error) = error {
            match error {
                CellCommitError::MissingWaiter => {
                    panic!("reserved waiter disappeared before acquisition delivery")
                }
                CellCommitError::UnexpectedState => {
                    panic!("reserved waiter entered an invalid acquisition-delivery state")
                }
            }
        }
        next
    }

    /// Returns the number of retained acquisition waiters for boundary tests.
    #[cfg(test)]
    pub(super) fn retained_waiters_for_test(&self) -> usize {
        self.state.lock().acquisitions.probe().retained
    }

    /// Registers one acquisition waiter in cell-local arrival order.
    ///
    /// The waiter and its demand snapshot are committed under the cell lock.
    /// A bounded cell publishes the snapshot to origin admission only after
    /// that lock is released. An unbounded cell retains only local order.
    pub(super) fn register_waiter(cell: &Arc<Self>, requirement: ProtocolRequirement) -> WaiterId {
        let (waiter, snapshot, h2_visible) = {
            let mut state = cell.state.lock();
            let (waiter, snapshot) = state.acquisitions.register_waiter(
                requirement,
                &cell.eligibility_group,
                cell.admission.is_some(),
            );
            (
                waiter,
                snapshot,
                state.h2.has_visible_h2() && requirement.accepts_h2(),
            )
        };

        if let (Some(admission), Some(snapshot), false) = (&cell.admission, snapshot, h2_visible) {
            OriginAdmission::submit_demand_snapshot(admission, cell.id.partition(), snapshot);
        }
        Self::offer_local_h2(cell);
        Self::offer_peer_h2(cell);
        waiter
    }

    /// Cancels a waiter and re-examines local idle H1 state atomically.
    ///
    /// A newly compatible successor may be serviceable by an H1 that an older
    /// HTTP/2-only head could not use. The local sender is selected under the
    /// same cell lock as cancellation, so it never needs a same-cell peer
    /// H1 match. Demand publication, fallback, and waking remain outside the lock.
    pub(super) fn cancel_waiter(cell: &Arc<Self>, waiter: WaiterId) -> bool {
        let transition = {
            let mut state = cell.state.lock();
            state.h2.cancel_flight_participant(waiter);
            state.h2.cancel_peer_activation(waiter);
            state.h2.cancel_pending_waiter(waiter);
            state
                .acquisitions
                .cancel_waiter(waiter, &cell.eligibility_group)
                .map(|mut cancelled| {
                    let mut local_install = if state.acquisitions.has_h1_compatible_waiter() {
                        state.h1.select_idle().map(|owner| {
                            state.h1.consume_local_turn();
                            let (waiter, resolution) = state.acquisitions.offer_returned_h1(
                                || AcquisitionOutcome::H1(H1Selection::new(cell, owner)),
                                &cell.eligibility_group,
                            );
                            (
                                waiter.expect(
                                    "compatible HTTP/1 waiter disappeared during local offer",
                                ),
                                resolution,
                            )
                        })
                    } else {
                        None
                    };
                    cancelled.demand_updates =
                        state.publishable_demand_updates(cancelled.demand_updates);
                    if let Some((waiter, install)) = &mut local_install {
                        state.h2.cancel_pending_waiter(*waiter);
                        install.demand_updates = state.publishable_demand_updates(std::mem::take(
                            &mut install.demand_updates,
                        ));
                    }
                    (cancelled, local_install)
                })
        };
        let Some((cancelled, local_install)) = transition else {
            return false;
        };

        if let Some(admission) = &cell.admission {
            for snapshot in cancelled
                .demand_updates
                .into_iter()
                .chain(
                    local_install
                        .as_ref()
                        .into_iter()
                        .flat_map(|(_, install)| install.demand_updates.iter().cloned()),
                )
                .flatten()
            {
                OriginAdmission::submit_demand_snapshot(admission, cell.id.partition(), snapshot);
            }
        }

        // Ready results and any locally rejected event cross the lock boundary
        // before their fallback can re-enter the pool.
        drop(cancelled.returned_steps);
        if let Some((_, install)) = local_install {
            drop(install.returned_step);
            if let Some(waker) = install.waker {
                waker.wake();
            }
        }

        let supply_update = {
            let mut state = cell.state.lock();
            let local_h1_demand = state.acquisitions.has_h1_compatible_waiter();
            state.h1.clear_unused_turn(local_h1_demand);
            state.assert_consistent();
            state.take_h1_supply_update()
        };
        cell.submit_h1_supply_update(supply_update);
        Self::offer_local_h2(cell);
        Self::offer_peer_h2(cell);
        true
    }

    /// Polls the next event for one acquisition waiter.
    ///
    /// The cell lock is held only for this synchronous state-machine poll. It
    /// is released before `Poll::Pending` returns to the async caller and is
    /// never held while polling Hyper, connector I/O, or user code.
    ///
    /// # Panics
    ///
    /// Panics if `waiter` is unknown, was cancelled, or is polled again after
    /// its ready result was consumed.
    pub(super) fn poll_waiter(
        &self,
        waiter: WaiterId,
        cx: &mut Context<'_>,
    ) -> Poll<AcquisitionStep> {
        let mut state = self.state.lock();
        state.acquisitions.poll_waiter(waiter, cx)
    }

    /// Marks an establishment attempt as started before its first connector poll.
    pub(super) fn start_establishment(&self, waiter: WaiterId) -> bool {
        self.state.lock().acquisitions.start_establishment(waiter)
    }

    /// Commits one terminal establishment result to its launching waiter.
    ///
    /// A returned H1 may already have served or cancelled the waiter. In that
    /// case the losing result leaves the cell lock and follows its ordinary
    /// sender-return or error-drop fallback.
    ///
    /// # Panics
    ///
    /// Panics if the launching waiter disappeared or left the launching state
    /// before its owned establishment completed.
    pub(super) fn complete_establishment(&self, waiter: WaiterId, result: AcquisitionOutcome) {
        let served_with_h1 = matches!(&result, AcquisitionOutcome::H1(_));
        let commit = {
            let mut state = self.state.lock();
            let commit = state.acquisitions.commit_establishment(waiter, result);
            let accepted = matches!(commit, CellCommitOutcome::Committed { .. });
            if accepted {
                if served_with_h1 {
                    state.h1.consume_local_turn();
                } else {
                    let local_h1_demand = state.acquisitions.has_h1_compatible_waiter();
                    state.h1.clear_unused_turn(local_h1_demand);
                }
                state.assert_consistent();
            }
            commit
        };
        let (returned, waker, error) = match commit {
            CellCommitOutcome::Committed { waker } => ([None, None], waker, None),
            CellCommitOutcome::Refused { returned, waker } => (returned, waker, None),
            CellCommitOutcome::Invalid {
                returned,
                waker,
                error,
            } => (returned, waker, Some(error)),
        };
        drop(returned);
        if let Some(waker) = waker {
            waker.wake();
        }
        if let Some(error) = error {
            match error {
                CellCommitError::MissingWaiter => {
                    unreachable!("establishment result reported a missing waiter")
                }
                CellCommitError::UnexpectedState => {
                    panic!("establishment completed for a waiter that was not launching")
                }
            }
        }
        let supply_update = {
            let mut state = self.state.lock();
            state.assert_consistent();
            state.take_h1_supply_update()
        };
        self.submit_h1_supply_update(supply_update);
    }

    #[cfg(test)]
    pub(super) fn take_ready_lease(cell: &Arc<Self>, waiter: WaiterId) -> Option<CapacityLease> {
        let permit = match cell.take_ready_event(waiter)? {
            AcquisitionStep::StartEstablishment(permit) => permit,
            AcquisitionStep::Resolved(_) => {
                panic!("capacity test received a terminal acquisition outcome")
            }
        };
        assert!(Self::cancel_waiter(cell, waiter));
        permit.into_lease()
    }

    #[cfg(test)]
    fn take_ready_h1(&self, waiter: WaiterId) -> Option<H1Selection> {
        match self.take_ready_event(waiter)? {
            AcquisitionStep::Resolved(AcquisitionOutcome::H1(selection)) => Some(selection),
            AcquisitionStep::Resolved(AcquisitionOutcome::H2(_)) => {
                panic!("HTTP/1 ownership test received an HTTP/2 activation")
            }
            AcquisitionStep::Resolved(AcquisitionOutcome::Failed(_)) => {
                panic!("HTTP/1 ownership test received establishment failure")
            }
            AcquisitionStep::Resolved(AcquisitionOutcome::RetryAcquisition) => {
                panic!("HTTP/1 ownership test received an internal reacquisition")
            }
            AcquisitionStep::StartEstablishment(_) => {
                panic!("HTTP/1 ownership test received establishment capacity")
            }
        }
    }

    #[cfg(test)]
    fn take_ready_event(&self, waiter: WaiterId) -> Option<AcquisitionStep> {
        let mut state = self.state.lock();
        match state
            .acquisitions
            .poll_waiter(waiter, &mut Context::from_waker(std::task::Waker::noop()))
        {
            Poll::Ready(event) => Some(event),
            Poll::Pending => None,
        }
    }

    #[cfg(test)]
    pub(super) fn register_waiter_without_publish(
        &self,
        requirement: ProtocolRequirement,
    ) -> (WaiterId, DemandSnapshot) {
        let (waiter, snapshot) = self.state.lock().acquisitions.register_waiter(
            requirement,
            &self.eligibility_group,
            true,
        );
        (
            waiter,
            snapshot.expect("first unpublished waiter did not create demand"),
        )
    }

    #[cfg(test)]
    fn probe(&self) -> AcquisitionProbe {
        self.state.lock().acquisitions.probe()
    }
}

impl std::fmt::Debug for OriginCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginCell")
            .field("id", &self.id)
            .field("eligibility_group", &self.eligibility_group)
            .field("admission", &self.admission)
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use http_1x::uri::Scheme;
    use std::num::NonZeroUsize;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;
    use std::task::{Wake, Waker};

    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: StdArc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }

        fn wake_by_ref(self: &StdArc<Self>) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn bounded_cell() -> (Arc<OriginAdmission>, Arc<OriginCell>) {
        bounded_cell_with_limit(1)
    }

    fn bounded_cell_with_limit(limit: usize) -> (Arc<OriginAdmission>, Arc<OriginCell>) {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(limit).unwrap());
        let candidate = Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            Some(admission.clone()),
            None,
        ));
        let cell = OriginAdmission::register_cell(&admission, candidate);
        (admission, cell)
    }

    fn saturated_bounded_cell() -> (Arc<OriginAdmission>, Arc<OriginCell>, CapacityLease) {
        let (admission, cell) = bounded_cell();
        let lease = OriginAdmission::lease_for_test(&admission);
        (admission, cell, lease)
    }

    fn bounded_peer_cells(
        limit: usize,
        owning_group: EligibilityGroup,
        requesting_group: EligibilityGroup,
    ) -> (Arc<OriginAdmission>, Arc<OriginCell>, Arc<OriginCell>) {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(limit).unwrap());
        let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
        let connection_cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                origin.clone(),
                owning_group,
                Some(admission.clone()),
                None,
            )),
        );
        let requesting_cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(2),
                origin,
                requesting_group,
                Some(admission.clone()),
                None,
            )),
        );
        (admission, connection_cell, requesting_cell)
    }

    fn unbounded_cell() -> Arc<OriginCell> {
        Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ))
    }

    fn connection_info(id: u64) -> Arc<ConnectionInfo> {
        ConnectionInfo::for_test(ConnectionId::new(id), PartitionId::from_index(1))
    }

    fn unbounded_connection(
        id: u64,
    ) -> (
        Arc<ConnectionState>,
        super::super::connection::PhysicalConnectionGuard,
    ) {
        ConnectionState::unbounded(connection_info(id))
    }

    #[test]
    fn h1_supply_revision_advances_only_when_status_changes() {
        let mut state = CellState::default();
        assert!(state.take_h1_supply_update().is_some());
        assert_eq!(1, state.h1_supply_revision.revision);
        assert!(state.take_h1_supply_update().is_none());
        assert_eq!(1, state.h1_supply_revision.revision);

        let (connection, _physical) = unbounded_connection(1);
        state
            .h1
            .insert_idle(connection, H1Sender::test(11), None)
            .unwrap();
        assert!(state.take_h1_supply_update().is_some());
        assert_eq!(2, state.h1_supply_revision.revision);
    }

    #[test]
    fn active_peer_reservation_blocks_h1_supply() {
        let mut state = CellState::default();
        let (connection, _physical) = unbounded_connection(1);
        let owner = state
            .h1
            .insert_selected(connection, H1Sender::test(11))
            .expect("fresh HTTP/1 record was rejected");
        let match_id = H1MatchId::for_test(1);
        assert!(state.h1.reserve_waiting(match_id));

        assert!(state.h1_supply_status().peer_use_blocked);

        assert!(state.h1.release_peer_reservation(match_id));
        assert!(state.h1.close_owned(&owner));
        drop(owner);
    }

    #[test]
    fn connection_stats_snapshot_reads_exact_cell_state() {
        let cell = unbounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let (connection, _physical) = unbounded_connection(1);
        OriginCell::insert_idle_h1(&cell, connection, H1Sender::test(11));

        let stats = cell.connection_stats_snapshot();

        assert_eq!(1, stats.pending_acquisitions());
        assert_eq!(1, stats.h1().idle());
        assert_eq!(0, stats.h1().active());
        assert_eq!(0, stats.h2().accepting());
        assert_eq!(0, stats.h2().active_requests());
        assert!(cell.take_ready_event(waiter).is_some());
    }

    #[test]
    fn idle_h1_selection_returns_to_its_owning_cell() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        OriginCell::insert_idle_h1(&cell, connection, H1Sender::test(11));

        let selection = OriginCell::select_h1(&cell).expect("idle sender was not selected");
        assert_eq!(ConnectionId::new(1), selection.connection_id());
        assert_eq!(11, selection.test_sender_id());
        assert!(selection.is_reused());
        assert_eq!((1, 0), cell.h1_counts());

        drop(selection);
        assert_eq!((1, 1), cell.h1_counts());
        assert_eq!(
            11,
            OriginCell::select_h1(&cell)
                .expect("returned sender was not reusable")
                .test_sender_id()
        );
    }

    #[test]
    fn selected_h1_closes_when_its_owning_cell_is_gone() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));

        drop(cell);
        drop(selection);

        assert_eq!(
            Some(CloseReason::PoolDropped),
            connection.probe().close_reason
        );
    }

    #[test]
    fn complete_h1_exchange_returns_sender_to_idle() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        assert!(!selection.is_reused());

        selection.into_exchange().offer_for_reuse();
        assert_eq!((1, 1), cell.h1_counts());
    }

    #[test]
    fn incomplete_h1_return_closes_and_releases_capacity() {
        let (admission, cell) = bounded_cell();
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));

        drop(selection.into_exchange());

        assert_eq!((0, 0), cell.h1_counts());
        assert_eq!(1, admission.available_capacity_for_test());
        assert_eq!(
            Some(CloseReason::IncompleteH1Exchange),
            connection.probe().close_reason
        );
    }

    #[test]
    fn returned_h1_satisfies_an_unbounded_waiter() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        drop(selection);

        let selected = cell
            .take_ready_h1(waiter)
            .expect("returned sender did not satisfy the waiter");
        assert_eq!(11, selected.test_sender_id());
        assert!(selected.is_reused());
        assert_eq!(0, cell.probe().retained);
        drop(selected);
        assert_eq!((1, 1), cell.h1_counts());
    }

    #[test]
    fn returned_h1_satisfies_bounded_demand_without_releasing_capacity() {
        let (admission, cell) = bounded_cell();
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        assert_eq!(1, admission.ordered_demand_count_for_test());

        drop(selection);

        assert_eq!(0, admission.ordered_demand_count_for_test());
        assert_eq!(0, admission.available_capacity_for_test());
        let selected = cell
            .take_ready_h1(waiter)
            .expect("returned sender did not satisfy bounded demand");
        drop(selected);
        assert_eq!((1, 1), cell.h1_counts());
    }

    #[test]
    fn eligible_peer_borrows_an_idle_h1_without_moving_capacity() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        OriginCell::insert_idle_h1(&connection_cell, connection, H1Sender::test(11));

        let waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);
        let borrowed = requesting_cell
            .take_ready_h1(waiter)
            .expect("eligible peer did not receive the idle HTTP/1 sender");

        assert_eq!(11, borrowed.test_sender_id());
        assert_eq!(0, admission.available_capacity_for_test());
        assert_eq!((1, 0), connection_cell.h1_counts());
        drop(borrowed);
        assert_eq!((1, 1), connection_cell.h1_counts());
    }

    #[test]
    fn peer_idle_probe_skips_a_busy_supplier_for_an_idle_one() {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(2).unwrap());
        let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
        let supplier = |index| {
            OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(index),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            )
        };
        let busy_cell = supplier(1);
        let idle_cell = supplier(2);
        let requesting_cell = supplier(3);

        let busy_lease = OriginAdmission::lease_for_test(&admission);
        let (busy_connection, _busy_physical) = ConnectionState::bounded(
            ConnectionInfo::for_test(ConnectionId::new(1), busy_cell.id().partition()),
            busy_lease,
        );
        let busy = OriginCell::insert_selected_h1(&busy_cell, busy_connection, H1Sender::test(11));

        let idle_lease = OriginAdmission::lease_for_test(&admission);
        let (idle_connection, _idle_physical) = ConnectionState::bounded(
            ConnectionInfo::for_test(ConnectionId::new(2), idle_cell.id().partition()),
            idle_lease,
        );
        OriginCell::insert_idle_h1(&idle_cell, idle_connection, H1Sender::test(22));

        let waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);
        let borrowed = requesting_cell
            .take_ready_h1(waiter)
            .expect("bounded idle probing did not find the second supplier");

        assert_eq!(22, borrowed.test_sender_id());
        assert_eq!((1, 0), busy_cell.h1_counts());
        assert_eq!((1, 0), idle_cell.h1_counts());
        drop(borrowed);
        drop(busy);
    }

    #[test]
    fn ineligible_peer_reclaims_h1_capacity_without_moving_the_sender() {
        let (admission, connection_cell, requesting_cell) = bounded_peer_cells(
            1,
            EligibilityGroup::Partition(PartitionId::from_index(1)),
            EligibilityGroup::Partition(PartitionId::from_index(2)),
        );
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        OriginCell::insert_idle_h1(&connection_cell, connection.clone(), H1Sender::test(11));

        let waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);
        let replacement = OriginCell::take_ready_lease(&requesting_cell, waiter)
            .expect("ineligible peer did not receive reclaimed capacity");

        assert_eq!(
            Some(CloseReason::Reclaimed),
            connection.probe().close_reason
        );
        assert_eq!((0, 0), connection_cell.h1_counts());
        assert_eq!(0, admission.available_capacity_for_test());
        drop(replacement);
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn installed_peer_reservation_intercepts_the_next_active_h1_return() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        let selected =
            OriginCell::insert_selected_h1(&connection_cell, connection, H1Sender::test(11));
        let waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);

        drop(selected);

        let borrowed = requesting_cell
            .take_ready_h1(waiter)
            .expect("installed peer reservation did not intercept the active return");
        assert_eq!(11, borrowed.test_sender_id());
        drop(borrowed);
        assert_eq!((1, 1), connection_cell.h1_counts());
    }

    #[test]
    fn cancelling_requesting_cell_releases_connection_reservation() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        let selected =
            OriginCell::insert_selected_h1(&connection_cell, connection, H1Sender::test(11));
        let waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);

        assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
        assert!(
            connection_cell
                .state
                .lock()
                .h1
                .peer_reservation_available(),
            "requesting cell cancellation did not reconcile the installed supplier-cell reservation"
        );
        drop(selected);

        assert_eq!((1, 1), connection_cell.h1_counts());
        assert_eq!(0, admission.ordered_demand_count_for_test());
    }

    #[test]
    fn cross_cell_borrow_owes_one_usable_turn_to_the_owning_cell() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        let selected =
            OriginCell::insert_selected_h1(&connection_cell, connection, H1Sender::test(11));

        let requesting_waiter =
            OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);
        let local_waiter =
            OriginCell::register_waiter(&connection_cell, ProtocolRequirement::H1Compatible);
        drop(selected);

        let borrowed = requesting_cell
            .take_ready_h1(requesting_waiter)
            .expect("older peer demand did not receive the intercepted sender");
        assert!(connection_cell.state.lock().h1.local_turn_owed());
        drop(borrowed);

        let local = connection_cell
            .take_ready_h1(local_waiter)
            .expect("connection-owning cell local demand did not receive its owed turn");
        assert!(!connection_cell.state.lock().h1.local_turn_owed());
        drop(local);
        assert_eq!((1, 1), connection_cell.h1_counts());
    }

    #[test]
    fn cancelling_ready_h1_returns_it_after_unlock() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        drop(selection);

        assert!(OriginCell::cancel_waiter(&cell, waiter));
        assert_eq!((1, 1), cell.h1_counts());
        assert_eq!(
            11,
            OriginCell::select_h1(&cell)
                .expect("cancelled result did not return its sender")
                .test_sender_id()
        );
    }

    #[test]
    fn returned_h1_and_establishment_complete_one_acquisition_attempt() {
        let cell = unbounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not start establishment")
        else {
            panic!("unbounded miss completed before establishment started");
        };
        assert!(cell.start_establishment(waiter));
        assert!(permit.into_lease().is_none());

        let (returning_connection, _returning_physical) = unbounded_connection(1);
        let returning =
            OriginCell::insert_selected_h1(&cell, returning_connection, H1Sender::test(11));
        let (fresh_connection, _fresh_physical) = unbounded_connection(2);
        let fresh = OriginCell::insert_selected_h1(&cell, fresh_connection, H1Sender::test(22));

        drop(returning);
        cell.complete_establishment(waiter, AcquisitionOutcome::H1(fresh));

        assert_eq!((2, 1), cell.h1_counts());
        let winner = cell
            .take_ready_h1(waiter)
            .expect("acquisition attempt had no terminal H1");
        assert_eq!(11, winner.test_sender_id());
        drop(winner);
        assert_eq!((2, 2), cell.h1_counts());
    }

    #[test]
    fn bounded_attempt_remains_eligible_for_a_returned_h1() {
        let (admission, cell) = bounded_cell_with_limit(2);
        let installed_lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), installed_lease);
        let returning = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));

        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("bounded miss received no capacity")
        else {
            panic!("bounded miss completed before establishment started");
        };
        let attempt_lease = permit
            .into_lease()
            .expect("bounded establishment carried no capacity");
        assert_eq!(0, admission.available_capacity_for_test());

        drop(returning);
        let selected = cell
            .take_ready_h1(waiter)
            .expect("returned H1 did not beat the pending establishment");
        drop(attempt_lease);
        assert_eq!(1, admission.available_capacity_for_test());
        drop(selected);
        assert_eq!((1, 1), cell.h1_counts());
    }

    #[test]
    fn older_launching_waiter_precedes_younger_capacity_waiter_for_returned_h1() {
        let (admission, cell) = bounded_cell_with_limit(2);
        let installed_lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), installed_lease);
        let returning = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));

        let older = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(older)
            .expect("older waiter received no establishment capacity")
        else {
            panic!("older waiter completed before establishment started");
        };
        let attempt_lease = permit
            .into_lease()
            .expect("bounded establishment carried no capacity");

        let younger = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        assert!(cell.take_ready_event(younger).is_none());

        drop(returning);

        let selected = cell
            .take_ready_h1(older)
            .expect("younger capacity waiter overtook the older launching waiter");
        assert_eq!(11, selected.test_sender_id());
        assert!(cell.take_ready_event(younger).is_none());

        assert!(OriginCell::cancel_waiter(&cell, younger));
        drop(attempt_lease);
        drop(selected);
        assert_eq!((1, 1), cell.h1_counts());
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn establishment_failure_is_a_terminal_acquisition_outcome() {
        let cell = unbounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not start establishment")
        else {
            panic!("unbounded miss completed before establishment started");
        };
        assert!(cell.start_establishment(waiter));
        drop(permit);

        cell.complete_establishment(
            waiter,
            AcquisitionOutcome::Failed(ConnectorError::io(
                std::io::Error::other("synthetic establishment failure").into(),
            )),
        );

        let AcquisitionStep::Resolved(AcquisitionOutcome::Failed(error)) = cell
            .take_ready_event(waiter)
            .expect("establishment failure was not delivered")
        else {
            panic!("establishment failure produced the wrong acquisition step");
        };
        assert!(error.is_io());
        assert_eq!(0, cell.probe().retained);
    }

    #[test]
    fn establishment_completion_clears_the_served_waiters_local_turn() {
        let cell = unbounded_cell();
        let match_id = H1MatchId::for_test(1);
        {
            let mut state = cell.state.lock();
            assert!(state.h1.reserve_resolving(match_id));
            assert!(state.h1.complete_peer_transfer(match_id, true));
        }
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not prepare establishment")
        else {
            panic!("unbounded miss completed before establishment was prepared");
        };
        assert!(cell.start_establishment(waiter));
        drop(permit);
        let (connection, _physical) = unbounded_connection(1);
        let fresh = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));

        cell.complete_establishment(waiter, AcquisitionOutcome::H1(fresh));

        assert!(!cell.state.lock().h1.local_turn_owed());
        drop(
            cell.take_ready_h1(waiter)
                .expect("successful establishment result was not retained"),
        );
    }

    #[test]
    fn failed_establishment_preserves_a_turn_for_compatible_successor() {
        let cell = unbounded_cell();
        let match_id = H1MatchId::for_test(1);
        {
            let mut state = cell.state.lock();
            assert!(state.h1.reserve_resolving(match_id));
            assert!(state.h1.complete_peer_transfer(match_id, true));
        }
        let failed = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(failed)
            .expect("unbounded miss did not prepare establishment")
        else {
            panic!("unbounded miss completed before establishment was prepared");
        };
        assert!(cell.start_establishment(failed));
        drop(permit);
        let successor = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        cell.complete_establishment(
            failed,
            AcquisitionOutcome::Failed(ConnectorError::io(
                std::io::Error::other("synthetic establishment failure").into(),
            )),
        );

        assert!(cell.state.lock().h1.local_turn_owed());
        assert!(matches!(
            cell.take_ready_event(failed),
            Some(AcquisitionStep::Resolved(AcquisitionOutcome::Failed(_)))
        ));
        assert!(OriginCell::cancel_waiter(&cell, successor));
        assert!(!cell.state.lock().h1.local_turn_owed());
    }

    #[test]
    fn failed_establishment_clears_a_turn_after_local_demand_drains() {
        let cell = unbounded_cell();
        let match_id = H1MatchId::for_test(1);
        {
            let mut state = cell.state.lock();
            assert!(state.h1.reserve_resolving(match_id));
            assert!(state.h1.complete_peer_transfer(match_id, true));
        }
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not prepare establishment")
        else {
            panic!("unbounded miss completed before establishment was prepared");
        };
        assert!(cell.start_establishment(waiter));
        drop(permit);

        cell.complete_establishment(
            waiter,
            AcquisitionOutcome::Failed(ConnectorError::io(
                std::io::Error::other("synthetic establishment failure").into(),
            )),
        );

        assert!(!cell.state.lock().h1.local_turn_owed());
        assert!(matches!(
            cell.take_ready_event(waiter),
            Some(AcquisitionStep::Resolved(AcquisitionOutcome::Failed(_)))
        ));
    }

    #[test]
    fn completion_after_waiter_cancellation_returns_the_new_h1() {
        let cell = unbounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not start establishment")
        else {
            panic!("unbounded miss completed before establishment started");
        };
        assert!(cell.start_establishment(waiter));
        assert!(OriginCell::cancel_waiter(&cell, waiter));
        drop(permit);

        let (connection, _physical) = unbounded_connection(1);
        let fresh = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        cell.complete_establishment(waiter, AcquisitionOutcome::H1(fresh));

        assert_eq!((1, 1), cell.h1_counts());
        assert_eq!(
            11,
            OriginCell::select_h1(&cell)
                .expect("cancelled acquisition lost its completed H1")
                .test_sender_id()
        );
    }

    #[test]
    fn returned_h1_prevents_the_first_establishment_poll() {
        let cell = unbounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not prepare establishment")
        else {
            panic!("unbounded miss completed before establishment was prepared");
        };
        let (connection, _physical) = unbounded_connection(1);
        let returning = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));

        drop(returning);
        assert!(!cell.start_establishment(waiter));
        drop(permit);
        let selected = cell
            .take_ready_h1(waiter)
            .expect("returned H1 did not win before establishment started");
        assert_eq!(11, selected.test_sender_id());
    }

    #[test]
    fn h2_required_head_does_not_take_h1() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);

        drop(selection);

        assert_eq!((1, 1), cell.h1_counts());
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("unbounded miss did not start establishment")
        else {
            panic!("H2-required waiter accepted an HTTP/1 selection");
        };
        assert!(OriginCell::cancel_waiter(&cell, waiter));
        drop(permit);
    }

    #[test]
    fn cancelling_h2_head_refunnels_reclaimed_capacity_to_successor() {
        let (admission, cell) = bounded_cell();
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
        OriginCell::insert_idle_h1(&cell, connection, H1Sender::test(11));
        let h2_head = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
        let h1_successor = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(h2_head)
            .expect("HTTP/2 demand did not receive reclaimed local capacity")
        else {
            panic!("HTTP/2 demand received a protocol handle instead of capacity");
        };
        assert!(OriginCell::cancel_waiter(&cell, h2_head));
        drop(permit);

        let successor = OriginCell::take_ready_lease(&cell, h1_successor)
            .expect("reclaimed capacity did not reach the compatible successor");
        assert_eq!((0, 0), cell.h1_counts());
        assert!(cell.state.lock().h1.peer_reservation_available());
        drop(successor);
    }

    #[test]
    fn close_marks_selected_record_until_sender_returns() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));
        let close = H1CloseHandle::new(&cell, selection.connection());

        assert!(close.close(CloseReason::Poisoned));
        assert!(OriginCell::select_h1(&cell).is_none());
        assert_eq!((1, 0), cell.h1_counts());

        drop(selection);
        assert_eq!((0, 0), cell.h1_counts());
        assert_eq!(Some(CloseReason::Poisoned), connection.probe().close_reason);
    }

    #[test]
    fn confirmed_upgrade_completes_an_earlier_driver_close() {
        let (admission, cell) = bounded_cell();
        let lease = OriginAdmission::lease_for_test(&admission);
        let (connection, physical) = ConnectionState::bounded(connection_info(1), lease);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));
        let dispatch = ConnectionState::try_commit_dispatch(&connection).unwrap();
        let driver = H1DriverGuard::new(H1CloseHandle::new(&cell, &connection));
        let return_task = selection.into_exchange();

        driver.protocol_closed();
        assert!(connection.probe().awaiting_h1_exchange);
        assert_eq!(0, admission.available_capacity_for_test());

        return_task.retire_connection(CloseReason::Upgraded);
        assert_eq!(Some(CloseReason::Upgraded), connection.probe().close_reason);
        assert_eq!(0, admission.available_capacity_for_test());
        assert_eq!((0, 0), cell.h1_counts());

        dispatch.release();
        physical.release();
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn driver_drop_closes_its_installed_generation() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));
        let driver = H1DriverGuard::new(H1CloseHandle::new(&cell, &connection));

        drop(driver);
        assert_eq!(
            Some(CloseReason::OwnerRuntimeShutdown),
            connection.probe().close_reason
        );
        drop(selection);
        assert_eq!((0, 0), cell.h1_counts());
    }

    #[test]
    fn owner_runtime_shutdown_drops_live_driver_guard() {
        let cell = unbounded_cell();
        let (connection, _physical) = unbounded_connection(1);
        let selection =
            OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));
        let driver = H1DriverGuard::new(H1CloseHandle::new(&cell, &connection));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        let (started, observed) = std::sync::mpsc::sync_channel(0);

        runtime.spawn(async move {
            let _driver = driver;
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        observed
            .recv_timeout(std::time::Duration::from_secs(1))
            .expect("driver task did not start");
        runtime.shutdown_timeout(std::time::Duration::from_secs(1));

        assert_eq!(
            Some(CloseReason::OwnerRuntimeShutdown),
            connection.probe().close_reason
        );
        drop(selection);
        assert_eq!((0, 0), cell.h1_counts());
    }

    #[test]
    fn three_waiters_receive_capacity_in_fifo_order() {
        let (_admission, cell) = bounded_cell();
        let first = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let third = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        let first_lease = OriginCell::take_ready_lease(&cell, first).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, second).is_none());
        assert!(OriginCell::take_ready_lease(&cell, third).is_none());

        drop(first_lease);
        let second_lease = OriginCell::take_ready_lease(&cell, second).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, third).is_none());

        drop(second_lease);
        let third_lease = OriginCell::take_ready_lease(&cell, third).unwrap();
        drop(third_lease);
        assert_eq!(0, cell.probe().retained);
    }

    #[test]
    fn cancelling_middle_and_tail_waiters_repairs_fifo_links() {
        let (_admission, cell, held) = saturated_bounded_cell();
        let first = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
        let third = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let fourth = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let demand = cell.probe().demand;

        assert!(OriginCell::cancel_waiter(&cell, second));
        assert!(OriginCell::cancel_waiter(&cell, fourth));
        assert_eq!(demand, cell.probe().demand);
        assert_eq!(2, cell.probe().waiting);
        let fifth = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        assert_eq!(3, cell.probe().waiting);

        drop(held);
        let first_lease = OriginCell::take_ready_lease(&cell, first).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, third).is_none());
        assert!(OriginCell::take_ready_lease(&cell, fifth).is_none());
        drop(first_lease);

        let third_lease = OriginCell::take_ready_lease(&cell, third).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, fifth).is_none());
        drop(third_lease);

        drop(OriginCell::take_ready_lease(&cell, fifth).unwrap());
        assert_eq!(0, cell.probe().retained);
    }

    #[test]
    fn cancelling_head_waiter_preserves_remaining_fifo_order() {
        let (_admission, cell, held) = saturated_bounded_cell();
        let first = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let third = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let fourth = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        assert!(OriginCell::cancel_waiter(&cell, first));
        drop(held);

        let second_lease = OriginCell::take_ready_lease(&cell, second).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, third).is_none());
        assert!(OriginCell::take_ready_lease(&cell, fourth).is_none());
        drop(second_lease);

        let third_lease = OriginCell::take_ready_lease(&cell, third).unwrap();
        assert!(OriginCell::take_ready_lease(&cell, fourth).is_none());
        drop(third_lease);

        drop(OriginCell::take_ready_lease(&cell, fourth).unwrap());
        assert_eq!(0, cell.probe().retained);
    }

    #[test]
    fn cancelling_ready_waiter_refunnels_capacity() {
        let (_admission, cell) = bounded_cell();
        let first = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

        assert!(OriginCell::cancel_waiter(&cell, first));
        let lease = OriginCell::take_ready_lease(&cell, second).unwrap();
        drop(lease);
    }

    #[test]
    fn cancelling_during_delivery_refunnels_capacity_after_unlock() {
        let (admission, cell) = bounded_cell();
        let (waiter, demand) =
            cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let mut delivery =
            OriginAdmission::submit_without_running(&admission, cell.id().partition(), demand)
                .expect("published demand did not reserve capacity");
        assert!(delivery.resolve_payload_for_test());
        let reservation = {
            let mut state = cell.state.lock();
            state
                .acquisitions
                .reserve_delivery_waiter(delivery.demand(), &cell.eligibility_group)
        };
        let DeliveryReservation::Reserved {
            waiter: reserved,
            successor,
        } = reservation
        else {
            panic!("current delivery was rejected");
        };
        assert_eq!(waiter, reserved);
        let (event, settlement) = delivery.into_step(successor);
        let AcquisitionStep::StartEstablishment(permit) = event else {
            panic!("capacity delivery materialized a non-capacity event");
        };

        assert!(OriginCell::cancel_waiter(&cell, waiter));
        let commitment = {
            let mut state = cell.state.lock();
            state.acquisitions.commit_capacity(waiter, permit)
        };
        let CellCommitOutcome::Refused { returned, .. } = commitment else {
            panic!("cancelled delivery accepted its capacity");
        };
        assert!(returned[0].is_none());
        assert!(matches!(
            returned[1],
            Some(AcquisitionStep::StartEstablishment(_))
        ));
        assert_eq!(0, cell.probe().retained);

        drop(returned);
        assert_eq!(1, admission.available_capacity_for_test());
        assert!(settlement.accept().is_none());
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn h1_return_during_capacity_delivery_refunnels_the_permit() {
        let (admission, cell) = bounded_cell_with_limit(3);
        let first_lease = OriginAdmission::lease_for_test(&admission);
        let (first_connection, _first_physical) =
            ConnectionState::bounded(connection_info(1), first_lease);
        let first_return =
            OriginCell::insert_selected_h1(&cell, first_connection, H1Sender::test(11));
        let second_lease = OriginAdmission::lease_for_test(&admission);
        let (second_connection, _second_physical) =
            ConnectionState::bounded(connection_info(2), second_lease);
        let second_return =
            OriginCell::insert_selected_h1(&cell, second_connection, H1Sender::test(22));
        let (waiter, demand) =
            cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let mut delivery =
            OriginAdmission::submit_without_running(&admission, cell.id().partition(), demand)
                .expect("published demand did not reserve capacity");
        assert!(delivery.resolve_payload_for_test());
        let reservation = {
            let mut state = cell.state.lock();
            state
                .acquisitions
                .reserve_delivery_waiter(delivery.demand(), &cell.eligibility_group)
        };
        let DeliveryReservation::Reserved {
            waiter: reserved,
            successor,
        } = reservation
        else {
            panic!("current delivery was rejected");
        };
        assert_eq!(waiter, reserved);

        drop(first_return);
        drop(second_return);
        assert_eq!((2, 1), cell.h1_counts());
        let (event, settlement) = delivery.into_step(successor);
        let AcquisitionStep::StartEstablishment(permit) = event else {
            panic!("capacity delivery materialized a non-capacity event");
        };
        let commitment = {
            let mut state = cell.state.lock();
            state.acquisitions.commit_capacity(waiter, permit)
        };
        let CellCommitOutcome::Refused { returned, .. } = commitment else {
            panic!("capacity delivery displaced a local HTTP/1 result");
        };
        assert!(returned[0].is_some());
        drop(returned);
        assert!(settlement.accept().is_none());

        assert_eq!(1, admission.available_capacity_for_test());
        let selected = cell
            .take_ready_h1(waiter)
            .expect("returning H1 did not win the crossing delivery");
        assert_eq!(11, selected.test_sender_id());
        drop(selected);
    }

    #[test]
    fn cancellation_during_h1_and_capacity_crossing_returns_both() {
        let (admission, cell) = bounded_cell_with_limit(2);
        let installed_lease = OriginAdmission::lease_for_test(&admission);
        let (connection, _physical) = ConnectionState::bounded(connection_info(1), installed_lease);
        let returning = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
        let (waiter, demand) =
            cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let mut delivery =
            OriginAdmission::submit_without_running(&admission, cell.id().partition(), demand)
                .expect("published demand did not reserve capacity");
        assert!(delivery.resolve_payload_for_test());
        let reservation = {
            let mut state = cell.state.lock();
            state
                .acquisitions
                .reserve_delivery_waiter(delivery.demand(), &cell.eligibility_group)
        };
        let DeliveryReservation::Reserved {
            waiter: reserved,
            successor,
        } = reservation
        else {
            panic!("current delivery was rejected");
        };
        assert_eq!(waiter, reserved);

        drop(returning);
        assert!(OriginCell::cancel_waiter(&cell, waiter));
        let (event, settlement) = delivery.into_step(successor);
        let AcquisitionStep::StartEstablishment(permit) = event else {
            panic!("capacity delivery materialized a non-capacity event");
        };
        let commitment = {
            let mut state = cell.state.lock();
            state.acquisitions.commit_capacity(waiter, permit)
        };
        let CellCommitOutcome::Refused { returned, .. } = commitment else {
            panic!("cancelled crossing retained one of its returned resources");
        };
        assert!(returned.iter().all(Option::is_some));
        drop(returned);
        assert!(settlement.accept().is_none());

        assert_eq!(1, admission.available_capacity_for_test());
        assert_eq!(0, cell.probe().retained);
        assert_eq!((1, 1), cell.h1_counts());
        assert_eq!(
            11,
            OriginCell::select_h1(&cell)
                .expect("cancelled crossing lost its H1 sender")
                .test_sender_id()
        );
    }

    #[test]
    fn stale_delivery_is_rejected_after_head_cancellation() {
        let (admission, cell) = bounded_cell();
        let (first, first_demand) =
            cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let stale = OriginAdmission::submit_without_running(
            &admission,
            cell.id().partition(),
            first_demand,
        )
        .expect("first demand did not reserve capacity");

        assert!(OriginCell::cancel_waiter(&cell, first));
        OriginAdmission::run_action_chain(Some(AdmissionAction::Deliver(stale)));

        let lease = OriginCell::take_ready_lease(&cell, second)
            .expect("replacement demand did not receive refunnelled capacity");
        drop(lease);
    }

    #[test]
    fn stale_delivery_returns_its_successor_to_the_driver() {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let stale = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        );
        let target = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(2),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        );

        let (stale_waiter, stale_demand) =
            stale.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let mut delivery = OriginAdmission::submit_without_running(
            &admission,
            stale.id().partition(),
            stale_demand,
        )
        .expect("stale demand did not reserve capacity");
        let (target_waiter, target_demand) =
            target.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        OriginAdmission::submit_demand_snapshot(&admission, target.id().partition(), target_demand);

        // Model cancellation winning in the cell before its retirement
        // publication reaches admission.
        let cancelled = stale
            .state
            .lock()
            .acquisitions
            .cancel_waiter(stale_waiter, &stale.eligibility_group)
            .expect("stale waiter was not cancelled");
        drop(cancelled.returned_steps);

        assert!(delivery.resolve_payload_for_test());
        let next = OriginCell::receive_delivery(&stale, delivery)
            .expect("stale rejection did not return the successor delivery");
        assert!(
            OriginCell::take_ready_lease(&target, target_waiter).is_none(),
            "stale rejection recursively drove its successor"
        );

        OriginAdmission::run_action_chain(Some(next));
        drop(
            OriginCell::take_ready_lease(&target, target_waiter)
                .expect("existing driver did not deliver the successor"),
        );
    }

    #[test]
    fn cancelling_the_only_waiter_retires_admission_demand() {
        let (admission, cell, held) = saturated_bounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        assert_eq!(1, admission.ordered_demand_count_for_test());

        assert!(OriginCell::cancel_waiter(&cell, waiter));
        assert_eq!(0, admission.ordered_demand_count_for_test());
        drop(held);
    }

    #[test]
    fn unknown_waiter_cancellation_is_a_noop() {
        let (_admission, cell) = bounded_cell();
        assert!(!OriginCell::cancel_waiter(&cell, WaiterId(99)));
    }

    #[test]
    fn unbounded_waiter_registers_and_cancels_locally() {
        let cell = Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ));

        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        assert_eq!(0, cell.probe().waiting);
        assert_eq!(1, cell.probe().retained);
        assert!(OriginCell::cancel_waiter(&cell, waiter));
        assert_eq!(0, cell.probe().retained);
    }

    #[test]
    fn delivered_waiter_is_woken_once() {
        let (_admission, cell, held) = saturated_bounded_cell();
        let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let counter = StdArc::new(WakeCounter(AtomicUsize::new(0)));
        let waker = Waker::from(counter.clone());
        let mut context = Context::from_waker(&waker);
        assert!(cell.poll_waiter(waiter, &mut context).is_pending());

        drop(held);
        assert_eq!(1, counter.0.load(Ordering::Relaxed));
        drop(OriginCell::take_ready_lease(&cell, waiter).expect("woken waiter had no capacity"));
        assert_eq!(1, counter.0.load(Ordering::Relaxed));
    }

    fn install_bounded_h2(
        admission: &Arc<OriginAdmission>,
        cell: &Arc<OriginCell>,
        connection_id: u64,
    ) -> (
        h2::H2GenerationId,
        Arc<ConnectionState>,
        super::super::connection::PhysicalConnectionGuard,
    ) {
        let lease = OriginAdmission::lease_for_test(admission);
        let (connection, physical) = ConnectionState::bounded(
            ConnectionInfo::for_test(ConnectionId::new(connection_id), cell.id().partition()),
            lease,
        );
        let generation =
            OriginCell::install_h2_for_test(cell, connection.clone(), connection_id, None);
        (generation, connection, physical)
    }

    fn take_ready_h2(cell: &OriginCell, waiter: WaiterId) -> h2::H2Activation {
        match cell
            .take_ready_event(waiter)
            .expect("HTTP/2 waiter did not receive an activation")
        {
            AcquisitionStep::Resolved(AcquisitionOutcome::H2(activation)) => activation,
            AcquisitionStep::StartEstablishment(_) => {
                panic!("HTTP/2 waiter received establishment capacity")
            }
            AcquisitionStep::Resolved(AcquisitionOutcome::H1(_)) => {
                panic!("HTTP/2 waiter received an HTTP/1 sender")
            }
            AcquisitionStep::Resolved(AcquisitionOutcome::RetryAcquisition) => {
                panic!("HTTP/2 waiter was returned for reacquisition")
            }
            AcquisitionStep::Resolved(AcquisitionOutcome::Failed(error)) => {
                panic!("HTTP/2 waiter received an establishment failure: {error}")
            }
        }
    }

    #[test]
    fn eligible_peer_h2_generation_satisfies_bounded_demand() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);

        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);
        let activation = take_ready_h2(&requesting_cell, waiter);

        assert_eq!(generation, activation.generation());
        assert!(Arc::ptr_eq(&connection, activation.connection()));
        assert_eq!(0, admission.available_capacity_for_test());
        assert_eq!(Some(generation), connection_cell.accepting_h2_generation());
        drop(activation);
        assert_eq!(0, requesting_cell.probe().retained);
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn prepared_h2_route_executes_and_settles_assignment() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        )
        .expect("peer demand did not prepare an HTTP/2 route");

        OriginAdmission::run_action_chain(Some(action));
        let activation = take_ready_h2(&requesting_cell, waiter);
        assert_eq!(generation, activation.generation());
        drop(activation);

        assert_eq!(0, requesting_cell.probe().retained);
        assert_eq!(0, admission.ordered_demand_count_for_test());
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn peer_h2_route_respects_eligibility_group() {
        let groups = [
            (
                EligibilityGroup::Partition(PartitionId::from_index(1)),
                EligibilityGroup::Partition(PartitionId::from_index(2)),
            ),
            (
                EligibilityGroup::NetworkInterface(Some(Arc::from("eth0"))),
                EligibilityGroup::NetworkInterface(Some(Arc::from("eth1"))),
            ),
        ];
        for (connection_group, requesting_group) in groups {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(1, connection_group, requesting_group);
            let (generation, _connection, _physical) =
                install_bounded_h2(&admission, &connection_cell, 1);
            let waiter =
                OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);

            assert!(requesting_cell.take_ready_event(waiter).is_none());
            assert_eq!(1, admission.ordered_demand_count_for_test());
            assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
            assert!(OriginCell::close_h2(
                &connection_cell,
                generation,
                CloseReason::PoolDropped,
            ));
            assert_eq!(1, admission.available_capacity_for_test());
        }
    }

    #[test]
    fn dropped_route_guard_retries_the_same_demand() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        )
        .expect("peer demand did not prepare a route");

        drop(action);

        drop(take_ready_h2(&requesting_cell, waiter));
        assert_eq!(0, admission.ordered_demand_count_for_test());
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }

    #[test]
    fn requesting_cell_cancellation_closes_an_in_flight_route_assignment() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        )
        .expect("peer demand did not prepare a route");

        assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
        OriginAdmission::run_action_chain(Some(action));

        assert_eq!(0, requesting_cell.probe().retained);
        assert_eq!(0, admission.ordered_demand_count_for_test());
        assert_eq!(0, admission.available_capacity_for_test());
        assert_eq!(Some(generation), connection_cell.accepting_h2_generation());
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }

    #[test]
    fn stale_route_retries_against_a_replacement_generation() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (first, _first_connection, _first_physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        )
        .expect("peer demand did not prepare a route");

        assert!(OriginCell::close_h2(
            &connection_cell,
            first,
            CloseReason::ProtocolClosed
        ));
        let (second, _second_connection, _second_physical) =
            install_bounded_h2(&admission, &connection_cell, 2);
        OriginAdmission::run_action_chain(Some(action));

        let activation = take_ready_h2(&requesting_cell, waiter);
        assert_eq!(second, activation.generation());
        drop(activation);
        assert!(OriginCell::close_h2(
            &connection_cell,
            second,
            CloseReason::PoolDropped
        ));
    }

    #[test]
    fn stale_peer_route_republishes_demand_to_a_replacement_generation() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (first, _first_connection, _first_physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        assert!(OriginCell::attach_h2_route(
            &requesting_cell,
            h2::H2Route::new(&connection_cell, first),
            &EligibilityGroup::Pool,
            demand.id_for_test(),
        ));

        assert!(OriginCell::close_h2(
            &connection_cell,
            first,
            CloseReason::ProtocolClosed
        ));
        let (second, _second_connection, _second_physical) =
            install_bounded_h2(&admission, &connection_cell, 2);
        OriginCell::offer_peer_h2(&requesting_cell);
        let activation = take_ready_h2(&requesting_cell, waiter);
        assert_eq!(second, activation.generation());
        drop(activation);
        assert!(OriginCell::close_h2(
            &connection_cell,
            second,
            CloseReason::PoolDropped
        ));
    }

    #[test]
    fn direct_selection_removes_a_stale_peer_route() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);
        drop(take_ready_h2(&requesting_cell, waiter));

        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::ProtocolClosed,
        ));
        assert!(OriginCell::select_h2(&requesting_cell).is_none());
        assert!(
            !requesting_cell.state.lock().h2.has_visible_h2(),
            "direct selection retained a stale peer route"
        );
    }

    #[test]
    fn open_peer_route_allows_concurrent_prospective_activations() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);
        drop(take_ready_h2(&requesting_cell, waiter));

        let first = OriginCell::select_h2(&requesting_cell)
            .expect("first direct peer activation was not selected");
        let second = OriginCell::select_h2(&requesting_cell)
            .expect("open peer route serialized the second activation");
        assert_eq!(generation, first.generation());
        assert_eq!(generation, second.generation());
        assert_eq!(Some((2, 0)), connection_cell.h2_request_counts(generation));
        drop(first);
        drop(second);
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped
        ));
    }

    #[test]
    fn peer_generation_gate_preserves_pre_route_order() {
        let (admission, connection_cell, requesting_cell) =
            bounded_peer_cells(1, EligibilityGroup::Pool, EligibilityGroup::Pool);
        let (generation, _connection, _physical) =
            install_bounded_h2(&admission, &connection_cell, 1);
        let (first, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
        let second = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        )
        .expect("peer demand did not prepare a route");
        OriginAdmission::run_action_chain(Some(action));
        let third = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H2Required);

        let first_activation = take_ready_h2(&requesting_cell, first);
        assert!(requesting_cell.take_ready_event(second).is_none());
        assert!(requesting_cell.take_ready_event(third).is_none());
        drop(first_activation);

        let second_activation = take_ready_h2(&requesting_cell, second);
        assert!(requesting_cell.take_ready_event(third).is_none());
        drop(second_activation);
        drop(take_ready_h2(&requesting_cell, third));
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }
}

#[cfg(all(test, smithy_http_client_loom))]
mod loom_tests {
    use super::*;
    use crate::sync::{AtomicBool, AtomicUsize, Ordering};
    use http_1x::uri::Scheme;
    use std::num::NonZeroUsize;

    fn bounded_cell() -> (Arc<OriginAdmission>, Arc<OriginCell>) {
        bounded_cell_with_limit(1)
    }

    fn bounded_cell_with_limit(limit: usize) -> (Arc<OriginAdmission>, Arc<OriginCell>) {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(limit).unwrap());
        let candidate = Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            Some(admission.clone()),
            None,
        ));
        let cell = OriginAdmission::register_cell(&admission, candidate);
        (admission, cell)
    }

    fn bounded_peer_cells(
        owning_group: EligibilityGroup,
        requesting_group: EligibilityGroup,
    ) -> (Arc<OriginAdmission>, Arc<OriginCell>, Arc<OriginCell>) {
        bounded_peer_cells_with_limit(1, owning_group, requesting_group)
    }

    fn bounded_peer_cells_with_limit(
        limit: usize,
        owning_group: EligibilityGroup,
        requesting_group: EligibilityGroup,
    ) -> (Arc<OriginAdmission>, Arc<OriginCell>, Arc<OriginCell>) {
        let admission = OriginAdmission::for_test(NonZeroUsize::new(limit).unwrap());
        let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
        let connection_cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                origin.clone(),
                owning_group,
                Some(admission.clone()),
                None,
            )),
        );
        let requesting_cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(2),
                origin,
                requesting_group,
                Some(admission.clone()),
                None,
            )),
        );
        (admission, connection_cell, requesting_cell)
    }

    fn unbounded_cell() -> Arc<OriginCell> {
        Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ))
    }

    fn connection_info(id: u64) -> Arc<ConnectionInfo> {
        ConnectionInfo::for_test(ConnectionId::new(id), PartitionId::from_index(1))
    }

    /// Races local HTTP/1 selection with close.
    ///
    /// The sender must have one winner and no installed record may survive.
    #[test]
    fn h1_selection_linearizes_against_close() {
        loom::model(|| {
            let cell = unbounded_cell();
            let (connection, _physical) = ConnectionState::unbounded(connection_info(1));
            OriginCell::insert_idle_h1(&cell, connection.clone(), H1Sender::test(11));
            let close = H1CloseHandle::new(&cell, &connection);

            let select_cell = cell.clone();
            let select = loom::thread::spawn(move || OriginCell::select_h1(&select_cell));
            let close = loom::thread::spawn(move || close.close(CloseReason::Poisoned));

            let selected = select.join().unwrap();
            assert!(close.join().unwrap());
            drop(selected);

            assert_eq!((0, 0), cell.h1_counts());
            assert_eq!(Some(CloseReason::Poisoned), connection.probe().close_reason);
        });
    }

    /// Races a reusable HTTP/1 return with close.
    ///
    /// Close must prevent the returning sender from becoming selectable again.
    #[test]
    fn h1_return_linearizes_against_close() {
        loom::model(|| {
            let cell = unbounded_cell();
            let (connection, _physical) = ConnectionState::unbounded(connection_info(1));
            let selection =
                OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::test(11));
            let exchange = selection.into_exchange();
            let close = H1CloseHandle::new(&cell, &connection);

            let returning = loom::thread::spawn(move || exchange.offer_for_reuse());
            let closing = loom::thread::spawn(move || close.close(CloseReason::Poisoned));
            returning.join().unwrap();
            assert!(closing.join().unwrap());

            assert_eq!((0, 0), cell.h1_counts());
            assert_eq!(Some(CloseReason::Poisoned), connection.probe().close_reason);
        });
    }

    /// Races local sender delivery with cancellation of its target waiter.
    ///
    /// Cancellation may win, but the exclusive sender must return exactly once.
    #[test]
    fn h1_delivery_and_waiter_cancellation_preserve_the_sender() {
        loom::model(|| {
            let cell = unbounded_cell();
            let (connection, _physical) = ConnectionState::unbounded(connection_info(1));
            let selection = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
            let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);

            let returning = loom::thread::spawn(move || drop(selection));
            let cancel_cell = cell.clone();
            let cancelling =
                loom::thread::spawn(move || OriginCell::cancel_waiter(&cancel_cell, waiter));
            returning.join().unwrap();
            assert!(cancelling.join().unwrap());

            assert_eq!((1, 1), cell.h1_counts());
            assert_eq!(0, cell.probe().retained);
            assert_eq!(
                11,
                OriginCell::select_h1(&cell)
                    .expect("sender was lost during return/cancel")
                    .test_sender_id()
            );
        });
    }

    /// Races an idle return with completion of an already-started establishment.
    ///
    /// Exactly one result satisfies the waiter and the losing sender remains reusable.
    #[test]
    fn returned_h1_and_establishment_race_for_one_waiter() {
        loom::model(|| {
            let cell = unbounded_cell();
            let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
            let AcquisitionStep::StartEstablishment(permit) = cell
                .take_ready_event(waiter)
                .expect("unbounded miss did not start establishment")
            else {
                panic!("unbounded miss completed before establishment started");
            };
            assert!(cell.start_establishment(waiter));
            drop(permit);

            let (returning_connection, _returning_physical) =
                ConnectionState::unbounded(connection_info(1));
            let returning =
                OriginCell::insert_selected_h1(&cell, returning_connection, H1Sender::test(11));
            let (fresh_connection, _fresh_physical) =
                ConnectionState::unbounded(connection_info(2));
            let fresh = OriginCell::insert_selected_h1(&cell, fresh_connection, H1Sender::test(22));

            let returning = loom::thread::spawn(move || drop(returning));
            let completion_cell = cell.clone();
            let completing = loom::thread::spawn(move || {
                completion_cell.complete_establishment(waiter, AcquisitionOutcome::H1(fresh));
            });
            returning.join().unwrap();
            completing.join().unwrap();

            assert_eq!((2, 1), cell.h1_counts());
            let winner = cell
                .take_ready_h1(waiter)
                .expect("both acquisition outcomes were lost");
            assert!(matches!(winner.test_sender_id(), 11 | 22));
            drop(winner);
            assert_eq!((2, 2), cell.h1_counts());
        });
    }

    /// Races first establishment commitment with a reusable HTTP/1 return.
    ///
    /// A return that wins before commitment must satisfy the waiter without being displaced.
    #[test]
    fn first_establishment_poll_races_returned_h1() {
        loom::model(|| {
            let cell = unbounded_cell();
            let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
            let AcquisitionStep::StartEstablishment(permit) = cell
                .take_ready_event(waiter)
                .expect("unbounded miss did not prepare establishment")
            else {
                panic!("unbounded miss completed before establishment was prepared");
            };

            let (returning_connection, _returning_physical) =
                ConnectionState::unbounded(connection_info(1));
            let returning =
                OriginCell::insert_selected_h1(&cell, returning_connection, H1Sender::test(11));
            let return_thread = loom::thread::spawn(move || drop(returning));
            let start_cell = cell.clone();
            let start_thread = loom::thread::spawn(move || start_cell.start_establishment(waiter));

            return_thread.join().unwrap();
            let started = start_thread.join().unwrap();
            drop(permit);

            if started {
                let (fresh_connection, _fresh_physical) =
                    ConnectionState::unbounded(connection_info(2));
                let fresh =
                    OriginCell::insert_selected_h1(&cell, fresh_connection, H1Sender::test(22));
                cell.complete_establishment(waiter, AcquisitionOutcome::H1(fresh));
            }

            let winner = cell
                .take_ready_h1(waiter)
                .expect("returned H1 was lost at the first-poll boundary");
            assert_eq!(11, winner.test_sender_id());
            drop(winner);
            assert_eq!(
                (usize::from(started) + 1, usize::from(started) + 1),
                cell.h1_counts()
            );
        });
    }

    /// Races local HTTP/1 return with bounded-capacity delivery.
    ///
    /// The sender wins the waiter while the unused capacity returns to admission.
    #[test]
    fn h1_return_and_capacity_delivery_complete_one_waiter() {
        loom::model(|| {
            let (admission, cell) = bounded_cell_with_limit(2);
            let installed_lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) =
                ConnectionState::bounded(connection_info(1), installed_lease);
            let returning = OriginCell::insert_selected_h1(&cell, connection, H1Sender::test(11));
            let (waiter, demand) =
                cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let delivery =
                OriginAdmission::submit_without_running(&admission, cell.id().partition(), demand)
                    .expect("published demand did not reserve capacity");

            let delivering = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(AdmissionAction::Deliver(delivery)));
            });
            let returning = loom::thread::spawn(move || drop(returning));
            delivering.join().unwrap();
            returning.join().unwrap();

            assert_eq!(1, admission.available_capacity_for_test());
            let selection = cell
                .take_ready_h1(waiter)
                .expect("return and capacity delivery lost the HTTP/1 sender");
            assert_eq!(11, selection.test_sender_id());
            drop(selection);
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races a retained peer match with cancellation of its requesting waiter.
    ///
    /// Three preemptions cover either actor entering first and a complete
    /// reservation round trip across the admission and cell locks.
    #[test]
    fn peer_match_and_request_cancellation_preserve_the_sender() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(EligibilityGroup::Pool, EligibilityGroup::Pool);
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            let returning =
                OriginCell::insert_selected_h1(&connection_cell, connection, H1Sender::test(11));
            let waiter =
                OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);

            // Loom's default 4 KiB coroutine stack is too small for the full
            // sender-return and admission-settlement path modeled here.
            let returning = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || drop(returning))
                .unwrap();
            let cancel_requesting_cell = requesting_cell.clone();
            let cancelling = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginCell::cancel_waiter(&cancel_requesting_cell, waiter))
                .unwrap();
            returning.join().unwrap();
            assert!(cancelling.join().unwrap());

            assert_eq!((1, 1), connection_cell.h1_counts());
            assert_eq!(0, requesting_cell.probe().retained);
            assert_eq!(0, admission.ordered_demand_count_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races a busy idle-probe reply with cancellation of its demand.
    ///
    /// The supplier returns to its index, no fallback reservation survives,
    /// and the active sender remains owned by its connection cell.
    #[test]
    fn busy_idle_probe_and_cancellation_leave_no_retained_match() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let admission = OriginAdmission::for_test(NonZeroUsize::new(2).unwrap());
            let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
            let first = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(1),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let second = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(2),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let requesting_cell = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(3),
                    origin,
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let first_lease = OriginAdmission::lease_for_test(&admission);
            let (first_connection, _first_physical) =
                ConnectionState::bounded(connection_info(1), first_lease);
            let first_returning =
                OriginCell::insert_selected_h1(&first, first_connection, H1Sender::test(11));
            let second_lease = OriginAdmission::lease_for_test(&admission);
            let (second_connection, _second_physical) =
                ConnectionState::bounded(connection_info(2), second_lease);
            let second_returning =
                OriginCell::insert_selected_h1(&second, second_connection, H1Sender::test(22));
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let action = OriginAdmission::submit_action_without_running(
                &admission,
                requesting_cell.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare an idle probe");

            let probing = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginAdmission::run_action_chain(Some(action)))
                .unwrap();
            let cancel_cell = requesting_cell.clone();
            let cancelling = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginCell::cancel_waiter(&cancel_cell, waiter))
                .unwrap();
            probing.join().unwrap();
            assert!(cancelling.join().unwrap());
            drop(first_returning);
            drop(second_returning);

            assert_eq!(vec![11], first.h1_idle_sender_ids());
            assert_eq!(vec![22], second.h1_idle_sender_ids());
            assert!(first.state.lock().h1.peer_reservation_available());
            assert!(second.state.lock().h1.peer_reservation_available());
            assert_eq!(0, admission.ordered_demand_count_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races cancellation with a later idle supplier winning bounded probing.
    ///
    /// Whether cancellation or candidate delivery wins, both sender identities
    /// return to their owning cells exactly once. Cancellation returns `true`
    /// when it removes either a waiting record or a ready, unclaimed selection;
    /// in the latter case it also owns cleanup of that selection.
    #[test]
    fn later_idle_probe_and_cancellation_preserve_each_sender() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let admission = OriginAdmission::for_test(NonZeroUsize::new(2).unwrap());
            let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
            let first = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(1),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let second = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(2),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let requester = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(3),
                    origin,
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let first_lease = OriginAdmission::lease_for_test(&admission);
            let (first_connection, _first_physical) =
                ConnectionState::bounded(connection_info(1), first_lease);
            let returning =
                OriginCell::insert_selected_h1(&first, first_connection, H1Sender::test(11));
            let second_lease = OriginAdmission::lease_for_test(&admission);
            let (second_connection, _second_physical) =
                ConnectionState::bounded(connection_info(2), second_lease);
            OriginCell::insert_idle_h1(&second, second_connection, H1Sender::test(22));
            let (waiter, demand) =
                requester.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let action = OriginAdmission::submit_action_without_running(
                &admission,
                requester.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare an idle probe");

            let probing = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginAdmission::run_action_chain(Some(action)))
                .unwrap();
            let cancel_cell = requester.clone();
            let cancelling = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginCell::cancel_waiter(&cancel_cell, waiter))
                .unwrap();
            probing.join().unwrap();
            assert!(cancelling.join().unwrap());
            drop(returning);

            let mut sender_ids = first.h1_idle_sender_ids();
            sender_ids.extend(second.h1_idle_sender_ids());
            sender_ids.extend(requester.h1_idle_sender_ids());
            sender_ids.sort_unstable();
            assert_eq!(vec![11, 22], sender_ids);
            assert!(first.state.lock().h1.peer_reservation_available());
            assert!(second.state.lock().h1.peer_reservation_available());
            assert_eq!(0, admission.ordered_demand_count_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races cancellation with the second idle probe and exact fallback reservation.
    ///
    /// Three busy suppliers force two distinct idle-only probes followed by
    /// reservation of the third supplier. Cancellation begins after the second
    /// probe is selected, so every explored schedule crosses the full bounded
    /// probe path while racing its final supplier transition.
    #[test]
    fn second_idle_probe_and_cancellation_preserve_all_senders() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let admission = OriginAdmission::for_test(NonZeroUsize::new(3).unwrap());
            let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
            let first = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(1),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let second = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(2),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let third = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(3),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let requester = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(4),
                    origin,
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );

            let first_lease = OriginAdmission::lease_for_test(&admission);
            let (first_connection, _first_physical) =
                ConnectionState::bounded(connection_info(1), first_lease);
            let first_returning =
                OriginCell::insert_selected_h1(&first, first_connection, H1Sender::test(11));
            let second_lease = OriginAdmission::lease_for_test(&admission);
            let (second_connection, _second_physical) =
                ConnectionState::bounded(connection_info(2), second_lease);
            let second_returning =
                OriginCell::insert_selected_h1(&second, second_connection, H1Sender::test(22));
            let third_lease = OriginAdmission::lease_for_test(&admission);
            let (third_connection, _third_physical) =
                ConnectionState::bounded(connection_info(3), third_lease);
            let third_returning =
                OriginCell::insert_selected_h1(&third, third_connection, H1Sender::test(33));

            let (waiter, demand) =
                requester.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let first_probe = OriginAdmission::submit_action_without_running(
                &admission,
                requester.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare its first idle probe");
            assert!(matches!(&first_probe, AdmissionAction::ProbeH1Supplier(_)));

            let probes = Arc::new(AtomicUsize::new(0));
            let second_probe_started = Arc::new(AtomicBool::new(false));
            let probing_count = probes.clone();
            let probing_started = second_probe_started.clone();
            let probing = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || {
                    probing_count.fetch_add(1, Ordering::SeqCst);
                    let second_probe = first_probe
                        .run_once_for_test()
                        .expect("first miss did not prepare a second idle probe");
                    assert!(matches!(&second_probe, AdmissionAction::ProbeH1Supplier(_)));
                    probing_count.fetch_add(1, Ordering::SeqCst);
                    probing_started.store(true, Ordering::SeqCst);
                    let fallback = second_probe.run_once_for_test();
                    OriginAdmission::run_action_chain(fallback);
                })
                .unwrap();
            let cancel_cell = requester.clone();
            let cancelling = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || {
                    while !second_probe_started.load(Ordering::SeqCst) {
                        loom::thread::yield_now();
                    }
                    OriginCell::cancel_waiter(&cancel_cell, waiter)
                })
                .unwrap();
            probing.join().unwrap();
            assert!(cancelling.join().unwrap());
            assert_eq!(2, probes.load(Ordering::SeqCst));

            drop(first_returning);
            drop(second_returning);
            drop(third_returning);

            let mut sender_ids = first.h1_idle_sender_ids();
            sender_ids.extend(second.h1_idle_sender_ids());
            sender_ids.extend(third.h1_idle_sender_ids());
            sender_ids.extend(requester.h1_idle_sender_ids());
            sender_ids.sort_unstable();
            assert_eq!(vec![11, 22, 33], sender_ids);
            assert!(first.state.lock().h1.peer_reservation_available());
            assert!(second.state.lock().h1.peer_reservation_available());
            assert!(third.state.lock().h1.peer_reservation_available());
            assert_eq!(0, admission.ordered_demand_count_for_test());
            assert_eq!(0, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races a probed supplier return with installation of the fallback reservation.
    ///
    /// The first supplier is busy when probed. Its sender returns after the
    /// miss while admission installs the exact reservation on the second
    /// supplier. The fallback must satisfy the waiter without losing,
    /// duplicating, or reserving the returning sender.
    #[test]
    fn supplier_return_between_probe_miss_and_fallback_preserves_both_senders() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let admission = OriginAdmission::for_test(NonZeroUsize::new(2).unwrap());
            let origin = OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap();
            let first = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(1),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let second = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(2),
                    origin.clone(),
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );
            let requester = OriginAdmission::register_cell(
                &admission,
                Arc::new(OriginCell::new(
                    PartitionId::from_index(3),
                    origin,
                    EligibilityGroup::Pool,
                    Some(admission.clone()),
                    None,
                )),
            );

            let first_lease = OriginAdmission::lease_for_test(&admission);
            let (first_connection, _first_physical) =
                ConnectionState::bounded(connection_info(1), first_lease);
            let returning =
                OriginCell::insert_selected_h1(&first, first_connection, H1Sender::test(11));

            let second_lease = OriginAdmission::lease_for_test(&admission);
            let (second_connection, _second_physical) =
                ConnectionState::bounded(connection_info(2), second_lease);
            OriginCell::insert_idle_h1(&second, second_connection, H1Sender::test(22));

            let (waiter, demand) =
                requester.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let probe = OriginAdmission::submit_action_without_running(
                &admission,
                requester.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare an idle probe");
            let fallback = probe
                .run_once_for_test()
                .expect("busy probe did not prepare the fallback reservation");

            let reserving = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || OriginAdmission::run_action_chain(Some(fallback)))
                .unwrap();
            let returning = loom::thread::Builder::new()
                .stack_size(16 * 1024)
                .spawn(move || drop(returning))
                .unwrap();
            reserving.join().unwrap();
            returning.join().unwrap();

            let selected = requester
                .take_ready_h1(waiter)
                .expect("fallback reservation did not satisfy the waiter");
            assert_eq!(22, selected.test_sender_id());
            drop(selected);

            let mut sender_ids = first.h1_idle_sender_ids();
            sender_ids.extend(second.h1_idle_sender_ids());
            sender_ids.extend(requester.h1_idle_sender_ids());
            sender_ids.sort_unstable();
            assert_eq!(vec![11, 22], sender_ids);
            assert!(first.state.lock().h1.peer_reservation_available());
            assert!(second.state.lock().h1.peer_reservation_available());
            assert_eq!(0, admission.ordered_demand_count_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races borrowed-sender delivery with a local sender return.
    ///
    /// Both exclusive sender identities must remain installed exactly once.
    /// Three preemptions cover either actor entering first and one crossing
    /// returning through both lock domains.
    #[test]
    fn borrowed_delivery_racing_a_local_return_preserves_both_senders() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells_with_limit(2, EligibilityGroup::Pool, EligibilityGroup::Pool);
            let owning_lease = OriginAdmission::lease_for_test(&admission);
            let (owned_connection, _owning_physical) =
                ConnectionState::bounded(connection_info(1), owning_lease);
            OriginCell::insert_idle_h1(&connection_cell, owned_connection, H1Sender::test(11));
            let requesting_lease = OriginAdmission::lease_for_test(&admission);
            let (requesting_connection, _requesting_physical) =
                ConnectionState::bounded(connection_info(2), requesting_lease);
            let local = OriginCell::insert_selected_h1(
                &requesting_cell,
                requesting_connection,
                H1Sender::test(22),
            );
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let install = OriginAdmission::submit_action_without_running(
                &admission,
                requesting_cell.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare an H1 match");
            let delivering = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(install));
            });
            let returning = loom::thread::spawn(move || drop(local));
            delivering.join().unwrap();
            returning.join().unwrap();

            let selected = requesting_cell
                .take_ready_h1(waiter)
                .expect("delivery/return race did not satisfy the requesting cell waiter");
            drop(selected);
            let mut sender_ids = connection_cell.h1_idle_sender_ids();
            sender_ids.extend(requesting_cell.h1_idle_sender_ids());
            sender_ids.sort_unstable();
            assert_eq!(
                vec![11, 22],
                sender_ids,
                "delivery/return race duplicated or lost an HTTP/1 sender"
            );
            assert_eq!(
                2,
                connection_cell.h1_counts().0 + requesting_cell.h1_counts().0
            );
            assert_eq!(
                2,
                connection_cell.h1_counts().1 + requesting_cell.h1_counts().1
            );
            assert!(connection_cell.state.lock().h1.peer_reservation_available());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races peer-sender materialization with close of the owning cell record.
    ///
    /// The request must receive the sender or establishment capacity, never
    /// become stranded. Three preemptions cover the delivery-close-delivery
    /// schedule across the two cell locks and admission lock.
    #[test]
    fn borrow_materialization_races_owning_cell_close_without_stranding_request() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(EligibilityGroup::Pool, EligibilityGroup::Pool);
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            OriginCell::insert_idle_h1(&connection_cell, connection.clone(), H1Sender::test(11));
            let close = H1CloseHandle::new(&connection_cell, &connection);
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let install = OriginAdmission::submit_action_without_running(
                &admission,
                requesting_cell.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare an H1 match");
            let delivering = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(install));
            });
            let closing = loom::thread::spawn(move || close.close(CloseReason::Poisoned));
            delivering.join().unwrap();
            closing.join().unwrap();

            let event = requesting_cell
                .take_ready_event(waiter)
                .expect("borrow/close race stranded its requesting cell waiter");
            match event {
                AcquisitionStep::Resolved(AcquisitionOutcome::H1(selection)) => {
                    drop(selection);
                }
                AcquisitionStep::StartEstablishment(permit) => {
                    assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
                    drop(permit);
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::Failed(error)) => {
                    panic!("unexpected establishment failure: {error}")
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::RetryAcquisition) => {
                    panic!("HTTP/1 reuse model requested reacquisition")
                }
                AcquisitionStep::Resolved(AcquisitionOutcome::H2(_)) => {
                    panic!("HTTP/1 reuse model received an HTTP/2 activation")
                }
            }
            assert_eq!(0, requesting_cell.probe().retained);
            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races HTTP/1 capacity reclaim with independent close of the same connection.
    ///
    /// Capacity must return exactly once. Three preemptions cover close and
    /// reclaim entering in either order and the resulting admission delivery.
    #[test]
    fn reclaim_and_connection_close_release_exactly_one_capacity_slot() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) = bounded_peer_cells(
                EligibilityGroup::Partition(PartitionId::from_index(1)),
                EligibilityGroup::Partition(PartitionId::from_index(2)),
            );
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            let returning = OriginCell::insert_selected_h1(
                &connection_cell,
                connection.clone(),
                H1Sender::test(11),
            );
            let close = H1CloseHandle::new(&connection_cell, &connection);
            let waiter =
                OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Compatible);
            let local_waiter =
                OriginCell::register_waiter(&connection_cell, ProtocolRequirement::H1Compatible);

            // Loom's default 4 KiB coroutine stack is too small for the full
            // sender-return and connection-close paths modeled here.
            let returning = loom::thread::Builder::new()
                .name("returning H1 sender".into())
                .stack_size(16 * 1024)
                .spawn(move || drop(returning))
                .unwrap();
            let closing = loom::thread::Builder::new()
                .name("closing H1 connection".into())
                .stack_size(16 * 1024)
                .spawn(move || close.close(CloseReason::Poisoned))
                .unwrap();
            returning.join().unwrap();
            let close_won = closing.join().unwrap();

            assert_eq!(
                !close_won,
                connection_cell.state.lock().h1.local_turn_owed(),
                "only a completed reclaim may earn the connection-owning cell a local fairness turn"
            );

            let replacement = OriginCell::take_ready_lease(&requesting_cell, waiter)
                .expect("close/reclaim race did not deliver released capacity");
            assert_eq!((0, 0), connection_cell.h1_counts());
            assert_eq!(0, admission.available_capacity_for_test());
            assert!(OriginCell::cancel_waiter(&connection_cell, local_waiter));
            drop(replacement);
            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races bounded-capacity delivery with cancellation of its target waiter.
    ///
    /// Refusal must refunnel the permit after the cell lock is released.
    #[test]
    fn delivery_and_cancellation_race_refunnels_capacity() {
        loom::model(|| {
            let (admission, cell) = bounded_cell();
            let (first, demand) =
                cell.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
            let delivery =
                OriginAdmission::submit_without_running(&admission, cell.id().partition(), demand)
                    .unwrap();

            let deliver = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(AdmissionAction::Deliver(delivery)));
            });
            let cancel_cell = cell.clone();
            let cancel =
                loom::thread::spawn(move || OriginCell::cancel_waiter(&cancel_cell, first));
            deliver.join().unwrap();
            cancel.join().unwrap();

            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races cell-local peer route attachment with waiter cancellation.
    ///
    /// This model isolates the requesting-cell transition. The production
    /// admission preparation and settlement path is exercised separately above.
    #[test]
    fn h2_route_attachment_linearizes_against_requesting_cell_cancellation() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(EligibilityGroup::Pool, EligibilityGroup::Pool);
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            let generation = OriginCell::install_h2_for_test(&connection_cell, connection, 1, None);
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
            let route = h2::H2Route::new(&connection_cell, generation);

            let install_cell = requesting_cell.clone();
            let installing = loom::thread::spawn(move || {
                OriginCell::attach_h2_route(
                    &install_cell,
                    route,
                    &EligibilityGroup::Pool,
                    demand.id_for_test(),
                )
            });
            let cancel_cell = requesting_cell.clone();
            let cancelling =
                loom::thread::spawn(move || OriginCell::cancel_waiter(&cancel_cell, waiter));
            let _installed = installing.join().unwrap();
            assert!(cancelling.join().unwrap());

            assert_eq!(0, requesting_cell.probe().retained);
            assert_eq!(0, admission.available_capacity_for_test());
            assert!(OriginCell::close_h2(
                &connection_cell,
                generation,
                CloseReason::PoolDropped,
            ));
            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races route publication, generation close, and peer route service.
    ///
    /// The three-preemption bound permits each actor to intervene before route
    /// settlement returns through admission to the requesting cell.
    #[test]
    fn h2_route_acknowledgement_races_generation_close_and_route_service() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(EligibilityGroup::Pool, EligibilityGroup::Pool);
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            let generation = OriginCell::install_h2_for_test(&connection_cell, connection, 1, None);
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
            let action = OriginAdmission::submit_action_without_running(
                &admission,
                requesting_cell.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare a route");

            let publishing = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(action));
            });
            let closing_cell = connection_cell.clone();
            let closing = loom::thread::spawn(move || {
                OriginCell::close_h2(&closing_cell, generation, CloseReason::Poisoned)
            });
            let servicing_cell = requesting_cell.clone();
            let servicing = loom::thread::spawn(move || {
                OriginCell::offer_peer_h2(&servicing_cell);
            });

            publishing.join().unwrap();
            assert!(closing.join().unwrap());
            servicing.join().unwrap();

            assert!(
                OriginCell::cancel_waiter(&requesting_cell, waiter),
                "route acknowledgement lost the live demand"
            );
            assert_eq!(0, requesting_cell.probe().retained);
            assert_eq!(0, admission.ordered_demand_count_for_test());
            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }

    /// Races peer route publication with close of the routed generation.
    ///
    /// Three preemptions cover either actor entering first and the stale route
    /// acknowledgement returning to admission.
    #[test]
    fn h2_route_close_race_preserves_capacity_ownership() {
        let mut model = loom::model::Builder::new();
        model.preemption_bound.get_or_insert(3);
        model.check(|| {
            let (admission, connection_cell, requesting_cell) =
                bounded_peer_cells(EligibilityGroup::Pool, EligibilityGroup::Pool);
            let lease = OriginAdmission::lease_for_test(&admission);
            let (connection, _physical) = ConnectionState::bounded(connection_info(1), lease);
            let generation = OriginCell::install_h2_for_test(&connection_cell, connection, 1, None);
            let (waiter, demand) =
                requesting_cell.register_waiter_without_publish(ProtocolRequirement::H2Required);
            let action = OriginAdmission::submit_action_without_running(
                &admission,
                requesting_cell.id().partition(),
                demand,
            )
            .expect("peer demand did not prepare a route");

            let publishing = loom::thread::spawn(move || {
                OriginAdmission::run_action_chain(Some(action));
            });
            let closing_cell = connection_cell.clone();
            let closing = loom::thread::spawn(move || {
                OriginCell::close_h2(&closing_cell, generation, CloseReason::Poisoned)
            });
            publishing.join().unwrap();
            assert!(closing.join().unwrap());

            assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
            assert_eq!(0, requesting_cell.probe().retained);
            assert_eq!(0, admission.ordered_demand_count_for_test());
            assert_eq!(1, admission.available_capacity_for_test());
            admission.clear_modeled_cells_for_test();
        });
    }
}
