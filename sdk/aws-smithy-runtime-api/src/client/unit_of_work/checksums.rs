/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Events that a generated client's response checksum validation reports to observers.
//!
//! Observers are registered in the config bag under [`SharedResponseChecksumObserver`]. Because
//! these types are in a 1.x crate, a client and an observer built against different releases of
//! the crates around them still find each other.

use aws_smithy_types::config_bag::{Storable, StoreAppend};
use std::fmt;
use std::sync::Arc;

/// Receives the response checksum validation events of one operation.
///
/// A generated client's response checksum validation calls [`on_event`](Self::on_event). Each
/// attempt that reaches the checksum decision produces either a
/// [`NotValidated`](ChecksumEventKind::NotValidated) event, or a
/// [`Validating`](ChecksumEventKind::Validating) event followed by one ending per wrapped body:
/// [`Matched`](ChecksumEventKind::Matched), [`Mismatched`](ChecksumEventKind::Mismatched),
/// [`Errored`](ChecksumEventKind::Errored) or [`Dropped`](ChecksumEventKind::Dropped). Events are
/// sent on the task that drives the attempt or reads the body, and `on_event` must not block.
pub trait ObserveResponseChecksum: Send + Sync + fmt::Debug {
    /// Receives one event.
    fn on_event(&self, event: &ChecksumEvent<'_>);
}

/// One response checksum validation event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChecksumEvent<'a> {
    attempt: u32,
    kind: ChecksumEventKind<'a>,
}

impl<'a> ChecksumEvent<'a> {
    /// Creates an event that happened during `attempt`, numbered from 1.
    pub fn new(attempt: u32, kind: ChecksumEventKind<'a>) -> Self {
        Self { attempt, kind }
    }

    /// The attempt the event happened during, numbered from 1.
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// What happened.
    pub fn kind(&self) -> &ChecksumEventKind<'a> {
        &self.kind
    }
}

/// What happened in a [`ChecksumEvent`]. Later releases may add kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChecksumEventKind<'a> {
    /// The response will not be validated.
    NotValidated(NotValidated),
    /// A validator was attached to the response body.
    Validating(Validating),
    /// The body's data ended and its checksum matched.
    Matched(Match<'a>),
    /// The body's data ended and its checksum did not match. This is sent before the body
    /// yields its mismatch error.
    Mismatched(Mismatch<'a>),
    /// The body yielded an error before its data ended.
    Errored,
    /// The body was dropped before its data ended.
    Dropped,
}

/// The data of a [`ChecksumEventKind::NotValidated`] event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotValidated {
    reason: ChecksumSkip,
}

impl NotValidated {
    /// Creates the data for a response that will not be validated because of `reason`.
    pub fn new(reason: ChecksumSkip) -> Self {
        Self { reason }
    }

    /// Why the response will not be validated.
    pub fn reason(&self) -> ChecksumSkip {
        self.reason
    }
}

/// The data of a [`ChecksumEventKind::Validating`] event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Validating {
    algorithm: &'static str,
}

impl Validating {
    /// Creates the data for a body that is validated with `algorithm`.
    pub fn new(algorithm: &'static str) -> Self {
        Self { algorithm }
    }

    /// The lowercase name of the checksum algorithm: `"crc32"`, `"crc32c"`, `"crc64nvme"`,
    /// `"sha1"` or `"sha256"` today. Later releases may add names.
    pub fn algorithm(&self) -> &'static str {
        self.algorithm
    }
}

/// The data of a [`ChecksumEventKind::Matched`] event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match<'a> {
    computed: &'a str,
}

impl<'a> Match<'a> {
    /// Creates the data for a body whose checksum matched. `computed` is base64.
    pub fn new(computed: &'a str) -> Self {
        Self { computed }
    }

    /// The checksum computed from the body, in base64.
    pub fn computed(&self) -> &'a str {
        self.computed
    }
}

/// The data of a [`ChecksumEventKind::Mismatched`] event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mismatch<'a> {
    expected: &'a str,
    computed: &'a str,
}

impl<'a> Mismatch<'a> {
    /// Creates the data for a body whose checksum did not match. Both values are base64.
    pub fn new(expected: &'a str, computed: &'a str) -> Self {
        Self { expected, computed }
    }

    /// The checksum from the response header, in base64.
    pub fn expected(&self) -> &'a str {
        self.expected
    }

    /// The checksum computed from the body, in base64.
    pub fn computed(&self) -> &'a str {
        self.computed
    }
}

/// Why a response's checksum will not be validated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ChecksumSkip {
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
}

/// A shared [`ObserveResponseChecksum`], and the config-bag key that observers are registered
/// under.
///
/// The key appends, so every registered observer receives every event.
#[derive(Clone, Debug)]
pub struct SharedResponseChecksumObserver(Arc<dyn ObserveResponseChecksum>);

impl SharedResponseChecksumObserver {
    /// Creates a new [`SharedResponseChecksumObserver`] from an observer.
    pub fn new(observer: impl ObserveResponseChecksum + 'static) -> Self {
        Self(Arc::new(observer))
    }
}

impl ObserveResponseChecksum for SharedResponseChecksumObserver {
    fn on_event(&self, event: &ChecksumEvent<'_>) {
        self.0.on_event(event)
    }
}

impl Storable for SharedResponseChecksumObserver {
    type Storer = StoreAppend<Self>;
}

#[cfg(test)]
mod tests {
    use super::{
        ChecksumEvent, ChecksumEventKind, ChecksumSkip, Match, Mismatch, NotValidated,
        ObserveResponseChecksum, SharedResponseChecksumObserver, Validating,
    };
    use aws_smithy_types::config_bag::{ConfigBag, Layer};
    use std::sync::{Arc, Mutex};

    /// Records the events it receives.
    #[derive(Clone, Debug, Default)]
    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl Recorder {
        fn record(&self, event: impl Into<String>) {
            self.0.lock().unwrap().push(event.into());
        }

        fn events(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }
    }

    impl ObserveResponseChecksum for Recorder {
        fn on_event(&self, event: &ChecksumEvent<'_>) {
            let kind = match event.kind() {
                ChecksumEventKind::NotValidated(data) => {
                    format!("not_validated {:?}", data.reason())
                }
                ChecksumEventKind::Validating(data) => format!("validating {}", data.algorithm()),
                ChecksumEventKind::Matched(data) => format!("matched {}", data.computed()),
                ChecksumEventKind::Mismatched(data) => {
                    format!("mismatched {} {}", data.expected(), data.computed())
                }
                ChecksumEventKind::Errored => "errored".to_string(),
                ChecksumEventKind::Dropped => "dropped".to_string(),
            };
            self.record(format!("{} {kind}", event.attempt()));
        }
    }

    #[test]
    fn every_appended_observer_receives_every_event() {
        let (first, second) = (Recorder::default(), Recorder::default());
        let mut layer = Layer::new("test");
        layer.store_append(SharedResponseChecksumObserver::new(first.clone()));
        layer.store_append(SharedResponseChecksumObserver::new(second.clone()));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let kinds = [
            ChecksumEventKind::Validating(Validating::new("crc32")),
            ChecksumEventKind::NotValidated(NotValidated::new(ChecksumSkip::NoChecksum)),
            ChecksumEventKind::Matched(Match::new("computed")),
            ChecksumEventKind::Mismatched(Mismatch::new("expected", "computed")),
            ChecksumEventKind::Errored,
            ChecksumEventKind::Dropped,
        ];
        for observer in cfg.load::<SharedResponseChecksumObserver>() {
            for kind in &kinds {
                observer.on_event(&ChecksumEvent::new(2, kind.clone()));
            }
        }

        let expected = [
            "2 validating crc32",
            "2 not_validated NoChecksum",
            "2 matched computed",
            "2 mismatched expected computed",
            "2 errored",
            "2 dropped",
        ];
        assert_eq!(first.events(), expected);
        assert_eq!(second.events(), expected);
    }
}
