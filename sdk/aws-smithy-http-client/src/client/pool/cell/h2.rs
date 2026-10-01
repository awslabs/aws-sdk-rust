/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/2 connection generations, routes, and request activation.
//!
//! HTTP/2 carries many concurrent request and response exchanges as streams on
//! one connection. The pool calls one installed lifetime of that connection a
//! generation. The cell that established the connection owns its generation,
//! including the Hyper request handle, driver, transport, and capacity.
//!
//! A flight is the post-ALPN handshake work shared by concurrent requests
//! waiting for that generation. A successful flight installs one accepting
//! generation; failed or superseded flight work installs nothing.
//!
//! An accepting generation may issue new request streams. A draining
//! generation issues no new streams but remains recorded while selected or
//! accepted requests still own it.
//!
//! A requesting cell may store an [`H2Route`] to a peer cell's generation.
//! The route carries only cell and generation identity. [`H2Activation`]
//! revalidates that identity under the connection-cell lock, reserves one
//! prospective stream, and carries a transient sender clone to Hyper.
//!
//! Each Hyper-accepted request owns one claim with independent upload and
//! response guards. The generation releases the request only after both sides
//! finish, including drop, error, and bodyless completion paths. The detached
//! request lifecycle lives in [`request`]; this module retains the cell-locked
//! generation, route, and activation-gate state.
//!
//! [`H2CellState`] is the invariant owner under the cell lock. It stores one
//! flight, accepting and draining generations, activation priority, and one
//! peer route. A peer route references another cell's generation; it does not
//! transfer connection ownership.
//!
//! ```text
//! no flight -- post-ALPN owner task ----------------------> Flight
//! Flight -- successful handshake and install ------------> Accepting
//! Flight -- failure, stale completion, or task drop ------> no flight
//! Accepting -- close with retained request claims --------> Draining
//! Accepting -- close without retained request claims -----> removed
//! Draining -- last prospective or accepted claim --------> removed
//! ```
//!
//! ```text
//! requesting cell                        connection-owning cell
//! peer route + local gate --identity---> exact accepting generation
//!                                      |-- reserve one prospective stream
//!                                      `-- clone transient sender
//!                                               |
//!                                               `--> H2Activation
//! ```
//!
//! Generation installation makes the sender visible before the owner task
//! submits the Hyper driver. An activation accepted in that interval remains
//! pending in Hyper's dispatch channel until the driver is polled.
//!
//! Activation reserves its prospective claim before the sender clone leaves
//! the cell lock. Hyper acceptance converts that reservation to an accepted
//! request claim with independent upload and response guards.

mod request;

pub(in crate::client::pool) use request::{
    H2Activation, H2DispatchParts, H2ResponseGuard, H2UploadGuard,
};
use request::{H2ActivationResources, H2ActivationTurnGuard};

use super::super::admission::{
    DemandId, DemandSnapshot, H2SupplyStatus, OriginAdmission, SupplyRevision,
};
use super::super::connection::{CloseReason, ConnectionState};
use super::super::partition::{EligibilityGroup, PartitionId};
use super::waiters::{AcquisitionQueue, WaiterResolution};
use super::{AcquisitionOutcome, AcquisitionStep, CellState, OriginCell, WaiterId};
use crate::sync::{Arc, Weak};
use aws_smithy_runtime_api::client::connection::ConnectionId;
use aws_smithy_types::body::SdkBody;
use std::collections::{BTreeSet, HashMap};
use std::time::SystemTime;

/// Identity of one post-ALPN HTTP/2 establishment flight.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) struct H2FlightId(u64);

/// Identity of one installed HTTP/2 generation within a cell.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::client::pool) struct H2GenerationId(u64);

#[cfg(test)]
impl H2GenerationId {
    /// Creates a generation identity for admission-index tests.
    pub(in crate::client::pool) fn for_test(value: u64) -> Self {
        Self(value)
    }
}

/// Cloneable Hyper request sender retained only by its owning generation.
///
/// The test-only variant exercises generation and claim transitions without a
/// parallel fake generation implementation.
#[derive(Clone)]
pub(in crate::client::pool) enum H2Sender {
    /// Sender returned by a successful Hyper HTTP/2 handshake.
    Hyper(hyper::client::conn::http2::SendRequest<SdkBody>),
    /// Synthetic sender identity used by state-machine tests.
    #[cfg(test)]
    Test(u64),
}

impl H2Sender {
    /// Wraps a sender produced by a successful Hyper HTTP/2 handshake.
    pub(in crate::client::pool) fn from_hyper(
        inner: hyper::client::conn::http2::SendRequest<SdkBody>,
    ) -> Self {
        Self::Hyper(inner)
    }

    /// Returns whether Hyper has observed connection closure.
    pub(in crate::client::pool) fn is_closed(&self) -> bool {
        match self {
            Self::Hyper(sender) => sender.is_closed(),
            #[cfg(test)]
            Self::Test(_) => false,
        }
    }

    /// Returns mutable access to a transient sender clone for dispatch.
    ///
    /// # Panics
    ///
    /// Panics when a test-only sender reaches real dispatch.
    pub(in crate::client::pool) fn hyper_mut(
        &mut self,
    ) -> &mut hyper::client::conn::http2::SendRequest<SdkBody> {
        match self {
            Self::Hyper(sender) => sender,
            #[cfg(test)]
            Self::Test(_) => panic!("test HTTP/2 sender reached Hyper dispatch"),
        }
    }

    /// Creates a synthetic sender for state-machine tests.
    #[cfg(test)]
    fn test(id: u64) -> Self {
        Self::Test(id)
    }
}

impl std::fmt::Debug for H2Sender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hyper(_) => f.write_str("H2Sender::Hyper"),
            #[cfg(test)]
            Self::Test(id) => f.debug_tuple("H2Sender::Test").field(id).finish(),
        }
    }
}

/// Identity-only reference to one accepting generation.
///
/// The route owns no sender, connection capacity, driver, or socket. Every
/// activation upgrades the cell reference and checks the generation identity.
#[derive(Clone, Debug)]
pub(in crate::client::pool) struct H2Route {
    id: H2RouteId,
    connection_cell: Weak<OriginCell>,
}

/// Exact connection-owning generation named by one peer route.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct H2RouteId {
    connection_partition: PartitionId,
    generation: H2GenerationId,
}

impl H2Route {
    /// Creates a route after its generation has been installed.
    pub(in crate::client::pool) fn new(
        connection_cell: &Arc<OriginCell>,
        generation: H2GenerationId,
    ) -> Self {
        Self {
            connection_cell: Weak::from_arc(connection_cell),
            id: H2RouteId {
                connection_partition: connection_cell.id().partition(),
                generation,
            },
        }
    }

    /// Returns the exact connection-owning generation named by this route.
    fn id(&self) -> H2RouteId {
        self.id
    }

    /// Returns the partition that owns the routed generation.
    pub(super) fn connection_partition(&self) -> PartitionId {
        self.id.connection_partition
    }

    /// Returns the routed generation identity.
    #[cfg(any(debug_assertions, test))]
    pub(in crate::client::pool) fn generation(&self) -> H2GenerationId {
        self.id.generation
    }

    /// Attempts to reserve a prospective request claim for one requesting partition.
    pub(super) fn activate(&self, request_partition: PartitionId) -> Option<H2Activation> {
        let cell = self.connection_cell.upgrade()?;
        OriginCell::activate_h2(&cell, self.id.generation, request_partition)
    }
}

/// Complete HTTP/2 acquisition state owned by one origin cell.
///
/// At every completed transition:
///
/// - at most one flight and one accepting generation exist;
/// - the accepting identity names an `Accepting` record;
/// - every other generation is `Draining`;
/// - prospective and active request counts are checked and non-wrapping;
/// - a draining record remains until both counts reach zero; and
/// - each flight participant identity appears at most once;
/// - the local activation gate names the accepting generation;
/// - a peer route gate names the routed generation; and
/// - at most one peer-route waiter is crossing the connection-cell lock.
#[derive(Debug, Default)]
pub(super) struct H2CellState {
    /// Post-ALPN convergence in progress for this cell.
    flight: Option<H2Flight>,
    /// Installed accepting and draining generations by exact identity.
    generations: HashMap<H2GenerationId, H2Generation>,
    /// Sole generation permitted to issue new activations.
    accepting_generation: Option<H2GenerationId>,
    /// Identity-only route to one peer cell's accepting generation.
    peer_route: Option<PeerH2Route>,
    /// Local activation order for the accepting generation.
    local_activation_gate: H2ActivationGate,
    /// Next cell-local flight identity.
    next_flight_id: u64,
    /// Next cell-local generation identity.
    next_generation_id: u64,
}

/// One post-ALPN flight and the waiters awaiting its result.
#[derive(Debug)]
struct H2Flight {
    /// Identity checked by the owner task at completion.
    id: H2FlightId,
    /// Waiters that receive the shared handshake result.
    participants: BTreeSet<WaiterId>,
}

/// One installed multiplexed connection generation.
#[derive(Debug)]
struct H2Generation {
    /// Establishment waiters transferred to this generation but not yet served.
    pending_waiters: BTreeSet<WaiterId>,
    /// Protocol-neutral connection and capacity owner.
    connection: Arc<ConnectionState>,
    /// Authoritative Hyper sender cloned only after generation validation.
    sender: H2Sender,
    /// Whether new activations may be issued.
    state: H2GenerationState,
    /// Activations selected but not yet accepted by Hyper.
    prospective_requests: usize,
    /// Whether Hyper has accepted a request on this generation.
    has_accepted_request: bool,
    /// Requests accepted by Hyper whose upload and response sides have not both finished.
    active_requests: usize,
    /// Expiration deadline while the generation remains accepting.
    idle_deadline: Option<SystemTime>,
}

/// Whether an installed generation may accept new requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum H2GenerationState {
    /// New requests may reserve prospective claims.
    Accepting,
    /// No new requests are admitted; retained claims may finish.
    Draining,
}

/// Peer route visibility and requesting-cell activation order.
#[derive(Debug)]
struct PeerH2Route {
    /// Exact connection-cell generation visible to this requesting cell.
    route: H2Route,
    /// Local waiter priority for uses of this route.
    activation_gate: H2ActivationGate,
    /// Waiter whose route activation is crossing the connection-cell lock.
    crossing_waiter: Option<WaiterId>,
}

/// Local priority state for one accepting generation.
#[derive(Debug, Default)]
enum H2ActivationGate {
    /// No accepting generation is visible.
    #[default]
    Closed,
    /// Waiters committed through `cutoff` precede later arrivals.
    Prioritizing {
        /// Exact generation governed by this gate.
        generation: H2GenerationId,
        /// Newest waiter committed before generation visibility.
        cutoff: WaiterId,
        /// Prioritized waiter whose activation has not accepted or cancelled.
        active_turn: Option<WaiterId>,
    },
    /// The peer-route cutoff drained; queued work still precedes direct arrivals.
    Open { generation: H2GenerationId },
}

/// One activation opportunity returned by a generation gate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum H2ActivationTurn {
    /// The gate cannot issue another activation.
    Unavailable,
    /// Any oldest compatible acquisition may activate.
    Open,
    /// Only an acquisition at or before this waiter may activate.
    Through(WaiterId),
}

impl H2ActivationGate {
    /// Creates a gate over every waiter committed through `cutoff`.
    fn for_generation(generation: H2GenerationId, cutoff: Option<WaiterId>) -> Self {
        match cutoff {
            Some(cutoff) => Self::Prioritizing {
                generation,
                cutoff,
                active_turn: None,
            },
            None => Self::Open { generation },
        }
    }

    fn generation(&self) -> Option<H2GenerationId> {
        match self {
            Self::Closed => None,
            Self::Prioritizing { generation, .. } | Self::Open { generation } => Some(*generation),
        }
    }

    /// Returns the next activation opportunity and opens a drained priority gate.
    fn take_next_turn(&mut self, has_prioritized: bool) -> H2ActivationTurn {
        match self {
            Self::Closed => H2ActivationTurn::Unavailable,
            Self::Prioritizing {
                cutoff,
                active_turn,
                generation,
            } => {
                if active_turn.is_some() {
                    return H2ActivationTurn::Unavailable;
                }
                if !has_prioritized {
                    let generation = *generation;
                    *self = Self::Open { generation };
                    H2ActivationTurn::Open
                } else {
                    H2ActivationTurn::Through(*cutoff)
                }
            }
            Self::Open { .. } => H2ActivationTurn::Open,
        }
    }

    /// Records a prioritized activation until it accepts or cancels.
    fn reserve_turn(&mut self, waiter: WaiterId) -> bool {
        let active_turn = match self {
            Self::Closed => {
                unreachable!("started an HTTP/2 activation while its gate was closed")
            }
            Self::Prioritizing { active_turn, .. } => active_turn,
            Self::Open { .. } => return false,
        };
        assert!(
            active_turn.replace(waiter).is_none(),
            "HTTP/2 generation gate admitted two activation opportunities"
        );
        true
    }

    /// Discharges one exact activation opportunity.
    fn release_turn(&mut self, generation: H2GenerationId, waiter: WaiterId) -> bool {
        if self.generation() != Some(generation) {
            return false;
        }
        let active_turn = match self {
            Self::Closed => return false,
            Self::Prioritizing { active_turn, .. } => active_turn,
            Self::Open { .. } => return false,
        };
        if *active_turn != Some(waiter) {
            return false;
        }
        *active_turn = None;
        true
    }

    fn priority_cutoff(&self) -> Option<WaiterId> {
        match self {
            Self::Prioritizing { cutoff, .. } => Some(*cutoff),
            Self::Closed | Self::Open { .. } => None,
        }
    }

    fn active_turn(&self) -> Option<WaiterId> {
        match self {
            Self::Prioritizing { active_turn, .. } => *active_turn,
            Self::Closed | Self::Open { .. } => None,
        }
    }

    fn is_open_for(&self, generation: H2GenerationId) -> bool {
        matches!(self, Self::Open { generation: current } if *current == generation)
    }
}

/// Result of atomically converging one post-ALPN attempt.
#[derive(Debug)]
pub(in crate::client::pool) enum H2FlightDecision {
    /// An installed generation can serve this waiter.
    UseGeneration(H2GenerationId),
    /// The waiter joined the current flight as a result participant.
    JoinedFlight,
    /// The caller owns the task that must drive this new flight.
    RunFlight(H2FlightId),
    /// Another transition already installed or retired the waiter's result.
    WaiterResolved,
}

/// Result of joining an accepting generation after ALPN selected HTTP/2.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::client::pool) enum H2GenerationJoinOutcome {
    /// The waiter was transferred to the named generation.
    Joined,
    /// The named generation stopped accepting before the transfer.
    GenerationChanged,
    /// Another transition already installed the waiter's result.
    WaiterResolved,
}

impl H2CellState {
    /// Returns accepting-generation and active-request counts.
    pub(super) fn connection_counts(&self) -> (usize, usize) {
        self.generations
            .values()
            .fold((0, 0), |(accepting, active_requests), record| {
                (
                    accepting + usize::from(record.state == H2GenerationState::Accepting),
                    active_requests + record.active_requests,
                )
            })
    }

    /// Uses an accepting generation, joins its flight, or starts a new flight.
    pub(super) fn converge_flight(&mut self, waiter: WaiterId) -> H2FlightDecision {
        if let Some(generation) = self.accepting_generation {
            return H2FlightDecision::UseGeneration(generation);
        }
        if let Some(flight) = &mut self.flight {
            assert!(
                flight.participants.insert(waiter),
                "HTTP/2 waiter joined one flight more than once"
            );
            self.assert_consistent();
            return H2FlightDecision::JoinedFlight;
        }

        let id = self.take_flight_id();
        self.flight = Some(H2Flight {
            id,
            participants: BTreeSet::from([waiter]),
        });
        self.assert_consistent();
        H2FlightDecision::RunFlight(id)
    }

    /// Removes one cancelled waiter from its current flight, when present.
    pub(super) fn cancel_flight_participant(&mut self, waiter: WaiterId) {
        if let Some(flight) = &mut self.flight {
            flight.participants.remove(&waiter);
        }
        self.assert_consistent();
    }

    /// Removes a waiter that no longer awaits an accepting generation.
    pub(super) fn cancel_pending_waiter(&mut self, waiter: WaiterId) {
        if let Some(generation) = self.accepting_generation {
            self.generations
                .get_mut(&generation)
                .expect("accepting HTTP/2 generation disappeared")
                .pending_waiters
                .remove(&waiter);
        }
        self.assert_consistent();
    }

    /// Installs the generation produced by the named flight.
    pub(in crate::client::pool) fn complete_flight(
        &mut self,
        flight: H2FlightId,
        connection: Arc<ConnectionState>,
        sender: H2Sender,
        idle_deadline: Option<SystemTime>,
    ) -> Result<H2GenerationId, (Arc<ConnectionState>, H2Sender)> {
        if self.flight.as_ref().map(|current| current.id) != Some(flight)
            || self.accepting_generation.is_some()
        {
            return Err((connection, sender));
        }
        let flight = self
            .flight
            .take()
            .expect("validated HTTP/2 flight disappeared");
        let generation =
            self.open_generation(flight.participants, connection, sender, idle_deadline);
        self.assert_consistent();
        Ok(generation)
    }

    /// Installs one accepting generation with its transferred waiters.
    fn open_generation(
        &mut self,
        pending_waiters: BTreeSet<WaiterId>,
        connection: Arc<ConnectionState>,
        sender: H2Sender,
        idle_deadline: Option<SystemTime>,
    ) -> H2GenerationId {
        let generation = self.take_generation_id();
        let replaced = self.generations.insert(
            generation,
            H2Generation {
                pending_waiters,
                connection,
                sender,
                state: H2GenerationState::Accepting,
                prospective_requests: 0,
                has_accepted_request: false,
                active_requests: 0,
                idle_deadline,
            },
        );
        assert!(replaced.is_none(), "HTTP/2 generation identity was reused");
        self.accepting_generation = Some(generation);
        self.peer_route = None;
        debug_assert!(matches!(
            self.local_activation_gate,
            H2ActivationGate::Closed
        ));
        self.local_activation_gate = H2ActivationGate::Open { generation };
        generation
    }

    /// Retires a failed or dropped flight and returns its participants.
    pub(super) fn fail_flight(&mut self, flight: H2FlightId) -> Option<Vec<WaiterId>> {
        if self.flight.as_ref().map(|current| current.id) != Some(flight) {
            return None;
        }
        let participants = self
            .flight
            .take()
            .expect("validated HTTP/2 flight disappeared")
            .participants
            .into_iter()
            .collect();
        self.assert_consistent();
        Some(participants)
    }

    /// Closes the gate over waiters committed when a generation became visible.
    pub(super) fn prioritize_through(&mut self, cutoff: Option<WaiterId>) {
        let generation = self
            .accepting_generation
            .expect("HTTP/2 generation gate opened without an accepting generation");
        self.local_activation_gate = H2ActivationGate::for_generation(generation, cutoff);
        self.assert_consistent();
    }

    /// Extends priority to a post-ALPN waiter that found this generation.
    pub(super) fn prioritize_waiter(
        &mut self,
        generation: H2GenerationId,
        waiter: WaiterId,
    ) -> bool {
        if self.accepting_generation != Some(generation) {
            return false;
        }
        match &mut self.local_activation_gate {
            H2ActivationGate::Closed => {
                unreachable!("accepting HTTP/2 generation had no generation gate")
            }
            H2ActivationGate::Prioritizing { cutoff, .. } => {
                *cutoff = (*cutoff).max(waiter);
            }
            H2ActivationGate::Open { .. } => {
                self.local_activation_gate = H2ActivationGate::Prioritizing {
                    generation,
                    cutoff: waiter,
                    active_turn: None,
                };
            }
        }
        self.generations
            .get_mut(&generation)
            .expect("accepting HTTP/2 generation disappeared")
            .pending_waiters
            .insert(waiter);
        self.assert_consistent();
        true
    }

    /// Returns the next queued activation opportunity.
    fn take_next_turn(&mut self, has_prioritized: bool) -> H2ActivationTurn {
        self.local_activation_gate.take_next_turn(has_prioritized)
    }

    /// Records a prioritized activation and releases its transferred waiter.
    pub(super) fn reserve_activation_turn(&mut self, waiter: WaiterId) -> bool {
        let gated = self.local_activation_gate.reserve_turn(waiter);
        if let Some(generation) = self.accepting_generation {
            self.generations
                .get_mut(&generation)
                .expect("accepting HTTP/2 generation disappeared")
                .pending_waiters
                .remove(&waiter);
        }
        self.assert_consistent();
        gated
    }

    /// Discharges one accepted or cancelled activation opportunity.
    pub(super) fn release_activation_turn(
        &mut self,
        generation: H2GenerationId,
        waiter: WaiterId,
    ) -> bool {
        let finished = self.local_activation_gate.release_turn(generation, waiter);
        self.assert_consistent();
        finished
    }

    /// Returns whether a direct arrival may use the accepting generation.
    pub(super) fn direct_is_allowed(&self, queued: bool) -> bool {
        !queued && matches!(self.local_activation_gate, H2ActivationGate::Open { .. })
    }

    /// Returns the peer-route cutoff while older waiters remain prioritized.
    fn priority_cutoff(&self) -> Option<WaiterId> {
        self.local_activation_gate.priority_cutoff()
    }

    /// Reserves one prospective claim against an accepting generation.
    fn activate(&mut self, generation: H2GenerationId) -> Option<H2ActivationResources> {
        if self.accepting_generation != Some(generation) {
            return None;
        }
        let record = self.generations.get_mut(&generation)?;
        if record.state != H2GenerationState::Accepting {
            return None;
        }
        record.prospective_requests = record
            .prospective_requests
            .checked_add(1)
            .expect("HTTP/2 prospective request count exhausted");
        let resources = H2ActivationResources {
            sender: record.sender.clone(),
            reused: record.has_accepted_request,
            connection: record.connection.clone(),
        };
        self.assert_consistent();
        Some(resources)
    }

    /// Converts one prospective reservation to an accepted request claim.
    fn accept(&mut self, generation: H2GenerationId) -> bool {
        let Some(record) = self.generations.get_mut(&generation) else {
            return false;
        };
        if record.prospective_requests == 0 {
            return false;
        }
        record.prospective_requests -= 1;
        record.active_requests = record
            .active_requests
            .checked_add(1)
            .expect("HTTP/2 active request count exhausted");
        record.has_accepted_request = true;
        self.assert_consistent();
        true
    }

    /// Cancels one prospective reservation and detaches an empty drain record.
    ///
    /// # Panics
    ///
    /// Panics if the activation's exact generation or prospective count is
    /// missing.
    #[must_use]
    fn cancel(&mut self, generation: H2GenerationId) -> Option<H2Generation> {
        let record = self
            .generations
            .get_mut(&generation)
            .expect("HTTP/2 activation generation disappeared before cancellation");
        assert!(
            record.prospective_requests > 0,
            "HTTP/2 prospective request count underflowed"
        );
        record.prospective_requests -= 1;
        self.remove_finished_drain(generation)
    }

    /// Releases one accepted request and detaches an empty drain record.
    ///
    /// # Panics
    ///
    /// Panics if the claim's exact generation or active request count is missing.
    #[must_use]
    fn complete_request(&mut self, generation: H2GenerationId) -> Option<H2Generation> {
        let record = self
            .generations
            .get_mut(&generation)
            .expect("HTTP/2 request generation disappeared before claim release");
        assert!(
            record.active_requests > 0,
            "HTTP/2 active request count underflowed"
        );
        record.active_requests -= 1;
        self.remove_finished_drain(generation)
    }

    /// Stops one exact generation from accepting new request streams.
    ///
    /// Selected and accepted streams retain the generation in `Draining`
    /// until their claims finish. A generation with no request work is removed
    /// immediately. Pending establishment participants are returned so they
    /// can acquire another connection.
    #[must_use]
    fn begin_close(&mut self, generation: H2GenerationId) -> Option<H2CloseTransition> {
        if self.accepting_generation != Some(generation) {
            return None;
        }
        let record = self.generations.get_mut(&generation)?;
        if record.state != H2GenerationState::Accepting {
            return None;
        }
        record.state = H2GenerationState::Draining;
        self.accepting_generation = None;
        self.local_activation_gate = H2ActivationGate::Closed;
        let pending_waiters = std::mem::take(&mut record.pending_waiters);
        let remove_record = record.prospective_requests == 0 && record.active_requests == 0;
        let connection = record.connection.clone();
        let removed_generation = remove_record
            .then(|| self.generations.remove(&generation))
            .flatten();
        Some(H2CloseTransition {
            connection,
            pending_waiters,
            removed_generation,
        })
    }

    /// Detaches an exact accepting generation only while it has no request work.
    #[must_use]
    fn begin_idle_reclaim(&mut self, generation: H2GenerationId) -> Option<H2CloseTransition> {
        self.is_idle(generation)
            .then(|| self.begin_close(generation))
            .flatten()
    }

    /// Returns the accepting generation when one is locally reusable.
    pub(in crate::client::pool) fn accepting(&self) -> Option<H2GenerationId> {
        self.accepting_generation
    }

    /// Returns whether an exact generation remains accepting.
    pub(super) fn is_accepting(&self, generation: H2GenerationId) -> bool {
        self.accepting_generation == Some(generation)
    }

    /// Returns the generation peers may discover after the local cutoff drains.
    pub(super) fn peer_routable_generation(&self) -> Option<H2GenerationId> {
        let generation = self.accepting_generation?;
        matches!(
            self.local_activation_gate,
            H2ActivationGate::Open { generation: gate_generation }
                if gate_generation == generation
        )
        .then_some(generation)
    }

    /// Returns whether an exact publishable generation has no request work.
    pub(super) fn is_idle(&self, generation: H2GenerationId) -> bool {
        if self.peer_routable_generation() != Some(generation) {
            return false;
        }
        self.generations.get(&generation).is_some_and(|record| {
            record.pending_waiters.is_empty()
                && record.prospective_requests == 0
                && record.active_requests == 0
        })
    }

    /// Returns whether a local generation or peer route suppresses admission demand.
    pub(super) fn has_visible_h2(&self) -> bool {
        self.accepting_generation.is_some() || self.peer_route.is_some()
    }

    /// Installs or refreshes one identity-only peer route.
    pub(super) fn attach_peer_route(&mut self, route: H2Route, cutoff: Option<WaiterId>) {
        if self.accepting_generation.is_some() {
            self.peer_route = None;
            self.assert_consistent();
            return;
        }
        let id = route.id();
        match &mut self.peer_route {
            Some(current) if current.route.id() == id => {
                if let Some(cutoff) = cutoff {
                    match &mut current.activation_gate {
                        H2ActivationGate::Closed => {
                            unreachable!("visible peer HTTP/2 route had a closed gate")
                        }
                        H2ActivationGate::Prioritizing {
                            cutoff: current, ..
                        } => *current = (*current).max(cutoff),
                        H2ActivationGate::Open { generation } => {
                            current.activation_gate = H2ActivationGate::Prioritizing {
                                generation: *generation,
                                cutoff,
                                active_turn: None,
                            };
                        }
                    }
                }
            }
            _ => {
                self.peer_route = Some(PeerH2Route {
                    activation_gate: H2ActivationGate::for_generation(id.generation, cutoff),
                    route,
                    crossing_waiter: None,
                });
            }
        }
        self.assert_consistent();
    }

    /// Marks one requesting waiter as the route's current activation opportunity.
    fn prepare_peer_activation(
        &mut self,
        waiters: &AcquisitionQueue,
    ) -> Option<PreparedPeerActivation> {
        let peer = self.peer_route.as_mut()?;
        if peer.crossing_waiter.is_some() {
            return None;
        }
        let has_prioritized = peer
            .activation_gate
            .priority_cutoff()
            .is_some_and(|cutoff| waiters.has_h2_compatible_waiter_through(cutoff));
        let cutoff = match peer.activation_gate.take_next_turn(has_prioritized) {
            H2ActivationTurn::Unavailable => return None,
            H2ActivationTurn::Open => None,
            H2ActivationTurn::Through(cutoff) => Some(cutoff),
        };
        let waiter = waiters.oldest_h2_compatible_waiter()?;
        if cutoff.is_some_and(|cutoff| waiter > cutoff) {
            return None;
        }
        let gated = peer.activation_gate.reserve_turn(waiter);
        peer.crossing_waiter = Some(waiter);
        Some(PreparedPeerActivation {
            route: peer.route.clone(),
            waiter,
            cutoff,
            gated,
        })
    }

    /// Revalidates one peer activation after the connection-cell crossing.
    fn peer_activation_is_current(
        &self,
        prepared: &PreparedPeerActivation,
        waiters: &AcquisitionQueue,
    ) -> bool {
        self.peer_route.as_ref().is_some_and(|peer| {
            peer.route.id() == prepared.route.id()
                && peer.crossing_waiter == Some(prepared.waiter)
                && (!prepared.gated || peer.activation_gate.active_turn() == Some(prepared.waiter))
                && waiters.is_oldest_h2_compatible_waiter(prepared.waiter)
        })
    }

    /// Ends one route crossing after its result is installed or rejected.
    fn release_peer_crossing(&mut self, route: H2RouteId, waiter: WaiterId) -> bool {
        let Some(peer) = &mut self.peer_route else {
            return false;
        };
        if peer.route.id() != route || peer.crossing_waiter != Some(waiter) {
            return false;
        }
        peer.crossing_waiter = None;
        self.assert_consistent();
        true
    }

    /// Discharges one prioritized peer-route activation opportunity.
    fn release_peer_turn(&mut self, route: H2RouteId, waiter: WaiterId) -> bool {
        let Some(peer) = &mut self.peer_route else {
            return false;
        };
        if peer.route.id() != route {
            return false;
        }
        let finished = peer.activation_gate.release_turn(route.generation, waiter);
        self.assert_consistent();
        finished
    }

    /// Clears a peer activation marker when its requesting waiter is cancelled.
    pub(super) fn cancel_peer_activation(&mut self, waiter: WaiterId) -> bool {
        let Some(route) = self.peer_route.as_ref().map(|peer| peer.route.id()) else {
            return false;
        };
        if self
            .peer_route
            .as_ref()
            .and_then(|peer| peer.activation_gate.active_turn())
            != Some(waiter)
        {
            return false;
        }
        self.release_peer_turn(route, waiter)
    }

    /// Returns an open peer route when no queued acquisition precedes it.
    fn open_peer_route(&self, queued: bool) -> Option<H2Route> {
        let peer = self.peer_route.as_ref()?;
        (!queued && peer.activation_gate.is_open_for(peer.route.id().generation))
            .then(|| peer.route.clone())
    }

    /// Revalidates a direct peer route after its connection-cell crossing.
    fn direct_peer_route_is_current(&self, route: H2RouteId, queued: bool) -> bool {
        !queued
            && self.peer_route.as_ref().is_some_and(|peer| {
                peer.route.id() == route && peer.activation_gate.is_open_for(route.generation)
            })
    }

    /// Removes one stale exact peer route.
    fn detach_peer_route(&mut self, route: H2RouteId) -> bool {
        if self.peer_route.as_ref().map(|peer| peer.route.id()) != Some(route) {
            return false;
        }
        self.peer_route = None;
        self.assert_consistent();
        true
    }

    /// Returns all installed generation identities for shutdown.
    pub(super) fn generation_ids(&self) -> Vec<H2GenerationId> {
        self.generations.keys().copied().collect()
    }

    /// Returns the accepting generation's idle deadline.
    pub(super) fn nearest_idle_deadline(&self) -> Option<SystemTime> {
        self.accepting_generation
            .and_then(|generation| self.generations.get(&generation))
            .and_then(|record| record.idle_deadline)
    }

    /// Returns an accepting generation whose idle deadline elapsed.
    pub(super) fn expired(&self, now: SystemTime) -> Option<H2GenerationId> {
        let generation = self.accepting_generation?;
        let deadline = self.generations.get(&generation)?.idle_deadline?;
        (deadline <= now).then_some(generation)
    }

    /// Replaces the accepting generation's idle deadline after dispatch.
    pub(super) fn reset_idle_deadline(
        &mut self,
        generation: H2GenerationId,
        deadline: Option<SystemTime>,
    ) -> bool {
        if self.accepting_generation != Some(generation) {
            return false;
        }
        let record = self
            .generations
            .get_mut(&generation)
            .expect("accepting HTTP/2 generation disappeared");
        record.idle_deadline = deadline;
        self.assert_consistent();
        true
    }

    /// Detaches a draining generation after its last retained request ends.
    #[must_use]
    fn remove_finished_drain(&mut self, generation: H2GenerationId) -> Option<H2Generation> {
        let remove = self.generations.get(&generation).is_some_and(|record| {
            record.state == H2GenerationState::Draining
                && record.prospective_requests == 0
                && record.active_requests == 0
        });
        remove
            .then(|| self.generations.remove(&generation))
            .flatten()
    }

    fn take_flight_id(&mut self) -> H2FlightId {
        let value = self.next_flight_id;
        self.next_flight_id = value
            .checked_add(1)
            .expect("HTTP/2 flight identity exhausted");
        H2FlightId(value)
    }

    fn take_generation_id(&mut self) -> H2GenerationId {
        let value = self.next_generation_id;
        self.next_generation_id = value
            .checked_add(1)
            .expect("HTTP/2 generation identity exhausted");
        H2GenerationId(value)
    }

    /// Checks flight, generation, residence, and request-count relationships.
    pub(super) fn assert_consistent(&self) {
        #[cfg(any(debug_assertions, test))]
        {
            if std::thread::panicking() {
                return;
            }
            let accepting_records = self
                .generations
                .iter()
                .filter(|(_, record)| record.state == H2GenerationState::Accepting)
                .map(|(generation, _)| *generation)
                .collect::<Vec<_>>();
            match self.accepting_generation {
                Some(generation) => assert_eq!(
                    vec![generation],
                    accepting_records,
                    "HTTP/2 accepting identity did not match generation state"
                ),
                None => assert!(
                    accepting_records.is_empty(),
                    "HTTP/2 accepting generation lacked an accepting identity"
                ),
            }
            assert_eq!(
                self.accepting_generation,
                self.local_activation_gate.generation(),
                "HTTP/2 generation gate did not name the accepting generation"
            );
            assert!(
                self.flight.is_none() || self.accepting_generation.is_none(),
                "HTTP/2 flight coexisted with an accepting generation"
            );
            if let Some(peer) = &self.peer_route {
                assert!(
                    self.accepting_generation.is_none(),
                    "local accepting generation retained a peer route"
                );
                assert_eq!(
                    Some(peer.route.generation()),
                    peer.activation_gate.generation(),
                    "peer route gate did not name the routed generation"
                );
            }

            for record in self.generations.values() {
                if record.state == H2GenerationState::Draining {
                    assert!(
                        record.prospective_requests > 0 || record.active_requests > 0,
                        "empty HTTP/2 draining generation was retained"
                    );
                    assert!(
                        record.pending_waiters.is_empty(),
                        "draining HTTP/2 generation retained unserved waiters"
                    );
                }
            }
        }
    }

    /// Checks flight and generation waiters against the cell's acquisition state.
    #[cfg(any(debug_assertions, test))]
    pub(super) fn assert_pending_waiters(&self, waiters: &AcquisitionQueue) {
        if let Some(flight) = &self.flight {
            for waiter in &flight.participants {
                assert!(
                    waiters.is_launching_h2_waiter(*waiter),
                    "HTTP/2 flight retained a waiter that was no longer launchable"
                );
            }
        }
        for record in self.generations.values() {
            for waiter in &record.pending_waiters {
                assert!(
                    waiters.is_launching_h2_waiter(*waiter),
                    "HTTP/2 generation retained a waiter that was no longer launchable"
                );
            }
        }
    }
}

/// One requesting-cell activation prepared before crossing to its connection cell.
struct PreparedPeerActivation {
    /// Route validated in the requesting cell before the lock crossing.
    route: H2Route,
    /// Oldest waiter selected for this activation.
    waiter: WaiterId,
    /// Peer-route cutoff that this waiter must satisfy.
    cutoff: Option<WaiterId>,
    /// Whether acceptance or cancellation must discharge a priority turn.
    gated: bool,
}

/// State detached when an accepting generation begins draining.
struct H2CloseTransition {
    /// Connection whose logical close follows cell unlock.
    connection: Arc<ConnectionState>,
    /// Unserved transferred waiters that must re-enter acquisition.
    pending_waiters: BTreeSet<WaiterId>,
    /// Removed record retained until the cell lock is released.
    removed_generation: Option<H2Generation>,
}

impl OriginCell {
    /// Reserves a prospective request claim from one exact local generation.
    pub(in crate::client::pool) fn activate_h2(
        cell: &Arc<Self>,
        generation: H2GenerationId,
        request_partition: PartitionId,
    ) -> Option<H2Activation> {
        let (parts, revision) = {
            let mut state = cell.state.lock();
            let parts = state.h2.activate(generation)?;
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (parts, revision)
        };
        Self::submit_h2_supply_update(cell, revision);
        Some(H2Activation::new(
            cell.clone(),
            generation,
            parts,
            request_partition,
            None,
        ))
    }

    /// Converts a prospective activation to an accepted request claim.
    fn accept_h2_activation(cell: &Arc<Self>, generation: H2GenerationId) -> bool {
        let deadline = cell.idle_deadline();
        let (accepted, reset_deadline) = {
            let mut state = cell.state.lock();
            let accepted = state.h2.accept(generation);
            let reset_deadline = accepted && state.h2.reset_idle_deadline(generation, deadline);
            state.assert_consistent();
            (accepted, reset_deadline)
        };
        if reset_deadline {
            cell.notify_maintenance(deadline);
        }
        accepted
    }

    /// Cancels one prospective activation after sender rejection or task drop.
    fn cancel_h2_activation(cell: &Arc<Self>, generation: H2GenerationId) {
        // Keep the detached generation outside the lock's unwind scope. Its
        // sender drop may wake Hyper's driver.
        let (removed, revision) = {
            let mut state = cell.state.lock();
            let removed = state.h2.cancel(generation);
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (removed, revision)
        };
        drop(removed);
        Self::submit_h2_supply_update(cell, revision);
        Self::offer_local_h2(cell);
    }

    /// Releases one accepted request after its upload and response sides finish.
    fn release_h2_request(cell: &Arc<Self>, generation: H2GenerationId) {
        // Keep the detached generation outside the lock's unwind scope. Its
        // sender drop may wake Hyper's driver.
        let (removed, revision) = {
            let mut state = cell.state.lock();
            let removed = state.h2.complete_request(generation);
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (removed, revision)
        };
        drop(removed);
        Self::submit_h2_supply_update(cell, revision);
    }

    /// Offers one local activation while preserving the generation cutoff.
    fn offer_local_h2_locked(
        cell: &Arc<Self>,
        state: &mut CellState,
        returned_step: &mut Option<AcquisitionStep>,
    ) -> Option<WaiterResolution> {
        let generation = state.h2.accepting()?;
        let has_prioritized = state
            .h2
            .priority_cutoff()
            .is_some_and(|cutoff| state.acquisitions.has_h2_compatible_waiter_through(cutoff));
        let cutoff = match state.h2.take_next_turn(has_prioritized) {
            H2ActivationTurn::Unavailable => return None,
            H2ActivationTurn::Open => None,
            H2ActivationTurn::Through(cutoff) => Some(cutoff),
        };
        state.acquisitions.oldest_h2_compatible_waiter()?;

        let h2 = &mut state.h2;
        let acquisitions = &mut state.acquisitions;
        let (waiter, mut install) = acquisitions.offer_h2_activation(
            cutoff,
            |waiter| {
                let parts = h2
                    .activate(generation)
                    .expect("accepting HTTP/2 generation could not create a gated activation");
                let gated = h2.reserve_activation_turn(waiter);
                AcquisitionOutcome::H2(H2Activation::new(
                    cell.clone(),
                    generation,
                    parts,
                    cell.id.partition(),
                    gated.then(|| H2ActivationTurnGuard::local(cell, generation, waiter)),
                ))
            },
            &cell.eligibility_group,
        );
        *returned_step = install.returned_step.take();
        install.demand_updates = state.publishable_demand_updates(install.demand_updates);
        state.assert_consistent();
        waiter.is_some().then_some(install)
    }

    /// Runs publication, fallback, and wake work after the cell lock is released.
    fn run_waiter_resolution(cell: &Arc<Self>, resolution: Option<WaiterResolution>) {
        let Some(resolution) = resolution else {
            return;
        };
        if let Some(admission) = &cell.admission {
            for snapshot in resolution.demand_updates.into_iter().flatten() {
                OriginAdmission::submit_demand_snapshot(admission, cell.id.partition(), snapshot);
            }
        }
        drop(resolution.returned_step);
        if let Some(waker) = resolution.waker {
            waker.wake();
        }
    }

    /// Submits one changed local-generation status after cell unlock.
    fn submit_h2_supply_update(cell: &Arc<Self>, revision: Option<SupplyRevision<H2SupplyStatus>>) {
        if let (Some(admission), Some(revision)) = (&cell.admission, revision) {
            OriginAdmission::apply_h2_supply_revision(
                admission,
                cell.id.partition(),
                cell.eligibility_group.clone(),
                revision,
            );
        }
    }

    /// Publishes the current local demand after its last H2 route disappears.
    fn publish_current_demand(cell: &Arc<Self>, snapshot: Option<DemandSnapshot>) {
        if let (Some(admission), Some(snapshot)) = (&cell.admission, snapshot) {
            OriginAdmission::submit_demand_snapshot(admission, cell.id.partition(), snapshot);
        }
    }

    /// Advances a generation gate after one activation accepts or cancels.
    fn release_local_h2_turn(cell: &Arc<Self>, generation: H2GenerationId, waiter: WaiterId) {
        let mut returned_step = None;
        let (install, revision) = {
            let mut state = cell.state.lock();
            if !state.h2.release_activation_turn(generation, waiter) {
                return;
            }
            let install = Self::offer_local_h2_locked(cell, &mut state, &mut returned_step);
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (install, revision)
        };
        drop(returned_step);
        Self::submit_h2_supply_update(cell, revision);
        Self::run_waiter_resolution(cell, install);
    }

    /// Offers the accepting local generation to one queued acquisition.
    pub(in crate::client::pool) fn offer_local_h2(cell: &Arc<Self>) {
        let mut returned_step = None;
        let (install, revision) = {
            let mut state = cell.state.lock();
            let install = Self::offer_local_h2_locked(cell, &mut state, &mut returned_step);
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (install, revision)
        };
        drop(returned_step);
        Self::submit_h2_supply_update(cell, revision);
        Self::run_waiter_resolution(cell, install);
    }

    /// Returns whether one exact connection generation still accepts activations.
    pub(in crate::client::pool) fn h2_generation_is_accepting(
        cell: &Arc<Self>,
        generation: H2GenerationId,
    ) -> bool {
        cell.state.lock().h2.is_accepting(generation)
    }

    /// Attaches requesting-cell visibility for one peer generation route.
    ///
    /// The named local demand remains queued behind the route gate. Advancing
    /// its snapshot version makes admission acknowledgement authoritative
    /// without losing the demand if this exact route later becomes stale.
    pub(in crate::client::pool) fn attach_h2_route(
        cell: &Arc<Self>,
        route: H2Route,
        group: &EligibilityGroup,
        demand: DemandId,
    ) -> bool {
        if &cell.eligibility_group != group || route.connection_partition() == cell.id.partition() {
            return false;
        }
        let mut state = cell.state.lock();
        if !state.acquisitions.supersede_demand_snapshot(demand) {
            return false;
        }
        let cutoff = state.acquisitions.route_cutoff();
        state.h2.attach_peer_route(route, cutoff);
        state.assert_consistent();
        true
    }

    /// Activates queued requests through the currently visible peer route.
    ///
    /// Preparing the requesting-cell opportunity, activating the connection
    /// generation, and committing the result each use a separate lock scope.
    pub(in crate::client::pool) fn offer_peer_h2(cell: &Arc<Self>) {
        loop {
            let prepared = {
                let mut state = cell.state.lock();
                let CellState {
                    h2, acquisitions, ..
                } = &mut *state;
                let prepared = h2.prepare_peer_activation(acquisitions);
                state.assert_consistent();
                prepared
            };
            let Some(prepared) = prepared else {
                return;
            };
            let route_id = prepared.route.id();
            let Some(activation) = prepared.route.activate(cell.id.partition()) else {
                let snapshot = {
                    let mut state = cell.state.lock();
                    if !state.h2.detach_peer_route(route_id) {
                        return;
                    }
                    let snapshot = state
                        .acquisitions
                        .current_demand_snapshot(&cell.eligibility_group);
                    state.assert_consistent();
                    snapshot
                };
                if let (Some(admission), Some(snapshot)) = (&cell.admission, snapshot) {
                    OriginAdmission::submit_demand_snapshot(
                        admission,
                        cell.id.partition(),
                        snapshot,
                    );
                }
                return;
            };

            let mut activation = Some(activation);
            let mut returned_step = None;
            let install = {
                let mut state = cell.state.lock();
                let current = state
                    .h2
                    .peer_activation_is_current(&prepared, &state.acquisitions);
                if !current {
                    state.h2.release_peer_crossing(route_id, prepared.waiter);
                    if prepared.gated {
                        state.h2.release_peer_turn(route_id, prepared.waiter);
                    }
                    state.assert_consistent();
                    None
                } else {
                    if prepared.gated {
                        activation
                            .as_mut()
                            .expect("peer HTTP/2 activation disappeared")
                            .attach_peer_turn(cell, route_id, prepared.waiter);
                    }
                    let (waiter, mut install) = state.acquisitions.offer_h2_activation(
                        prepared.cutoff,
                        |_| {
                            AcquisitionOutcome::H2(
                                activation
                                    .take()
                                    .expect("peer HTTP/2 activation was installed twice"),
                            )
                        },
                        &cell.eligibility_group,
                    );
                    returned_step = install.returned_step.take();
                    install.demand_updates =
                        state.publishable_demand_updates(install.demand_updates);
                    state.h2.release_peer_crossing(route_id, prepared.waiter);
                    if waiter.is_none() && prepared.gated {
                        state.h2.release_peer_turn(route_id, prepared.waiter);
                    }
                    state.assert_consistent();
                    waiter.is_some().then_some(install)
                }
            };
            drop(activation);
            drop(returned_step);
            if install.is_some() {
                Self::run_waiter_resolution(cell, install);
                return;
            }
        }
    }

    /// Advances an exact peer-route gate after acceptance or cancellation.
    fn release_peer_h2_turn(cell: &Arc<Self>, route: H2RouteId, waiter: WaiterId) {
        let finished = {
            let mut state = cell.state.lock();
            let finished = state.h2.release_peer_turn(route, waiter);
            state.assert_consistent();
            finished
        };
        if finished {
            Self::offer_peer_h2(cell);
        }
    }
}

/// Generation-specific close authority that does not retain the cell.
#[derive(Clone, Debug)]
pub(in crate::client::pool) struct H2CloseHandle {
    /// Cell weak reference so driver lifetime does not retain the pool.
    cell: Weak<OriginCell>,
    /// Exact generation this handle may close.
    generation: H2GenerationId,
}

impl H2CloseHandle {
    /// Creates close authority for one installed generation.
    pub(in crate::client::pool) fn new(cell: &Arc<OriginCell>, generation: H2GenerationId) -> Self {
        Self {
            cell: Weak::from_arc(cell),
            generation,
        }
    }

    /// Begins drain when the cell still contains this generation.
    pub(in crate::client::pool) fn close(&self, reason: CloseReason) -> bool {
        self.cell
            .upgrade()
            .is_some_and(|cell| OriginCell::close_h2(&cell, self.generation, reason))
    }
}

/// Closes an H2 generation if its owner-runtime driver ends or is dropped.
pub(in crate::client::pool) struct H2DriverGuard {
    /// Exact-generation close authority.
    close: H2CloseHandle,
    /// Whether drop must report owner-runtime shutdown.
    active: bool,
}

impl H2DriverGuard {
    /// Arms generation cleanup before driver submission.
    pub(in crate::client::pool) fn new(close: H2CloseHandle) -> Self {
        Self {
            close,
            active: true,
        }
    }

    /// Records ordinary driver completion.
    pub(in crate::client::pool) fn protocol_closed(mut self) {
        self.active = false;
        self.close.close(CloseReason::ProtocolClosed);
    }
}

impl Drop for H2DriverGuard {
    fn drop(&mut self) {
        if self.active {
            self.close.close(CloseReason::OwnerRuntimeShutdown);
        }
    }
}

impl OriginCell {
    /// Selects an accepting local generation before consulting a peer route.
    pub(in crate::client::pool) fn select_h2(cell: &Arc<Self>) -> Option<H2Activation> {
        let local = {
            let mut state = cell.state.lock();
            let queued = state.acquisitions.has_h2_compatible_waiter();
            if state.h2.direct_is_allowed(queued) {
                let generation = state.h2.accepting()?;
                let parts = state.h2.activate(generation)?;
                state.assert_consistent();
                let revision = state.take_h2_supply_update();
                Some((generation, parts, revision))
            } else {
                None
            }
        };
        if let Some((generation, parts, revision)) = local {
            Self::submit_h2_supply_update(cell, revision);
            return Some(H2Activation::new(
                cell.clone(),
                generation,
                parts,
                cell.id.partition(),
                None,
            ));
        }

        let route = {
            let state = cell.state.lock();
            state
                .h2
                .open_peer_route(state.acquisitions.has_h2_compatible_waiter())?
        };
        let route_id = route.id();
        let Some(activation) = route.activate(cell.id.partition()) else {
            Self::detach_stale_peer_route(cell, route_id);
            return None;
        };
        let current = {
            let state = cell.state.lock();
            state.h2.direct_peer_route_is_current(
                route_id,
                state.acquisitions.has_h2_compatible_waiter(),
            )
        };
        current.then_some(activation)
    }

    /// Removes one exact stale route and republishes the requesting demand.
    fn detach_stale_peer_route(cell: &Arc<Self>, route: H2RouteId) {
        let snapshot = {
            let mut state = cell.state.lock();
            if !state.h2.detach_peer_route(route) {
                return;
            }
            let snapshot = state
                .acquisitions
                .current_demand_snapshot(&cell.eligibility_group);
            state.assert_consistent();
            snapshot
        };
        Self::publish_current_demand(cell, snapshot);
    }

    /// Places a post-ALPN waiter behind an already accepting generation.
    pub(in crate::client::pool) fn join_h2_generation(
        cell: &Arc<Self>,
        waiter: WaiterId,
        generation: H2GenerationId,
    ) -> H2GenerationJoinOutcome {
        let mut returned_step = None;
        let (install, revision) = {
            let mut state = cell.state.lock();
            if !state.acquisitions.is_launching_h2_waiter(waiter) {
                return H2GenerationJoinOutcome::WaiterResolved;
            }
            if !state.h2.prioritize_waiter(generation, waiter) {
                return H2GenerationJoinOutcome::GenerationChanged;
            }
            let install = Self::offer_local_h2_locked(cell, &mut state, &mut returned_step);
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (install, revision)
        };
        drop(returned_step);
        Self::submit_h2_supply_update(cell, revision);
        Self::run_waiter_resolution(cell, install);
        H2GenerationJoinOutcome::Joined
    }

    /// Atomically selects, joins, or installs the cell's post-ALPN flight.
    pub(in crate::client::pool) fn converge_h2_flight(&self, waiter: WaiterId) -> H2FlightDecision {
        let mut state = self.state.lock();
        if !state.acquisitions.is_launching_h2_waiter(waiter) {
            return H2FlightDecision::WaiterResolved;
        }
        let result = state.h2.converge_flight(waiter);
        state.assert_consistent();
        result
    }

    /// Installs one successful flight as the accepting generation.
    pub(in crate::client::pool) fn complete_h2_flight(
        cell: &Arc<Self>,
        flight: H2FlightId,
        connection: Arc<ConnectionState>,
        sender: H2Sender,
        idle_deadline: Option<SystemTime>,
    ) -> Result<H2GenerationId, (Arc<ConnectionState>, H2Sender)> {
        let mut returned_step = None;
        let (completion, install, revision) = {
            let mut state = cell.state.lock();
            let completion = state
                .h2
                .complete_flight(flight, connection, sender, idle_deadline);
            let install = if completion.is_ok() {
                let cutoff = state.acquisitions.route_cutoff();
                state.h2.prioritize_through(cutoff);
                Self::offer_local_h2_locked(cell, &mut state, &mut returned_step)
            } else {
                None
            };
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (completion, install, revision)
        };
        drop(returned_step);
        Self::submit_h2_supply_update(cell, revision);
        Self::run_waiter_resolution(cell, install);
        if completion.is_ok() {
            cell.notify_maintenance(idle_deadline);
        }
        completion
    }

    /// Retires one failed flight and returns its retained participants.
    pub(in crate::client::pool) fn fail_h2_flight(
        &self,
        flight: H2FlightId,
    ) -> Option<Vec<WaiterId>> {
        let mut state = self.state.lock();
        let participants = state.h2.fail_flight(flight);
        state.assert_consistent();
        participants
    }

    /// Returns the complete current H2 supply revision for an unlocked crossing.
    pub(in crate::client::pool) fn current_h2_supply_revision(
        &self,
    ) -> SupplyRevision<H2SupplyStatus> {
        let mut state = self.state.lock();
        let revision = state.current_h2_supply_revision();
        state.assert_consistent();
        revision
    }

    /// Reclaims one exact idle generation and reports its resulting availability.
    pub(in crate::client::pool) fn reclaim_idle_h2(
        cell: &Arc<Self>,
        generation: H2GenerationId,
    ) -> (SupplyRevision<H2SupplyStatus>, Option<ConnectionId>) {
        // A zero-request generation may hold the last capacity-owning
        // connection reference. Keep it outside the cell-lock unwind scope.
        let detached;
        let (revision, demand) = {
            let mut state = cell.state.lock();
            detached = state.h2.begin_idle_reclaim(generation);
            let revision = state.current_h2_supply_revision();
            let demand = detached.as_ref().and_then(|_| {
                state
                    .acquisitions
                    .current_demand_snapshot(&cell.eligibility_group)
            });
            state.assert_consistent();
            (revision, demand)
        };
        let Some(detached) = detached else {
            return (revision, None);
        };
        let H2CloseTransition {
            connection,
            pending_waiters,
            removed_generation,
        } = detached;
        assert!(
            pending_waiters.is_empty(),
            "idle HTTP/2 reclaim detached pending generation waiters"
        );
        let connection_id = connection.id();
        drop(removed_generation);
        Self::publish_current_demand(cell, demand);
        let reclaimed = connection.logical_close(CloseReason::Reclaimed);
        (revision, reclaimed.then_some(connection_id))
    }

    /// Moves one exact generation to draining and closes its connection.
    pub(super) fn close_h2(
        cell: &Arc<Self>,
        generation: H2GenerationId,
        reason: CloseReason,
    ) -> bool {
        // A zero-request generation may hold the last capacity-owning
        // connection reference. Keep it outside the cell-lock unwind scope.
        let detached;
        let (revision, demand) = {
            let mut state = cell.state.lock();
            detached = state.h2.begin_close(generation);
            let revision = detached
                .as_ref()
                .and_then(|_| state.take_h2_supply_update());
            let demand = detached.as_ref().and_then(|_| {
                state
                    .acquisitions
                    .current_demand_snapshot(&cell.eligibility_group)
            });
            state.assert_consistent();
            (revision, demand)
        };
        let Some(detached) = detached else {
            return false;
        };
        let H2CloseTransition {
            connection,
            pending_waiters,
            removed_generation,
        } = detached;
        drop(removed_generation);
        Self::submit_h2_supply_update(cell, revision);
        Self::publish_current_demand(cell, demand);
        for waiter in pending_waiters {
            cell.complete_establishment(waiter, AcquisitionOutcome::RetryAcquisition);
        }
        connection.logical_close(reason)
    }

    /// Returns the exact accepting generation for peer routing.
    #[cfg(test)]
    pub(in crate::client::pool) fn accepting_h2_generation(&self) -> Option<H2GenerationId> {
        self.state.lock().h2.accepting()
    }

    /// Returns the prospective and active request counts for one generation.
    #[cfg(test)]
    pub(in crate::client::pool) fn h2_request_counts(
        &self,
        generation: H2GenerationId,
    ) -> Option<(usize, usize)> {
        self.state
            .lock()
            .h2
            .generations
            .get(&generation)
            .map(|record| (record.prospective_requests, record.active_requests))
    }

    /// Installs an accepting generation without a Hyper handshake.
    #[cfg(test)]
    pub(in crate::client::pool) fn install_h2_for_test(
        cell: &Arc<Self>,
        connection: Arc<ConnectionState>,
        sender_id: u64,
        idle_deadline: Option<SystemTime>,
    ) -> H2GenerationId {
        let (generation, revision) = {
            let mut state = cell.state.lock();
            let generation = state.h2.open_generation(
                BTreeSet::new(),
                connection,
                H2Sender::test(sender_id),
                idle_deadline,
            );
            state.assert_consistent();
            let revision = state.take_h2_supply_update();
            (generation, revision)
        };
        Self::submit_h2_supply_update(cell, revision);
        cell.notify_maintenance(idle_deadline);
        generation
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use crate::client::pool::admission::ProtocolRequirement;
    use crate::client::pool::connection::{CloseReason, ConnectionInfo};
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::EligibilityGroup;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use http_1x::uri::Scheme;

    fn cell() -> Arc<OriginCell> {
        Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ))
    }

    fn bounded_cell(
        admission: &Arc<crate::client::pool::admission::OriginAdmission>,
        partition: usize,
    ) -> Arc<OriginCell> {
        crate::client::pool::admission::OriginAdmission::register_cell(
            admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(partition),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        )
    }

    fn connection(
        id: u64,
    ) -> (
        Arc<ConnectionState>,
        super::super::super::connection::PhysicalConnectionGuard,
    ) {
        ConnectionState::unbounded(ConnectionInfo::for_test(
            ConnectionId::new(id),
            PartitionId::from_index(1),
        ))
    }

    fn begin_waiter(cell: &Arc<OriginCell>) -> WaiterId {
        let waiter = OriginCell::register_waiter(cell, ProtocolRequirement::H2Required);
        let event = cell
            .take_ready_event(waiter)
            .expect("unbounded H2 waiter did not receive establishment authority");
        let super::super::AcquisitionStep::StartEstablishment(permit) = event else {
            panic!("new H2 waiter completed before establishment");
        };
        assert!(cell.start_establishment(waiter));
        drop(permit);
        waiter
    }

    fn open_test_generation(
        cell: &Arc<OriginCell>,
        connection_id: u64,
    ) -> (
        H2GenerationId,
        Arc<ConnectionState>,
        super::super::super::connection::PhysicalConnectionGuard,
    ) {
        let waiter = begin_waiter(cell);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(waiter) else {
            panic!("fresh cell did not create an HTTP/2 flight");
        };
        let (connection, physical) = connection(connection_id);
        let generation = OriginCell::complete_h2_flight(
            cell,
            flight,
            connection.clone(),
            H2Sender::test(connection_id),
            None,
        )
        .expect("fresh flight did not install");
        let event = cell
            .take_ready_event(waiter)
            .expect("flight completion did not satisfy its waiter");
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            activation,
        )) = event
        else {
            panic!("flight completion produced a non-H2 result");
        };
        drop(activation);
        (generation, connection, physical)
    }

    fn generation_counts(cell: &OriginCell, generation: H2GenerationId) -> (usize, usize) {
        cell.h2_request_counts(generation)
            .expect("generation was not installed")
    }

    #[test]
    fn flight_converges_participants_and_cancellation() {
        let mut records = H2CellState::default();
        let first = WaiterId(1);
        let second = WaiterId(2);

        let H2FlightDecision::RunFlight(flight) = records.converge_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            records.converge_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        records.cancel_flight_participant(second);
        assert_eq!(Some(vec![first]), records.fail_flight(flight));
        assert!(records.flight.is_none());
    }

    #[test]
    #[should_panic(expected = "HTTP/2 flight retained a waiter that was no longer launchable")]
    fn flight_participant_must_remain_launchable() {
        let cell = cell();
        let waiter = begin_waiter(&cell);
        assert!(matches!(
            cell.converge_h2_flight(waiter),
            H2FlightDecision::RunFlight(_)
        ));

        let mut state = cell.state.lock();
        let cancellation = state
            .acquisitions
            .cancel_waiter(waiter, &cell.eligibility_group)
            .expect("launching waiter did not cancel");
        drop(cancellation);
        state.h2.assert_pending_waiters(&state.acquisitions);
    }

    #[test]
    fn accepting_generation_prevents_a_second_flight() {
        let mut records = H2CellState::default();
        let (connection, _physical) = connection(1);
        let generation =
            records.open_generation(BTreeSet::new(), connection, H2Sender::test(1), None);

        assert!(matches!(
            records.converge_flight(WaiterId(1)),
            H2FlightDecision::UseGeneration(current) if current == generation
        ));
        assert!(records.flight.is_none());
    }

    #[test]
    fn flight_convergence_does_not_retain_a_cancelled_waiter() {
        let cell = cell();
        let waiter = begin_waiter(&cell);

        assert!(OriginCell::cancel_waiter(&cell, waiter));
        assert!(matches!(
            cell.converge_h2_flight(waiter),
            H2FlightDecision::WaiterResolved
        ));
        assert!(cell.state.lock().h2.flight.is_none());
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn generation_join_does_not_retain_a_waiter_already_served_by_the_gate() {
        let cell = cell();
        let first = begin_waiter(&cell);
        let second = begin_waiter(&cell);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first waiter did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));

        let (connection, _physical) = connection(1);
        let generation =
            OriginCell::complete_h2_flight(&cell, flight, connection, H2Sender::test(1), None)
                .expect("flight did not install");

        assert_eq!(
            H2GenerationJoinOutcome::WaiterResolved,
            OriginCell::join_h2_generation(&cell, first, generation)
        );
        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::PoolDropped,
        ));
        assert!(matches!(
            cell.converge_h2_flight(first),
            H2FlightDecision::WaiterResolved
        ));
        assert!(cell.state.lock().h2.flight.is_none());
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            activation,
        )) = cell
            .take_ready_event(first)
            .expect("generation gate did not serve the first waiter")
        else {
            panic!("first waiter received a non-H2 result");
        };

        drop(activation);
        assert!(matches!(
            cell.take_ready_event(second),
            Some(super::super::AcquisitionStep::Resolved(
                super::super::AcquisitionOutcome::RetryAcquisition
            ))
        ));
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn generation_gate_services_committed_waiters_before_direct_arrivals() {
        let cell = cell();
        let first = begin_waiter(&cell);
        let second = begin_waiter(&cell);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        let (connection, _physical) = connection(1);
        OriginCell::complete_h2_flight(&cell, flight, connection, H2Sender::test(1), None)
            .expect("flight did not install");

        assert!(
            OriginCell::select_h2(&cell).is_none(),
            "direct arrival bypassed committed waiters"
        );
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first waiter was not served")
        else {
            panic!("first waiter received a non-H2 result");
        };
        drop(first_activation);

        assert!(
            OriginCell::select_h2(&cell).is_none(),
            "direct arrival bypassed the second committed waiter"
        );
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            second_activation,
        )) = cell
            .take_ready_event(second)
            .expect("second waiter was not served")
        else {
            panic!("second waiter received a non-H2 result");
        };
        drop(second_activation);

        assert!(
            OriginCell::select_h2(&cell).is_some(),
            "gate did not open after committed waiters drained"
        );
    }

    #[test]
    fn cancelling_an_open_gate_activation_services_the_next_waiter() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test(NonZeroUsize::new(1).unwrap());
        let cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        );
        let (connection, _physical) = ConnectionState::bounded(
            ConnectionInfo::for_test(ConnectionId::new(1), cell.id().partition()),
            OriginAdmission::lease_for_test(&admission),
        );
        let generation = OriginCell::install_h2_for_test(&cell, connection, 1, None);
        cell.state.lock().h2.prioritize_through(Some(WaiterId(0)));

        let first = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first waiter was not served")
        else {
            panic!("first waiter received a non-H2 result");
        };
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
        let third = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
        assert_eq!((1, 0), generation_counts(&cell, generation));

        drop(first_activation);
        assert_eq!((1, 0), generation_counts(&cell, generation));
        assert!(OriginCell::cancel_waiter(&cell, second));
        assert_eq!(
            (1, 0),
            generation_counts(&cell, generation),
            "cancelling an open-gate activation stranded its successor"
        );

        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            third_activation,
        )) = cell
            .take_ready_event(third)
            .expect("successor was not served after cancellation")
        else {
            panic!("successor received a non-H2 result");
        };
        drop(third_activation);
        assert_eq!((0, 0), generation_counts(&cell, generation));
        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }

    #[test]
    fn cancelling_an_unserved_flight_participant_prunes_the_generation() {
        let cell = cell();
        let first = begin_waiter(&cell);
        let second = begin_waiter(&cell);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        let (connection, _physical) = connection(1);
        let generation =
            OriginCell::complete_h2_flight(&cell, flight, connection, H2Sender::test(1), None)
                .expect("flight did not install");
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first participant was not served")
        else {
            panic!("first participant received a non-H2 result");
        };

        assert!(OriginCell::cancel_waiter(&cell, second));
        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::PoolDropped,
        ));
        drop(first_activation);
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn returned_h1_prunes_a_transferred_h2_waiter() {
        let cell = cell();
        let first = begin_waiter(&cell);
        let second = OriginCell::register_waiter(&cell, ProtocolRequirement::H1Compatible);
        let super::super::AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(second)
            .expect("second participant did not receive establishment authority")
        else {
            panic!("second participant completed before establishment");
        };
        assert!(cell.start_establishment(second));
        drop(permit);

        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        let (h2_connection, _h2_physical) = connection(1);
        let generation =
            OriginCell::complete_h2_flight(&cell, flight, h2_connection, H2Sender::test(1), None)
                .expect("flight did not install");
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first participant was not served")
        else {
            panic!("first participant received a non-H2 result");
        };

        let (h1_connection, _h1_physical) = connection(2);
        let returning = OriginCell::insert_selected_h1(
            &cell,
            h1_connection,
            super::super::h1::H1Sender::test(2),
        );
        drop(returning);
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H1(
            selection,
        )) = cell
            .take_ready_event(second)
            .expect("returned HTTP/1 sender did not serve the compatible waiter")
        else {
            panic!("compatible waiter received a non-H1 result");
        };
        drop(selection);

        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::PoolDropped,
        ));
        drop(first_activation);
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn later_local_activation_does_not_hide_an_open_generation_from_peers() {
        let mut records = H2CellState::default();
        let committed = WaiterId(1);
        let later = WaiterId(2);
        let H2FlightDecision::RunFlight(flight) = records.converge_flight(committed) else {
            panic!("first participant did not become the flight driver");
        };
        let (connection, _physical) = connection(1);
        let generation = records
            .complete_flight(flight, connection, H2Sender::test(1), None)
            .expect("flight did not install");
        records.prioritize_through(Some(committed));

        assert_eq!(
            H2ActivationTurn::Through(committed),
            records.take_next_turn(true),
            "committed waiter did not retain initial priority"
        );
        records.reserve_activation_turn(committed);
        assert_eq!(None, records.peer_routable_generation());
        assert!(records.release_activation_turn(generation, committed));

        assert_eq!(H2ActivationTurn::Open, records.take_next_turn(false));
        records.reserve_activation_turn(later);
        assert_eq!(
            Some(generation),
            records.peer_routable_generation(),
            "a post-cutoff local activation hid the generation from peers"
        );
    }

    #[test]
    fn generation_close_reacquires_unserved_flight_participant() {
        let cell = cell();
        let first = begin_waiter(&cell);
        let second = begin_waiter(&cell);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        let (connection, _physical) = connection(1);
        let generation =
            OriginCell::complete_h2_flight(&cell, flight, connection, H2Sender::test(1), None)
                .expect("flight did not install");

        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first participant was not served")
        else {
            panic!("first participant received a non-H2 result");
        };
        assert!(cell.take_ready_event(second).is_none());

        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::Poisoned
        ));
        assert!(matches!(
            cell.take_ready_event(second),
            Some(super::super::AcquisitionStep::Resolved(
                super::super::AcquisitionOutcome::RetryAcquisition
            ))
        ));
        drop(first_activation);
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn bounded_generation_close_returns_capacity_and_reacquires_participant() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test(NonZeroUsize::new(2).unwrap());
        let cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        );
        let mut participants = Vec::new();
        for _ in 0..2 {
            let waiter = OriginCell::register_waiter(&cell, ProtocolRequirement::H2Required);
            let super::super::AcquisitionStep::StartEstablishment(permit) = cell
                .take_ready_event(waiter)
                .expect("bounded participant did not receive capacity")
            else {
                panic!("bounded participant completed before establishment");
            };
            assert!(cell.start_establishment(waiter));
            participants.push((waiter, permit));
        }
        let (first, first_permit) = participants.remove(0);
        let (second, second_permit) = participants.remove(0);
        let H2FlightDecision::RunFlight(flight) = cell.converge_h2_flight(first) else {
            panic!("first participant did not become the flight driver");
        };
        assert!(matches!(
            cell.converge_h2_flight(second),
            H2FlightDecision::JoinedFlight
        ));
        drop(second_permit);
        let (connection, _physical) = ConnectionState::bounded(
            ConnectionInfo::for_test(ConnectionId::new(1), cell.id().partition()),
            first_permit
                .into_lease()
                .expect("bounded permit had no lease"),
        );
        let generation =
            OriginCell::complete_h2_flight(&cell, flight, connection, H2Sender::test(1), None)
                .expect("bounded flight did not install");
        let super::super::AcquisitionStep::Resolved(super::super::AcquisitionOutcome::H2(
            first_activation,
        )) = cell
            .take_ready_event(first)
            .expect("first participant was not served")
        else {
            panic!("first participant received a non-H2 result");
        };

        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::Poisoned
        ));
        assert!(matches!(
            cell.take_ready_event(second),
            Some(super::super::AcquisitionStep::Resolved(
                super::super::AcquisitionOutcome::RetryAcquisition
            ))
        ));
        drop(first_activation);
        assert_eq!(2, admission.available_capacity_for_test());
        assert_eq!(0, cell.retained_waiters_for_test());
    }

    #[test]
    fn generation_is_reused_only_after_hyper_acceptance() {
        let cell = cell();
        let (_generation, connection, _physical) = open_test_generation(&cell, 1);

        let mut activation =
            OriginCell::select_h2(&cell).expect("accepting generation was not selected");
        assert!(!activation.is_reused());
        let H2DispatchParts {
            sender: _sender,
            upload,
            response,
        } = activation.take_dispatch_parts();
        let dispatch = ConnectionState::try_commit_dispatch(&connection)
            .expect("open connection rejected dispatch");
        activation.accept(dispatch);
        drop(upload);
        drop(response);

        let reused = OriginCell::select_h2(&cell).expect("accepted generation was not reusable");
        assert!(reused.is_reused());
        drop(reused);
    }

    #[test]
    #[should_panic(expected = "HTTP/2 activation dispatch parts already taken")]
    fn activation_transfers_dispatch_parts_once() {
        let cell = cell();
        let (_generation, _connection, _physical) = open_test_generation(&cell, 1);
        let mut activation =
            OriginCell::select_h2(&cell).expect("accepting generation was not selected");

        let _first = activation.take_dispatch_parts();
        let _second = activation.take_dispatch_parts();
    }

    #[test]
    fn prospective_activation_cancellation_returns_its_generation_count() {
        let cell = cell();
        let (generation, _connection, _physical) = open_test_generation(&cell, 1);

        let activation =
            OriginCell::select_h2(&cell).expect("accepting generation was not selected");
        assert_eq!((1, 0), generation_counts(&cell, generation));
        drop(activation);
        assert_eq!((0, 0), generation_counts(&cell, generation));
    }

    #[test]
    fn accepted_claim_waits_for_both_sides_in_either_order() {
        for upload_first in [true, false] {
            let cell = cell();
            let (generation, connection, _physical) = open_test_generation(&cell, 1);
            let mut activation =
                OriginCell::select_h2(&cell).expect("accepting generation was not selected");
            let H2DispatchParts {
                sender: _sender,
                upload,
                response,
            } = activation.take_dispatch_parts();
            let dispatch = ConnectionState::try_commit_dispatch(&connection)
                .expect("open connection rejected dispatch");
            activation.accept(dispatch);
            assert_eq!((0, 1), generation_counts(&cell, generation));
            assert_eq!(1, connection.probe().in_flight);

            if upload_first {
                drop(upload);
                assert_eq!((0, 1), generation_counts(&cell, generation));
                response.finish();
            } else {
                response.finish();
                assert_eq!((0, 1), generation_counts(&cell, generation));
                drop(upload);
            }

            assert_eq!((0, 0), generation_counts(&cell, generation));
            assert_eq!(0, connection.probe().in_flight);
        }
    }

    #[test]
    fn stale_route_cannot_activate_a_replacement_or_retained_drain() {
        let cell = cell();
        let (first, _first_connection, _first_physical) = open_test_generation(&cell, 1);
        let stale_route = H2Route::new(&cell, first);
        let retained = OriginCell::select_h2(&cell).expect("first generation was not selectable");
        assert!(OriginCell::close_h2(
            &cell,
            first,
            CloseReason::ProtocolClosed
        ));

        let (second, _second_connection, _second_physical) = open_test_generation(&cell, 2);
        assert_ne!(first, second);
        assert!(
            stale_route.activate(PartitionId::from_index(2)).is_none(),
            "stale route activated a draining or replacement generation"
        );
        drop(retained);
    }

    #[test]
    fn accepting_identity_rejects_a_retained_draining_generation() {
        let cell = cell();
        let (first, _first_connection, _first_physical) = open_test_generation(&cell, 1);
        let retained = OriginCell::select_h2(&cell).expect("first generation was not selectable");
        assert!(OriginCell::close_h2(
            &cell,
            first,
            CloseReason::ProtocolClosed
        ));
        let (second, _second_connection, _second_physical) = open_test_generation(&cell, 2);

        assert!(cell.state.lock().h2.is_accepting(second));
        assert!(!cell.state.lock().h2.is_accepting(first));
        drop(retained);
    }

    #[test]
    fn activation_requires_the_exact_accepting_generation() {
        let cell = cell();
        let (first, _first_connection, _first_physical) = open_test_generation(&cell, 1);
        let retained = OriginCell::select_h2(&cell).expect("first generation was not selectable");
        assert!(OriginCell::close_h2(
            &cell,
            first,
            CloseReason::ProtocolClosed
        ));
        let (second, _second_connection, _second_physical) = open_test_generation(&cell, 2);

        {
            let mut state = cell.state.lock();
            let first_record = state
                .h2
                .generations
                .get_mut(&first)
                .expect("retained generation disappeared");
            assert_eq!(H2GenerationState::Draining, first_record.state);
            first_record.state = H2GenerationState::Accepting;
            assert!(
                state.h2.activate(first).is_none(),
                "non-current generation accepted an activation"
            );
            state
                .h2
                .generations
                .get_mut(&first)
                .expect("retained generation disappeared")
                .state = H2GenerationState::Draining;
            state.assert_consistent();
        }

        assert!(cell.state.lock().h2.is_accepting(second));
        drop(retained);
    }

    #[test]
    fn activation_requires_accepting_generation_state() {
        let cell = cell();
        let (first, _first_connection, _first_physical) = open_test_generation(&cell, 1);
        let retained = OriginCell::select_h2(&cell).expect("first generation was not selectable");
        assert!(OriginCell::close_h2(
            &cell,
            first,
            CloseReason::ProtocolClosed
        ));
        let (second, _second_connection, _second_physical) = open_test_generation(&cell, 2);

        {
            let mut state = cell.state.lock();
            assert_eq!(
                H2GenerationState::Draining,
                state.h2.generations[&first].state
            );
            state.h2.accepting_generation = Some(first);
            assert!(
                state.h2.activate(first).is_none(),
                "draining generation accepted an activation"
            );
            state.h2.accepting_generation = Some(second);
            state.assert_consistent();
        }

        drop(retained);
    }

    #[test]
    fn local_generation_excludes_a_peer_route() {
        let local_cell = cell();
        let (generation, _connection, _physical) = open_test_generation(&local_cell, 1);
        let peer_cell = Arc::new(OriginCell::new(
            PartitionId::from_index(2),
            local_cell.id().origin().clone(),
            EligibilityGroup::Pool,
            None,
            None,
        ));
        let route = H2Route::new(&peer_cell, H2GenerationId::for_test(99));

        let mut state = local_cell.state.lock();
        state.h2.attach_peer_route(route, None);
        assert!(state.h2.peer_route.is_none());
        assert_eq!(Some(generation), state.h2.accepting());
    }

    #[test]
    fn open_generation_allows_concurrent_prospective_activations() {
        let cell = cell();
        let (generation, _connection, _physical) = open_test_generation(&cell, 1);

        let first = OriginCell::select_h2(&cell).expect("first activation was not selected");
        let second =
            OriginCell::select_h2(&cell).expect("open generation serialized the second activation");
        assert_eq!((2, 0), generation_counts(&cell, generation));
        drop(first);
        drop(second);
        assert_eq!((0, 0), generation_counts(&cell, generation));
    }

    #[test]
    fn consistency_checks_do_not_repanic_during_unwind() {
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let cell = cell();
        let (generation, _connection, _physical) = open_test_generation(&cell, 1);
        let activation =
            OriginCell::select_h2(&cell).expect("accepting generation did not activate");
        let (corrupt_connection, corrupt_physical) = connection(2);
        {
            let mut state = cell.state.lock();
            state.h2.generations.insert(
                H2GenerationId(99),
                H2Generation {
                    pending_waiters: BTreeSet::new(),
                    connection: corrupt_connection,
                    sender: H2Sender::test(2),
                    state: H2GenerationState::Draining,
                    prospective_requests: 0,
                    has_accepted_request: false,
                    active_requests: 0,
                    idle_deadline: None,
                },
            );
        }

        let result = catch_unwind(AssertUnwindSafe(move || {
            let _activation = activation;
            panic!("primary test panic");
        }));
        assert!(result.is_err(), "primary panic was not observed");

        let removed = {
            let mut state = cell.state.lock();
            state.h2.generations.remove(&H2GenerationId(99))
        };
        drop(removed);
        drop(corrupt_physical);
        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }

    #[test]
    fn removed_generation_id_cannot_activate_or_close_the_current_generation() {
        let cell = cell();
        let (first, first_connection, _first_physical) = open_test_generation(&cell, 1);
        let stale = H2CloseHandle::new(&cell, first);
        assert!(stale.close(CloseReason::ProtocolClosed));
        assert_eq!(
            Some(CloseReason::ProtocolClosed),
            first_connection.probe().close_reason
        );

        let (second, second_connection, _second_physical) = open_test_generation(&cell, 2);
        assert_ne!(first, second);
        assert!(OriginCell::activate_h2(&cell, first, cell.id.partition()).is_none());
        assert!(!stale.close(CloseReason::Poisoned));
        assert_eq!(None, second_connection.probe().close_reason);
        assert_eq!(Some(second), cell.accepting_h2_generation());
    }

    #[test]
    fn close_drops_detached_capacity_only_after_cell_unlock() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;
        use std::panic::{catch_unwind, AssertUnwindSafe};

        let admission = OriginAdmission::for_test(NonZeroUsize::MIN);
        let cell = OriginAdmission::register_cell(
            &admission,
            Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                Some(admission.clone()),
                None,
            )),
        );
        let info = ConnectionInfo::for_test(ConnectionId::new(1), PartitionId::from_index(1));
        let (bounded_connection, physical) = ConnectionState::pending_open_for_test(info);
        bounded_connection
            .open(Some(OriginAdmission::lease_for_test(&admission)))
            .expect("bounded test connection did not open");
        let generation =
            OriginCell::install_h2_for_test(&cell, bounded_connection.clone(), 1, None);
        drop(bounded_connection);
        drop(physical);

        // Keep an unrelated empty drain so the close transition's final
        // consistency check panics after detaching the bounded connection.
        let (corrupt_connection, corrupt_physical) = connection(2);
        {
            let mut state = cell.state.lock();
            state.h2.generations.insert(
                H2GenerationId(99),
                H2Generation {
                    pending_waiters: BTreeSet::new(),
                    connection: corrupt_connection,
                    sender: H2Sender::test(2),
                    state: H2GenerationState::Draining,
                    prospective_requests: 0,
                    has_accepted_request: false,
                    active_requests: 0,
                    idle_deadline: None,
                },
            );
        }
        drop(corrupt_physical);

        let result = catch_unwind(AssertUnwindSafe(|| {
            OriginCell::close_h2(&cell, generation, CloseReason::ProtocolClosed);
        }));
        assert!(
            result.is_err(),
            "corrupt H2 state did not reach its consistency assertion"
        );

        // This second lease exists only if unwinding dropped the connection
        // after the cell guard, allowing capacity to return through admission.
        drop(OriginAdmission::lease_for_test(&admission));
    }

    #[test]
    fn close_retains_an_accepted_generation_until_both_request_sides_finish() {
        let cell = cell();
        let (generation, connection, _physical) = open_test_generation(&cell, 1);
        let mut activation =
            OriginCell::select_h2(&cell).expect("accepting generation was not selected");
        let H2DispatchParts {
            sender: _sender,
            upload,
            response,
        } = activation.take_dispatch_parts();
        let dispatch = ConnectionState::try_commit_dispatch(&connection)
            .expect("open connection rejected dispatch");
        activation.accept(dispatch);

        assert!(OriginCell::close_h2(
            &cell,
            generation,
            CloseReason::ProtocolClosed
        ));
        assert!(cell.state.lock().h2.generations.contains_key(&generation));
        drop(upload);
        assert!(cell.state.lock().h2.generations.contains_key(&generation));
        drop(response);
        assert!(!cell.state.lock().h2.generations.contains_key(&generation));
        assert_eq!(0, connection.probe().in_flight);
    }

    #[test]
    fn driver_task_drop_closes_its_exact_generation() {
        let cell = cell();
        let (generation, connection, _physical) = open_test_generation(&cell, 1);
        let guard = H2DriverGuard::new(H2CloseHandle::new(&cell, generation));

        drop(guard);

        assert_eq!(
            Some(CloseReason::OwnerRuntimeShutdown),
            connection.probe().close_reason
        );
        assert_eq!(None, cell.accepting_h2_generation());
    }

    #[test]
    fn local_selection_withdraws_idle_h2_from_reclaim() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test(NonZeroUsize::MIN);
        let connection_cell = bounded_cell(&admission, 1);
        let requesting_cell = bounded_cell(&admission, 2);
        let info = ConnectionInfo::for_test(ConnectionId::new(1), connection_cell.id().partition());
        let (connection, _physical) = ConnectionState::pending_open_for_test(info);
        connection
            .open(Some(OriginAdmission::lease_for_test(&admission)))
            .expect("bounded HTTP/2 connection did not open");
        let generation =
            OriginCell::install_h2_for_test(&connection_cell, connection.clone(), 1, None);
        let activation = OriginCell::select_h2(&connection_cell)
            .expect("local accepting generation did not activate");

        let (waiter, demand) =
            requesting_cell.register_waiter_without_publish(ProtocolRequirement::H1Required);
        let action = OriginAdmission::submit_action_without_running(
            &admission,
            requesting_cell.id().partition(),
            demand,
        );
        assert!(
            action.is_none(),
            "busy local HTTP/2 generation remained eligible for reclaim"
        );

        assert!(OriginCell::cancel_waiter(&requesting_cell, waiter));
        drop(activation);
        assert_eq!(None, connection.probe().close_reason);
        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::PoolDropped,
        ));
    }

    #[test]
    fn h1_required_demand_preserves_h2_without_an_http1_guarantee() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test_with_h2_reclaim(NonZeroUsize::MIN, false);
        let connection_cell = bounded_cell(&admission, 1);
        let requesting_cell = bounded_cell(&admission, 2);
        let info = ConnectionInfo::for_test(ConnectionId::new(1), connection_cell.id().partition());
        let (connection, _physical) = ConnectionState::pending_open_for_test(info);
        connection
            .open(Some(OriginAdmission::lease_for_test(&admission)))
            .expect("bounded HTTP/2 connection did not open");
        let generation =
            OriginCell::install_h2_for_test(&connection_cell, connection.clone(), 1, None);

        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Required);
        assert!(
            requesting_cell.take_ready_event(waiter).is_none(),
            "H1-required demand reclaimed capacity without an HTTP/1 guarantee"
        );
        assert_eq!(None, connection.probe().close_reason);
        assert_eq!(Some(generation), connection_cell.accepting_h2_generation());

        assert!(OriginCell::close_h2(
            &connection_cell,
            generation,
            CloseReason::ProtocolClosed,
        ));
        let super::super::AcquisitionStep::StartEstablishment(permit) = requesting_cell
            .take_ready_event(waiter)
            .expect("ordinary H2 close did not release capacity")
        else {
            panic!("released capacity produced a non-establishment result");
        };
        drop(permit);
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn h1_required_demand_reclaims_idle_h2_capacity() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test(NonZeroUsize::MIN);
        let connection_cell = bounded_cell(&admission, 1);
        let requesting_cell = bounded_cell(&admission, 2);
        let info = ConnectionInfo::for_test(ConnectionId::new(1), connection_cell.id().partition());
        let (connection, _physical) = ConnectionState::pending_open_for_test(info);
        connection
            .open(Some(OriginAdmission::lease_for_test(&admission)))
            .expect("bounded HTTP/2 connection did not open");
        OriginCell::install_h2_for_test(&connection_cell, connection.clone(), 1, None);

        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Required);
        let super::super::AcquisitionStep::StartEstablishment(permit) = requesting_cell
            .take_ready_event(waiter)
            .expect("idle HTTP/2 reclaim did not deliver capacity")
        else {
            panic!("idle HTTP/2 reclaim produced a non-capacity result");
        };

        assert_eq!(
            Some(CloseReason::Reclaimed),
            connection.probe().close_reason
        );
        assert_eq!(None, connection_cell.accepting_h2_generation());
        drop(permit);
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn h1_required_demand_waits_for_active_h2_before_reclaim() {
        use crate::client::pool::admission::OriginAdmission;
        use std::num::NonZeroUsize;

        let admission = OriginAdmission::for_test(NonZeroUsize::MIN);
        let connection_cell = bounded_cell(&admission, 1);
        let requesting_cell = bounded_cell(&admission, 2);
        let info = ConnectionInfo::for_test(ConnectionId::new(1), connection_cell.id().partition());
        let (connection, _physical) = ConnectionState::pending_open_for_test(info);
        connection
            .open(Some(OriginAdmission::lease_for_test(&admission)))
            .expect("bounded HTTP/2 connection did not open");
        let generation =
            OriginCell::install_h2_for_test(&connection_cell, connection.clone(), 1, None);
        let activation = OriginCell::activate_h2(
            &connection_cell,
            generation,
            requesting_cell.id().partition(),
        )
        .expect("accepting HTTP/2 generation did not activate");

        let waiter = OriginCell::register_waiter(&requesting_cell, ProtocolRequirement::H1Required);
        assert!(
            requesting_cell.take_ready_event(waiter).is_none(),
            "active HTTP/2 generation was reclaimed"
        );
        assert_eq!(None, connection.probe().close_reason);

        drop(activation);
        let super::super::AcquisitionStep::StartEstablishment(permit) = requesting_cell
            .take_ready_event(waiter)
            .expect("idle transition did not resume H1-required demand")
        else {
            panic!("idle HTTP/2 reclaim produced a non-capacity result");
        };
        assert_eq!(
            Some(CloseReason::Reclaimed),
            connection.probe().close_reason
        );
        drop(permit);
        assert_eq!(1, admission.available_capacity_for_test());
    }

    #[test]
    fn pool_shutdown_closes_an_accepting_generation() {
        let cell = cell();
        let (_generation, connection, _physical) = open_test_generation(&cell, 1);

        OriginCell::close_all(&cell, CloseReason::PoolDropped);

        assert_eq!(
            Some(CloseReason::PoolDropped),
            connection.probe().close_reason
        );
        assert_eq!(None, cell.accepting_h2_generation());
    }
}

#[cfg(all(test, smithy_http_client_loom))]
mod loom_tests {
    use super::*;
    use crate::client::pool::admission::ProtocolRequirement;
    use crate::client::pool::connection::{CloseReason, ConnectionInfo};
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::EligibilityGroup;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use http_1x::uri::Scheme;

    fn cell() -> Arc<OriginCell> {
        Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            OriginKey::from_parts(Scheme::HTTPS, "example.com", None).unwrap(),
            EligibilityGroup::Pool,
            None,
            None,
        ))
    }

    fn open_test_generation(cell: &Arc<OriginCell>) -> (H2GenerationId, Arc<ConnectionState>) {
        let (connection, _physical) = ConnectionState::unbounded(ConnectionInfo::for_test(
            ConnectionId::new(1),
            PartitionId::from_index(1),
        ));
        let generation = OriginCell::install_h2_for_test(cell, connection.clone(), 1, None);
        (generation, connection)
    }

    fn launching_waiter(cell: &Arc<OriginCell>) -> WaiterId {
        let waiter = OriginCell::register_waiter(cell, ProtocolRequirement::H2Required);
        let AcquisitionStep::StartEstablishment(permit) = cell
            .take_ready_event(waiter)
            .expect("new HTTP/2 waiter did not receive establishment authority")
        else {
            panic!("new HTTP/2 waiter completed before establishment");
        };
        assert!(cell.start_establishment(waiter));
        drop(permit);
        waiter
    }

    /// Races activation of one exact generation with close of that generation.
    ///
    /// Close either rejects activation or retains the draining generation until
    /// the prospective request releases its claim.
    #[test]
    fn activation_linearizes_against_generation_close() {
        loom::model(|| {
            let cell = cell();
            let (generation, connection) = open_test_generation(&cell);
            let activating_cell = cell.clone();
            let activation = loom::thread::spawn(move || {
                OriginCell::activate_h2(
                    &activating_cell,
                    generation,
                    activating_cell.id().partition(),
                )
            });
            let closing_cell = cell.clone();
            let close = loom::thread::spawn(move || {
                OriginCell::close_h2(&closing_cell, generation, CloseReason::Poisoned)
            });

            let activation = activation.join().unwrap();
            assert!(close.join().unwrap());
            assert_eq!(Some(CloseReason::Poisoned), connection.probe().close_reason);
            if let Some(activation) = activation {
                assert_eq!(
                    Some((1, 0)),
                    cell.h2_request_counts(generation),
                    "close did not retain the prospective activation"
                );
                drop(activation);
            }
            assert!(!cell.state.lock().h2.generations.contains_key(&generation));
        });
    }

    /// Completes the upload and response sides of one accepted request concurrently.
    ///
    /// The two guards share one request claim and must release its dispatch once.
    #[test]
    fn concurrent_request_side_completion_releases_one_dispatch() {
        loom::model(|| {
            let cell = cell();
            let (generation, connection) = open_test_generation(&cell);
            let mut activation = OriginCell::activate_h2(&cell, generation, cell.id().partition())
                .expect("generation did not activate");
            let H2DispatchParts {
                sender: _sender,
                upload,
                response,
            } = activation.take_dispatch_parts();
            let dispatch = ConnectionState::try_commit_dispatch(&connection)
                .expect("open connection rejected dispatch");
            activation.accept(dispatch);

            let upload = loom::thread::spawn(move || drop(upload));
            let response = loom::thread::spawn(move || drop(response));
            upload.join().unwrap();
            response.join().unwrap();

            assert_eq!(0, connection.probe().in_flight);
            let state = cell.state.lock();
            let record = state
                .h2
                .generations
                .get(&generation)
                .expect("accepting generation disappeared");
            assert_eq!(0, record.active_requests);
        });
    }

    /// Converges two post-ALPN attempts, then races flight completion with cancellation.
    ///
    /// Exactly one attempt owns the flight task. The live participant receives
    /// the installed generation while the cancelled participant is removed from
    /// either the flight or the generation gate.
    #[test]
    fn concurrent_h2_flight_convergence_installs_one_generation() {
        loom::model(|| {
            let cell = cell();
            let first = launching_waiter(&cell);
            let second = launching_waiter(&cell);

            let first_cell = cell.clone();
            let first_convergence =
                loom::thread::spawn(move || first_cell.converge_h2_flight(first));
            let second_cell = cell.clone();
            let second_convergence =
                loom::thread::spawn(move || second_cell.converge_h2_flight(second));
            let first_decision = first_convergence.join().unwrap();
            let second_decision = second_convergence.join().unwrap();
            let flight = match (first_decision, second_decision) {
                (H2FlightDecision::RunFlight(flight), H2FlightDecision::JoinedFlight)
                | (H2FlightDecision::JoinedFlight, H2FlightDecision::RunFlight(flight)) => flight,
                decisions => panic!("convergence did not select one flight owner: {decisions:?}"),
            };

            let (connection, physical) = ConnectionState::unbounded(ConnectionInfo::for_test(
                ConnectionId::new(1),
                PartitionId::from_index(1),
            ));
            let completing_cell = cell.clone();
            let completing_connection = connection.clone();
            let completing = loom::thread::spawn(move || {
                OriginCell::complete_h2_flight(
                    &completing_cell,
                    flight,
                    completing_connection,
                    H2Sender::test(1),
                    None,
                )
            });
            let cancelling_cell = cell.clone();
            let cancelling =
                loom::thread::spawn(move || OriginCell::cancel_waiter(&cancelling_cell, second));

            let generation = completing
                .join()
                .unwrap()
                .expect("the sole HTTP/2 flight did not install");
            assert!(cancelling.join().unwrap());
            let AcquisitionStep::Resolved(AcquisitionOutcome::H2(activation)) = cell
                .take_ready_event(first)
                .expect("live flight participant did not receive the generation")
            else {
                panic!("live flight participant received a non-HTTP/2 result");
            };
            drop(activation);

            assert_eq!(0, cell.retained_waiters_for_test());
            assert!(OriginCell::close_h2(
                &cell,
                generation,
                CloseReason::PoolDropped,
            ));
            drop(physical);
        });
    }
}
