/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Time source abstraction
///
/// Simple abstraction representing time either real-time or manually-specified for testing
///
/// # Examples
///
/// ```rust
/// # struct Client {
/// #  // stub
/// # }
/// #
/// # impl Client {
/// #     fn with_timesource(ts: TimeSource) -> Self {
/// #         Client { }
/// #     }
/// # }
/// use aws_credential_types::time_source::TimeSource;
/// let time = TimeSource::default();
/// let client = Client::with_timesource(time);
/// ```
#[derive(Debug, Clone)]
pub struct TimeSource(Inner);

impl TimeSource {
    /// Creates `TimeSource` from the manually specified `time_source`.
    pub fn testing(time_source: &TestingTimeSource) -> Self {
        TimeSource(Inner::Testing(time_source.clone()))
    }

    /// Returns the current system time based on the mode.
    pub fn now(&self) -> SystemTime {
        match &self.0 {
            Inner::Default => SystemTime::now(),
            Inner::Testing(testing) => testing.now(),
        }
    }
}

impl Default for TimeSource {
    /// Creates `TimeSource` from the current system time.
    fn default() -> Self {
        TimeSource(Inner::Default)
    }
}

/// Time Source that can be manually moved for tests
///
/// # Examples
///
/// ```rust
/// # struct Client {
/// #  // stub
/// # }
/// #
/// # impl Client {
/// #     fn with_timesource(ts: TimeSource) -> Self {
/// #         Client { }
/// #     }
/// # }
/// use aws_credential_types::time_source::{TestingTimeSource, TimeSource};
/// use std::time::{UNIX_EPOCH, Duration};
/// let mut time = TestingTimeSource::new(UNIX_EPOCH);
/// let client = Client::with_timesource(TimeSource::testing(&time));
/// time.advance(Duration::from_secs(100));
/// ```
#[derive(Clone, Debug)]
pub struct TestingTimeSource {
    queries: Arc<Mutex<Vec<SystemTime>>>,
    now: Arc<Mutex<SystemTime>>,
}

impl TestingTimeSource {
    /// Creates `TestingTimeSource` with `start_time`.
    pub fn new(start_time: SystemTime) -> Self {
        Self {
            queries: Default::default(),
            now: Arc::new(Mutex::new(start_time)),
        }
    }

    /// Sets time to the specified `time`.
    pub fn set_time(&mut self, time: SystemTime) {
        let mut now = self.now.lock().unwrap();
        *now = time;
    }

    /// Advances time by `delta`.
    pub fn advance(&mut self, delta: Duration) {
        let mut now = self.now.lock().unwrap();
        *now += delta;
    }

    /// Returns a `Vec` of queried times so far.
    pub fn queries(&self) -> impl Deref<Target = Vec<SystemTime>> + '_ {
        self.queries.lock().unwrap()
    }

    /// Returns the current time understood by `TestingTimeSource`.
    pub fn now(&self) -> SystemTime {
        let ts = *self.now.lock().unwrap();
        self.queries.lock().unwrap().push(ts);
        ts
    }
}

// In the future, if needed we can add a time source trait, however, the testing time source
// should cover most test use cases.
#[derive(Debug, Clone)]
enum Inner {
    Default,
    Testing(TestingTimeSource),
}

#[cfg(test)]
mod test {
    use super::{TestingTimeSource, TimeSource};

    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn default_time_source_should_not_panic_on_calling_now() {
        let time_source = TimeSource::default();
        // no panics
        let _ = time_source.now();
    }

    #[test]
    fn testing_time_source_should_behave_as_expected() {
        let mut testing = TestingTimeSource::new(UNIX_EPOCH);
        let time_source = TimeSource::testing(&testing);
        assert_eq!(time_source.now(), UNIX_EPOCH);
        testing.advance(Duration::from_secs(10));
        assert_eq!(time_source.now(), UNIX_EPOCH + Duration::from_secs(10));
    }
}
