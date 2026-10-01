/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/1 connection records and exclusive request-sender ownership.
//!
//! Every installed connection remains in its connection-owning [`H1CellState`]
//! until logical close. While the record is live, its exclusive sender exists
//! in exactly one place: inside an `Idle` record or in one external owner while
//! the record is `Selected` or `ReservedForPeer`. Transitions move the sender; they
//! never copy it. The residence records that ownership:
//!
//! ```text
//! completed handshake -- insert_selected ---------------------> Selected
//! test fixture ------- insert_idle ----------------------------> Idle(sender)
//!
//! Idle(sender) -- select_idle --------------------------------> Selected
//! Idle(sender) -- take_idle_candidate ----------------> ReservedForPeer
//! Selected ---- reserve_for_peer ---------------------> ReservedForPeer
//! ReservedForPeer -- commit_return_to_waiter -----------------> Selected
//! Selected ---- return_idle ----------------------------------> Idle(sender)
//! ReservedForPeer -- return_idle -----------------------------> Idle(sender)
//!
//! Idle(sender) ------------ begin_close ----------------------> Closing
//! Selected / ReservedForPeer -- begin_close / close_owned ---> Closing
//! Closing -------- remove_closed -----------------------------> removed
//! ```
//!
//! Outside the lock, ownership moves through values whose drop behavior is
//! specific to the protocol phase:
//!
//! ```text
//! H1Selection
//!   |-- dropped ------------------------------> return to connection-owning cell
//!   |-- retire_connection --------------------------------> logical close
//!   `-- request accepted --> H1Exchange
//!                              |-- incomplete or dropped --> logical close
//!                              `-- offer_for_reuse --------> return to connection-owning cell
//!
//! cross-cell reuse --> ProvisionalH1
//!                        |-- borrow ----------------------> H1Selection
//!                        |-- reclaim ---------------------> logical close
//!                        `-- rejected or dropped ---------> return to connection-owning cell
//! ```
//!
//! [`H1Selection`], [`H1Exchange`], and [`ProvisionalH1`] are the
//! sender-owning values outside the cell lock. Their drop behavior returns or
//! retires the sender so a cancellation cannot leave a record with no terminal
//! owner. They retain the owning cell weakly because a selected sender may
//! temporarily be the ready result in that same cell; cell teardown makes the
//! fallback close the connection directly.

use super::super::admission::{
    AdmissionAction, H1Candidate, H1MatchId, H1SupplyStatus, OriginAdmission, PreparedH1IdleProbe,
    PreparedH1Reservation, SupplyRevision,
};
use super::super::connection::{CloseReason, ConnectionState};
use super::{AcquisitionOutcome, OriginCell};
use crate::sync::{Arc, Weak};
use aws_smithy_runtime_api::client::connection::ConnectionId;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::task::{Context, Poll};
use std::time::SystemTime;

use aws_smithy_types::body::SdkBody;

/// Exclusive request handle for one Hyper HTTP/1 client connection.
///
/// Hyper creates one `SendRequest<SdkBody>` handle when the client handshake
/// completes. Pool code calls this handle the HTTP/1 sender. Moving it between
/// residence guards transfers authority to send the next request; it does not
/// move the socket or protocol driver. The test-only variant exercises those
/// ownership transitions without running Hyper.
pub(in crate::client::pool) enum H1Sender {
    /// Hyper's exclusive HTTP/1 request sender.
    Hyper(hyper::client::conn::http1::SendRequest<SdkBody>),
    /// Synthetic sender identity used only by ownership tests.
    #[cfg(test)]
    Test { id: u64, ready: bool },
}

impl H1Sender {
    /// Wraps a sender returned by a successful Hyper HTTP/1 handshake.
    pub(in crate::client::pool) fn from_hyper(
        sender: hyper::client::conn::http1::SendRequest<SdkBody>,
    ) -> Self {
        Self::Hyper(sender)
    }

    /// Returns the Hyper sender for readiness and dispatch.
    ///
    /// # Panics
    ///
    /// Panics when a test-only sender reaches the real dispatch path.
    pub(in crate::client::pool) fn hyper_mut(
        &mut self,
    ) -> &mut hyper::client::conn::http1::SendRequest<SdkBody> {
        match self {
            Self::Hyper(sender) => sender,
            #[cfg(test)]
            Self::Test { .. } => panic!("test HTTP/1 sender reached Hyper dispatch"),
        }
    }

    /// Returns whether Hyper already permits another request.
    fn is_ready(&self) -> bool {
        match self {
            Self::Hyper(sender) => sender.is_ready(),
            #[cfg(test)]
            Self::Test { ready, .. } => *ready,
        }
    }

    /// Polls Hyper for proof that another request may be sent.
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), hyper::Error>> {
        match self {
            Self::Hyper(sender) => sender.poll_ready(cx),
            #[cfg(test)]
            Self::Test { ready: true, .. } => Poll::Ready(Ok(())),
            #[cfg(test)]
            Self::Test { ready: false, .. } => Poll::Pending,
        }
    }

    /// Creates a synthetic sender for state-machine tests.
    #[cfg(test)]
    pub(in crate::client::pool) fn test(id: u64) -> Self {
        Self::Test { id, ready: true }
    }

    /// Creates a synthetic sender whose readiness remains pending.
    #[cfg(test)]
    pub(in crate::client::pool) fn pending_test(id: u64) -> Self {
        Self::Test { id, ready: false }
    }

    /// Returns the synthetic sender identity.
    #[cfg(test)]
    pub(super) fn test_id(&self) -> u64 {
        match self {
            Self::Test { id, .. } => *id,
            Self::Hyper(_) => panic!("Hyper sender used in a synthetic ownership test"),
        }
    }
}

impl fmt::Debug for H1Sender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hyper(_) => f.write_str("H1Sender::Hyper"),
            #[cfg(test)]
            Self::Test { id, ready } => f
                .debug_struct("H1Sender::Test")
                .field("id", id)
                .field("ready", ready)
                .finish(),
        }
    }
}

/// Local reservation for one peer match and its fairness debt.
///
/// Reservation residence and fairness debt are separate because a completed
/// transfer releases the reservation immediately, while the debt remains until
/// later local service or the disappearance of compatible local demand.
#[derive(Debug, Default)]
struct H1Reservation {
    /// State of the current peer reservation.
    state: H1ReservationState,
    /// Whether this cell's next usable HTTP/1 turn must remain local after the
    /// operation that earned it has completed.
    local_turn_owed: bool,
}

/// Authoritative state of one cell-local peer reservation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum H1ReservationState {
    /// No peer match may intercept a sender return.
    #[default]
    Available,
    /// The next reusable sender return is reserved for this match.
    Installed(H1MatchId),
    /// A provisional sender is outside the cell lock for this match.
    Resolving(H1MatchId),
}

impl H1Reservation {
    /// Reserves a future reusable return for one retained match.
    fn reserve_waiting(&mut self, match_id: H1MatchId) -> bool {
        if !matches!(self.state, H1ReservationState::Available) {
            return false;
        }
        self.state = H1ReservationState::Installed(match_id);
        true
    }

    /// Reserves a match that already owns an extracted idle sender.
    fn reserve_resolving(&mut self, match_id: H1MatchId) -> bool {
        if !matches!(self.state, H1ReservationState::Available) {
            return false;
        }
        self.state = H1ReservationState::Resolving(match_id);
        true
    }

    /// Transfers a waiting match into resolution on sender return.
    fn take_return_match(&mut self) -> Option<H1MatchId> {
        let H1ReservationState::Installed(match_id) = self.state else {
            return None;
        };
        self.state = H1ReservationState::Resolving(match_id);
        Some(match_id)
    }

    /// Releases a matching reservation without earning a local fairness turn.
    fn release(&mut self, match_id: H1MatchId) -> bool {
        if !self.names(match_id) {
            return false;
        }
        self.state = H1ReservationState::Available;
        true
    }

    /// Completes an irreversible transfer and records any usable local turn.
    fn complete_transfer(&mut self, match_id: H1MatchId, local_h1_demand: bool) -> bool {
        if !matches!(self.state, H1ReservationState::Resolving(current) if current == match_id) {
            return false;
        }
        self.state = H1ReservationState::Available;
        self.local_turn_owed |= local_h1_demand;
        true
    }

    /// Returns whether a usable local turn currently excludes a peer H1 match.
    fn blocks_peer_selection(&self, local_h1_demand: bool) -> bool {
        self.local_turn_owed && local_h1_demand
    }

    /// Consumes an owed turn when local HTTP/1 service wins.
    fn consume_local_turn(&mut self) -> bool {
        if !self.local_turn_owed {
            return false;
        }
        self.local_turn_owed = false;
        true
    }

    /// Clears debt that can no longer be consumed by local demand.
    fn clear_unused_turn(&mut self, local_h1_demand: bool) -> bool {
        if !self.local_turn_owed || local_h1_demand {
            return false;
        }
        self.local_turn_owed = false;
        true
    }

    /// Returns whether another peer reservation may be installed.
    fn is_available(&self) -> bool {
        matches!(self.state, H1ReservationState::Available)
    }

    /// Returns whether this reservation still names the given operation.
    fn names(&self, match_id: H1MatchId) -> bool {
        matches!(
            self.state,
            H1ReservationState::Installed(current) | H1ReservationState::Resolving(current)
                if current == match_id
        )
    }

    /// Checks relationships that are not already encoded by the state enum.
    #[cfg(any(debug_assertions, test))]
    fn assert_consistent(&self, supports_peer_reservation: bool) {
        if std::thread::panicking() {
            return;
        }
        if matches!(self.state, H1ReservationState::Installed(_)) {
            assert!(
                supports_peer_reservation,
                "installed HTTP/1 peer reservation had no externally owned sender record to settle it"
            );
        }
    }

    #[cfg(test)]
    fn local_turn_owed(&self) -> bool {
        self.local_turn_owed
    }
}

/// Complete HTTP/1 sender and peer-reservation state under one cell lock.
#[derive(Debug, Default)]
pub(super) struct H1CellState {
    /// Every installed record that has not completed logical close.
    records: HashMap<ConnectionId, H1Record>,
    /// Reusable records in return order; selection takes the newest sender.
    idle_order: VecDeque<ConnectionId>,
    /// Peer match that may intercept the next reusable sender return.
    peer_reservation: H1Reservation,
}

/// One cell-owned HTTP/1 connection record.
#[derive(Debug)]
struct H1Record {
    /// Shared logical and physical connection lifetime.
    connection: Arc<ConnectionState>,
    /// Location of the record's exclusive sender.
    sender_state: H1SenderResidence,
}

/// Authoritative location of one exclusive HTTP/1 sender.
#[derive(Debug)]
enum H1SenderResidence {
    /// The sender is stored in this record and available for local selection.
    Idle {
        /// Exclusive sender available for the next local request.
        sender: H1Sender,
        /// Maintenance deadline, absent when idle expiry is disabled.
        deadline: Option<SystemTime>,
    },
    /// A request-side [`H1Selection`] or [`H1Exchange`] owns the sender.
    Selected,
    /// A [`ProvisionalH1`] owns the sender for cross-cell reuse.
    ReservedForPeer,
    /// Logical close has started and no new selection or return may commit.
    Closing,
}

impl H1CellState {
    /// Returns whether admission may install another peer reservation.
    pub(super) fn peer_reservation_available(&self) -> bool {
        self.peer_reservation.is_available()
    }

    /// Returns whether fairness currently excludes peer selection.
    pub(super) fn blocks_peer_selection(&self, local_h1_demand: bool) -> bool {
        self.peer_reservation.blocks_peer_selection(local_h1_demand)
    }

    /// Reserves a future sender return for one retained peer match.
    pub(super) fn reserve_waiting(&mut self, match_id: H1MatchId) -> bool {
        self.peer_reservation.reserve_waiting(match_id)
    }

    /// Reserves a peer match that already owns an extracted sender.
    pub(super) fn reserve_resolving(&mut self, match_id: H1MatchId) -> bool {
        self.peer_reservation.reserve_resolving(match_id)
    }

    /// Takes a waiting peer match when a reusable sender returns.
    pub(super) fn take_return_match(&mut self) -> Option<H1MatchId> {
        self.peer_reservation.take_return_match()
    }

    /// Releases a peer reservation without recording a local turn.
    pub(super) fn release_peer_reservation(&mut self, match_id: H1MatchId) -> bool {
        self.peer_reservation.release(match_id)
    }

    /// Returns whether the peer reservation still names `match_id`.
    pub(super) fn names_peer_reservation(&self, match_id: H1MatchId) -> bool {
        self.peer_reservation.names(match_id)
    }

    /// Completes a transferred peer sender and records any local fairness turn.
    pub(super) fn complete_peer_transfer(
        &mut self,
        match_id: H1MatchId,
        local_h1_demand: bool,
    ) -> bool {
        self.peer_reservation
            .complete_transfer(match_id, local_h1_demand)
    }

    /// Consumes an owed local turn after local HTTP/1 service wins.
    pub(super) fn consume_local_turn(&mut self) -> bool {
        self.peer_reservation.consume_local_turn()
    }

    /// Clears fairness debt that no live local demand can consume.
    pub(super) fn clear_unused_turn(&mut self, local_h1_demand: bool) -> bool {
        self.peer_reservation.clear_unused_turn(local_h1_demand)
    }

    /// Returns whether this cell owes its next usable sender to local demand.
    #[cfg(test)]
    pub(super) fn local_turn_owed(&self) -> bool {
        self.peer_reservation.local_turn_owed()
    }

    /// Installs a fresh connection as selected by its launching acquisition.
    pub(super) fn insert_selected(
        &mut self,
        connection: Arc<ConnectionState>,
        sender: H1Sender,
    ) -> Result<OwnedH1Sender, OwnedH1Sender> {
        let owner = OwnedH1Sender::new(connection, sender, false);
        if self.records.contains_key(&owner.id()) {
            return Err(owner);
        }
        self.records.insert(
            owner.id(),
            H1Record {
                connection: owner.connection.clone(),
                sender_state: H1SenderResidence::Selected,
            },
        );
        self.assert_consistent();
        Ok(owner)
    }

    /// Installs a connection that completed without a live launching waiter.
    #[cfg(test)]
    pub(super) fn insert_idle(
        &mut self,
        connection: Arc<ConnectionState>,
        sender: H1Sender,
        deadline: Option<SystemTime>,
    ) -> Result<(), OwnedH1Sender> {
        let owner = OwnedH1Sender::new(connection, sender, true);
        if self.records.contains_key(&owner.id()) {
            return Err(owner);
        }
        let id = owner.id();
        let OwnedH1Sender {
            connection, sender, ..
        } = owner;
        self.records.insert(
            id,
            H1Record {
                connection,
                sender_state: H1SenderResidence::Idle { sender, deadline },
            },
        );
        self.idle_order.push_back(id);
        self.assert_consistent();
        Ok(())
    }

    /// Takes the most recently returned idle sender for one request.
    pub(super) fn select_idle(&mut self) -> Option<OwnedH1Sender> {
        let id = self.idle_order.pop_back()?;
        let record = self
            .records
            .get_mut(&id)
            .expect("idle HTTP/1 record disappeared");
        if !matches!(record.sender_state, H1SenderResidence::Idle { .. }) {
            panic!("idle HTTP/1 order named a non-idle record");
        }
        let H1SenderResidence::Idle { sender, .. } =
            std::mem::replace(&mut record.sender_state, H1SenderResidence::Selected)
        else {
            unreachable!("HTTP/1 residence changed under the cell lock");
        };
        let owner = OwnedH1Sender::new(record.connection.clone(), sender, true);
        self.assert_consistent();
        Some(owner)
    }

    /// Extracts the newest idle sender into provisional return residence.
    ///
    /// A returning sender uses this path so rejection can follow the same ordinary
    /// return fallback as a sender intercepted after an active exchange.
    pub(super) fn take_idle_candidate(&mut self) -> Option<OwnedH1Sender> {
        let id = self.idle_order.pop_back()?;
        let record = self
            .records
            .get_mut(&id)
            .expect("idle HTTP/1 record disappeared");
        let H1SenderResidence::Idle { sender, .. } =
            std::mem::replace(&mut record.sender_state, H1SenderResidence::ReservedForPeer)
        else {
            panic!("idle HTTP/1 order named a non-idle record");
        };
        let owner = OwnedH1Sender::new(record.connection.clone(), sender, true);
        self.assert_consistent();
        Some(owner)
    }

    /// Returns whether a sender could satisfy peer selection now or on return.
    pub(super) fn has_returnable(&self) -> bool {
        self.records.values().any(|record| {
            matches!(
                record.sender_state,
                H1SenderResidence::Idle { .. }
                    | H1SenderResidence::Selected
                    | H1SenderResidence::ReservedForPeer
            )
        })
    }

    /// Returns whether `owner` may still re-enter reusable return policy.
    pub(super) fn accepts_return(&self, owner: &OwnedH1Sender) -> bool {
        self.records.get(&owner.id()).is_some_and(|record| {
            matches!(
                record.sender_state,
                H1SenderResidence::Selected | H1SenderResidence::ReservedForPeer
            )
        })
    }

    /// Returns whether an installed peer reservation still has sender state.
    ///
    /// Logical close may move an externally owned sender to `Closing` before
    /// its return resolves the installed match, so reservation consistency is
    /// broader than current peer selectability.
    #[cfg(any(debug_assertions, test))]
    fn supports_peer_reservation(&self) -> bool {
        self.records.values().any(|record| {
            matches!(
                record.sender_state,
                H1SenderResidence::Selected
                    | H1SenderResidence::ReservedForPeer
                    | H1SenderResidence::Closing
            )
        })
    }

    /// Reserves an external sender for cross-cell reuse.
    pub(super) fn reserve_for_peer(&mut self, owner: &OwnedH1Sender) -> bool {
        let Some(record) = self.records.get_mut(&owner.id()) else {
            return false;
        };
        match record.sender_state {
            H1SenderResidence::Selected => {
                record.sender_state = H1SenderResidence::ReservedForPeer;
                self.assert_consistent();
                true
            }
            H1SenderResidence::ReservedForPeer => true,
            H1SenderResidence::Idle { .. } | H1SenderResidence::Closing => false,
        }
    }

    /// Restores a returned sender to idle storage.
    pub(super) fn return_idle(
        &mut self,
        owner: OwnedH1Sender,
        deadline: Option<SystemTime>,
    ) -> Result<(), OwnedH1Sender> {
        let Some(record) = self.records.get_mut(&owner.id()) else {
            return Err(owner);
        };
        if !matches!(
            record.sender_state,
            H1SenderResidence::Selected | H1SenderResidence::ReservedForPeer
        ) {
            return Err(owner);
        }
        let id = owner.id();
        let OwnedH1Sender { sender, .. } = owner;
        record.sender_state = H1SenderResidence::Idle { sender, deadline };
        self.idle_order.push_back(id);
        self.assert_consistent();
        Ok(())
    }

    /// Commits a selected or returning sender to a waiting request.
    pub(super) fn commit_return_to_waiter(&mut self, owner: &OwnedH1Sender) -> bool {
        let Some(record) = self.records.get_mut(&owner.id()) else {
            return false;
        };
        match record.sender_state {
            H1SenderResidence::Selected => true,
            H1SenderResidence::ReservedForPeer => {
                record.sender_state = H1SenderResidence::Selected;
                self.assert_consistent();
                true
            }
            H1SenderResidence::Idle { .. } | H1SenderResidence::Closing => false,
        }
    }

    /// Marks a selected or returning sender as closing.
    ///
    /// The caller still owns the sender and must complete logical close after
    /// releasing the cell lock.
    pub(super) fn close_owned(&mut self, owner: &OwnedH1Sender) -> bool {
        let should_close = match self.records.get_mut(&owner.id()) {
            Some(record) => match record.sender_state {
                H1SenderResidence::Selected | H1SenderResidence::ReservedForPeer => {
                    record.sender_state = H1SenderResidence::Closing;
                    true
                }
                H1SenderResidence::Closing => false,
                H1SenderResidence::Idle { .. } => {
                    panic!("externally owned HTTP/1 sender was recorded as idle")
                }
            },
            None => true,
        };
        self.assert_consistent();
        should_close
    }

    /// Starts close for a record named without its external sender.
    ///
    /// Idle close extracts the sender. Selected and returning records stay
    /// represented as `Closing` until their external owner comes back.
    pub(super) fn begin_close(&mut self, id: ConnectionId) -> Option<H1CloseTransition> {
        let record = self.records.get_mut(&id)?;
        let connection = record.connection.clone();
        let sender = match &record.sender_state {
            H1SenderResidence::Idle { .. } => {
                let position = self
                    .idle_order
                    .iter()
                    .position(|candidate| *candidate == id)
                    .expect("idle HTTP/1 record was absent from idle order");
                self.idle_order.remove(position);
                let H1SenderResidence::Idle { sender, .. } =
                    std::mem::replace(&mut record.sender_state, H1SenderResidence::Closing)
                else {
                    unreachable!("HTTP/1 residence changed under the cell lock");
                };
                Some(sender)
            }
            H1SenderResidence::Selected | H1SenderResidence::ReservedForPeer => {
                record.sender_state = H1SenderResidence::Closing;
                None
            }
            H1SenderResidence::Closing => return None,
        };
        self.assert_consistent();
        Some(H1CloseTransition { connection, sender })
    }

    /// Removes a record after logical close and sender destruction complete.
    pub(super) fn remove_closed(&mut self, id: ConnectionId) {
        let Some(record) = self.records.get(&id) else {
            return;
        };
        if !matches!(record.sender_state, H1SenderResidence::Closing) {
            return;
        }
        self.records.remove(&id);
        self.assert_consistent();
    }

    /// Returns idle and externally owned sender counts.
    pub(super) fn connection_counts(&self) -> (usize, usize) {
        self.records
            .values()
            .fold((0, 0), |(idle, active), record| match record.sender_state {
                H1SenderResidence::Idle { .. } => (idle + 1, active),
                H1SenderResidence::Selected | H1SenderResidence::ReservedForPeer => {
                    (idle, active + 1)
                }
                H1SenderResidence::Closing => (idle, active),
            })
    }

    /// Returns the installed record and idle counts.
    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize) {
        (self.records.len(), self.idle_order.len())
    }

    /// Returns the synthetic identities of every idle sender.
    #[cfg(all(test, smithy_http_client_loom))]
    pub(super) fn idle_sender_ids(&self) -> Vec<u64> {
        self.records
            .values()
            .filter_map(|record| match &record.sender_state {
                H1SenderResidence::Idle { sender, .. } => Some(sender.test_id()),
                H1SenderResidence::Selected
                | H1SenderResidence::ReservedForPeer
                | H1SenderResidence::Closing => None,
            })
            .collect()
    }

    /// Returns the sole installed connection for focused dispatch tests.
    #[cfg(all(test, feature = "rt-tokio"))]
    pub(super) fn only_connection_for_test(&self) -> Arc<ConnectionState> {
        assert_eq!(
            1,
            self.records.len(),
            "expected exactly one installed HTTP/1 record"
        );
        self.records
            .values()
            .next()
            .expect("HTTP/1 record count changed under the cell lock")
            .connection
            .clone()
    }

    /// Returns idle records whose configured deadline has elapsed.
    pub(super) fn expired_idle(&self, now: SystemTime) -> Vec<ConnectionId> {
        self.idle_order
            .iter()
            .copied()
            .filter(|id| {
                matches!(
                    self.records.get(id).map(|record| &record.sender_state),
                    Some(H1SenderResidence::Idle {
                        deadline: Some(deadline),
                        ..
                    }) if *deadline <= now
                )
            })
            .collect()
    }

    /// Returns the nearest configured deadline among reusable senders.
    pub(super) fn nearest_idle_deadline(&self) -> Option<SystemTime> {
        self.idle_order
            .iter()
            .filter_map(|id| {
                match &self
                    .records
                    .get(id)
                    .expect("idle HTTP/1 record disappeared")
                    .sender_state
                {
                    H1SenderResidence::Idle { deadline, .. } => *deadline,
                    _ => unreachable!("idle HTTP/1 order named a non-idle record"),
                }
            })
            .min()
    }

    /// Returns every installed record identity for pool-wide shutdown.
    pub(super) fn connection_ids(&self) -> Vec<ConnectionId> {
        self.records.keys().copied().collect()
    }

    /// Checks that idle records and the idle order describe the same set.
    pub(super) fn assert_consistent(&self) {
        #[cfg(any(debug_assertions, test))]
        {
            if std::thread::panicking() {
                return;
            }
            self.peer_reservation
                .assert_consistent(self.supports_peer_reservation());
            let idle_records = self
                .records
                .values()
                .filter(|record| matches!(record.sender_state, H1SenderResidence::Idle { .. }))
                .count();
            assert_eq!(
                idle_records,
                self.idle_order.len(),
                "HTTP/1 idle count did not match idle order"
            );
            let mut seen = HashMap::new();
            for id in &self.idle_order {
                assert!(
                    matches!(
                        self.records.get(id).map(|record| &record.sender_state),
                        Some(H1SenderResidence::Idle { .. })
                    ),
                    "HTTP/1 idle order named a missing or non-idle record"
                );
                assert!(
                    seen.insert(*id, ()).is_none(),
                    "HTTP/1 idle order contained a duplicate record"
                );
            }
        }
    }
}

/// Exclusive sender and connection state detached from a connection-owning cell record.
#[derive(Debug)]
pub(super) struct OwnedH1Sender {
    /// Shared lifetime state for the physical connection.
    connection: Arc<ConnectionState>,
    /// The one Hyper HTTP/1 sender for the connection.
    sender: H1Sender,
    /// Whether this sender came from an already installed reusable record.
    reused: bool,
}

impl OwnedH1Sender {
    /// Creates detached sender ownership.
    fn new(connection: Arc<ConnectionState>, sender: H1Sender, reused: bool) -> Self {
        Self {
            connection,
            sender,
            reused,
        }
    }

    /// Returns the installed connection identity.
    pub(super) fn id(&self) -> ConnectionId {
        self.connection.id()
    }

    /// Returns whether this selection came from an existing reusable record.
    pub(super) fn is_reused(&self) -> bool {
        self.reused
    }

    /// Marks the next request as reusing an already established connection.
    pub(super) fn mark_reused(&mut self) {
        self.reused = true;
    }

    /// Returns the shared connection state.
    pub(super) fn connection(&self) -> &Arc<ConnectionState> {
        &self.connection
    }

    /// Returns the exclusive sender.
    pub(super) fn sender_mut(&mut self) -> &mut H1Sender {
        &mut self.sender
    }

    /// Returns the synthetic sender identity.
    #[cfg(test)]
    pub(super) fn test_sender_id(&self) -> u64 {
        self.sender.test_id()
    }
}

/// Returns an external sender to its connection-owning cell.
///
/// A sender whose cell was torn down closes its connection directly.
fn return_to_connection_cell(connection_cell: &Weak<OriginCell>, owner: OwnedH1Sender) {
    if let Some(connection_cell) = connection_cell.upgrade() {
        OriginCell::return_h1_owner(&connection_cell, owner);
    } else {
        owner.connection().logical_close(CloseReason::PoolDropped);
        drop(owner);
    }
}

/// Retires an external sender through its connection-owning cell.
///
/// A sender whose cell was torn down closes its connection directly.
fn retire_at_connection_cell(
    connection_cell: &Weak<OriginCell>,
    owner: OwnedH1Sender,
    reason: CloseReason,
) {
    if let Some(connection_cell) = connection_cell.upgrade() {
        OriginCell::retire_h1_owner(&connection_cell, owner, reason);
    } else {
        owner.connection().logical_close(reason);
        drop(owner);
    }
}

/// State detached when close begins for a record named by identity.
pub(super) struct H1CloseTransition {
    /// Connection whose logical lifetime close must end.
    pub(super) connection: Arc<ConnectionState>,
    /// Idle sender extracted for destruction, if close found one in the cell.
    pub(super) sender: Option<H1Sender>,
}

/// Exclusive sender checked out for readiness and one request dispatch.
///
/// Local selection, successful establishment, or peer reuse creates this value
/// while the installed record is `Selected`. The value alone may use the
/// sender. Hyper accepting the request transfers ownership to [`H1Exchange`];
/// explicit retirement closes the installed record. Dropping an undispatched
/// selection returns the sender through ordinary connection-owning-cell policy.
pub(in crate::client::pool) struct H1Selection {
    /// Non-retaining reference to the cell that owns the installed record.
    connection_cell: Weak<OriginCell>,
    /// Sender ownership until return, retirement, or response transfer.
    owner: Option<OwnedH1Sender>,
}

impl H1Selection {
    /// Creates a selected sender owned outside the connection-owning cell lock.
    pub(super) fn new(connection_cell: &Arc<OriginCell>, owner: OwnedH1Sender) -> Self {
        Self {
            connection_cell: Weak::from_arc(connection_cell),
            owner: Some(owner),
        }
    }

    /// Returns this selection's physical connection identity.
    pub(in crate::client::pool) fn connection_id(&self) -> ConnectionId {
        self.owner
            .as_ref()
            .expect("HTTP/1 selection consumed more than once")
            .id()
    }

    /// Returns whether this sender came from a reusable installed record.
    pub(in crate::client::pool) fn is_reused(&self) -> bool {
        self.owner
            .as_ref()
            .expect("HTTP/1 selection consumed more than once")
            .is_reused()
    }

    /// Returns the selected connection state.
    pub(in crate::client::pool) fn connection(&self) -> &Arc<ConnectionState> {
        self.owner
            .as_ref()
            .expect("HTTP/1 selection consumed more than once")
            .connection()
    }

    /// Returns the exclusive sender for readiness and dispatch.
    pub(in crate::client::pool) fn sender_mut(&mut self) -> &mut H1Sender {
        self.owner
            .as_mut()
            .expect("HTTP/1 selection consumed more than once")
            .sender_mut()
    }

    /// Transfers the sender after Hyper accepts the request.
    ///
    /// The installed record remains `Selected`; [`H1Exchange`] now owns the
    /// sender until the response proves reuse or requires retirement.
    pub(in crate::client::pool) fn into_exchange(mut self) -> H1Exchange {
        H1Exchange {
            connection_cell: self.connection_cell.clone(),
            owner: self.owner.take(),
        }
    }

    /// Returns non-retaining close authority for this selected connection.
    pub(in crate::client::pool) fn close_handle(&self) -> H1CloseHandle {
        let owner = self
            .owner
            .as_ref()
            .expect("HTTP/1 selection consumed more than once");
        H1CloseHandle {
            connection_cell: self.connection_cell.clone(),
            connection: Weak::from_arc(owner.connection()),
            connection_id: owner.id(),
        }
    }

    /// Retires this selection instead of returning it for reuse.
    pub(in crate::client::pool) fn retire_connection(mut self, reason: CloseReason) {
        if let Some(owner) = self.owner.take() {
            retire_at_connection_cell(&self.connection_cell, owner, reason);
        }
    }

    /// Returns the synthetic sender identity.
    #[cfg(test)]
    pub(super) fn test_sender_id(&self) -> u64 {
        self.owner
            .as_ref()
            .expect("HTTP/1 selection consumed more than once")
            .test_sender_id()
    }
}

impl fmt::Debug for H1Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("H1Selection")
            .field(
                "connection-owning cell",
                &self
                    .connection_cell
                    .upgrade()
                    .map(|connection_cell| connection_cell.id().clone()),
            )
            .field("connection_id", &self.owner.as_ref().map(OwnedH1Sender::id))
            .finish()
    }
}

impl Drop for H1Selection {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.take() {
            return_to_connection_cell(&self.connection_cell, owner);
        }
    }
}

/// Exclusive sender for a request Hyper accepted on this connection.
///
/// The installed record remains `Selected` while response-body processing owns
/// this value. After a complete response and successful Hyper readiness,
/// [`H1Exchange::offer_for_reuse`] runs owning-cell return arbitration.
/// Explicit retirement handles protocol failure or upgrade. Dropping the
/// exchange means a reusable message boundary was not proven and closes the
/// connection as an incomplete HTTP/1 exchange.
pub(in crate::client::pool) struct H1Exchange {
    /// Non-retaining reference to the cell that owns the selected record.
    connection_cell: Weak<OriginCell>,
    /// Sender held until Hyper proves it may return.
    owner: Option<OwnedH1Sender>,
}

impl H1Exchange {
    /// Returns the installed connection that owns this exchange.
    pub(in crate::client::pool) fn connection(&self) -> &Arc<ConnectionState> {
        self.owner
            .as_ref()
            .expect("HTTP/1 exchange consumed more than once")
            .connection()
    }

    /// Returns whether Hyper already permits another request.
    pub(in crate::client::pool) fn is_ready(&self) -> bool {
        self.owner
            .as_ref()
            .expect("HTTP/1 exchange consumed more than once")
            .sender
            .is_ready()
    }

    /// Polls Hyper for proof that the sender can accept another request.
    pub(in crate::client::pool) fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), hyper::Error>> {
        self.owner
            .as_mut()
            .expect("HTTP/1 exchange consumed more than once")
            .sender_mut()
            .poll_ready(cx)
    }

    /// Offers a proven-ready sender to owning-cell reuse policy.
    pub(in crate::client::pool) fn offer_for_reuse(mut self) {
        let mut owner = self
            .owner
            .take()
            .expect("HTTP/1 exchange consumed more than once");
        let connection = owner.connection().clone();
        owner.mark_reused();
        return_to_connection_cell(&self.connection_cell, owner);
        connection.complete_h1_exchange(CloseReason::ProtocolClosed);
    }

    /// Retires the sender instead of returning it to the connection-owning cell.
    pub(in crate::client::pool) fn retire_connection(mut self, reason: CloseReason) {
        if let Some(owner) = self.owner.take() {
            let connection = owner.connection().clone();
            retire_at_connection_cell(&self.connection_cell, owner, reason);
            connection.complete_h1_exchange(reason);
        }
    }
}

impl fmt::Debug for H1Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("H1Exchange")
            .field(
                "connection-owning cell",
                &self
                    .connection_cell
                    .upgrade()
                    .map(|connection_cell| connection_cell.id().clone()),
            )
            .field("connection_id", &self.owner.as_ref().map(OwnedH1Sender::id))
            .finish()
    }
}

impl Drop for H1Exchange {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.take() {
            let connection = owner.connection().clone();
            retire_at_connection_cell(
                &self.connection_cell,
                owner,
                CloseReason::IncompleteH1Exchange,
            );
            connection.complete_h1_exchange(CloseReason::IncompleteH1Exchange);
        }
    }
}

/// Sender temporarily detached from a `ReservedForPeer` record.
///
/// Admission matching creates this value before crossing from the
/// connection-owning cell to a requesting cell. A successful borrow converts
/// it into [`H1Selection`]; reclaim closes the connection and releases its
/// capacity. Rejection, cancellation, or `Drop` returns the sender through
/// ordinary connection-owning-cell policy.
pub(in crate::client::pool) struct ProvisionalH1 {
    /// Non-retaining owning cell whose record remains `ReservedForPeer`.
    connection_cell: Weak<OriginCell>,
    /// Sender reserved by the provisional action.
    owner: Option<OwnedH1Sender>,
}

impl ProvisionalH1 {
    /// Creates a provisional owner for a sender extracted by admission matching.
    pub(super) fn new(connection_cell: &Arc<OriginCell>, owner: OwnedH1Sender) -> Self {
        Self {
            connection_cell: Weak::from_arc(connection_cell),
            owner: Some(owner),
        }
    }

    /// Returns the selected connection identity without consuming the sender.
    pub(in crate::client::pool) fn connection_id(&self) -> ConnectionId {
        self.owner
            .as_ref()
            .expect("provisional HTTP/1 sender consumed more than once")
            .id()
    }

    /// Transfers the cell reference and sender into the next match transition.
    pub(super) fn into_parts(mut self) -> (Weak<OriginCell>, OwnedH1Sender) {
        let owner = self
            .owner
            .take()
            .expect("provisional HTTP/1 sender consumed more than once");
        (self.connection_cell.clone(), owner)
    }

    /// Restores provisional ownership when a match transition cannot commit.
    pub(super) fn from_parts(connection_cell: Weak<OriginCell>, owner: OwnedH1Sender) -> Self {
        Self {
            connection_cell,
            owner: Some(owner),
        }
    }
}

impl fmt::Debug for ProvisionalH1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProvisionalH1")
            .field(
                "connection-owning cell",
                &self
                    .connection_cell
                    .upgrade()
                    .map(|connection_cell| connection_cell.id().clone()),
            )
            .field("connection_id", &self.owner.as_ref().map(OwnedH1Sender::id))
            .finish()
    }
}

impl Drop for ProvisionalH1 {
    fn drop(&mut self) {
        if let Some(owner) = self.owner.take() {
            return_to_connection_cell(&self.connection_cell, owner);
        }
    }
}

/// Non-retaining authority to retire one installed H1 record.
#[derive(Clone, Debug)]
pub(in crate::client::pool) struct H1CloseHandle {
    /// Connection-owning cell used to remove dispatch eligibility before
    /// logical close.
    connection_cell: Weak<OriginCell>,
    /// Core fallback when the connection-owning cell no longer exists.
    connection: Weak<ConnectionState>,
    /// Generation identity rejected by stale close actions.
    connection_id: ConnectionId,
}

impl H1CloseHandle {
    /// Creates a close handle for a newly installed H1 record.
    pub(in crate::client::pool) fn new(
        connection_cell: &Arc<OriginCell>,
        connection: &Arc<ConnectionState>,
    ) -> Self {
        Self {
            connection_cell: Weak::from_arc(connection_cell),
            connection: Weak::from_arc(connection),
            connection_id: connection.id(),
        }
    }

    /// Begins logical close and returns whether this signal won.
    pub(in crate::client::pool) fn close(&self, reason: CloseReason) -> bool {
        if let Some(connection_cell) = self.connection_cell.upgrade() {
            return OriginCell::close_h1(&connection_cell, self.connection_id, reason);
        }
        self.connection
            .upgrade()
            .is_some_and(|connection| connection.logical_close(reason))
    }
}

/// Driver-owned fallback that closes its H1 record on termination.
#[derive(Debug)]
pub(in crate::client::pool) struct H1DriverGuard {
    /// Non-retaining connection close authority.
    close: H1CloseHandle,
    /// Whether drop still represents owner-runtime shutdown.
    active: bool,
}

impl H1DriverGuard {
    /// Arms driver-lifecycle cleanup for an installed H1 record.
    pub(in crate::client::pool) fn new(close: H1CloseHandle) -> Self {
        Self {
            close,
            active: true,
        }
    }

    /// Records ordinary protocol-driver completion.
    pub(in crate::client::pool) fn protocol_closed(mut self) {
        self.active = false;
        self.close.close(CloseReason::ProtocolClosed);
    }
}

impl Drop for H1DriverGuard {
    fn drop(&mut self) {
        if self.active {
            self.close.close(CloseReason::OwnerRuntimeShutdown);
        }
    }
}

impl OriginCell {
    /// Installs a fresh H1 record selected by its launching request.
    ///
    /// # Panics
    ///
    /// Panics if this cell already contains the connection identity.
    pub(in crate::client::pool) fn insert_selected_h1(
        cell: &Arc<Self>,
        connection: Arc<ConnectionState>,
        sender: H1Sender,
    ) -> H1Selection {
        let (installed, supply_update) = {
            let mut state = cell.state.lock();
            let installed = state.h1.insert_selected(connection, sender);
            state.assert_consistent();
            (installed, state.take_h1_supply_update())
        };
        cell.submit_h1_supply_update(supply_update);
        match installed {
            Ok(owner) => H1Selection::new(cell, owner),
            Err(owner) => {
                owner
                    .connection()
                    .logical_close(CloseReason::ProtocolClosed);
                drop(owner);
                panic!("duplicate HTTP/1 connection identity installed in one cell");
            }
        }
    }

    /// Installs an H1 record whose launching acquisition already completed.
    ///
    /// # Panics
    ///
    /// Panics if this cell already contains the connection identity.
    #[cfg(test)]
    pub(in crate::client::pool) fn insert_idle_h1(
        cell: &Arc<Self>,
        connection: Arc<ConnectionState>,
        sender: H1Sender,
    ) {
        let deadline = cell.idle_deadline();
        let (installed, supply_update) = {
            let mut state = cell.state.lock();
            let installed = state.h1.insert_idle(connection, sender, deadline);
            state.assert_consistent();
            (installed, state.take_h1_supply_update())
        };
        cell.submit_h1_supply_update(supply_update);
        if let Err(owner) = installed {
            owner
                .connection()
                .logical_close(CloseReason::ProtocolClosed);
            drop(owner);
            panic!("duplicate HTTP/1 connection identity installed in one cell");
        }
        cell.notify_maintenance(deadline);
    }

    /// Selects the newest reusable H1 sender without origin-wide coordination.
    pub(in crate::client::pool) fn select_h1(cell: &Arc<Self>) -> Option<H1Selection> {
        let (owner, supply_update) = {
            let mut state = cell.state.lock();
            let owner = state.h1.select_idle();
            if owner.is_some() {
                state.h1.consume_local_turn();
            }
            state.assert_consistent();
            let supply_update = cell
                .admission
                .as_ref()
                .and_then(|_| state.take_h1_supply_update());
            (owner, supply_update)
        };
        cell.submit_h1_supply_update(supply_update);
        let owner = owner?;
        Some(H1Selection::new(cell, owner))
    }

    /// Commits one admission-selected supplier reservation at this cell.
    pub(in crate::client::pool) fn commit_h1_reservation(
        cell: &Arc<Self>,
        admission: Arc<OriginAdmission>,
        prepared: PreparedH1Reservation,
    ) -> Option<AdmissionAction> {
        let decision = {
            let mut state = cell.state.lock();
            state.commit_h1_reservation(prepared.match_id)
        };

        let decision = decision.map_candidate(|owner| {
            let provisional = ProvisionalH1::new(cell, owner);
            H1Candidate::new(
                admission.clone(),
                prepared.match_id,
                cell.id.partition(),
                provisional,
            )
        });
        OriginAdmission::settle_h1_reservation(
            &admission,
            prepared.match_id,
            cell.id.partition(),
            decision,
        )
    }

    /// Attempts to extract an idle sender without intercepting a future return.
    pub(in crate::client::pool) fn try_take_idle_h1(
        cell: &Arc<Self>,
        admission: Arc<OriginAdmission>,
        prepared: PreparedH1IdleProbe,
    ) -> Option<AdmissionAction> {
        let decision = {
            let mut state = cell.state.lock();
            state.try_take_idle_h1(prepared.match_id)
        };
        let decision = decision.map_candidate(|owner| {
            let provisional = ProvisionalH1::new(cell, owner);
            H1Candidate::new(
                admission.clone(),
                prepared.match_id,
                cell.id.partition(),
                provisional,
            )
        });
        OriginAdmission::settle_h1_idle_probe(
            &admission,
            prepared.match_id,
            cell.id.partition(),
            decision,
        )
    }

    /// Returns the current admission-facing HTTP/1 supply revision.
    pub(in crate::client::pool) fn current_h1_supply_revision(
        &self,
    ) -> SupplyRevision<H1SupplyStatus> {
        self.state.lock().current_h1_supply_revision()
    }

    /// Clears an installed or resolving reservation after request cancellation.
    pub(in crate::client::pool) fn cancel_h1_reservation(
        &self,
        match_id: H1MatchId,
    ) -> SupplyRevision<H1SupplyStatus> {
        self.state.lock().cancel_h1_reservation(match_id)
    }

    /// Returns a rejected provisional sender through ordinary connection-owning cell handling.
    pub(in crate::client::pool) fn reject_h1_match(
        cell: &Arc<Self>,
        match_id: H1MatchId,
        provisional: ProvisionalH1,
    ) -> SupplyRevision<H1SupplyStatus> {
        {
            let mut state = cell.state.lock();
            state.reject_h1_match(match_id);
        }
        drop(provisional);
        let mut state = cell.state.lock();
        state.assert_consistent();
        state.current_h1_supply_revision()
    }

    /// Revalidates one match and commits its provisional sender for dispatch.
    pub(in crate::client::pool) fn commit_h1_match(
        cell: &Arc<Self>,
        match_id: H1MatchId,
        provisional: ProvisionalH1,
    ) -> Result<H1Selection, ProvisionalH1> {
        let (connection_cell, owner) = provisional.into_parts();
        let committed = cell.state.lock().commit_h1_match(match_id, &owner);
        if committed {
            Ok(H1Selection::new(cell, owner))
        } else {
            Err(ProvisionalH1::from_parts(connection_cell, owner))
        }
    }

    /// Closes a selected sender and records fairness only if close wins.
    pub(in crate::client::pool) fn reclaim_h1_candidate(
        cell: &Arc<Self>,
        match_id: H1MatchId,
        provisional: ProvisionalH1,
    ) -> Result<(SupplyRevision<H1SupplyStatus>, bool), ProvisionalH1> {
        let (connection_cell, owner) = provisional.into_parts();
        if !cell.state.lock().h1.names_peer_reservation(match_id) {
            return Err(ProvisionalH1::from_parts(connection_cell, owner));
        }

        let close_won = Self::retire_h1_owner(cell, owner, CloseReason::Reclaimed);
        let supply_update = {
            let mut state = cell.state.lock();
            let revision = state.complete_h1_match(match_id, close_won);
            state.assert_consistent();
            revision
        };
        Ok((supply_update, close_won))
    }

    /// Completes local match state after a requesting cell accepts the sender.
    pub(in crate::client::pool) fn complete_h1_match(
        &self,
        match_id: H1MatchId,
        transferred: bool,
    ) -> SupplyRevision<H1SupplyStatus> {
        self.state.lock().complete_h1_match(match_id, transferred)
    }

    /// Submits this cell's changed HTTP/1 supply when the origin is bounded.
    pub(super) fn submit_h1_supply_update(&self, revision: Option<SupplyRevision<H1SupplyStatus>>) {
        if let (Some(admission), Some(revision)) = (&self.admission, revision) {
            OriginAdmission::apply_h1_supply_revision(
                admission,
                self.id.partition(),
                self.eligibility_group.clone(),
                revision,
            );
        }
    }

    /// Returns a reusable sender to the oldest compatible waiter or idle set.
    ///
    /// Demand publication, task wakeup, and any rejected-result fallback all
    /// run after the cell lock is released. `owner` remains outside the locked
    /// scope so sender or connection drop cannot run while the cell guard is
    /// live.
    fn return_h1_owner(cell: &Arc<Self>, owner: OwnedH1Sender) {
        let connection_id = owner.id();
        let mut owner = Some(owner);
        let mut installation = None;
        let mut intercepted = None;
        let mut rejected_match = None;
        let idle_deadline = cell.idle_deadline();
        let should_retire = {
            let mut state = cell.state.lock();
            let returnable = state
                .h1
                .accepts_return(owner.as_ref().expect("HTTP/1 owner disappeared"));
            if !returnable {
                if let Some(match_id) = state.h1.take_return_match() {
                    let revision = state.complete_h1_match(match_id, false);
                    rejected_match = Some((match_id, revision));
                } else {
                    state.assert_consistent();
                }
                true
            } else if let Some(match_id) = state.h1.take_return_match() {
                if state
                    .h1
                    .reserve_for_peer(owner.as_ref().expect("HTTP/1 owner disappeared"))
                {
                    state.assert_consistent();
                    intercepted = Some((
                        match_id,
                        ProvisionalH1::new(cell, owner.take().expect("HTTP/1 owner disappeared")),
                    ));
                    false
                } else {
                    let revision = state.complete_h1_match(match_id, false);
                    rejected_match = Some((match_id, revision));
                    true
                }
            } else if state.acquisitions.has_h1_compatible_waiter()
                && state
                    .h1
                    .commit_return_to_waiter(owner.as_ref().expect("HTTP/1 owner disappeared"))
            {
                state.h1.consume_local_turn();
                state.assert_consistent();
                let mut returned = owner.take().expect("HTTP/1 owner disappeared");
                returned.mark_reused();
                let (waiter, mut install) = state.acquisitions.offer_returned_h1(
                    || AcquisitionOutcome::H1(H1Selection::new(cell, returned)),
                    &cell.eligibility_group,
                );
                state.h2.cancel_pending_waiter(
                    waiter.expect("compatible HTTP/1 waiter disappeared during offer"),
                );
                install.demand_updates =
                    state.publishable_demand_updates(std::mem::take(&mut install.demand_updates));
                installation = Some(install);
                false
            } else {
                let returned = state.h1.return_idle(
                    owner.take().expect("HTTP/1 owner disappeared"),
                    idle_deadline,
                );
                match returned {
                    Ok(()) => {
                        state.assert_consistent();
                        false
                    }
                    Err(returned) => {
                        owner = Some(returned);
                        true
                    }
                }
            }
        };

        if should_retire {
            if tracing::level_enabled!(tracing::Level::TRACE) {
                trace_h1_return(cell, connection_id, H1ReturnTrace::Rejected);
            }
            if let Some((match_id, revision)) = rejected_match {
                let admission = cell
                    .admission
                    .as_ref()
                    .expect("an H1 match requires bounded admission");
                OriginAdmission::reject_returned_h1_match(
                    admission,
                    match_id,
                    cell.id.partition(),
                    revision,
                );
            }
            Self::retire_h1_owner(
                cell,
                owner.take().expect("retired HTTP/1 owner disappeared"),
                CloseReason::ProtocolClosed,
            );
            return;
        }

        if let Some((match_id, provisional)) = intercepted {
            if tracing::level_enabled!(tracing::Level::TRACE) {
                trace_h1_return(
                    cell,
                    connection_id,
                    H1ReturnTrace::PeerReservationIntercepted,
                );
            }
            let admission = cell
                .admission
                .as_ref()
                .expect("an H1 match requires bounded admission");
            let candidate = H1Candidate::new(
                admission.clone(),
                match_id,
                cell.id.partition(),
                provisional,
            );
            let action = OriginAdmission::resolve_h1_match(admission, match_id, candidate);
            OriginAdmission::run_action_chain(action);
            return;
        }

        let Some(installation) = installation else {
            if tracing::level_enabled!(tracing::Level::TRACE) {
                trace_h1_return(cell, connection_id, H1ReturnTrace::Idle);
            }
            cell.notify_maintenance(idle_deadline);
            if cell.admission.is_some() {
                let revision = cell.state.lock().take_h1_supply_update();
                cell.submit_h1_supply_update(revision);
            }
            return;
        };
        if tracing::level_enabled!(tracing::Level::TRACE) {
            trace_h1_return(cell, connection_id, H1ReturnTrace::LocalDemand);
        }
        if let Some(admission) = &cell.admission {
            for snapshot in installation.demand_updates.into_iter().flatten() {
                OriginAdmission::submit_demand_snapshot(admission, cell.id.partition(), snapshot);
            }
        }
        drop(installation.returned_step);
        if let Some(waker) = installation.waker {
            waker.wake();
        }
        if cell.admission.is_some() {
            let revision = cell.state.lock().take_h1_supply_update();
            cell.submit_h1_supply_update(revision);
        }
    }

    /// Retires an externally owned H1 sender and removes its connection-owning cell record.
    ///
    /// Returns whether this path won the connection's logical-close race.
    fn retire_h1_owner(cell: &Arc<Self>, owner: OwnedH1Sender, reason: CloseReason) -> bool {
        let should_close = {
            let mut state = cell.state.lock();
            let should_close = state.h1.close_owned(&owner);
            state.assert_consistent();
            should_close
        };

        let id = owner.id();
        let won = should_close && owner.connection().logical_close(reason);
        drop(owner);

        let mut state = cell.state.lock();
        state.h1.remove_closed(id);
        state.assert_consistent();
        let revision = state.take_h1_supply_update();
        drop(state);
        cell.submit_h1_supply_update(revision);
        won
    }

    /// Begins close for an installed H1 record named without its sender.
    ///
    /// Returns whether this signal won the connection's logical-close race.
    pub(super) fn close_h1(cell: &Arc<Self>, id: ConnectionId, reason: CloseReason) -> bool {
        let Some((close, supply_update)) = ({
            let mut state = cell.state.lock();
            let close = state.h1.begin_close(id);
            state.assert_consistent();
            close.map(|close| (close, state.take_h1_supply_update()))
        }) else {
            return false;
        };

        cell.submit_h1_supply_update(supply_update);
        let remove_record = close.sender.is_some();
        let won = close.connection.logical_close(reason);
        drop(close.sender);
        if remove_record {
            let mut state = cell.state.lock();
            state.h1.remove_closed(id);
            state.assert_consistent();
        }
        won
    }

    /// Returns H1 record counts for focused ownership tests.
    #[cfg(test)]
    pub(super) fn h1_counts(&self) -> (usize, usize) {
        self.state.lock().h1.counts()
    }

    /// Returns the synthetic identities of every idle HTTP/1 sender.
    #[cfg(all(test, smithy_http_client_loom))]
    pub(super) fn h1_idle_sender_ids(&self) -> Vec<u64> {
        self.state.lock().h1.idle_sender_ids()
    }

    /// Returns the sole installed HTTP/1 connection for focused dispatch tests.
    #[cfg(all(test, feature = "rt-tokio"))]
    pub(in crate::client::pool) fn only_h1_connection_for_test(&self) -> Arc<ConnectionState> {
        self.state.lock().h1.only_connection_for_test()
    }
}

/// Terminal outcome for one reusable HTTP/1 sender return.
enum H1ReturnTrace {
    /// The owning cell no longer accepts the sender.
    Rejected,
    /// Origin admission reserved the sender for peer demand.
    PeerReservationIntercepted,
    /// No compatible demand exists, so the sender became idle.
    Idle,
    /// A waiter in the owning cell accepted the sender.
    LocalDemand,
}

/// Emits a committed HTTP/1 return outcome outside the cell lock.
// Keep field formatting out of a return transition that can synchronously
// enter admission and sender fallbacks.
#[inline(never)]
fn trace_h1_return(cell: &OriginCell, connection_id: ConnectionId, outcome: H1ReturnTrace) {
    match outcome {
        H1ReturnTrace::Rejected => tracing::trace!(
            connection_id = %connection_id,
            connection_partition = ?cell.id.partition(),
            origin_scheme = %cell.id.origin().scheme(),
            origin_host = cell.id.origin().host(),
            origin_port = ?cell.id.origin().port(),
            "HTTP/1 return was rejected by its connection-owning cell"
        ),
        H1ReturnTrace::PeerReservationIntercepted => tracing::trace!(
            connection_id = %connection_id,
            connection_partition = ?cell.id.partition(),
            origin_scheme = %cell.id.origin().scheme(),
            origin_host = cell.id.origin().host(),
            origin_port = ?cell.id.origin().port(),
            "HTTP/1 return was intercepted by a peer reservation"
        ),
        H1ReturnTrace::Idle => tracing::trace!(
            connection_id = %connection_id,
            connection_partition = ?cell.id.partition(),
            origin_scheme = %cell.id.origin().scheme(),
            origin_host = cell.id.origin().host(),
            origin_port = ?cell.id.origin().port(),
            "HTTP/1 connection returned to idle storage"
        ),
        H1ReturnTrace::LocalDemand => tracing::trace!(
            connection_id = %connection_id,
            request_partition = ?cell.id.partition(),
            connection_partition = ?cell.id.partition(),
            origin_scheme = %cell.id.origin().scheme(),
            origin_host = cell.id.origin().host(),
            origin_port = ?cell.id.origin().port(),
            "HTTP/1 return satisfied local demand"
        ),
    }
}

#[cfg(all(test, not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use crate::client::pool::connection::ConnectionInfo;
    use crate::client::pool::PartitionId;

    #[test]
    fn closing_a_selected_record_rejects_its_later_return() {
        let info = ConnectionInfo::for_test(ConnectionId::new(1), PartitionId::from_index(1));
        let (connection, _physical) = ConnectionState::unbounded(info);
        let mut records = H1CellState::default();
        let owner = records
            .insert_selected(connection, H1Sender::test(11))
            .expect("fresh HTTP/1 record was rejected");

        assert!(records.accepts_return(&owner));
        assert!(records.begin_close(owner.id()).is_some());
        assert!(!records.accepts_return(&owner));

        drop(owner);
        records.remove_closed(ConnectionId::new(1));
    }
    #[test]
    fn peer_reservation_distinguishes_waiting_resolution_and_completion() {
        let match_id = H1MatchId::for_test(1);
        let mut slot = H1Reservation::default();

        assert!(slot.reserve_waiting(match_id));
        assert!(!slot.reserve_waiting(H1MatchId::for_test(2)));
        assert_eq!(Some(match_id), slot.take_return_match());
        assert!(slot.complete_transfer(match_id, true));
        assert!(slot.local_turn_owed());
        assert!(slot.is_available());
    }

    #[test]
    fn rejection_does_not_manufacture_a_fairness_turn() {
        let match_id = H1MatchId::for_test(1);
        let mut slot = H1Reservation::default();

        assert!(slot.reserve_resolving(match_id));
        assert!(slot.release(match_id));
        assert!(!slot.local_turn_owed());
    }

    #[test]
    fn owed_turn_blocks_only_while_local_h1_demand_can_use_it() {
        let match_id = H1MatchId::for_test(1);
        let mut slot = H1Reservation::default();
        assert!(slot.reserve_resolving(match_id));
        assert!(slot.complete_transfer(match_id, true));

        assert!(slot.blocks_peer_selection(true));
        assert!(!slot.blocks_peer_selection(false));
        assert!(slot.clear_unused_turn(false));
        assert!(!slot.blocks_peer_selection(true));
    }
}
