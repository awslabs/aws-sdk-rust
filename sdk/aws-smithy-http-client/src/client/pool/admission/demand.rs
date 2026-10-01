/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Versioned cross-cell demand scheduling for one bounded origin.
//!
//! Each cell publishes at most one active demand: its acquisition queue's
//! current head. That demand may be satisfied either by origin-wide connection
//! capacity or by an existing HTTP/2 connection in its eligibility group.
//! These resources require different orderings.
//!
//! For example, the origin order may contain `A(group X), B(group Y)` while
//! only group Y has a reusable HTTP/2 connection. A remains first for the next
//! available connection permit, while the group-Y connection can serve B
//! without consuming that permit or delaying A. The group order finds B
//! directly instead of scanning every partition.
//!
//! The same demand occupies both orders because capacity delivery and HTTP/2
//! route installation can race to satisfy it. Assigning either position keeps
//! both positions attached until the requesting cell accepts or refuses the
//! handoff. The other resource cannot select the same demand while the first
//! handoff runs outside the admission lock.

use super::{CapacityPermit, IntrusiveLinks, IntrusiveOrder};
use crate::client::pool::partition::{EligibilityGroup, PartitionId};
use std::collections::HashMap;

/// Protocol capability required by the head waiter in a cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::client::pool) enum ProtocolRequirement {
    /// The waiter requires HTTP/1 wire semantics.
    H1Required,
    /// The waiter may dispatch over HTTP/1 or HTTP/2.
    H1Compatible,
    /// The waiter requires HTTP/2.
    H2Required,
}

impl ProtocolRequirement {
    pub(in crate::client::pool) fn accepts_h1(self) -> bool {
        self != Self::H2Required
    }

    pub(in crate::client::pool) fn accepts_h2(self) -> bool {
        self != Self::H1Required
    }
}

/// Identity of one cell-local demand generation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) struct DemandId(u64);

impl DemandId {
    pub(in crate::client::pool) const fn from_u64(value: u64) -> Self {
        Self(value)
    }
}

/// Strict ordering of complete snapshots within one [`DemandId`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) struct SnapshotVersion(u64);

impl SnapshotVersion {
    pub(in crate::client::pool) const INITIAL: Self = Self(0);

    pub(in crate::client::pool) fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("demand snapshot version exhausted"),
        )
    }
}

/// Complete state submitted for one demand identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::client::pool) enum DemandState {
    Active {
        requirement: ProtocolRequirement,
        eligibility_group: EligibilityGroup,
    },
    Inactive,
}

/// Versioned replacement state for one cell's current demand generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::client::pool) struct DemandSnapshot {
    pub(super) id: DemandId,
    pub(super) version: SnapshotVersion,
    pub(super) state: DemandState,
}

impl DemandSnapshot {
    pub(in crate::client::pool) fn active(
        id: DemandId,
        version: SnapshotVersion,
        requirement: ProtocolRequirement,
        eligibility_group: EligibilityGroup,
    ) -> Self {
        Self {
            id,
            version,
            state: DemandState::Active {
                requirement,
                eligibility_group,
            },
        }
    }

    pub(in crate::client::pool) fn accepts_h2(&self) -> bool {
        matches!(
            self.state,
            DemandState::Active { requirement, .. } if requirement.accepts_h2()
        )
    }

    pub(in crate::client::pool) fn inactive(id: DemandId, version: SnapshotVersion) -> Self {
        Self {
            id,
            version,
            state: DemandState::Inactive,
        }
    }

    #[cfg(test)]
    pub(in crate::client::pool) fn id_for_test(&self) -> DemandId {
        self.id
    }

    pub(in crate::client::pool) fn is_active(&self) -> bool {
        matches!(self.state, DemandState::Active { .. })
    }

    fn is_newer_than(&self, current: &Self) -> bool {
        self.id > current.id || (self.id == current.id && self.version > current.version)
    }
}

/// Never-reused identity of one resource-to-demand assignment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct DemandAssignmentId(pub(super) u64);

/// Exact demand temporarily assigned while a resource crosses lock domains.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DemandAssignment {
    pub(super) id: DemandAssignmentId,
    pub(super) requester: PartitionId,
    pub(super) demand: DemandId,
}

/// Admission's settlement of one detached demand assignment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum DemandAssignmentOutcome {
    Accepted { successor: Option<DemandSnapshot> },
    RetrySamePosition,
    Refused { successor: Option<DemandSnapshot> },
}

/// Cross-cell demand records and their scheduling orders.
///
/// At every completed transition:
///
/// - `records` owns the newest snapshot for every retained cell.
/// - the origin order and the record's eligibility-group order contain exactly
///   the `Queued` and `PendingAssignment` records;
/// - links for both orders live inside those schedule states;
/// - an origin assignment owns the origin head; and
/// - an HTTP/2 route assignment owns its eligibility-group head.
///
/// Admission coordinates capacity extraction with this schedule while holding
/// the same origin lock.
#[derive(Debug, Default)]
pub(super) struct DemandSchedule {
    /// Latest demand and scheduling residence for each retained cell.
    records: HashMap<PartitionId, DemandRecord>,
    /// Origin-wide order used by one-to-one capacity and HTTP/1 reuse.
    origin_order: IntrusiveOrder<PartitionId>,
    /// Reusable-HTTP/2 demand order for each connection-reuse group.
    group_orders: HashMap<EligibilityGroup, IntrusiveOrder<PartitionId>>,
}

/// Complete origin-order head used to choose one HTTP/1 reuse action.
#[derive(Clone, Debug)]
pub(super) struct QueuedDemand {
    /// Cell whose oldest waiter owns this demand generation.
    pub(super) requester: PartitionId,
    /// Demand identity revalidated at each crossing.
    pub(super) demand: DemandId,
    /// Protocol capability required by the requesting cell waiter.
    pub(super) requirement: ProtocolRequirement,
    /// Connection-owning cells whose H1 senders may satisfy this demand.
    pub(super) eligibility_group: EligibilityGroup,
}

/// Latest snapshot and scheduling residence for one stable cell.
#[derive(Debug)]
struct DemandRecord {
    /// Newest complete snapshot observed for the cell.
    latest: DemandSnapshot,
    /// Stable group retained while an inactive replacement crosses an assignment.
    eligibility_group: Option<EligibilityGroup>,
    /// Scheduling residence, including links while ordered.
    schedule_state: DemandScheduleState,
}

/// Links retained by one demand in both scheduling views.
#[derive(Clone, Debug)]
struct DemandLinks {
    /// Position in origin-wide capacity and HTTP/1 order.
    origin: IntrusiveLinks<PartitionId>,
    /// Position in the demand's all-protocol eligibility-group order.
    group: IntrusiveLinks<PartitionId>,
}

/// Order whose head is held by one detached demand assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DemandOrder {
    /// Capacity or HTTP/1 delivery selected from origin order.
    Origin,
    /// HTTP/2 route selected from eligibility-group order.
    Group,
}

/// Scheduling state for one partition's latest demand.
///
/// ```text
/// Unscheduled -- active snapshot --------------------------> Queued
/// Queued -- prepare assignment ----------------------------> PendingAssignment
/// Queued -- inactive or replacement snapshot -------------> Unscheduled
/// PendingAssignment -- retry unchanged demand -------------> Queued
/// PendingAssignment -- accept/refuse/replacement retry ----> Unscheduled
/// Unscheduled -- active successor remains -----------------> Queued
/// ```
#[derive(Clone, Debug)]
enum DemandScheduleState {
    /// The cell has no demand represented in scheduling.
    Unscheduled,
    /// The demand is waiting in origin order.
    Queued {
        /// Demand generation represented by this residence.
        demand: DemandId,
        /// Origin-wide and eligibility-group scheduling links.
        links: DemandLinks,
    },
    /// One delivery or route guard owns this demand assignment.
    PendingAssignment {
        /// Exact assignment allowed to settle this state.
        assignment: DemandAssignment,
        /// Snapshot version current when admission created the assignment.
        version: super::SnapshotVersion,
        /// Scheduling order whose head selected this assignment.
        #[cfg_attr(
            not(debug_assertions),
            allow(
                dead_code,
                reason = "the scheduling view is retained for debug invariant checks"
            )
        )]
        selected_order: DemandOrder,
        /// Both scheduling positions retained until acknowledgement.
        links: DemandLinks,
    },
}

impl DemandScheduleState {
    #[cfg(any(debug_assertions, test))]
    /// Borrows one view's links while this record is ordered.
    fn links(&self, order: DemandOrder) -> Option<&IntrusiveLinks<PartitionId>> {
        let links = match self {
            Self::Unscheduled => return None,
            Self::Queued { links, .. } | Self::PendingAssignment { links, .. } => links,
        };
        match order {
            DemandOrder::Origin => Some(&links.origin),
            DemandOrder::Group => Some(&links.group),
        }
    }

    /// Mutably borrows one view's links from an ordered record.
    ///
    /// # Panics
    ///
    /// Panics when called for an idle record.
    fn links_mut(&mut self, order: DemandOrder) -> &mut IntrusiveLinks<PartitionId> {
        let links = match self {
            Self::Unscheduled => panic!("unscheduled demand has no scheduling links"),
            Self::Queued { links, .. } | Self::PendingAssignment { links, .. } => links,
        };
        match order {
            DemandOrder::Origin => &mut links.origin,
            DemandOrder::Group => &mut links.group,
        }
    }

    /// Detaches both link sets while moving a record out of scheduling.
    fn into_links(self) -> Option<DemandLinks> {
        match self {
            Self::Unscheduled => None,
            Self::Queued { links, .. } | Self::PendingAssignment { links, .. } => Some(links),
        }
    }
}

/// Connection capacity extracted under admission lock before crossing to a cell.
#[derive(Debug)]
pub(super) struct PreparedCapacityDelivery {
    pub(super) assignment: DemandAssignment,
    pub(super) permit: CapacityPermit,
}

impl DemandSchedule {
    /// Applies a complete snapshot and updates its cell's scheduling state.
    pub(super) fn apply_snapshot(&mut self, requester: PartitionId, snapshot: DemandSnapshot) {
        if let Some(current) = self.records.get(&requester) {
            if !snapshot.is_newer_than(&current.latest) {
                return;
            }
        } else {
            self.records.insert(
                requester,
                DemandRecord {
                    latest: snapshot.clone(),
                    eligibility_group: None,
                    schedule_state: DemandScheduleState::Unscheduled,
                },
            );
        }

        let should_remove = matches!(
            &self
                .records
                .get(&requester)
                .expect("demand record disappeared")
                .schedule_state,
            DemandScheduleState::Queued { demand, .. }
                if *demand != snapshot.id || !snapshot.is_active()
        );
        if should_remove {
            self.remove_from_order(&requester);
        }

        if let DemandState::Active {
            eligibility_group, ..
        } = &snapshot.state
        {
            self.records
                .get_mut(&requester)
                .expect("demand record disappeared")
                .eligibility_group = Some(eligibility_group.clone());
        }
        self.records
            .get_mut(&requester)
            .expect("demand record disappeared")
            .latest = snapshot;

        let record = self
            .records
            .get(&requester)
            .expect("demand record disappeared");
        if record.latest.is_active()
            && matches!(&record.schedule_state, DemandScheduleState::Unscheduled)
        {
            self.enqueue(requester);
        }
        self.assert_consistent();
    }

    /// Appends an idle active demand to both scheduling orders.
    fn enqueue(&mut self, requester: PartitionId) {
        let group = self
            .group_for(&requester)
            .expect("enqueued demand was inactive");
        let origin = self.origin_order.push_back(requester);
        let group_links = self
            .group_orders
            .entry(group)
            .or_default()
            .push_back(requester);
        if let Some(previous) = origin.previous {
            self.records
                .get_mut(&previous)
                .expect("origin demand tail disappeared")
                .schedule_state
                .links_mut(DemandOrder::Origin)
                .next = Some(requester);
        }
        if let Some(previous) = group_links.previous {
            self.records
                .get_mut(&previous)
                .expect("group demand tail disappeared")
                .schedule_state
                .links_mut(DemandOrder::Group)
                .next = Some(requester);
        }

        let record = self
            .records
            .get_mut(&requester)
            .expect("queued demand record disappeared");
        debug_assert!(matches!(
            record.schedule_state,
            DemandScheduleState::Unscheduled
        ));
        let demand = record.latest.id;
        record.schedule_state = DemandScheduleState::Queued {
            demand,
            links: DemandLinks {
                origin,
                group: group_links,
            },
        };
    }

    /// Removes an ordered demand from both views and leaves its record idle.
    fn remove_from_order(&mut self, requester: &PartitionId) {
        let group = self
            .group_for(requester)
            .expect("removed demand had no eligibility group");
        let schedule_state = {
            let record = self
                .records
                .get_mut(requester)
                .expect("removed demand record disappeared");
            std::mem::replace(&mut record.schedule_state, DemandScheduleState::Unscheduled)
        };
        let links = schedule_state
            .into_links()
            .expect("removed demand had no scheduling links");

        if let Some(previous) = links.origin.previous {
            self.records
                .get_mut(&previous)
                .expect("previous origin demand disappeared")
                .schedule_state
                .links_mut(DemandOrder::Origin)
                .next = links.origin.next;
        }
        if let Some(next) = links.origin.next {
            self.records
                .get_mut(&next)
                .expect("next origin demand disappeared")
                .schedule_state
                .links_mut(DemandOrder::Origin)
                .previous = links.origin.previous;
        }
        self.origin_order.remove(*requester, links.origin);

        if let Some(previous) = links.group.previous {
            self.records
                .get_mut(&previous)
                .expect("previous group demand disappeared")
                .schedule_state
                .links_mut(DemandOrder::Group)
                .next = links.group.next;
        }
        if let Some(next) = links.group.next {
            self.records
                .get_mut(&next)
                .expect("next group demand disappeared")
                .schedule_state
                .links_mut(DemandOrder::Group)
                .previous = links.group.previous;
        }
        let order = self
            .group_orders
            .get_mut(&group)
            .expect("ordered demand lost its eligibility-group order");
        order.remove(*requester, links.group);
        if order.len() == 0 {
            self.group_orders.remove(&group);
        }
    }

    /// Returns whether the head can begin a new one-to-one delivery.
    pub(super) fn head_is_queued(&self) -> bool {
        let Some(head) = self.origin_order.head() else {
            return false;
        };
        matches!(
            &self
                .records
                .get(&head)
                .expect("order head disappeared")
                .schedule_state,
            DemandScheduleState::Queued { .. }
        )
    }

    /// Returns the complete origin-order head when it may begin reuse.
    pub(super) fn queued_head(&self) -> Option<QueuedDemand> {
        let head = self.origin_order.head()?;
        let record = self.records.get(&head).expect("order head disappeared");
        let DemandScheduleState::Queued { demand, .. } = &record.schedule_state else {
            return None;
        };
        let DemandState::Active {
            requirement,
            eligibility_group,
        } = &record.latest.state
        else {
            unreachable!("queued demand became inactive");
        };
        Some(QueuedDemand {
            requester: head,
            demand: *demand,
            requirement: *requirement,
            eligibility_group: eligibility_group.clone(),
        })
    }

    /// Returns one eligibility-group head when it may receive H2 visibility.
    pub(super) fn queued_group_head(&self, group: &EligibilityGroup) -> Option<QueuedDemand> {
        let head = self.group_orders.get(group)?.head()?;
        let record = self
            .records
            .get(&head)
            .expect("group demand head disappeared");
        let DemandScheduleState::Queued { demand, .. } = &record.schedule_state else {
            return None;
        };
        let DemandState::Active {
            requirement,
            eligibility_group,
        } = &record.latest.state
        else {
            unreachable!("queued group demand became inactive");
        };
        debug_assert_eq!(eligibility_group, group);
        Some(QueuedDemand {
            requester: head,
            demand: *demand,
            requirement: *requirement,
            eligibility_group: eligibility_group.clone(),
        })
    }

    /// Returns the latest eligibility group retained for one cell.
    pub(super) fn group_for(&self, requester: &PartitionId) -> Option<EligibilityGroup> {
        self.records
            .get(requester)
            .and_then(|record| record.eligibility_group.clone())
    }

    /// Returns whether `requesting_partition` still has this demand queued for a new action.
    pub(super) fn is_current_queued(&self, requester: &PartitionId, demand: DemandId) -> bool {
        self.records.get(requester).is_some_and(|record| {
            record.latest.id == demand
                && record.latest.is_active()
                && matches!(
                    record.schedule_state,
                    DemandScheduleState::Queued {
                        demand: current,
                        ..
                    } if current == demand
                )
        })
    }

    /// Assigns exact origin-head demand after an H1 match resolves.
    pub(super) fn prepare_h1_assignment(
        &mut self,
        requester: &PartitionId,
        demand: DemandId,
        assignment_id: DemandAssignmentId,
    ) -> Option<DemandAssignment> {
        if !self.is_current_queued(requester, demand) || self.origin_order.head()? != *requester {
            return None;
        }
        self.prepare_assignment(*requester, assignment_id, DemandOrder::Origin)
    }

    /// Assigns one exact eligibility-group head to an H2 route.
    pub(super) fn prepare_group_assignment(
        &mut self,
        eligibility_group: &EligibilityGroup,
        requester: &PartitionId,
        demand: DemandId,
        assignment_id: DemandAssignmentId,
    ) -> Option<DemandAssignment> {
        if !self.is_current_queued(requester, demand)
            || self.group_orders.get(eligibility_group)?.head() != Some(*requester)
        {
            return None;
        }
        self.prepare_assignment(*requester, assignment_id, DemandOrder::Group)
    }

    /// Assigns the origin-order head to one capacity delivery.
    pub(super) fn prepare_origin_assignment(
        &mut self,
        assignment_id: DemandAssignmentId,
    ) -> Option<DemandAssignment> {
        let requester = self.origin_order.head()?;
        self.prepare_assignment(requester, assignment_id, DemandOrder::Origin)
    }

    fn prepare_assignment(
        &mut self,
        requester: PartitionId,
        assignment_id: DemandAssignmentId,
        selected_order: DemandOrder,
    ) -> Option<DemandAssignment> {
        let record = self
            .records
            .get_mut(&requester)
            .expect("order head disappeared");
        let schedule_state =
            std::mem::replace(&mut record.schedule_state, DemandScheduleState::Unscheduled);
        match schedule_state {
            DemandScheduleState::Queued { demand, links } => {
                debug_assert_eq!(record.latest.id, demand);
                debug_assert!(record.latest.is_active());
                let assignment = DemandAssignment {
                    id: assignment_id,
                    requester,
                    demand,
                };
                record.schedule_state = DemandScheduleState::PendingAssignment {
                    assignment: assignment.clone(),
                    version: record.latest.version,
                    selected_order,
                    links,
                };
                self.assert_consistent();
                Some(assignment)
            }
            schedule_state => {
                record.schedule_state = schedule_state;
                None
            }
        }
    }

    #[cfg(test)]
    pub(super) fn assignment_is_current(&self, assignment: &DemandAssignment) -> bool {
        let Some(record) = self.records.get(&assignment.requester) else {
            return false;
        };
        matches!(
            &record.schedule_state,
            DemandScheduleState::PendingAssignment { assignment: current, .. }
                if current == assignment
                    && record.latest.id == assignment.demand
                    && record.latest.is_active()
        )
    }

    /// Settles one detached assignment and resolves the demand's next state.
    pub(super) fn settle_assignment(
        &mut self,
        assignment: &DemandAssignment,
        outcome: DemandAssignmentOutcome,
    ) {
        let requester = &assignment.requester;
        let Some(record) = self.records.get(requester) else {
            return;
        };
        let assigned = match &record.schedule_state {
            DemandScheduleState::PendingAssignment {
                assignment: current,
                version,
                ..
            } if current == assignment => (current.demand, *version),
            _ => return,
        };
        let (assigned_demand, assigned_version) = assigned;

        match outcome {
            DemandAssignmentOutcome::RetrySamePosition => {
                let record = self
                    .records
                    .get(requester)
                    .expect("assigned demand record disappeared");
                if record.latest.id == assigned_demand && record.latest.is_active() {
                    let record = self
                        .records
                        .get_mut(requester)
                        .expect("assigned demand record disappeared");
                    let schedule_state = std::mem::replace(
                        &mut record.schedule_state,
                        DemandScheduleState::Unscheduled,
                    );
                    let DemandScheduleState::PendingAssignment {
                        assignment: current,
                        links,
                        ..
                    } = schedule_state
                    else {
                        unreachable!("demand assignment disappeared");
                    };
                    debug_assert_eq!(current, *assignment);
                    record.schedule_state = DemandScheduleState::Queued {
                        demand: current.demand,
                        links,
                    };
                    self.assert_consistent();
                    return;
                }

                self.remove_from_order(requester);
                if self
                    .records
                    .get(requester)
                    .expect("assigned demand record disappeared")
                    .latest
                    .is_active()
                {
                    self.enqueue(*requester);
                }
            }
            DemandAssignmentOutcome::Accepted { successor }
            | DemandAssignmentOutcome::Refused { successor } => {
                self.remove_from_order(requester);

                let install_successor = successor.as_ref().is_some_and(|successor| {
                    successor.id > assigned_demand
                        && successor.is_newer_than(
                            &self
                                .records
                                .get(requester)
                                .expect("assigned demand record disappeared")
                                .latest,
                        )
                });
                if install_successor {
                    self.records
                        .get_mut(requester)
                        .expect("assigned demand record disappeared")
                        .latest = successor.expect("validated successor disappeared");
                } else {
                    let retirement =
                        DemandSnapshot::inactive(assigned_demand, assigned_version.next());
                    let record = self
                        .records
                        .get_mut(requester)
                        .expect("assigned demand record disappeared");
                    if retirement.is_newer_than(&record.latest) {
                        record.latest = retirement;
                    }
                }

                if self
                    .records
                    .get(requester)
                    .expect("assigned demand record disappeared")
                    .latest
                    .is_active()
                {
                    self.enqueue(*requester);
                }
            }
        }
        self.assert_consistent();
    }

    /// Returns the latest complete snapshot retained for `requester`.
    #[cfg(test)]
    pub(super) fn latest_for_test(&self, requester: &PartitionId) -> Option<&DemandSnapshot> {
        self.records.get(requester).map(|record| &record.latest)
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.origin_order.len()
    }

    #[cfg(test)]
    pub(super) fn queued_len(&self) -> usize {
        self.records
            .values()
            .filter(|record| matches!(&record.schedule_state, DemandScheduleState::Queued { .. }))
            .count()
    }

    #[cfg(test)]
    pub(super) fn pending_assignment_count(&self) -> usize {
        self.records
            .values()
            .filter(|record| {
                matches!(
                    &record.schedule_state,
                    DemandScheduleState::PendingAssignment { .. }
                )
            })
            .count()
    }

    /// Checks residence, link, length, group, and assignment relationships.
    fn assert_consistent(&self) {
        #[cfg(any(debug_assertions, test))]
        {
            if std::thread::panicking() {
                return;
            }
            self.assert_consistent_debug();
        }
    }

    #[cfg(any(debug_assertions, test))]
    fn assert_consistent_debug(&self) {
        let ordered_records = self
            .records
            .values()
            .filter(|record| record.schedule_state.links(DemandOrder::Origin).is_some())
            .count();
        let origin_assignments = self
            .records
            .values()
            .filter(|record| {
                matches!(
                    record.schedule_state,
                    DemandScheduleState::PendingAssignment {
                        selected_order: DemandOrder::Origin,
                        ..
                    }
                )
            })
            .count();
        assert!(
            origin_assignments <= 1,
            "more than one origin demand assignment was active"
        );
        self.origin_order.assert_consistent(
            ordered_records,
            self.records.len(),
            "origin demand order",
            |requesting_partition| {
                let record = self
                    .records
                    .get(&requesting_partition)
                    .expect("origin-ordered demand disappeared");
                match &record.schedule_state {
                    DemandScheduleState::Unscheduled => {
                        unreachable!("origin-ordered demand became unscheduled")
                    }
                    DemandScheduleState::Queued { demand, .. } => {
                        assert!(record.latest.is_active(), "queued demand became inactive");
                        assert_eq!(
                            record.latest.id, *demand,
                            "queued schedule state did not match its latest demand"
                        );
                    }
                    DemandScheduleState::PendingAssignment {
                        assignment,
                        selected_order,
                        ..
                    } => {
                        if *selected_order == DemandOrder::Origin {
                            assert_eq!(
                                Some(requesting_partition),
                                self.origin_order.head(),
                                "origin demand assignment moved away from its head"
                            );
                        }
                        assert!(
                            record.latest.id >= assignment.demand,
                            "demand assignment named a future demand"
                        );
                    }
                }
                *record
                    .schedule_state
                    .links(DemandOrder::Origin)
                    .expect("origin-ordered demand lost its links")
            },
        );

        let group_ordered_records = self
            .records
            .values()
            .filter(|record| record.schedule_state.links(DemandOrder::Group).is_some())
            .count();
        for record in self
            .records
            .values()
            .filter(|record| record.schedule_state.links(DemandOrder::Group).is_some())
        {
            let group = record
                .eligibility_group
                .as_ref()
                .expect("group-ordered demand lost its eligibility group");
            assert!(
                self.group_orders.contains_key(group),
                "ordered demand lost its eligibility-group order"
            );
        }
        assert_eq!(
            group_ordered_records,
            self.group_orders
                .values()
                .map(IntrusiveOrder::len)
                .sum::<usize>(),
            "eligibility-group orders did not contain every ordered demand"
        );

        for (group, order) in &self.group_orders {
            let group_assignments = self
                .records
                .values()
                .filter(|record| {
                    record.eligibility_group.as_ref() == Some(group)
                        && matches!(
                            record.schedule_state,
                            DemandScheduleState::PendingAssignment {
                                selected_order: DemandOrder::Group,
                                ..
                            }
                        )
                })
                .count();
            assert!(
                group_assignments <= 1,
                "more than one group demand assignment was active"
            );
            let expected = self
                .records
                .values()
                .filter(|record| {
                    record.schedule_state.links(DemandOrder::Group).is_some()
                        && record.eligibility_group.as_ref() == Some(group)
                })
                .count();
            order.assert_consistent(
                expected,
                self.records.len(),
                "eligibility-group demand order",
                |requesting_partition| {
                    let record = self
                        .records
                        .get(&requesting_partition)
                        .expect("group-ordered demand disappeared");
                    assert_eq!(
                        record.eligibility_group.as_ref(),
                        Some(group),
                        "demand occupied the wrong eligibility-group order"
                    );
                    if matches!(
                        record.schedule_state,
                        DemandScheduleState::PendingAssignment {
                            selected_order: DemandOrder::Group,
                            ..
                        }
                    ) {
                        assert_eq!(
                            Some(requesting_partition),
                            order.head(),
                            "group route assignment moved away from its head"
                        );
                    }
                    *record
                        .schedule_state
                        .links(DemandOrder::Group)
                        .expect("group-ordered demand lost its links")
                },
            );
        }
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;

    fn partition(index: usize) -> PartitionId {
        PartitionId::from_index(index)
    }

    fn active(id: u64, group: EligibilityGroup) -> DemandSnapshot {
        DemandSnapshot::active(
            DemandId::from_u64(id),
            super::super::SnapshotVersion::INITIAL,
            ProtocolRequirement::H1Compatible,
            group,
        )
    }

    #[test]
    fn origin_and_group_orders_repair_independently() {
        let pool = EligibilityGroup::Pool;
        let isolated = EligibilityGroup::Partition(partition(2));
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition(1), active(1, pool.clone()));
        schedule.apply_snapshot(partition(2), active(2, isolated.clone()));
        schedule.apply_snapshot(partition(3), active(3, pool.clone()));

        assert_eq!(
            Some(partition(1)),
            schedule.queued_head().map(|head| head.requester)
        );
        assert_eq!(
            Some(partition(1)),
            schedule.queued_group_head(&pool).map(|head| head.requester)
        );
        assert_eq!(
            Some(partition(2)),
            schedule
                .queued_group_head(&isolated)
                .map(|head| head.requester)
        );

        let assignment = schedule
            .prepare_group_assignment(
                &pool,
                &partition(1),
                DemandId::from_u64(1),
                DemandAssignmentId(7),
            )
            .expect("pool group head should reserve");
        assert!(schedule.queued_group_head(&pool).is_none());
        assert_eq!(
            Some(partition(2)),
            schedule
                .queued_group_head(&isolated)
                .map(|head| head.requester)
        );
        assert!(schedule.queued_head().is_none());

        schedule.settle_assignment(
            &assignment,
            DemandAssignmentOutcome::Accepted { successor: None },
        );
        assert_eq!(
            Some(partition(2)),
            schedule.queued_head().map(|head| head.requester)
        );
        assert_eq!(
            Some(partition(3)),
            schedule.queued_group_head(&pool).map(|head| head.requester)
        );
    }

    #[test]
    fn retry_preserves_both_order_positions() {
        let group = EligibilityGroup::Pool;
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition(1), active(1, group.clone()));
        schedule.apply_snapshot(partition(2), active(2, group.clone()));

        let assignment = schedule
            .prepare_group_assignment(
                &group,
                &partition(1),
                DemandId::from_u64(1),
                DemandAssignmentId(9),
            )
            .expect("group head should reserve");
        schedule.settle_assignment(&assignment, DemandAssignmentOutcome::RetrySamePosition);

        assert_eq!(
            Some(partition(1)),
            schedule.queued_head().map(|head| head.requester)
        );
        assert_eq!(
            Some(partition(1)),
            schedule
                .queued_group_head(&group)
                .map(|head| head.requester)
        );
    }

    #[test]
    fn inactive_snapshot_during_group_assignment_retires_without_losing_group() {
        let group = EligibilityGroup::Pool;
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition(1), active(1, group.clone()));
        let assignment = schedule
            .prepare_group_assignment(
                &group,
                &partition(1),
                DemandId::from_u64(1),
                DemandAssignmentId(11),
            )
            .expect("group head should reserve");
        schedule.apply_snapshot(
            partition(1),
            DemandSnapshot::inactive(
                DemandId::from_u64(1),
                super::super::SnapshotVersion::INITIAL.next(),
            ),
        );

        schedule.settle_assignment(&assignment, DemandAssignmentOutcome::RetrySamePosition);
        assert_eq!(0, schedule.len());
        assert!(schedule.queued_group_head(&group).is_none());
    }

    #[test]
    fn accepted_assignment_does_not_retire_a_newer_active_snapshot() {
        let group = EligibilityGroup::Pool;
        let partition = partition(1);
        let demand = DemandId::from_u64(1);
        let initial = super::super::SnapshotVersion::INITIAL;
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(
            partition,
            DemandSnapshot::active(
                demand,
                initial,
                ProtocolRequirement::H2Required,
                group.clone(),
            ),
        );
        let assignment = schedule
            .prepare_group_assignment(&group, &partition, demand, DemandAssignmentId(12))
            .expect("group head should reserve");
        let republished = DemandSnapshot::active(
            demand,
            initial.next().next(),
            ProtocolRequirement::H2Required,
            group.clone(),
        );
        schedule.apply_snapshot(partition, republished.clone());

        schedule.settle_assignment(
            &assignment,
            DemandAssignmentOutcome::Accepted { successor: None },
        );

        assert_eq!(Some(&republished), schedule.latest_for_test(&partition));
        assert_eq!(
            Some(partition),
            schedule.queued_head().map(|head| head.requester)
        );
        assert_eq!(
            Some(partition),
            schedule
                .queued_group_head(&group)
                .map(|head| head.requester)
        );
    }

    #[test]
    fn group_assignment_rejects_a_stale_demand_identity() {
        let group = EligibilityGroup::Pool;
        let partition = partition(1);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition, active(1, group.clone()));
        schedule.apply_snapshot(partition, active(2, group.clone()));

        assert!(schedule
            .prepare_group_assignment(
                &group,
                &partition,
                DemandId::from_u64(1),
                DemandAssignmentId(13),
            )
            .is_none());
        assert_eq!(
            Some(DemandId::from_u64(2)),
            schedule
                .queued_group_head(&group)
                .map(|queued| queued.demand)
        );
    }

    #[test]
    fn stale_settlement_does_not_close_a_newer_assignment() {
        let group = EligibilityGroup::Pool;
        let partition = partition(1);
        let demand = DemandId::from_u64(1);
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition, active(1, group.clone()));
        let assignment = schedule
            .prepare_group_assignment(&group, &partition, demand, DemandAssignmentId(14))
            .expect("group head should reserve");
        let stale = DemandAssignment {
            id: DemandAssignmentId(15),
            requester: partition,
            demand,
        };

        schedule.settle_assignment(
            &stale,
            DemandAssignmentOutcome::Accepted { successor: None },
        );

        assert!(schedule.assignment_is_current(&assignment));
        assert!(!schedule.assignment_is_current(&stale));
        assert_eq!(
            Some(&active(1, group)),
            schedule.latest_for_test(&partition)
        );
    }

    #[test]
    #[should_panic(expected = "ordered demand lost its eligibility-group order")]
    fn consistency_check_rejects_an_orphaned_group_link() {
        let group = EligibilityGroup::Pool;
        let mut schedule = DemandSchedule::default();
        schedule.apply_snapshot(partition(1), active(1, group.clone()));

        schedule.group_orders.remove(&group);
        schedule.assert_consistent();
    }
}
