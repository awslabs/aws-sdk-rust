/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Protocol-specific document coercion settings.
//!
//! [`DocumentSettings`] is implemented by codec crates (e.g.
//! `aws-smithy-json`, `aws-smithy-cbor`) to teach
//! [`DiscriminatedDocument`](super::DiscriminatedDocument) how to
//! coerce JSON-style stringly-typed values back into their native
//! Smithy variants — most importantly base64-encoded blobs and
//! string-formatted timestamps. Wire formats that have native
//! representations for these types (e.g. CBOR major type 2 for byte
//! strings, CBOR tag 1 for timestamps) leave the trait methods at
//! their defaults and let the format-aware accessors return the
//! variant directly.
//!
//! See the type-level docs on [`DiscriminatedDocument`](super::DiscriminatedDocument::as_blob)
//! for how these methods feed into the `as_blob` / `as_timestamp`
//! coercion path.

use crate::{DateTime, DocumentError, Number};

/// Protocol-specific settings used by
/// [`DiscriminatedDocument`](super::DiscriminatedDocument)'s
/// format-aware accessors.
///
/// Implementations live alongside the format-specific codec they apply
/// to. For example, the JSON codec's `JsonDocumentSettings` will
/// implement this trait to base64-decode strings into blobs and to
/// parse timestamps according to the protocol's configured
/// `@timestampFormat` default. A protocol whose wire format already
/// represents blobs and timestamps natively (CBOR, Sparrowhawk) need
/// only implement [`Self::protocol_id`] and let every coercion fall
/// through to the default `UnsupportedOperation` body.
///
/// All implementations are required to be `Debug + Send + Sync` so
/// `DiscriminatedDocument` can be cloned freely and shared across
/// threads.
pub trait DocumentSettings: std::fmt::Debug + Send + Sync {
    /// The Smithy fully-qualified shape ID of the protocol that
    /// produced this document, e.g. `"aws.protocols#restJson1"`.
    ///
    /// Returned as a string rather than a typed `ShapeId`: `ShapeId`
    /// lives in `aws-smithy-schema`, which `aws-smithy-types` cannot
    /// depend on (the dependency direction is fixed). The FQN string
    /// is what implementations need anyway — diagnostics, error
    /// messages, future protocol-swap dispatch.
    fn protocol_id(&self) -> &str;

    /// Coerces a string value to a blob.
    ///
    /// JSON-style protocols transmit blobs as base64-encoded strings
    /// and override this method to decode. Protocols with a native
    /// blob representation (CBOR, Sparrowhawk) leave this at the
    /// default, which returns
    /// [`DocumentError::UnsupportedOperation`] — those protocols
    /// produce `Document::Blob(_)` directly during deserialization,
    /// so the type-aware path on
    /// [`DiscriminatedDocument::as_blob`](super::DiscriminatedDocument::as_blob)
    /// returns the bytes without ever calling this method.
    fn coerce_string_to_blob(&self, s: &str) -> Result<Vec<u8>, DocumentError> {
        let _ = s;
        Err(DocumentError::unsupported(format!(
            "protocol {} does not support coercing a string to a blob",
            self.protocol_id()
        )))
    }

    /// Coerces a string value to a timestamp.
    ///
    /// Used by JSON-style protocols when the configured timestamp
    /// format encodes as a string (e.g. `date-time`, `http-date`).
    /// Protocols that transmit timestamps only as numbers (epoch
    /// seconds) or natively (CBOR's tag 1) leave this at the default.
    fn coerce_string_to_timestamp(&self, s: &str) -> Result<DateTime, DocumentError> {
        let _ = s;
        Err(DocumentError::unsupported(format!(
            "protocol {} does not support coercing a string to a timestamp",
            self.protocol_id()
        )))
    }

    /// Coerces a numeric value to a timestamp.
    ///
    /// Used by JSON-style protocols when the configured timestamp
    /// format is `epoch-seconds`. Protocols that transmit timestamps
    /// only as strings, or natively, leave this at the default.
    fn coerce_number_to_timestamp(&self, n: &Number) -> Result<DateTime, DocumentError> {
        let _ = n;
        Err(DocumentError::unsupported(format!(
            "protocol {} does not support coercing a number to a timestamp",
            self.protocol_id()
        )))
    }
}
