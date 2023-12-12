/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::collections::VecDeque;
use std::fmt;
use std::time::{Duration, SystemTime};

/// Throughput representation for use when configuring [`super::MinimumThroughputBody`]
#[derive(Debug, Clone, Copy)]
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
    pub fn new_bytes_per_second(bytes: u64) -> Self {
        Self {
            bytes_read: bytes,
            per_time_elapsed: Duration::from_secs(1),
        }
    }

    /// Create a new throughput in kilobytes per second.
    pub fn new_kilobytes_per_second(kilobytes: u64) -> Self {
        Self {
            bytes_read: kilobytes * 1000,
            per_time_elapsed: Duration::from_secs(1),
        }
    }

    /// Create a new throughput in megabytes per second.
    pub fn new_megabytes_per_second(megabytes: u64) -> Self {
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

#[derive(Clone)]
pub(super) struct ThroughputLogs {
    max_length: usize,
    inner: VecDeque<(SystemTime, u64)>,
    bytes_processed: u64,
}

impl ThroughputLogs {
    pub(super) fn new(max_length: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(max_length),
            max_length,
            bytes_processed: 0,
        }
    }

    pub(super) fn push(&mut self, throughput: (SystemTime, u64)) {
        // When the number of logs exceeds the max length, toss the oldest log.
        if self.inner.len() == self.max_length {
            self.bytes_processed -= self.inner.pop_front().map(|(_, sz)| sz).unwrap_or_default();
        }

        debug_assert!(self.inner.capacity() > self.inner.len());
        self.bytes_processed += throughput.1;
        self.inner.push_back(throughput);
    }

    fn buffer_full(&self) -> bool {
        self.inner.len() == self.max_length
    }

    pub(super) fn calculate_throughput(
        &self,
        now: SystemTime,
        time_window: Duration,
    ) -> Option<Throughput> {
        // There are a lot of pathological cases that are 0 throughput. These cases largely shouldn't
        // happen, because the check interval MUST be less than the check window
        let total_length = self
            .inner
            .iter()
            .last()?
            .0
            .duration_since(self.inner.get(0)?.0)
            .ok()?;
        // during a "healthy" request we'll only have a few milliseconds of logs (shorter than the check window)
        if total_length < time_window {
            // if we haven't hit our requested time window & the buffer still isn't full, then
            // return `None` — this is the "startup grace period"
            return if !self.buffer_full() {
                None
            } else {
                // Otherwise, if the entire buffer fits in the timewindow, we can the shortcut to
                // avoid recomputing all the data
                Some(Throughput {
                    bytes_read: self.bytes_processed,
                    per_time_elapsed: total_length,
                })
            };
        }
        let minimum_ts = now - time_window;
        let first_item = self.inner.iter().find(|(ts, _)| *ts >= minimum_ts)?.0;

        let time_elapsed = now.duration_since(first_item).unwrap_or_default();

        let total_bytes_logged = self
            .inner
            .iter()
            .rev()
            .take_while(|(ts, _)| *ts > minimum_ts)
            .map(|t| t.1)
            .sum::<u64>();

        Some(Throughput {
            bytes_read: total_bytes_logged,
            per_time_elapsed: time_elapsed,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{Throughput, ThroughputLogs};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_throughput_eq() {
        let t1 = Throughput::new(1, Duration::from_secs(1));
        let t2 = Throughput::new(25, Duration::from_secs(25));
        let t3 = Throughput::new(100, Duration::from_secs(100));

        assert_eq!(t1, t2);
        assert_eq!(t2, t3);
    }

    fn build_throughput_log(
        length: u32,
        tick_duration: Duration,
        rate: u64,
    ) -> (ThroughputLogs, SystemTime) {
        let mut throughput_logs = ThroughputLogs::new(length as usize);
        for i in 1..=length {
            throughput_logs.push((UNIX_EPOCH + (tick_duration * i), rate));
        }

        assert_eq!(length as usize, throughput_logs.inner.len());
        (throughput_logs, UNIX_EPOCH + (tick_duration * length))
    }

    const EPSILON: f64 = 0.001;
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !(($x as f64) - $y < $d || $y - ($x as f64) < $d) {
                panic!();
            }
        };
    }

    #[test]
    fn test_throughput_log_calculate_throughput_1() {
        let (throughput_logs, now) = build_throughput_log(1000, Duration::from_secs(1), 1);

        for dur in [10, 100, 100] {
            let throughput = throughput_logs
                .calculate_throughput(now, Duration::from_secs(dur))
                .unwrap();
            assert_eq!(1.0, throughput.bytes_per_second());
        }
        let throughput = throughput_logs
            .calculate_throughput(now, Duration::from_secs_f64(101.5))
            .unwrap();
        assert_delta!(1, throughput.bytes_per_second(), EPSILON);
    }

    #[test]
    fn test_throughput_log_calculate_throughput_2() {
        let (throughput_logs, now) = build_throughput_log(1000, Duration::from_secs(5), 5);

        let throughput = throughput_logs
            .calculate_throughput(now, Duration::from_secs(1000))
            .unwrap();
        assert_eq!(1.0, throughput.bytes_per_second());
    }

    #[test]
    fn test_throughput_log_calculate_throughput_3() {
        let (throughput_logs, now) = build_throughput_log(1000, Duration::from_millis(200), 1024);

        let throughput = throughput_logs
            .calculate_throughput(now, Duration::from_secs(5))
            .unwrap();
        let expected_throughput = 1024.0 * 5.0;
        assert_eq!(expected_throughput, throughput.bytes_per_second());
    }

    #[test]
    fn test_throughput_log_calculate_throughput_4() {
        let (throughput_logs, now) = build_throughput_log(1000, Duration::from_millis(100), 12);

        let throughput = throughput_logs
            .calculate_throughput(now, Duration::from_secs(1))
            .unwrap();
        let expected_throughput = 12.0 * 10.0;

        assert_eq!(expected_throughput, throughput.bytes_per_second());
    }

    #[test]
    fn test_throughput_followed_by_0() {
        let tick = Duration::from_millis(100);
        let (mut throughput_logs, now) = build_throughput_log(1000, tick, 12);
        let throughput = throughput_logs
            .calculate_throughput(now, Duration::from_secs(1))
            .unwrap();
        let expected_throughput = 12.0 * 10.0;

        assert_eq!(expected_throughput, throughput.bytes_per_second());
        throughput_logs.push((now + tick, 0));
        let throughput = throughput_logs
            .calculate_throughput(now + tick, Duration::from_secs(1))
            .unwrap();
        assert_eq!(108.0, throughput.bytes_per_second());
    }
}
