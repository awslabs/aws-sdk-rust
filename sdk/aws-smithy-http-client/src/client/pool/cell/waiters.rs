/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Cell-local acquisition order and delivered-result ownership.
//!
//! [`AcquisitionQueue`] is the sole owner of every live acquisition attempt
//! in one origin cell. It combines the bounded FIFO, unlocked delivery
//! crossings, establishment launch, and terminal result handoff in one state
//! machine.
//!
//! Normal transitions are:
//!
//! ```text
//! bounded register ----------------------------------> Waiting
//! unbounded register --------------------------------> ReadyToEstablish
//! Waiting --reserve admission or reuse delivery-----> DeliveryPending
//! Waiting --local H1 return--------------------------> Ready
//! DeliveryPending --capacity-------------------------> ReadyToEstablish
//! DeliveryPending --borrowed H1----------------------> Ready
//! DeliveryPending --local H1--> DeliveryPending(pending H1)
//! DeliveryPending(pending H1) --delivery--> Ready + return delivered value
//! ReadyToEstablish --local H1 return-----------------> Ready + return permit
//! ReadyToEstablish --poll----------------------------> Launching(Submitted)
//! Launching(Submitted) --first owner-runtime poll----> Launching(Started)
//! Launching --local H1 or establishment result------> Ready
//! Waiting --local or peer H2 activation--------------> Ready
//! DeliveryPending --H2 activation--------------------> DeliveryPending(pending H2)
//! ReadyToEstablish / Launching --H2 activation-------> Ready
//! Ready --poll---------------------------------------> removed; result transferred
//! ```
//!
//! Cancellation has one residence used only while a committed delivery is
//! crossing the cell lock boundary:
//!
//! ```text
//! Waiting ----------------------> removed; retire or advance demand
//! DeliveryPending --------------> DeliveryCancelled
//! DeliveryCancelled --delivery-> removed; return payloads
//! ReadyToEstablish / Ready -----> removed; return the held event
//! Launching --------------------> removed; task settles separately
//! ```
//!
//! Admission updates, returned values, and task wakes are detached for the
//! caller to process after releasing the cell lock.

use super::super::admission::{DemandId, DemandSnapshot, ProtocolRequirement, SnapshotVersion};
use super::super::partition::EligibilityGroup;
use super::{AcquisitionOutcome, AcquisitionStep, EstablishmentPermit};
use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroUsize;
use std::task::{Context, Poll, Waker};

/// Local waiter identity within one cell.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) struct WaiterId(pub(in crate::client::pool) u64);

/// Complete acquisition state protected by one origin cell lock.
///
/// At every completed transition:
///
/// - `records` owns every retained acquisition attempt.
/// - [`WaitingQueueState::Active`] contains exactly the linked
///   [`WaiterState::Waiting`] records.
/// - The active demand identifies and describes the FIFO head.
/// - `h1_compatible_waiters` contains exactly each H1-compatible
///   [`WaiterState::DeliveryPending`] without a pending H1,
///   [`WaiterState::ReadyToEstablish`], or [`WaiterState::Launching`] attempt.
/// - `h2_compatible_waiters` contains every delivery-pending, ready-to-establish, or
///   launching attempt that has not received another result. The waiting FIFO
///   head is compared with this set without duplicating its queue residence.
/// - Delivery-pending, ready-to-establish, launching, delivery-cancelled, and
///   ready attempts remain in `records` but never in the bounded FIFO.
///
/// Transitions return admission publications, values requiring fallback, and
/// wakers to the caller; none runs while this state is locked.
#[derive(Debug, Default)]
pub(super) struct AcquisitionQueue {
    /// Every attempt still waiting, launching, or holding a terminal result.
    records: HashMap<WaiterId, WaiterRecord>,
    /// FIFO endpoints and demand for the records currently waiting.
    waiting: WaitingQueueState,
    /// H1-compatible attempts still competing with a returned sender.
    h1_compatible_waiters: BTreeSet<WaiterId>,
    /// Attempts outside the FIFO that may accept an H2 activation.
    h2_compatible_waiters: BTreeSet<WaiterId>,
    /// Next cell-local waiter identity.
    next_waiter_id: u64,
    /// Next identity for a head-waiter demand generation.
    next_demand_id: u64,
}

/// Owner-runtime progress of one submitted establishment task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EstablishmentPhase {
    /// The task was submitted but has not begun polling the connector.
    Submitted,
    /// The owner runtime polled the task and the pool owns completion.
    Started,
}

/// Whether the cell has an active FIFO and aggregate head demand.
#[derive(Debug, Default)]
enum WaitingQueueState {
    /// No waiter is linked and no demand is active.
    #[default]
    Empty,
    /// A nonempty FIFO represented by its endpoints, length, and head demand.
    Active {
        /// Oldest waiting record.
        head: WaiterId,
        /// Youngest waiting record.
        tail: WaiterId,
        /// Number of records linked from `head` through `tail`.
        len: NonZeroUsize,
        /// Demand generation represented by `head`.
        demand: DemandTicket,
    },
}

/// One request retained while waiting for or owning an acquisition outcome.
#[derive(Debug)]
struct WaiterRecord {
    /// Protocol requirement published while this waiter is the head.
    requirement: ProtocolRequirement,
    /// Queue residence, delivery state, and any owned outcome.
    state: WaiterState,
}

/// Authoritative cell-local ownership state for one waiter.
#[derive(Debug)]
enum WaiterState {
    /// The waiter is linked in the cell-local FIFO.
    Waiting {
        /// Older waiting record, or `None` at the head.
        previous: Option<WaiterId>,
        /// Newer waiting record, or `None` at the tail.
        next: Option<WaiterId>,
        /// Latest task waiting for an acquisition step.
        waker: Option<Waker>,
    },
    /// A committed capacity or reuse delivery reserved this waiter but has
    /// not installed its payload.
    DeliveryPending {
        /// Latest task waiting for the crossing delivery.
        waker: Option<Waker>,
        /// Local protocol outcome that won while delivery was crossing.
        pending_result: Option<AcquisitionOutcome>,
    },
    /// Cancellation won after reservation while the committed delivery was
    /// outside the lock.
    DeliveryCancelled {
        /// Task detached for wake after the crossing payload is returned.
        waker: Option<Waker>,
        /// Protocol outcome returned after the crossing closes, if one arrived first.
        pending_result: Option<AcquisitionOutcome>,
    },
    /// Capacity is ready to start an establishment attempt.
    ReadyToEstablish {
        /// Optional bounded-origin capacity owned until connection install.
        permit: EstablishmentPermit,
    },
    /// Establishment was submitted to the owner runtime and may be unpolled.
    Launching {
        /// Ownership phase used to distinguish submission from first poll.
        phase: EstablishmentPhase,
        /// Latest task waiting for the attempt or returned H1 to complete.
        waker: Option<Waker>,
    },
    /// The waiter owns its terminal acquisition outcome.
    Ready(AcquisitionOutcome),
}

/// Cell-local ownership of the aggregate head demand.
#[derive(Debug)]
struct DemandTicket {
    /// Identity of this head-waiter generation.
    id: DemandId,
    /// Version assigned to the next complete publication for this demand
    /// identity.
    version: SnapshotVersion,
    /// Protocol capability required by this head waiter.
    requirement: ProtocolRequirement,
}

impl DemandTicket {
    /// Creates the complete active state published for this head generation.
    fn snapshot(&self, eligibility_group: &EligibilityGroup) -> DemandSnapshot {
        DemandSnapshot::active(
            self.id,
            self.version,
            self.requirement,
            eligibility_group.clone(),
        )
    }
}

impl AcquisitionQueue {
    /// Registers one acquisition attempt.
    ///
    /// An unbounded attempt starts ready to establish. A bounded attempt joins
    /// the capacity FIFO and publishes demand only when it becomes the head.
    pub(super) fn register_waiter(
        &mut self,
        requirement: ProtocolRequirement,
        eligibility_group: &EligibilityGroup,
        bounded: bool,
    ) -> (WaiterId, Option<DemandSnapshot>) {
        let waiter = self.take_waiter_id();
        if !bounded {
            self.records.insert(
                waiter,
                WaiterRecord {
                    requirement,
                    state: WaiterState::ReadyToEstablish {
                        permit: EstablishmentPermit::unbounded(),
                    },
                },
            );
            self.add_protocol_waiters(waiter, requirement);
            self.assert_consistent();
            return (waiter, None);
        }

        let previous = match &self.waiting {
            WaitingQueueState::Empty => None,
            WaitingQueueState::Active { tail, .. } => Some(*tail),
        };
        let initial_demand = previous.is_none().then(|| self.new_demand(requirement));

        if let Some(previous) = previous {
            let previous = self
                .records
                .get_mut(&previous)
                .expect("waiting tail disappeared");
            let WaiterState::Waiting { next, .. } = &mut previous.state else {
                unreachable!("waiting tail left the waiting state");
            };
            debug_assert!(next.is_none());
            *next = Some(waiter);
        }

        let replaced = self.records.insert(
            waiter,
            WaiterRecord {
                requirement,
                state: WaiterState::Waiting {
                    previous,
                    next: None,
                    waker: None,
                },
            },
        );
        debug_assert!(replaced.is_none());

        let snapshot = match (&mut self.waiting, initial_demand) {
            (waiting @ WaitingQueueState::Empty, Some(demand)) => {
                let snapshot = demand.snapshot(eligibility_group);
                *waiting = WaitingQueueState::Active {
                    head: waiter,
                    tail: waiter,
                    len: NonZeroUsize::MIN,
                    demand,
                };
                Some(snapshot)
            }
            (WaitingQueueState::Active { tail, len, .. }, None) => {
                *tail = waiter;
                *len = len.checked_add(1).expect("waiter queue length exhausted");
                None
            }
            _ => unreachable!("waiter queue occupancy changed during registration"),
        };
        self.assert_consistent();
        (waiter, snapshot)
    }

    /// Returns the oldest acquisition attempt that can accept HTTP/1.
    ///
    /// A bounded waiter remains compatible after receiving capacity so a
    /// returned sender can still beat its lazy or pool-owned establishment
    /// attempt. Comparing waiter identities preserves arrival order between
    /// that set and the current capacity-waiting head.
    fn oldest_h1_compatible_waiter(&self) -> Option<WaiterId> {
        let waiting = match &self.waiting {
            WaitingQueueState::Active { head, .. }
                if self.records[head].requirement.accepts_h1() =>
            {
                Some(*head)
            }
            WaitingQueueState::Empty | WaitingQueueState::Active { .. } => None,
        };
        match (waiting, self.h1_compatible_waiters.first().copied()) {
            (Some(waiting), Some(launching)) => Some(waiting.min(launching)),
            (Some(waiting), None) => Some(waiting),
            (None, launching) => launching,
        }
    }

    /// Returns whether a returned H1 can satisfy a live acquisition attempt.
    pub(super) fn has_h1_compatible_waiter(&self) -> bool {
        self.oldest_h1_compatible_waiter().is_some()
    }

    /// Returns whether a previously admitted local attempt still accepts H1.
    ///
    /// These waiters left the capacity FIFO before its current head and
    /// therefore precede cross-cell reuse for that newer aggregate demand.
    pub(super) fn has_prior_h1_waiter(&self) -> bool {
        !self.h1_compatible_waiters.is_empty()
    }

    /// Returns the oldest attempt that may accept an HTTP/2 activation.
    pub(super) fn oldest_h2_compatible_waiter(&self) -> Option<WaiterId> {
        let waiting = match &self.waiting {
            WaitingQueueState::Active { head, .. }
                if self.records[head].requirement.accepts_h2() =>
            {
                Some(*head)
            }
            WaitingQueueState::Active { .. } => None,
            WaitingQueueState::Empty => None,
        };
        match (waiting, self.h2_compatible_waiters.first().copied()) {
            (Some(waiting), Some(active)) => Some(waiting.min(active)),
            (Some(waiting), None) => Some(waiting),
            (None, active) => active,
        }
    }

    /// Advances the active demand beyond an accepted peer route.
    ///
    /// Admission retires the published version when it acknowledges route
    /// visibility. The waiter remains queued locally while the route gate
    /// offers activations. Reserving the following version here ensures that
    /// route invalidation can republish the same demand generation without an
    /// older inactive snapshot winning.
    pub(super) fn supersede_demand_snapshot(&mut self, demand: DemandId) -> bool {
        let WaitingQueueState::Active {
            demand: current, ..
        } = &mut self.waiting
        else {
            return false;
        };
        if current.id != demand {
            return false;
        }
        let acknowledged = current.version.next();
        current.version = acknowledged.next();
        self.assert_consistent();
        true
    }

    /// Returns the current head demand for publication after route loss.
    pub(super) fn current_demand_snapshot(
        &self,
        eligibility_group: &EligibilityGroup,
    ) -> Option<DemandSnapshot> {
        match &self.waiting {
            WaitingQueueState::Empty => None,
            WaitingQueueState::Active { demand, .. } => Some(demand.snapshot(eligibility_group)),
        }
    }

    /// Returns whether `waiter` is still the oldest H2-compatible waiter.
    pub(super) fn is_oldest_h2_compatible_waiter(&self, waiter: WaiterId) -> bool {
        self.oldest_h2_compatible_waiter() == Some(waiter)
    }

    /// Returns the newest waiter identity committed before peer route installation.
    pub(super) fn route_cutoff(&self) -> Option<WaiterId> {
        (!self.records.is_empty()).then(|| {
            WaiterId(
                self.next_waiter_id
                    .checked_sub(1)
                    .expect("nonempty waiter set had no allocated identity"),
            )
        })
    }

    /// Returns whether an acquisition at or before `cutoff` still needs H2.
    pub(super) fn has_h2_compatible_waiter_through(&self, cutoff: WaiterId) -> bool {
        self.oldest_h2_compatible_waiter()
            .is_some_and(|waiter| waiter <= cutoff)
    }

    /// Returns whether any acquisition must precede a new direct H2 arrival.
    pub(super) fn has_h2_compatible_waiter(&self) -> bool {
        self.oldest_h2_compatible_waiter().is_some()
    }

    /// Returns whether one transferred generation waiter remains launchable.
    pub(super) fn is_launching_h2_waiter(&self, waiter: WaiterId) -> bool {
        self.h2_compatible_waiters.contains(&waiter)
            && self.records.get(&waiter).is_some_and(|record| {
                record.requirement.accepts_h2()
                    && matches!(record.state, WaiterState::Launching { .. })
            })
    }

    /// Adds one waiter to each compatible-protocol index.
    fn add_protocol_waiters(&mut self, waiter: WaiterId, requirement: ProtocolRequirement) {
        if requirement.accepts_h1() {
            self.h1_compatible_waiters.insert(waiter);
        }
        if requirement.accepts_h2() {
            self.h2_compatible_waiters.insert(waiter);
        }
    }

    /// Removes one waiter from both compatible-protocol indexes.
    fn remove_protocol_waiters(&mut self, waiter: WaiterId) {
        self.h1_compatible_waiters.remove(&waiter);
        self.h2_compatible_waiters.remove(&waiter);
    }

    /// Commits a returned HTTP/1 sender to the oldest compatible attempt.
    ///
    /// Payload construction is delayed until all panic-capable requesting-cell checks
    /// have completed. Once constructed, the result is either state-owned or
    /// returned to the caller for cleanup after the cell lock is released.
    pub(super) fn offer_returned_h1(
        &mut self,
        result: impl FnOnce() -> AcquisitionOutcome,
        eligibility_group: &EligibilityGroup,
    ) -> (Option<WaiterId>, WaiterResolution) {
        let Some(waiter) = self.oldest_h1_compatible_waiter() else {
            return (None, WaiterResolution::refused(result()));
        };
        (
            Some(waiter),
            self.assign_protocol_outcome(waiter, result, eligibility_group),
        )
    }

    /// Installs a selected protocol result through the shared waiter transition.
    fn assign_protocol_outcome(
        &mut self,
        waiter: WaiterId,
        result: impl FnOnce() -> AcquisitionOutcome,
        eligibility_group: &EligibilityGroup,
    ) -> WaiterResolution {
        if matches!(
            self.records.get(&waiter).map(|record| &record.state),
            Some(WaiterState::Waiting { .. })
        ) {
            let removed = self.pop_head(eligibility_group);
            debug_assert_eq!(removed.waiter, waiter);
            let record = self
                .records
                .get_mut(&waiter)
                .expect("selected requesting waiter disappeared");
            let WaiterState::Waiting { waker, .. } = &mut record.state else {
                unreachable!("selected requesting waiter left the waiting state");
            };
            let waker = waker.take();
            record.state = WaiterState::Ready(result());
            let retired =
                DemandSnapshot::inactive(removed.demand.id, removed.demand.version.next());
            self.assert_consistent();
            return WaiterResolution {
                demand_updates: [Some(retired), removed.successor],
                returned_step: None,
                waker,
            };
        }

        let record = self
            .records
            .get_mut(&waiter)
            .expect("selected compatible waiter disappeared");
        let (returned_step, waker) = match &mut record.state {
            WaiterState::DeliveryPending {
                waker: _,
                pending_result,
            } => {
                debug_assert!(pending_result.is_none());
                *pending_result = Some(result());
                (None, None)
            }
            WaiterState::ReadyToEstablish { .. } | WaiterState::Launching { .. } => {
                let previous = std::mem::replace(&mut record.state, WaiterState::Ready(result()));
                match previous {
                    WaiterState::ReadyToEstablish { permit } => {
                        (Some(AcquisitionStep::StartEstablishment(permit)), None)
                    }
                    WaiterState::Launching { waker, .. } => (None, waker),
                    _ => {
                        unreachable!("selected compatible waiter changed state under the cell lock")
                    }
                }
            }
            WaiterState::Waiting { .. }
            | WaiterState::DeliveryCancelled { .. }
            | WaiterState::Ready(_) => {
                unreachable!("selected waiter was not eligible for a protocol result")
            }
        };
        self.remove_protocol_waiters(waiter);
        if returned_step.is_none() {
            // The result is state-owned and no permit is detached, so an
            // invariant failure cannot drop a pool value under this lock.
            self.assert_consistent();
        }
        WaiterResolution {
            demand_updates: [None, None],
            returned_step,
            waker,
        }
    }

    /// Cancels one waiter and detaches all cross-lock cleanup work.
    ///
    /// Removing the head retires its demand and starts a successor generation.
    /// Removing another waiting record leaves the active demand unchanged.
    /// Cancellation while delivery is pending leaves a marker for the exact
    /// waiter commit to observe. A ready step is returned for cleanup after unlock.
    pub(super) fn cancel_waiter(
        &mut self,
        waiter: WaiterId,
        eligibility_group: &EligibilityGroup,
    ) -> Option<WaiterCancellation> {
        let state = &self.records.get(&waiter)?.state;
        let cancellation = if matches!(state, WaiterState::Waiting { .. }) {
            let is_head = matches!(
                self.waiting,
                WaitingQueueState::Active { head, .. } if head == waiter
            );
            let demand_updates = if is_head {
                let removed = self.pop_head(eligibility_group);
                debug_assert_eq!(waiter, removed.waiter);
                let record = self
                    .records
                    .remove(&waiter)
                    .expect("cancelled head waiter disappeared");
                debug_assert!(matches!(record.state, WaiterState::Waiting { .. }));
                let retired =
                    DemandSnapshot::inactive(removed.demand.id, removed.demand.version.next());
                [Some(retired), removed.successor]
            } else {
                self.remove_non_head(waiter);
                [None, None]
            };
            WaiterCancellation {
                demand_updates,
                returned_steps: [None, None],
            }
        } else if matches!(state, WaiterState::DeliveryPending { .. }) {
            self.remove_protocol_waiters(waiter);
            let record = self.records.get_mut(&waiter)?;
            let WaiterState::DeliveryPending {
                waker,
                pending_result,
            } = &mut record.state
            else {
                unreachable!("delivery-pending waiter changed state under the cell lock");
            };
            let waker = waker.take();
            let pending_result = pending_result.take();
            record.state = WaiterState::DeliveryCancelled {
                waker,
                pending_result,
            };
            WaiterCancellation {
                demand_updates: [None, None],
                returned_steps: [None, None],
            }
        } else if matches!(
            state,
            WaiterState::ReadyToEstablish { .. } | WaiterState::Ready(_)
        ) {
            // Validate before detaching the event. Once it is local,
            // this method must return without another panic-capable check.
            self.assert_consistent();
            let record = self.records.remove(&waiter)?;
            self.remove_protocol_waiters(waiter);
            let event = match record.state {
                WaiterState::ReadyToEstablish { permit } => {
                    AcquisitionStep::StartEstablishment(permit)
                }
                WaiterState::Ready(result) => AcquisitionStep::Resolved(result),
                _ => unreachable!("ready waiter changed state under the cell lock"),
            };
            return Some(WaiterCancellation {
                demand_updates: [None, None],
                returned_steps: [Some(event), None],
            });
        } else if matches!(state, WaiterState::Launching { .. }) {
            self.assert_consistent();
            self.remove_protocol_waiters(waiter);
            self.records.remove(&waiter)?;
            return Some(WaiterCancellation {
                demand_updates: [None, None],
                returned_steps: [None, None],
            });
        } else {
            debug_assert!(matches!(state, WaiterState::DeliveryCancelled { .. }));
            return None;
        };

        self.assert_consistent();
        Some(cancellation)
    }

    /// Returns the next event or records the latest waker for a pending waiter.
    ///
    /// # Panics
    ///
    /// Panics if `waiter` is unknown, was cancelled, or was already consumed
    /// by an earlier ready poll.
    pub(super) fn poll_waiter(
        &mut self,
        waiter: WaiterId,
        cx: &mut Context<'_>,
    ) -> Poll<AcquisitionStep> {
        if matches!(
            self.records.get(&waiter).map(|record| &record.state),
            Some(WaiterState::ReadyToEstablish { .. })
        ) {
            self.assert_consistent();
            let record = self
                .records
                .get_mut(&waiter)
                .expect("ready establishment waiter disappeared");
            let previous = std::mem::replace(
                &mut record.state,
                WaiterState::Launching {
                    phase: EstablishmentPhase::Submitted,
                    waker: None,
                },
            );
            let WaiterState::ReadyToEstablish { permit } = previous else {
                unreachable!("ready establishment waiter changed state under the cell lock");
            };
            return Poll::Ready(AcquisitionStep::StartEstablishment(permit));
        }

        if matches!(
            self.records.get(&waiter).map(|record| &record.state),
            Some(WaiterState::Ready(_))
        ) {
            return Poll::Ready(
                self.take_ready_result(waiter)
                    .map(AcquisitionStep::Resolved)
                    .expect("ready waiter lost its acquisition outcome"),
            );
        }

        let record = self
            .records
            .get_mut(&waiter)
            .expect("polled a cancelled, consumed, or unknown acquisition waiter");
        let waker = match &mut record.state {
            WaiterState::Waiting { waker, .. }
            | WaiterState::DeliveryPending { waker, .. }
            | WaiterState::Launching { waker, .. } => waker,
            WaiterState::DeliveryCancelled { .. } => {
                panic!("polled a cancelled acquisition waiter")
            }
            WaiterState::ReadyToEstablish { .. } | WaiterState::Ready(_) => {
                unreachable!("ready waiter changed state under the cell lock")
            }
        };
        if waker
            .as_ref()
            .is_none_or(|waker| !waker.will_wake(cx.waker()))
        {
            *waker = Some(cx.waker().clone());
        }
        Poll::Pending
    }

    /// Transfers completion ownership to the pool immediately before the first poll.
    ///
    /// A returned H1 or cancellation that wins first removes the submitted
    /// attempt's authority, allowing its unpolled future and permit to drop.
    pub(super) fn start_establishment(&mut self, waiter: WaiterId) -> bool {
        let Some(record) = self.records.get_mut(&waiter) else {
            return false;
        };
        match &mut record.state {
            WaiterState::Launching { phase, .. } if *phase == EstablishmentPhase::Submitted => {
                *phase = EstablishmentPhase::Started;
                self.assert_consistent();
                true
            }
            WaiterState::Ready(_) => false,
            WaiterState::Launching {
                phase: EstablishmentPhase::Started,
                ..
            } => {
                panic!("establishment attempt was started more than once")
            }
            _ => {
                panic!("establishment start did not name a launching waiter")
            }
        }
    }

    /// Reserves the current head when a delivery still names its demand
    /// generation.
    ///
    /// The reserved waiter leaves the FIFO and changes in place to
    /// [`WaiterState::DeliveryPending`]. Any remaining head receives a new demand
    /// generation returned for admission acknowledgement.
    pub(super) fn reserve_delivery_waiter(
        &mut self,
        demand: DemandId,
        eligibility_group: &EligibilityGroup,
    ) -> DeliveryReservation {
        let current = matches!(
            &self.waiting,
            WaitingQueueState::Active {
                demand: current,
                ..
            } if current.id == demand
        );
        if !current {
            return DeliveryReservation::Rejected;
        }

        let removed = self.pop_head(eligibility_group);
        debug_assert_eq!(removed.demand.id, demand);
        let record = self
            .records
            .get_mut(&removed.waiter)
            .expect("reserved queue head disappeared");
        let WaiterState::Waiting { waker, .. } = &mut record.state else {
            unreachable!("reserved queue head left the waiting state");
        };
        let waker = waker.take();
        let requirement = record.requirement;
        record.state = WaiterState::DeliveryPending {
            waker,
            pending_result: None,
        };
        self.add_protocol_waiters(removed.waiter, requirement);

        self.assert_consistent();
        DeliveryReservation::Reserved {
            waiter: removed.waiter,
            successor: removed.successor,
        }
    }

    /// Completes the unlocked capacity crossing without panicking while the
    /// incoming permit remains a droppable local.
    ///
    /// A live receiver takes ownership of the permit before another
    /// panic-capable operation. If an H1 return already won, both resources
    /// leave the lock as separate events and the permit is refunnelled.
    pub(super) fn commit_capacity(
        &mut self,
        waiter: WaiterId,
        permit: EstablishmentPermit,
    ) -> CellCommitOutcome {
        let Some(record) = self.records.get_mut(&waiter) else {
            return CellCommitOutcome::invalid(
                AcquisitionStep::StartEstablishment(permit),
                CellCommitError::MissingWaiter,
            );
        };

        match &mut record.state {
            WaiterState::DeliveryPending {
                waker,
                pending_result,
            } => {
                let waker = waker.take();
                if let Some(result) = pending_result.take() {
                    record.state = WaiterState::Ready(result);
                    self.remove_protocol_waiters(waiter);
                    // The rejected permit must cross the lock boundary even
                    // if other state is already inconsistent.
                    return CellCommitOutcome::Refused {
                        returned: [Some(AcquisitionStep::StartEstablishment(permit)), None],
                        waker,
                    };
                }
                record.state = WaiterState::ReadyToEstablish { permit };
                // The permit is state-owned before this check can panic.
                self.assert_consistent();
                CellCommitOutcome::Committed { waker }
            }
            WaiterState::DeliveryCancelled {
                waker,
                pending_result,
            } => {
                let waker = waker.take();
                let pending_result = pending_result.take();
                self.records.remove(&waiter);
                CellCommitOutcome::Refused {
                    returned: [
                        pending_result.map(AcquisitionStep::Resolved),
                        Some(AcquisitionStep::StartEstablishment(permit)),
                    ],
                    waker,
                }
            }
            WaiterState::Waiting { .. }
            | WaiterState::ReadyToEstablish { .. }
            | WaiterState::Launching { .. }
            | WaiterState::Ready(_) => CellCommitOutcome::invalid(
                AcquisitionStep::StartEstablishment(permit),
                CellCommitError::UnexpectedState,
            ),
        }
    }

    /// Installs a borrowed HTTP/1 result after its requesting waiter was reserved.
    ///
    /// A local return may have won while the borrowed sender crossed its
    /// owning-cell and requesting-cell locks. In that case the local result stays ready and
    /// the borrowed result leaves the lock for ordinary owning-cell return.
    pub(super) fn commit_borrowed_h1(
        &mut self,
        waiter: WaiterId,
        result: AcquisitionOutcome,
    ) -> CellCommitOutcome {
        let Some(record) = self.records.get_mut(&waiter) else {
            return CellCommitOutcome::invalid(
                AcquisitionStep::Resolved(result),
                CellCommitError::MissingWaiter,
            );
        };

        match &mut record.state {
            WaiterState::DeliveryPending {
                waker,
                pending_result,
            } => {
                let waker = waker.take();
                if let Some(local_result) = pending_result.take() {
                    record.state = WaiterState::Ready(local_result);
                    self.remove_protocol_waiters(waiter);
                    // The rejected borrowed sender must cross the lock
                    // boundary before any panic-capable consistency check.
                    return CellCommitOutcome::Refused {
                        returned: [Some(AcquisitionStep::Resolved(result)), None],
                        waker,
                    };
                }
                record.state = WaiterState::Ready(result);
                self.remove_protocol_waiters(waiter);
                self.assert_consistent();
                CellCommitOutcome::Committed { waker }
            }
            WaiterState::DeliveryCancelled {
                waker,
                pending_result,
            } => {
                let waker = waker.take();
                let pending_result = pending_result.take();
                self.records.remove(&waiter);
                CellCommitOutcome::Refused {
                    returned: [
                        pending_result.map(AcquisitionStep::Resolved),
                        Some(AcquisitionStep::Resolved(result)),
                    ],
                    waker,
                }
            }
            WaiterState::Waiting { .. }
            | WaiterState::ReadyToEstablish { .. }
            | WaiterState::Launching { .. }
            | WaiterState::Ready(_) => CellCommitOutcome::invalid(
                AcquisitionStep::Resolved(result),
                CellCommitError::UnexpectedState,
            ),
        }
    }

    /// Commits one terminal establishment result when its waiter is still live.
    ///
    /// Completion may occur from either launch phase: normal execution starts
    /// the connector first, while dropping an unpolled submitted task installs
    /// a terminal runtime error.
    pub(super) fn commit_establishment(
        &mut self,
        waiter: WaiterId,
        result: AcquisitionOutcome,
    ) -> CellCommitOutcome {
        let Some(record) = self.records.get_mut(&waiter) else {
            return CellCommitOutcome::refused(AcquisitionStep::Resolved(result));
        };
        if matches!(record.state, WaiterState::Ready(_)) {
            return CellCommitOutcome::refused(AcquisitionStep::Resolved(result));
        }
        if !matches!(record.state, WaiterState::Launching { .. }) {
            return CellCommitOutcome::invalid(
                AcquisitionStep::Resolved(result),
                CellCommitError::UnexpectedState,
            );
        }
        let WaiterState::Launching { waker, .. } = &mut record.state else {
            unreachable!("validated launching waiter changed state")
        };
        let waker = waker.take();
        record.state = WaiterState::Ready(result);
        self.remove_protocol_waiters(waiter);
        self.assert_consistent();
        CellCommitOutcome::Committed { waker }
    }

    /// Removes a ready waiter and transfers ownership of its result.
    pub(super) fn take_ready_result(&mut self, waiter: WaiterId) -> Option<AcquisitionOutcome> {
        if !matches!(
            self.records.get(&waiter).map(|record| &record.state),
            Some(WaiterState::Ready(_))
        ) {
            return None;
        }

        // Validate before detaching the result. Returning it must be the last
        // operation performed while the cell lock is held.
        self.assert_consistent();
        let record = self.records.remove(&waiter)?;
        let WaiterState::Ready(result) = record.state else {
            unreachable!("ready waiter changed state under the cell lock");
        };
        Some(result)
    }

    /// Offers an H2 activation to the oldest compatible acquisition.
    ///
    /// `cutoff` retains peer-route priority. A later waiter is left pending
    /// until every live acquisition at or before the cutoff has received an
    /// activation opportunity.
    pub(super) fn offer_h2_activation(
        &mut self,
        cutoff: Option<WaiterId>,
        result: impl FnOnce(WaiterId) -> AcquisitionOutcome,
        eligibility_group: &EligibilityGroup,
    ) -> (Option<WaiterId>, WaiterResolution) {
        let Some(waiter) = self.oldest_h2_compatible_waiter() else {
            return (None, WaiterResolution::empty());
        };
        if cutoff.is_some_and(|cutoff| waiter > cutoff) {
            return (None, WaiterResolution::empty());
        }

        (
            Some(waiter),
            self.assign_protocol_outcome(waiter, || result(waiter), eligibility_group),
        )
    }

    /// Unlinks the FIFO head and installs any successor demand.
    ///
    /// The head record remains in `records`; the caller either changes it to
    /// `DeliveryPending` or removes it for cancellation.
    fn pop_head(&mut self, eligibility_group: &EligibilityGroup) -> RemovedHead {
        let waiting = std::mem::take(&mut self.waiting);
        let WaitingQueueState::Active {
            head,
            tail,
            len,
            demand,
        } = waiting
        else {
            unreachable!("removed a head from an empty waiter queue");
        };
        let next = match &self
            .records
            .get(&head)
            .expect("waiting head disappeared")
            .state
        {
            WaiterState::Waiting { previous, next, .. } => {
                debug_assert!(previous.is_none());
                *next
            }
            _ => unreachable!("waiting head left the waiting state"),
        };

        let successor = match next {
            Some(next) => {
                let requirement = {
                    let next_record = self
                        .records
                        .get_mut(&next)
                        .expect("next waiter disappeared");
                    let WaiterState::Waiting { previous, .. } = &mut next_record.state else {
                        unreachable!("next waiter left the waiting state");
                    };
                    debug_assert_eq!(*previous, Some(head));
                    *previous = None;
                    next_record.requirement
                };

                let next_demand = self.new_demand(requirement);
                let snapshot = next_demand.snapshot(eligibility_group);
                let len = NonZeroUsize::new(
                    len.get()
                        .checked_sub(1)
                        .expect("waiter queue length underflowed"),
                )
                .expect("nonempty waiter queue lost its length");
                self.waiting = WaitingQueueState::Active {
                    head: next,
                    tail,
                    len,
                    demand: next_demand,
                };
                Some(snapshot)
            }
            None => {
                debug_assert_eq!(head, tail);
                debug_assert_eq!(len, NonZeroUsize::MIN);
                None
            }
        };

        RemovedHead {
            waiter: head,
            demand,
            successor,
        }
    }

    /// Removes a non-head waiting record without changing aggregate demand.
    fn remove_non_head(&mut self, waiter: WaiterId) -> WaiterRecord {
        let record = self
            .records
            .remove(&waiter)
            .expect("removed waiter disappeared");
        let (previous, next) = match &record.state {
            WaiterState::Waiting { previous, next, .. } => (*previous, *next),
            _ => unreachable!("removed waiter left the waiting state"),
        };
        let previous = previous.expect("non-head waiter had no predecessor");

        let previous_record = self
            .records
            .get_mut(&previous)
            .expect("previous waiter disappeared");
        let WaiterState::Waiting {
            next: previous_next,
            ..
        } = &mut previous_record.state
        else {
            unreachable!("previous waiter left the waiting state");
        };
        debug_assert_eq!(*previous_next, Some(waiter));
        *previous_next = next;

        if let Some(next) = next {
            let next_record = self
                .records
                .get_mut(&next)
                .expect("next waiter disappeared");
            let WaiterState::Waiting {
                previous: next_previous,
                ..
            } = &mut next_record.state
            else {
                unreachable!("next waiter left the waiting state");
            };
            debug_assert_eq!(*next_previous, Some(waiter));
            *next_previous = Some(previous);
        }

        let WaitingQueueState::Active {
            head, tail, len, ..
        } = &mut self.waiting
        else {
            unreachable!("removed a waiter from an empty queue");
        };
        debug_assert_ne!(*head, waiter);
        if next.is_none() {
            debug_assert_eq!(*tail, waiter);
            *tail = previous;
        }
        *len = NonZeroUsize::new(
            len.get()
                .checked_sub(1)
                .expect("waiter queue length underflowed"),
        )
        .expect("removing a non-head waiter emptied the queue");
        record
    }

    /// Allocates an identity that is never reused within this cell.
    fn take_waiter_id(&mut self) -> WaiterId {
        let value = self.next_waiter_id;
        self.next_waiter_id = value.checked_add(1).expect("waiter identity exhausted");
        WaiterId(value)
    }

    /// Starts a demand generation for a waiter that became the FIFO head.
    fn new_demand(&mut self, requirement: ProtocolRequirement) -> DemandTicket {
        let value = self.next_demand_id;
        self.next_demand_id = value.checked_add(1).expect("demand identity exhausted");
        DemandTicket {
            id: DemandId::from_u64(value),
            version: SnapshotVersion::INITIAL,
            requirement,
        }
    }

    /// Checks map, FIFO, and demand relationships in debug and test builds.
    pub(super) fn assert_consistent(&self) {
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
        let waiting_records = self
            .records
            .values()
            .filter(|record| matches!(record.state, WaiterState::Waiting { .. }))
            .count();
        match &self.waiting {
            WaitingQueueState::Empty => {
                assert_eq!(0, waiting_records, "empty queue retained waiting records");
            }
            WaitingQueueState::Active {
                head,
                tail,
                len,
                demand,
            } => {
                let head_record = self.records.get(head).expect("waiting head disappeared");
                assert_eq!(
                    demand.requirement, head_record.requirement,
                    "aggregate demand did not describe the waiting head"
                );

                let mut current = Some(*head);
                let mut previous = None;
                let mut traversed = 0;
                while let Some(waiter) = current {
                    assert!(
                        traversed < self.records.len(),
                        "waiter queue contains a cycle"
                    );
                    let record = self
                        .records
                        .get(&waiter)
                        .expect("linked waiter disappeared");
                    let WaiterState::Waiting {
                        previous: linked_previous,
                        next,
                        ..
                    } = &record.state
                    else {
                        panic!("linked waiter left the waiting state");
                    };
                    assert_eq!(
                        previous, *linked_previous,
                        "waiter queue contains inconsistent backward links"
                    );
                    traversed += 1;
                    previous = Some(waiter);
                    current = *next;
                }

                assert_eq!(Some(*tail), previous, "waiting tail was not reachable");
                assert_eq!(
                    len.get(),
                    traversed,
                    "waiter queue length did not match its links"
                );
                assert_eq!(
                    waiting_records, traversed,
                    "waiting record was not reachable from the queue head"
                );
            }
        }

        let expected_h1_waiters = self
            .records
            .iter()
            .filter_map(|(waiter, record)| {
                (record.requirement.accepts_h1()
                    && matches!(
                        record.state,
                        WaiterState::DeliveryPending {
                            pending_result: None,
                            ..
                        } | WaiterState::ReadyToEstablish { .. }
                            | WaiterState::Launching { .. }
                    ))
                .then_some(*waiter)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected_h1_waiters, self.h1_compatible_waiters,
            "HTTP/1 compatible-waiter index did not match waiter state"
        );

        let expected_h2_waiters = self
            .records
            .iter()
            .filter_map(|(waiter, record)| {
                (record.requirement.accepts_h2()
                    && matches!(
                        record.state,
                        WaiterState::DeliveryPending {
                            pending_result: None,
                            ..
                        } | WaiterState::ReadyToEstablish { .. }
                            | WaiterState::Launching { .. }
                    ))
                .then_some(*waiter)
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            expected_h2_waiters, self.h2_compatible_waiters,
            "HTTP/2 compatible-waiter index did not match waiter state"
        );
    }

    /// Returns requests that have not received a terminal acquisition outcome.
    pub(super) fn pending_count(&self) -> usize {
        self.records
            .values()
            .filter(|record| {
                matches!(
                    record.state,
                    WaiterState::Waiting { .. }
                        | WaiterState::DeliveryPending { .. }
                        | WaiterState::ReadyToEstablish { .. }
                        | WaiterState::Launching { .. }
                )
            })
            .count()
    }

    #[cfg(test)]
    pub(super) fn probe(&self) -> AcquisitionProbe {
        let (waiting, demand) = match &self.waiting {
            WaitingQueueState::Empty => (0, None),
            WaitingQueueState::Active { len, demand, .. } => (len.get(), Some(demand.id)),
        };
        AcquisitionProbe {
            waiting,
            retained: self.records.len(),
            demand,
        }
    }
}

/// Result of reserving a requesting waiter for one unlocked delivery.
pub(super) enum DeliveryReservation {
    /// The current head was reserved and may have a successor demand.
    Reserved {
        waiter: WaiterId,
        successor: Option<DemandSnapshot>,
    },
    /// The delivery no longer matches live cell demand.
    Rejected,
}

/// Values detached from cell state by cancellation.
pub(super) struct WaiterCancellation {
    /// Demand retirement and optional successor published after unlocking.
    pub(super) demand_updates: [Option<DemandSnapshot>; 2],
    /// Intermediate and terminal events returned for cleanup after unlocking.
    pub(super) returned_steps: [Option<AcquisitionStep>; 2],
}

/// Values detached after offering a selected protocol result.
pub(super) struct WaiterResolution {
    /// Demand retirement and optional successor published after unlocking.
    pub(super) demand_updates: [Option<DemandSnapshot>; 2],
    /// Establishment permit or rejected result returned after unlocking.
    pub(super) returned_step: Option<AcquisitionStep>,
    /// Waiting task woken after demand publication.
    pub(super) waker: Option<Waker>,
}

impl WaiterResolution {
    /// Preserves a result when no live attempt can accept HTTP/1.
    fn refused(result: AcquisitionOutcome) -> Self {
        Self {
            demand_updates: [None, None],
            returned_step: Some(AcquisitionStep::Resolved(result)),
            waker: None,
        }
    }

    /// Reports that no live attempt can accept an HTTP/2 activation.
    fn empty() -> Self {
        Self {
            demand_updates: [None, None],
            returned_step: None,
            waker: None,
        }
    }
}

/// Invalid state observed while a committed event was crossing to the cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CellCommitError {
    /// The waiter reserved for the delivery no longer exists.
    MissingWaiter,
    /// The waiter exists but cannot receive the reserved delivery.
    UnexpectedState,
}

/// Result of committing one exact acquisition step to a reserved waiter.
pub(super) enum CellCommitOutcome {
    /// Cell state accepted the step and became authoritative for it.
    Committed {
        /// Waiting task woken after assignment settlement.
        waker: Option<Waker>,
    },
    /// The waiter no longer accepted the step; fallback remains valid.
    Refused {
        /// Steps returned after unlocking.
        returned: [Option<AcquisitionStep>; 2],
        /// Waiting task woken after assignment settlement.
        waker: Option<Waker>,
    },
    /// The reserved waiter disappeared or entered an impossible state.
    Invalid {
        /// Steps returned before reporting the invariant failure.
        returned: [Option<AcquisitionStep>; 2],
        /// Waiting task, if one remained attached.
        waker: Option<Waker>,
        /// Invalid cell transition observed under the lock.
        error: CellCommitError,
    },
}

impl CellCommitOutcome {
    /// Preserves the incoming step for unlocked cleanup after invalid state.
    fn invalid(step: AcquisitionStep, error: CellCommitError) -> Self {
        Self::Invalid {
            returned: [Some(step), None],
            waker: None,
            error,
        }
    }

    /// Preserves a losing step for its ordinary unlocked fallback.
    fn refused(step: AcquisitionStep) -> Self {
        Self::Refused {
            returned: [Some(step), None],
            waker: None,
        }
    }
}

/// FIFO state detached while advancing the active head.
struct RemovedHead {
    /// Identity of the unlinked waiter.
    waiter: WaiterId,
    /// Retired demand generation that represented this head.
    demand: DemandTicket,
    /// New demand published if another waiter became the head.
    successor: Option<DemandSnapshot>,
}

#[cfg(test)]
#[derive(Debug)]
pub(super) struct AcquisitionProbe {
    pub(super) waiting: usize,
    pub(super) retained: usize,
    pub(super) demand: Option<DemandId>,
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;

    #[test]
    fn h1_required_waiter_is_not_h2_compatible() {
        let mut queue = AcquisitionQueue::default();
        let (waiter, snapshot) = queue.register_waiter(
            ProtocolRequirement::H1Required,
            &EligibilityGroup::Pool,
            false,
        );

        assert!(snapshot.is_none());
        assert_eq!(Some(waiter), queue.oldest_h1_compatible_waiter());
        assert_eq!(None, queue.oldest_h2_compatible_waiter());
    }

    #[test]
    fn head_cancellation_uses_the_successors_protocol_requirement() {
        let mut queue = AcquisitionQueue::default();
        let (head, initial) = queue.register_waiter(
            ProtocolRequirement::H2Required,
            &EligibilityGroup::Pool,
            true,
        );
        let (_successor, no_new_demand) = queue.register_waiter(
            ProtocolRequirement::H1Compatible,
            &EligibilityGroup::Pool,
            true,
        );
        assert!(initial.is_some());
        assert!(no_new_demand.is_none());

        let cancelled = queue
            .cancel_waiter(head, &EligibilityGroup::Pool)
            .expect("head waiter was not cancelled");
        assert_eq!(
            Some(DemandSnapshot::active(
                DemandId::from_u64(1),
                SnapshotVersion::INITIAL,
                ProtocolRequirement::H1Compatible,
                EligibilityGroup::Pool,
            )),
            cancelled.demand_updates[1]
        );
    }

    #[test]
    fn retired_demand_cannot_reserve_the_successor() {
        let mut queue = AcquisitionQueue::default();
        let (head, _initial) = queue.register_waiter(
            ProtocolRequirement::H1Compatible,
            &EligibilityGroup::Pool,
            true,
        );
        queue.register_waiter(
            ProtocolRequirement::H1Compatible,
            &EligibilityGroup::Pool,
            true,
        );
        queue
            .cancel_waiter(head, &EligibilityGroup::Pool)
            .expect("head waiter was not cancelled");

        assert!(matches!(
            queue.reserve_delivery_waiter(DemandId::from_u64(0), &EligibilityGroup::Pool),
            DeliveryReservation::Rejected
        ));
    }
}
