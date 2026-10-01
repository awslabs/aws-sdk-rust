/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP/1 connection establishment.
//!
//! Establishment receives an already connected transport, performs Hyper's
//! client handshake, installs the resulting exclusive sender in the origin
//! cell, and submits the protocol driver to the connection-owning partition.

use super::super::cell::h1::{H1CloseHandle, H1DriverGuard, H1Selection, H1Sender};
use super::super::cell::{EstablishmentPermit, OriginCell};
use super::super::connection::{
    CloseReason, ConnectionInfo, ConnectionIo, ConnectionProtocol, ConnectionState,
};
use super::super::dispatch::AcquisitionContext;
use super::super::events::ConnectionEstablishment;
use super::{next_connection_id, ConnectedTransport};
use crate::client::downcast_error;
use aws_smithy_runtime_api::client::result::ConnectorError;
use aws_smithy_types::body::SdkBody;

/// Handshakes, installs, and starts the owner-partition driver.
///
/// `permit` remains the sole owner of optional bounded capacity until the
/// connection state is created. An earlier failure returns that capacity
/// through the permit's fallback. A handshake failure logically closes the
/// new connection state, while success transfers driver ownership to the
/// partition runtime and returns the installed sender selection.
pub(in crate::client::pool) async fn establish_h1(
    context: AcquisitionContext,
    permit: EstablishmentPermit,
    transport: ConnectedTransport,
    establishment: ConnectionEstablishment,
) -> Result<H1Selection, ConnectorError> {
    let request_partition = context.partition.id();
    let connection_cell = context.cell.clone();
    tracing::debug!(
        request_partition = ?request_partition,
        connection_partition = ?connection_cell.id().partition(),
        origin_scheme = %connection_cell.id().origin().scheme(),
        origin_host = connection_cell.id().origin().host(),
        origin_port = ?connection_cell.id().origin().port(),
        "HTTP/1 connection establishment started"
    );

    let result = run_h1_handshake(context, permit, transport, establishment).await;
    match &result {
        Ok(selection) => {
            let connection = selection.connection();
            tracing::debug!(
                connection_id = %connection.id(),
                request_partition = ?request_partition,
                connection_partition = ?connection.owner_partition(),
                origin_scheme = %connection.info().origin().scheme(),
                origin_host = connection.info().origin().host(),
                origin_port = ?connection.info().origin().port(),
                "HTTP/1 connection established"
            );
        }
        Err(error) => tracing::debug!(
            request_partition = ?request_partition,
            connection_partition = ?connection_cell.id().partition(),
            origin_scheme = %connection_cell.id().origin().scheme(),
            origin_host = connection_cell.id().origin().host(),
            origin_port = ?connection_cell.id().origin().port(),
            error = ?error,
            "HTTP/1 connection establishment failed"
        ),
    }
    result
}

/// Handshakes and installs one already connected HTTP/1 transport.
async fn run_h1_handshake(
    context: AcquisitionContext,
    permit: EstablishmentPermit,
    transport: ConnectedTransport,
    mut establishment: ConnectionEstablishment,
) -> Result<H1Selection, ConnectorError> {
    let AcquisitionContext {
        pool,
        partition,
        cell,
        owner_spawner,
        ..
    } = context;

    let id = match next_connection_id(&pool) {
        Ok(id) => id,
        Err(error) => {
            let error = ConnectorError::other(error.into(), None);
            establishment.failed(&error);
            return Err(error);
        }
    };
    let info = ConnectionInfo::new(
        id,
        cell.id().origin().clone(),
        partition.id(),
        ConnectionProtocol::Http1,
        transport.metadata,
    );
    let (connection, physical) =
        ConnectionState::pending_open(info, owner_spawner, cell.connection_stats());
    let io = ConnectionIo::new(transport.io, physical);

    establishment.protocol_handshake_started();
    let (sender, driver) = match hyper::client::conn::http1::Builder::new()
        .handshake::<_, SdkBody>(io)
        .await
    {
        Ok(established) => established,
        Err(error) => {
            let error = downcast_error(Box::new(error));
            establishment.protocol_handshake_failed();
            connection.logical_close(CloseReason::ProtocolClosed);
            establishment.failed(&error);
            return Err(error);
        }
    };
    establishment.protocol_handshake_completed();

    if let Err(lease) = connection.open(permit.into_lease()) {
        drop(lease);
        connection.logical_close(CloseReason::ProtocolClosed);
        let error = ConnectorError::io("HTTP/1 connection closed before installation".into());
        establishment.failed(&error);
        return Err(error);
    }
    establishment.installed(&connection);

    let selection =
        OriginCell::insert_selected_h1(&cell, connection.clone(), H1Sender::from_hyper(sender));
    let driver_guard = H1DriverGuard::new(H1CloseHandle::new(&cell, &connection));
    let driver_info = connection.info().clone();
    connection.owner_spawner().spawn(Box::pin(async move {
        let result = driver.with_upgrades().await;
        if let Err(error) = result {
            tracing::debug!(
                connection_id = %driver_info.id(),
                connection_partition = ?driver_info.owner_partition(),
                origin_scheme = %driver_info.origin().scheme(),
                origin_host = driver_info.origin().host(),
                origin_port = ?driver_info.origin().port(),
                error = ?error,
                "HTTP/1 connection driver failed"
            );
        }
        driver_guard.protocol_closed();
    }));
    establishment.opened(&connection);
    Ok(selection)
}
