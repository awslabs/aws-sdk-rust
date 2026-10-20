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
use crate::client::{downcast_error, error_chain};
use aws_smithy_runtime_api::box_error::BoxError;
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
            let error = transport_failure(error);
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

/// Classifies a failure to establish a connection's transport: DNS, the TCP connect, the TLS
/// handshake, or a proxy tunnel. Nothing has been sent, so the only question is whether a
/// new connection could succeed.
///
/// - A connect timeout, a DNS failure, and a `ConnectorError` returned by the transport keep
///   `downcast_error`'s classification.
/// - A failure in the network is an I/O error, so it is retried.
/// - Anything else is terminal: a TLS refusal, a proxy's refusal, or a configuration error
///   such as an unsupported scheme.
fn transport_failure(error: BoxError) -> ConnectorError {
    let classified_by_transport = error.is::<ConnectorError>();
    let error = downcast_error(error);
    if classified_by_transport || !error.is_other() {
        return error;
    }
    if failed_in_network(&error) {
        ConnectorError::io(error.into_source())
    } else {
        error
    }
}

/// Whether a transport failure happened in the network.
///
/// Walks [`error_chain`], which also visits each `io::Error`'s payload. The first
/// `TlsConnectError` decides, by its kind; before that, any `io::Error` that
/// [`retryable_io`] accepts is a network failure.
fn failed_in_network(error: &(dyn Error + 'static)) -> bool {
    for error in error_chain(error) {
        #[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
        if let Some(tls) = error.downcast_ref::<crate::tls::TlsConnectError>() {
            return tls.kind() == crate::tls::TlsConnectErrorKind::Io;
        }
        if let Some(io) = error.downcast_ref::<std::io::Error>() {
            if retryable_io(io) {
                return true;
            }
        }
    }
    false
}

/// Whether an I/O error is one a new connection could avoid.
///
/// - Retried with or without an operating-system errno, because connectors and resolvers
///   also synthesize these from a kind alone: a refused, reset, aborted, or unconnected
///   connection; a closed connection (`BrokenPipe`, `UnexpectedEof`, or `WriteZero`, a write
///   that sent no bytes); an unavailable local address; an unreachable network or host, or a
///   network that is down; and a timeout.
/// - Never retried: `PermissionDenied`, `InvalidInput`, and `Unsupported`.
/// - Any other kind is retried only as an operating-system error. That includes ENOBUFS,
///   which has no stable kind.
fn retryable_io(error: &std::io::Error) -> bool {
    use std::io::ErrorKind::*;
    match error.kind() {
        ConnectionRefused | ConnectionReset | ConnectionAborted | NotConnected
        | AddrNotAvailable | NetworkUnreachable | HostUnreachable | NetworkDown | BrokenPipe
        | TimedOut | UnexpectedEof | WriteZero => true,
        PermissionDenied | InvalidInput | Unsupported => false,
        _ => error.raw_os_error().is_some(),
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
    use aws_smithy_async::future::timeout::TimedOutError;
    use aws_smithy_runtime_api::client::dns::ResolveDnsError;
    use std::io;

    #[test]
    fn negotiated_protocol_mismatch_is_not_a_user_error() {
        let error = negotiated_protocol_mismatch(ProtocolRequirement::H1Required);

        assert!(error.is_other());
        assert!(!error.is_user());
    }

    /// A transport error that names a stage and carries its cause as `source()`, the shape of
    /// hyper-util's unexported `ConnectError`.
    #[derive(Debug)]
    struct StageError {
        stage: &'static str,
        cause: BoxError,
    }

    impl fmt::Display for StageError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.stage)
        }
    }

    impl Error for StageError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&*self.cause)
        }
    }

    fn stage_error(stage: &'static str, cause: impl Into<BoxError>) -> BoxError {
        Box::new(StageError {
            stage,
            cause: cause.into(),
        })
    }

    #[track_caller]
    fn assert_io(error: BoxError) -> ConnectorError {
        let error = transport_failure(error);
        assert!(error.is_io(), "expected an I/O error, got {error:?}");
        error
    }

    #[track_caller]
    fn assert_terminal(error: BoxError) {
        let error = transport_failure(error);
        assert!(
            error.is_other() && error.as_other().is_none(),
            "expected Other(None), got {error:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn os_connect_errors_are_io() {
        for errno in [libc::ECONNREFUSED, libc::ENOBUFS, libc::EADDRNOTAVAIL] {
            assert_io(stage_error(
                "tcp connect error",
                io::Error::from_raw_os_error(errno),
            ));
        }
    }

    #[test]
    fn reclassified_io_error_keeps_its_source_chain() {
        let error = assert_io(stage_error(
            "tcp connect error",
            io::Error::from(io::ErrorKind::ConnectionReset),
        ));

        let source = error.into_source();
        let stage = source
            .downcast_ref::<StageError>()
            .expect("the original error is the source");
        assert_eq!("tcp connect error", stage.stage);
    }

    #[test]
    fn synthesized_network_kind_is_io() {
        // A connector may build an `io::Error` from a kind alone, with no errno.
        assert_io(stage_error(
            "tcp connect error",
            io::Error::from(io::ErrorKind::ConnectionRefused),
        ));
    }

    #[test]
    fn handshake_eof_is_io() {
        // hyper-rustls wraps tokio-rustls's handshake EOF in `io::Error::other`.
        assert_io(Box::new(io::Error::other(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "tls handshake eof",
        ))));
    }

    #[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
    #[test]
    fn tls_io_failure_is_io() {
        use crate::tls::{TlsConnectError, TlsConnectErrorKind};

        // The source alone would be terminal; the shim's classification decides.
        assert_io(Box::new(TlsConnectError::new(
            TlsConnectErrorKind::Io,
            "underlying I/O operation failed",
        )));
    }

    #[test]
    fn dns_failure_is_io() {
        let dns = ResolveDnsError::new(io::Error::other("failed to lookup address information"));

        assert_io(Box::new(dns));
    }

    #[test]
    fn wrapped_dns_failure_is_io() {
        let dns = ResolveDnsError::new(io::Error::other("failed to lookup address information"));

        assert_io(stage_error("dns error", dns));
    }

    #[cfg(unix)]
    #[test]
    fn os_errors_a_new_connection_cannot_fix_are_terminal() {
        for errno in [libc::EACCES, libc::EINVAL, libc::ENOSYS] {
            assert_terminal(stage_error(
                "tcp connect error",
                io::Error::from_raw_os_error(errno),
            ));
        }
    }

    #[cfg(any(feature = "__rustls", feature = "s2n-tls"))]
    #[test]
    fn tls_protocol_failure_is_terminal() {
        use crate::tls::{TlsConnectError, TlsConnectErrorKind};

        // The source alone would be retried; the shim's classification decides.
        assert_terminal(Box::new(TlsConnectError::new(
            TlsConnectErrorKind::Protocol,
            io::Error::new(io::ErrorKind::UnexpectedEof, "alert received"),
        )));
    }

    #[test]
    fn unsupported_scheme_is_terminal() {
        assert_terminal(Box::new(io::Error::other("unsupported scheme")));
    }

    #[test]
    fn plain_string_error_is_terminal() {
        assert_terminal("missing host in URI for TLS handshake".into());
    }

    #[test]
    fn connect_timeout_stays_timeout() {
        let error = transport_failure(stage_error("connect timeout", TimedOutError));

        assert!(error.is_timeout(), "expected a timeout, got {error:?}");
    }

    #[test]
    fn transport_connector_error_keeps_its_kind() {
        let user = transport_failure(Box::new(ConnectorError::user("invalid request".into())));
        assert!(user.is_user(), "expected a user error, got {user:?}");

        // The transport's `Other(None)` stands even over a source that would be retried.
        let other = transport_failure(Box::new(ConnectorError::other(
            Box::new(io::Error::from(io::ErrorKind::ConnectionReset)),
            None,
        )));
        assert!(
            other.is_other() && other.as_other().is_none(),
            "expected Other(None), got {other:?}"
        );
    }

    #[test]
    fn retryable_io_kinds() {
        use io::ErrorKind::*;

        // Network kinds are retried without an errno.
        for kind in [
            ConnectionRefused,
            ConnectionReset,
            ConnectionAborted,
            NotConnected,
            AddrNotAvailable,
            NetworkUnreachable,
            HostUnreachable,
            NetworkDown,
            BrokenPipe,
            TimedOut,
            UnexpectedEof,
            WriteZero,
        ] {
            assert!(retryable_io(&io::Error::from(kind)), "{kind:?}");
        }
        for kind in [PermissionDenied, InvalidInput, Unsupported] {
            assert!(!retryable_io(&io::Error::from(kind)), "{kind:?}");
        }
        // Other kinds are retried only as operating-system errors.
        for kind in [InvalidData, Other] {
            assert!(!retryable_io(&io::Error::from(kind)), "{kind:?}");
        }
        assert!(!retryable_io(&io::Error::other("unsupported scheme")));
    }
}
