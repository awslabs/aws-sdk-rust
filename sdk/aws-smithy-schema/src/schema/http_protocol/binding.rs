/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP binding protocol for REST-style APIs.

use crate::codec::{Codec, FinishSerializer};
use crate::protocol::{apply_http_endpoint, ClientProtocolInner};
use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
use crate::{Schema, ShapeId};
use aws_smithy_runtime_api::http::{Headers, Request, Response};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;
use std::borrow::Cow;

/// An HTTP protocol for REST-style APIs that use HTTP bindings.
///
/// This protocol splits input members between HTTP locations (headers, query
/// strings, URI labels) and the payload based on HTTP binding traits
/// (`@httpHeader`, `@httpQuery`, `@httpLabel`, `@httpPayload`, etc.).
/// Non-bound members are serialized into the body using the provided codec.
///
/// # Type parameters
///
/// * `C` — the payload codec (e.g., `JsonCodec`, `XmlCodec`)
#[derive(Debug)]
pub struct HttpBindingProtocol<C> {
    protocol_id: ShapeId,
    codec: C,
    content_type: &'static str,
}

impl<C: Codec> HttpBindingProtocol<C> {
    /// Creates a new HTTP binding protocol.
    pub fn new(protocol_id: ShapeId, codec: C, content_type: &'static str) -> Self {
        Self {
            protocol_id,
            codec,
            content_type,
        }
    }

    /// Returns a reference to the body codec. Used by wrapper protocols
    /// (e.g. `AwsRestXmlProtocol`) that need to construct and pre-configure
    /// a body serializer before delegating to
    /// [`serialize_request_with_body`](Self::serialize_request_with_body).
    pub fn codec(&self) -> &C {
        &self.codec
    }

    /// Body-providable variant of [`serialize_request`](Self::serialize_request).
    /// The caller supplies an already-constructed body serializer, allowing
    /// codec-specific pre-configuration (e.g. setting one-shot state on the
    /// body codec before binding-driven serialization begins). The default
    /// `serialize_request` implementation calls this with a fresh serializer
    /// from the codec.
    ///
    /// This is the extension point used by `AwsRestXmlProtocol` to inject the
    /// member-level `@xmlName` for an `@httpPayload` struct member into the
    /// body codec — a value the codec couldn't otherwise see because codegen
    /// passes the *target* shape's `SCHEMA` for that member, which carries
    /// the target's `@xmlName` but not the member's.
    pub fn serialize_request_with_body(
        &self,
        body: <C as Codec>::Serializer,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        // Construct the request up front with an empty body. The binder is
        // given a `&mut Headers` reference into this request and inserts
        // headers directly as it walks members — avoiding the cost of an
        // intermediate `Vec<(...)>` plus a late flush loop. The body and URI
        // are populated after the binder's borrow is released.
        let mut request = Request::new(SdkBody::empty());

        // Check if there's an @httpPayload member targeting a structure/union.
        // In that case, the payload member's own write_struct provides the body
        // framing, so we must not add top-level struct framing.
        let has_struct_payload = input_schema.members().iter().any(|m| {
            m.http_payload().is_some()
                && matches!(
                    m.shape_type(),
                    crate::ShapeType::Structure | crate::ShapeType::Union
                )
        });
        // If the schema declares zero body members (every member is HTTP-bound,
        // and any `@httpPayload` is on a scalar that bypasses the codec),
        // we can skip body-codec invocation entirely. The wasted work would be:
        //   - XmlSerializer/JsonSerializer::write_struct opens a wrapper element
        //   - Proxy::serialize_members re-enters the binder
        //   - close-element is emitted
        //   - bytes are collected by `body.finish()` and then discarded
        //     (since `has_body_members == false` later forces `body = Vec::new()`)
        // Skipping all of that just calls `serialize_members` directly through
        // the binder so HTTP-bound members are still routed to headers / query /
        // labels. `is_top_level` is cleared first so any nested struct that
        // happens to still pass through (none do in this branch by definition,
        // but defensive) takes the body-delegation path.
        //
        // Codegen sets `with_no_body_members()` on operation input shapes whose
        // members are all HTTP-bound (e.g., S3 PutObjectInput, CopyObjectInput).
        // Hand-constructed schemas default to `has_body_members == true` so this
        // optimization is never silently applied to a schema that actually has
        // body members.
        let skip_body_codec = !input_schema.has_body_members() && !has_struct_payload;

        // Run the binder in a scope so its `&mut Headers` borrow on `request`
        // is released before we mutate the request again (set_uri / body swap
        // / Content-Type / Content-Length).
        let (raw_payload, body_bytes, query_params, labels) = {
            let mut binder =
                HttpBindingSerializer::new(body, Some(input_schema), request.headers_mut());

            if skip_body_codec || has_struct_payload {
                // skip_body_codec: input has no body members at all → all members
                //                  route to HTTP bindings, body bytes are unused.
                // has_struct_payload: an @httpPayload struct member writes itself
                //                     to the body without wrapping — call
                //                     serialize_members directly so framing comes
                //                     from the payload struct, not from the binder.
                binder.is_top_level = false;
                input.serialize_members(&mut binder)?;
            } else {
                binder.write_struct(input_schema, input)?;
            }
            let raw_payload = binder.raw_payload;
            let body_bytes = if raw_payload.is_some() || skip_body_codec {
                // @httpPayload blob/string — don't use the codec output.
                // skip_body_codec — body codec was never written to.
                Vec::new()
            } else {
                binder.body.finish()
            };
            (raw_payload, body_bytes, binder.query_params, binder.labels)
        };

        // Per the REST-JSON content-type handling spec:
        // - If @httpPayload targets a blob/string: send raw bytes, no Content-Type when empty
        // - If body members exist (even if all optional and unset): send `{}` with Content-Type
        // - If no body members at all (everything is in headers/query/labels): empty body, no Content-Type
        let has_blob_or_string_payload = raw_payload.is_some();
        // Mirror the schema's compile-time signal at runtime. When the schema
        // says no body members AND there's no struct-payload override, this
        // is straightforwardly false.
        let has_body_members = has_struct_payload
            || (input_schema.has_body_members()
                && input_schema.members().iter().any(|m| {
                    m.http_header().is_none()
                        && m.http_query().is_none()
                        && m.http_label().is_none()
                        && m.http_prefix_headers().is_none()
                        && m.http_query_params().is_none()
                        && m.http_payload().is_none()
                }));

        let mut body_bytes = body_bytes;
        let set_content_type = if has_blob_or_string_payload {
            // Blob/string payload: Content-Type comes from the @httpHeader("Content-Type")
            // member if present, or defaults to application/octet-stream for blobs.
            // Don't set the protocol's codec content type (e.g., application/json).
            false
        } else if has_body_members {
            // Operation has body members — body includes framing (e.g., `{}`).
            // Per the REST-JSON spec, even if all members are optional and unset, send `{}`.
            true
        } else {
            // No body members at all — empty body, no Content-Type.
            body_bytes = Vec::new();
            false
        };

        // Build URI: write directly into a single, capacity-hinted String
        // instead of repeatedly `format!`-allocating placeholders and
        // `replace`-allocating new copies of the path. Profiling on PutObject
        // SER showed `format::format_inner` + `alloc::str::replace` together
        // were ~25% of bench loop. The new path is one allocation per request
        // for the URI string itself; percent-encoding writes through
        // `percent_encode_into` to avoid per-segment String allocs.
        let template_opt = input_schema.http().map(|h| h.uri());
        // Capacity heuristic: endpoint + template + slack for label expansion
        // (greedy labels typically expand by O(1.5x)). Better-than-default
        // initial capacity avoids the first 1-2 reallocs.
        let mut uri =
            String::with_capacity(endpoint.len() + template_opt.map(|t| t.len()).unwrap_or(1) + 64);
        match template_opt {
            Some(template) => {
                if !endpoint.is_empty() {
                    uri.push_str(endpoint);
                }
                append_uri_with_labels(template, &labels, &mut uri);
            }
            None => {
                if endpoint.is_empty() {
                    uri.push('/');
                } else {
                    // Endpoint may contain `{...}` label placeholders to
                    // substitute (this branch is for shapes without an
                    // `@http` trait, where the endpoint *is* the template).
                    append_uri_with_labels(endpoint, &labels, &mut uri);
                }
            }
        }
        if !query_params.is_empty() {
            uri.push(if uri.contains('?') { '&' } else { '?' });
            let mut first = true;
            for (k, v) in &query_params {
                if !first {
                    uri.push('&');
                }
                percent_encode_into(k, &mut uri);
                uri.push('=');
                percent_encode_into(v, &mut uri);
                first = false;
            }
        }

        // Swap the body in place. Headers were inserted directly during the
        // binder phase, so no flush loop is needed here.
        *request.body_mut() = if let Some(payload) = raw_payload {
            SdkBody::from(payload)
        } else {
            SdkBody::from(body_bytes)
        };
        // Set HTTP method from @http trait
        if let Some(http) = input_schema.http() {
            request
                .set_method(http.method())
                .map_err(|e| SerdeError::custom(format!("invalid HTTP method: {e}")))?;
        }
        request
            .set_uri(uri.as_str())
            .map_err(|e| SerdeError::custom(format!("invalid endpoint URI: {e}")))?;
        // Customer-supplied @httpHeader("Content-Type") wins over the
        // protocol default. (Pre-opt2 the late flush loop overwrote our
        // default after we set it; with direct insertion the customer header
        // is already present, so we must not clobber it.)
        //
        // A presigning interceptor (or any other caller that stored a
        // `SharedHeaderOmitSettings` in the config bag) can request the
        // runtime suppress these defaults so they don't end up in the signed-
        // header set of a presigned URL.
        let omit = cfg.load::<crate::header_omit_settings::SharedHeaderOmitSettings>();
        let omit_content_type = omit
            .map(|s| s.should_omit_default_content_type())
            .unwrap_or(false);
        let omit_content_length = omit
            .map(|s| s.should_omit_default_content_length())
            .unwrap_or(false);
        if !omit_content_type && set_content_type && request.headers().get("Content-Type").is_none()
        {
            request
                .headers_mut()
                .insert("Content-Type", self.content_type);
        }
        if !omit_content_length {
            if let Some(len) = request.body().content_length() {
                if (len > 0 || set_content_type)
                    && request.headers().get("Content-Length").is_none()
                {
                    request
                        .headers_mut()
                        .insert("Content-Length", len.to_string());
                }
            }
        }
        Ok(request)
    }
}

// Note: there is a percent_encoding crate we use some other places for this, but I'm trying to keep
// the dependencies to a minimum.
/// Percent-encode a string per RFC 3986 section 2.3 (unreserved characters only).
pub fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    percent_encode_into(input, &mut out);
    out
}

/// Percent-encode `input` per RFC 3986 section 2.3 (unreserved characters only),
/// appending the result to `out`. Bulk-copies runs of already-safe bytes via
/// `push_str` instead of pushing one byte at a time, which is the common case
/// for URI labels and query values (typical inputs need no escaping).
pub fn percent_encode_into(input: &str, out: &mut String) {
    let bytes = input.as_bytes();
    let mut start = 0usize;
    for (i, &b) in bytes.iter().enumerate() {
        let safe = matches!(
            b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
        );
        if !safe {
            // Bulk-copy the run of safe bytes ending just before `i`.
            // SAFETY: `start..i` is a slice of `input`'s UTF-8 bytes, and
            // every byte in `start..i` was confirmed `safe` (ASCII), so the
            // slice is valid UTF-8.
            if start < i {
                out.push_str(&input[start..i]);
            }
            out.push('%');
            out.push(char::from(HEX[(b >> 4) as usize]));
            out.push(char::from(HEX[(b & 0x0f) as usize]));
            start = i + 1;
        }
    }
    if start < bytes.len() {
        out.push_str(&input[start..]);
    }
}

/// Walk a URI template like `/{Bucket}/{Key+}` and append the substituted
/// path to `out`. `labels` is a small list (typically <4) so a linear
/// scan per label-site is fine. Greedy labels (`{Name+}`) preserve `/`
/// separators and percent-encode each segment independently; regular
/// labels percent-encode the value as a whole.
///
/// Replaces an older implementation that did
/// `path.replace(&format!("{{{name}}}"), ...)` per label — multiple
/// String allocations per label and quadratic full-string scans. Top
/// hot path on PutObject SER (~25% of bench loop pre-fix).
fn append_uri_with_labels(
    template: &str,
    labels: &[(Cow<'static, str>, String)],
    out: &mut String,
) {
    let mut rem = template;
    while let Some(open) = rem.find('{') {
        out.push_str(&rem[..open]);
        let after_open = &rem[open + 1..];
        let close = match after_open.find('}') {
            Some(c) => c,
            None => {
                // Malformed template (unmatched `{`); preserve verbatim.
                out.push('{');
                rem = after_open;
                continue;
            }
        };
        let label = &after_open[..close];
        let (name, greedy) = match label.strip_suffix('+') {
            Some(n) => (n, true),
            None => (label, false),
        };
        // Linear lookup — labels.len() is typically <= 4.
        let value = labels
            .iter()
            .find(|(n, _)| n.as_ref() == name)
            .map(|(_, v)| v.as_str());
        if let Some(v) = value {
            if greedy {
                // Encode each `/`-separated segment independently to preserve `/`.
                let mut first = true;
                for seg in v.split('/') {
                    if !first {
                        out.push('/');
                    }
                    percent_encode_into(seg, out);
                    first = false;
                }
            } else {
                percent_encode_into(v, out);
            }
        }
        // else: label not provided — leave it as nothing (matches previous
        // behavior where `replace` would not match because the input never
        // contained the placeholder).
        rem = &after_open[close + 1..];
    }
    if !rem.is_empty() {
        out.push_str(rem);
    }
}

pub(crate) const HEX: &[u8; 16] = b"0123456789ABCDEF";

/// A ShapeSerializer that intercepts member writes and routes HTTP-bound
/// members to headers, query params, or URI labels instead of the body.
///
/// Members without HTTP binding traits are forwarded to the inner body
/// serializer unchanged.
struct HttpBindingSerializer<'a, S> {
    body: S,
    /// Headers are inserted directly into the `Request`'s header map as they
    /// are encountered, avoiding the cost of a `Vec<(...)>` intermediate plus
    /// a late flush loop. The borrow ends when the binder is dropped at the
    /// end of `serialize_request_with_body`'s binder-scope.
    headers: &'a mut Headers,
    query_params: Vec<(Cow<'static, str>, String)>,
    labels: Vec<(Cow<'static, str>, String)>,
    /// When set, member schemas are resolved from this schema by name to find
    /// HTTP binding traits. This allows the protocol to override bindings
    /// (e.g., for presigning where body members become query params).
    input_schema: Option<&'a Schema>,
    /// True for the top-level input struct in serialize_request.
    /// Cleared after the first write_struct so nested structs delegate directly.
    is_top_level: bool,
    /// Raw payload bytes for `@httpPayload` blob/string members. When a member
    /// has `@httpPayload` and targets a blob or string, the raw bytes bypass
    /// the codec serializer entirely and are used as the HTTP body directly.
    /// Safety: the referenced bytes are borrowed from the input struct passed to
    /// `serialize_request`, which outlives this serializer.
    raw_payload: Option<&'a [u8]>,
    /// Tracks member indices that have already been routed to HTTP bindings
    /// (`@httpHeader`, `@httpQuery`, `@httpLabel`, `@httpPrefixHeaders`,
    /// `@httpQueryParams`). Some body codecs (notably `XmlSerializer`) call
    /// `serialize_members` more than once on a single struct (a two-pass for
    /// attribute / element ordering). Without this guard each HTTP-bound
    /// member would be appended to its target collection on every pass,
    /// duplicating header / query entries and breaking presigning signatures.
    ///
    /// Implementation: see [`VisitedMembers`]. Stack-only for shapes with up
    /// to `VisitedMembers::INLINE_CAPACITY` HTTP-bound members; the JSON
    /// path (single-pass codec, never observes a duplicate) therefore pays
    /// no per-request allocation here.
    visited_bound_members: VisitedMembers,
}

/// Compact dedup set for member indices seen during HTTP-binding routing.
///
/// Replaces `HashSet<usize>` for two reasons:
/// 1. `HashSet::new()` is zero-alloc, but the first `.insert()` allocates
///    the bucket array. Single-pass codecs (e.g. `JsonSerializer`) call
///    `should_route_binding` once per HTTP-bound member but never observe a
///    duplicate, so that allocation is pure waste on a hot path.
/// 2. For typical structs (≤ `INLINE_CAPACITY` HTTP-bound members), inline
///    storage avoids the heap entirely and a linear scan over `u32`s is
///    cheaper than a hash + bucket lookup.
///
/// Spills to a `Vec` for larger structures (rare — S3 `CopyObject`'s
/// 22-binding worst case still fits within `INLINE_CAPACITY`).
#[derive(Debug)]
struct VisitedMembers {
    inline: [u32; Self::INLINE_CAPACITY],
    inline_len: u8,
    overflow: Vec<u32>,
}

impl VisitedMembers {
    /// Sized to cover the widest real Smithy operation input shapes we
    /// know about (S3 `CopyObject`: 22 HTTP-bound members) without
    /// spilling to the heap. The dedup logic stays correct beyond this
    /// limit; only the no-allocation property is lost.
    const INLINE_CAPACITY: usize = 24;

    const fn new() -> Self {
        Self {
            inline: [0; Self::INLINE_CAPACITY],
            inline_len: 0,
            overflow: Vec::new(),
        }
    }

    /// Record `idx` as visited. Returns `true` if newly inserted, `false`
    /// if already present (matching `HashSet::insert`'s semantics).
    fn insert(&mut self, idx: usize) -> bool {
        // Cap the cast at u32::MAX. Smithy member indices in practice are
        // tiny (the largest model in the AWS catalog has fewer than 1000
        // members on any single shape), so the truncation is unreachable
        // except in pathological hand-constructed schemas — in which case
        // dedup is a no-op for the truncated indices, which is correct
        // behavior (no duplicate routing) at the cost of a possible spurious
        // re-route.
        let idx = idx.min(u32::MAX as usize) as u32;
        let len = self.inline_len as usize;
        if self.inline[..len].contains(&idx) {
            return false;
        }
        if !self.overflow.is_empty() && self.overflow.contains(&idx) {
            return false;
        }
        if len < Self::INLINE_CAPACITY {
            self.inline[len] = idx;
            self.inline_len += 1;
        } else {
            self.overflow.push(idx);
        }
        true
    }
}

impl<'a, S> HttpBindingSerializer<'a, S> {
    fn new(body: S, input_schema: Option<&'a Schema>, headers: &'a mut Headers) -> Self {
        Self {
            body,
            headers,
            query_params: Vec::new(),
            labels: Vec::new(),
            input_schema,
            is_top_level: true,
            raw_payload: None,
            visited_bound_members: VisitedMembers::new(),
        }
    }

    /// Returns `true` the first time this member's HTTP binding is observed
    /// on this serializer, marking it visited. Some body codecs (notably
    /// `XmlSerializer`) invoke `serialize_members` more than once on the same
    /// struct (a two-pass for attribute / element ordering). Without this
    /// guard each HTTP-bound member would be appended to its target
    /// collection on every pass, duplicating header / query / label entries
    /// and producing wrong-signed presigned URLs.
    ///
    /// HTTP-bound members are always struct members and so always have an
    /// index. The `unwrap_or(true)` fallback for schemas without an index
    /// keeps the helper conservative — it routes when it can't dedupe.
    fn should_route_binding(&mut self, schema: &Schema) -> bool {
        schema
            .member_index()
            .map(|idx| self.visited_bound_members.insert(idx))
            .unwrap_or(true)
    }

    /// Resolve the effective member schema: if an input_schema override is set,
    /// look up the member by name there (to get the correct HTTP bindings).
    /// Otherwise use the schema as-is.
    fn resolve_member<'s>(&self, schema: &'s Schema) -> &'s Schema
    where
        'a: 's,
    {
        if let (Some(input_schema), Some(idx)) = (self.input_schema, schema.member_index()) {
            input_schema.member_schema_by_index(idx).unwrap_or(schema)
        } else if let (Some(input_schema), Some(name)) = (self.input_schema, schema.member_name()) {
            // Fallback to name lookup for schemas without a member index
            input_schema.member_schema(name).unwrap_or(schema)
        } else {
            schema
        }
    }
}

impl<'a, S: ShapeSerializer> ShapeSerializer for HttpBindingSerializer<'a, S> {
    fn write_struct(
        &mut self,
        schema: &Schema,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        if self.is_top_level {
            // Top-level input struct: route serialize_members through the binder
            // so HTTP-bound members are intercepted. The body serializer's
            // write_struct is used for framing (e.g., { } for JSON), with a
            // proxy whose serialize_members delegates back to the binder.
            struct Proxy<'a, 'b, S> {
                binder: &'a mut HttpBindingSerializer<'b, S>,
                value: &'a dyn SerializableStruct,
            }
            impl<S: ShapeSerializer> SerializableStruct for Proxy<'_, '_, S> {
                fn serialize_members(
                    &self,
                    _serializer: &mut dyn ShapeSerializer,
                ) -> Result<(), SerdeError> {
                    let binder = self.binder as *const HttpBindingSerializer<'_, S>
                        as *mut HttpBindingSerializer<'_, S>;
                    // SAFETY: The body serializer called serialize_members on
                    // this proxy, passing &mut self (body). The binder wraps
                    // that same body serializer. We need mutable access to the
                    // binder to route writes. This is safe because:
                    // 1. The body serializer's write_struct only calls
                    //    serialize_members once, synchronously.
                    // 2. Body member writes from the binder go back to the
                    //    body serializer, which is in a valid state (between
                    //    the { and } it emitted).
                    self.value.serialize_members(unsafe { &mut *binder })
                }
            }
            // Clear is_top_level so nested write_struct calls (from body members)
            // take the else branch and delegate directly to the body serializer.
            // input_schema is preserved so resolve_member continues to work.
            self.is_top_level = false;
            let proxy = Proxy {
                binder: self,
                value,
            };
            let binder_ptr = &mut *proxy.binder as *mut HttpBindingSerializer<'_, S>;
            // SAFETY: `proxy` holds a shared reference to `binder` (via &mut that
            // we reborrow). We need to call `binder.body.write_struct(schema, &proxy)`
            // but can't do so through normal references because `proxy` borrows `binder`.
            // The raw pointer dereference is safe because:
            // 1. `binder_ptr` points to a valid, live `HttpBindingSerializer` (it was
            //    just derived from `proxy.binder`).
            // 2. `body.write_struct` is called synchronously and returns before `proxy`
            //    is dropped, so the binder is not moved or deallocated.
            // 3. The only re-entrant access is through `proxy.serialize_members`, which
            //    uses the same raw-pointer pattern with its own safety justification above.
            unsafe { (*binder_ptr).body.write_struct(schema, &proxy) }
        } else {
            // Nested struct (a body member targeting a structure): delegate
            // entirely to the body serializer.
            let schema = self.resolve_member(schema);
            if schema.http_payload().is_some() {
                // @httpPayload struct/union: codegen routes these by passing the
                // target struct's schema directly (not the member schema), so this
                // path is normally unreachable. Kept as a safety net.
                self.body.write_struct(schema, value)?;
                return Ok(());
            }
            self.body.write_struct(schema, value)
        }
    }

    fn write_list(
        &mut self,
        schema: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        // @httpHeader on a list: collect elements as comma-separated header value
        if let Some(header) = schema.http_header() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            let mut collector = ListElementCollector::for_header();
            write_elements(&mut collector)?;
            // RFC 7230: string values containing commas or quotes need quoting.
            // Timestamps are NOT quoted even though http-date contains commas.
            let header_val = collector
                .values
                .iter()
                .zip(collector.quotable.iter())
                .map(|(s, &quotable)| {
                    if quotable && (s.contains(',') || s.contains('"')) {
                        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
                    } else {
                        s.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.headers.insert(header.value(), header_val);
            return Ok(());
        }
        // @httpQuery on a list: add each element as a separate query param
        if let Some(query) = schema.http_query() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            let mut collector = ListElementCollector::for_query();
            write_elements(&mut collector)?;
            for val in collector.values {
                self.query_params.push((Cow::Borrowed(query.value()), val));
            }
            return Ok(());
        }
        self.body.write_list(schema, write_elements)
    }

    fn write_map(
        &mut self,
        schema: &Schema,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        // @httpPrefixHeaders: serialize map entries as prefixed headers
        if let Some(prefix) = schema.http_prefix_headers() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            // Collect entries via a temporary serializer
            let mut collector = MapEntryCollector::new(prefix.value().to_string());
            write_entries(&mut collector)?;
            // Names are dynamic (prefix + map key) — owned Strings.
            for (k, v) in collector.entries {
                self.headers.insert(k, v);
            }
            return Ok(());
        }
        // @httpQueryParams: serialize map entries as query params
        if schema.http_query_params().is_some() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            let mut collector = MapEntryCollector::new(String::new());
            write_entries(&mut collector)?;
            // Filter out keys that overlap with explicit @httpQuery params
            // (query params take precedence over query params map entries)
            let explicit_query_keys: Vec<&str> = self
                .input_schema
                .map(|s| {
                    s.members()
                        .iter()
                        .filter_map(|m| m.http_query().map(|q| q.value()))
                        .collect()
                })
                .unwrap_or_default();
            for (k, v) in collector.entries {
                if !explicit_query_keys.contains(&k.as_str()) {
                    self.query_params.push((Cow::Owned(k), v));
                }
            }
            return Ok(());
        }
        self.body.write_map(schema, write_entries)
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_boolean(schema, value)
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_byte(schema, value)
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_short(schema, value)
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_integer(schema, value)
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_long(schema, value)
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &format_float_f32(value));
        }
        self.body.write_float(schema, value)
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, &format_float_f64(value));
        }
        self.body.write_double(schema, value)
    }

    fn write_big_integer(
        &mut self,
        schema: &Schema,
        value: &aws_smithy_types::BigInteger,
    ) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, value.as_ref());
        }
        self.body.write_big_integer(schema, value)
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema,
        value: &aws_smithy_types::BigDecimal,
    ) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.add_binding(binding, schema, value.as_ref());
        }
        self.body.write_big_decimal(schema, value)
    }

    fn write_string(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            // @mediaType on a header: base64-encode the value
            if schema.media_type().is_some() {
                let encoded = aws_smithy_types::base64::encode(value.as_bytes());
                return self.add_binding(binding, schema, &encoded);
            }
            return self.add_binding(binding, schema, value);
        }
        if schema.http_payload().is_some() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            // SAFETY: We extend the lifetime of `value.as_bytes()` from its anonymous
            // lifetime to `'a`. This is sound because:
            // 1. `value` is borrowed from the input struct passed to `serialize_request`.
            // 2. `HttpBindingSerializer` is a local variable within `serialize_request`
            //    and is dropped before `serialize_request` returns.
            // 3. The input struct (and thus `value`) outlives the serializer.
            // 4. `raw_payload` is read in `serialize_request` immediately after
            //    `serialize_members` returns, before the input is dropped.
            // We use transmute rather than copying to avoid allocating for potentially
            // multi-GB string payloads.
            self.raw_payload =
                Some(unsafe { std::mem::transmute::<&[u8], &'a [u8]>(value.as_bytes()) });
            return Ok(());
        }
        self.body.write_string(schema, value)
    }

    fn write_blob(&mut self, schema: &Schema, value: &[u8]) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if schema.http_header().is_some() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            let encoded = aws_smithy_types::base64::encode(value);
            self.headers
                .insert(schema.http_header().unwrap().value(), encoded);
            return Ok(());
        }
        if schema.http_payload().is_some() {
            if !self.should_route_binding(schema) {
                return Ok(());
            }
            // SAFETY: We extend the lifetime of `value` (a `&[u8]`) from its
            // anonymous lifetime to `'a`. This is sound because:
            // 1. `value` is borrowed from the input struct passed to `serialize_request`.
            // 2. `HttpBindingSerializer` is a local variable within `serialize_request`
            //    and is dropped before `serialize_request` returns.
            // 3. The input struct (and thus `value`) outlives the serializer.
            // 4. `raw_payload` is read in `serialize_request` immediately after
            //    `serialize_members` returns, before the input is dropped.
            // We use transmute rather than copying to avoid allocating for potentially
            // multi-GB blob payloads.
            self.raw_payload = Some(unsafe { std::mem::transmute::<&[u8], &'a [u8]>(value) });
            return Ok(());
        }
        self.body.write_blob(schema, value)
    }

    fn write_timestamp(
        &mut self,
        schema: &Schema,
        value: &aws_smithy_types::DateTime,
    ) -> Result<(), SerdeError> {
        let schema = self.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            // Headers default to http-date, query/label default to date-time
            let format = if let Some(ts_trait) = schema.timestamp_format() {
                match ts_trait.format() {
                    crate::traits::TimestampFormat::EpochSeconds => {
                        aws_smithy_types::date_time::Format::EpochSeconds
                    }
                    crate::traits::TimestampFormat::HttpDate => {
                        aws_smithy_types::date_time::Format::HttpDate
                    }
                    crate::traits::TimestampFormat::DateTime => {
                        aws_smithy_types::date_time::Format::DateTime
                    }
                }
            } else {
                match binding {
                    HttpBinding::Header(_) => aws_smithy_types::date_time::Format::HttpDate,
                    _ => aws_smithy_types::date_time::Format::DateTime,
                }
            };
            let formatted = value
                .fmt(format)
                .map_err(|e| SerdeError::custom(format!("failed to format timestamp: {e}")))?;
            return self.add_binding(binding, schema, &formatted);
        }
        self.body.write_timestamp(schema, value)
    }

    fn write_document(
        &mut self,
        schema: &Schema,
        value: &aws_smithy_types::Document,
    ) -> Result<(), SerdeError> {
        self.body.write_document(schema, value)
    }

    fn write_null(&mut self, schema: &Schema) -> Result<(), SerdeError> {
        self.body.write_null(schema)
    }
}

/// Which HTTP location a member is bound to.
enum HttpBinding {
    Header(&'static str),
    Query(&'static str),
    Label,
}

/// Determine the HTTP binding for a member schema, if any.
fn http_string_binding(schema: &Schema) -> Option<HttpBinding> {
    if let Some(h) = schema.http_header() {
        return Some(HttpBinding::Header(h.value()));
    }
    if let Some(q) = schema.http_query() {
        return Some(HttpBinding::Query(q.value()));
    }
    if schema.http_label().is_some() {
        return Some(HttpBinding::Label);
    }
    None
}

impl<'a, S> HttpBindingSerializer<'a, S> {
    fn add_binding(
        &mut self,
        binding: HttpBinding,
        schema: &Schema,
        value: &str,
    ) -> Result<(), SerdeError> {
        // Dedupe per-member: see `should_route_binding`. Without this, a
        // multi-pass body codec invokes `serialize_members` more than once
        // and each pass would append to `headers` / `query_params` / `labels`.
        if !self.should_route_binding(schema) {
            return Ok(());
        }
        match binding {
            HttpBinding::Header(name) => {
                self.headers.insert(name, value.to_string());
            }
            HttpBinding::Query(name) => {
                self.query_params
                    .push((Cow::Borrowed(name), value.to_string()));
            }
            HttpBinding::Label => {
                let name = schema
                    .member_name()
                    .ok_or_else(|| SerdeError::custom("httpLabel on non-member schema"))?;
                self.labels.push((Cow::Borrowed(name), value.to_string()));
            }
        }
        Ok(())
    }
}

/// Whether a `ListElementCollector` is gathering values for a header or query param.
/// Affects default timestamp format: `http-date` for headers, `date-time` for query.
#[derive(Copy, Clone)]
enum HttpListTarget {
    Header,
    Query,
}

/// Collects list element values as strings for @httpHeader and @httpQuery on lists.
struct ListElementCollector {
    values: Vec<String>,
    /// Whether each value should be quoted if it contains commas (strings yes, timestamps no)
    quotable: Vec<bool>,
    target: HttpListTarget,
}

impl ListElementCollector {
    fn for_header() -> Self {
        Self::new(HttpListTarget::Header)
    }

    fn for_query() -> Self {
        Self::new(HttpListTarget::Query)
    }

    fn new(target: HttpListTarget) -> Self {
        Self {
            values: Vec::new(),
            quotable: Vec::new(),
            target,
        }
    }

    fn push(&mut self, value: String) {
        self.quotable.push(true);
        self.values.push(value);
    }

    fn push_unquotable(&mut self, value: String) {
        self.quotable.push(false);
        self.values.push(value);
    }
}

impl ShapeSerializer for ListElementCollector {
    fn write_string(&mut self, _schema: &Schema, value: &str) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_boolean(&mut self, _: &Schema, value: bool) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_byte(&mut self, _: &Schema, value: i8) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_short(&mut self, _: &Schema, value: i16) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_integer(&mut self, _: &Schema, value: i32) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_long(&mut self, _: &Schema, value: i64) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_float(&mut self, _: &Schema, value: f32) -> Result<(), SerdeError> {
        self.push(format_float_f32(value));
        Ok(())
    }
    fn write_double(&mut self, _: &Schema, value: f64) -> Result<(), SerdeError> {
        self.push(format_float_f64(value));
        Ok(())
    }
    fn write_timestamp(
        &mut self,
        schema: &Schema,
        value: &aws_smithy_types::DateTime,
    ) -> Result<(), SerdeError> {
        let format = match schema.timestamp_format() {
            Some(ts) => match ts.format() {
                crate::traits::TimestampFormat::EpochSeconds => {
                    aws_smithy_types::date_time::Format::EpochSeconds
                }
                crate::traits::TimestampFormat::HttpDate => {
                    aws_smithy_types::date_time::Format::HttpDate
                }
                crate::traits::TimestampFormat::DateTime => {
                    aws_smithy_types::date_time::Format::DateTime
                }
            },
            // Default: headers use http-date, query params use date-time
            None => match self.target {
                HttpListTarget::Header => aws_smithy_types::date_time::Format::HttpDate,
                HttpListTarget::Query => aws_smithy_types::date_time::Format::DateTime,
            },
        };
        self.push_unquotable(
            value
                .fmt(format)
                .map_err(|e| SerdeError::custom(format!("failed to format timestamp: {e}")))?,
        );
        Ok(())
    }
    fn write_blob(&mut self, _schema: &Schema, value: &[u8]) -> Result<(), SerdeError> {
        self.push(aws_smithy_types::base64::encode(value));
        Ok(())
    }
    // Remaining methods are no-ops for list element collection
    fn write_struct(&mut self, _: &Schema, _: &dyn SerializableStruct) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_list(
        &mut self,
        _: &Schema,
        _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_map(
        &mut self,
        _: &Schema,
        _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_big_integer(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::BigInteger,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_big_decimal(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::BigDecimal,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_document(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::Document,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
        Ok(())
    }
}

/// Format a float for HTTP headers/query/labels.
/// Rust's Display writes "inf"/"-inf" but HTTP requires "Infinity"/"-Infinity".
fn format_float_f32(value: f32) -> String {
    if value.is_infinite() {
        if value.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        value.to_string()
    }
}

fn format_float_f64(value: f64) -> String {
    if value.is_infinite() {
        if value.is_sign_positive() {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        value.to_string()
    }
}

/// Collects map key-value pairs written via ShapeSerializer for
/// @httpPrefixHeaders and @httpQueryParams.
struct MapEntryCollector {
    prefix: String,
    entries: Vec<(String, String)>,
    pending_key: Option<String>,
}

impl MapEntryCollector {
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            entries: Vec::new(),
            pending_key: None,
        }
    }
}

impl ShapeSerializer for MapEntryCollector {
    fn write_string(&mut self, _schema: &Schema, value: &str) -> Result<(), SerdeError> {
        if let Some(key) = self.pending_key.take() {
            self.entries
                .push((format!("{}{}", self.prefix, key), value.to_string()));
        } else {
            self.pending_key = Some(value.to_string());
        }
        Ok(())
    }

    // All other methods are no-ops — maps in HTTP bindings only have string keys/values.
    // Exception: write_list handles Map<String, List<String>> for @httpQueryParams.
    fn write_struct(&mut self, _: &Schema, _: &dyn SerializableStruct) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_list(
        &mut self,
        _: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        // Map<String, List<String>>: each list element becomes a separate entry
        // with the same key (for @httpQueryParams).
        if let Some(key) = self.pending_key.take() {
            let mut collector = ListElementCollector::for_query(); // query params context
            write_elements(&mut collector)?;
            for val in collector.values {
                self.entries.push((format!("{}{}", self.prefix, key), val));
            }
        }
        Ok(())
    }
    fn write_map(
        &mut self,
        _: &Schema,
        _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_boolean(&mut self, _: &Schema, _: bool) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_byte(&mut self, _: &Schema, _: i8) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_short(&mut self, _: &Schema, _: i16) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_integer(&mut self, _: &Schema, _: i32) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_long(&mut self, _: &Schema, _: i64) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_float(&mut self, _: &Schema, _: f32) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_double(&mut self, _: &Schema, _: f64) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_big_integer(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::BigInteger,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_big_decimal(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::BigDecimal,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_blob(&mut self, _: &Schema, _: &[u8]) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_timestamp(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::DateTime,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_document(
        &mut self,
        _: &Schema,
        _: &aws_smithy_types::Document,
    ) -> Result<(), SerdeError> {
        Ok(())
    }
    fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
        Ok(())
    }
}

impl<C> ClientProtocolInner for HttpBindingProtocol<C>
where
    C: Codec + Send + Sync + std::fmt::Debug + 'static,
    for<'a> C::Deserializer<'a>: ShapeDeserializer,
{
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId {
        &self.protocol_id
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        let body = self.codec.create_serializer();
        self.serialize_request_with_body(body, input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        _output_schema: &Schema,
        _cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        // For non-streaming responses the orchestrator has already loaded
        // the body into an in-memory `Once(...)`, so `bytes()` returns the
        // payload to feed into the codec. For streaming responses (whose
        // outputs have an `@httpPayload` streaming blob or event-stream
        // member) the body is left as a streaming `BoxBody` — possibly
        // further wrapped by interceptors such as `ResponseChecksumInterceptor`
        // — and `bytes()` returns `None`. The streaming codegen path
        // doesn't actually feed the body through this deserializer (it
        // passes `&[]` to `deserialize_with_response`), so we hand back an
        // empty-input deserializer instead of erroring. Empty input is
        // interpreted by the codec as "no body members to read", which
        // matches the streaming path's contract.
        let body = response.body().bytes().unwrap_or(&[]);
        Ok(Box::new(self.codec.create_deserializer(body)))
    }

    fn payload_codec(&self) -> Option<&dyn crate::codec::DynCodec> {
        Some(&self.codec)
    }

    fn update_endpoint(
        &self,
        request: &mut Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        apply_http_endpoint(request, endpoint, cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::SerializableStruct;
    use crate::{prelude::*, ShapeType};

    #[test]
    fn visited_members_inline_dedup() {
        let mut v = VisitedMembers::new();
        // First insert returns true; identical second insert returns false.
        assert!(v.insert(3));
        assert!(!v.insert(3));
        // Distinct indices return true.
        assert!(v.insert(7));
        assert!(v.insert(0));
        // Re-asserting any of the above returns false.
        assert!(!v.insert(3));
        assert!(!v.insert(7));
        assert!(!v.insert(0));
    }

    #[test]
    fn visited_members_spills_to_overflow() {
        // Exceeding INLINE_CAPACITY pushes additional entries into the heap
        // overflow vec. Both halves must dedup correctly.
        let mut v = VisitedMembers::new();
        let n = VisitedMembers::INLINE_CAPACITY;
        // Fill inline storage.
        for i in 0..n {
            assert!(v.insert(i), "fresh inline insert at {i} must return true");
        }
        // Inserting again into inline range returns false (no allocation).
        for i in 0..n {
            assert!(
                !v.insert(i),
                "duplicate inline insert at {i} must return false"
            );
        }
        // Cross the capacity boundary — these go into overflow.
        assert!(v.insert(n));
        assert!(v.insert(n + 5));
        // Duplicates of overflow entries return false.
        assert!(!v.insert(n));
        assert!(!v.insert(n + 5));
        // And duplicates of inline entries still return false even after
        // the overflow vec is non-empty.
        assert!(!v.insert(0));
    }

    struct TestSerializer {
        output: Vec<u8>,
    }

    impl FinishSerializer for TestSerializer {
        fn finish(self) -> Vec<u8> {
            self.output
        }
    }

    impl ShapeSerializer for TestSerializer {
        fn write_struct(
            &mut self,
            _: &Schema,
            value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            self.output.push(b'{');
            value.serialize_members(self)?;
            self.output.push(b'}');
            Ok(())
        }
        fn write_list(
            &mut self,
            _: &Schema,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_map(
            &mut self,
            _: &Schema,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_boolean(&mut self, _: &Schema, _: bool) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_byte(&mut self, _: &Schema, _: i8) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_short(&mut self, _: &Schema, _: i16) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_integer(&mut self, _: &Schema, _: i32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_long(&mut self, _: &Schema, _: i64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_float(&mut self, _: &Schema, _: f32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_double(&mut self, _: &Schema, _: f64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_integer(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_decimal(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_string(&mut self, _: &Schema, v: &str) -> Result<(), SerdeError> {
            self.output.extend_from_slice(v.as_bytes());
            Ok(())
        }
        fn write_blob(&mut self, _: &Schema, _: &[u8]) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_timestamp(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_document(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    struct TestDeserializer<'a> {
        input: &'a [u8],
    }

    impl ShapeDeserializer for TestDeserializer<'_> {
        fn read_struct(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_list(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _: &Schema) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _: &Schema) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _: &Schema) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _: &Schema) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _: &Schema) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _: &Schema) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _: &Schema) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(
            &mut self,
            _: &Schema,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }
        fn read_big_decimal(
            &mut self,
            _: &Schema,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }
        fn read_string(&mut self, _: &Schema) -> Result<String, SerdeError> {
            Ok(String::from_utf8_lossy(self.input).into_owned())
        }
        fn read_blob(&mut self, _: &Schema) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(vec![]))
        }
        fn read_timestamp(&mut self, _: &Schema) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }
        fn read_document(&mut self, _: &Schema) -> Result<aws_smithy_types::Document, SerdeError> {
            Ok(aws_smithy_types::Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    #[derive(Debug)]
    struct TestCodec;

    impl Codec for TestCodec {
        type Serializer = TestSerializer;
        type Deserializer<'a> = TestDeserializer<'a>;
        fn create_serializer(&self) -> Self::Serializer {
            TestSerializer { output: Vec::new() }
        }
        fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
            TestDeserializer { input }
        }
    }

    static TEST_SCHEMA: Schema =
        Schema::new(crate::shape_id!("test", "TestStruct"), ShapeType::Structure);

    struct EmptyStruct;
    impl SerializableStruct for EmptyStruct {
        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static NAME_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::String,
        "name",
        0,
    );
    static MEMBERS: &[&Schema] = &[&NAME_MEMBER];
    static STRUCT_WITH_MEMBER: Schema = Schema::new_struct(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::Structure,
        MEMBERS,
    );

    struct NameStruct;
    impl SerializableStruct for NameStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&NAME_MEMBER, "Alice")
        }
    }

    fn make_protocol() -> HttpBindingProtocol<TestCodec> {
        HttpBindingProtocol::new(
            crate::shape_id!("test", "proto"),
            TestCodec,
            "application/test",
        )
    }

    #[test]
    fn serialize_sets_content_type() {
        // A struct with body members gets Content-Type
        let request = make_protocol()
            .serialize_request(
                &EmptyStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/test"
        );
    }

    #[test]
    fn serialize_no_body_members_omits_content_type() {
        // A struct with no members gets no Content-Type per REST-JSON spec
        let request = make_protocol()
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert!(request.headers().get("Content-Type").is_none());
    }

    /// The bug fix at the center of PR #4686 review: when a presigning
    /// interceptor (or any other caller) stores a `SharedHeaderOmitSettings`
    /// in the config bag, the runtime must not insert protocol-default
    /// Content-Type or Content-Length headers — even on the standard-body
    /// path used by ordinary structure inputs.
    #[test]
    fn presigning_omit_settings_suppress_default_content_headers() {
        use crate::header_omit_settings::{HeaderOmitSettings, SharedHeaderOmitSettings};
        use aws_smithy_types::config_bag::Layer;

        #[derive(Debug)]
        struct OmitBoth;
        impl HeaderOmitSettings for OmitBoth {
            fn should_omit_default_content_type(&self) -> bool {
                true
            }
            fn should_omit_default_content_length(&self) -> bool {
                true
            }
        }

        let mut layer = Layer::new("test_omit");
        layer.store_put(SharedHeaderOmitSettings::new(OmitBoth));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let request = make_protocol()
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &cfg,
            )
            .unwrap();
        assert!(
            request.headers().get("Content-Type").is_none(),
            "presigning omit suppresses default Content-Type"
        );
        assert!(
            request.headers().get("Content-Length").is_none(),
            "presigning omit suppresses default Content-Length"
        );
    }

    /// Companion to `presigning_omit_settings_suppress_default_content_headers`:
    /// when no `SharedHeaderOmitSettings` is in the config bag, the runtime
    /// inserts the protocol's default Content-Type plus a body-length-derived
    /// Content-Length. `NameStruct` writes a single `"Alice"` member, which
    /// the test codec frames as `{Alice}` for a 7-byte body — exercises the
    /// `len > 0` branch of the Content-Length insertion logic.
    #[test]
    fn default_content_headers_inserted_when_omit_settings_absent() {
        let request = make_protocol()
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request
                .headers()
                .get("Content-Type")
                .expect("Content-Type set"),
            "application/test"
        );
        assert_eq!(
            request
                .headers()
                .get("Content-Length")
                .expect("Content-Length set"),
            "7"
        );
    }

    /// When a schema is annotated `with_no_body_members()`, the body codec
    /// must not be invoked (no XML/JSON wrapper element gets opened, no
    /// `serialize_members` re-entry through a Proxy). HTTP-bound members
    /// still get routed to headers/query/labels via the binder. Verified by
    /// substituting a body codec that panics on any write call.
    #[test]
    fn serialize_skips_body_codec_when_no_body_members() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static WRITE_CALLS: AtomicUsize = AtomicUsize::new(0);

        struct PanicSerializer;
        impl FinishSerializer for PanicSerializer {
            fn finish(self) -> Vec<u8> {
                panic!("body codec finish() called — short-circuit failed");
            }
        }
        impl ShapeSerializer for PanicSerializer {
            fn write_struct(
                &mut self,
                _: &Schema,
                _: &dyn SerializableStruct,
            ) -> Result<(), SerdeError> {
                WRITE_CALLS.fetch_add(1, Ordering::SeqCst);
                panic!("body codec write_struct() called — short-circuit failed");
            }
            fn write_list(
                &mut self,
                _: &Schema,
                _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_list() called");
            }
            fn write_map(
                &mut self,
                _: &Schema,
                _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_map() called");
            }
            fn write_boolean(&mut self, _: &Schema, _: bool) -> Result<(), SerdeError> {
                panic!("body codec write_boolean() called");
            }
            fn write_byte(&mut self, _: &Schema, _: i8) -> Result<(), SerdeError> {
                panic!("body codec write_byte() called");
            }
            fn write_short(&mut self, _: &Schema, _: i16) -> Result<(), SerdeError> {
                panic!("body codec write_short() called");
            }
            fn write_integer(&mut self, _: &Schema, _: i32) -> Result<(), SerdeError> {
                panic!("body codec write_integer() called");
            }
            fn write_long(&mut self, _: &Schema, _: i64) -> Result<(), SerdeError> {
                panic!("body codec write_long() called");
            }
            fn write_float(&mut self, _: &Schema, _: f32) -> Result<(), SerdeError> {
                panic!("body codec write_float() called");
            }
            fn write_double(&mut self, _: &Schema, _: f64) -> Result<(), SerdeError> {
                panic!("body codec write_double() called");
            }
            fn write_big_integer(
                &mut self,
                _: &Schema,
                _: &aws_smithy_types::BigInteger,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_big_integer() called");
            }
            fn write_big_decimal(
                &mut self,
                _: &Schema,
                _: &aws_smithy_types::BigDecimal,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_big_decimal() called");
            }
            fn write_string(&mut self, _: &Schema, _: &str) -> Result<(), SerdeError> {
                panic!("body codec write_string() called");
            }
            fn write_blob(&mut self, _: &Schema, _: &[u8]) -> Result<(), SerdeError> {
                panic!("body codec write_blob() called");
            }
            fn write_timestamp(
                &mut self,
                _: &Schema,
                _: &aws_smithy_types::DateTime,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_timestamp() called");
            }
            fn write_document(
                &mut self,
                _: &Schema,
                _: &aws_smithy_types::Document,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_document() called");
            }
            fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
                panic!("body codec write_null() called");
            }
        }

        #[derive(Debug)]
        struct PanicCodec;
        impl Codec for PanicCodec {
            type Serializer = PanicSerializer;
            type Deserializer<'a> = TestDeserializer<'a>;
            fn create_serializer(&self) -> Self::Serializer {
                PanicSerializer
            }
            fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
                TestDeserializer { input }
            }
        }

        // Header-only struct: one `@httpHeader` member, marked
        // `with_no_body_members()`. The runtime should never touch the body
        // codec.
        static HEADER_MEMBER: Schema = Schema::new_member(
            crate::shape_id!("test", "HeaderOnlyStruct"),
            ShapeType::String,
            "x_header",
            0,
        )
        .with_http_header("X-Header");
        static HEADER_MEMBERS: &[&Schema] = &[&HEADER_MEMBER];
        static HEADER_ONLY_SCHEMA: Schema = Schema::new_struct(
            crate::shape_id!("test", "HeaderOnlyStruct"),
            ShapeType::Structure,
            HEADER_MEMBERS,
        )
        .with_no_body_members();

        struct HeaderOnlyStruct;
        impl SerializableStruct for HeaderOnlyStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&HEADER_MEMBER, "hello")
            }
        }

        let protocol = HttpBindingProtocol::new(
            crate::shape_id!("test", "testProtocol"),
            PanicCodec,
            "application/test",
        );
        let request = protocol
            .serialize_request(
                &HeaderOnlyStruct,
                &HEADER_ONLY_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();

        // No body, no Content-Type, header is set
        assert_eq!(request.body().bytes().unwrap_or(&[]), b"");
        assert!(request.headers().get("Content-Type").is_none());
        assert_eq!(request.headers().get("X-Header").unwrap(), "hello");
        // Sanity: the panic-on-write codec was never written to
        assert_eq!(WRITE_CALLS.load(Ordering::SeqCst), 0);
    }

    /// Inverse case: a struct WITHOUT `with_no_body_members()` (default
    /// `has_body_members == true`) and an actual body member must still
    /// invoke the body codec. Guards against accidentally short-circuiting
    /// schemas that were never opted in.
    #[test]
    fn serialize_invokes_body_codec_when_has_body_members() {
        let request = make_protocol()
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        // TestSerializer writes "{Alice}" — the wrapper braces prove the
        // body codec was invoked.
        assert_eq!(request.body().bytes().unwrap(), b"{Alice}");
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/test"
        );
    }

    #[test]
    fn serialize_sets_uri() {
        let request = make_protocol()
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://example.com/path",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com/path");
    }

    #[test]
    fn serialize_body() {
        let request = make_protocol()
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.body().bytes().unwrap(), b"{Alice}");
    }

    #[test]
    fn deserialize_response() {
        let response = Response::new(
            200u16.try_into().unwrap(),
            SdkBody::from(r#"{"name":"Bob"}"#),
        );
        let mut deser = make_protocol()
            .deserialize_response(&response, &TEST_SCHEMA, &ConfigBag::base())
            .unwrap();
        assert_eq!(deser.read_string(&STRING).unwrap(), r#"{"name":"Bob"}"#);
    }

    #[test]
    fn update_endpoint() {
        let mut request = make_protocol()
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://old.example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        let endpoint = aws_smithy_types::endpoint::Endpoint::builder()
            .url("https://new.example.com")
            .build();
        make_protocol()
            .update_endpoint(&mut request, &endpoint, &ConfigBag::base())
            .unwrap();
        assert_eq!(request.uri(), "https://new.example.com/");
    }

    #[test]
    fn protocol_id() {
        let protocol = HttpBindingProtocol::new(
            crate::shape_id!("aws.protocols", "restJson1"),
            TestCodec,
            "application/json",
        );
        assert_eq!(protocol.protocol_id().as_str(), "aws.protocols#restJson1");
    }

    #[test]
    fn invalid_uri_returns_error() {
        assert!(make_protocol()
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "not a valid uri\n\n",
                &ConfigBag::base()
            )
            .is_err());
    }

    // -- @httpHeader tests --

    static HEADER_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "xToken",
        0,
    )
    .with_http_header("X-Token");

    static HEADER_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&HEADER_MEMBER],
    );

    struct HeaderStruct;
    impl SerializableStruct for HeaderStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&HEADER_MEMBER, "my-token-value")
        }
    }

    #[test]
    fn http_header_string() {
        let request = make_protocol()
            .serialize_request(
                &HeaderStruct,
                &HEADER_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.headers().get("X-Token").unwrap(), "my-token-value");
    }

    static INT_HEADER_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Integer,
        "retryCount",
        0,
    )
    .with_http_header("X-Retry-Count");

    static INT_HEADER_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&INT_HEADER_MEMBER],
    );

    struct IntHeaderStruct;
    impl SerializableStruct for IntHeaderStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_integer(&INT_HEADER_MEMBER, 3)
        }
    }

    #[test]
    fn http_header_integer() {
        let request = make_protocol()
            .serialize_request(
                &IntHeaderStruct,
                &INT_HEADER_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.headers().get("X-Retry-Count").unwrap(), "3");
    }

    static BOOL_HEADER_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Boolean,
        "verbose",
        0,
    )
    .with_http_header("X-Verbose");

    static BOOL_HEADER_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&BOOL_HEADER_MEMBER],
    );

    struct BoolHeaderStruct;
    impl SerializableStruct for BoolHeaderStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_boolean(&BOOL_HEADER_MEMBER, true)
        }
    }

    #[test]
    fn http_header_boolean() {
        let request = make_protocol()
            .serialize_request(
                &BoolHeaderStruct,
                &BOOL_HEADER_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.headers().get("X-Verbose").unwrap(), "true");
    }

    // -- @httpQuery tests --

    static QUERY_MEMBER: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "color", 0)
            .with_http_query("color");

    static QUERY_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&QUERY_MEMBER],
    );

    struct QueryStruct;
    impl SerializableStruct for QueryStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&QUERY_MEMBER, "blue")
        }
    }

    #[test]
    fn http_query_string() {
        let request = make_protocol()
            .serialize_request(
                &QueryStruct,
                &QUERY_SCHEMA,
                "https://example.com/things",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com/things?color=blue");
    }

    static INT_QUERY_MEMBER: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Integer, "size", 0)
            .with_http_query("size");

    static INT_QUERY_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&INT_QUERY_MEMBER],
    );

    struct IntQueryStruct;
    impl SerializableStruct for IntQueryStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_integer(&INT_QUERY_MEMBER, 42)
        }
    }

    #[test]
    fn http_query_integer() {
        let request = make_protocol()
            .serialize_request(
                &IntQueryStruct,
                &INT_QUERY_SCHEMA,
                "https://example.com/things",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com/things?size=42");
    }

    // -- Multiple @httpQuery params --

    static Q1: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "a", 0)
            .with_http_query("a");
    static Q2: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "b", 1)
            .with_http_query("b");
    static MULTI_QUERY_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&Q1, &Q2],
    );

    struct MultiQueryStruct;
    impl SerializableStruct for MultiQueryStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&Q1, "x")?;
            s.write_string(&Q2, "y")
        }
    }

    #[test]
    fn http_query_multiple_params() {
        let request = make_protocol()
            .serialize_request(
                &MultiQueryStruct,
                &MULTI_QUERY_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com?a=x&b=y");
    }

    // -- @httpQuery with percent-encoding --

    #[test]
    fn http_query_percent_encodes_values() {
        struct SpaceQueryStruct;
        impl SerializableStruct for SpaceQueryStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&QUERY_MEMBER, "hello world")
            }
        }
        let request = make_protocol()
            .serialize_request(
                &SpaceQueryStruct,
                &QUERY_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com?color=hello%20world");
    }

    // -- @httpLabel tests --

    static LABEL_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "bucketName",
        0,
    )
    .with_http_label();

    static LABEL_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&LABEL_MEMBER],
    );

    struct LabelStruct;
    impl SerializableStruct for LabelStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&LABEL_MEMBER, "my-bucket")
        }
    }

    #[test]
    fn http_label_substitution() {
        let request = make_protocol()
            .serialize_request(
                &LabelStruct,
                &LABEL_SCHEMA,
                "https://example.com/{bucketName}/objects",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com/my-bucket/objects");
    }

    #[test]
    fn http_label_percent_encodes() {
        struct SpecialLabelStruct;
        impl SerializableStruct for SpecialLabelStruct {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(&LABEL_MEMBER, "my bucket/name")
            }
        }
        let request = make_protocol()
            .serialize_request(
                &SpecialLabelStruct,
                &LABEL_SCHEMA,
                "https://example.com/{bucketName}",
                &ConfigBag::base(),
            )
            .unwrap();
        assert!(request.uri().contains("my%20bucket%2Fname"));
    }

    static INT_LABEL_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Integer,
        "itemId",
        0,
    )
    .with_http_label();

    static INT_LABEL_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&INT_LABEL_MEMBER],
    );

    struct IntLabelStruct;
    impl SerializableStruct for IntLabelStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_integer(&INT_LABEL_MEMBER, 123)
        }
    }

    #[test]
    fn http_label_integer() {
        let request = make_protocol()
            .serialize_request(
                &IntLabelStruct,
                &INT_LABEL_SCHEMA,
                "https://example.com/items/{itemId}",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com/items/123");
    }

    // -- Combined: @httpHeader + @httpQuery + @httpLabel + body --

    static COMBINED_LABEL: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "id", 0)
            .with_http_label();
    static COMBINED_HEADER: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "token", 1)
            .with_http_header("X-Token");
    static COMBINED_QUERY: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "filter",
        2,
    )
    .with_http_query("filter");
    static COMBINED_BODY: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "data", 3);
    static COMBINED_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[
            &COMBINED_LABEL,
            &COMBINED_HEADER,
            &COMBINED_QUERY,
            &COMBINED_BODY,
        ],
    );

    struct CombinedStruct;
    impl SerializableStruct for CombinedStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&COMBINED_LABEL, "item-42")?;
            s.write_string(&COMBINED_HEADER, "secret")?;
            s.write_string(&COMBINED_QUERY, "active")?;
            s.write_string(&COMBINED_BODY, "payload-data")
        }
    }

    #[test]
    fn combined_bindings() {
        let request = make_protocol()
            .serialize_request(
                &CombinedStruct,
                &COMBINED_SCHEMA,
                "https://example.com/{id}/details",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request.uri(),
            "https://example.com/item-42/details?filter=active"
        );
        // Header
        assert_eq!(request.headers().get("X-Token").unwrap(), "secret");
        // Body contains only the unbound member
        let body = request.body().bytes().unwrap();
        assert!(body
            .windows(b"payload-data".len())
            .any(|w| w == b"payload-data"));
    }

    // -- @httpPrefixHeaders tests --

    static PREFIX_MEMBER: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Map, "metadata", 0)
            .with_http_prefix_headers("X-Meta-");

    static PREFIX_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&PREFIX_MEMBER],
    );

    struct PrefixHeaderStruct;
    impl SerializableStruct for PrefixHeaderStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_map(&PREFIX_MEMBER, &|s| {
                s.write_string(&STRING, "Color")?;
                s.write_string(&STRING, "red")?;
                s.write_string(&STRING, "Size")?;
                s.write_string(&STRING, "large")?;
                Ok(())
            })
        }
    }

    #[test]
    fn http_prefix_headers() {
        let request = make_protocol()
            .serialize_request(
                &PrefixHeaderStruct,
                &PREFIX_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.headers().get("X-Meta-Color").unwrap(), "red");
        assert_eq!(request.headers().get("X-Meta-Size").unwrap(), "large");
    }

    // -- @httpQueryParams tests --

    static QUERY_PARAMS_MEMBER: Schema =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Map, "params", 0)
            .with_http_query_params();

    static QUERY_PARAMS_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&QUERY_PARAMS_MEMBER],
    );

    struct QueryParamsStruct;
    impl SerializableStruct for QueryParamsStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_map(&QUERY_PARAMS_MEMBER, &|s| {
                s.write_string(&STRING, "page")?;
                s.write_string(&STRING, "2")?;
                s.write_string(&STRING, "limit")?;
                s.write_string(&STRING, "50")?;
                Ok(())
            })
        }
    }

    #[test]
    fn http_query_params() {
        let request = make_protocol()
            .serialize_request(
                &QueryParamsStruct,
                &QUERY_PARAMS_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.uri(), "https://example.com?page=2&limit=50");
    }

    // -- Timestamp in header defaults to http-date --

    static TS_HEADER_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Timestamp,
        "ifModified",
        0,
    )
    .with_http_header("If-Modified-Since");

    static TS_HEADER_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&TS_HEADER_MEMBER],
    );

    struct TimestampHeaderStruct;
    impl SerializableStruct for TimestampHeaderStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_timestamp(&TS_HEADER_MEMBER, &aws_smithy_types::DateTime::from_secs(0))
        }
    }

    #[test]
    fn timestamp_header_uses_http_date() {
        let request = make_protocol()
            .serialize_request(
                &TimestampHeaderStruct,
                &TS_HEADER_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        let value = request.headers().get("If-Modified-Since").unwrap();
        // http-date format: "Thu, 01 Jan 1970 00:00:00 GMT"
        assert!(value.contains("1970"), "expected http-date, got: {value}");
    }

    // -- Timestamp in query defaults to date-time --

    static TS_QUERY_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Timestamp,
        "since",
        0,
    )
    .with_http_query("since");

    static TS_QUERY_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&TS_QUERY_MEMBER],
    );

    struct TimestampQueryStruct;
    impl SerializableStruct for TimestampQueryStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_timestamp(&TS_QUERY_MEMBER, &aws_smithy_types::DateTime::from_secs(0))
        }
    }

    #[test]
    fn timestamp_query_uses_date_time() {
        let request = make_protocol()
            .serialize_request(
                &TimestampQueryStruct,
                &TS_QUERY_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request.uri(),
            "https://example.com?since=1970-01-01T00%3A00%3A00Z"
        );
    }

    // -- Unbound members go to body, bound members do not --

    static BOUND_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "headerVal",
        0,
    )
    .with_http_header("X-Val");
    static UNBOUND_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "bodyVal",
        1,
    );
    static MIXED_SCHEMA: Schema = Schema::new_struct(
        crate::shape_id!("test", "S"),
        ShapeType::Structure,
        &[&BOUND_MEMBER, &UNBOUND_MEMBER],
    );

    struct MixedStruct;
    impl SerializableStruct for MixedStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&BOUND_MEMBER, "in-header")?;
            s.write_string(&UNBOUND_MEMBER, "in-body")
        }
    }

    #[test]
    fn bound_members_not_in_body() {
        let request = make_protocol()
            .serialize_request(
                &MixedStruct,
                &MIXED_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert!(
            body.contains("in-body"),
            "body should contain unbound member"
        );
        assert!(
            !body.contains("in-header"),
            "body should NOT contain header-bound member"
        );
        assert_eq!(request.headers().get("X-Val").unwrap(), "in-header");
    }
}
