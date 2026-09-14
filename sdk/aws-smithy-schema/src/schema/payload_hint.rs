/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/// Whether a structure has an `@httpPayload` member that supplies its own body
/// framing, ex: a member carrying `@httpPayload` whose target is a
/// `structure` or a `union`.
///
/// # Why this is a hint and not a fact
///
/// The HTTP binding protocol needs this answer on every request, and deriving
/// it means scanning the input schema's members. Codegen already knows the
/// answer, so it records it here and the runtime reads it instead of scanning.
///
/// [`Unknown`](PayloadHint::Unknown) is the default, and the runtime falls back
/// to scanning when it sees it. That fallback is permanent, not
/// transitional. Two callers rely on it:
///
/// - schemas constructed by hand or materialized at runtime, which have no
///   codegen step to set the hint;
/// - generated crates produced *before* codegen began emitting the hint, which
///   Cargo permits to link against a newer `aws-smithy-schema` within the same
///   major version.
///
/// So this value must never become load-bearing for correctness. Anything that
/// consumes it has to produce identical output for `Unknown` by deriving the
/// answer itself.
///
/// # Why the framing question rather than the payload's shape
///
/// The variants deliberately answer "does a payload member frame the body?"
/// rather than describing the payload's type. `@httpPayload` may also target a
/// `blob`, `string`, or `document`, and those are not grouped together by the
/// runtime: blob and string payloads bypass the codec entirely (their bytes
/// become the body verbatim), while a document payload is written through the
/// codec. A variant named for one of those groupings would either be wrong or
/// would have to change meaning if a consumer for the distinction appeared.
///
/// The enum is `#[non_exhaustive]`, so a finer classification can be added
/// later without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum PayloadHint {
    /// Not recorded. Consumers must derive the answer themselves.
    ///
    /// This is the default for every schema, including every hand-constructed
    /// one.
    #[default]
    Unknown,

    /// No member carries `@httpPayload` targeting a `structure` or `union`.
    ///
    /// Body framing, if any, comes from the codec.
    NoStructPayload,

    /// A member carries `@httpPayload` targeting a `structure` or `union`.
    ///
    /// That member writes its own framing, so the codec must not add
    /// top-level framing around it.
    StructPayload,
}
