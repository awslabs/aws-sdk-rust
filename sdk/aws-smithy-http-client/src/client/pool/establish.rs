/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Transport connection followed by HTTP protocol establishment.
//!
//! [`transport`] creates connected I/O and reports the negotiated protocol.
//! This module then routes that transport through the matching Hyper handshake,
//! creates the protocol-neutral connection identity, and returns or transfers
//! completion ownership for the launching waiter.

mod h1;
mod h2;
mod transport;

#[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
pub(super) use transport::from_cached_interface_connector;
#[cfg(any(
    all(feature = "test-util", aws_sdk_unstable),
    all(test, feature = "rt-tokio")
))]
pub(super) use transport::from_connector;
pub(super) use transport::{from_interface_connector, TransportFactory, TransportTimeout};

use self::transport::TransportConnectContext;
use super::admission::ProtocolRequirement;
use super::cell::{AcquisitionOutcome, EstablishmentPermit, WaiterId};
use super::connection::ConnectionProtocol;
use super::dispatch::AcquisitionContext;
use super::PoolInner;
use crate::client::connect::BoxConn;
use crate::client::downcast_error;
use aws_smithy_runtime_api::client::connection::ConnectionId;
use aws_smithy_runtime_api::client::result::ConnectorError;
use hyper_util::client::legacy::connect::Connected;
use std::error::Error;
use std::fmt;
use std::sync::atomic::Ordering;

/// Result of one owner-runtime establishment task.
pub(super) enum EstablishmentOutcome {
    /// The launching waiter receives this terminal result.
    Complete(AcquisitionOutcome),
    /// An H2 flight or generation now owns the launching waiter's completion.
    WaiterCompletionTransferred,
}

/// Connected transport and the connector metadata that describes it.
struct ConnectedTransport {
    io: BoxConn,
    metadata: Connected,
}

/// Connects one transport and dispatches protocol establishment after ALPN.
pub(super) async fn establish(
    context: AcquisitionContext,
    waiter: WaiterId,
    permit: EstablishmentPermit,
    requirement: ProtocolRequirement,
) -> EstablishmentOutcome {
    let mut establishment = context.pool.connection_events.establishment_started(
        context.cell.id().origin(),
        context.partition.id(),
        context.cell.connection_stats(),
    );
    let connect = TransportConnectContext::new(
        &context.partition,
        context.absolute_uri.clone(),
        context.connect_timeout.clone(),
        requirement,
    );
    let io = match context.pool.transport.connect(connect).await {
        Ok(io) => io,
        Err(error) => {
            let error = downcast_error(error);
            tracing::debug!(
                request_partition = ?context.partition.id(),
                connection_partition = ?context.cell.id().partition(),
                origin_scheme = %context.cell.id().origin().scheme(),
                origin_host = context.cell.id().origin().host(),
                origin_port = ?context.cell.id().origin().port(),
                error = ?error,
                "transport establishment failed"
            );
            establishment.failed(&error);
            return EstablishmentOutcome::Complete(AcquisitionOutcome::Failed(error));
        }
    };
    let transport = ConnectedTransport {
        metadata: io.connected(),
        io,
    };
    establishment.transport_completed(connector_remote_addr(&transport.metadata));
    let negotiated_h2 = transport.metadata.is_negotiated_h2();
    let protocol = if negotiated_h2 {
        ConnectionProtocol::Http2
    } else {
        ConnectionProtocol::Http1
    };
    establishment.protocol_selected(protocol);
    tracing::debug!(
        request_partition = ?context.partition.id(),
        connection_partition = ?context.cell.id().partition(),
        origin_scheme = %context.cell.id().origin().scheme(),
        origin_host = context.cell.id().origin().host(),
        origin_port = ?context.cell.id().origin().port(),
        negotiated_protocol = if negotiated_h2 { "HTTP/2" } else { "HTTP/1.1" },
        "transport protocol negotiated"
    );
    if negotiated_h2 && !requirement.accepts_h2() {
        drop(transport);
        drop(permit);
        let error = negotiated_protocol_mismatch(requirement);
        establishment.failed(&error);
        return EstablishmentOutcome::Complete(AcquisitionOutcome::Failed(error));
    }

    if negotiated_h2 {
        h2::establish_h2(context, permit, transport, establishment, waiter).await
    } else {
        EstablishmentOutcome::Complete(
            h1::establish_h1(context, permit, transport, establishment)
                .await
                .map(AcquisitionOutcome::H1)
                .unwrap_or_else(AcquisitionOutcome::Failed),
        )
    }
}
/// An established transport selected a protocol incompatible with the request.
#[derive(Debug)]
struct NegotiatedProtocolMismatch {
    requirement: ProtocolRequirement,
}

impl fmt::Display for NegotiatedProtocolMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "transport negotiated HTTP/2, which does not satisfy {:?} request semantics",
            self.requirement
        )
    }
}

impl Error for NegotiatedProtocolMismatch {}

fn negotiated_protocol_mismatch(requirement: ProtocolRequirement) -> ConnectorError {
    ConnectorError::other(NegotiatedProtocolMismatch { requirement }.into(), None)
}

/// Returns the connector-reported peer address after transport establishment.
fn connector_remote_addr(
    connected: &hyper_util::client::legacy::connect::Connected,
) -> Option<std::net::SocketAddr> {
    let mut extras = http_1x::Extensions::new();
    connected.get_extras(&mut extras);
    extras
        .get::<hyper_util::client::legacy::connect::HttpInfo>()
        .map(hyper_util::client::legacy::connect::HttpInfo::remote_addr)
}

/// Mints one non-wrapping physical-connection identity.
fn next_connection_id(pool: &PoolInner) -> Result<ConnectionId, ConnectionIdExhausted> {
    let value = pool
        .next_connection_id
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| ConnectionIdExhausted)?;
    Ok(ConnectionId::new(value))
}

/// The pool exhausted its monotonic physical-connection identity space.
#[derive(Debug)]
struct ConnectionIdExhausted;

impl fmt::Display for ConnectionIdExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("connection identifier space exhausted")
    }
}

impl Error for ConnectionIdExhausted {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negotiated_protocol_mismatch_is_not_a_user_error() {
        let error = negotiated_protocol_mismatch(ProtocolRequirement::H1Required);

        assert!(error.is_other());
        assert!(!error.is_user());
    }
}
