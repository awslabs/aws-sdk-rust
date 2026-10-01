/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Partition-owned idle connection maintenance.
//!
//! Idle insertion publishes only a deadline earlier than the task's current
//! wakeup. One task per partition snapshots retained cells, closes expired
//! records, drops the snapshot, and then waits for the nearest deadline,
//! earlier work, or explicit pool shutdown.

use super::cell::{CellId, OriginCell};
use super::partition::DriverSpawner;
use crate::sync::{Arc, AtomicBool, Mutex, Ordering, Weak};
use aws_smithy_async::rt::sleep::{AsyncSleep, SharedAsyncSleep};
use aws_smithy_async::time::SharedTimeSource;
use std::collections::HashMap;
use std::future::{poll_fn, Future};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, SystemTime};

/// Immutable pool policy copied into each partition scheduler.
#[derive(Clone, Debug, Default)]
pub(super) struct MaintenanceConfig {
    /// Idle duration applied to reusable H1 records.
    pub(super) idle_timeout: Option<Duration>,
    /// Clock used to assign and evaluate deadlines.
    pub(super) time_source: SharedTimeSource,
    /// Sleeper used by partition tasks.
    pub(super) sleep: Option<SharedAsyncSleep>,
}

/// Clock and wakeup state shared by one partition's maintenance task.
#[derive(Debug)]
pub(super) struct PartitionMaintenance {
    /// Idle duration applied whenever an H1 sender enters reusable storage.
    idle_timeout: Option<Duration>,
    /// Pool-owned clock used to assign and evaluate idle deadlines.
    time_source: SharedTimeSource,
    /// Pool-owned sleeper used by the partition task.
    sleep: Option<SharedAsyncSleep>,
    /// Fast start-once gate kept off the per-request lock path.
    started: AtomicBool,
    /// Registered cells, task state, and wake revision.
    state: Mutex<MaintenanceState>,
}

impl PartitionMaintenance {
    /// Creates a scheduler from immutable pool maintenance policy.
    pub(super) fn new(config: MaintenanceConfig) -> Arc<Self> {
        Arc::new(Self {
            idle_timeout: config.idle_timeout,
            time_source: config.time_source,
            sleep: config.sleep,
            started: AtomicBool::new(false),
            state: Mutex::new(MaintenanceState::default()),
        })
    }

    /// Returns the deadline for a sender becoming idle now.
    pub(super) fn idle_deadline(&self) -> Option<SystemTime> {
        self.idle_timeout
            .and_then(|timeout| self.time_source.now().checked_add(timeout))
    }

    /// Registers a retained cell without waking an idle task.
    pub(super) fn register(&self, cell: &Arc<OriginCell>) {
        self.state
            .lock()
            .cells
            .insert(cell.id().clone(), Weak::from_arc(cell));
    }

    /// Publishes newly idle work when it precedes the current wakeup.
    pub(super) fn notify_deadline(&self, deadline: Option<SystemTime>) {
        let Some(deadline) = deadline else {
            return;
        };
        let wake = {
            let mut state = self.state.lock();
            if state.shutdown
                || state
                    .scheduled_deadline
                    .is_some_and(|scheduled| scheduled <= deadline)
            {
                return;
            }
            state.scheduled_deadline = Some(deadline);
            state.signal()
        };
        if let Some(waker) = wake {
            waker.wake();
        }
    }

    /// Starts this partition's maintenance task at most once.
    pub(super) fn start(this: &Arc<Self>, spawner: &DriverSpawner) {
        if this.idle_timeout.is_none()
            || this
                .started
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return;
        }

        let task = MaintenanceTask::new(Weak::from_arc(this));
        spawner.spawn(Box::pin(async move {
            run(task.scheduler()).await;
            task.complete();
        }));
    }

    /// Stops the task and wakes it even when no idle record remains.
    pub(super) fn shutdown(&self) {
        let wake = {
            let mut state = self.state.lock();
            if state.shutdown {
                return;
            }
            state.shutdown = true;
            state.scheduled_deadline = None;
            state.signal()
        };
        if let Some(waker) = wake {
            waker.wake();
        }
    }

    /// Starts a scan and retires the deadline whose wake triggered it.
    ///
    /// Clearing the represented deadline under the scheduler lock ensures any
    /// idle publication during the unlocked cell scan advances the revision.
    fn begin_scan(&self) -> Option<u64> {
        let mut state = self.state.lock();
        if state.shutdown {
            return None;
        }
        state.scheduled_deadline = None;
        Some(state.revision)
    }

    /// Returns live registered cells and prunes expired weak entries.
    fn cells(&self) -> Vec<Arc<OriginCell>> {
        let mut state = self.state.lock();
        let cells = state
            .cells
            .values()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        state.cells.retain(|_, cell| cell.upgrade().is_some());
        cells
    }

    /// Commits the scan result when no publication raced the scan.
    fn schedule(&self, observed: u64, deadline: Option<SystemTime>) -> ScheduleResult {
        let mut state = self.state.lock();
        if state.shutdown {
            return ScheduleResult::Shutdown;
        }
        if state.revision != observed {
            return ScheduleResult::Retry;
        }
        state.scheduled_deadline = deadline;
        ScheduleResult::Wait {
            revision: state.revision,
            deadline,
        }
    }

    /// Polls until a deadline publication or shutdown changes `observed`.
    fn poll_revision(&self, observed: u64, cx: &Context<'_>) -> Poll<()> {
        let mut state = self.state.lock();
        if state.shutdown || state.revision != observed {
            return Poll::Ready(());
        }
        if state
            .waker
            .as_ref()
            .is_none_or(|registered| !registered.will_wake(cx.waker()))
        {
            state.waker = Some(cx.waker().clone());
        }
        Poll::Pending
    }

    #[cfg(all(test, smithy_http_client_loom))]
    pub(super) fn clear_modeled_cells_for_test(&self) {
        // Loom has no modeled Weak, so its synchronization facade retains
        // registered cells strongly. Explicit teardown prevents that model-only
        // substitution from creating a maintenance -> cell -> maintenance cycle.
        self.state.lock().cells.clear();
    }

    #[cfg(all(test, feature = "rt-tokio", not(smithy_http_client_loom)))]
    fn probe(&self) -> MaintenanceProbe {
        let state = self.state.lock();
        MaintenanceProbe {
            started: self.started.load(Ordering::Acquire),
            shutdown: state.shutdown,
            scheduled_deadline: state.scheduled_deadline,
        }
    }
}

/// Mutable scheduler state protected independently from every origin cell.
#[derive(Debug, Default)]
struct MaintenanceState {
    /// Retained partition cells indexed for replacement on duplicate publish.
    cells: HashMap<CellId, Weak<OriginCell>>,
    /// Monotonic change counter observed by the maintenance future.
    revision: u64,
    /// Task waiting for a new revision.
    waker: Option<Waker>,
    /// Deadline the current sleep represents, if any.
    scheduled_deadline: Option<SystemTime>,
    /// Whether pool drop requested task termination.
    shutdown: bool,
}

impl MaintenanceState {
    /// Advances the wake revision and detaches the current task waker.
    fn signal(&mut self) -> Option<Waker> {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("partition maintenance revision exhausted");
        self.waker.take()
    }
}

/// Result of publishing one completed maintenance scan.
enum ScheduleResult {
    /// A publication raced the scan; rescan before waiting.
    Retry,
    /// Explicit pool shutdown won.
    Shutdown,
    /// Wait for this revision's nearest deadline or a new publication.
    Wait {
        revision: u64,
        deadline: Option<SystemTime>,
    },
}

/// Owns the start latch while one submitted maintenance future is live.
struct MaintenanceTask {
    /// Scheduler whose start latch this submitted future owns.
    scheduler: Weak<PartitionMaintenance>,
    /// Whether `Drop` still owes start-latch recovery.
    active: bool,
}

impl MaintenanceTask {
    /// Arms task-drop recovery for one submitted maintenance future.
    fn new(scheduler: Weak<PartitionMaintenance>) -> Self {
        Self {
            scheduler,
            active: true,
        }
    }

    /// Returns the weak scheduler handle polled by the task body.
    fn scheduler(&self) -> Weak<PartitionMaintenance> {
        self.scheduler.clone()
    }

    /// Disarms start-latch recovery after the task exits normally.
    fn complete(mut self) {
        self.active = false;
    }
}

impl Drop for MaintenanceTask {
    fn drop(&mut self) {
        if self.active {
            if let Some(scheduler) = self.scheduler.upgrade() {
                scheduler.started.store(false, Ordering::Release);
            }
        }
    }
}

/// Runs one scan per deadline or earlier idle publication.
async fn run(scheduler: Weak<PartitionMaintenance>) {
    let mut expiration_floor = None;
    loop {
        let Some(current) = scheduler.upgrade() else {
            return;
        };
        let Some(observed) = current.begin_scan() else {
            return;
        };
        let now = expiration_floor.take().map_or_else(
            || current.time_source.now(),
            |floor| current.time_source.now().max(floor),
        );
        let cells = current.cells();
        drop(current);

        for cell in &cells {
            OriginCell::expire_idle(cell, now);
        }
        let nearest = cells
            .iter()
            .filter_map(|cell| cell.nearest_idle_deadline())
            .min();
        drop(cells);

        let Some(current) = scheduler.upgrade() else {
            return;
        };
        let schedule = current.schedule(observed, nearest);
        let sleep = current.sleep.clone();
        drop(current);

        let (observed, deadline) = match schedule {
            ScheduleResult::Retry => continue,
            ScheduleResult::Shutdown => return,
            ScheduleResult::Wait { revision, deadline } => (revision, deadline),
        };
        match (deadline, sleep) {
            (Some(deadline), Some(sleep)) => {
                let now = scheduler
                    .upgrade()
                    .map(|current| current.time_source.now())
                    .unwrap_or(deadline);
                let duration = deadline.duration_since(now).unwrap_or(Duration::ZERO);
                if wait_for_sleep_or_revision(&scheduler, observed, sleep, duration).await
                    == MaintenanceWake::DeadlineElapsed
                {
                    expiration_floor = Some(deadline);
                }
            }
            _ => wait_for_revision(&scheduler, observed).await,
        }
    }
}

/// Waits for a scheduler publication when no idle deadline exists.
async fn wait_for_revision(scheduler: &Weak<PartitionMaintenance>, observed: u64) {
    poll_fn(|cx| {
        scheduler.upgrade().map_or(Poll::Ready(()), |current| {
            current.poll_revision(observed, cx)
        })
    })
    .await
}

/// The event that ended a partition maintenance wait.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MaintenanceWake {
    /// The sleep for the nearest observed deadline completed.
    DeadlineElapsed,
    /// Idle residence changed or the scheduler shut down.
    Revision,
}

/// Races one pool-provided sleep with a scheduler publication.
async fn wait_for_sleep_or_revision(
    scheduler: &Weak<PartitionMaintenance>,
    observed: u64,
    sleep: SharedAsyncSleep,
    duration: Duration,
) -> MaintenanceWake {
    let mut sleeping = Box::pin(sleep.sleep(duration));
    poll_fn(|cx| {
        if sleeping.as_mut().poll(cx).is_ready() {
            return Poll::Ready(MaintenanceWake::DeadlineElapsed);
        }
        scheduler
            .upgrade()
            .map_or(Poll::Ready(MaintenanceWake::Revision), |current| {
                current
                    .poll_revision(observed, cx)
                    .map(|()| MaintenanceWake::Revision)
            })
    })
    .await
}

#[cfg(all(test, feature = "rt-tokio", not(smithy_http_client_loom)))]
struct MaintenanceProbe {
    started: bool,
    shutdown: bool,
    scheduled_deadline: Option<SystemTime>,
}

#[cfg(all(test, smithy_http_client_loom))]
mod loom_tests {
    use super::*;
    use crate::client::pool::cell::h1::{H1CloseHandle, H1Sender};
    use crate::client::pool::connection::{CloseReason, ConnectionInfo, ConnectionState};
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::{EligibilityGroup, PartitionId};
    use crate::sync::AtomicUsize;
    use aws_smithy_async::test_util::ManualTimeSource;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use http_1x::uri::Scheme;
    use std::sync::Arc as StdArc;
    use std::task::{Context, Wake, Waker};
    use std::time::UNIX_EPOCH;

    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: StdArc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &StdArc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Races deadline publication with registration of the maintenance waker.
    ///
    /// A task that observes the old revision must either retry immediately or
    /// receive exactly one wake after it parks.
    #[test]
    fn deadline_publication_wakes_a_registered_task() {
        loom::model(|| {
            let maintenance = PartitionMaintenance::new(MaintenanceConfig::default());
            let observed = maintenance
                .begin_scan()
                .expect("new maintenance scheduler was shut down");
            let counter = StdArc::new(WakeCounter(AtomicUsize::new(0)));
            let waker = Waker::from(counter.clone());
            let mut context = Context::from_waker(&waker);
            let publisher = maintenance.clone();
            let publish = loom::thread::spawn(move || {
                publisher.notify_deadline(Some(UNIX_EPOCH + Duration::from_secs(1)));
            });

            let before_join = maintenance.poll_revision(observed, &mut context);
            publish.join().unwrap();
            if before_join.is_pending() {
                assert_eq!(1, counter.0.load(Ordering::SeqCst));
            }
            assert!(maintenance.poll_revision(observed, &mut context).is_ready());
        });
    }

    /// Publishes an earlier maintenance deadline while a scan is being committed.
    ///
    /// The scan must retry or retain a revision that forces the next poll to
    /// observe the concurrent publication.
    #[test]
    fn deadline_published_during_a_scan_forces_retry_or_wake() {
        loom::model(|| {
            let maintenance = PartitionMaintenance::new(MaintenanceConfig::default());
            let elapsed = UNIX_EPOCH + Duration::from_secs(1);
            let later = UNIX_EPOCH + Duration::from_secs(2);
            maintenance.notify_deadline(Some(elapsed));
            let observed = maintenance
                .begin_scan()
                .expect("new maintenance scheduler was shut down");
            assert_eq!(None, maintenance.state.lock().scheduled_deadline);

            let publisher = maintenance.clone();
            let publish = loom::thread::spawn(move || publisher.notify_deadline(Some(later)));
            let result = maintenance.schedule(observed, None);
            publish.join().unwrap();

            match result {
                ScheduleResult::Retry => {}
                ScheduleResult::Wait { revision, deadline } => {
                    assert_eq!(None, deadline);
                    let state = maintenance.state.lock();
                    assert_ne!(revision, state.revision);
                    assert_eq!(Some(later), state.scheduled_deadline);
                }
                ScheduleResult::Shutdown => panic!("maintenance unexpectedly shut down"),
            }
        });
    }

    /// Races shutdown with registration of the maintenance waker.
    ///
    /// A task that parks before shutdown must be woken and all later polls must
    /// observe terminal shutdown.
    #[test]
    fn shutdown_wakes_a_registered_task() {
        loom::model(|| {
            let maintenance = PartitionMaintenance::new(MaintenanceConfig::default());
            let observed = maintenance
                .begin_scan()
                .expect("new maintenance scheduler was shut down");
            let counter = StdArc::new(WakeCounter(AtomicUsize::new(0)));
            let waker = Waker::from(counter.clone());
            let mut context = Context::from_waker(&waker);
            let shutdown = maintenance.clone();
            let stop = loom::thread::spawn(move || shutdown.shutdown());

            let before_join = maintenance.poll_revision(observed, &mut context);
            stop.join().unwrap();
            if before_join.is_pending() {
                assert_eq!(1, counter.0.load(Ordering::SeqCst));
            }
            assert!(maintenance.poll_revision(observed, &mut context).is_ready());
        });
    }

    /// Races an expired idle HTTP/1 record with local sender selection.
    ///
    /// Selection either takes the sender before the scan observes it or expiry
    /// closes the exact record. The sender and record must have one final owner.
    #[test]
    fn idle_expiry_linearizes_against_h1_selection() {
        loom::model(|| {
            let maintenance = PartitionMaintenance::new(MaintenanceConfig {
                idle_timeout: Some(Duration::ZERO),
                time_source: SharedTimeSource::new(ManualTimeSource::new(UNIX_EPOCH)),
                sleep: None,
            });
            let cell = Arc::new(OriginCell::new(
                PartitionId::from_index(1),
                OriginKey::from_parts(Scheme::HTTP, "example.com", None).unwrap(),
                EligibilityGroup::Pool,
                None,
                Some(maintenance.clone()),
            ));
            maintenance.register(&cell);
            let (connection, physical) = ConnectionState::unbounded(ConnectionInfo::for_test(
                ConnectionId::new(1),
                cell.id().partition(),
            ));
            OriginCell::insert_idle_h1(&cell, connection.clone(), H1Sender::test(1));

            let selecting_cell = cell.clone();
            let selecting = loom::thread::spawn(move || OriginCell::select_h1(&selecting_cell));
            let scanning = maintenance.clone();
            let expiring = loom::thread::spawn(move || {
                let observed = scanning
                    .begin_scan()
                    .expect("maintenance shut down before its scan");
                let cells = scanning.cells();
                for cell in &cells {
                    OriginCell::expire_idle(cell, UNIX_EPOCH);
                }
                let nearest = cells
                    .iter()
                    .filter_map(|cell| cell.nearest_idle_deadline())
                    .min();
                drop(cells);
                scanning.schedule(observed, nearest)
            });

            let selected = selecting.join().unwrap();
            assert!(matches!(
                expiring.join().unwrap(),
                ScheduleResult::Wait { .. }
            ));
            drop(selected);

            if connection.probe().close_reason.is_none() {
                assert!(H1CloseHandle::new(&cell, &connection).close(CloseReason::PoolDropped));
            }
            let stats = cell.connection_stats_snapshot();
            assert_eq!(0, stats.h1().idle());
            assert_eq!(0, stats.h1().active());
            assert!(matches!(
                connection.probe().close_reason,
                Some(CloseReason::IdleTimeout | CloseReason::PoolDropped)
            ));
            drop(physical);
            maintenance.clear_modeled_cells_for_test();
        });
    }
}

#[cfg(all(test, feature = "rt-tokio", not(smithy_http_client_loom)))]
mod tests {
    use super::*;
    use crate::client::pool::cell::h1::H1Sender;
    use crate::client::pool::connection::{ConnectionInfo, ConnectionProtocol, ConnectionState};
    use crate::client::pool::origin::OriginKey;
    use crate::client::pool::partition::{DriverSpawner, EligibilityGroup, PartitionId, Spawn};
    use aws_smithy_async::test_util::controlled_time_and_sleep;
    use aws_smithy_async::time::TimeSource;
    use aws_smithy_runtime_api::client::connection::ConnectionId;
    use http_1x::uri::Scheme;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, UNIX_EPOCH};

    #[derive(Clone, Debug)]
    struct DroppingSpawner {
        submitted: Arc<AtomicU64>,
    }

    impl Spawn for DroppingSpawner {
        fn spawn(&self, driver: std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            self.submitted.fetch_add(1, Ordering::SeqCst);
            drop(driver);
        }
    }

    #[test]
    fn discarded_task_releases_the_start_latch() {
        let (maintenance, _cell, _gate) = managed_cell(Duration::from_secs(10));
        let submitted = Arc::new(AtomicU64::new(0));
        let spawner = DroppingSpawner {
            submitted: submitted.clone(),
        };

        PartitionMaintenance::start(&maintenance, &DriverSpawner::new(spawner.clone()));
        assert!(!maintenance.probe().started);
        PartitionMaintenance::start(&maintenance, &DriverSpawner::new(spawner.clone()));
        assert_eq!(2, submitted.load(Ordering::SeqCst));
        assert!(!maintenance.probe().started);
    }

    #[test]
    fn disabled_idle_timeout_submits_no_maintenance_task() {
        let maintenance = PartitionMaintenance::new(MaintenanceConfig {
            idle_timeout: None,
            ..MaintenanceConfig::default()
        });
        let submitted = Arc::new(AtomicU64::new(0));
        let spawner = DroppingSpawner {
            submitted: submitted.clone(),
        };

        PartitionMaintenance::start(&maintenance, &DriverSpawner::new(spawner.clone()));

        assert_eq!(0, submitted.load(Ordering::SeqCst));
        assert!(!maintenance.probe().started);
    }

    #[derive(Clone, Debug)]
    struct RewindableTimeSource(Arc<AtomicU64>);

    impl RewindableTimeSource {
        fn new(seconds: u64) -> Self {
            Self(Arc::new(AtomicU64::new(seconds)))
        }

        fn set(&self, seconds: u64) {
            self.0.store(seconds, Ordering::SeqCst);
        }
    }

    impl TimeSource for RewindableTimeSource {
        fn now(&self) -> SystemTime {
            UNIX_EPOCH + Duration::from_secs(self.0.load(Ordering::SeqCst))
        }
    }

    fn cell_with_maintenance(
        timeout: Duration,
        time_source: SharedTimeSource,
        sleep: SharedAsyncSleep,
    ) -> (Arc<PartitionMaintenance>, Arc<OriginCell>) {
        let maintenance = PartitionMaintenance::new(MaintenanceConfig {
            idle_timeout: Some(timeout),
            time_source,
            sleep: Some(sleep),
        });
        let origin = OriginKey::from_parts(Scheme::HTTP, "example.com", None).unwrap();
        let cell = Arc::new(OriginCell::new(
            PartitionId::from_index(1),
            origin,
            EligibilityGroup::Pool,
            None,
            Some(maintenance.clone()),
        ));
        maintenance.register(&cell);
        (maintenance, cell)
    }

    fn managed_cell(
        timeout: Duration,
    ) -> (
        Arc<PartitionMaintenance>,
        Arc<OriginCell>,
        aws_smithy_async::test_util::SleepGate,
    ) {
        let (time, sleep, gate) = controlled_time_and_sleep(UNIX_EPOCH);
        let (maintenance, cell) = cell_with_maintenance(
            timeout,
            SharedTimeSource::new(time),
            SharedAsyncSleep::new(sleep),
        );
        (maintenance, cell, gate)
    }

    fn connection(id: u64) -> Arc<ConnectionState> {
        let info = ConnectionInfo::new(
            ConnectionId::new(id),
            OriginKey::from_parts(Scheme::HTTP, "example.com", None).unwrap(),
            PartitionId::from_index(1),
            ConnectionProtocol::Http1,
            hyper_util::client::legacy::connect::Connected::new(),
        );
        let (connection, _physical) = ConnectionState::unbounded(info);
        connection
    }

    fn h2_connection(id: u64) -> Arc<ConnectionState> {
        let info = ConnectionInfo::new(
            ConnectionId::new(id),
            OriginKey::from_parts(Scheme::HTTP, "example.com", None).unwrap(),
            PartitionId::from_index(1),
            ConnectionProtocol::Http2,
            hyper_util::client::legacy::connect::Connected::new(),
        );
        let (connection, _physical) = ConnectionState::unbounded(info);
        connection
    }

    #[derive(Clone, Debug)]
    struct TrackingSpawner {
        submitted: Arc<AtomicU64>,
        active: Arc<AtomicU64>,
    }

    impl TrackingSpawner {
        fn new() -> Self {
            Self {
                submitted: Arc::new(AtomicU64::new(0)),
                active: Arc::new(AtomicU64::new(0)),
            }
        }
    }

    impl Spawn for TrackingSpawner {
        fn spawn(&self, driver: std::pin::Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
            self.submitted.fetch_add(1, Ordering::SeqCst);
            let active = self.active.clone();
            active.fetch_add(1, Ordering::SeqCst);
            tokio::spawn(async move {
                struct ActiveGuard(Arc<AtomicU64>);
                impl Drop for ActiveGuard {
                    fn drop(&mut self) {
                        self.0.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                let _guard = ActiveGuard(active);
                driver.await;
            });
        }
    }

    #[tokio::test]
    async fn start_is_once_and_shutdown_terminates_the_task() {
        let (maintenance, _cell, _gate) = managed_cell(Duration::from_secs(10));
        let spawner = TrackingSpawner::new();

        for _ in 0..10 {
            PartitionMaintenance::start(&maintenance, &DriverSpawner::new(spawner.clone()));
        }
        for _ in 0..10 {
            if spawner.active.load(Ordering::SeqCst) == 1 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(1, spawner.submitted.load(Ordering::SeqCst));
        assert_eq!(1, spawner.active.load(Ordering::SeqCst));
        assert!(maintenance.probe().started);

        maintenance.shutdown();
        for _ in 0..10 {
            if spawner.active.load(Ordering::SeqCst) == 0 {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert!(maintenance.probe().shutdown);
        assert_eq!(0, spawner.active.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn idle_insertion_wakes_a_task_with_no_scheduled_deadline() {
        let timeout = Duration::from_secs(10);
        let (maintenance, cell, mut gate) = managed_cell(timeout);
        PartitionMaintenance::start(
            &maintenance,
            &DriverSpawner::tokio(tokio::runtime::Handle::current()),
        );
        tokio::task::yield_now().await;
        assert_eq!(None, maintenance.probe().scheduled_deadline);

        OriginCell::insert_idle_h1(&cell, connection(1), H1Sender::test(1));
        let sleep = gate.expect_sleep().await;
        assert_eq!(timeout, sleep.duration());
        assert!(maintenance.probe().scheduled_deadline.is_some());
        maintenance.shutdown();
    }

    #[test]
    fn only_an_earlier_deadline_advances_the_revision() {
        let (time, sleep, _gate) = controlled_time_and_sleep(UNIX_EPOCH);
        let maintenance = PartitionMaintenance::new(MaintenanceConfig {
            idle_timeout: Some(Duration::from_secs(10)),
            time_source: SharedTimeSource::new(time),
            sleep: Some(SharedAsyncSleep::new(sleep)),
        });
        let later = UNIX_EPOCH + Duration::from_secs(20);
        let latest = UNIX_EPOCH + Duration::from_secs(30);
        let earlier = UNIX_EPOCH + Duration::from_secs(10);

        maintenance.notify_deadline(Some(later));
        let revision = maintenance.state.lock().revision;
        maintenance.notify_deadline(Some(latest));
        assert_eq!(revision, maintenance.state.lock().revision);
        maintenance.notify_deadline(Some(earlier));
        assert!(maintenance.state.lock().revision > revision);
        assert_eq!(Some(earlier), maintenance.probe().scheduled_deadline);
    }

    #[tokio::test]
    async fn idle_record_closes_at_its_fake_time_deadline() {
        let timeout = Duration::from_secs(10);
        let (maintenance, cell, mut gate) = managed_cell(timeout);
        let connection = connection(1);
        OriginCell::insert_idle_h1(&cell, connection.clone(), H1Sender::test(1));
        PartitionMaintenance::start(
            &maintenance,
            &DriverSpawner::tokio(tokio::runtime::Handle::current()),
        );

        let sleep = gate.expect_sleep().await;
        assert_eq!(timeout, sleep.duration());
        assert_eq!(None, connection.probe().close_reason);
        sleep.allow_progress();

        for _ in 0..10 {
            if connection.probe().close_reason == Some(super::super::CloseReason::IdleTimeout) {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(
            Some(super::super::CloseReason::IdleTimeout),
            connection.probe().close_reason
        );
    }
    #[tokio::test]
    async fn h2_generation_closes_at_its_fake_time_deadline() {
        let timeout = Duration::from_secs(10);
        let (maintenance, cell, mut gate) = managed_cell(timeout);
        let connection = h2_connection(1);
        OriginCell::install_h2_for_test(&cell, connection.clone(), 1, maintenance.idle_deadline());
        PartitionMaintenance::start(
            &maintenance,
            &DriverSpawner::tokio(tokio::runtime::Handle::current()),
        );

        let sleep = gate.expect_sleep().await;
        assert_eq!(timeout, sleep.duration());
        assert_eq!(None, connection.probe().close_reason);
        sleep.allow_progress();

        for _ in 0..10 {
            if connection.probe().close_reason == Some(super::super::CloseReason::IdleTimeout) {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(
            Some(super::super::CloseReason::IdleTimeout),
            connection.probe().close_reason
        );
        assert_eq!(None, cell.accepting_h2_generation());
    }

    #[test]
    fn accepted_h2_dispatch_resets_the_idle_deadline() {
        let timeout = Duration::from_secs(10);
        let time = RewindableTimeSource::new(0);
        let (_unused_time, sleep, _gate) = controlled_time_and_sleep(UNIX_EPOCH);
        let (maintenance, cell) = cell_with_maintenance(
            timeout,
            SharedTimeSource::new(time.clone()),
            SharedAsyncSleep::new(sleep),
        );
        let connection = h2_connection(1);
        OriginCell::install_h2_for_test(&cell, connection.clone(), 1, maintenance.idle_deadline());
        assert_eq!(Some(UNIX_EPOCH + timeout), cell.nearest_idle_deadline());

        time.set(20);
        let mut activation =
            OriginCell::select_h2(&cell).expect("HTTP/2 generation was not selectable");
        let _dispatch_parts = activation.take_dispatch_parts();
        let dispatch = ConnectionState::try_commit_dispatch(&connection)
            .expect("HTTP/2 connection rejected dispatch");
        activation.accept(dispatch);

        assert_eq!(
            Some(UNIX_EPOCH + Duration::from_secs(20) + timeout),
            cell.nearest_idle_deadline()
        );
    }

    #[tokio::test]
    async fn selected_record_has_no_idle_deadline() {
        let timeout = Duration::from_secs(10);
        let (maintenance, cell, mut gate) = managed_cell(timeout);
        let connection = connection(1);
        OriginCell::insert_idle_h1(&cell, connection.clone(), H1Sender::test(1));
        PartitionMaintenance::start(
            &maintenance,
            &DriverSpawner::tokio(tokio::runtime::Handle::current()),
        );

        let sleep = gate.expect_sleep().await;
        let selection = OriginCell::select_h1(&cell).expect("idle H1 was not selected");
        sleep.allow_progress();
        for _ in 0..10 {
            tokio::task::yield_now().await;
        }

        assert_eq!(None, connection.probe().close_reason);
        drop(selection);
    }

    #[tokio::test]
    async fn completed_sleep_is_an_expiration_floor_after_clock_moves_backward() {
        let timeout = Duration::from_secs(10);
        let clock = RewindableTimeSource::new(100);
        let (_unused_time, sleep, mut gate) = controlled_time_and_sleep(UNIX_EPOCH);
        let (maintenance, cell) = cell_with_maintenance(
            timeout,
            SharedTimeSource::new(clock.clone()),
            SharedAsyncSleep::new(sleep),
        );
        let connection = connection(1);
        OriginCell::insert_idle_h1(&cell, connection.clone(), H1Sender::test(1));
        PartitionMaintenance::start(
            &maintenance,
            &DriverSpawner::tokio(tokio::runtime::Handle::current()),
        );

        let sleep = gate.expect_sleep().await;
        assert_eq!(timeout, sleep.duration());
        clock.set(90);
        sleep.allow_progress();

        for _ in 0..10 {
            if connection.probe().close_reason == Some(super::super::CloseReason::IdleTimeout) {
                break;
            }
            tokio::task::yield_now().await;
        }
        assert_eq!(
            Some(super::super::CloseReason::IdleTimeout),
            connection.probe().close_reason
        );
    }
}
