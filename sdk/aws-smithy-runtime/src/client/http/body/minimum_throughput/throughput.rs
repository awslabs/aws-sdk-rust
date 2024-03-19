/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fmt;
use std::time::{Duration, SystemTime};

/// Throughput representation for use when configuring [`super::MinimumThroughputBody`]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(test, derive(Eq))]
pub struct Throughput {
    pub(super) bytes_read: u64,
    pub(super) per_time_elapsed: Duration,
}

impl Throughput {
    /// Create a new throughput with the given bytes read and time elapsed.
    pub fn new(bytes_read: u64, per_time_elapsed: Duration) -> Self {
        debug_assert!(
            !per_time_elapsed.is_zero(),
            "cannot create a throughput if per_time_elapsed == 0"
        );

        Self {
            bytes_read,
            per_time_elapsed,
        }
    }

    /// Create a new throughput in bytes per second.
    pub const fn new_bytes_per_second(bytes: u64) -> Self {
        Self {
            bytes_read: bytes,
            per_time_elapsed: Duration::from_secs(1),
        }
    }

    /// Create a new throughput in kilobytes per second.
    pub const fn new_kilobytes_per_second(kilobytes: u64) -> Self {
        Self {
            bytes_read: kilobytes * 1000,
            per_time_elapsed: Duration::from_secs(1),
        }
    }

    /// Create a new throughput in megabytes per second.
    pub const fn new_megabytes_per_second(megabytes: u64) -> Self {
        Self {
            bytes_read: megabytes * 1000 * 1000,
            per_time_elapsed: Duration::from_secs(1),
        }
    }

    pub(super) fn bytes_per_second(&self) -> f64 {
        let per_time_elapsed_secs = self.per_time_elapsed.as_secs_f64();
        if per_time_elapsed_secs == 0.0 {
            return 0.0; // Avoid dividing by zero.
        };

        self.bytes_read as f64 / per_time_elapsed_secs
    }
}

impl PartialEq for Throughput {
    fn eq(&self, other: &Self) -> bool {
        self.bytes_per_second() == other.bytes_per_second()
    }
}

impl PartialOrd for Throughput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.bytes_per_second()
            .partial_cmp(&other.bytes_per_second())
    }
}

impl fmt::Display for Throughput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The default float formatting behavior will ensure the a number like 2.000 is rendered as 2
        // while a number like 0.9982107441748642 will be rendered as 0.9982107441748642. This
        // multiplication and division will truncate a float to have a precision of no greater than 3.
        // For example, 0.9982107441748642 would become 0.999. This will fail for very large floats
        // but should suffice for the numbers we're dealing with.
        let pretty_bytes_per_second = (self.bytes_per_second() * 1000.0).round() / 1000.0;

        write!(f, "{pretty_bytes_per_second} B/s")
    }
}

impl From<(u64, Duration)> for Throughput {
    fn from(value: (u64, Duration)) -> Self {
        Self {
            bytes_read: value.0,
            per_time_elapsed: value.1,
        }
    }
}

/// Represents a bin (or a cell) in a linear grid that represents a small chunk of time.
#[derive(Copy, Clone, Debug)]
enum Bin {
    /// There is no data in this bin.
    Empty,

    /// No polling took place during this bin.
    NoPolling,

    /// This many bytes were transferred during this bin.
    TransferredBytes(u64),

    /// The user/remote was not providing/consuming data fast enough during this bin.
    ///
    /// The number is the number of bytes transferred, if this replaced TransferredBytes.
    Pending(u64),
}
impl Bin {
    fn merge(&mut self, other: Bin) {
        use Bin::*;
        // Assign values based on this priority order (highest priority higher up):
        //   1. Pending
        //   2. TransferredBytes
        //   3. NoPolling
        //   4. Empty
        *self = match (other, *self) {
            (Pending(other), this) => Pending(other + this.bytes()),
            (TransferredBytes(other), this) => TransferredBytes(other + this.bytes()),
            (other, NoPolling) => other,
            (NoPolling, _) => panic!("can't merge NoPolling into bin"),
            (other, Empty) => other,
            (Empty, _) => panic!("can't merge Empty into bin"),
        };
    }

    /// Number of bytes transferred during this bin
    fn bytes(&self) -> u64 {
        match *self {
            Bin::Empty | Bin::NoPolling => 0,
            Bin::TransferredBytes(bytes) | Bin::Pending(bytes) => bytes,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct BinCounts {
    /// Number of bins with no data.
    empty: usize,
    /// Number of "no polling" bins.
    no_polling: usize,
    /// Number of "bytes transferred" bins.
    transferred: usize,
    /// Number of "pending" bins.
    pending: usize,
}

/// Underlying stack-allocated linear grid buffer for tracking
/// throughput events for [`ThroughputLogs`].
#[derive(Copy, Clone, Debug)]
struct LogBuffer<const N: usize> {
    entries: [Bin; N],
    // The length only needs to exist so that the `fill_gaps` function
    // can differentiate between `Empty` due to there not having been enough
    // time to establish a full buffer worth of data vs. `Empty` due to a
    // polling gap. Once the length reaches N, it will never change again.
    length: usize,
}
impl<const N: usize> LogBuffer<N> {
    fn new() -> Self {
        Self {
            entries: [Bin::Empty; N],
            length: 0,
        }
    }

    /// Mutably returns the tail of the buffer.
    ///
    /// ## Panics
    ///
    /// The buffer MUST have at least one bin it it before this is called.
    fn tail_mut(&mut self) -> &mut Bin {
        debug_assert!(self.length > 0);
        &mut self.entries[self.length - 1]
    }

    /// Pushes a bin into the buffer. If the buffer is already full,
    /// then this will rotate the entire buffer to the left.
    fn push(&mut self, bin: Bin) {
        if self.filled() {
            self.entries.rotate_left(1);
            self.entries[N - 1] = bin;
        } else {
            self.entries[self.length] = bin;
            self.length += 1;
        }
    }

    /// Returns the total number of bytes transferred within the time window.
    fn bytes_transferred(&self) -> u64 {
        self.entries.iter().take(self.length).map(Bin::bytes).sum()
    }

    #[inline]
    fn filled(&self) -> bool {
        self.length == N
    }

    /// Fills in missing NoData entries.
    ///
    /// We want NoData entries to represent when a future hasn't been polled.
    /// Since the future is in charge of logging in the first place, the only
    /// way we can know about these is by examining gaps in time.
    fn fill_gaps(&mut self) {
        for entry in self.entries.iter_mut().take(self.length) {
            if matches!(entry, Bin::Empty) {
                *entry = Bin::NoPolling;
            }
        }
    }

    /// Returns the counts of each bin type in the buffer.
    fn counts(&self) -> BinCounts {
        let mut counts = BinCounts::default();
        for entry in &self.entries {
            match entry {
                Bin::Empty => counts.empty += 1,
                Bin::NoPolling => counts.no_polling += 1,
                Bin::TransferredBytes(_) => counts.transferred += 1,
                Bin::Pending(_) => counts.pending += 1,
            }
        }
        counts
    }
}

/// Report/summary of all the events in a time window.
#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) enum ThroughputReport {
    /// Not enough data to draw any conclusions. This happens early in a request/response.
    Incomplete,
    /// The stream hasn't been polled for most of this time window.
    NoPolling,
    /// The stream has been waiting for most of the time window.
    Pending,
    /// The stream transferred this amount of throughput during the time window.
    Transferred(Throughput),
}

const BUFFER_SIZE: usize = 10;

/// Log of throughput in a request or response stream.
///
/// Used to determine if a configured minimum throughput is being met or not
/// so that a request or response stream can be timed out in the event of a
/// stall.
///
/// Request/response streams push data transfer or pending events to this log
/// based on what's going on in their poll functions. The log tracks three kinds
/// of events despite only receiving two: the third is "no polling". The poll
/// functions cannot know when they're not being polled, so the log examines gaps
/// in the event history to know when no polling took place.
///
/// The event logging is simplified down to a linear grid consisting of 10 "bins",
/// with each bin representing 1/10th the total time window. When an event is pushed,
/// it is either merged into the current tail bin, or all the bins are rotated
/// left to create a new empty tail bin, and then it is merged into that one.
#[derive(Clone, Debug)]
pub(super) struct ThroughputLogs {
    resolution: Duration,
    current_tail: SystemTime,
    buffer: LogBuffer<BUFFER_SIZE>,
}

impl ThroughputLogs {
    /// Creates a new log starting at `now` with the given `time_window`.
    ///
    /// Note: the `time_window` gets divided by 10 to create smaller sub-windows
    /// to track throughput. The time window should configured to be large enough
    /// so that these sub-windows aren't too small for network-based events.
    /// A time window of 10ms probably won't work, but 500ms might. The default
    /// is one second.
    pub(super) fn new(time_window: Duration, now: SystemTime) -> Self {
        assert!(!time_window.is_zero());
        let resolution = time_window.div_f64(BUFFER_SIZE as f64);
        Self {
            resolution,
            current_tail: now,
            buffer: LogBuffer::new(),
        }
    }

    /// Returns the resolution at which events are logged at.
    pub(super) fn resolution(&self) -> Duration {
        self.resolution
    }

    /// Pushes a "pending" event.
    ///
    /// Pending indicates the streaming future is waiting for something.
    /// In an upload, it is waiting for data from the user, and in a download,
    /// it is waiting for data from the server.
    pub(super) fn push_pending(&mut self, time: SystemTime) {
        self.push(time, Bin::Pending(0));
    }

    /// Pushes a data transferred event.
    ///
    /// Indicates that this number of bytes were transferred at this time.
    pub(super) fn push_bytes_transferred(&mut self, time: SystemTime, bytes: u64) {
        self.push(time, Bin::TransferredBytes(bytes));
    }

    fn push(&mut self, now: SystemTime, value: Bin) {
        self.catch_up(now);
        self.buffer.tail_mut().merge(value);
        self.buffer.fill_gaps();
    }

    /// Pushes empty bins until `current_tail` is caught up to `now`.
    fn catch_up(&mut self, now: SystemTime) {
        while now >= self.current_tail {
            self.current_tail += self.resolution;
            self.buffer.push(Bin::Empty);
        }
        assert!(self.current_tail >= now);
    }

    /// Generates an overall report of the time window.
    pub(super) fn report(&mut self, now: SystemTime) -> ThroughputReport {
        self.catch_up(now);
        self.buffer.fill_gaps();

        let BinCounts {
            empty,
            no_polling,
            transferred,
            pending,
        } = self.buffer.counts();

        // If there are any empty cells at all, then we haven't been tracking
        // long enough to make any judgements about the stream's progress.
        if empty > 0 {
            return ThroughputReport::Incomplete;
        }

        let bytes = self.buffer.bytes_transferred();
        let time = self.resolution * (BUFFER_SIZE - empty) as u32;
        let throughput = Throughput::new(bytes, time);

        let half = BUFFER_SIZE / 2;
        match (transferred > 0, no_polling >= half, pending >= half) {
            (true, _, _) => ThroughputReport::Transferred(throughput),
            (_, true, _) => ThroughputReport::NoPolling,
            (_, _, true) => ThroughputReport::Pending,
            _ => ThroughputReport::Incomplete,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_throughput_eq() {
        let t1 = Throughput::new(1, Duration::from_secs(1));
        let t2 = Throughput::new(25, Duration::from_secs(25));
        let t3 = Throughput::new(100, Duration::from_secs(100));

        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
    }

    #[test]
    fn incomplete_no_entries() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);
        let report = logs.report(start);
        assert_eq!(ThroughputReport::Incomplete, report);
    }

    #[test]
    fn incomplete_with_entries() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);
        logs.push_pending(start);

        let report = logs.report(start + Duration::from_millis(300));
        assert_eq!(ThroughputReport::Incomplete, report);
    }

    #[test]
    fn incomplete_with_transferred() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);
        logs.push_pending(start);
        logs.push_bytes_transferred(start + Duration::from_millis(100), 10);

        let report = logs.report(start + Duration::from_millis(300));
        assert_eq!(ThroughputReport::Incomplete, report);
    }

    #[test]
    fn push_pending_at_the_beginning_of_each_tick() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);

        let mut now = start;
        for i in 1..=BUFFER_SIZE {
            logs.push_pending(now);
            now += logs.resolution();

            assert_eq!(i, logs.buffer.counts().pending);
        }

        let report = dbg!(&mut logs).report(now);
        assert_eq!(ThroughputReport::Pending, report);
    }

    #[test]
    fn push_pending_at_the_end_of_each_tick() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);

        let mut now = start;
        for i in 1..BUFFER_SIZE {
            now += logs.resolution();
            logs.push_pending(now);

            assert_eq!(i, dbg!(&logs).buffer.counts().pending);
            assert_eq!(0, logs.buffer.counts().transferred);
            assert_eq!(1, logs.buffer.counts().no_polling);
        }
        // This should replace the initial "no polling" bin
        now += logs.resolution();
        logs.push_pending(now);
        assert_eq!(0, logs.buffer.counts().no_polling);

        let report = dbg!(&mut logs).report(now);
        assert_eq!(ThroughputReport::Pending, report);
    }

    #[test]
    fn push_transferred_at_the_beginning_of_each_tick() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);

        let mut now = start;
        for i in 1..=BUFFER_SIZE {
            logs.push_bytes_transferred(now, 10);
            if i != BUFFER_SIZE {
                now += logs.resolution();
            }

            assert_eq!(i, logs.buffer.counts().transferred);
            assert_eq!(0, logs.buffer.counts().pending);
            assert_eq!(0, logs.buffer.counts().no_polling);
        }

        let report = dbg!(&mut logs).report(now);
        assert_eq!(
            ThroughputReport::Transferred(Throughput::new(100, Duration::from_secs(1))),
            report
        );
    }

    #[test]
    fn no_polling() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);
        let report = logs.report(start + Duration::from_secs(2));
        assert_eq!(ThroughputReport::NoPolling, report);
    }

    // Transferred bytes MUST take priority over pending
    #[test]
    fn mixed_bag_mostly_pending() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);

        logs.push_bytes_transferred(start + Duration::from_millis(50), 10);
        logs.push_pending(start + Duration::from_millis(150));
        logs.push_pending(start + Duration::from_millis(250));
        logs.push_bytes_transferred(start + Duration::from_millis(350), 10);
        logs.push_pending(start + Duration::from_millis(450));
        // skip 550
        logs.push_pending(start + Duration::from_millis(650));
        logs.push_pending(start + Duration::from_millis(750));
        logs.push_pending(start + Duration::from_millis(850));

        let report = logs.report(start + Duration::from_millis(999));
        assert_eq!(
            ThroughputReport::Transferred(Throughput::new_bytes_per_second(20)),
            report
        );
    }

    #[test]
    fn mixed_bag_mostly_pending_no_transferred() {
        let start = SystemTime::UNIX_EPOCH;
        let mut logs = ThroughputLogs::new(Duration::from_secs(1), start);

        logs.push_pending(start + Duration::from_millis(50));
        logs.push_pending(start + Duration::from_millis(150));
        logs.push_pending(start + Duration::from_millis(250));
        // skip 350
        logs.push_pending(start + Duration::from_millis(450));
        // skip 550
        logs.push_pending(start + Duration::from_millis(650));
        logs.push_pending(start + Duration::from_millis(750));
        logs.push_pending(start + Duration::from_millis(850));

        let report = logs.report(start + Duration::from_millis(999));
        assert_eq!(ThroughputReport::Pending, report);
    }
}
