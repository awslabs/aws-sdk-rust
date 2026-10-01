/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! One-to-one acquisition payload delivery across admission and cell locks.
//!
//! Admission reserves one demand generation and moves either bounded capacity
//! or a provisional HTTP/1 sender into a [`DeliveryGuard`]. The guard
//! commits any connection cell transition before taking the requesting
//! cell lock, then owns the payload until that cell accepts it. Its
//! settlement keeps the demand assignment installed until requesting-cell
//! state is authoritative.

use super::h1::{H1Candidate, H1MatchId, H1SupplyOutcome};
use super::{
    AdmissionAction, CapacityLease, CapacityPermit, DemandAssignment, DemandAssignmentOutcome,
    DemandId, DemandSnapshot, OriginAdmission,
};
use crate::client::pool::cell::{
    AcquisitionOutcome, AcquisitionStep, EstablishmentPermit, OriginCell,
};
use crate::client::pool::partition::PartitionId;
use crate::sync::Arc;
use aws_smithy_runtime_api::client::connection::ConnectionId;
use std::fmt;

/// Capacity or a borrowed HTTP/1 sender crossing to waiting demand.
enum DeliveryPayload {
    /// Permit removed from admission but not yet represented by a lease.
    Capacity(CapacityPermit),
    /// Provisional sender whose owning-cell reservation must revalidate.
    BorrowedH1 {
        /// Retained H1 match that selected this sender.
        match_id: H1MatchId,
        /// Cell that owns the sender and local reservation.
        supplier: PartitionId,
        /// Sender owner with cancellation fallback.
        candidate: H1Candidate,
    },
}

/// One acquisition payload and its admission-owned demand assignment.
///
/// Dropping an undelivered guard refunnels its payload before making the
/// requesting cell's demand schedulable again. Once committed,
/// [`DeliverySettlement`] owns assignment settlement and the requesting cell
/// owns the acquisition step.
pub(in crate::client::pool) struct DeliveryGuard {
    /// Admission authority that owns the assignment and fallback.
    admission: Arc<OriginAdmission>,
    /// Exact demand excluded from competing admission while the payload crosses.
    assignment: DemandAssignment,
    /// Payload ownership before resolution, after resolution, or after transfer.
    state: DeliveryState,
}

/// Ownership phase of one admission-to-cell delivery crossing.
///
/// ```text
/// Pending -- resolve connection-cell work ------------> Ready
/// Pending -- peer sender rejection or drop -----------> admission fallback
/// Ready   -- commit to requesting cell ---------------> Disarmed + DeliverySettlement
/// Ready   -- drop ------------------------------------> admission fallback
/// ```
///
/// `Disarmed` owns no payload. After commit, `DeliverySettlement` owns
/// assignment and connection-cell completion.
enum DeliveryState {
    /// Admission selected the payload, but connection cell-side work may fail.
    Pending(DeliveryPayload),
    /// Connection-owning-cell work completed; the exact acquisition step may
    /// now cross to the requesting cell.
    Ready {
        step: AcquisitionStep,
        settlement: DeliverySettlementKind,
    },
    /// Payload and fallback responsibility moved to requesting cell-owned state.
    Disarmed,
}

impl DeliveryGuard {
    /// Creates a delivery for one permit removed from admission.
    pub(super) fn capacity(
        admission: Arc<OriginAdmission>,
        assignment: DemandAssignment,
        permit: CapacityPermit,
    ) -> Self {
        Self {
            admission,
            assignment,
            state: DeliveryState::Pending(DeliveryPayload::Capacity(permit)),
        }
    }

    /// Creates a delivery for one provisional sender selected for borrowing.
    pub(super) fn borrowed_h1(
        admission: Arc<OriginAdmission>,
        assignment: DemandAssignment,
        match_id: H1MatchId,
        supplier: PartitionId,
        candidate: H1Candidate,
    ) -> Self {
        Self {
            admission,
            assignment,
            state: DeliveryState::Pending(DeliveryPayload::BorrowedH1 {
                match_id,
                supplier,
                candidate,
            }),
        }
    }

    /// Returns the demand generation owned by this delivery assignment.
    pub(in crate::client::pool) fn demand(&self) -> DemandId {
        self.assignment.demand
    }

    /// Returns whether admission still recognizes this delivery assignment.
    #[cfg(test)]
    pub(in crate::client::pool) fn is_current(&self) -> bool {
        self.admission.assignment_is_current(&self.assignment)
    }

    /// Materializes owning-cell state and attempts one requesting-cell delivery.
    pub(super) fn deliver(mut self) -> Option<AdmissionAction> {
        if !self.resolve_payload() {
            return None;
        }
        match self.admission.cell(&self.assignment.requester) {
            Some(requesting_cell) => OriginCell::receive_delivery(&requesting_cell, self),
            None => self.refuse(None),
        }
    }

    /// Completes fallible owning-cell work before reserving requesting-cell state.
    fn resolve_payload(&mut self) -> bool {
        let state = std::mem::replace(&mut self.state, DeliveryState::Disarmed);
        let DeliveryState::Pending(payload) = state else {
            unreachable!("delivery payload resolved more than once");
        };
        let (step, settlement) = match payload {
            DeliveryPayload::Capacity(permit) => (
                AcquisitionStep::StartEstablishment(EstablishmentPermit::bounded(
                    CapacityLease::new(self.admission.clone(), permit),
                )),
                DeliverySettlementKind::Capacity,
            ),
            DeliveryPayload::BorrowedH1 {
                match_id,
                supplier,
                candidate,
            } => match candidate.commit() {
                Ok(selection) => {
                    let connection_id = selection.connection_id();
                    (
                        AcquisitionStep::Resolved(AcquisitionOutcome::H1(selection)),
                        DeliverySettlementKind::BorrowedH1 {
                            connection_id,
                            match_id,
                            supplier,
                        },
                    )
                }
                Err(candidate) => {
                    let outcome = candidate.reject();
                    let next = OriginAdmission::settle_borrow_delivery(
                        &self.admission,
                        match_id,
                        &self.assignment,
                        DemandAssignmentOutcome::RetrySamePosition,
                        None,
                        Some(outcome),
                    );
                    OriginAdmission::run_action_chain(next);
                    return false;
                }
            },
        };
        self.state = DeliveryState::Ready { step, settlement };
        true
    }

    /// Materializes this guard before tests manually split the requesting cell
    /// reservation and installation transitions.
    #[cfg(test)]
    pub(in crate::client::pool) fn resolve_payload_for_test(&mut self) -> bool {
        self.resolve_payload()
    }

    /// Moves the resolved payload into a requesting cell-owned acquisition step.
    pub(in crate::client::pool) fn into_step(
        mut self,
        successor: Option<DemandSnapshot>,
    ) -> (AcquisitionStep, DeliverySettlement) {
        let state = std::mem::replace(&mut self.state, DeliveryState::Disarmed);
        let DeliveryState::Ready { step, settlement } = state else {
            unreachable!("delivery committed before payload resolution");
        };
        (
            step,
            DeliverySettlement {
                admission: self.admission.clone(),
                assignment: self.assignment.clone(),
                successor,
                kind: Some(settlement),
            },
        )
    }

    /// Refunnels this delivery after requesting cell revalidation rejects it.
    pub(in crate::client::pool) fn refuse(
        mut self,
        successor: Option<DemandSnapshot>,
    ) -> Option<AdmissionAction> {
        let state = std::mem::replace(&mut self.state, DeliveryState::Disarmed);
        self.settle_state(state, DemandAssignmentOutcome::Refused { successor })
    }

    /// Resolves payload fallback before settling the admission-owned assignment.
    fn settle_state(
        &self,
        state: DeliveryState,
        outcome: DemandAssignmentOutcome,
    ) -> Option<AdmissionAction> {
        match state {
            DeliveryState::Pending(DeliveryPayload::Capacity(permit)) => {
                OriginAdmission::settle_delivery(
                    &self.admission,
                    &self.assignment,
                    Some(permit),
                    outcome,
                )
            }
            DeliveryState::Pending(DeliveryPayload::BorrowedH1 {
                match_id,
                candidate,
                ..
            }) => {
                let supply = candidate.reject();
                OriginAdmission::settle_borrow_delivery(
                    &self.admission,
                    match_id,
                    &self.assignment,
                    outcome,
                    None,
                    Some(supply),
                )
            }
            DeliveryState::Ready { step, settlement } => match settlement {
                DeliverySettlementKind::Capacity => {
                    drop(step);
                    OriginAdmission::settle_delivery(
                        &self.admission,
                        &self.assignment,
                        None,
                        outcome,
                    )
                }
                DeliverySettlementKind::BorrowedH1 {
                    match_id, supplier, ..
                } => {
                    let supplier_cell = self.admission.cell(&supplier);
                    drop(step);
                    let supply = match supplier_cell {
                        Some(supplier_cell) => H1SupplyOutcome::supplier_live(
                            supplier,
                            supplier_cell.cancel_h1_reservation(match_id),
                        ),
                        None => H1SupplyOutcome::supplier_expired(supplier),
                    };
                    OriginAdmission::settle_borrow_delivery(
                        &self.admission,
                        match_id,
                        &self.assignment,
                        outcome,
                        None,
                        Some(supply),
                    )
                }
            },
            DeliveryState::Disarmed => None,
        }
    }
}

impl fmt::Debug for DeliveryGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeliveryGuard")
            .field("assignment", &self.assignment)
            .finish_non_exhaustive()
    }
}

impl Drop for DeliveryGuard {
    fn drop(&mut self) {
        let state = std::mem::replace(&mut self.state, DeliveryState::Disarmed);
        if matches!(state, DeliveryState::Disarmed) {
            return;
        }
        let next = self.settle_state(state, DemandAssignmentOutcome::RetrySamePosition);
        OriginAdmission::run_action_chain(next);
    }
}

/// Assignment settlement after a step becomes owned by the requesting cell.
///
/// Explicit refusal returns the step before settling the assignment. Once the
/// requesting cell owns the step, dropping this value accepts the assignment
/// and completes any borrowed connection's H1 match.
pub(in crate::client::pool) struct DeliverySettlement {
    /// Admission authority that owns the outstanding assignment.
    admission: Arc<OriginAdmission>,
    /// Exact assignment settled by this value.
    assignment: DemandAssignment,
    /// Successor demand to publish after the old generation finishes.
    successor: Option<DemandSnapshot>,
    /// Payload-specific connection cell completion still owed by this guard.
    kind: Option<DeliverySettlementKind>,
}

/// Terminal work required after the requesting cell takes ownership.
enum DeliverySettlementKind {
    /// Only the admission demand assignment remains to settle.
    Capacity,
    /// The connection cell must learn whether sender transfer succeeded.
    BorrowedH1 {
        /// Connection whose sender crossed to the requesting cell.
        connection_id: ConnectionId,
        /// Retained H1 match completed by this settlement.
        match_id: H1MatchId,
        /// Cell whose local peer reservation must complete.
        supplier: PartitionId,
    },
}

impl DeliverySettlement {
    /// Avoids republishing a successor already covered by visible HTTP/2 state.
    ///
    /// Capacity or HTTP/1 delivery may reveal another waiter after the
    /// requesting cell already gained a local generation or peer route. That
    /// visible HTTP/2 state can serve an H2-compatible successor directly. If
    /// it later closes, its close path publishes the cell's current demand.
    pub(in crate::client::pool) fn suppress_h2_successor(&mut self) {
        if self
            .successor
            .as_ref()
            .is_some_and(DemandSnapshot::accepts_h2)
        {
            self.successor = None;
        }
    }

    /// Records that requesting cell state accepted the acquisition step.
    pub(in crate::client::pool) fn accept(mut self) -> Option<AdmissionAction> {
        let kind = self
            .kind
            .take()
            .expect("delivery settlement completed more than once");
        let successor = self.successor.take();
        self.settle(kind, DemandAssignmentOutcome::Accepted { successor }, None)
    }

    /// Refunnels steps refused after requesting cell reservation.
    pub(in crate::client::pool) fn refuse(
        mut self,
        returned_steps: [Option<AcquisitionStep>; 2],
    ) -> Option<AdmissionAction> {
        let kind = self
            .kind
            .take()
            .expect("delivery settlement completed more than once");
        let successor = self.successor.take();
        self.settle(
            kind,
            DemandAssignmentOutcome::Refused { successor },
            Some(returned_steps),
        )
    }

    /// Completes payload-specific H1 work and settles the demand assignment.
    fn settle(
        &self,
        kind: DeliverySettlementKind,
        outcome: DemandAssignmentOutcome,
        returned_steps: Option<[Option<AcquisitionStep>; 2]>,
    ) -> Option<AdmissionAction> {
        match kind {
            DeliverySettlementKind::Capacity => {
                drop(returned_steps);
                OriginAdmission::settle_delivery(&self.admission, &self.assignment, None, outcome)
            }
            DeliverySettlementKind::BorrowedH1 {
                connection_id,
                match_id,
                supplier,
            } => {
                let refused = returned_steps.is_some();
                let supplier_cell = refused.then(|| self.admission.cell(&supplier)).flatten();
                drop(returned_steps);
                let refused_outcome = refused.then(|| match supplier_cell {
                    Some(supplier_cell) => H1SupplyOutcome::supplier_live(
                        supplier,
                        supplier_cell.cancel_h1_reservation(match_id),
                    ),
                    None => H1SupplyOutcome::supplier_expired(supplier),
                });
                let transferred_supplier = (!refused).then_some(supplier);
                let action = OriginAdmission::settle_borrow_delivery(
                    &self.admission,
                    match_id,
                    &self.assignment,
                    outcome,
                    transferred_supplier,
                    refused_outcome,
                );
                if !refused {
                    tracing::trace!(
                        connection_id = %connection_id,
                        request_partition = ?self.assignment.requester,
                        connection_partition = ?supplier,
                        origin_scheme = %self.admission.origin().scheme(),
                        origin_host = self.admission.origin().host(),
                        origin_port = ?self.admission.origin().port(),
                        "HTTP/1 connection borrowed for peer demand"
                    );
                }
                action
            }
        }
    }
}

impl fmt::Debug for DeliverySettlement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeliverySettlement")
            .field("assignment", &self.assignment)
            .finish_non_exhaustive()
    }
}

impl Drop for DeliverySettlement {
    fn drop(&mut self) {
        let Some(kind) = self.kind.take() else {
            return;
        };
        let successor = self.successor.take();
        let next = self.settle(kind, DemandAssignmentOutcome::Accepted { successor }, None);
        OriginAdmission::run_action_chain(next);
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::super::{ProtocolRequirement, SnapshotVersion};
    use super::*;
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::EligibilityGroup;
    use std::num::NonZeroUsize;

    fn cell(origin: &Arc<OriginAdmission>, partition: usize) -> Arc<OriginCell> {
        let cell = Arc::new(OriginCell::new(
            PartitionId::from_index(partition),
            OriginKey::from_parts(http_1x::uri::Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            Some(origin.clone()),
            None,
        ));
        OriginAdmission::register_cell(origin, cell)
    }

    fn demand(id: u64) -> DemandSnapshot {
        DemandSnapshot::active(
            DemandId::from_u64(id),
            SnapshotVersion::INITIAL,
            ProtocolRequirement::H1Compatible,
            EligibilityGroup::Pool,
        )
    }

    #[test]
    fn assignment_currency_includes_the_demand_id() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let requesting_partition = cell(&origin, 1);
        let delivery = OriginAdmission::submit_without_running(
            &origin,
            requesting_partition.id().partition(),
            demand(1),
        )
        .unwrap();
        assert!(delivery.is_current());

        origin
            .state
            .lock()
            .apply_demand_snapshot(requesting_partition.id().partition(), demand(2));
        assert!(!delivery.is_current());
        delivery.refuse(None);
    }

    #[test]
    fn stale_successor_cannot_leave_active_demand_idle() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let requesting_partition = cell(&origin, 1);
        let delivery = OriginAdmission::submit_without_running(
            &origin,
            requesting_partition.id().partition(),
            demand(1),
        )
        .unwrap();
        origin.state.lock().apply_demand_snapshot(
            requesting_partition.id().partition(),
            DemandSnapshot::active(
                DemandId::from_u64(1),
                SnapshotVersion::INITIAL.next(),
                ProtocolRequirement::H1Compatible,
                EligibilityGroup::Pool,
            ),
        );
        delivery.refuse(Some(demand(1)));

        assert_eq!(1, origin.probe().available);
        assert_eq!(0, origin.probe().ordered);
    }

    #[test]
    fn dropped_delivery_refunnels_capacity_and_preserves_order() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let first = cell(&origin, 1);
        let second = cell(&origin, 2);
        let (first_waiter, first_demand) =
            first.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let (second_waiter, second_demand) =
            second.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let delivery =
            OriginAdmission::submit_without_running(&origin, first.id().partition(), first_demand)
                .unwrap();
        {
            let mut state = origin.state.lock();
            state.apply_demand_snapshot(second.id().partition(), second_demand);
        }
        drop(delivery);

        let first_lease = OriginCell::take_ready_lease(&first, first_waiter)
            .expect("dropped delivery did not retry the original head");
        assert!(OriginCell::take_ready_lease(&second, second_waiter).is_none());
        assert_eq!(1, origin.probe().ordered);

        drop(first_lease);
        let second_lease = OriginCell::take_ready_lease(&second, second_waiter)
            .expect("younger demand did not run after the original head");
        drop(second_lease);
    }

    #[test]
    fn expired_requesting_cell_refunnels_capacity() {
        let origin = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let requesting_partition = cell(&origin, 1);
        let requesting_cell_id = requesting_partition.id().partition();
        let (_waiter, snapshot) =
            requesting_partition.register_waiter_without_publish(ProtocolRequirement::H1Compatible);
        let delivery =
            OriginAdmission::submit_without_running(&origin, requesting_cell_id, snapshot).unwrap();

        drop(requesting_partition);
        assert!(origin.cell(&requesting_cell_id).is_none());
        OriginAdmission::run_action_chain(Some(AdmissionAction::Deliver(delivery)));

        let probe = origin.probe();
        assert_eq!(1, probe.available);
        assert_eq!(0, probe.assigned);
        assert_eq!(0, probe.ordered);
    }
}
