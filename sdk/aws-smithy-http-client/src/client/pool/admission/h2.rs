/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Admission's view of HTTP/2 connection supply for one bounded origin.
//!
//! A partition may send a request on another partition's HTTP/2 connection
//! when both partitions belong to the same eligibility group. The connection
//! cell reports the exact accepting generation to admission. Admission pairs
//! that report with compatible demand and installs an identity-only route in
//! the requesting cell. The connection cell keeps the Hyper request handle,
//! protocol driver, transport, and capacity lease.
//!
//! ```text
//! supplier cell A:   submit accepting generation G
//! admission:         demand from B + supply(A, G) -> assign demand
//! connection cell A: validate that G still accepts requests
//! requesting cell B: install route(A, G)
//! admission:         settle the assigned demand
//! ```
//!
//! The route carries no connection state. Each use upgrades the connection
//! cell reference and revalidates generation `G` before reserving one request
//! stream. A stale generation, cancelled requesting cell, or rejected route
//! settles the assignment so live demand remains schedulable.
//!
//! Idle accepting generations have a second use under an origin connection
//! limit. H1-required demand may reserve one exact idle generation, validate
//! that it is still idle, and close it to return capacity for an HTTP/1
//! connection attempt. A busy generation never enters this reclaim order.
//!
//! Admission, connection-cell, and requesting-cell locks are acquired in
//! separate steps. [`H2RouteGuard`] owns the demand assignment while route
//! installation crosses those steps and settles it on drop.
//! [`H2CapacityReclaim`] provides the fallback for idle-generation reclaim.

use super::{
    AdmissionAction, DemandAssignment, DemandAssignmentId, DemandAssignmentOutcome, DemandId,
    DemandSchedule, IntrusiveLinks, IntrusiveOrder, OriginAdmission, SupplyRevision,
};
use crate::client::pool::cell::h2::{H2GenerationId, H2Route};
use crate::client::pool::cell::OriginCell;
use crate::client::pool::partition::{EligibilityGroup, PartitionId};
use crate::sync::Arc;
use std::collections::{BTreeSet, HashMap};
use std::ops::Bound::{Excluded, Unbounded};

/// Admission-facing HTTP/2 status derived under one cell lock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::client::pool) enum H2SupplyStatus {
    Unavailable,
    Accepting {
        generation: H2GenerationId,
        idle: bool,
    },
}

/// Admission-owned H2 supply index and exact idle-reclaim reservation.
///
/// At every completed transition:
///
/// - each connection cell has at most one retained supply record;
/// - each accepting connection cell is linked once in its eligibility-group order;
/// - each idle accepting generation is linked once in the origin reclaim
///   order unless it owns the active reclaim reservation;
/// - each linked identity names the newest reported generation;
/// - `ready_groups` contains exactly the groups with H2-compatible queued
///   demand and peer supply;
/// - at most one reclaim reservation crosses to a connection cell; and
/// - supply records, prepared routes, and reclaim reservations own no
///   connection capacity or protocol sender.
#[derive(Debug, Default)]
pub(super) struct H2Supply {
    index: H2SupplyIndex,
    /// Exact idle generation currently crossing to its connection cell.
    reclaiming: Option<PreparedH2Reclaim>,
}

#[derive(Debug, Default)]
struct H2SupplyIndex {
    records: HashMap<PartitionId, H2SupplyRecord>,
    suppliers_by_group: HashMap<EligibilityGroup, IntrusiveOrder<PartitionId>>,
    reclaim_order: IntrusiveOrder<PartitionId>,
    route_ready_groups: BTreeSet<EligibilityGroup>,
    last_route_group: Option<EligibilityGroup>,
}

/// Admission's latest complete report for one connection cell.
#[derive(Debug)]
struct H2SupplyRecord {
    /// Reuse group in which this connection may be advertised.
    eligibility_group: EligibilityGroup,
    /// Newest connection-cell report retained by admission.
    revision: u64,
    /// Exact accepting generation named by the report.
    status: H2SupplyStatus,
    /// Intrusive-order residence for an available generation.
    group_index: H2GroupIndexState,
    /// Links in the origin-wide idle-generation order.
    reclaim_links: Option<IntrusiveLinks<PartitionId>>,
}

/// Whether one connection cell occupies its eligibility-group supplier order.
#[derive(Debug, Default)]
enum H2GroupIndexState {
    /// This connection cell has no generation available to peers.
    #[default]
    Unavailable,
    /// The cell is linked in its eligibility-group supplier order.
    Linked {
        /// Intrusive links owned by this residence.
        group: IntrusiveLinks<PartitionId>,
    },
}

impl H2GroupIndexState {
    fn links(&self) -> Option<&IntrusiveLinks<PartitionId>> {
        match self {
            Self::Unavailable => None,
            Self::Linked { group } => Some(group),
        }
    }

    fn links_mut(&mut self) -> &mut IntrusiveLinks<PartitionId> {
        match self {
            Self::Unavailable => panic!("unavailable H2 connection had supplier-order links"),
            Self::Linked { group } => group,
        }
    }
}

/// Identity selected before connection and requesting cell validation.
#[derive(Clone, Debug)]
pub(super) struct PreparedH2Route {
    pub(super) assignment: DemandAssignment,
    /// Cell that owns the routed connection.
    pub(super) supplier: PartitionId,
    /// Exact accepting generation selected from retained supply.
    pub(super) generation: H2GenerationId,
    /// Eligibility group shared by demand and supply.
    pub(super) eligibility_group: EligibilityGroup,
}

/// Exact idle generation selected to release bounded origin capacity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreparedH2Reclaim {
    /// Cell whose H1-required demand caused reclaim.
    pub(super) requester: PartitionId,
    /// Exact demand generation that caused reclaim.
    pub(super) demand: DemandId,
    /// Cell that owns the idle H2 generation.
    pub(super) supplier: PartitionId,
    /// Exact accepting generation observed idle by admission.
    pub(super) generation: H2GenerationId,
    /// Eligibility group retained for supply-index repair.
    pub(super) eligibility_group: EligibilityGroup,
}

/// Candidate selected from stored demand and supplier heads.
struct H2RouteMatch {
    /// Requesting cell selected from the group demand head.
    requester: PartitionId,
    /// Demand generation at that head.
    demand: DemandId,
    /// Peer connection cell selected from the supplier head.
    supplier: PartitionId,
    /// Routed generation in that connection cell.
    generation: H2GenerationId,
    /// Eligibility group whose turn was selected.
    eligibility_group: EligibilityGroup,
}

impl H2SupplyRecord {
    fn generation(&self) -> Option<H2GenerationId> {
        match self.status {
            H2SupplyStatus::Unavailable => None,
            H2SupplyStatus::Accepting { generation, .. } => Some(generation),
        }
    }

    fn idle(&self) -> bool {
        matches!(self.status, H2SupplyStatus::Accepting { idle: true, .. })
    }
}

impl H2SupplyIndex {
    fn has_route_ready_group(&self) -> bool {
        !self.route_ready_groups.is_empty()
    }

    fn apply_revision(
        &mut self,
        supplier: PartitionId,
        eligibility_group: EligibilityGroup,
        revision: SupplyRevision<H2SupplyStatus>,
        excluded_reclaim: Option<PartitionId>,
        demand: &DemandSchedule,
    ) {
        if self
            .records
            .get(&supplier)
            .is_some_and(|record| record.revision >= revision.revision)
        {
            return;
        }
        let old_group = self
            .records
            .get(&supplier)
            .map(|record| record.eligibility_group.clone());
        self.unlink_supplier(&supplier);
        let record = self
            .records
            .entry(supplier)
            .or_insert_with(|| H2SupplyRecord {
                eligibility_group: eligibility_group.clone(),
                revision: revision.revision,
                status: revision.status,
                group_index: H2GroupIndexState::Unavailable,
                reclaim_links: None,
            });
        record.eligibility_group = eligibility_group.clone();
        record.revision = revision.revision;
        record.status = revision.status;
        if record.generation().is_some() {
            self.link_group_supplier(supplier);
        }
        self.link_reclaim_if_selectable(supplier, excluded_reclaim);
        if let Some(old_group) = old_group {
            self.reconcile_group(&old_group, demand);
        }
        self.reconcile_group(&eligibility_group, demand);
    }

    fn remove_exact_generation(
        &mut self,
        supplier: &PartitionId,
        generation: H2GenerationId,
        demand: &DemandSchedule,
    ) {
        let Some(record) = self.records.get(supplier) else {
            return;
        };
        if record.generation() != Some(generation) {
            return;
        }
        let group = record.eligibility_group.clone();
        self.unlink_supplier(supplier);
        self.records
            .get_mut(supplier)
            .expect("validated H2 supplier disappeared")
            .status = H2SupplyStatus::Unavailable;
        self.reconcile_group(&group, demand);
    }

    fn reconcile_group(&mut self, group: &EligibilityGroup, demand: &DemandSchedule) {
        let ready = demand
            .queued_group_head(group)
            .and_then(|queued| {
                queued
                    .requirement
                    .accepts_h2()
                    .then(|| self.first_peer_supplier(group, queued.requester))
                    .flatten()
            })
            .is_some();
        if ready {
            self.route_ready_groups.insert(group.clone());
        } else {
            self.route_ready_groups.remove(group);
        }
    }

    fn select_route_match(&mut self, demand: &DemandSchedule) -> Option<H2RouteMatch> {
        let eligibility_group = self
            .last_route_group
            .as_ref()
            .and_then(|last| {
                self.route_ready_groups
                    .range((Excluded(last), Unbounded))
                    .next()
            })
            .or_else(|| self.route_ready_groups.first())?
            .clone();
        let Some(queued) = demand.queued_group_head(&eligibility_group) else {
            self.route_ready_groups.remove(&eligibility_group);
            return None;
        };
        if !queued.requirement.accepts_h2() {
            self.route_ready_groups.remove(&eligibility_group);
            return None;
        }
        let Some(supplier) = self.first_peer_supplier(&eligibility_group, queued.requester) else {
            self.route_ready_groups.remove(&eligibility_group);
            return None;
        };
        let generation = self
            .records
            .get(&supplier)
            .expect("selected H2 supplier disappeared")
            .generation()
            .expect("selected H2 supplier had no generation");
        Some(H2RouteMatch {
            requester: queued.requester,
            demand: queued.demand,
            supplier,
            generation,
            eligibility_group,
        })
    }

    fn select_reclaim_generation(
        &mut self,
    ) -> Option<(PartitionId, H2GenerationId, EligibilityGroup)> {
        let supplier = self.reclaim_order.head()?;
        let record = self
            .records
            .get(&supplier)
            .expect("reclaimable H2 supplier disappeared");
        let generation = record
            .generation()
            .expect("reclaimable H2 supplier had no generation");
        let eligibility_group = record.eligibility_group.clone();
        self.unlink_reclaim_supplier(&supplier);
        Some((supplier, generation, eligibility_group))
    }

    fn link_group_supplier(&mut self, supplier: PartitionId) {
        let eligibility_group = self
            .records
            .get(&supplier)
            .expect("linked H2 supplier disappeared")
            .eligibility_group
            .clone();
        let group = self
            .suppliers_by_group
            .entry(eligibility_group)
            .or_default()
            .push_back(supplier);
        if let Some(previous) = group.previous {
            self.records
                .get_mut(&previous)
                .expect("previous H2 supplier disappeared")
                .group_index
                .links_mut()
                .next = Some(supplier);
        }
        self.records
            .get_mut(&supplier)
            .expect("linked H2 supplier disappeared")
            .group_index = H2GroupIndexState::Linked { group };
    }

    fn link_reclaim_if_selectable(
        &mut self,
        supplier: PartitionId,
        excluded_reclaim: Option<PartitionId>,
    ) {
        let Some(record) = self.records.get(&supplier) else {
            return;
        };
        if !record.idle() || excluded_reclaim == Some(supplier) || record.reclaim_links.is_some() {
            return;
        }
        let links = self.reclaim_order.push_back(supplier);
        if let Some(previous) = links.previous {
            self.records
                .get_mut(&previous)
                .expect("previous reclaimable H2 supplier disappeared")
                .reclaim_links
                .as_mut()
                .expect("previous reclaimable H2 supplier lost its links")
                .next = Some(supplier);
        }
        self.records
            .get_mut(&supplier)
            .expect("reclaimable H2 supplier disappeared")
            .reclaim_links = Some(links);
    }

    fn unlink_reclaim_supplier(&mut self, supplier: &PartitionId) {
        let Some(record) = self.records.get_mut(supplier) else {
            return;
        };
        let Some(links) = record.reclaim_links.take() else {
            return;
        };
        if let Some(previous) = links.previous {
            self.records
                .get_mut(&previous)
                .expect("previous reclaimable H2 supplier disappeared")
                .reclaim_links
                .as_mut()
                .expect("previous reclaimable H2 supplier lost its links")
                .next = links.next;
        }
        if let Some(next) = links.next {
            self.records
                .get_mut(&next)
                .expect("next reclaimable H2 supplier disappeared")
                .reclaim_links
                .as_mut()
                .expect("next reclaimable H2 supplier lost its links")
                .previous = links.previous;
        }
        self.reclaim_order.remove(*supplier, links);
    }

    fn unlink_supplier(&mut self, supplier: &PartitionId) {
        self.unlink_reclaim_supplier(supplier);
        let Some(record) = self.records.get_mut(supplier) else {
            return;
        };
        let group_index = std::mem::take(&mut record.group_index);
        let H2GroupIndexState::Linked { group } = group_index else {
            return;
        };
        let eligibility_group = record.eligibility_group.clone();
        if let Some(previous) = group.previous {
            self.records
                .get_mut(&previous)
                .expect("previous H2 supplier disappeared")
                .group_index
                .links_mut()
                .next = group.next;
        }
        if let Some(next) = group.next {
            self.records
                .get_mut(&next)
                .expect("next H2 supplier disappeared")
                .group_index
                .links_mut()
                .previous = group.previous;
        }
        let order = self
            .suppliers_by_group
            .get_mut(&eligibility_group)
            .expect("linked H2 supplier lost its group order");
        order.remove(*supplier, group);
        if order.len() == 0 {
            self.suppliers_by_group.remove(&eligibility_group);
        }
    }

    fn first_peer_supplier(
        &self,
        eligibility_group: &EligibilityGroup,
        requester: PartitionId,
    ) -> Option<PartitionId> {
        let head = self.suppliers_by_group.get(eligibility_group)?.head()?;
        if head != requester {
            return Some(head);
        }
        self.records
            .get(&head)
            .expect("H2 supplier order head disappeared")
            .group_index
            .links()
            .and_then(|links| links.next)
    }

    #[cfg(any(debug_assertions, test))]
    fn assert_consistent(&self, demand: &DemandSchedule, excluded_reclaim: Option<PartitionId>) {
        for (supplier, record) in &self.records {
            assert_eq!(
                record.generation().is_some(),
                record.group_index.links().is_some(),
                "H2 group index did not match routable supply"
            );
            if record.group_index.links().is_some() {
                assert!(
                    self.suppliers_by_group
                        .contains_key(&record.eligibility_group),
                    "H2 supplier lost its eligibility-group order"
                );
            }
            assert_eq!(
                record.idle() && excluded_reclaim != Some(*supplier),
                record.reclaim_links.is_some(),
                "H2 reclaim index did not match idle supply"
            );
            debug_assert_ne!(
                record.reclaim_links.as_ref().and_then(|links| links.next),
                Some(*supplier)
            );
            debug_assert_ne!(
                record.group_index.links().and_then(|links| links.next),
                Some(*supplier)
            );
        }
        for (group, order) in &self.suppliers_by_group {
            let expected = self
                .records
                .values()
                .filter(|record| {
                    record.eligibility_group == *group && record.group_index.links().is_some()
                })
                .count();
            order.assert_consistent(
                expected,
                self.records.len(),
                "HTTP/2 supplier order",
                |supplier| {
                    *self
                        .records
                        .get(&supplier)
                        .expect("ordered H2 supplier disappeared")
                        .group_index
                        .links()
                        .expect("ordered H2 supplier lost its links")
                },
            );
        }
        let expected_reclaim = self
            .records
            .values()
            .filter(|record| record.reclaim_links.is_some())
            .count();
        self.reclaim_order.assert_consistent(
            expected_reclaim,
            self.records.len(),
            "HTTP/2 reclaim order",
            |supplier| {
                self.records
                    .get(&supplier)
                    .expect("ordered reclaimable H2 supplier disappeared")
                    .reclaim_links
                    .expect("ordered reclaimable H2 supplier lost its links")
            },
        );
        let expected_ready = self
            .suppliers_by_group
            .keys()
            .filter(|group| {
                demand.queued_group_head(group).is_some_and(|queued| {
                    queued.requirement.accepts_h2()
                        && self.first_peer_supplier(group, queued.requester).is_some()
                })
            })
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected_ready, self.route_ready_groups,
            "HTTP/2 route-ready groups did not match demand and supply"
        );
    }
}

impl H2Supply {
    pub(super) fn has_route_ready_group(&self) -> bool {
        self.index.has_route_ready_group()
    }

    fn apply_revision(
        &mut self,
        supplier: PartitionId,
        eligibility_group: EligibilityGroup,
        revision: SupplyRevision<H2SupplyStatus>,
        demand: &DemandSchedule,
    ) {
        let excluded_reclaim = self.reclaiming.as_ref().map(|reclaim| reclaim.supplier);
        self.index.apply_revision(
            supplier,
            eligibility_group,
            revision,
            excluded_reclaim,
            demand,
        );
        self.assert_consistent(demand);
    }

    fn remove_exact_generation(
        &mut self,
        supplier: &PartitionId,
        generation: H2GenerationId,
        demand: &DemandSchedule,
    ) {
        self.index
            .remove_exact_generation(supplier, generation, demand);
        self.assert_consistent(demand);
    }

    pub(super) fn reconcile_group(&mut self, group: &EligibilityGroup, demand: &DemandSchedule) {
        self.index.reconcile_group(group, demand);
    }

    pub(super) fn prepare_route(
        &mut self,
        demand: &mut DemandSchedule,
        assignment_id: DemandAssignmentId,
    ) -> Option<PreparedH2Route> {
        let route_match = self.index.select_route_match(demand)?;
        let assignment = demand.prepare_group_assignment(
            &route_match.eligibility_group,
            &route_match.requester,
            route_match.demand,
            assignment_id,
        )?;
        self.index.last_route_group = Some(route_match.eligibility_group.clone());
        self.index
            .reconcile_group(&route_match.eligibility_group, demand);
        self.assert_consistent(demand);
        Some(PreparedH2Route {
            assignment,
            supplier: route_match.supplier,
            generation: route_match.generation,
            eligibility_group: route_match.eligibility_group,
        })
    }

    pub(super) fn prepare_reclaim(&mut self, demand: &DemandSchedule) -> Option<PreparedH2Reclaim> {
        if self.reclaiming.is_some() {
            return None;
        }
        let queued = demand.queued_head()?;
        if queued.requirement.accepts_h2() {
            return None;
        }
        let (supplier, generation, eligibility_group) = self.index.select_reclaim_generation()?;
        let prepared = PreparedH2Reclaim {
            requester: queued.requester,
            demand: queued.demand,
            supplier,
            generation,
            eligibility_group,
        };
        self.reclaiming = Some(prepared.clone());
        self.assert_consistent(demand);
        Some(prepared)
    }

    fn settle_reclaim(
        &mut self,
        prepared: &PreparedH2Reclaim,
        revision: Option<SupplyRevision<H2SupplyStatus>>,
        demand: &DemandSchedule,
    ) {
        assert_eq!(self.reclaiming.as_ref(), Some(prepared));
        self.reclaiming = None;
        match revision {
            Some(revision) => self.index.apply_revision(
                prepared.supplier,
                prepared.eligibility_group.clone(),
                revision,
                None,
                demand,
            ),
            None => {
                self.index.unlink_supplier(&prepared.supplier);
                self.index.records.remove(&prepared.supplier);
                self.index
                    .reconcile_group(&prepared.eligibility_group, demand);
            }
        }
        self.index
            .link_reclaim_if_selectable(prepared.supplier, None);
        self.assert_consistent(demand);
    }

    fn assert_consistent(&self, demand: &DemandSchedule) {
        #[cfg(not(any(debug_assertions, test)))]
        let _ = demand;
        #[cfg(any(debug_assertions, test))]
        {
            if std::thread::panicking() {
                return;
            }
            self.index.assert_consistent(
                demand,
                self.reclaiming.as_ref().map(|reclaim| reclaim.supplier),
            );
        }
    }
}

/// Identity-only route installation with a terminal admission fallback.
pub(in crate::client::pool) struct H2RouteGuard {
    /// Admission owner of the assigned demand and indexed supply.
    admission: Arc<OriginAdmission>,
    /// Identities revalidated across the unlocked cell transitions.
    prepared: PreparedH2Route,
    /// Terminal acknowledgement submitted if the crossing unwinds.
    on_drop: Option<DemandAssignmentOutcome>,
}

impl H2RouteGuard {
    pub(super) fn new(admission: Arc<OriginAdmission>, prepared: PreparedH2Route) -> Self {
        Self {
            admission,
            prepared,
            on_drop: Some(DemandAssignmentOutcome::RetrySamePosition),
        }
    }

    /// Attaches the selected peer route, then settles the demand assignment.
    ///
    /// A missing connection cell or stale generation withdraws that exact
    /// supply revision and retries the same demand residence. A missing
    /// requesting cell retires the demand. Requesting-cell rejection retries
    /// only when the original demand and route installation are still useful.
    pub(super) fn attach_route(self) -> Option<AdmissionAction> {
        let generation = self.prepared.generation;
        let Some(connection_cell) = self.admission.cell(&self.prepared.supplier) else {
            return self.settle(DemandAssignmentOutcome::RetrySamePosition, Some(generation));
        };
        if !OriginCell::h2_generation_is_accepting(&connection_cell, self.prepared.generation) {
            return self.settle(DemandAssignmentOutcome::RetrySamePosition, Some(generation));
        }

        let Some(requesting_cell) = self.admission.cell(&self.prepared.assignment.requester) else {
            return self.settle(DemandAssignmentOutcome::Refused { successor: None }, None);
        };
        let route = H2Route::new(&connection_cell, self.prepared.generation);
        if !OriginCell::attach_h2_route(
            &requesting_cell,
            route,
            &self.prepared.eligibility_group,
            self.prepared.assignment.demand,
        ) {
            return self.settle(DemandAssignmentOutcome::RetrySamePosition, None);
        }

        let next = self.settle(DemandAssignmentOutcome::Accepted { successor: None }, None);
        OriginCell::offer_local_h2(&requesting_cell);
        OriginCell::offer_peer_h2(&requesting_cell);
        next
    }

    fn settle(
        mut self,
        outcome: DemandAssignmentOutcome,
        stale_generation: Option<H2GenerationId>,
    ) -> Option<AdmissionAction> {
        let trace_outcome = match &outcome {
            DemandAssignmentOutcome::Accepted { .. } => "accepted",
            DemandAssignmentOutcome::RetrySamePosition => "retry",
            DemandAssignmentOutcome::Refused { .. } => "refused",
        };
        // The unlocked cell crossing is complete. Admission owns the assignment
        // again, so unwinding must not replay a failed acknowledgement.
        self.on_drop = None;
        let next = OriginAdmission::settle_h2_route(
            &self.admission,
            &self.prepared,
            stale_generation,
            outcome,
        );
        self.trace(trace_outcome);
        next
    }

    fn trace(&self, outcome: &str) {
        tracing::trace!(
            request_partition = ?self.prepared.assignment.requester,
            connection_partition = ?self.prepared.supplier,
            origin_scheme = %self.admission.origin().scheme(),
            origin_host = self.admission.origin().host(),
            origin_port = ?self.admission.origin().port(),
            h2_generation = ?self.prepared.generation,
            demand = ?self.prepared.assignment.demand,
            outcome,
            "HTTP/2 peer route completed"
        );
    }
}

impl Drop for H2RouteGuard {
    fn drop(&mut self) {
        let Some(result) = self.on_drop.take() else {
            return;
        };
        let next = OriginAdmission::settle_h2_route(&self.admission, &self.prepared, None, result);
        self.trace("guard_drop");
        OriginAdmission::run_action_chain(next);
    }
}

/// Exact idle-generation crossing with an admission repair fallback.
pub(in crate::client::pool) struct H2CapacityReclaim {
    /// Admission authority that owns the reclaim reservation.
    admission: Arc<OriginAdmission>,
    /// Prepared reclaim still owned by this fallback.
    prepared: Option<PreparedH2Reclaim>,
}

impl H2CapacityReclaim {
    /// Creates one unlocked exact-generation reclaim crossing.
    pub(super) fn new(admission: Arc<OriginAdmission>, prepared: PreparedH2Reclaim) -> Self {
        Self {
            admission,
            prepared: Some(prepared),
        }
    }

    /// Attempts idle reclaim and returns the next bounded-origin action.
    pub(super) fn reclaim_capacity(mut self) -> Option<AdmissionAction> {
        let prepared = self
            .prepared
            .as_ref()
            .expect("HTTP/2 reclaim action consumed more than once")
            .clone();
        let (revision, reclaimed) = match self.admission.cell(&prepared.supplier) {
            Some(cell) => {
                let (revision, connection_id) =
                    OriginCell::reclaim_idle_h2(&cell, prepared.generation);
                (Some(revision), connection_id)
            }
            None => (None, None),
        };
        self.prepared = None;
        let next = OriginAdmission::settle_h2_reclaim(&self.admission, &prepared, revision);
        tracing::trace!(
            connection_id = ?reclaimed,
            request_partition = ?prepared.requester,
            connection_partition = ?prepared.supplier,
            origin_scheme = %self.admission.origin().scheme(),
            origin_host = self.admission.origin().host(),
            origin_port = ?self.admission.origin().port(),
            h2_generation = ?prepared.generation,
            demand = ?prepared.demand,
            outcome = if reclaimed.is_some() { "reclaimed" } else { "rejected" },
            "HTTP/2 idle reclaim completed"
        );
        next
    }
}

impl Drop for H2CapacityReclaim {
    fn drop(&mut self) {
        let Some(prepared) = self.prepared.take() else {
            return;
        };
        let revision = self
            .admission
            .cell(&prepared.supplier)
            .map(|cell| cell.current_h2_supply_revision());
        let next = OriginAdmission::settle_h2_reclaim(&self.admission, &prepared, revision);
        tracing::trace!(
            request_partition = ?prepared.requester,
            connection_partition = ?prepared.supplier,
            origin_scheme = %self.admission.origin().scheme(),
            origin_host = self.admission.origin().host(),
            origin_port = ?self.admission.origin().port(),
            h2_generation = ?prepared.generation,
            demand = ?prepared.demand,
            outcome = "guard_drop",
            "HTTP/2 idle reclaim completed"
        );
        OriginAdmission::run_action_chain(next);
    }
}

pub(super) fn apply_supply_revision(
    admission: &Arc<OriginAdmission>,
    supplier: PartitionId,
    eligibility_group: EligibilityGroup,
    revision: SupplyRevision<H2SupplyStatus>,
) {
    let action = {
        let mut state = admission.state.lock();
        let super::AdmissionState {
            h2_supply, demand, ..
        } = &mut *state;
        h2_supply.apply_revision(supplier, eligibility_group, revision, demand);
        OriginAdmission::prepare_action(admission, &mut state)
    };
    OriginAdmission::run_action_chain(action);
}

pub(super) fn settle_route(
    admission: &Arc<OriginAdmission>,
    prepared: &PreparedH2Route,
    stale_generation: Option<H2GenerationId>,
    outcome: DemandAssignmentOutcome,
) -> Option<AdmissionAction> {
    let mut state = admission.state.lock();
    if let Some(generation) = stale_generation {
        let super::AdmissionState {
            h2_supply, demand, ..
        } = &mut *state;
        h2_supply.remove_exact_generation(&prepared.supplier, generation, demand);
    }
    state.settle_assignment(&prepared.assignment, outcome);
    OriginAdmission::prepare_action(admission, &mut state)
}

pub(super) fn settle_reclaim(
    admission: &Arc<OriginAdmission>,
    prepared: &PreparedH2Reclaim,
    revision: Option<SupplyRevision<H2SupplyStatus>>,
) -> Option<AdmissionAction> {
    let mut state = admission.state.lock();
    let super::AdmissionState {
        h2_supply, demand, ..
    } = &mut *state;
    h2_supply.settle_reclaim(prepared, revision, demand);
    OriginAdmission::prepare_action(admission, &mut state)
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use crate::client::pool::admission::{DemandSnapshot, ProtocolRequirement, SnapshotVersion};

    fn partition(index: usize) -> PartitionId {
        PartitionId::from_index(index)
    }

    fn generation(value: u64) -> H2GenerationId {
        H2GenerationId::for_test(value)
    }

    fn demand(id: u64, group: EligibilityGroup) -> DemandSnapshot {
        DemandSnapshot::active(
            DemandId::from_u64(id),
            SnapshotVersion::INITIAL,
            ProtocolRequirement::H2Required,
            group,
        )
    }

    fn h1_demand(id: u64, group: EligibilityGroup) -> DemandSnapshot {
        DemandSnapshot::active(
            DemandId::from_u64(id),
            SnapshotVersion::INITIAL,
            ProtocolRequirement::H1Required,
            group,
        )
    }

    fn accepting(revision: u64, generation: H2GenerationId) -> SupplyRevision<H2SupplyStatus> {
        SupplyRevision::new(
            revision,
            H2SupplyStatus::Accepting {
                generation,
                idle: false,
            },
        )
    }

    fn idle(revision: u64, generation: H2GenerationId) -> SupplyRevision<H2SupplyStatus> {
        SupplyRevision::new(
            revision,
            H2SupplyStatus::Accepting {
                generation,
                idle: true,
            },
        )
    }

    #[test]
    fn exact_generation_replacement_rejects_stale_removal() {
        let group = EligibilityGroup::Pool;
        let connection_partition = partition(1);
        let requesting_partition = partition(2);
        let first = generation(10);
        let second = generation(11);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group.clone(),
            accepting(1, first),
            &schedule,
        );
        supply.apply_revision(
            connection_partition,
            group.clone(),
            accepting(2, second),
            &schedule,
        );

        supply.remove_exact_generation(&connection_partition, first, &schedule);
        let prepared = supply
            .prepare_route(&mut schedule, DemandAssignmentId(1))
            .expect("newer supply revision should remain selectable");
        assert_eq!(second, prepared.generation);
        assert_eq!(connection_partition, prepared.supplier);
        assert_eq!(requesting_partition, prepared.assignment.requester);
    }

    #[test]
    fn current_generation_removal_repairs_the_connection_index() {
        let group = EligibilityGroup::Pool;
        let connection_partition = partition(1);
        let requesting_partition = partition(2);
        let current = generation(10);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group,
            accepting(1, current),
            &schedule,
        );

        supply.remove_exact_generation(&connection_partition, current, &schedule);

        assert!(!supply.has_route_ready_group());
        assert!(supply.index.suppliers_by_group.is_empty());
        assert!(matches!(
            supply.index.records[&connection_partition].group_index,
            H2GroupIndexState::Unavailable
        ));
    }

    #[test]
    fn route_selection_skips_the_requesting_cell_and_selects_a_peer_in_its_group() {
        let pool = EligibilityGroup::Pool;
        let isolated = EligibilityGroup::Partition(partition(3));
        let requesting_partition = partition(1);
        let peer = partition(2);
        let other_group = partition(3);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, demand(1, pool.clone()));
        schedule.apply_snapshot(other_group, demand(2, isolated.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            requesting_partition,
            pool.clone(),
            accepting(1, generation(1)),
            &schedule,
        );
        supply.apply_revision(peer, pool, accepting(1, generation(2)), &schedule);
        supply.apply_revision(
            other_group,
            isolated,
            accepting(1, generation(3)),
            &schedule,
        );

        let prepared = supply
            .prepare_route(&mut schedule, DemandAssignmentId(1))
            .expect("pool demand should find its peer connection");
        assert_eq!(requesting_partition, prepared.assignment.requester);
        assert_eq!(peer, prepared.supplier);
        assert_eq!(generation(2), prepared.generation);
    }

    #[test]
    fn older_supply_revision_cannot_replace_the_current_generation() {
        let group = EligibilityGroup::Pool;
        let connection_partition = partition(1);
        let requesting_partition = partition(2);
        let current = generation(2);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group.clone(),
            accepting(2, current),
            &schedule,
        );
        supply.apply_revision(
            connection_partition,
            group,
            accepting(1, generation(1)),
            &schedule,
        );

        let prepared = supply
            .prepare_route(&mut schedule, DemandAssignmentId(1))
            .expect("current supply revision should remain selectable");
        assert_eq!(current, prepared.generation);
    }

    #[test]
    fn h1_required_demand_is_not_route_ready() {
        let group = EligibilityGroup::Pool;
        let requesting_partition = partition(1);
        let connection_partition = partition(2);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(
            requesting_partition,
            DemandSnapshot::active(
                DemandId::from_u64(1),
                SnapshotVersion::INITIAL,
                ProtocolRequirement::H1Required,
                group.clone(),
            ),
        );
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group,
            accepting(1, generation(1)),
            &schedule,
        );

        assert!(!supply.has_route_ready_group());
        assert!(supply
            .prepare_route(&mut schedule, DemandAssignmentId(1))
            .is_none());
    }

    #[test]
    fn route_turns_rotate_across_ready_groups() {
        let first_group = EligibilityGroup::Partition(partition(10));
        let second_group = EligibilityGroup::Partition(partition(20));
        let first_request = partition(1);
        let first_connection = partition(2);
        let second_request = partition(3);
        let second_connection = partition(4);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(first_request, demand(1, first_group.clone()));
        schedule.apply_snapshot(second_request, demand(2, second_group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            first_connection,
            first_group.clone(),
            accepting(1, generation(1)),
            &schedule,
        );
        supply.apply_revision(
            second_connection,
            second_group.clone(),
            accepting(1, generation(2)),
            &schedule,
        );

        let first = supply
            .prepare_route(&mut schedule, DemandAssignmentId(1))
            .expect("first ready group was not selected");
        schedule.settle_assignment(
            &first.assignment,
            DemandAssignmentOutcome::RetrySamePosition,
        );
        supply.reconcile_group(&first.eligibility_group, &schedule);

        let second = supply
            .prepare_route(&mut schedule, DemandAssignmentId(2))
            .expect("second ready group was not selected");
        assert_ne!(first.eligibility_group, second.eligibility_group);
    }

    #[test]
    #[should_panic(expected = "HTTP/2 route-ready groups did not match demand and supply")]
    fn consistency_check_rejects_a_missing_ready_group() {
        let group = EligibilityGroup::Pool;
        let connection_partition = partition(1);
        let requesting_partition = partition(2);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group,
            accepting(1, generation(1)),
            &schedule,
        );

        supply.index.route_ready_groups.clear();

        supply.assert_consistent(&schedule);
    }

    #[test]
    fn only_idle_h2_is_selected_for_h1_required_reclaim() {
        let group = EligibilityGroup::Pool;
        let requesting_partition = partition(1);
        let connection_partition = partition(2);
        let current = generation(10);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, h1_demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(
            connection_partition,
            group.clone(),
            accepting(1, current),
            &schedule,
        );

        assert!(
            supply.prepare_reclaim(&schedule).is_none(),
            "busy HTTP/2 generation entered reclaim order"
        );

        supply.apply_revision(connection_partition, group, idle(2, current), &schedule);
        let prepared = supply
            .prepare_reclaim(&schedule)
            .expect("idle HTTP/2 generation was not selected");
        assert_eq!(requesting_partition, prepared.requester);
        assert_eq!(connection_partition, prepared.supplier);
        assert_eq!(current, prepared.generation);
        assert!(supply.index.reclaim_order.head().is_none());

        supply.settle_reclaim(&prepared, Some(idle(2, current)), &schedule);
        assert_eq!(
            Some(connection_partition),
            supply.index.reclaim_order.head()
        );
    }

    #[test]
    fn reclaim_completion_cannot_restore_a_replaced_generation() {
        let group = EligibilityGroup::Pool;
        let requesting_partition = partition(1);
        let connection_partition = partition(2);
        let old = generation(10);
        let replacement = generation(11);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(requesting_partition, h1_demand(1, group.clone()));
        let mut supply = H2Supply::default();
        supply.apply_revision(connection_partition, group.clone(), idle(1, old), &schedule);
        let prepared = supply
            .prepare_reclaim(&schedule)
            .expect("idle HTTP/2 generation was not selected");

        supply.apply_revision(connection_partition, group, idle(2, replacement), &schedule);
        supply.settle_reclaim(&prepared, Some(idle(1, old)), &schedule);

        let next = supply
            .prepare_reclaim(&schedule)
            .expect("replacement generation was not restored to reclaim order");
        assert_eq!(replacement, next.generation);
    }
}
