/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/2 request authority and two-sided completion.
//!
//! A cell creates [`H2Activation`] only after reserving one prospective request
//! against an exact generation. The activation owns the transient Hyper sender
//! and the right to convert that prospective count into an accepted request.
//! Dropping it before acceptance cancels the prospective count.
//!
//! Hyper acceptance creates one [`H2RequestClaim`]. The request upload and
//! response lifetime hold distinct guard types so transposing the two sides is
//! a type error. Either side may finish first. The claim releases the accepted
//! generation count only after both sides finish, including drop and error
//! paths.
//!
//! The claim mutex is never held while locking cell or connection state.

use super::super::super::connection::{ConnectionInfo, ConnectionState, DispatchGuard};
use super::super::super::partition::PartitionId;
use super::super::{OriginCell, WaiterId};
use super::{H2CloseHandle, H2GenerationId, H2RouteId, H2Sender};
use crate::sync::{Arc, Mutex, Weak};

/// Sender and connection cloned while a prospective request is cell-owned.
pub(super) struct H2ActivationResources {
    /// Transient sender clone for one activation.
    pub(super) sender: H2Sender,
    /// Whether the generation accepted an earlier request.
    pub(super) reused: bool,
    /// Connection retained through prospective dispatch.
    pub(super) connection: Arc<ConnectionState>,
}

/// Resources retained before and after dispatch transfer.
enum H2ActivationState {
    /// Sender and connection still owned by the activation.
    Ready(H2ActivationResources),
    /// Immutable identity retained after the sender leaves the activation.
    Dispatched(H2RequestIdentity),
}

/// Values transferred together when an activation begins Hyper dispatch.
pub(in crate::client::pool) struct H2DispatchParts {
    /// Transient sender clone for one dispatch attempt.
    pub(in crate::client::pool) sender: H2Sender,
    /// Upload-completion guard retained by the request body.
    pub(in crate::client::pool) upload: H2UploadGuard,
    /// Response-lifetime guard retained by the response future or body.
    pub(in crate::client::pool) response: H2ResponseGuard,
}

/// Prospective request authority for one exact generation.
///
/// The activation leaves the cell lock with one counted prospective request.
/// Taking its dispatch parts transfers the sender and creates the two request
/// guards. Hyper acceptance converts the prospective count to an accepted
/// claim; rejection or drop cancels it.
pub(in crate::client::pool) struct H2Activation {
    /// Cell that owns the selected generation.
    connection_cell: Arc<OriginCell>,
    /// Exact generation that owns the prospective count.
    generation: H2GenerationId,
    /// Partition issuing this request.
    request_partition: PartitionId,
    /// Mutually exclusive pre-dispatch resources or post-transfer identity.
    state: Option<H2ActivationState>,
    /// Shared accounting for the upload and response sides.
    claim: Arc<H2RequestClaim>,
    /// Requesting-cell priority released at acceptance or cancellation.
    turn: Option<H2ActivationTurnGuard>,
    /// Whether drop must cancel the prospective generation count.
    active: bool,
}

impl std::fmt::Debug for H2Activation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("H2Activation")
            .field("generation", &self.generation)
            .field(
                "connection_id",
                &self.state.as_ref().map(|state| match state {
                    H2ActivationState::Ready(resources) => resources.connection.id(),
                    H2ActivationState::Dispatched(identity) => identity.connection.id(),
                }),
            )
            .field("active", &self.active)
            .finish()
    }
}

impl H2Activation {
    /// Builds an activation after the generation records its prospective claim.
    pub(super) fn new(
        connection_cell: Arc<OriginCell>,
        generation: H2GenerationId,
        resources: H2ActivationResources,
        request_partition: PartitionId,
        turn: Option<H2ActivationTurnGuard>,
    ) -> Self {
        let claim = Arc::new(H2RequestClaim {
            connection_cell: Weak::from_arc(&connection_cell),
            generation,
            state: Mutex::new(H2RequestClaimState::Prospective {
                upload_finished: false,
                response_finished: false,
            }),
        });
        Self {
            connection_cell,
            generation,
            request_partition,
            state: Some(H2ActivationState::Ready(resources)),
            claim,
            turn,
            active: true,
        }
    }

    /// Returns the exact selected generation.
    #[cfg(test)]
    pub(in crate::client::pool) fn generation(&self) -> H2GenerationId {
        self.generation
    }

    /// Returns whether Hyper previously accepted a request on this generation.
    ///
    /// # Panics
    ///
    /// Panics if dispatch parts were already taken.
    pub(in crate::client::pool) fn is_reused(&self) -> bool {
        match self
            .state
            .as_ref()
            .expect("HTTP/2 activation state missing")
        {
            H2ActivationState::Ready(resources) => resources.reused,
            H2ActivationState::Dispatched(_) => {
                panic!("HTTP/2 activation resources already taken")
            }
        }
    }

    /// Returns the selected protocol-neutral connection.
    ///
    /// # Panics
    ///
    /// Panics if dispatch parts were already taken.
    pub(in crate::client::pool) fn connection(&self) -> &Arc<ConnectionState> {
        match self
            .state
            .as_ref()
            .expect("HTTP/2 activation state missing")
        {
            H2ActivationState::Ready(resources) => &resources.connection,
            H2ActivationState::Dispatched(_) => {
                panic!("HTTP/2 activation resources already taken")
            }
        }
    }

    /// Transfers the sender and typed request guards into one dispatch attempt.
    ///
    /// # Panics
    ///
    /// Panics if dispatch parts were already taken.
    pub(in crate::client::pool) fn take_dispatch_parts(&mut self) -> H2DispatchParts {
        let state = self.state.take().expect("HTTP/2 activation state missing");
        let H2ActivationState::Ready(resources) = state else {
            self.state = Some(state);
            panic!("HTTP/2 activation dispatch parts already taken");
        };
        let identity = H2RequestIdentity {
            request_partition: self.request_partition,
            connection: resources.connection.info().clone(),
            generation: self.generation,
        };
        self.state = Some(H2ActivationState::Dispatched(identity.clone()));
        H2DispatchParts {
            sender: resources.sender,
            upload: H2UploadGuard::new(self.claim.clone(), Some(identity.clone())),
            response: H2ResponseGuard::new(self.claim.clone(), Some(identity)),
        }
    }

    /// Retains a requesting-cell route turn until acceptance or cancellation.
    pub(super) fn attach_peer_turn(
        &mut self,
        requesting_cell: &Arc<OriginCell>,
        route: H2RouteId,
        waiter: WaiterId,
    ) {
        debug_assert_eq!(
            self.request_partition,
            requesting_cell.id().partition(),
            "peer HTTP/2 activation changed requesting partition"
        );
        assert!(
            self.turn.is_none(),
            "HTTP/2 activation acquired two requesting-cell turns"
        );
        self.turn = Some(H2ActivationTurnGuard::peer(requesting_cell, route, waiter));
    }

    /// Returns close authority for the connection-owning generation.
    pub(in crate::client::pool) fn close_handle(&self) -> H2CloseHandle {
        H2CloseHandle::new(&self.connection_cell, self.generation)
    }

    /// Converts the prospective reservation after Hyper accepts the request.
    ///
    /// # Panics
    ///
    /// Panics if dispatch parts were not transferred or the prospective
    /// reservation disappeared before acceptance.
    pub(in crate::client::pool) fn accept(mut self, dispatch: DispatchGuard) {
        let Some(H2ActivationState::Dispatched(identity)) = self.state.as_ref() else {
            panic!("HTTP/2 activation accepted before dispatch parts were taken");
        };
        assert!(
            OriginCell::accept_h2_activation(&self.connection_cell, self.generation),
            "prospective HTTP/2 activation disappeared before acceptance"
        );
        self.active = false;
        self.claim.accept(dispatch);
        if let Some(turn) = self.turn.take() {
            turn.release();
        }
        tracing::trace!(
            connection_id = %identity.connection.id(),
            request_partition = ?identity.request_partition,
            connection_partition = ?identity.connection.owner_partition(),
            origin_scheme = %identity.connection.origin().scheme(),
            origin_host = identity.connection.origin().host(),
            origin_port = ?identity.connection.origin().port(),
            h2_generation = ?identity.generation,
            "HTTP/2 request accepted"
        );
    }
}

impl Drop for H2Activation {
    fn drop(&mut self) {
        if self.active {
            OriginCell::cancel_h2_activation(&self.connection_cell, self.generation);
            self.claim.cancel();
            if let Some(turn) = self.turn.take() {
                turn.release();
            }
            match &self.state {
                Some(H2ActivationState::Ready(resources)) => trace_activation_cancelled(
                    self.request_partition,
                    self.generation,
                    resources.connection.info(),
                ),
                Some(H2ActivationState::Dispatched(identity)) => trace_activation_cancelled(
                    identity.request_partition,
                    identity.generation,
                    &identity.connection,
                ),
                None => {}
            }
        }
    }
}

/// One activation-gate turn retained until Hyper acceptance or cancellation.
pub(super) struct H2ActivationTurnGuard {
    /// Requesting cell whose gate owns this turn.
    cell: Weak<OriginCell>,
    /// Waiter that received the activation opportunity.
    waiter: WaiterId,
    /// Local-generation or peer-route gate identity.
    owner: H2ActivationGateOwner,
}

/// Cell transition run when an activation accepts or cancels.
enum H2ActivationGateOwner {
    /// Gate attached to the connection cell's local generation.
    Local { generation: H2GenerationId },
    /// Gate attached to a requesting cell's peer route.
    Peer { route: H2RouteId },
}

impl H2ActivationTurnGuard {
    pub(super) fn local(
        cell: &Arc<OriginCell>,
        generation: H2GenerationId,
        waiter: WaiterId,
    ) -> Self {
        Self {
            cell: Weak::from_arc(cell),
            waiter,
            owner: H2ActivationGateOwner::Local { generation },
        }
    }

    fn peer(cell: &Arc<OriginCell>, route: H2RouteId, waiter: WaiterId) -> Self {
        Self {
            cell: Weak::from_arc(cell),
            waiter,
            owner: H2ActivationGateOwner::Peer { route },
        }
    }

    fn release(self) {
        if let Some(cell) = self.cell.upgrade() {
            match self.owner {
                H2ActivationGateOwner::Local { generation } => {
                    OriginCell::release_local_h2_turn(&cell, generation, self.waiter)
                }
                H2ActivationGateOwner::Peer { route } => {
                    OriginCell::release_peer_h2_turn(&cell, route, self.waiter)
                }
            }
        }
    }
}

/// Shared accounting for one prospective or accepted HTTP/2 request.
struct H2RequestClaim {
    /// Connection cell updated after both accepted sides finish.
    connection_cell: Weak<OriginCell>,
    /// Exact generation whose request count this claim owns.
    generation: H2GenerationId,
    /// Request phase; never held while locking cell or connection state.
    state: Mutex<H2RequestClaimState>,
}

/// Shared request phase and independent completion bits.
enum H2RequestClaimState {
    /// Generation count reserved before Hyper accepts the request.
    Prospective {
        upload_finished: bool,
        response_finished: bool,
    },
    /// Hyper accepted the request and connection dispatch is retained.
    Accepted {
        dispatch: DispatchGuard,
        upload_finished: bool,
        response_finished: bool,
    },
    /// Both sides finished or prospective dispatch was cancelled.
    Complete,
}

impl H2RequestClaim {
    /// Converts prospective request state to accepted state.
    fn accept(&self, dispatch: DispatchGuard) {
        let completed_dispatch = {
            let mut state = self.state.lock();
            let previous = std::mem::replace(&mut *state, H2RequestClaimState::Complete);
            let (upload_finished, response_finished) = match previous {
                H2RequestClaimState::Prospective {
                    upload_finished,
                    response_finished,
                } => (upload_finished, response_finished),
                other @ (H2RequestClaimState::Accepted { .. } | H2RequestClaimState::Complete) => {
                    *state = other;
                    drop(state);
                    panic!("HTTP/2 request claim accepted outside prospective state");
                }
            };
            if upload_finished && response_finished {
                Some(dispatch)
            } else {
                *state = H2RequestClaimState::Accepted {
                    dispatch,
                    upload_finished,
                    response_finished,
                };
                None
            }
        };
        if let Some(dispatch) = completed_dispatch {
            // Dispatch completion takes the connection lifecycle lock. Keep it
            // outside the claim lock so completion cannot nest pool locks.
            drop(dispatch);
            self.release_generation();
        }
    }

    /// Cancels request state after the prospective reservation ends.
    fn cancel(&self) {
        let mut state = self.state.lock();
        if matches!(*state, H2RequestClaimState::Prospective { .. }) {
            *state = H2RequestClaimState::Complete;
        }
    }

    fn finish_upload(&self) -> bool {
        self.finish_side(|upload_finished, _| *upload_finished = true)
    }

    fn finish_response(&self) -> bool {
        self.finish_side(|_, response_finished| *response_finished = true)
    }

    /// Marks one request side finished and releases the claim on the second.
    fn finish_side(&self, finish: impl FnOnce(&mut bool, &mut bool)) -> bool {
        let completed_dispatch = {
            let mut state = self.state.lock();
            let previous = std::mem::replace(&mut *state, H2RequestClaimState::Complete);
            match previous {
                H2RequestClaimState::Prospective {
                    mut upload_finished,
                    mut response_finished,
                } => {
                    finish(&mut upload_finished, &mut response_finished);
                    *state = H2RequestClaimState::Prospective {
                        upload_finished,
                        response_finished,
                    };
                    None
                }
                H2RequestClaimState::Accepted {
                    dispatch,
                    mut upload_finished,
                    mut response_finished,
                } => {
                    finish(&mut upload_finished, &mut response_finished);
                    if upload_finished && response_finished {
                        Some(dispatch)
                    } else {
                        *state = H2RequestClaimState::Accepted {
                            dispatch,
                            upload_finished,
                            response_finished,
                        };
                        None
                    }
                }
                H2RequestClaimState::Complete => None,
            }
        };
        let request_complete = completed_dispatch.is_some();
        if let Some(dispatch) = completed_dispatch {
            // Dispatch completion takes the connection lifecycle lock. Keep it
            // outside the claim lock so completion cannot nest pool locks.
            drop(dispatch);
            self.release_generation();
        }
        request_complete
    }

    /// Releases the generation count after the claim lock is released.
    fn release_generation(&self) {
        if let Some(cell) = self.connection_cell.upgrade() {
            OriginCell::release_h2_request(&cell, self.generation);
        }
    }

    #[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
    fn for_test(cell: &Arc<OriginCell>) -> Arc<Self> {
        Arc::new(Self {
            connection_cell: Weak::from_arc(cell),
            generation: H2GenerationId(0),
            state: Mutex::new(H2RequestClaimState::Prospective {
                upload_finished: false,
                response_finished: false,
            }),
        })
    }
}

/// Request and connection identity retained for completion logs.
#[derive(Clone)]
struct H2RequestIdentity {
    request_partition: PartitionId,
    connection: Arc<ConnectionInfo>,
    generation: H2GenerationId,
}

fn trace_activation_cancelled(
    request_partition: PartitionId,
    generation: H2GenerationId,
    connection: &ConnectionInfo,
) {
    tracing::trace!(
        connection_id = %connection.id(),
        request_partition = ?request_partition,
        connection_partition = ?connection.owner_partition(),
        origin_scheme = %connection.origin().scheme(),
        origin_host = connection.origin().host(),
        origin_port = ?connection.origin().port(),
        h2_generation = ?generation,
        "HTTP/2 activation cancelled before request acceptance"
    );
}

/// Linear guard for request-upload completion.
pub(in crate::client::pool) struct H2UploadGuard {
    claim: Arc<H2RequestClaim>,
    identity: Option<H2RequestIdentity>,
    active: bool,
}

impl H2UploadGuard {
    fn new(claim: Arc<H2RequestClaim>, identity: Option<H2RequestIdentity>) -> Self {
        Self {
            claim,
            identity,
            active: true,
        }
    }

    #[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
    pub(in crate::client::pool) fn for_test(cell: &Arc<OriginCell>) -> (Self, H2RequestClaimProbe) {
        let claim = H2RequestClaim::for_test(cell);
        (
            Self::new(claim.clone(), None),
            H2RequestClaimProbe { claim },
        )
    }

    /// Finishes upload ownership before dropping the guard.
    pub(in crate::client::pool) fn finish(mut self) {
        self.finish_once();
    }

    fn finish_once(&mut self) {
        if self.active {
            self.active = false;
            let request_complete = self.claim.finish_upload();
            trace_request_side(self.identity.take(), "upload", request_complete);
        }
    }
}

impl Drop for H2UploadGuard {
    fn drop(&mut self) {
        self.finish_once();
    }
}

/// Linear guard for response-future and response-body completion.
pub(in crate::client::pool) struct H2ResponseGuard {
    claim: Arc<H2RequestClaim>,
    identity: Option<H2RequestIdentity>,
    active: bool,
}

impl H2ResponseGuard {
    fn new(claim: Arc<H2RequestClaim>, identity: Option<H2RequestIdentity>) -> Self {
        Self {
            claim,
            identity,
            active: true,
        }
    }

    #[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
    pub(in crate::client::pool) fn for_test(cell: &Arc<OriginCell>) -> (Self, H2RequestClaimProbe) {
        let claim = H2RequestClaim::for_test(cell);
        (
            Self::new(claim.clone(), None),
            H2RequestClaimProbe { claim },
        )
    }

    /// Finishes response ownership before dropping the guard.
    pub(in crate::client::pool) fn finish(mut self) {
        self.finish_once();
    }

    fn finish_once(&mut self) {
        if self.active {
            self.active = false;
            let request_complete = self.claim.finish_response();
            trace_request_side(self.identity.take(), "response", request_complete);
        }
    }
}

impl Drop for H2ResponseGuard {
    fn drop(&mut self) {
        self.finish_once();
    }
}

fn trace_request_side(
    identity: Option<H2RequestIdentity>,
    request_side: &'static str,
    request_complete: bool,
) {
    if let Some(identity) = identity {
        tracing::trace!(
            connection_id = %identity.connection.id(),
            request_partition = ?identity.request_partition,
            connection_partition = ?identity.connection.owner_partition(),
            origin_scheme = %identity.connection.origin().scheme(),
            origin_host = identity.connection.origin().host(),
            origin_port = ?identity.connection.origin().port(),
            h2_generation = ?identity.generation,
            request_side,
            request_complete,
            "HTTP/2 request side finished"
        );
    }
}

/// Test observation of one request claim's production state.
#[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
pub(in crate::client::pool) struct H2RequestClaimProbe {
    claim: Arc<H2RequestClaim>,
}

#[cfg(all(test, not(smithy_http_client_loom), feature = "rt-tokio"))]
impl H2RequestClaimProbe {
    /// Returns whether the upload side reached its terminal transition.
    pub(in crate::client::pool) fn upload_finished(&self) -> bool {
        matches!(
            &*self.claim.state.lock(),
            H2RequestClaimState::Prospective {
                upload_finished: true,
                ..
            } | H2RequestClaimState::Accepted {
                upload_finished: true,
                ..
            } | H2RequestClaimState::Complete
        )
    }

    /// Returns whether the response side reached its terminal transition.
    pub(in crate::client::pool) fn response_finished(&self) -> bool {
        matches!(
            &*self.claim.state.lock(),
            H2RequestClaimState::Prospective {
                response_finished: true,
                ..
            } | H2RequestClaimState::Accepted {
                response_finished: true,
                ..
            } | H2RequestClaimState::Complete
        )
    }
}
