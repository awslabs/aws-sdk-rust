/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Types for carrying values selected from an operation's input through to telemetry.
//!
//! Telemetry runs at the generic layer, where the operation input is type-erased and, during
//! serialization, consumed before any result-bearing hook runs. After that point a value such as
//! the target resource identifier survives only inside the serialized request. `CapturedTelemetryAttributes`
//! is the bridge: generated code selects an input member once, before the input is consumed, and
//! writes it here into the `ConfigBag`. Any downstream interceptor — and the built-in metrics
//! implementation — can then read it via `cfg.load`.
//!
//! This type lives in `aws-smithy-types` (a stable crate) deliberately: it carries no dependency on
//! `aws-smithy-observability` and can therefore appear in stable, generated configuration without
//! leaking a 0.x type.
//!
//! It is off by default. When no input member is selected, nothing is captured and this value is
//! absent from the `ConfigBag`.

use crate::config_bag::{Storable, StoreReplace};
use std::collections::HashMap;
use std::sync::Arc;

/// A set of string-keyed values selected from an operation's input, carried through the `ConfigBag`
/// for telemetry.
///
/// Cheap to clone regardless of value length, so propagating it as the `ConfigBag` moves through
/// config-bag layers stays inexpensive.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CapturedTelemetryAttributes {
    values: HashMap<Arc<str>, Arc<str>>,
}

impl CapturedTelemetryAttributes {
    /// Creates an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a captured value under `name`, replacing any existing value for that name.
    ///
    /// Takes `impl AsRef<str>` so the public API doesn't commit to the internal storage type;
    /// values are cloned into the backing representation here.
    pub fn insert(&mut self, name: impl AsRef<str>, value: impl AsRef<str>) {
        self.values
            .insert(Arc::from(name.as_ref()), Arc::from(value.as_ref()));
    }

    /// Returns the captured value for `name`, if one was captured.
    ///
    /// This is the read path for a downstream interceptor that wants a captured value directly,
    /// e.g. `cfg.load::<CapturedTelemetryAttributes>().and_then(|a| a.get("Bucket"))`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|v| v.as_ref())
    }

    /// Iterates over the captured `(name, value)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
    }
}

impl Storable for CapturedTelemetryAttributes {
    type Storer = StoreReplace<Self>;
}

/// The operation-input member names a customer has opted in to for telemetry, split into two
/// independent policies.
///
/// Every requested member is *captured* into `CapturedTelemetryAttributes` on the config bag; the
/// two sets differ only in whether the value is also *emitted* on the built-in client metrics:
/// * **emit** — the value is captured *and* attached to the built-in client metrics as an
///   attribute. This is the common case (`emit_input_attributes`).
/// * **capture-only** — the value is captured so a custom interceptor can read it during the
///   operation, but it is *not* attached to the built-in metrics (`capture_input_attributes`).
///   This keeps a high-cardinality value out of the metric label set while still making it
///   available in-process.
///
/// The generated per-operation interceptor captures the *union* of both sets; the built-in metrics
/// implementation emits only the *emit* set. Absent unless the customer opts in, so both are a
/// no-op by default.
///
/// Names are the Smithy member names (e.g. `"Bucket"`), matched by generated code against the
/// operation's input members.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RequestedTelemetryAttributes {
    emit: Vec<Arc<str>>,
    capture_only: Vec<Arc<str>>,
}

impl RequestedTelemetryAttributes {
    /// Creates a selection whose members are both captured and emitted on the metrics.
    ///
    /// Takes `impl AsRef<str>` items so the public API doesn't commit to the internal storage
    /// type; names are cloned into the backing representation here.
    pub fn new(names: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Self {
            emit: names.into_iter().map(|n| Arc::from(n.as_ref())).collect(),
            capture_only: Vec::new(),
        }
    }

    /// Adds member names to the *emit* set (captured and attached to the built-in metrics).
    pub fn emit(&mut self, names: impl IntoIterator<Item = impl AsRef<str>>) {
        self.emit
            .extend(names.into_iter().map(|n| Arc::from(n.as_ref())));
    }

    /// Adds member names to the *capture-only* set (captured for in-process reads, not emitted on
    /// the built-in metrics).
    pub fn capture_only(&mut self, names: impl IntoIterator<Item = impl AsRef<str>>) {
        self.capture_only
            .extend(names.into_iter().map(|n| Arc::from(n.as_ref())));
    }

    /// Returns `true` if `name` should be captured (in either set).
    pub fn should_capture(&self, name: &str) -> bool {
        self.emit
            .iter()
            .chain(self.capture_only.iter())
            .any(|n| n.as_ref() == name)
    }

    /// Returns `true` if `name` should be emitted on the built-in metrics.
    pub fn should_emit(&self, name: &str) -> bool {
        self.emit.iter().any(|n| n.as_ref() == name)
    }

    /// Returns `true` if nothing is requested for capture in either set.
    pub fn is_empty(&self) -> bool {
        self.emit.is_empty() && self.capture_only.is_empty()
    }
}

impl Storable for RequestedTelemetryAttributes {
    type Storer = StoreReplace<Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut attrs = CapturedTelemetryAttributes::new();
        assert_eq!(attrs.iter().count(), 0);

        attrs.insert("bucket", "example-bucket");
        assert_eq!(attrs.get("bucket"), Some("example-bucket"));
        assert_eq!(attrs.get("missing"), None);
        assert_eq!(attrs.iter().count(), 1);
    }

    #[test]
    fn insert_replaces_existing() {
        let mut attrs = CapturedTelemetryAttributes::new();
        attrs.insert("bucket", "first");
        attrs.insert("bucket", "second");
        assert_eq!(attrs.get("bucket"), Some("second"));
        assert_eq!(attrs.iter().count(), 1);
    }

    #[test]
    fn iter_yields_all_pairs() {
        let mut attrs = CapturedTelemetryAttributes::new();
        attrs.insert("bucket", "b");
        attrs.insert("table", "t");
        let mut pairs: Vec<_> = attrs.iter().collect();
        pairs.sort();
        assert_eq!(pairs, vec![("bucket", "b"), ("table", "t")]);
    }

    #[test]
    fn emit_set_is_captured_and_emitted() {
        let requested = RequestedTelemetryAttributes::new(["Bucket", "Key"]);
        // Members in the emit set are both captured and emitted.
        assert!(requested.should_capture("Bucket"));
        assert!(requested.should_emit("Bucket"));
        assert!(requested.should_capture("Key"));
        assert!(requested.should_emit("Key"));
        assert!(!requested.should_capture("VersionId"));
        assert!(!requested.is_empty());

        let empty = RequestedTelemetryAttributes::default();
        assert!(empty.is_empty());
        assert!(!empty.should_capture("Bucket"));
    }

    #[test]
    fn capture_only_set_is_captured_but_not_emitted() {
        let mut requested = RequestedTelemetryAttributes::default();
        requested.emit(["Bucket"]);
        requested.capture_only(["Prefix"]);

        // Prefix is captured for in-process reads but must not be emitted on the metrics.
        assert!(requested.should_capture("Prefix"));
        assert!(!requested.should_emit("Prefix"));

        // Bucket stays both captured and emitted.
        assert!(requested.should_capture("Bucket"));
        assert!(requested.should_emit("Bucket"));

        assert!(!requested.is_empty());
    }
}
