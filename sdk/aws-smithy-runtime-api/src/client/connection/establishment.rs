/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::sync::Arc;
use std::time::Duration;

/// Measurements from one successful connection establishment.
///
/// The total and transport durations are available when an HTTP client exposes
/// this value. Transport substages remain optional because a custom connector
/// may expose only its aggregate connection work.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ConnectionEstablishmentMetadata {
    inner: Arc<ConnectionEstablishmentMetadataInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct ConnectionEstablishmentMetadataInner {
    total_duration: Duration,
    transport_duration: Duration,
    protocol_handshake_duration: Option<Duration>,
    dns_duration: Option<Duration>,
    socket_connect_duration: Option<Duration>,
    proxy_duration: Option<Duration>,
    tls_duration: Option<Duration>,
}

impl ConnectionEstablishmentMetadata {
    /// Creates a builder for successful establishment measurements.
    pub fn builder() -> ConnectionEstablishmentMetadataBuilder {
        ConnectionEstablishmentMetadataBuilder::new()
    }

    /// Returns elapsed time from establishment start until pool installation.
    pub fn total_duration(&self) -> Duration {
        self.inner.total_duration
    }

    /// Returns elapsed time spent in the configured transport connector.
    pub fn transport_duration(&self) -> Duration {
        self.inner.transport_duration
    }

    /// Returns elapsed time spent in the HTTP protocol handshake, when known.
    pub fn protocol_handshake_duration(&self) -> Option<Duration> {
        self.inner.protocol_handshake_duration
    }

    /// Returns elapsed time spent resolving DNS, when reported by the connector.
    pub fn dns_duration(&self) -> Option<Duration> {
        self.inner.dns_duration
    }

    /// Returns elapsed time spent establishing the selected socket, when reported.
    pub fn socket_connect_duration(&self) -> Option<Duration> {
        self.inner.socket_connect_duration
    }

    /// Returns elapsed time spent negotiating a proxy path, when reported.
    pub fn proxy_duration(&self) -> Option<Duration> {
        self.inner.proxy_duration
    }

    /// Returns elapsed time spent in TLS establishment, when reported.
    pub fn tls_duration(&self) -> Option<Duration> {
        self.inner.tls_duration
    }
}

/// Builder for [`ConnectionEstablishmentMetadata`].
#[derive(Clone, Debug, Default)]
pub struct ConnectionEstablishmentMetadataBuilder {
    total_duration: Option<Duration>,
    transport_duration: Option<Duration>,
    protocol_handshake_duration: Option<Duration>,
    dns_duration: Option<Duration>,
    socket_connect_duration: Option<Duration>,
    proxy_duration: Option<Duration>,
    tls_duration: Option<Duration>,
}

impl ConnectionEstablishmentMetadataBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets total elapsed establishment time.
    pub fn total_duration(mut self, duration: Duration) -> Self {
        self.set_total_duration(Some(duration));
        self
    }

    /// Sets total elapsed establishment time.
    pub fn set_total_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.total_duration = duration;
        self
    }

    /// Sets aggregate transport-connector time.
    pub fn transport_duration(mut self, duration: Duration) -> Self {
        self.set_transport_duration(Some(duration));
        self
    }

    /// Sets aggregate transport-connector time.
    pub fn set_transport_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.transport_duration = duration;
        self
    }

    /// Sets HTTP protocol-handshake time.
    pub fn protocol_handshake_duration(mut self, duration: Duration) -> Self {
        self.set_protocol_handshake_duration(Some(duration));
        self
    }

    /// Sets HTTP protocol-handshake time.
    pub fn set_protocol_handshake_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.protocol_handshake_duration = duration;
        self
    }

    /// Sets DNS resolution time.
    pub fn dns_duration(mut self, duration: Duration) -> Self {
        self.set_dns_duration(Some(duration));
        self
    }

    /// Sets DNS resolution time.
    pub fn set_dns_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.dns_duration = duration;
        self
    }

    /// Sets socket establishment time.
    pub fn socket_connect_duration(mut self, duration: Duration) -> Self {
        self.set_socket_connect_duration(Some(duration));
        self
    }

    /// Sets socket establishment time.
    pub fn set_socket_connect_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.socket_connect_duration = duration;
        self
    }

    /// Sets proxy negotiation time.
    pub fn proxy_duration(mut self, duration: Duration) -> Self {
        self.set_proxy_duration(Some(duration));
        self
    }

    /// Sets proxy negotiation time.
    pub fn set_proxy_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.proxy_duration = duration;
        self
    }

    /// Sets TLS establishment time.
    pub fn tls_duration(mut self, duration: Duration) -> Self {
        self.set_tls_duration(Some(duration));
        self
    }

    /// Sets TLS establishment time.
    pub fn set_tls_duration(&mut self, duration: Option<Duration>) -> &mut Self {
        self.tls_duration = duration;
        self
    }

    /// Builds successful connection-establishment metadata.
    ///
    /// # Panics
    ///
    /// Panics when total or transport duration is unset.
    pub fn build(self) -> ConnectionEstablishmentMetadata {
        ConnectionEstablishmentMetadata {
            inner: Arc::new(ConnectionEstablishmentMetadataInner {
                total_duration: self
                    .total_duration
                    .expect("total_duration is required for connection establishment metadata"),
                transport_duration: self
                    .transport_duration
                    .expect("transport_duration is required for connection establishment metadata"),
                protocol_handshake_duration: self.protocol_handshake_duration,
                dns_duration: self.dns_duration,
                socket_connect_duration: self.socket_connect_duration,
                proxy_duration: self.proxy_duration,
                tls_duration: self.tls_duration,
            }),
        }
    }
}
