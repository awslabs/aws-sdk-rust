/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Diagnostic connection state for one origin or partition-origin cell.
//!
//! Admission and cell state remain authoritative. These values combine exact
//! observations under their existing locks with relaxed lifetime counts for
//! work that can outlive a cell record. They must not be used to make pool
//! admission or dispatch decisions.
//!
//! [`ConnectionPool::origin_stats`](super::ConnectionPool::origin_stats)
//! reports the shared origin capacity budget.
//! [`ConnectionPool::partition_stats`](super::ConnectionPool::partition_stats)
//! reports one exact partition-origin cell without scanning other partitions.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Bounded connection capacity shared by every partition for one origin.
///
/// Capacity is retained by establishment, open connections, and detached
/// HTTP/1 upgrades. Ordinary draining connections return capacity when they
/// stop accepting pool dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct ConnectionCapacityStats {
    limit: usize,
    in_use: usize,
}

impl ConnectionCapacityStats {
    pub(super) fn new(limit: usize, in_use: usize) -> Self {
        Self { limit, in_use }
    }

    /// Returns the configured maximum across all partitions for this origin.
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// Returns capacity currently owned by establishment or connections.
    pub fn in_use(&self) -> usize {
        self.in_use
    }
}

/// Origin-wide connection capacity.
///
/// This snapshot contains no aggregate of partition-local protocol state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct OriginConnectionStats {
    capacity: Option<ConnectionCapacityStats>,
}

impl OriginConnectionStats {
    pub(super) fn new(capacity: Option<ConnectionCapacityStats>) -> Self {
        Self { capacity }
    }

    /// Returns bounded-origin capacity, or `None` when the origin is unbounded.
    pub fn capacity(&self) -> Option<&ConnectionCapacityStats> {
        self.capacity.as_ref()
    }
}

/// HTTP/1 connection state owned by one partition-origin cell.
///
/// Idle and active counts are exact under the cell lock. Draining and upgraded
/// counts use relaxed lifetime accounting because those connections can outlive
/// their cell records.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct Http1ConnectionStats {
    idle: usize,
    active: usize,
    draining: usize,
    upgraded: usize,
}

impl Http1ConnectionStats {
    /// Returns connections available for immediate local selection.
    pub fn idle(&self) -> usize {
        self.idle
    }

    /// Returns connections whose sender is selected or reserved outside idle storage.
    ///
    /// This includes senders checked out for local dispatch and senders reserved
    /// for eligible peer demand.
    pub fn active(&self) -> usize {
        self.active
    }

    /// Returns ordinary logically closed connections that still own root I/O.
    pub fn draining(&self) -> usize {
        self.draining
    }

    /// Returns upgraded connections whose root transport I/O is caller-owned.
    pub fn upgraded(&self) -> usize {
        self.upgraded
    }
}

/// HTTP/2 connection state owned by one partition-origin cell.
///
/// Accepting connections and active requests are exact under the cell lock.
/// Draining connections use relaxed lifetime accounting because their root I/O
/// can outlive the installed generation record.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct Http2ConnectionStats {
    accepting: usize,
    draining: usize,
    active_requests: usize,
}

impl Http2ConnectionStats {
    /// Returns installed connections that may accept new HTTP/2 requests.
    pub fn accepting(&self) -> usize {
        self.accepting
    }

    /// Returns logically closed connections that still own root transport I/O.
    pub fn draining(&self) -> usize {
        self.draining
    }

    /// Returns accepted requests whose upload and response sides have not both finished.
    pub fn active_requests(&self) -> usize {
        self.active_requests
    }
}

/// Diagnostic connection state for one configured partition and origin.
///
/// This snapshot is not transactional across its exact cell-owned values and
/// relaxed lifetime counts. The relaxed counts are nonnegative and converge
/// after concurrent transitions settle.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub struct PartitionConnectionStats {
    pending_acquisitions: usize,
    establishing_connections: usize,
    h1: Http1ConnectionStats,
    h2: Http2ConnectionStats,
    physically_live_connections: usize,
}

impl PartitionConnectionStats {
    /// Returns requests without a terminal cell-local acquisition outcome.
    pub fn pending_acquisitions(&self) -> usize {
        self.pending_acquisitions
    }

    /// Returns connection establishments that have not reached a terminal outcome.
    ///
    /// The count spans connector, protocol selection, Hyper handshake, and pool
    /// installation work.
    pub fn establishing_connections(&self) -> usize {
        self.establishing_connections
    }

    /// Returns HTTP/1 connection state.
    pub fn h1(&self) -> &Http1ConnectionStats {
        &self.h1
    }

    /// Returns HTTP/2 connection state.
    pub fn h2(&self) -> &Http2ConnectionStats {
        &self.h2
    }

    /// Returns root transport handles still owned by the client.
    ///
    /// The operating system may continue transport teardown after this count
    /// decreases.
    pub fn physically_live_connections(&self) -> usize {
        self.physically_live_connections
    }
}

/// Relaxed lifetime counts retained by one partition-origin cell.
///
/// These counts cover lifetimes that can outlive the corresponding cell
/// record. Exact waiter and accepting-connection state remains under the cell
/// lock and is supplied when a snapshot is composed.
#[derive(Debug, Default)]
pub(super) struct CellConnectionStats {
    establishing: AtomicUsize,
    h1_draining: AtomicUsize,
    h1_upgraded: AtomicUsize,
    h2_draining: AtomicUsize,
    physically_live: AtomicUsize,
}

impl CellConnectionStats {
    pub(super) fn establishment_started(&self) {
        increment(
            &self.establishing,
            "connection establishment count exhausted",
        );
    }

    pub(super) fn establishment_finished(&self) {
        decrement(
            &self.establishing,
            "connection establishment completed without a matching start",
        );
    }

    pub(super) fn physical_connection_started(&self) {
        increment(&self.physically_live, "physical connection count exhausted");
    }

    pub(super) fn physical_connection_finished(&self) {
        decrement(
            &self.physically_live,
            "physical connection completed without a matching start",
        );
    }

    pub(super) fn h1_drain_started(&self) {
        increment(&self.h1_draining, "HTTP/1 draining count exhausted");
    }

    pub(super) fn h1_drain_finished(&self) {
        decrement(
            &self.h1_draining,
            "HTTP/1 drain completed without a matching start",
        );
    }

    pub(super) fn h1_drain_upgraded(&self) {
        self.h1_drain_finished();
        increment(&self.h1_upgraded, "HTTP/1 upgraded count exhausted");
    }

    pub(super) fn h1_upgrade_started(&self) {
        increment(&self.h1_upgraded, "HTTP/1 upgraded count exhausted");
    }

    pub(super) fn h1_upgrade_finished(&self) {
        decrement(
            &self.h1_upgraded,
            "HTTP/1 upgrade completed without a matching start",
        );
    }

    pub(super) fn h2_drain_started(&self) {
        increment(&self.h2_draining, "HTTP/2 draining count exhausted");
    }

    pub(super) fn h2_drain_finished(&self) {
        decrement(
            &self.h2_draining,
            "HTTP/2 drain completed without a matching start",
        );
    }

    pub(super) fn snapshot(
        &self,
        pending_acquisitions: usize,
        h1_idle: usize,
        h1_active: usize,
        h2_accepting: usize,
        h2_active_requests: usize,
    ) -> PartitionConnectionStats {
        PartitionConnectionStats {
            pending_acquisitions,
            establishing_connections: self.establishing.load(Ordering::Relaxed),
            h1: Http1ConnectionStats {
                idle: h1_idle,
                active: h1_active,
                draining: self.h1_draining.load(Ordering::Relaxed),
                upgraded: self.h1_upgraded.load(Ordering::Relaxed),
            },
            h2: Http2ConnectionStats {
                accepting: h2_accepting,
                draining: self.h2_draining.load(Ordering::Relaxed),
                active_requests: h2_active_requests,
            },
            physically_live_connections: self.physically_live.load(Ordering::Relaxed),
        }
    }
}

fn increment(counter: &AtomicUsize, message: &'static str) {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .expect(message);
}

fn decrement(counter: &AtomicUsize, message: &'static str) {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_sub(1)
        })
        .expect(message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_combines_exact_and_lifetime_state() {
        let stats = CellConnectionStats::default();
        stats.establishment_started();
        stats.physical_connection_started();
        stats.h1_drain_started();
        stats.h1_drain_upgraded();
        stats.h2_drain_started();

        let snapshot = stats.snapshot(2, 3, 4, 5, 6);

        assert_eq!(2, snapshot.pending_acquisitions());
        assert_eq!(1, snapshot.establishing_connections());
        assert_eq!(3, snapshot.h1().idle());
        assert_eq!(4, snapshot.h1().active());
        assert_eq!(0, snapshot.h1().draining());
        assert_eq!(1, snapshot.h1().upgraded());
        assert_eq!(5, snapshot.h2().accepting());
        assert_eq!(1, snapshot.h2().draining());
        assert_eq!(6, snapshot.h2().active_requests());
        assert_eq!(1, snapshot.physically_live_connections());
    }
}
