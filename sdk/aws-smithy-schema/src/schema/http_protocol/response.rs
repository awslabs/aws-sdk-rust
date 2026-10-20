/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! The composite response deserializer that makes a REST protocol the sole owner of
//! response deserialization.
//!
//! A REST protocol returns one of these instead of a bare body deserializer. It reads
//! `@httpHeader`, `@httpPrefixHeaders`, `@httpResponseCode`, and `@httpPayload` members from
//! the HTTP message itself, delegates everything else to the protocol's body codec, and
//! drives the same generated member-index consumer either way. Body-only protocols (RPC,
//! Query, CBOR) keep returning their codec's deserializer and therefore ignore HTTP binding
//! traits, as the SEP requires.
//!
//! # Where the decisions live
//!
//! Which members are bound comes from the const-derived routing word on the structure schema,
//! so the common case visits only bound members rather than scanning. How each bound value
//! parses lives in [`super::bound_value`]. This module is the router between them, plus the
//! `NonUtf8HeaderHandling` policy and the body/transport precedence rule.

use super::binding::HTTP_HEADER_NAME;
use super::bound_value::{
    HeaderValues, HttpHeaderValueDeserializer, HttpPrefixHeadersDeserializer,
    HttpRawPayloadDeserializer, HttpStatusDeserializer,
};
use crate::serde::{SerdeError, ShapeDeserializer};
use crate::{Schema, ShapeType};
use aws_smithy_runtime_api::http::{Headers, NonUtf8HeaderHandling, Response};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::error::display::DisplayErrorContext;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document};

/// Reads the non-UTF-8 header policy out of a request's config, defaulting to
/// [`NonUtf8HeaderHandling::Reject`].
pub(crate) fn non_utf8_policy(cfg: &ConfigBag) -> NonUtf8HeaderHandling {
    cfg.load::<NonUtf8HeaderHandling>()
        .cloned()
        .unwrap_or_default()
}

/// Where the composite gets its [`NonUtf8HeaderHandling`] from.
///
/// The policy only matters once a bound header has failed to parse *and* holds an unreadable
/// value, which almost never happens. A [`ConfigBag`] load walks every layer, and doing it up
/// front was a measurable share of a small response, so the production path keeps the bag and
/// reads it only at that point — the same order the generated legacy parser used.
#[derive(Clone)]
pub(crate) enum NonUtf8Policy<'a> {
    /// Already known, so no bag needs to be kept.
    Resolved(NonUtf8HeaderHandling),
    /// Read from the request's config only when a decision is needed.
    Deferred(&'a ConfigBag),
}

impl NonUtf8Policy<'_> {
    #[cold]
    fn resolve(&self) -> NonUtf8HeaderHandling {
        match self {
            NonUtf8Policy::Resolved(policy) => policy.clone(),
            NonUtf8Policy::Deferred(cfg) => non_utf8_policy(cfg),
        }
    }
}

impl From<NonUtf8HeaderHandling> for NonUtf8Policy<'_> {
    fn from(policy: NonUtf8HeaderHandling) -> Self {
        NonUtf8Policy::Resolved(policy)
    }
}

impl<'a> From<&'a ConfigBag> for NonUtf8Policy<'a> {
    fn from(cfg: &'a ConfigBag) -> Self {
        NonUtf8Policy::Deferred(cfg)
    }
}

/// Wraps a protocol's body deserializer so a **successful** response's transport-bound members
/// are read from the HTTP message.
///
/// Every REST construction site must go through this or [`http_error_deserializer`], for two
/// reasons. It is the single place binding behavior is decided, so a protocol wrapper cannot
/// accidentally return a bare body deserializer and silently drop header members. And it is
/// generic over the concrete body deserializer, so the composite is built *before* the result
/// is coerced to `Box<dyn ShapeDeserializer>` — exactly one allocation per response, matching
/// what the body-only path already costs. Boxing the body first and wrapping the box would add
/// a second.
///
/// # The body-only fast path
///
/// When the output structure has no response-bound members but does have body members, the
/// composite has nothing to contribute and the body deserializer is returned directly. This
/// keeps body-only REST outputs off the composite entirely.
///
/// Both halves of the condition matter. An *empty* output has no body members either, and for
/// it the composite is the better answer: it skips the codec instead of asking the codec to
/// accept an empty body. A non-structure schema — such as
/// [`prelude::DOCUMENT`](crate::prelude::DOCUMENT) — carries a zeroed routing word, so it
/// reports no body members and also routes through the composite rather than being mistaken
/// for a body-only output.
pub fn http_output_deserializer<'a, D>(
    body: D,
    response: &'a Response,
    output_schema: &Schema<'_>,
    cfg: &'a ConfigBag,
) -> Box<dyn ShapeDeserializer + 'a>
where
    D: ShapeDeserializer + 'a,
{
    if is_body_only(output_schema) {
        return Box::new(body);
    }
    Box::new(HttpResponseDeserializer::for_output(
        body,
        response.headers(),
        response.status().as_u16(),
        response.body().bytes(),
        cfg,
    ))
}

/// Wraps a protocol's body deserializer so a **modeled error** response's transport-bound
/// members are read from the HTTP message.
///
/// Separate from [`http_output_deserializer`] rather than a mode flag, because the two differ
/// in ways that must not be reachable from the wrong call site:
///
/// - An error tolerates an empty body while still populating header and status members, which a
///   successful response must not do — the codec's empty-body strictness is the current
///   behavior there.
/// - There is no error schema to pass. Which error variant is being built is not known until
///   the generated variant calls `read_struct`, so the body-only fast path cannot be evaluated.
///   Taking no schema makes that structural: an error path has nothing to apply the fast path
///   to, so it cannot bypass the composite.
pub fn http_error_deserializer<'a, D>(
    body: D,
    response: &'a Response,
    cfg: &'a ConfigBag,
) -> Box<dyn ShapeDeserializer + 'a>
where
    D: ShapeDeserializer + 'a,
{
    Box::new(HttpResponseDeserializer::for_error(
        body,
        response.headers(),
        response.status().as_u16(),
        response.body().bytes(),
        cfg,
    ))
}

/// True when the schema's members all belong to the body, so the composite would only add
/// indirection.
///
/// Requires an exact routing word: a structure whose bindings sit beyond the mask reports
/// `needs_scan`, and its mask is then meaningless rather than empty.
fn is_body_only(schema: &Schema<'_>) -> bool {
    !schema.response_bindings_need_scan()
        && schema.response_binding_mask() == 0
        && schema.has_response_body_members()
}

/// Deserializes a complete HTTP response: transport-bound members plus the protocol body.
///
/// Generic over the body deserializer so a protocol can construct this with its concrete
/// codec type and box the result once. Boxing the body separately would add an allocation to
/// every response.
pub(crate) struct HttpResponseDeserializer<'a, D> {
    body: D,
    headers: &'a Headers,
    status: u16,
    /// The buffered body, or `None` when the body is still a live stream.
    ///
    /// A streaming response leaves the body unbuffered; its payload member is `@streaming` and
    /// is owned by the generated streaming path, so nothing here needs the bytes.
    body_bytes: Option<&'a [u8]>,
    non_utf8: NonUtf8Policy<'a>,
    /// Whether an empty body is acceptable instead of being handed to the codec.
    ///
    /// True only for modeled errors. Services send empty error bodies — S3's `HEAD` responses,
    /// for instance — and those responses must still populate header and status members.
    allow_empty_body: bool,
}

/// What a bound member turned out to be, so the caller knows whether the body codec still has
/// the outer structure to read.
#[derive(PartialEq, Eq)]
enum Routed {
    /// An ordinary binding: header, prefix map, or status.
    Binding,
    /// A `@httpPayload` member. The body belongs to this member, so the outer structure must
    /// not also be read from it.
    Payload,
}

impl<'a, D> HttpResponseDeserializer<'a, D>
where
    D: ShapeDeserializer,
{
    /// Builds a deserializer for a successful response.
    pub(crate) fn for_output(
        body: D,
        headers: &'a Headers,
        status: u16,
        body_bytes: Option<&'a [u8]>,
        non_utf8: impl Into<NonUtf8Policy<'a>>,
    ) -> Self {
        Self {
            body,
            headers,
            status,
            body_bytes,
            non_utf8: non_utf8.into(),
            allow_empty_body: false,
        }
    }

    /// Builds a deserializer for a modeled error response, which tolerates an empty body.
    pub(crate) fn for_error(
        body: D,
        headers: &'a Headers,
        status: u16,
        body_bytes: Option<&'a [u8]>,
        non_utf8: impl Into<NonUtf8Policy<'a>>,
    ) -> Self {
        Self {
            body,
            headers,
            status,
            body_bytes,
            non_utf8: non_utf8.into(),
            allow_empty_body: true,
        }
    }

    /// True when `Skip` applies, i.e. the policy is `Skip` *and* some raw value really is
    /// unreadable.
    ///
    /// The unreadable check is what keeps `Skip` from hiding ordinary parse failures: a
    /// malformed but valid-UTF-8 value still fails.
    fn skips(&self, has_unreadable_value: bool) -> bool {
        // Order matters: the policy is read only once a value really is unreadable.
        has_unreadable_value && self.non_utf8.resolve() == NonUtf8HeaderHandling::Skip
    }

    /// Routes one response-bound member to its source.
    #[inline]
    fn route(
        &mut self,
        member: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<Routed, SerdeError> {
        if let Some(header) = member.http_header() {
            self.route_header(member, header.value(), consumer)?;
            return Ok(Routed::Binding);
        }
        if let Some(prefix) = member.http_prefix_headers() {
            self.route_prefix(member, prefix.value(), consumer)?;
            return Ok(Routed::Binding);
        }
        if member.http_response_code().is_some() {
            // Unconditional: a response always has a status, so this member always has a
            // value. That matches the generated path, which assigns it with no guard.
            let mut status = HttpStatusDeserializer::new(self.status);
            consumer(member, &mut status)?;
            return Ok(Routed::Binding);
        }
        debug_assert!(
            member.http_payload().is_some(),
            "route is only called for response-bound members"
        );
        self.route_payload(member, consumer)?;
        Ok(Routed::Payload)
    }

    #[inline]
    fn route_header(
        &mut self,
        member: &Schema<'_>,
        name: &str,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // The name parsed once and cached on the member, shared with request
        // serialization. A name that does not parse is looked up as given, which finds
        // nothing.
        let values = match member.extension(&HTTP_HEADER_NAME) {
            Some(parsed) => HeaderValues::by_name(self.headers, parsed),
            None => HeaderValues::new(self.headers, name),
        };
        if !values.is_present() {
            // An absent header leaves the member absent. Invoking the consumer would make it
            // produce a value.
            return Ok(());
        }
        let mut value = HttpHeaderValueDeserializer::new(values);
        if value.parses_to_no_value(member) {
            // Present but yielding nothing to parse. The generated path reports this as an
            // absent member, and no `read_*` return type can express it.
            return Ok(());
        }
        match consumer(member, &mut value) {
            Ok(()) => Ok(()),
            Err(err) if self.skips(values.has_unreadable_value()) => {
                // Leave the member absent. The header itself is untouched, so an interceptor
                // can still read the original octets.
                let _ = err;
                Ok(())
            }
            Err(err) => Err(header_error(member, "header", name, err)),
        }
    }

    // Out of line so `route` stays small for the common header case.
    #[inline(never)]
    fn route_prefix(
        &mut self,
        member: &Schema<'_>,
        prefix: &str,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Invoked even when nothing matches: an absent prefix is `Some(empty_map)`, not `None`.
        let mut map = HttpPrefixHeadersDeserializer::new(self.headers, prefix);
        match consumer(member, &mut map) {
            Ok(()) => Ok(()),
            Err(err) if self.skips(map.has_unreadable_value()) => {
                // The whole member is dropped rather than exposing a partial map.
                let _ = err;
                Ok(())
            }
            Err(err) => Err(header_error(member, "prefix header", prefix, err)),
        }
    }

    #[inline(never)]
    fn route_payload(
        &mut self,
        member: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        if member.streaming() {
            // The generated streaming path owns the live body or event receiver and installs
            // it on the builder itself. Invoking the consumer here would overwrite it.
            return Ok(());
        }
        let Some(body_bytes) = self.body_bytes else {
            // Not buffered, yet not `@streaming`. Nothing can be read, and silently producing
            // an absent member would hide a protocol wiring mistake.
            return Err(SerdeError::invalid_input(format!(
                "member `{}` has a non-streaming @httpPayload but the response body was not buffered",
                member_name(member)
            )));
        };
        if body_bytes.is_empty() {
            // An empty payload leaves the member absent, for every payload kind. The generated
            // path guards each assignment with `if !body.is_empty()`.
            return Ok(());
        }
        match member.shape_type() {
            // Raw payloads bypass the codec: the body *is* the value.
            ShapeType::Blob | ShapeType::String => {
                let mut raw = HttpRawPayloadDeserializer::new(body_bytes);
                consumer(member, &mut raw)
            }
            // Everything else is a structured payload, read by the protocol's codec with the
            // body root as the member's root.
            _ => consumer(member, &mut self.body),
        }
    }
}

/// Builds the contextual error for a failed binding, naming the member and the header.
///
/// The parser's chain is already inside `err`'s message; [`DisplayErrorContext`] also picks up
/// any source [`SerdeError`] carries. Without this context a caller sees only "failed to parse
/// input as i32" with no indication of which header produced it.
#[cold]
fn header_error(member: &Schema<'_>, kind: &str, name: &str, err: SerdeError) -> SerdeError {
    SerdeError::invalid_input(format!(
        "failed to parse {} from {kind} `{name}`: {}",
        member_name(member),
        DisplayErrorContext(&err),
    ))
}

fn member_name<'d>(member: &Schema<'d>) -> &'d str {
    member.member_name().unwrap_or("<unnamed member>")
}

impl<D> ShapeDeserializer for HttpResponseDeserializer<'_, D>
where
    D: ShapeDeserializer,
{
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Transport bindings are read first, then the body. The body pass skips any field that
        // corresponds to a bound member, so the transport value is authoritative and a
        // malformed body copy cannot overwrite it.
        let mut has_payload = false;
        if schema.response_bindings_need_scan() {
            // A bound member sits beyond the mask's reach, so the mask is incomplete and must
            // be ignored entirely.
            for member in schema.members() {
                if member.has_http_response_binding()
                    && self.route(member, consumer)? == Routed::Payload
                {
                    has_payload = true;
                }
            }
        } else {
            let mut mask = schema.response_binding_mask();
            while mask != 0 {
                let index = mask.trailing_zeros() as usize;
                mask &= mask - 1;
                let member = schema
                    .member_schema_by_index(index)
                    .expect("the mask is derived from this schema's own member array");
                if self.route(member, consumer)? == Routed::Payload {
                    has_payload = true;
                }
            }
        }

        if has_payload {
            // The body belongs to the payload member; there is no outer document to read.
            return Ok(());
        }
        if !schema.has_response_body_members() {
            // Every member is transport-bound, or there are none. Invoking the codec would
            // make an empty or absent body an error for no reason.
            return Ok(());
        }
        if self.allow_empty_body && self.body_bytes.map(|b| b.is_empty()).unwrap_or(true) {
            // A modeled error with an empty body still populated its header and status
            // members above; handing an empty body to the codec would fail the response.
            return Ok(());
        }

        // Filter the body pass so a field for a transport-bound member is consumed and
        // discarded rather than assigned.
        self.body.read_struct(schema, &mut |member, deser| {
            // The index-less callback is how JSON and CBOR report an unknown union variant;
            // filtering it would hide those. Only a real modeled member can be bound.
            if member.member_index().is_some() && member.has_http_response_binding() {
                deser.skip_value()
            } else {
                consumer(member, deser)
            }
        })
    }

    // Everything below is delegation: once a member has been routed, any nested value is the
    // body codec's or a bound-value deserializer's to read. These are reached only when a
    // caller uses the composite where a non-structure was expected, which for a response means
    // the body root.

    fn read_list(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.body.read_list(schema, consumer)
    }

    fn read_map(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.body.read_map(schema, consumer)
    }

    fn read_boolean(&mut self, schema: &Schema<'_>) -> Result<bool, SerdeError> {
        self.body.read_boolean(schema)
    }

    fn read_byte(&mut self, schema: &Schema<'_>) -> Result<i8, SerdeError> {
        self.body.read_byte(schema)
    }

    fn read_short(&mut self, schema: &Schema<'_>) -> Result<i16, SerdeError> {
        self.body.read_short(schema)
    }

    fn read_integer(&mut self, schema: &Schema<'_>) -> Result<i32, SerdeError> {
        self.body.read_integer(schema)
    }

    fn read_long(&mut self, schema: &Schema<'_>) -> Result<i64, SerdeError> {
        self.body.read_long(schema)
    }

    fn read_float(&mut self, schema: &Schema<'_>) -> Result<f32, SerdeError> {
        self.body.read_float(schema)
    }

    fn read_double(&mut self, schema: &Schema<'_>) -> Result<f64, SerdeError> {
        self.body.read_double(schema)
    }

    fn read_big_integer(&mut self, schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        self.body.read_big_integer(schema)
    }

    fn read_big_decimal(&mut self, schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        self.body.read_big_decimal(schema)
    }

    fn read_string(&mut self, schema: &Schema<'_>) -> Result<String, SerdeError> {
        self.body.read_string(schema)
    }

    fn read_blob(&mut self, schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        self.body.read_blob(schema)
    }

    fn read_timestamp(&mut self, schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        self.body.read_timestamp(schema)
    }

    fn read_document(&mut self, schema: &Schema<'_>) -> Result<Document, SerdeError> {
        self.body.read_document(schema)
    }

    fn is_null(&self) -> bool {
        self.body.is_null()
    }

    fn read_null(&mut self) -> Result<(), SerdeError> {
        self.body.read_null()
    }

    fn skip_value(&mut self) -> Result<(), SerdeError> {
        self.body.skip_value()
    }

    fn container_size(&self) -> Option<usize> {
        self.body.container_size()
    }

    fn read_string_list(&mut self, schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        self.body.read_string_list(schema)
    }

    fn read_blob_list(&mut self, schema: &Schema<'_>) -> Result<Vec<Blob>, SerdeError> {
        self.body.read_blob_list(schema)
    }

    fn read_integer_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        self.body.read_integer_list(schema)
    }

    fn read_long_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        self.body.read_long_list(schema)
    }

    fn read_string_string_map(
        &mut self,
        schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        self.body.read_string_string_map(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentShapeDeserializer;
    use crate::traits::TimestampFormat;
    use crate::{shape_id, ShapeId};
    use std::cell::RefCell;
    use std::collections::HashMap;

    const ID: ShapeId<'static> = shape_id!("test", "S");

    fn headers(pairs: &[(&str, &[u8])]) -> Headers {
        let mut map = http::HeaderMap::new();
        for (name, value) in pairs {
            map.append(
                http::HeaderName::from_bytes(name.as_bytes()).expect("valid header name"),
                http::HeaderValue::from_bytes(value).expect("valid header value"),
            );
        }
        Headers::try_from(map).expect("valid headers")
    }

    /// What the generated member-index consumer would have been handed, keyed by member name.
    #[derive(Default, Debug, PartialEq)]
    struct Populated {
        strings: HashMap<String, String>,
        integers: HashMap<String, i32>,
        lists: HashMap<String, Vec<String>>,
        maps: HashMap<String, HashMap<String, String>>,
        blobs: HashMap<String, Vec<u8>>,
        timestamps: HashMap<String, i64>,
        floats: HashMap<String, f32>,
    }

    /// A consumer that dispatches on the member's modeled type, the way a generated arm does.
    ///
    /// The real generated consumer matches on `member_index()`; dispatching on shape type keeps
    /// these tests from needing a per-shape fixture while exercising the same call pattern.
    fn consume(
        out: &mut Populated,
    ) -> impl FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError> + '_ {
        move |member, deser| {
            let name = member.member_name().unwrap_or("?").to_string();
            match member.shape_type() {
                ShapeType::String => {
                    out.strings.insert(name, deser.read_string(member)?);
                }
                ShapeType::Integer => {
                    out.integers.insert(name, deser.read_integer(member)?);
                }
                ShapeType::Timestamp => {
                    out.timestamps
                        .insert(name, deser.read_timestamp(member)?.secs());
                }
                ShapeType::Blob => {
                    out.blobs
                        .insert(name, deser.read_blob(member)?.into_inner());
                }
                ShapeType::Float => {
                    out.floats.insert(name, deser.read_float(member)?);
                }
                ShapeType::List => {
                    out.lists.insert(name, deser.read_string_list(member)?);
                }
                ShapeType::Map => {
                    out.maps.insert(name, deser.read_string_string_map(member)?);
                }
                other => {
                    return Err(SerdeError::unsupported(format!(
                        "test consumer does not handle {other:?}"
                    )))
                }
            }
            Ok(())
        }
    }

    /// What the stub body observed. Recorded through a shared `RefCell` because the composite
    /// takes the body deserializer by value — there is deliberately no
    /// `impl ShapeDeserializer for &mut T`, so a test cannot hold the body aside and inspect it.
    #[derive(Default, Debug)]
    struct BodyLog {
        /// Whether `read_struct` was reached at all.
        read: bool,
        /// Members whose value the composite's filter consumed via `skip_value`, in body order.
        skipped: Vec<String>,
        /// The shape name of the schema `read_struct` was called with.
        ///
        /// For a structured `@httpPayload` this is decided by the generated arm, not by the
        /// composite, which is what makes member-level `@xmlName` root naming a codegen concern.
        root: Option<String>,
    }

    /// A body deserializer that reports a fixed set of members, so a test can put a copy of a
    /// transport-bound member in the "body" and observe what the filter does with it.
    #[derive(Debug)]
    struct StubBody<'s> {
        members: &'s [&'s Schema<'s>],
        log: &'s RefCell<BodyLog>,
    }

    impl<'s> StubBody<'s> {
        fn new(members: &'s [&'s Schema<'s>], log: &'s RefCell<BodyLog>) -> Self {
            Self { members, log }
        }
    }

    /// The marker a stub body puts in every value it produces, so a test can tell a value that
    /// came from the body apart from one that came from the transport.
    const FROM_BODY: &str = "from-body";

    impl ShapeDeserializer for StubBody<'_> {
        fn read_struct(
            &mut self,
            schema: &Schema<'_>,
            consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            {
                let mut log = self.log.borrow_mut();
                log.read = true;
                log.root = Some(schema.shape_id().shape_name().to_string());
            }
            for member in self.members {
                let mut value = StubValue {
                    name: member.member_name().unwrap_or("?").to_string(),
                    log: self.log,
                };
                consumer(member, &mut value)?;
            }
            Ok(())
        }

        fn read_list(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_map(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_boolean(&mut self, _s: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _s: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _s: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _s: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _s: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _s: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _s: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(&mut self, _s: &Schema<'_>) -> Result<BigInteger, SerdeError> {
            Err(SerdeError::unsupported("stub"))
        }
        fn read_big_decimal(&mut self, _s: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
            Err(SerdeError::unsupported("stub"))
        }
        fn read_string(&mut self, _s: &Schema<'_>) -> Result<String, SerdeError> {
            Ok(FROM_BODY.to_string())
        }
        fn read_blob(&mut self, _s: &Schema<'_>) -> Result<Blob, SerdeError> {
            Ok(Blob::new(FROM_BODY.as_bytes().to_vec()))
        }
        fn read_timestamp(&mut self, _s: &Schema<'_>) -> Result<DateTime, SerdeError> {
            Ok(DateTime::from_secs(0))
        }
        fn read_document(&mut self, _s: &Schema<'_>) -> Result<Document, SerdeError> {
            Ok(Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    /// One member's value inside [`StubBody`], which records a `skip_value` call.
    #[derive(Debug)]
    struct StubValue<'s> {
        name: String,
        log: &'s RefCell<BodyLog>,
    }

    impl ShapeDeserializer for StubValue<'_> {
        fn read_struct(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_list(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _schema: &Schema<'_>,
            _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _s: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(true)
        }
        fn read_byte(&mut self, _s: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(-9)
        }
        fn read_short(&mut self, _s: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(-9)
        }
        fn read_integer(&mut self, _s: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(-999)
        }
        fn read_long(&mut self, _s: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(-999)
        }
        fn read_float(&mut self, _s: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(-9.0)
        }
        fn read_double(&mut self, _s: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(-9.0)
        }
        fn read_big_integer(&mut self, _s: &Schema<'_>) -> Result<BigInteger, SerdeError> {
            Err(SerdeError::unsupported("stub"))
        }
        fn read_big_decimal(&mut self, _s: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
            Err(SerdeError::unsupported("stub"))
        }
        fn read_string(&mut self, _s: &Schema<'_>) -> Result<String, SerdeError> {
            Ok(FROM_BODY.to_string())
        }
        fn read_blob(&mut self, _s: &Schema<'_>) -> Result<Blob, SerdeError> {
            Ok(Blob::new(FROM_BODY.as_bytes().to_vec()))
        }
        fn read_timestamp(&mut self, _s: &Schema<'_>) -> Result<DateTime, SerdeError> {
            Ok(DateTime::from_secs(-1))
        }
        fn read_document(&mut self, _s: &Schema<'_>) -> Result<Document, SerdeError> {
            Ok(Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn skip_value(&mut self) -> Result<(), SerdeError> {
            self.log.borrow_mut().skipped.push(self.name.clone());
            Ok(())
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    // -- schemas --

    static NAME_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::String, "name", 0).with_http_header("x-name");
    static COUNT_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Integer, "count", 1).with_http_header("x-count");
    static STATUS: Schema<'static> =
        Schema::new_member(ID, ShapeType::Integer, "status", 2).with_http_response_code();
    static BODY_FIELD: Schema<'static> = Schema::new_member(ID, ShapeType::String, "note", 3);

    static MIXED: Schema<'static> = Schema::new_struct(
        ID,
        ShapeType::Structure,
        &[&NAME_HEADER, &COUNT_HEADER, &STATUS, &BODY_FIELD],
    );

    /// Every member is transport-bound, so the body codec has nothing to read.
    static BOUND_NAME: Schema<'static> =
        Schema::new_member(ID, ShapeType::String, "name", 0).with_http_header("x-name");
    static BOUND_STATUS: Schema<'static> =
        Schema::new_member(ID, ShapeType::Integer, "status", 1).with_http_response_code();
    static FULLY_BOUND: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&BOUND_NAME, &BOUND_STATUS]);

    static EMPTY_STRUCT: Schema<'static> = Schema::new_struct(ID, ShapeType::Structure, &[]);

    static BODY_ONLY_FIELD: Schema<'static> = Schema::new_member(ID, ShapeType::String, "note", 0);
    static BODY_ONLY: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&BODY_ONLY_FIELD]);

    /// The success-path composite over a stub body, with the default Reject policy.
    fn output<'a>(
        members: &'a [&'a Schema<'a>],
        log: &'a RefCell<BodyLog>,
        h: &'a Headers,
        status: u16,
        body_bytes: Option<&'a [u8]>,
    ) -> HttpResponseDeserializer<'a, StubBody<'a>> {
        HttpResponseDeserializer::for_output(
            StubBody::new(members, log),
            h,
            status,
            body_bytes,
            NonUtf8HeaderHandling::Reject,
        )
    }

    const NO_BODY_MEMBERS: &[&Schema<'static>] = &[];

    // ---------------------------------------------------------------------------------
    // routing
    // ---------------------------------------------------------------------------------

    #[test]
    fn headers_status_and_body_members_all_reach_the_consumer() {
        let h = headers(&[("x-name", b"widget"), ("x-count", b"7")]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 201, Some(b"{}"))
            .read_struct(&MIXED, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
        assert_eq!(out.integers.get("count"), Some(&7));
        assert_eq!(out.integers.get("status"), Some(&201));
        assert_eq!(out.strings.get("note"), Some(&FROM_BODY.to_string()));
        assert!(
            log.borrow().read,
            "the body codec must be invoked for body members"
        );
    }

    #[test]
    fn header_members_are_found_by_their_cached_names() {
        // A mixed-case modeled name, a repeated header (re-read by name) and a modeled name
        // that is not a valid header name, which has no cached name and finds nothing.
        static MIXED_CASE: Schema<'static> =
            Schema::new_member(ID, ShapeType::String, "md5", 0).with_http_header("Content-MD5");
        static REPEATED: Schema<'static> = Schema::new_member(ID, ShapeType::List, "tags", 1)
            .with_http_header("x-tags")
            .with_list_member(&crate::prelude::STRING);
        static INVALID: Schema<'static> =
            Schema::new_member(ID, ShapeType::String, "bad", 2).with_http_header("bad name");
        static NAMED: Schema<'static> = Schema::new_struct(
            ID,
            ShapeType::Structure,
            &[&MIXED_CASE, &REPEATED, &INVALID],
        );
        assert!(MIXED_CASE.extension(&HTTP_HEADER_NAME).is_some());
        assert!(INVALID.extension(&HTTP_HEADER_NAME).is_none());

        let h = headers(&[
            ("content-md5", b"abc=="),
            ("x-tags", b"a"),
            ("x-tags", b"b, c"),
        ]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&NAMED, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.get("md5"), Some(&"abc==".to_string()));
        assert_eq!(
            out.lists.get("tags"),
            Some(&vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        assert!(!out.strings.contains_key("bad"));
    }

    #[test]
    fn an_absent_header_leaves_its_member_absent() {
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&FULLY_BOUND, &mut consume(&mut out))
            .unwrap();
        assert!(
            !out.strings.contains_key("name"),
            "the consumer must not run for an absent header"
        );
        assert_eq!(out.integers.get("status"), Some(&200));
    }

    #[test]
    fn a_present_but_valueless_header_leaves_its_member_absent() {
        // The generated path reports an empty value as an absent member for everything except a
        // plain string, and no `read_*` return type can express that.
        let h = headers(&[("x-name", b""), ("x-count", b"")]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 200, Some(b"{}"))
            .read_struct(&MIXED, &mut consume(&mut out))
            .unwrap();
        assert!(
            !out.integers.contains_key("count"),
            "an empty integer header must not become Some(0)"
        );
        assert_eq!(
            out.strings.get("name"),
            Some(&String::new()),
            "an empty plain-string header is the empty string, not absent"
        );
    }

    #[test]
    fn the_body_codec_is_skipped_when_no_member_belongs_to_it() {
        let h = headers(&[("x-name", b"widget")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&FULLY_BOUND, &mut consume(&mut out))
            .unwrap();
        assert!(
            !log.borrow().read,
            "a fully bound structure must not invoke the body codec"
        );
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
    }

    #[test]
    fn an_empty_structure_invokes_nothing() {
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 204, Some(b""))
            .read_struct(&EMPTY_STRUCT, &mut consume(&mut out))
            .unwrap();
        assert!(!log.borrow().read);
        assert_eq!(out, Populated::default());
    }

    #[test]
    fn a_body_only_structure_goes_straight_to_the_codec() {
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_ONLY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 200, Some(b"{}"))
            .read_struct(&BODY_ONLY, &mut consume(&mut out))
            .unwrap();
        assert!(log.borrow().read);
        assert_eq!(out.strings.get("note"), Some(&FROM_BODY.to_string()));
        assert!(log.borrow().skipped.is_empty());
    }

    // ---------------------------------------------------------------------------------
    // transport wins over the body
    // ---------------------------------------------------------------------------------

    #[test]
    fn a_body_copy_of_a_bound_member_is_consumed_and_discarded() {
        // The transport value is authoritative. A body field for the same member must be
        // consumed — so a cursor-based codec stays aligned — but must not overwrite it.
        let h = headers(&[("x-name", b"from-header"), ("x-count", b"7")]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&NAME_HEADER, &COUNT_HEADER, &STATUS, &BODY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 201, Some(b"{}"))
            .read_struct(&MIXED, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.get("name"), Some(&"from-header".to_string()));
        assert_eq!(out.integers.get("count"), Some(&7));
        assert_eq!(out.integers.get("status"), Some(&201));
        assert_eq!(
            log.borrow().skipped,
            vec![
                "name".to_string(),
                "count".to_string(),
                "status".to_string()
            ],
            "each bound member's body copy must be skipped, in body order"
        );
        assert_eq!(
            out.strings.get("note"),
            Some(&FROM_BODY.to_string()),
            "only the unbound member may come from the body"
        );
    }

    #[test]
    fn an_index_less_callback_is_forwarded_rather_than_filtered() {
        // JSON and CBOR report an unknown union variant through a callback whose schema has no
        // member index. Filtering it would hide unknown variants.
        let body_members: &[&Schema<'_>] = &[&crate::prelude::DOCUMENT];
        let log = RefCell::new(BodyLog::default());
        let h = headers(&[]);
        let mut seen = Vec::new();
        output(body_members, &log, &h, 200, Some(b"{}"))
            .read_struct(&BODY_ONLY, &mut |member, _| {
                seen.push(member.member_index());
                Ok(())
            })
            .unwrap();
        assert_eq!(
            seen,
            vec![None],
            "the index-less callback must reach the consumer"
        );
        assert!(log.borrow().skipped.is_empty());
    }

    // ---------------------------------------------------------------------------------
    // prefix headers
    // ---------------------------------------------------------------------------------

    static PREFIX_KEY: Schema<'static> = Schema::new(shape_id!("test", "K"), ShapeType::String);
    static PREFIX_VALUE: Schema<'static> = Schema::new(shape_id!("test", "V"), ShapeType::String);
    static PREFIX_MEMBER: Schema<'static> = Schema::new_member(ID, ShapeType::Map, "meta", 0)
        .with_http_prefix_headers("x-meta-")
        .with_map_members(&PREFIX_KEY, &PREFIX_VALUE);
    static PREFIX_SCHEMA: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&PREFIX_MEMBER]);

    #[test]
    fn a_prefix_map_is_populated_from_matching_headers() {
        let h = headers(&[("x-meta-a", b"1"), ("x-other", b"2")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&PREFIX_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(
            out.maps.get("meta").and_then(|m| m.get("a")),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn a_prefix_with_no_matches_is_an_empty_map_not_an_absent_member() {
        let h = headers(&[("x-other", b"2")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&PREFIX_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(
            out.maps.get("meta"),
            Some(&HashMap::new()),
            "existing Smithy behavior is Some(empty_map)"
        );
    }

    // ---------------------------------------------------------------------------------
    // payloads
    // ---------------------------------------------------------------------------------

    static BLOB_PAYLOAD: Schema<'static> =
        Schema::new_member(ID, ShapeType::Blob, "payload", 0).with_http_payload();
    static PAYLOAD_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::String, "name", 1).with_http_header("x-name");
    static BLOB_PAYLOAD_SCHEMA: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&BLOB_PAYLOAD, &PAYLOAD_HEADER]);

    #[test]
    fn a_raw_payload_comes_from_the_body_bytes_and_bypasses_the_codec() {
        let h = headers(&[("x-name", b"widget")]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 200, Some(b"\x00\x01raw"))
            .read_struct(&BLOB_PAYLOAD_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(
            out.blobs.get("payload").map(|b| b.as_slice()),
            Some(&b"\x00\x01raw"[..])
        );
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
        assert!(
            !log.borrow().read,
            "a payload member means there is no outer document for the codec to read"
        );
    }

    #[test]
    fn an_empty_payload_leaves_the_member_absent() {
        let h = headers(&[("x-name", b"widget")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&BLOB_PAYLOAD_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert!(
            !out.blobs.contains_key("payload"),
            "an empty payload must not become an empty blob"
        );
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
    }

    static STRUCT_PAYLOAD: Schema<'static> =
        Schema::new_member(ID, ShapeType::Structure, "payload", 0).with_http_payload();
    static STRUCT_PAYLOAD_SCHEMA: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&STRUCT_PAYLOAD]);

    #[test]
    fn a_structured_payload_is_handed_to_the_body_codec() {
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let mut saw_payload = false;
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b"{}"))
            .read_struct(&STRUCT_PAYLOAD_SCHEMA, &mut |member, d| {
                assert_eq!(member.member_name(), Some("payload"));
                // The codec is positioned at the payload root, so a generated arm's
                // `Foo::deserialize(deser)` reads the body root as the member's value.
                d.read_struct(member, &mut |_, _| Ok(()))?;
                saw_payload = true;
                Ok(())
            })
            .unwrap();
        assert!(saw_payload);
        assert!(
            log.borrow().read,
            "the structured payload must be read through the body codec"
        );
    }

    static STREAM_PAYLOAD: Schema<'static> = Schema::new_member(ID, ShapeType::Blob, "payload", 0)
        .with_http_payload()
        .with_streaming();
    static STREAM_SCHEMA: Schema<'static> = Schema::new_struct(
        ID,
        ShapeType::Structure,
        &[&STREAM_PAYLOAD, &PAYLOAD_HEADER],
    );

    #[test]
    fn a_streaming_payload_member_is_never_handed_to_a_consumer() {
        // A streaming response leaves the body unbuffered, which is the realistic shape.
        let h = headers(&[("x-name", b"widget")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, None)
            .read_struct(&STREAM_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert!(
            !out.blobs.contains_key("payload"),
            "the generated streaming path owns the live body and installs it itself"
        );
        assert_eq!(
            out.strings.get("name"),
            Some(&"widget".to_string()),
            "header members must still be populated for a streaming output"
        );
        assert!(
            !log.borrow().read,
            "the live stream must not be parsed as a document"
        );
    }

    #[test]
    fn a_non_streaming_payload_without_a_buffered_body_is_an_error() {
        // A wiring mistake rather than a service behavior, so reporting an absent member would
        // hide it.
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let err = output(NO_BODY_MEMBERS, &log, &h, 200, None)
            .read_struct(&BLOB_PAYLOAD_SCHEMA, &mut |_, _| Ok(()))
            .expect_err("no buffered body");
        assert!(
            format!("{err}").contains("was not buffered"),
            "unexpected error: {err}"
        );
    }

    // ---------------------------------------------------------------------------------
    // empty bodies
    // ---------------------------------------------------------------------------------

    #[test]
    fn a_modeled_error_with_an_empty_body_still_gets_its_headers_and_status() {
        // Services send empty error bodies; S3's HEAD responses are the standing example.
        let h = headers(&[("x-name", b"widget")]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let mut out = Populated::default();
        HttpResponseDeserializer::for_error(
            StubBody::new(body_members, &log),
            &h,
            404,
            Some(b""),
            NonUtf8HeaderHandling::Reject,
        )
        .read_struct(&MIXED, &mut consume(&mut out))
        .unwrap();
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
        assert_eq!(out.integers.get("status"), Some(&404));
        assert!(
            !log.borrow().read,
            "an empty error body must not be handed to the codec"
        );
    }

    #[test]
    fn a_modeled_error_with_a_body_still_reads_it() {
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_ONLY_FIELD];
        let mut out = Populated::default();
        HttpResponseDeserializer::for_error(
            StubBody::new(body_members, &log),
            &h,
            400,
            Some(b"{}"),
            NonUtf8HeaderHandling::Reject,
        )
        .read_struct(&BODY_ONLY, &mut consume(&mut out))
        .unwrap();
        assert!(log.borrow().read);
        assert_eq!(out.strings.get("note"), Some(&FROM_BODY.to_string()));
    }

    #[test]
    fn a_successful_response_keeps_the_codecs_empty_body_strictness() {
        // Only errors tolerate an empty body. A success must still reach the codec so its
        // existing behavior is preserved rather than decided here.
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_ONLY_FIELD];
        let mut out = Populated::default();
        output(body_members, &log, &h, 200, Some(b""))
            .read_struct(&BODY_ONLY, &mut consume(&mut out))
            .unwrap();
        assert!(
            log.borrow().read,
            "the composite must not decide emptiness on behalf of a successful response"
        );
    }

    // ---------------------------------------------------------------------------------
    // NonUtf8HeaderHandling
    // ---------------------------------------------------------------------------------

    fn with_policy<'a>(
        h: &'a Headers,
        policy: NonUtf8HeaderHandling,
        log: &'a RefCell<BodyLog>,
    ) -> HttpResponseDeserializer<'a, StubBody<'a>> {
        HttpResponseDeserializer::for_output(
            StubBody::new(NO_BODY_MEMBERS, log),
            h,
            200,
            Some(b""),
            policy,
        )
    }

    #[test]
    fn an_unreadable_header_is_rejected_by_default_with_member_and_header_context() {
        let h = headers(&[("x-name", b"value-\xe9")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        let err = with_policy(&h, NonUtf8HeaderHandling::Reject, &log)
            .read_struct(&FULLY_BOUND, &mut consume(&mut out))
            .expect_err("non-UTF-8 must be rejected");
        let message = format!("{err}");
        for expected in ["name", "x-name", "not valid utf-8"] {
            assert!(
                message.contains(expected),
                "expected {expected:?} in {message:?}"
            );
        }
    }

    #[test]
    fn skip_leaves_an_unreadable_member_absent_and_the_header_intact() {
        let h = headers(&[("x-name", b"value-\xe9")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        with_policy(&h, NonUtf8HeaderHandling::Skip, &log)
            .read_struct(&FULLY_BOUND, &mut consume(&mut out))
            .expect("Skip must not fail the response");
        assert!(!out.strings.contains_key("name"));
        assert_eq!(out.integers.get("status"), Some(&200));
        assert_eq!(
            h.get_bytes("x-name"),
            Some(&b"value-\xe9"[..]),
            "the raw octets must stay readable for interceptors"
        );
    }

    #[test]
    fn skip_does_not_hide_a_readable_but_malformed_value() {
        // The distinction that keeps Skip narrow: it applies to unreadable encodings only.
        let h = headers(&[("x-name", b"ok"), ("x-count", b"notanint")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        let err = with_policy(&h, NonUtf8HeaderHandling::Skip, &log)
            .read_struct(&MIXED, &mut consume(&mut out))
            .expect_err("a malformed integer is still an error under Skip");
        assert!(format!("{err}").contains("x-count"), "unexpected: {err}");
    }

    static LIST_ELEMENT: Schema<'static> = Schema::new(shape_id!("test", "E"), ShapeType::String);
    static LIST_HEADER: Schema<'static> = Schema::new_member(ID, ShapeType::List, "tags", 0)
        .with_http_header("x-tags")
        .with_list_member(&LIST_ELEMENT);
    static LIST_SCHEMA: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&LIST_HEADER]);

    #[test]
    fn skip_applies_to_the_whole_member_regardless_of_value_order() {
        // PR #4868's order-independence rule. List parsing stops at its first failure, so the
        // decision must come from scanning all raw values rather than from the error.
        for pairs in [
            vec![("x-tags", b"value-\xe9".as_slice()), ("x-tags", b"ok")],
            vec![("x-tags", b"ok".as_slice()), ("x-tags", b"value-\xe9")],
        ] {
            let h = headers(&pairs);
            let log = RefCell::new(BodyLog::default());
            let mut out = Populated::default();
            with_policy(&h, NonUtf8HeaderHandling::Skip, &log)
                .read_struct(&LIST_SCHEMA, &mut consume(&mut out))
                .expect("Skip must apply in both orders");
            assert!(!out.lists.contains_key("tags"));
        }
    }

    #[test]
    fn a_list_header_is_populated_when_every_value_is_readable() {
        let h = headers(&[("x-tags", b"a,b"), ("x-tags", b"c")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&LIST_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(
            out.lists.get("tags"),
            Some(&vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn a_prefix_map_is_suppressed_whole_under_skip() {
        let h = headers(&[("x-meta-good", b"ok"), ("x-meta-bad", b"value-\xe9")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        with_policy(&h, NonUtf8HeaderHandling::Skip, &log)
            .read_struct(&PREFIX_SCHEMA, &mut consume(&mut out))
            .expect("Skip must not fail the response");
        assert!(
            !out.maps.contains_key("meta"),
            "a partial map would conceal the dropped entry"
        );
    }

    #[test]
    fn an_unreadable_prefix_entry_is_rejected_by_default_naming_the_prefix() {
        let h = headers(&[("x-meta-bad", b"value-\xe9")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        let err = with_policy(&h, NonUtf8HeaderHandling::Reject, &log)
            .read_struct(&PREFIX_SCHEMA, &mut consume(&mut out))
            .expect_err("non-UTF-8 must be rejected");
        let message = format!("{err}");
        for expected in ["meta", "prefix header", "x-meta-"] {
            assert!(
                message.contains(expected),
                "expected {expected:?} in {message:?}"
            );
        }
    }

    #[test]
    fn the_policy_defaults_to_reject_and_round_trips_through_the_config_bag() {
        assert_eq!(
            non_utf8_policy(&ConfigBag::base()),
            NonUtf8HeaderHandling::Reject,
            "an empty bag must read as Reject rather than requiring callers to default it"
        );
        let mut layer = aws_smithy_types::config_bag::CloneableLayer::new("test");
        layer.store_put(NonUtf8HeaderHandling::Skip);
        let bag = ConfigBag::of_layers(vec![layer.into()]);
        assert_eq!(non_utf8_policy(&bag), NonUtf8HeaderHandling::Skip);
    }

    // ---------------------------------------------------------------------------------
    // mask vs scan
    // ---------------------------------------------------------------------------------

    /// Builds a structure whose members are all header-bound, at runtime, to reach member
    /// counts that are impractical as statics.
    fn all_bound(count: usize) -> &'static Schema<'static> {
        let mut leaked: Vec<&'static Schema<'static>> = Vec::new();
        for index in 0..count {
            let name: &'static str = Box::leak(format!("m{index}").into_boxed_str());
            let header: &'static str = Box::leak(format!("x-m{index}").into_boxed_str());
            leaked.push(Box::leak(Box::new(
                Schema::new_member(ID, ShapeType::String, name, index).with_http_header(header),
            )));
        }
        let members: &'static [&'static Schema<'static>] = Box::leak(leaked.into_boxed_slice());
        Box::leak(Box::new(Schema::new_struct(
            ID,
            ShapeType::Structure,
            members,
        )))
    }

    fn header_pairs(indices: impl Iterator<Item = usize>) -> Vec<(String, Vec<u8>)> {
        indices
            .map(|i| (format!("x-m{i}"), format!("v{i}").into_bytes()))
            .collect()
    }

    #[test]
    fn the_scan_fallback_routes_the_same_members_as_the_mask() {
        // A bound member beyond the mask's reach forces the scan path; it must find all of them.
        let schema = all_bound(64);
        assert!(
            schema.response_bindings_need_scan(),
            "precondition: 64 bound members exceeds the mask"
        );
        let pairs = header_pairs(0..64);
        let borrowed: Vec<(&str, &[u8])> = pairs
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_slice()))
            .collect();
        let h = headers(&borrowed);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(schema, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.len(), 64);
        assert_eq!(out.strings.get("m0"), Some(&"v0".to_string()));
        assert_eq!(out.strings.get("m63"), Some(&"v63".to_string()));
    }

    #[test]
    fn the_mask_path_and_the_scan_path_agree_on_the_same_members() {
        // 62 bound members is the largest exact mask; 64 forces the scan. Both must populate
        // every member they share.
        let exact = all_bound(62);
        let scanned = all_bound(64);
        assert!(!exact.response_bindings_need_scan());
        assert!(scanned.response_bindings_need_scan());

        let pairs = header_pairs(0..62);
        let borrowed: Vec<(&str, &[u8])> = pairs
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_slice()))
            .collect();
        let h = headers(&borrowed);

        let mut from_mask = Populated::default();
        let log = RefCell::new(BodyLog::default());
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(exact, &mut consume(&mut from_mask))
            .unwrap();

        let mut from_scan = Populated::default();
        let log = RefCell::new(BodyLog::default());
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(scanned, &mut consume(&mut from_scan))
            .unwrap();

        assert_eq!(
            from_mask, from_scan,
            "the two routing paths must produce identical results"
        );
        assert_eq!(from_mask.strings.len(), 62);
    }

    #[test]
    fn a_sparse_binding_layout_visits_only_the_bound_members() {
        let mut leaked: Vec<&'static Schema<'static>> = Vec::new();
        for index in 0..50usize {
            let name: &'static str = Box::leak(format!("m{index}").into_boxed_str());
            let schema = if index % 12 == 0 {
                let header: &'static str = Box::leak(format!("x-m{index}").into_boxed_str());
                Schema::new_member(ID, ShapeType::String, name, index).with_http_header(header)
            } else {
                Schema::new_member(ID, ShapeType::String, name, index)
            };
            leaked.push(Box::leak(Box::new(schema)));
        }
        let members: &'static [&'static Schema<'static>] = Box::leak(leaked.into_boxed_slice());
        let schema = Schema::new_struct(ID, ShapeType::Structure, members);
        assert!(!schema.response_bindings_need_scan());
        assert_eq!(schema.response_binding_mask().count_ones(), 5);

        let pairs = header_pairs((0..50).filter(|i| i % 12 == 0));
        let borrowed: Vec<(&str, &[u8])> = pairs
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_slice()))
            .collect();
        let h = headers(&borrowed);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b"{}"))
            .read_struct(&schema, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.len(), 5);
        assert!(
            log.borrow().read,
            "the 45 unbound members still belong to the body"
        );
    }

    static TS_HEADER: Schema<'static> = Schema::new_member(ID, ShapeType::Timestamp, "at", 0)
        .with_http_header("x-at")
        .with_timestamp_format(TimestampFormat::EpochSeconds);
    static TS_SCHEMA: Schema<'static> = Schema::new_struct(ID, ShapeType::Structure, &[&TS_HEADER]);

    #[test]
    fn a_timestamp_header_uses_the_schemas_format() {
        let h = headers(&[("x-at", b"1445412480")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&TS_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.timestamps.get("at"), Some(&1_445_412_480));
    }

    // ---------------------------------------------------------------------------------
    // delegation
    // ---------------------------------------------------------------------------------

    #[test]
    fn non_structure_reads_fall_through_to_the_body() {
        // Reached when a caller uses the composite where the body root is not a structure.
        let doc = Document::String("hello".to_string());
        let h = headers(&[]);
        let mut deser = HttpResponseDeserializer::for_output(
            DocumentShapeDeserializer::new(&doc),
            &h,
            200,
            Some(b""),
            NonUtf8HeaderHandling::Reject,
        );
        assert_eq!(deser.read_string(&crate::prelude::STRING).unwrap(), "hello");
    }

    // ---------------------------------------------------------------------------------
    // value semantics, end to end through the composite
    //
    // The value deserializers are unit-tested directly in `bound_value`. These cases prove the
    // composite hands them the whole member — every raw value, and the member's own schema — so
    // parsing behavior that depends on either is not lost in routing. They mirror the rows of
    // the generated-path test `schema response header parsing matches legacy semantics`, which
    // is deleted when the generated parser is.
    // ---------------------------------------------------------------------------------

    static NAN_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Float, "nan", 0).with_http_header("x-nan");
    static INF_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Float, "inf", 1).with_http_header("x-inf");
    static NEG_INF_HEADER: Schema<'static> =
        Schema::new_member(ID, ShapeType::Float, "neg", 2).with_http_header("x-neg");
    static FLOATS: Schema<'static> = Schema::new_struct(
        ID,
        ShapeType::Structure,
        &[&NAN_HEADER, &INF_HEADER, &NEG_INF_HEADER],
    );

    #[test]
    fn smithy_special_float_values_survive_the_composite() {
        // Smithy spells these `NaN`/`Infinity`/`-Infinity` on the wire, which is not what
        // `f32::from_str` accepts. Reaching the consumer with the right values proves the
        // composite routes through the Smithy primitive parser rather than a plain `FromStr`.
        let h = headers(&[
            ("x-nan", b"NaN"),
            ("x-inf", b"Infinity"),
            ("x-neg", b"-Infinity"),
        ]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&FLOATS, &mut consume(&mut out))
            .unwrap();
        assert!(out.floats["nan"].is_nan());
        assert_eq!(out.floats["inf"], f32::INFINITY);
        assert_eq!(out.floats["neg"], f32::NEG_INFINITY);
    }

    static MEDIA_HEADER: Schema<'static> = Schema::new_member(ID, ShapeType::String, "config", 0)
        .with_http_header("x-config")
        .with_media_type("application/json");
    static MEDIA: Schema<'static> = Schema::new_struct(ID, ShapeType::Structure, &[&MEDIA_HEADER]);

    #[test]
    fn a_media_type_header_is_base64_decoded_through_the_composite() {
        // `@mediaType` changes a string header from "take the value whole" to "base64-decode
        // it". The trait is on the member schema, so this only works if the composite passes
        // the member schema down rather than a prelude schema.
        let h = headers(&[("x-config", b"eyJhIjoxfQ==")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&MEDIA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(out.strings.get("config"), Some(&r#"{"a":1}"#.to_string()));
    }

    #[test]
    fn a_media_type_header_that_decodes_to_invalid_utf8_is_an_error_not_a_skip() {
        // The base64 is well-formed and every raw octet is valid UTF-8, so nothing about the
        // header is "unreadable" — the failure is in the decoded bytes. Skip must not suppress
        // it, otherwise a malformed payload becomes an absent member.
        let h = headers(&[("x-config", b"/w==")]); // decodes to 0xFF
        for policy in [NonUtf8HeaderHandling::Reject, NonUtf8HeaderHandling::Skip] {
            let log = RefCell::new(BodyLog::default());
            let mut out = Populated::default();
            let err = HttpResponseDeserializer::for_output(
                StubBody::new(NO_BODY_MEMBERS, &log),
                &h,
                200,
                Some(b""),
                policy.clone(),
            )
            .read_struct(&MEDIA, &mut consume(&mut out))
            .expect_err("invalid UTF-8 after base64 decoding is a parse error");
            let message = format!("{}", DisplayErrorContext(&err));
            assert!(
                message.contains("config") && message.contains("x-config"),
                "error must name the member and header, got: {message}"
            );
            assert!(
                !out.strings.contains_key("config"),
                "a failed parse must not populate the member"
            );
        }
    }

    #[test]
    fn quoted_list_values_keep_their_commas_through_the_composite() {
        // Repeated lines are already covered by `a_list_header_is_populated_when_every_value_is_readable`.
        // This adds the RFC 7230 quoting rule, which a naive comma split would break.
        let h = headers(&[("x-tags", br#""a,b", c, "d\"e""#)]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&LIST_SCHEMA, &mut consume(&mut out))
            .unwrap();
        assert_eq!(
            out.lists.get("tags"),
            Some(&vec![
                "a,b".to_string(),
                "c".to_string(),
                "d\"e".to_string()
            ])
        );
    }

    #[test]
    fn a_scalar_receiving_repeated_lines_is_a_cardinality_error() {
        // Not last-write-wins. The message is the parser's own, so the count reaches the caller
        // through the composite's context wrapper.
        let h = headers(&[("x-count", b"1"), ("x-count", b"2")]);
        let log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        let err = output(NO_BODY_MEMBERS, &log, &h, 200, Some(b""))
            .read_struct(&MIXED, &mut consume(&mut out))
            .expect_err("two values for a scalar header is an error");
        let message = format!("{}", DisplayErrorContext(&err));
        assert!(
            message.contains("expected one item but found 2"),
            "the parser's cardinality message must survive, got: {message}"
        );
        assert!(
            message.contains("count") && message.contains("x-count"),
            "error must name the member and header, got: {message}"
        );
    }

    // ---------------------------------------------------------------------------------
    // structured payload root
    // ---------------------------------------------------------------------------------

    const PAYLOAD_ID: ShapeId<'static> = shape_id!("test", "Payload");

    static XML_NAMED_PAYLOAD: Schema<'static> =
        Schema::new_member(ID, ShapeType::Structure, "payload", 0)
            .with_http_payload()
            .with_xml_name("CustomRoot");
    static XML_NAMED_OUTPUT: Schema<'static> =
        Schema::new_struct(ID, ShapeType::Structure, &[&XML_NAMED_PAYLOAD]);

    static PAYLOAD_FIELD: Schema<'static> =
        Schema::new_member(PAYLOAD_ID, ShapeType::String, "inner", 0);
    static PAYLOAD_TARGET: Schema<'static> =
        Schema::new_struct(PAYLOAD_ID, ShapeType::Structure, &[&PAYLOAD_FIELD]);

    #[test]
    fn a_structured_payload_root_is_chosen_by_the_consumer_not_the_composite() {
        // The composite positions the codec at the body root and hands the consumer the *member*
        // schema; the schema the codec actually sees as its root is whichever one the generated
        // arm forwards. The legacy generated arm forwards the target's schema
        // (`Target::deserialize(deser)`), so member-level `@xmlName` is not consulted for the
        // payload root — in either path. This pins that the composite does not change it.
        let h = headers(&[]);
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&PAYLOAD_FIELD];
        let mut saw_member_xml_name = None;
        let mut inner = None;
        output(body_members, &log, &h, 200, Some(b"<CustomRoot/>"))
            .read_struct(&XML_NAMED_OUTPUT, &mut |member, deser| {
                saw_member_xml_name = member.xml_name().map(|n| n.value().to_string());
                // Exactly what the generated arm does: forward the *target's* schema.
                deser.read_struct(&PAYLOAD_TARGET, &mut |field, value| {
                    inner = Some((
                        field.member_name().unwrap_or("?").to_string(),
                        value.read_string(field)?,
                    ));
                    Ok(())
                })
            })
            .unwrap();
        assert_eq!(
            saw_member_xml_name.as_deref(),
            Some("CustomRoot"),
            "the member's traits must be reachable from the schema the composite passes"
        );
        assert_eq!(
            log.borrow().root.as_deref(),
            Some("Payload"),
            "the codec's root is the schema the consumer forwarded, not the member's xmlName"
        );
        assert_eq!(
            inner,
            Some(("inner".to_string(), FROM_BODY.to_string())),
            "the codec must be positioned at the payload root"
        );
    }

    // ---------------------------------------------------------------------------------
    // the shared wrapping helpers
    //
    // These are the only sanctioned way for a REST protocol to build a response
    // deserializer, so their branch decisions are load-bearing: a wrong body-only verdict
    // silently drops every transport-bound member.
    // ---------------------------------------------------------------------------------

    use aws_smithy_types::body::SdkBody;

    fn http_response(status: u16, pairs: &[(&str, &[u8])], body: SdkBody) -> Response {
        let mut builder = http::Response::builder().status(status);
        for (name, value) in pairs {
            builder = builder.header(*name, http::HeaderValue::from_bytes(value).expect("value"));
        }
        Response::try_from(builder.body(body).expect("response")).expect("convertible")
    }

    /// A body deserializer that reports one string member, so a test can tell whether the codec
    /// was reached without needing a real codec.
    fn one_member_body<'a>(
        members: &'a [&'a Schema<'a>],
        log: &'a RefCell<BodyLog>,
    ) -> StubBody<'a> {
        StubBody::new(members, log)
    }

    #[test]
    fn a_body_only_output_takes_the_bare_body_fast_path() {
        assert!(
            is_body_only(&BODY_ONLY),
            "an output with only body members has nothing for the composite to do"
        );
        // The observable behavior is identical either way — that is the point of the fast path —
        // so the assertion above is the discriminator, and this half proves the returned
        // deserializer still works.
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_ONLY_FIELD];
        let response = http_response(200, &[], SdkBody::from("{}"));
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(body_members, &log),
            &response,
            &BODY_ONLY,
            &ConfigBag::base(),
        )
        .read_struct(&BODY_ONLY, &mut consume(&mut out))
        .unwrap();
        assert_eq!(out.strings.get("note"), Some(&FROM_BODY.to_string()));
    }

    #[test]
    fn an_empty_output_does_not_take_the_fast_path() {
        // An empty output has no body members either. Handing it to the codec would make an
        // empty body an error for nothing, so it must route through the composite.
        assert!(!is_body_only(&EMPTY_STRUCT));
        let log = RefCell::new(BodyLog::default());
        let response = http_response(204, &[], SdkBody::empty());
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(NO_BODY_MEMBERS, &log),
            &response,
            &EMPTY_STRUCT,
            &ConfigBag::base(),
        )
        .read_struct(&EMPTY_STRUCT, &mut consume(&mut out))
        .unwrap();
        assert!(
            !log.borrow().read,
            "the codec must not be invoked for an empty output"
        );
    }

    #[test]
    fn a_bound_output_does_not_take_the_fast_path() {
        assert!(!is_body_only(&MIXED));
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let response = http_response(201, &[("x-name", b"widget")], SdkBody::from("{}"));
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(body_members, &log),
            &response,
            &MIXED,
            &ConfigBag::base(),
        )
        .read_struct(&MIXED, &mut consume(&mut out))
        .unwrap();
        assert_eq!(out.strings.get("name"), Some(&"widget".to_string()));
        assert_eq!(
            out.integers.get("status"),
            Some(&201),
            "the status must come from the response the helper was given"
        );
    }

    #[test]
    fn a_non_structure_schema_does_not_take_the_fast_path() {
        // The default error forwarding passes `prelude::DOCUMENT`. Its routing word is zeroed,
        // which reports no body members, so it must not be mistaken for a body-only output.
        assert!(!is_body_only(&crate::prelude::DOCUMENT));
    }

    /// No error path can reach the success body-only fast path, whatever schema it is later
    /// handed.
    ///
    /// The fast path exists so a REST output with no response bindings costs nothing, but it
    /// returns the *bare* codec — a deserializer that cannot read a header, a prefix map or a
    /// status. Applying it to an error would silently drop every bound error member, and the
    /// concrete error schema is not even known when the deserializer is built: the generated
    /// error variant supplies it later, when it calls `read_struct`.
    ///
    /// [`http_error_deserializer`] therefore takes no schema at all, which makes the property
    /// structural rather than a rule to remember. This test pins the consequence by choosing the
    /// one schema most likely to defeat it — a genuinely body-only one, for which the success
    /// helper *does* take the fast path on identical inputs.
    #[test]
    fn the_error_path_never_takes_the_body_only_fast_path() {
        assert!(
            is_body_only(&BODY_ONLY),
            "precondition: this schema is the fast path's best case"
        );
        let body_members: &[&Schema<'_>] = &[&BODY_ONLY_FIELD];
        let response = || http_response(404, &[], SdkBody::empty());

        // Error mode: the composite owns the empty-body decision and skips the codec.
        let error_log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        http_error_deserializer(
            one_member_body(body_members, &error_log),
            &response(),
            &ConfigBag::base(),
        )
        .read_struct(&BODY_ONLY, &mut consume(&mut out))
        .unwrap();
        assert!(
            !error_log.borrow().read,
            "an error must go through the composite even for a body-only schema"
        );

        // Success mode on the same inputs: the fast path hands the body straight to the codec.
        // This half is what gives the assertion above its teeth — it proves the two modes really
        // do diverge here, so the error result is not simply what any deserializer would do.
        let output_log = RefCell::new(BodyLog::default());
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(body_members, &output_log),
            &response(),
            &BODY_ONLY,
            &ConfigBag::base(),
        )
        .read_struct(&BODY_ONLY, &mut consume(&mut out))
        .unwrap();
        assert!(
            output_log.borrow().read,
            "precondition: the success helper takes the fast path for this schema"
        );
    }

    #[test]
    fn a_binding_beyond_the_mask_still_defeats_the_fast_path() {
        // The case the `needs_scan` guard exists for: member 63 is bound, but 63 is outside the
        // mask, so the mask reads as zero. Without the guard this would look body-only and the
        // binding would be silently dropped.
        let mut leaked: Vec<&'static Schema<'static>> = Vec::new();
        for index in 0..64usize {
            let name: &'static str = Box::leak(format!("m{index}").into_boxed_str());
            let schema = if index == 63 {
                Schema::new_member(ID, ShapeType::String, name, index).with_http_header("x-late")
            } else {
                Schema::new_member(ID, ShapeType::String, name, index)
            };
            leaked.push(Box::leak(Box::new(schema)));
        }
        let members: &'static [&'static Schema<'static>] = Box::leak(leaked.into_boxed_slice());
        let schema: &'static Schema<'static> = Box::leak(Box::new(Schema::new_struct(
            ID,
            ShapeType::Structure,
            members,
        )));
        assert_eq!(
            schema.response_binding_mask(),
            0,
            "precondition: the only binding is outside the mask"
        );
        assert!(schema.response_bindings_need_scan());
        assert!(schema.has_response_body_members());
        assert!(
            !is_body_only(schema),
            "an inexact routing word must never be read as body-only"
        );

        let log = RefCell::new(BodyLog::default());
        let response = http_response(200, &[("x-late", b"found")], SdkBody::from("{}"));
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(NO_BODY_MEMBERS, &log),
            &response,
            schema,
            &ConfigBag::base(),
        )
        .read_struct(schema, &mut consume(&mut out))
        .unwrap();
        assert_eq!(out.strings.get("m63"), Some(&"found".to_string()));
    }

    #[test]
    fn the_error_helper_tolerates_an_empty_body_while_still_reading_bindings() {
        // What separates the two helpers. A modeled error with an empty body — an S3 `HEAD`, for
        // instance — must still populate its header and status members.
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let response = http_response(404, &[("x-name", b"missing")], SdkBody::empty());
        let mut out = Populated::default();
        http_error_deserializer(
            one_member_body(body_members, &log),
            &response,
            &ConfigBag::base(),
        )
        .read_struct(&MIXED, &mut consume(&mut out))
        .unwrap();
        assert_eq!(out.strings.get("name"), Some(&"missing".to_string()));
        assert_eq!(out.integers.get("status"), Some(&404));
        assert!(
            !log.borrow().read,
            "an empty error body must not be handed to the codec"
        );
    }

    #[test]
    fn the_success_helper_keeps_the_codecs_empty_body_strictness() {
        // The same response through the output helper still reaches the codec, preserving
        // whatever the codec does with an empty body today.
        let log = RefCell::new(BodyLog::default());
        let body_members: &[&Schema<'_>] = &[&BODY_FIELD];
        let response = http_response(200, &[("x-name", b"here")], SdkBody::empty());
        let mut out = Populated::default();
        http_output_deserializer(
            one_member_body(body_members, &log),
            &response,
            &MIXED,
            &ConfigBag::base(),
        )
        .read_struct(&MIXED, &mut consume(&mut out))
        .unwrap();
        assert!(
            log.borrow().read,
            "a successful response must still hand an empty body to the codec"
        );
    }

    #[test]
    fn the_helpers_read_the_policy_from_the_config_bag() {
        let response = http_response(200, &[("x-name", b"value-\xe9")], SdkBody::from("{}"));
        let mut cfg = ConfigBag::base();
        cfg.interceptor_state()
            .store_put(NonUtf8HeaderHandling::Skip);

        for label in ["output", "error"] {
            let log = RefCell::new(BodyLog::default());
            let mut out = Populated::default();
            let mut deser = if label == "output" {
                http_output_deserializer(
                    one_member_body(NO_BODY_MEMBERS, &log),
                    &response,
                    &FULLY_BOUND,
                    &cfg,
                )
            } else {
                http_error_deserializer(one_member_body(NO_BODY_MEMBERS, &log), &response, &cfg)
            };
            deser
                .read_struct(&FULLY_BOUND, &mut consume(&mut out))
                .unwrap_or_else(|e| panic!("{label}: Skip must reach the composite: {e}"));
            assert!(
                !out.strings.contains_key("name"),
                "{label}: the unreadable member must be absent"
            );
        }
    }

    #[test]
    fn an_unbuffered_body_is_reported_as_unbuffered_not_as_empty() {
        // The helper passes `bytes()` through as an `Option` rather than collapsing `None` into
        // `&[]`. That distinction is what lets a `@streaming` payload be skipped while a
        // non-streaming payload over an unbuffered body is reported as a wiring mistake instead
        // of silently becoming an absent member.
        let log = RefCell::new(BodyLog::default());
        let response = http_response(200, &[], SdkBody::taken());
        assert!(
            response.body().bytes().is_none(),
            "precondition: the body is not buffered"
        );
        let mut out = Populated::default();
        let err = http_output_deserializer(
            one_member_body(NO_BODY_MEMBERS, &log),
            &response,
            &BLOB_PAYLOAD_SCHEMA,
            &ConfigBag::base(),
        )
        .read_struct(&BLOB_PAYLOAD_SCHEMA, &mut consume(&mut out))
        .expect_err("a non-streaming payload cannot be read from an unbuffered body");
        assert!(
            format!("{err}").contains("was not buffered"),
            "unexpected: {err}"
        );
    }
}
