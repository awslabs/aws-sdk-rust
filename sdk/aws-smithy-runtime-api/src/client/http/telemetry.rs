/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Provider-neutral observations from one HTTP request attempt.
//!
//! A runtime installs [`CaptureHttpAttemptTelemetry`] on a request before
//! transmission. A compatible HTTP client records the facts it owns without
//! depending on metric instruments, exporters, or tracing providers.

use crate::client::connection::ConnectionMetadata;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, SystemTime};

/// Whether the selected connection had accepted an earlier request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ConnectionUsage {
    /// The connection had not accepted an earlier request.
    ///
    /// For a multiplexed connection, only the first accepted request is fresh.
    /// Concurrent requests accepted afterward observe reuse.
    Fresh,
    /// The connection had accepted at least one earlier request.
    Reused,
}

/// Timing and reuse state for the connection selected by one request attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ConnectionAcquisitionTelemetry {
    duration: Option<Duration>,
    usage: ConnectionUsage,
}

impl ConnectionAcquisitionTelemetry {
    /// Creates a completed connection-acquisition observation.
    pub fn new(duration: Duration, usage: ConnectionUsage) -> Self {
        Self {
            duration: Some(duration),
            usage,
        }
    }

    /// Creates an observation from acquisition start and completion times.
    ///
    /// The duration is absent when `completed_at` precedes `started_at`. The
    /// selected connection and its reuse state remain valid observations.
    pub fn from_interval(
        started_at: SystemTime,
        completed_at: SystemTime,
        usage: ConnectionUsage,
    ) -> Self {
        Self {
            duration: completed_at.duration_since(started_at).ok(),
            usage,
        }
    }

    /// Returns elapsed time until the selected connection accepted the request.
    ///
    /// This is absent when the HTTP client's clock did not produce a valid
    /// interval.
    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Returns whether the selected connection had accepted an earlier request.
    pub fn usage(&self) -> ConnectionUsage {
        self.usage
    }
}

/// Facts recorded by a compatible HTTP client for one request attempt.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct HttpAttemptTelemetry {
    selection: Option<ConnectionSelection>,
    connector_call_duration: Option<Duration>,
}

impl HttpAttemptTelemetry {
    /// Returns completed connection-acquisition telemetry, when supplied.
    pub fn acquisition(&self) -> Option<&ConnectionAcquisitionTelemetry> {
        self.selection
            .as_ref()
            .map(|selection| &selection.acquisition)
    }

    /// Returns metadata for the connection that accepted the request, when supplied.
    pub fn connection(&self) -> Option<&ConnectionMetadata> {
        self.selection
            .as_ref()
            .map(|selection| &selection.connection)
    }

    /// Returns the complete HTTP connector call duration, when supplied.
    ///
    /// This ends when the connector returns a response head or terminal error.
    /// It does not measure response-body transfer or time to first response byte.
    pub fn connector_call_duration(&self) -> Option<Duration> {
        self.connector_call_duration
    }
}

#[derive(Clone, Debug)]
struct ConnectionSelection {
    acquisition: ConnectionAcquisitionTelemetry,
    connection: ConnectionMetadata,
}

/// Shared request extension used to capture HTTP-attempt telemetry.
///
/// Each observation is recorded at most once. Acquisition timing and selected
/// connection metadata are committed together.
#[derive(Clone, Default)]
pub struct CaptureHttpAttemptTelemetry {
    state: Arc<Mutex<HttpAttemptTelemetry>>,
}

impl CaptureHttpAttemptTelemetry {
    /// Creates an empty attempt capture.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the observations recorded so far.
    pub fn get(&self) -> HttpAttemptTelemetry {
        self.lock().clone()
    }

    /// Records the complete HTTP connector call duration.
    ///
    /// Returns `true` when this call recorded the value.
    pub fn record_connector_call_duration(&self, duration: Duration) -> bool {
        let mut state = self.lock();
        if state.connector_call_duration.is_some() {
            return false;
        }
        state.connector_call_duration = Some(duration);
        true
    }

    /// Records the connection selection that accepted the request.
    ///
    /// Acquisition and connection metadata are committed together. Returns
    /// `true` when this call recorded the selection.
    pub fn record_connection_selection(
        &self,
        acquisition: ConnectionAcquisitionTelemetry,
        connection: ConnectionMetadata,
    ) -> bool {
        let mut state = self.lock();
        if state.selection.is_some() {
            return false;
        }
        state.selection = Some(ConnectionSelection {
            acquisition,
            connection,
        });
        true
    }

    fn lock(&self) -> MutexGuard<'_, HttpAttemptTelemetry> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl fmt::Debug for CaptureHttpAttemptTelemetry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CaptureHttpAttemptTelemetry")
            .finish_non_exhaustive()
    }
}

impl Storable for CaptureHttpAttemptTelemetry {
    type Storer = StoreReplace<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> ConnectionMetadata {
        ConnectionMetadata::builder()
            .proxied(false)
            .poison_fn(|| {})
            .build()
    }

    #[test]
    fn records_connector_call_and_selection_independently() {
        let capture = CaptureHttpAttemptTelemetry::new();
        let acquisition =
            ConnectionAcquisitionTelemetry::new(Duration::from_millis(3), ConnectionUsage::Fresh);

        assert!(capture.record_connection_selection(acquisition, connection()));
        assert!(capture.record_connector_call_duration(Duration::from_millis(7)));

        let telemetry = capture.get();
        assert_eq!(telemetry.acquisition(), Some(&acquisition));
        assert_eq!(
            telemetry.connector_call_duration(),
            Some(Duration::from_millis(7))
        );
        assert!(telemetry.connection().is_some());
    }

    #[test]
    fn first_recorded_value_wins() {
        let capture = CaptureHttpAttemptTelemetry::new();
        let first =
            ConnectionAcquisitionTelemetry::new(Duration::from_millis(3), ConnectionUsage::Fresh);
        let second =
            ConnectionAcquisitionTelemetry::new(Duration::from_millis(9), ConnectionUsage::Reused);
        assert!(capture.record_connection_selection(first, connection()));
        assert!(!capture.record_connection_selection(second, connection()));
        assert!(capture.record_connector_call_duration(Duration::from_millis(4)));
        assert!(!capture.record_connector_call_duration(Duration::from_millis(8)));

        let telemetry = capture.get();
        assert_eq!(telemetry.acquisition(), Some(&first));
        assert_eq!(
            telemetry.connector_call_duration(),
            Some(Duration::from_millis(4))
        );
    }

    #[test]
    fn backwards_acquisition_interval_retains_selection_facts() {
        let started_at = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
        let completed_at = SystemTime::UNIX_EPOCH + Duration::from_secs(1);

        let acquisition = ConnectionAcquisitionTelemetry::from_interval(
            started_at,
            completed_at,
            ConnectionUsage::Fresh,
        );
        let capture = CaptureHttpAttemptTelemetry::new();
        assert!(capture.record_connection_selection(acquisition, connection()));

        let telemetry = capture.get();
        assert_eq!(telemetry.acquisition().expect("selection").duration(), None);
        assert_eq!(
            telemetry.acquisition().expect("selection").usage(),
            ConnectionUsage::Fresh
        );
        assert!(telemetry.connection().is_some());
    }
}
