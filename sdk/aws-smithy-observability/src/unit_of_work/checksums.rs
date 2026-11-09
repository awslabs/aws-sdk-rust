/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Records whether a call's response body was validated against its checksum.
//!
//! The SDK reports to a [`ChecksumCapture`] through
//! [`aws_smithy_runtime_api::client::unit_of_work::checksums`], whose types are stable, so a
//! capture and a client built against different releases of this crate still find each other.

use super::{sealed, AsHandle, Capture};
use aws_smithy_runtime_api::client::unit_of_work::checksums::{
    ChecksumEvent, ChecksumEventKind, ChecksumSkip, ObserveResponseChecksum,
    SharedResponseChecksumObserver,
};
use aws_smithy_types::config_bag::{ConfigBag, Storable, StoreAppend};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Records whether one call's response body was validated against its checksum.
///
/// Create one for each call and attach it with [`Capture::with`]. Clones share one record.
#[derive(Clone, Debug, Default)]
pub struct ChecksumCapture {
    slot: Arc<Mutex<ChecksumSlot>>,
}

impl ChecksumCapture {
    /// Creates a handle that is not attached to any call.
    pub fn new() -> Self {
        Self::default()
    }

    /// Where the record stands now. Never waits.
    pub fn state(&self) -> ChecksumState {
        self.lock().state()
    }

    /// The outcome, once it is complete. Never waits.
    ///
    /// The outcome is complete once the call has ended and, if the response body is validated,
    /// once that body has ended too. For a validated streaming response, that is after the body
    /// is read to its end or dropped.
    pub fn try_outcome(&self) -> Option<ChecksumOutcome> {
        match self.state() {
            ChecksumState::Complete(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub(super) fn bind(&self, cfg: &mut ConfigBag) {
        // The same handle can reach one call twice, through one `Capture` or two. It records
        // that call once.
        let already_bound = cfg
            .load::<CompleteOnDrop>()
            .any(|guard| Arc::ptr_eq(&guard.0, &self.slot));
        if !already_bound && self.lock().bind() {
            cfg.interceptor_state()
                .store_append(SharedResponseChecksumObserver::new(SlotObserver(
                    self.slot.clone(),
                )))
                .store_append(CompleteOnDrop(self.slot.clone()));
        }
    }

    pub(super) fn attempt_started(&self) {
        self.lock().attempt_started();
    }

    pub(super) fn call_ended(&self, succeeded: bool) {
        self.lock().call_ended(succeeded);
    }

    pub(super) fn capture_dropped(&self) {
        self.lock().capture_dropped();
    }

    fn lock(&self) -> MutexGuard<'_, ChecksumSlot> {
        lock(&self.slot)
    }
}

impl AsHandle for ChecksumCapture {}

impl sealed::Sealed for ChecksumCapture {
    fn add_to(&self, capture: &mut Capture) {
        capture.checksums.push(self.clone());
    }
}

/// Where a [`ChecksumCapture`]'s record stands.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChecksumState {
    /// The handle has not been attached to a call that ran.
    NotInstalled,
    /// The call is running, and no response body is being validated.
    AwaitingResponse,
    /// A response body is being validated.
    Validating {
        /// The algorithm the body is validated with.
        algorithm: ResponseChecksumAlgorithm,
    },
    /// The outcome is final.
    Complete(ChecksumOutcome),
}

/// Whether a call's response body was validated against its checksum.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChecksumOutcome {
    /// The body was read to its end, and its checksum matched.
    Validated {
        /// The algorithm the body was validated with.
        algorithm: ResponseChecksumAlgorithm,
        /// The checksum computed from the body, in base64.
        computed: String,
    },
    /// The body was read to its end, and its checksum did not match.
    Mismatch {
        /// The algorithm the body was validated with.
        algorithm: ResponseChecksumAlgorithm,
        /// The checksum from the response header, in base64.
        expected: String,
        /// The checksum computed from the body, in base64.
        computed: String,
    },
    /// The body was not validated.
    NotValidated {
        /// Why the body was not validated.
        reason: NotValidatedReason,
    },
    /// Validation started, but the body did not reach its end.
    Incomplete {
        /// The algorithm the body was being validated with.
        algorithm: ResponseChecksumAlgorithm,
        /// Why the body did not reach its end.
        cause: IncompleteCause,
    },
}

/// Why a response body was not validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotValidatedReason {
    /// The request did not enable checksum mode, and the client's `response_checksum_validation`
    /// is `WhenRequired`.
    Disabled,
    /// The response has no checksum header for an algorithm the operation lists.
    NoChecksum,
    /// The checksum header holds a part-level value, which ends in `-N`.
    PartLevelChecksum,
    /// The checksum header is present but is not valid base64.
    InvalidChecksumHeader,
    /// The response's only checksum headers are for algorithms the client does not compute.
    UnsupportedAlgorithm,
    /// The response's status is not 2xx, such as a 404 or a 503. Its body is never validated.
    ErrorResponse,
    /// The call succeeded, but nothing decided whether to validate its response, for example
    /// because the operation does not validate response checksums.
    NotApplicable,
    /// The call failed or was cancelled before a response was validated.
    NoResponse,
    /// The handle was not attached to a call that ran.
    NotInstalled,
    /// The handle was attached to more than one call.
    Reused,
    /// A reason this release does not know, reported by a newer `aws-smithy-runtime-api`.
    Other,
}

/// Why validation did not reach the end of the body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum IncompleteCause {
    /// The body was dropped before its end.
    BodyDropped,
    /// Reading the body failed before its end.
    BodyError,
}

/// A response checksum algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ResponseChecksumAlgorithm {
    /// CRC32.
    Crc32,
    /// CRC32C.
    Crc32c,
    /// CRC64NVME.
    Crc64Nvme,
    /// SHA-1.
    Sha1,
    /// SHA-256.
    Sha256,
    /// An algorithm this release does not know. Read its name with
    /// [`as_str`](ResponseChecksumAlgorithm::as_str).
    Unknown(UnknownAlgorithm),
}

/// The name of an algorithm that this release does not know.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnknownAlgorithm(&'static str);

impl ResponseChecksumAlgorithm {
    /// The algorithm's lowercase name, such as `"crc32"`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Crc32 => "crc32",
            Self::Crc32c => "crc32c",
            Self::Crc64Nvme => "crc64nvme",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Unknown(UnknownAlgorithm(name)) => name,
        }
    }

    fn from_name(name: &'static str) -> Self {
        match name {
            "crc32" => Self::Crc32,
            "crc32c" => Self::Crc32c,
            "crc64nvme" => Self::Crc64Nvme,
            "sha1" => Self::Sha1,
            "sha256" => Self::Sha256,
            other => Self::Unknown(UnknownAlgorithm(other)),
        }
    }
}

/// A handle's record. It changes only through these methods, under the handle's lock.
#[derive(Debug, Default)]
struct ChecksumSlot {
    status: Status,
}

#[derive(Debug, Default)]
enum Status {
    /// Not attached to a call that ran.
    #[default]
    NotInstalled,
    /// Attached to a call whose outcome is not final yet.
    Running(Run),
    Complete(ChecksumOutcome),
}

#[derive(Debug, Default)]
struct Run {
    /// What the producer decided for the latest attempt. Each attempt clears it, so the latest
    /// attempt wins even when it never reaches the checksum decision.
    decision: Option<Decision>,
    /// Whether the call succeeded, once it has ended.
    call_succeeded: Option<bool>,
}

#[derive(Debug)]
enum Decision {
    NotValidated(NotValidatedReason),
    Validating {
        algorithm: ResponseChecksumAlgorithm,
        ending: Option<Ending>,
    },
}

/// How a validated body ended.
#[derive(Debug)]
enum Ending {
    Dropped,
    Errored,
    Matched { computed: String },
    Mismatched { expected: String, computed: String },
}

impl Ending {
    /// `SdkBody::map` wraps each rebuilt body, so one attempt's body can end more than once. The
    /// strongest ending is kept, so a late drop never hides a mismatch.
    fn strength(&self) -> u8 {
        match self {
            Self::Dropped => 0,
            Self::Errored => 1,
            Self::Matched { .. } => 2,
            Self::Mismatched { .. } => 3,
        }
    }

    fn outcome(&self, algorithm: ResponseChecksumAlgorithm) -> ChecksumOutcome {
        match self {
            Self::Dropped => ChecksumOutcome::Incomplete {
                algorithm,
                cause: IncompleteCause::BodyDropped,
            },
            Self::Errored => ChecksumOutcome::Incomplete {
                algorithm,
                cause: IncompleteCause::BodyError,
            },
            Self::Matched { computed } => ChecksumOutcome::Validated {
                algorithm,
                computed: computed.clone(),
            },
            Self::Mismatched { expected, computed } => ChecksumOutcome::Mismatch {
                algorithm,
                expected: expected.clone(),
                computed: computed.clone(),
            },
        }
    }
}

impl ChecksumSlot {
    fn state(&self) -> ChecksumState {
        match &self.status {
            Status::NotInstalled => ChecksumState::NotInstalled,
            Status::Running(Run {
                decision: Some(Decision::Validating { algorithm, .. }),
                ..
            }) => ChecksumState::Validating {
                algorithm: *algorithm,
            },
            Status::Running(_) => ChecksumState::AwaitingResponse,
            Status::Complete(outcome) => ChecksumState::Complete(outcome.clone()),
        }
    }

    /// Attaches the handle to a call that is starting. Returns `false` if the handle already
    /// recorded another call.
    fn bind(&mut self) -> bool {
        match &self.status {
            Status::NotInstalled
            | Status::Complete(ChecksumOutcome::NotValidated {
                reason: NotValidatedReason::NotInstalled,
            }) => {
                self.status = Status::Running(Run::default());
                true
            }
            Status::Complete(ChecksumOutcome::NotValidated {
                reason: NotValidatedReason::Reused,
            }) => false,
            _ => {
                tracing::warn!(
                    "a ChecksumCapture was attached to a second call, so it now reads \
                     NotValidated {{ reason: Reused }}; attach a new ChecksumCapture to each call"
                );
                self.status = Status::Complete(ChecksumOutcome::NotValidated {
                    reason: NotValidatedReason::Reused,
                });
                false
            }
        }
    }

    fn capture_dropped(&mut self) {
        if matches!(self.status, Status::NotInstalled) {
            self.status = Status::Complete(ChecksumOutcome::NotValidated {
                reason: NotValidatedReason::NotInstalled,
            });
        }
    }

    fn attempt_started(&mut self) {
        if let Status::Running(run) = &mut self.status {
            run.decision = None;
        }
    }

    fn validating(&mut self, algorithm: ResponseChecksumAlgorithm) {
        if let Status::Running(run) = &mut self.status {
            run.decision = Some(Decision::Validating {
                algorithm,
                ending: None,
            });
        }
    }

    fn not_validated(&mut self, reason: NotValidatedReason) {
        if let Status::Running(run) = &mut self.status {
            run.decision = Some(Decision::NotValidated(reason));
        }
    }

    fn ended(&mut self, new: Ending) {
        if let Status::Running(Run {
            decision: Some(Decision::Validating { ending, .. }),
            ..
        }) = &mut self.status
        {
            if ending
                .as_ref()
                .is_none_or(|old| new.strength() > old.strength())
            {
                *ending = Some(new);
            }
            self.complete_if_final();
        }
    }

    fn call_ended(&mut self, succeeded: bool) {
        if let Status::Running(run) = &mut self.status {
            // The finalizers report first, and the config bag's guard reports again when it drops.
            run.call_succeeded.get_or_insert(succeeded);
        }
        self.complete_if_final();
    }

    /// The outcome is final once the call has ended and, if the latest attempt's body is being
    /// validated, once that body has ended too.
    fn complete_if_final(&mut self) {
        let Status::Running(Run {
            decision,
            call_succeeded: Some(succeeded),
        }) = &self.status
        else {
            return;
        };
        let outcome = match decision {
            None if *succeeded => ChecksumOutcome::NotValidated {
                reason: NotValidatedReason::NotApplicable,
            },
            None => ChecksumOutcome::NotValidated {
                reason: NotValidatedReason::NoResponse,
            },
            Some(Decision::NotValidated(reason)) => {
                ChecksumOutcome::NotValidated { reason: *reason }
            }
            Some(Decision::Validating { ending: None, .. }) => return,
            Some(Decision::Validating {
                algorithm,
                ending: Some(ending),
            }) => ending.outcome(*algorithm),
        };
        self.status = Status::Complete(outcome);
    }
}

/// Locks a record. A record is never left half-updated, so a poisoned lock is still usable.
fn lock(slot: &Mutex<ChecksumSlot>) -> MutexGuard<'_, ChecksumSlot> {
    slot.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Feeds the producer's events into one handle's record.
#[derive(Debug)]
struct SlotObserver(Arc<Mutex<ChecksumSlot>>);

impl ObserveResponseChecksum for SlotObserver {
    fn on_event(&self, event: &ChecksumEvent<'_>) {
        let mut slot = lock(&self.0);
        match event.kind() {
            ChecksumEventKind::Validating(data) => {
                slot.validating(ResponseChecksumAlgorithm::from_name(data.algorithm()))
            }
            ChecksumEventKind::NotValidated(data) => {
                slot.not_validated(not_validated_reason(data.reason()))
            }
            ChecksumEventKind::Matched(data) => slot.ended(Ending::Matched {
                computed: data.computed().to_owned(),
            }),
            ChecksumEventKind::Mismatched(data) => slot.ended(Ending::Mismatched {
                expected: data.expected().to_owned(),
                computed: data.computed().to_owned(),
            }),
            ChecksumEventKind::Errored => slot.ended(Ending::Errored),
            ChecksumEventKind::Dropped => slot.ended(Ending::Dropped),
            // A kind added by a later aws-smithy-runtime-api. Only `Matched` makes the record
            // read `Validated`, so ignoring it never reports a match.
            _ => {}
        }
    }
}

fn not_validated_reason(skip: ChecksumSkip) -> NotValidatedReason {
    match skip {
        ChecksumSkip::Disabled => NotValidatedReason::Disabled,
        ChecksumSkip::NoChecksum => NotValidatedReason::NoChecksum,
        ChecksumSkip::PartLevelChecksum => NotValidatedReason::PartLevelChecksum,
        ChecksumSkip::InvalidChecksumHeader => NotValidatedReason::InvalidChecksumHeader,
        ChecksumSkip::UnsupportedAlgorithm => NotValidatedReason::UnsupportedAlgorithm,
        ChecksumSkip::ErrorResponse => NotValidatedReason::ErrorResponse,
        // A reason added by a later aws-smithy-runtime-api.
        _ => NotValidatedReason::Other,
    }
}

/// Ends a handle's call when the call's config bag is dropped, which covers a call that is
/// cancelled before its finalizers run.
#[derive(Debug)]
struct CompleteOnDrop(Arc<Mutex<ChecksumSlot>>);

impl Drop for CompleteOnDrop {
    fn drop(&mut self) {
        lock(&self.0).call_ended(false);
    }
}

impl Storable for CompleteOnDrop {
    type Storer = StoreAppend<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_runtime_api::client::interceptors::context::{
        Input, InterceptorContext, Output,
    };
    use aws_smithy_runtime_api::client::interceptors::Intercept;
    use aws_smithy_runtime_api::client::orchestrator::OrchestratorError;
    use aws_smithy_runtime_api::client::runtime_components::RuntimeComponentsBuilder;
    use aws_smithy_runtime_api::client::unit_of_work::checksums::{
        Match, Mismatch, NotValidated, Validating,
    };
    use ResponseChecksumAlgorithm::Crc32;

    /// One call to a generated client, with its hooks run the way the orchestrator runs them.
    struct Call {
        context: InterceptorContext,
        cfg: ConfigBag,
    }

    impl Call {
        fn start(captures: &[&Capture]) -> Self {
            let mut call = Self {
                context: InterceptorContext::new(Input::doesnt_matter()),
                cfg: ConfigBag::base(),
            };
            for capture in captures {
                capture
                    .read_before_execution(&(&call.context).into(), &mut call.cfg)
                    .unwrap();
            }
            call
        }

        fn attempt(&mut self, captures: &[&Capture]) {
            let components = RuntimeComponentsBuilder::for_tests().build().unwrap();
            for capture in captures {
                capture
                    .read_before_attempt(&(&self.context).into(), &components, &mut self.cfg)
                    .unwrap();
            }
        }

        /// The observers the SDK's response-checksum producer finds in this call's config bag.
        fn observers(&self) -> Vec<SharedResponseChecksumObserver> {
            self.cfg
                .load::<SharedResponseChecksumObserver>()
                .cloned()
                .collect()
        }

        /// Reports a validator, as the producer does, and returns the observers that the
        /// body's completion callback holds.
        fn validating(&self, algorithm: &'static str) -> Vec<SharedResponseChecksumObserver> {
            let observers = self.observers();
            send(
                &observers,
                ChecksumEventKind::Validating(Validating::new(algorithm)),
            );
            observers
        }

        fn not_validated(&self, skip: ChecksumSkip) {
            send(
                &self.observers(),
                ChecksumEventKind::NotValidated(NotValidated::new(skip)),
            );
        }

        fn end(&mut self, captures: &[&Capture], succeeded: bool) {
            let components = RuntimeComponentsBuilder::for_tests().build().unwrap();
            self.context.set_output_or_error(if succeeded {
                Ok(Output::doesnt_matter())
            } else {
                Err(OrchestratorError::other("the call failed"))
            });
            for capture in captures {
                capture
                    .read_after_execution(&(&self.context).into(), &components, &mut self.cfg)
                    .unwrap();
            }
        }
    }

    /// How a test body ends.
    #[derive(Clone, Copy, Debug)]
    enum End {
        Drop,
        Error,
        Match,
        Mismatch,
    }

    impl End {
        fn report(self, body: &[SharedResponseChecksumObserver]) {
            let kind = match self {
                End::Drop => ChecksumEventKind::Dropped,
                End::Error => ChecksumEventKind::Errored,
                End::Match => ChecksumEventKind::Matched(Match::new("computed==")),
                End::Mismatch => {
                    ChecksumEventKind::Mismatched(Mismatch::new("expected==", "computed=="))
                }
            };
            send(body, kind);
        }
    }

    /// Sends one event to every observer, as the SDK's response-checksum producer does.
    fn send(observers: &[SharedResponseChecksumObserver], kind: ChecksumEventKind<'_>) {
        let event = ChecksumEvent::new(1, kind);
        observers.iter().for_each(|o| o.on_event(&event));
    }

    fn handle_and_capture() -> (ChecksumCapture, Capture) {
        let checksum = ChecksumCapture::new();
        let capture = Capture::new().with(&checksum);
        (checksum, capture)
    }

    fn not_validated(reason: NotValidatedReason) -> Option<ChecksumOutcome> {
        Some(ChecksumOutcome::NotValidated { reason })
    }

    fn validated() -> Option<ChecksumOutcome> {
        Some(ChecksumOutcome::Validated {
            algorithm: Crc32,
            computed: "computed==".into(),
        })
    }

    fn mismatch() -> Option<ChecksumOutcome> {
        Some(ChecksumOutcome::Mismatch {
            algorithm: Crc32,
            expected: "expected==".into(),
            computed: "computed==".into(),
        })
    }

    #[test]
    fn a_handle_reads_not_installed_until_its_call_starts() {
        let (checksum, capture) = handle_and_capture();
        assert_eq!(checksum.state(), ChecksumState::NotInstalled);
        assert_eq!(checksum.try_outcome(), None);

        let _call = Call::start(&[&capture]);
        assert_eq!(checksum.state(), ChecksumState::AwaitingResponse);
    }

    #[test]
    fn a_capture_dropped_before_its_call_runs_reads_not_installed() {
        let (checksum, capture) = handle_and_capture();
        drop(capture);
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::NotInstalled)
        );

        // The handle never recorded a call, so a later call can still use it.
        let capture = Capture::new().with(&checksum);
        let mut call = Call::start(&[&capture]);
        call.end(&[&capture], true);
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::NotApplicable)
        );
    }

    #[test]
    fn a_streamed_body_completes_the_outcome_after_the_call_returns() {
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        let body = call.validating("crc32");
        assert_eq!(
            checksum.state(),
            ChecksumState::Validating { algorithm: Crc32 }
        );

        // The call returns, and its config bag drops, before the body is read.
        call.end(&[&capture], true);
        drop(call);
        assert_eq!(checksum.try_outcome(), None);

        End::Match.report(&body);
        assert_eq!(checksum.try_outcome(), validated());
    }

    #[test]
    fn a_body_read_inside_the_call_completes_the_outcome_when_the_call_ends() {
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        End::Match.report(&call.validating("crc32"));
        // Another attempt could still replace this one.
        assert_eq!(checksum.try_outcome(), None);

        call.end(&[&capture], true);
        assert_eq!(checksum.try_outcome(), validated());
    }

    #[test]
    fn a_mismatch_is_recorded_as_soon_as_the_body_reports_it() {
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        let body = call.validating("crc32");
        call.end(&[&capture], true);

        // The body reports the mismatch just before it yields its error.
        End::Mismatch.report(&body);
        assert_eq!(checksum.try_outcome(), mismatch());
    }

    #[test]
    fn a_body_that_stops_short_reads_incomplete() {
        for (end, cause) in [
            (End::Error, IncompleteCause::BodyError),
            (End::Drop, IncompleteCause::BodyDropped),
        ] {
            let (checksum, capture) = handle_and_capture();
            let mut call = Call::start(&[&capture]);
            call.attempt(&[&capture]);
            let body = call.validating("crc32");
            call.end(&[&capture], true);

            end.report(&body);
            assert_eq!(
                checksum.try_outcome(),
                Some(ChecksumOutcome::Incomplete {
                    algorithm: Crc32,
                    cause
                }),
                "{end:?}"
            );
        }
    }

    #[test]
    fn the_strongest_ending_of_an_attempt_wins() {
        let body_error = Some(ChecksumOutcome::Incomplete {
            algorithm: Crc32,
            cause: IncompleteCause::BodyError,
        });
        for (ends, expected) in [
            ([End::Match, End::Drop], validated()),
            ([End::Drop, End::Match], validated()),
            ([End::Error, End::Match], validated()),
            ([End::Match, End::Mismatch], mismatch()),
            ([End::Mismatch, End::Match], mismatch()),
            ([End::Drop, End::Error], body_error),
        ] {
            let (checksum, capture) = handle_and_capture();
            let mut call = Call::start(&[&capture]);
            call.attempt(&[&capture]);
            let body = call.validating("crc32");
            ends.iter().for_each(|end| end.report(&body));

            call.end(&[&capture], true);
            assert_eq!(checksum.try_outcome(), expected, "{ends:?}");
        }
    }

    #[test]
    fn each_skip_reason_reads_as_its_not_validated_reason() {
        for (skip, reason) in [
            (ChecksumSkip::Disabled, NotValidatedReason::Disabled),
            (ChecksumSkip::NoChecksum, NotValidatedReason::NoChecksum),
            (
                ChecksumSkip::PartLevelChecksum,
                NotValidatedReason::PartLevelChecksum,
            ),
            (
                ChecksumSkip::InvalidChecksumHeader,
                NotValidatedReason::InvalidChecksumHeader,
            ),
            (
                ChecksumSkip::UnsupportedAlgorithm,
                NotValidatedReason::UnsupportedAlgorithm,
            ),
            (
                ChecksumSkip::ErrorResponse,
                NotValidatedReason::ErrorResponse,
            ),
        ] {
            let (checksum, capture) = handle_and_capture();
            let mut call = Call::start(&[&capture]);
            call.attempt(&[&capture]);
            call.not_validated(skip);

            call.end(&[&capture], true);
            assert_eq!(checksum.try_outcome(), not_validated(reason), "{skip:?}");
        }
    }

    #[test]
    fn a_call_without_a_checksum_decision_reads_not_applicable_or_no_response() {
        for (succeeded, reason) in [
            (true, NotValidatedReason::NotApplicable),
            (false, NotValidatedReason::NoResponse),
        ] {
            let (checksum, capture) = handle_and_capture();
            let mut call = Call::start(&[&capture]);
            call.attempt(&[&capture]);

            call.end(&[&capture], succeeded);
            assert_eq!(checksum.try_outcome(), not_validated(reason));
        }
    }

    #[test]
    fn the_latest_attempt_wins() {
        // A mismatched attempt, then an attempt that never connects.
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        End::Mismatch.report(&call.validating("crc32"));
        call.attempt(&[&capture]);
        call.end(&[&capture], false);
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::NoResponse)
        );

        // A mismatched attempt, then one that validates.
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        End::Mismatch.report(&call.validating("crc32"));
        call.attempt(&[&capture]);
        End::Match.report(&call.validating("crc32"));
        call.end(&[&capture], true);
        assert_eq!(checksum.try_outcome(), validated());
    }

    #[test]
    fn a_dropped_call_reads_no_response_right_away() {
        let (checksum, capture) = handle_and_capture();
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);

        drop(call);
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::NoResponse)
        );
    }

    #[test]
    fn two_captures_on_one_call_each_record_it() {
        let (first, first_capture) = handle_and_capture();
        let (second, second_capture) = handle_and_capture();
        let captures = [&first_capture, &second_capture];
        let mut call = Call::start(&captures);
        call.attempt(&captures);
        let body = call.validating("crc32");
        assert_eq!(body.len(), 2, "each capture appends its own observer");

        // Dropping the bag drops both guards; neither may end its handle's call again.
        call.end(&captures, true);
        drop(call);
        End::Match.report(&body);
        for checksum in [first, second] {
            assert_eq!(checksum.try_outcome(), validated());
        }
    }

    #[test]
    fn one_capture_with_two_handles_records_the_call_in_each() {
        let (first, second) = (ChecksumCapture::new(), ChecksumCapture::new());
        let capture = Capture::new().with(&first).with(&second);
        let mut call = Call::start(&[&capture]);
        call.attempt(&[&capture]);
        let body = call.validating("crc32");
        assert_eq!(body.len(), 2, "each handle appends its own observer");

        call.end(&[&capture], true);
        End::Match.report(&body);
        for checksum in [first, second] {
            assert_eq!(checksum.try_outcome(), validated());
        }
    }

    #[test]
    fn a_handle_carried_twice_into_one_call_records_it_once() {
        let setups: [fn(&ChecksumCapture) -> Vec<Capture>; 2] = [
            |c| vec![Capture::new().with(c).with(c)],
            |c| vec![Capture::new().with(c), Capture::new().with(c)],
        ];
        for setup in setups {
            let checksum = ChecksumCapture::new();
            let captures = setup(&checksum);
            let captures: Vec<&Capture> = captures.iter().collect();
            let mut call = Call::start(&captures);
            call.attempt(&captures);
            let body = call.validating("crc32");
            assert_eq!(body.len(), 1, "the handle registers one observer");

            call.end(&captures, true);
            End::Match.report(&body);
            assert_eq!(checksum.try_outcome(), validated());
        }
    }

    #[test]
    fn a_handle_attached_to_a_second_call_reads_reused() {
        // A capture installed on the client runs for every call.
        let (checksum, capture) = handle_and_capture();
        let mut first = Call::start(&[&capture]);
        first.attempt(&[&capture]);
        let body = first.validating("crc32");

        let second = Call::start(&[&capture]);
        assert!(second.observers().is_empty());
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::Reused)
        );

        // The first call's result never becomes the handle's outcome.
        first.end(&[&capture], true);
        End::Match.report(&body);
        assert_eq!(
            checksum.try_outcome(),
            not_validated(NotValidatedReason::Reused)
        );
    }

    #[test]
    fn every_algorithm_the_sdk_validates_has_a_variant() {
        for name in aws_smithy_checksums::http::CHECKSUM_ALGORITHMS_IN_PRIORITY_ORDER {
            let sdk_name = name
                .parse::<aws_smithy_checksums::ChecksumAlgorithm>()
                .unwrap()
                .as_str();
            let algorithm = ResponseChecksumAlgorithm::from_name(sdk_name);
            assert!(
                !matches!(algorithm, ResponseChecksumAlgorithm::Unknown(_)),
                "{sdk_name} has no variant"
            );
            assert_eq!(algorithm.as_str(), sdk_name);
        }
    }

    #[test]
    fn an_algorithm_this_release_does_not_know_keeps_its_name() {
        let algorithm = ResponseChecksumAlgorithm::from_name("xxhash64");
        assert!(matches!(algorithm, ResponseChecksumAlgorithm::Unknown(_)));
        assert_eq!(algorithm.as_str(), "xxhash64");
    }
}
