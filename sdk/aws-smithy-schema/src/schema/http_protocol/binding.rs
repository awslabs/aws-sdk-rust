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
use std::cell::Cell;

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
    protocol_id: ShapeId<'static>,
    codec: C,
    content_type: &'static str,
}

impl<C: Codec> HttpBindingProtocol<C> {
    /// Creates a new HTTP binding protocol.
    pub fn new(protocol_id: ShapeId<'static>, codec: C, content_type: &'static str) -> Self {
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

    /// Returns the Content-Type string this protocol stamps onto the
    /// outgoing request. Used by wrapper protocols that rebuild the
    /// inner [`HttpBindingProtocol`] when reconfiguring the codec.
    pub fn content_type(&self) -> &'static str {
        self.content_type
    }

    /// Replaces the body codec, returning a new protocol instance
    /// with all other fields preserved. Used by wrapper protocols
    /// (e.g. AWS REST JSON) that need to swap in a reconfigured codec.
    pub fn with_codec(self, codec: C) -> Self {
        Self {
            protocol_id: self.protocol_id,
            codec,
            content_type: self.content_type,
        }
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
        input_schema: &Schema<'_>,
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
        //
        // Codegen records this on the schema so the common path is a single
        // discriminant read. `PayloadHint::Unknown`, every hand-constructed or
        // runtime-materialized schema, and every crate generated before codegen
        // began recording the hint. It falls back to deriving it by scanning
        // members, which is what this did unconditionally before. The two
        // branches must stay in agreement: `PayloadHint` is an optimization, not
        // a correctness input.
        let has_struct_payload = match input_schema.payload_hint() {
            crate::PayloadHint::StructPayload => true,
            crate::PayloadHint::NoStructPayload => false,
            _ => input_schema.members().iter().any(|m| {
                m.http_payload().is_some()
                    && matches!(
                        m.shape_type(),
                        crate::ShapeType::Structure | crate::ShapeType::Union
                    )
            }),
        };
        // If the schema declares zero body members (every member is HTTP-bound,
        // and any `@httpPayload` is on a scalar that bypasses the codec),
        // we can skip body-codec invocation entirely. The wasted work would be:
        //   - XmlSerializer/JsonSerializer::write_struct opens a wrapper element
        //   - Proxy::serialize_members routes members through a BindingRouter
        //   - close-element is emitted
        //   - bytes are collected by `body.finish()` and then discarded
        //     (since `has_body_members == false` later forces `body = Vec::new()`)
        // Skipping all of that just calls `serialize_members` directly through
        // a `BindingRouter` so HTTP-bound members are still routed to headers /
        // query / labels. No `Proxy` is involved, so nothing can emit framing;
        // a nested struct that happened to pass through (none do in this branch
        // by definition) would delegate straight to the body serializer.
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
            let mut body = body;
            let mut state = BindingState::new(Some(input_schema), request.headers_mut());

            if skip_body_codec || has_struct_payload {
                // skip_body_codec: input has no body members at all → all members
                //                  route to HTTP bindings, body bytes are unused.
                // has_struct_payload: an @httpPayload struct member writes itself
                //                     to the body without wrapping — call
                //                     serialize_members directly so framing comes
                //                     from the payload struct, not from the codec.
                //
                // No codec framing is wanted, so route members directly and skip
                // the `Proxy` entirely.
                let mut router = BindingRouter {
                    state: &mut state,
                    body: &mut body,
                };
                input.serialize_members(&mut router)?;
            } else {
                // Framing comes from the codec: hand it a `Proxy` in place of the
                // real input so that `{`/`}` (JSON) or the enclosing element (XML)
                // is emitted by the codec, while each member still routes through
                // `BindingRouter`.
                //
                // `state` is lent to the proxy for the duration of this call only.
                // The disjoint borrows of `state` and `body` are what make this
                // safe without a raw pointer; see `BindingState`'s documentation
                // for the aliasing UB this replaced.
                let proxy = Proxy {
                    state: Cell::new(Some(&mut state)),
                    value: input,
                };
                body.write_struct(input_schema, &proxy)?;
            }
            let raw_payload = state.raw_payload;
            let body_bytes = if raw_payload.is_some() || skip_body_codec {
                // @httpPayload blob/string — don't use the codec output.
                // skip_body_codec — body codec was never written to.
                Vec::new()
            } else {
                body.finish()
            };
            (raw_payload, body_bytes, state.query_params, state.labels)
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
        // Capacity heuristic: template + slack for label expansion (greedy
        // labels typically expand by O(1.5x)), or the endpoint when it is
        // serving as the template. Better-than-default initial capacity avoids
        // the first 1-2 reallocs.
        let mut uri =
            String::with_capacity(endpoint.len() + template_opt.map(|t| t.len()).unwrap_or(1) + 64);
        match template_opt {
            Some(template) => {
                // The `@http` template is authoritative and `endpoint` is ignored. A REST protocol
                // owns its route, so a path computed for a different protocol must not be prefixed
                // onto it — an rpcv2Cbor-generated client that selects restJson1 at runtime passes
                // `/service/{service}/operation/{operation}` here, and awsJson passes `/`.
                // Generated REST clients pass `""`, so this costs them nothing.
                //
                // This mirrors the assertion `AwsJsonRpcProtocol` and `AwsQueryProtocol` make about
                // their own fixed routes. See `ClientProtocolInner::serialize_request` for the
                // general rule that `endpoint` is advisory.
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
            // `into_bytes` unwraps the `Blob`'s `Bytes` and `SdkBody::from(Bytes)`
            // clones the handle, so no payload bytes are copied here.
            SdkBody::from(payload.into_bytes())
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
pub(crate) fn percent_encode_into(input: &str, out: &mut String) {
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
fn append_uri_with_labels<'sc>(
    template: &str,
    labels: &[(Cow<'sc, str>, String)],
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

/// The HTTP-binding half of request serialization: everything a member can be
/// routed *to* other than the body.
///
/// Deliberately holds no body serializer. Keeping the two apart is what makes
/// the routing sound: [`BindingRouter`] borrows this and the body codec as two
/// disjoint fields, so the compiler proves they cannot alias. An earlier version
/// combined them into one `HttpBindingSerializer` that owned the codec, which
/// forced the re-entrant `serialize_members` call to resurrect the combined
/// object through a `*mut` derived from a shared reference — undefined behavior
/// under both Stacked Borrows and Tree Borrows, and reproducible under Miri from
/// this crate's own tests. Do not reunite these two halves.
///
/// Members without HTTP binding traits never reach this type; [`BindingRouter`]
/// forwards them to the body serializer.
struct BindingState<'a> {
    /// Headers are inserted directly into the `Request`'s header map as they
    /// are encountered, avoiding the cost of a `Vec<(...)>` intermediate plus
    /// a late flush loop. The borrow ends when this state is dropped at the
    /// end of `serialize_request_with_body`'s binder scope.
    headers: &'a mut Headers,
    query_params: Vec<(Cow<'a, str>, String)>,
    labels: Vec<(Cow<'a, str>, String)>,
    /// When set, member schemas are resolved from this schema by name to find
    /// HTTP binding traits. This allows the protocol to override bindings
    /// (e.g., for presigning where body members become query params).
    input_schema: Option<&'a Schema<'a>>,
    /// Raw payload bytes for `@httpPayload` blob/string members. When a member
    /// has `@httpPayload` and targets a blob or string, the raw bytes bypass
    /// the codec serializer entirely and are used as the HTTP body directly.
    ///
    /// Owned rather than borrowed. `ShapeSerializer::write_blob` takes an owned
    /// [`Blob`](aws_smithy_types::Blob), which is `bytes::Bytes`-backed, so storing
    /// a blob payload here is a refcount bump and handing it to `SdkBody` is
    /// another — no payload copy anywhere on the blob path. A string payload does
    /// cost one copy, because `write_string` takes `&str` and `str` has no shared
    /// representation.
    ///
    /// This field used to be `Option<&'a [u8]>`, populated by transmuting the
    /// `write_*` argument's anonymous lifetime to `'a`. That was unsound: `'a`
    /// is tied to the input schema and headers, not to the serialized value, so
    /// a caller whose `serialize_members` wrote a locally-computed payload
    /// produced a use-after-free. Do not reintroduce a borrow here.
    raw_payload: Option<aws_smithy_types::Blob>,
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

impl<'a> BindingState<'a> {
    fn new(input_schema: Option<&'a Schema<'a>>, headers: &'a mut Headers) -> Self {
        Self {
            headers,
            query_params: Vec::new(),
            labels: Vec::new(),
            input_schema,
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
    fn should_route_binding(&mut self, schema: &Schema<'_>) -> bool {
        schema
            .member_index()
            .map(|idx| self.visited_bound_members.insert(idx))
            .unwrap_or(true)
    }

    /// Resolve the effective member schema: if an input_schema override is set,
    /// look up the member by name there (to get the correct HTTP bindings).
    /// Otherwise use the schema as-is.
    fn resolve_member<'s>(&self, schema: &'s Schema<'s>) -> &'s Schema<'s>
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

    /// Like [`Self::resolve_member`] but only succeeds when the member can be
    /// resolved through `input_schema`. Returns the member with the
    /// binder's `'a` data lifetime so callers can push into `'a`-bound
    /// collections (e.g. `labels: Vec<(Cow<'a, str>, String)>`) without
    /// allocating, even when the trait-method schema parameter's
    /// anonymous lifetime is unrelated to `'a`.
    fn resolve_to_input_schema(&self, schema: &Schema<'_>) -> Option<&'a Schema<'a>> {
        let input_schema = self.input_schema?;
        if let Some(idx) = schema.member_index() {
            if let Some(s) = input_schema.member_schema_by_index(idx) {
                return Some(s);
            }
        }
        if let Some(name) = schema.member_name() {
            return input_schema.member_schema(name);
        }
        None
    }

    /// The `@httpQuery` parameter name for a member, as a `Cow<'a, str>` so it
    /// can be pushed into `query_params` without allocating.
    ///
    /// `@httpQuery` values carry the schema's data lifetime, and the schema
    /// arriving through a `ShapeSerializer` method has an anonymous lifetime
    /// unrelated to the binder's `'a`. Resolving the member through
    /// `input_schema` recovers a value that lives for `'a`; when that fails
    /// (no `input_schema`, or the member is not found there) the name is
    /// copied. Mirrors the `@httpLabel` handling in [`Self::add_binding`].
    fn query_param_name(
        &self,
        schema: &Schema<'_>,
        query: &crate::traits::HttpQueryTrait<'_>,
    ) -> Cow<'a, str> {
        match self
            .resolve_to_input_schema(schema)
            .and_then(|resolved| resolved.http_query())
        {
            Some(resolved_query) => Cow::Borrowed(resolved_query.value()),
            None => Cow::Owned(query.value().to_string()),
        }
    }
}

/// Resolves an `@httpHeader` name into the `Cow<'static, str>` that
/// `Headers::insert` requires.
///
/// `value_static()` is `Some` for every schema that can be built today — the
/// only `@httpHeader` constructor takes `&'static str` — so this is a
/// zero-allocation borrow in practice. The owned arm exists so that relaxing
/// `@httpHeader` to accept arena-borrowed names stays an additive change
/// instead of breaking this call site.
fn header_name(header: &crate::traits::HttpHeaderTrait<'_>) -> Cow<'static, str> {
    match header.value_static() {
        Some(name) => Cow::Borrowed(name),
        None => Cow::Owned(header.value().to_string()),
    }
}

/// Routes each member of the top-level input struct either into
/// [`BindingState`] (HTTP-bound members) or into the body codec (everything
/// else).
///
/// The two references are separate fields precisely so that the compiler can
/// see they do not alias. `body` is the `&mut dyn ShapeSerializer` that the
/// codec hands to `SerializableStruct::serialize_members`; an earlier version
/// discarded that argument and reconstructed the body serializer from a raw
/// pointer instead, which was the source of the aliasing UB. Use the argument.
///
/// The three lifetimes are load-bearing and must stay distinct. Collapsing
/// `'s` and `'b` into one shortens the state borrow to the body serializer's
/// and produces an unfixable variance error at the [`Cell`] in `Proxy`, because
/// `Cell<T>` is invariant over `T`.
struct BindingRouter<'s, 'b, 'a> {
    state: &'s mut BindingState<'a>,
    body: &'b mut dyn ShapeSerializer,
}

/// Bridges the codec's `write_struct` framing back into HTTP-binding routing.
///
/// The body codec is asked to serialize *this* rather than the real input, so
/// that framing (`{`/`}` for JSON, the element for XML) comes from the codec
/// while member routing still passes through [`BindingRouter`].
///
/// `serialize_members` receives `&self`, so the state is lent out through a
/// [`Cell`] take/put rather than held as `&mut`. A `Cell` is used over a
/// `RefCell` to stay allocation-free and to avoid a second panicking path; the
/// take/put is required rather than merely convenient because some codecs
/// (notably XML) call `serialize_members` more than once and each call needs
/// the state back.
struct Proxy<'p, 'a> {
    state: Cell<Option<&'p mut BindingState<'a>>>,
    value: &'p dyn SerializableStruct,
}

impl<'p, 'a> SerializableStruct for Proxy<'p, 'a> {
    fn serialize_members(&self, serializer: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        // Returning an error rather than panicking: this is unreachable with
        // every codec in this repo, because a codec that nests
        // `serialize_members` on the *same* struct value would have to call it
        // re-entrantly rather than sequentially. Sequential calls (XML's two
        // passes) put the state back before the next one begins. A future codec
        // that does nest should get a serialization error, not a panic in a
        // customer's request path.
        let state = self.state.take().ok_or_else(|| {
            SerdeError::custom(
                "HTTP binding state is already borrowed: the body codec re-entered \
                 serialize_members on the same struct before the previous call returned",
            )
        })?;
        let mut router = BindingRouter {
            state,
            body: serializer,
        };
        let result = self.value.serialize_members(&mut router);
        // Put the state back even on the error path, so a codec that ignores
        // one member's error and continues does not then hit the branch above.
        self.state.set(Some(router.state));
        result
    }
}

impl<'s, 'b, 'a> ShapeSerializer for BindingRouter<'s, 'b, 'a> {
    fn write_struct(
        &mut self,
        schema: &Schema<'_>,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        // A nested struct (a body member targeting a structure): delegate
        // entirely to the body serializer. Only the top-level input struct is
        // routed, and that entry point is `serialize_request_with_body`'s
        // explicit `Proxy` construction rather than a flag checked here.
        let schema = self.state.resolve_member(schema);
        // `@httpPayload` struct/union: codegen routes these by passing the
        // target struct's schema directly (not the member schema), so the
        // payload branch is normally unreachable. Both arms are the same call;
        // the `if` is kept as documentation of that intent.
        self.body.write_struct(schema, value)
    }

    fn write_list(
        &mut self,
        schema: &Schema<'_>,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        // @httpHeader on a list: collect elements as comma-separated header value
        if let Some(header) = schema.http_header() {
            if !self.state.should_route_binding(schema) {
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
            self.state.headers.insert(header_name(header), header_val);
            return Ok(());
        }
        // @httpQuery on a list: add each element as a separate query param
        if let Some(query) = schema.http_query() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            let mut collector = ListElementCollector::for_query();
            write_elements(&mut collector)?;
            // Prefer the `'a`-lifetime member from `input_schema` so the pushed
            // `Cow<'a, str>` can be `Borrowed` (zero-alloc). The trait method's
            // `&Schema<'_>` has an anonymous lifetime unrelated to `'a`, and
            // `@httpQuery` values carry the schema's data lifetime, so we fall
            // back to allocating when the member cannot be resolved.
            let name = self.state.query_param_name(schema, query);
            for val in collector.values {
                self.state.query_params.push((name.clone(), val));
            }
            return Ok(());
        }
        self.body.write_list(schema, write_elements)
    }

    fn write_map(
        &mut self,
        schema: &Schema<'_>,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        // @httpPrefixHeaders: serialize map entries as prefixed headers
        if let Some(prefix) = schema.http_prefix_headers() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            // Collect entries via a temporary serializer
            let mut collector = MapEntryCollector::new(prefix.value().to_string());
            write_entries(&mut collector)?;
            // Names are dynamic (prefix + map key) — owned Strings.
            for (k, v) in collector.entries {
                self.state.headers.insert(k, v);
            }
            return Ok(());
        }
        // @httpQueryParams: serialize map entries as query params
        if schema.http_query_params().is_some() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            let mut collector = MapEntryCollector::new(String::new());
            write_entries(&mut collector)?;
            // Filter out keys that overlap with explicit @httpQuery params
            // (query params take precedence over query params map entries)
            let explicit_query_keys: Vec<&str> = self
                .state
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
                    self.state.query_params.push((Cow::Owned(k), v));
                }
            }
            return Ok(());
        }
        self.body.write_map(schema, write_entries)
    }

    fn write_boolean(&mut self, schema: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_boolean(schema, value)
    }

    fn write_byte(&mut self, schema: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_byte(schema, value)
    }

    fn write_short(&mut self, schema: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_short(schema, value)
    }

    fn write_integer(&mut self, schema: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_integer(schema, value)
    }

    fn write_long(&mut self, schema: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, &value.to_string());
        }
        self.body.write_long(schema, value)
    }

    fn write_float(&mut self, schema: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self
                .state
                .add_binding(binding, schema, &format_float_f32(value));
        }
        self.body.write_float(schema, value)
    }

    fn write_double(&mut self, schema: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self
                .state
                .add_binding(binding, schema, &format_float_f64(value));
        }
        self.body.write_double(schema, value)
    }

    fn write_big_integer(
        &mut self,
        schema: &Schema<'_>,
        value: &aws_smithy_types::BigInteger,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, value.as_ref());
        }
        self.body.write_big_integer(schema, value)
    }

    fn write_big_decimal(
        &mut self,
        schema: &Schema<'_>,
        value: &aws_smithy_types::BigDecimal,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            return self.state.add_binding(binding, schema, value.as_ref());
        }
        self.body.write_big_decimal(schema, value)
    }

    fn write_string(&mut self, schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if let Some(binding) = http_string_binding(schema) {
            // @mediaType on a header: base64-encode the value
            if schema.media_type().is_some() {
                let encoded = aws_smithy_types::base64::encode(value.as_bytes());
                return self.state.add_binding(binding, schema, &encoded);
            }
            return self.state.add_binding(binding, schema, value);
        }
        if schema.http_payload().is_some() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            // One copy, and it is unavoidable: `write_string` hands over a `&str`
            // with an anonymous lifetime that has no relationship to `'a`, and
            // `str` has no shared representation to take a cheap handle on. The
            // previous implementation transmuted the lifetime to `'a` instead,
            // which was a use-after-free for any caller that computed its
            // payload into a local. `should_route_binding` above means the XML
            // codec's second `serialize_members` pass returns before we get
            // here, so this copy happens at most once per payload.
            self.state.raw_payload = Some(aws_smithy_types::Blob::new(value));
            return Ok(());
        }
        self.body.write_string(schema, value)
    }

    fn write_blob(
        &mut self,
        schema: &Schema<'_>,
        value: aws_smithy_types::Blob,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
        if schema.http_header().is_some() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            let encoded = aws_smithy_types::base64::encode(value.as_ref());
            self.state
                .headers
                .insert(header_name(schema.http_header().unwrap()), encoded);
            return Ok(());
        }
        if schema.http_payload().is_some() {
            if !self.state.should_route_binding(schema) {
                return Ok(());
            }
            // Zero copies: `Blob` is `bytes::Bytes`-backed, so this moves a
            // refcounted handle, and `SdkBody::from(Bytes)` at the consume site
            // takes another handle rather than copying. This is the reason
            // `write_blob` takes an owned `Blob` — see its trait documentation.
            self.state.raw_payload = Some(value);
            return Ok(());
        }
        self.body.write_blob(schema, value)
    }

    fn write_timestamp(
        &mut self,
        schema: &Schema<'_>,
        value: &aws_smithy_types::DateTime,
    ) -> Result<(), SerdeError> {
        let schema = self.state.resolve_member(schema);
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
                    HttpBinding::Header => aws_smithy_types::date_time::Format::HttpDate,
                    _ => aws_smithy_types::date_time::Format::DateTime,
                }
            };
            let formatted = value
                .fmt(format)
                .map_err(|e| SerdeError::custom(format!("failed to format timestamp: {e}")))?;
            return self.state.add_binding(binding, schema, &formatted);
        }
        self.body.write_timestamp(schema, value)
    }

    fn write_document(
        &mut self,
        schema: &Schema<'_>,
        value: &aws_smithy_types::Document,
    ) -> Result<(), SerdeError> {
        self.body.write_document(schema, value)
    }

    fn write_null(&mut self, schema: &Schema<'_>) -> Result<(), SerdeError> {
        self.body.write_null(schema)
    }
}

/// Which HTTP location a member is bound to.
///
/// Carries no name: every caller passes the same member schema to
/// [`BindingState::add_binding`], which resolves the name there. That
/// keeps `@httpQuery`'s schema-lifetime value out of a `'static` slot.
enum HttpBinding {
    Header,
    Query,
    Label,
}

/// Determine the HTTP binding for a member schema, if any.
fn http_string_binding(schema: &Schema<'_>) -> Option<HttpBinding> {
    if schema.http_header().is_some() {
        return Some(HttpBinding::Header);
    }
    if schema.http_query().is_some() {
        return Some(HttpBinding::Query);
    }
    if schema.http_label().is_some() {
        return Some(HttpBinding::Label);
    }
    None
}

impl<'a> BindingState<'a> {
    fn add_binding(
        &mut self,
        binding: HttpBinding,
        schema: &Schema<'_>,
        value: &str,
    ) -> Result<(), SerdeError> {
        // Dedupe per-member: see `should_route_binding`. Without this, a
        // multi-pass body codec invokes `serialize_members` more than once
        // and each pass would append to `headers` / `query_params` / `labels`.
        if !self.should_route_binding(schema) {
            return Ok(());
        }
        match binding {
            HttpBinding::Header => {
                // `Headers::insert` needs a `'static` name; `header_name`
                // recovers one from the trait, which is a zero-allocation
                // borrow for every schema constructible today.
                if let Some(header) = schema.http_header() {
                    self.headers.insert(header_name(header), value.to_string());
                }
            }
            HttpBinding::Query => {
                if let Some(query) = schema.http_query() {
                    let name = self.query_param_name(schema, query);
                    self.query_params.push((name, value.to_string()));
                }
            }
            HttpBinding::Label => {
                // Prefer the `'a`-lifetime member from `input_schema` so the
                // pushed `Cow<'a, str>` can be `Borrowed` (zero-alloc). The
                // trait method's `&Schema<'_>` schema has an anonymous
                // lifetime not bounded by `'a`, so we'd otherwise have to
                // allocate. Falls back to allocation when no `input_schema`
                // is available.
                let cow_name = if let Some(resolved) = self.resolve_to_input_schema(schema) {
                    let name = resolved
                        .member_name()
                        .ok_or_else(|| SerdeError::custom("httpLabel on non-member schema"))?;
                    Cow::Borrowed(name)
                } else {
                    let name = schema
                        .member_name()
                        .ok_or_else(|| SerdeError::custom("httpLabel on non-member schema"))?;
                    Cow::Owned(name.to_string())
                };
                self.labels.push((cow_name, value.to_string()));
            }
        }
        Ok(())
    }
}

/// Generates inert [`ShapeSerializer`] write methods (each returning
/// `Ok(())`) for the named methods. The HTTP-binding collectors below
/// implement only the writes that map a scalar to its string form;
/// every other write is a no-op. Listing those no-ops through this
/// macro keeps each collector's impl focused on the writes it actually
/// handles. Each entry is `method_name(value_arg_types...)`; methods
/// with no value beyond the schema (e.g. `write_null`) list no types.
macro_rules! noop_writes {
    ($($method:ident($($arg:ty),*)),+ $(,)?) => {
        $(
            fn $method(&mut self, _: &Schema<'_>, $(_: $arg),*) -> Result<(), SerdeError> {
                Ok(())
            }
        )+
    };
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
    fn write_string(&mut self, _schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_boolean(&mut self, _: &Schema<'_>, value: bool) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_byte(&mut self, _: &Schema<'_>, value: i8) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_short(&mut self, _: &Schema<'_>, value: i16) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_integer(&mut self, _: &Schema<'_>, value: i32) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_long(&mut self, _: &Schema<'_>, value: i64) -> Result<(), SerdeError> {
        self.push(value.to_string());
        Ok(())
    }
    fn write_float(&mut self, _: &Schema<'_>, value: f32) -> Result<(), SerdeError> {
        self.push(format_float_f32(value));
        Ok(())
    }
    fn write_double(&mut self, _: &Schema<'_>, value: f64) -> Result<(), SerdeError> {
        self.push(format_float_f64(value));
        Ok(())
    }
    fn write_timestamp(
        &mut self,
        schema: &Schema<'_>,
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
    fn write_blob(
        &mut self,
        _schema: &Schema<'_>,
        value: aws_smithy_types::Blob,
    ) -> Result<(), SerdeError> {
        self.push(aws_smithy_types::base64::encode(value.as_ref()));
        Ok(())
    }
    // Remaining writes are no-ops for list element collection.
    noop_writes! {
        write_struct(&dyn SerializableStruct),
        write_list(&dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>),
        write_map(&dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>),
        write_big_integer(&aws_smithy_types::BigInteger),
        write_big_decimal(&aws_smithy_types::BigDecimal),
        write_document(&aws_smithy_types::Document),
        write_null(),
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
    fn write_string(&mut self, _schema: &Schema<'_>, value: &str) -> Result<(), SerdeError> {
        if let Some(key) = self.pending_key.take() {
            self.entries
                .push((format!("{}{}", self.prefix, key), value.to_string()));
        } else {
            self.pending_key = Some(value.to_string());
        }
        Ok(())
    }

    fn write_list(
        &mut self,
        _: &Schema<'_>,
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
    // Every other write is a no-op: HTTP-binding maps have string keys
    // and values, and the `write_list` above handles the
    // Map<String, List<String>> case for @httpQueryParams.
    noop_writes! {
        write_struct(&dyn SerializableStruct),
        write_map(&dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>),
        write_boolean(bool),
        write_byte(i8),
        write_short(i16),
        write_integer(i32),
        write_long(i64),
        write_float(f32),
        write_double(f64),
        write_big_integer(&aws_smithy_types::BigInteger),
        write_big_decimal(&aws_smithy_types::BigDecimal),
        write_blob(aws_smithy_types::Blob),
        write_timestamp(&aws_smithy_types::DateTime),
        write_document(&aws_smithy_types::Document),
        write_null(),
    }
}

impl<C> ClientProtocolInner for HttpBindingProtocol<C>
where
    C: Codec + Send + Sync + std::fmt::Debug + 'static,
    for<'a> C::Deserializer<'a>: ShapeDeserializer,
{
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId<'static> {
        &self.protocol_id
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        let body = self.codec.create_serializer();
        self.serialize_request_with_body(body, input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        _output_schema: &Schema<'_>,
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
            _: &Schema<'_>,
            value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            self.output.push(b'{');
            value.serialize_members(self)?;
            self.output.push(b'}');
            Ok(())
        }
        fn write_list(
            &mut self,
            _: &Schema<'_>,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_map(
            &mut self,
            _: &Schema<'_>,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_boolean(&mut self, _: &Schema<'_>, _: bool) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_byte(&mut self, _: &Schema<'_>, _: i8) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_short(&mut self, _: &Schema<'_>, _: i16) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_integer(&mut self, _: &Schema<'_>, _: i32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_long(&mut self, _: &Schema<'_>, _: i64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_float(&mut self, _: &Schema<'_>, _: f32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_double(&mut self, _: &Schema<'_>, _: f64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_integer(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_decimal(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_string(&mut self, _: &Schema<'_>, v: &str) -> Result<(), SerdeError> {
            self.output.extend_from_slice(v.as_bytes());
            Ok(())
        }
        fn write_blob(
            &mut self,
            _: &Schema<'_>,
            _: aws_smithy_types::Blob,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_timestamp(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_document(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_null(&mut self, _: &Schema<'_>) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    struct TestDeserializer<'a> {
        input: &'a [u8],
    }

    impl ShapeDeserializer for TestDeserializer<'_> {
        fn read_struct(
            &mut self,
            _: &Schema<'_>,
            _: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_list(
            &mut self,
            _: &Schema<'_>,
            _: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _: &Schema<'_>,
            _: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }
        fn read_big_decimal(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }
        fn read_string(&mut self, _: &Schema<'_>) -> Result<String, SerdeError> {
            Ok(String::from_utf8_lossy(self.input).into_owned())
        }
        fn read_blob(&mut self, _: &Schema<'_>) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(vec![]))
        }
        fn read_timestamp(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }
        fn read_document(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::Document, SerdeError> {
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

    static TEST_SCHEMA: Schema<'static> =
        Schema::new(crate::shape_id!("test", "TestStruct"), ShapeType::Structure);

    struct EmptyStruct;
    impl SerializableStruct for EmptyStruct {
        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static NAME_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::String,
        "name",
        0,
    );
    static MEMBERS: &[&Schema<'_>] = &[&NAME_MEMBER];
    static STRUCT_WITH_MEMBER: Schema<'static> = Schema::new_struct(
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

    /// A REST protocol resolves its route from the operation's `@http` template, so a path that
    /// codegen computed for a *different* protocol must not be prefixed onto it.
    ///
    /// This is the mirror image of the assertion `AwsJsonRpcProtocol` and `AwsQueryProtocol` make
    /// about their own fixed routes: whichever layer owns the route ignores a foreign one. It is
    /// reachable only through `Config::builder().protocol(..)` — generated REST clients pass `""`,
    /// so the request would otherwise be routed to the concatenation of both protocols' paths, e.g.
    /// `/service/Svc/operation/GetStats/stats` on an rpcv2Cbor-generated client.
    #[test]
    fn serialize_request_ignores_a_route_computed_for_another_protocol() {
        static HTTP_SCHEMA: Schema<'static> = Schema::new_struct(
            crate::shape_id!("test", "GetStatsRequest"),
            ShapeType::Structure,
            &[],
        )
        .with_http(crate::traits::HttpTrait::new("PUT", "/stats", None));

        for foreign_route in [
            // An rpcv2Cbor route, as codegen emits for that protocol.
            "/service/Svc/operation/GetStats",
            // awsJson's and awsQuery's fixed route.
            "/",
        ] {
            let request = make_protocol()
                .serialize_request(
                    &EmptyStruct,
                    &HTTP_SCHEMA,
                    foreign_route,
                    &ConfigBag::base(),
                )
                .unwrap();
            assert_eq!(
                "/stats",
                request.uri(),
                "the `@http` template is authoritative; the route {foreign_route} was computed for \
                 another protocol and must be ignored",
            );
            assert_eq!("PUT", request.method());
        }
    }

    /// The counterpart guard: with no `@http` trait there is no template, so the endpoint *is* the
    /// template and is still honored — including label expansion. Several tests below rely on this.
    #[test]
    fn serialize_request_uses_endpoint_as_template_without_an_http_trait() {
        let request = make_protocol()
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "/some/path", &ConfigBag::base())
            .unwrap();
        assert_eq!("/some/path", request.uri());
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
                _: &Schema<'_>,
                _: &dyn SerializableStruct,
            ) -> Result<(), SerdeError> {
                WRITE_CALLS.fetch_add(1, Ordering::SeqCst);
                panic!("body codec write_struct() called — short-circuit failed");
            }
            fn write_list(
                &mut self,
                _: &Schema<'_>,
                _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_list() called");
            }
            fn write_map(
                &mut self,
                _: &Schema<'_>,
                _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_map() called");
            }
            fn write_boolean(&mut self, _: &Schema<'_>, _: bool) -> Result<(), SerdeError> {
                panic!("body codec write_boolean() called");
            }
            fn write_byte(&mut self, _: &Schema<'_>, _: i8) -> Result<(), SerdeError> {
                panic!("body codec write_byte() called");
            }
            fn write_short(&mut self, _: &Schema<'_>, _: i16) -> Result<(), SerdeError> {
                panic!("body codec write_short() called");
            }
            fn write_integer(&mut self, _: &Schema<'_>, _: i32) -> Result<(), SerdeError> {
                panic!("body codec write_integer() called");
            }
            fn write_long(&mut self, _: &Schema<'_>, _: i64) -> Result<(), SerdeError> {
                panic!("body codec write_long() called");
            }
            fn write_float(&mut self, _: &Schema<'_>, _: f32) -> Result<(), SerdeError> {
                panic!("body codec write_float() called");
            }
            fn write_double(&mut self, _: &Schema<'_>, _: f64) -> Result<(), SerdeError> {
                panic!("body codec write_double() called");
            }
            fn write_big_integer(
                &mut self,
                _: &Schema<'_>,
                _: &aws_smithy_types::BigInteger,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_big_integer() called");
            }
            fn write_big_decimal(
                &mut self,
                _: &Schema<'_>,
                _: &aws_smithy_types::BigDecimal,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_big_decimal() called");
            }
            fn write_string(&mut self, _: &Schema<'_>, _: &str) -> Result<(), SerdeError> {
                panic!("body codec write_string() called");
            }
            fn write_blob(
                &mut self,
                _: &Schema<'_>,
                _: aws_smithy_types::Blob,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_blob() called");
            }
            fn write_timestamp(
                &mut self,
                _: &Schema<'_>,
                _: &aws_smithy_types::DateTime,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_timestamp() called");
            }
            fn write_document(
                &mut self,
                _: &Schema<'_>,
                _: &aws_smithy_types::Document,
            ) -> Result<(), SerdeError> {
                panic!("body codec write_document() called");
            }
            fn write_null(&mut self, _: &Schema<'_>) -> Result<(), SerdeError> {
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
        static HEADER_MEMBER: Schema<'static> = Schema::new_member(
            crate::shape_id!("test", "HeaderOnlyStruct"),
            ShapeType::String,
            "x_header",
            0,
        )
        .with_http_header("X-Header");
        static HEADER_MEMBERS: &[&Schema<'_>] = &[&HEADER_MEMBER];
        static HEADER_ONLY_SCHEMA: Schema<'static> = Schema::new_struct(
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

    // -- PayloadHint must agree with the scan it replaces -----------------------------------------
    //
    // `PayloadHint` is an optimization, so the only property worth pinning is
    // that a recorded hint and the derived scan produce byte-identical requests.
    // The negative control below proves these are not vacuous by showing that a
    // *wrong* hint does change the body — i.e. the hint really is consulted.

    static STRUCT_PAYLOAD_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "StructPayloadStruct"),
        ShapeType::Structure,
        "body",
        0,
    )
    .with_http_payload();
    static STRUCT_PAYLOAD_MEMBERS: &[&Schema<'_>] = &[&STRUCT_PAYLOAD_MEMBER];

    /// No hint recorded — the runtime derives `has_struct_payload` by scanning.
    static STRUCT_PAYLOAD_UNKNOWN: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "StructPayloadStruct"),
        ShapeType::Structure,
        STRUCT_PAYLOAD_MEMBERS,
    );
    /// Same schema, correct hint recorded.
    static STRUCT_PAYLOAD_HINTED: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "StructPayloadStruct"),
        ShapeType::Structure,
        STRUCT_PAYLOAD_MEMBERS,
    )
    .with_payload_hint(crate::PayloadHint::StructPayload);
    /// Same schema, deliberately *wrong* hint — negative control only.
    static STRUCT_PAYLOAD_MIS_HINTED: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "StructPayloadStruct"),
        ShapeType::Structure,
        STRUCT_PAYLOAD_MEMBERS,
    )
    .with_payload_hint(crate::PayloadHint::NoStructPayload);

    /// Writes an `@httpPayload` struct member. `TestSerializer::write_struct`
    /// emits `{` .. `}`, so the framing is directly observable: one pair of
    /// braces means only the payload member framed the body, two means the
    /// codec wrapped it as well.
    struct StructPayloadStruct;
    impl SerializableStruct for StructPayloadStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_struct(&STRUCT_PAYLOAD_MEMBER, &NameStruct)
        }
    }

    fn body_for(schema: &Schema<'_>, input: &dyn SerializableStruct) -> Vec<u8> {
        make_protocol()
            .serialize_request(input, schema, "https://example.com", &ConfigBag::base())
            .unwrap()
            .body()
            .bytes()
            .unwrap_or(&[])
            .to_vec()
    }

    #[test]
    fn struct_payload_hint_agrees_with_derived_scan() {
        let derived = body_for(&STRUCT_PAYLOAD_UNKNOWN, &StructPayloadStruct);
        let hinted = body_for(&STRUCT_PAYLOAD_HINTED, &StructPayloadStruct);

        // Only the payload member frames the body.
        assert_eq!(derived, b"{Alice}");
        assert_eq!(hinted, derived);
    }

    #[test]
    fn no_struct_payload_hint_agrees_with_derived_scan() {
        static HINTED: Schema<'static> = Schema::new_struct(
            crate::shape_id!("test", "TestStruct"),
            ShapeType::Structure,
            MEMBERS,
        )
        .with_payload_hint(crate::PayloadHint::NoStructPayload);

        let derived = body_for(&STRUCT_WITH_MEMBER, &NameStruct);
        let hinted = body_for(&HINTED, &NameStruct);

        // Codec supplies the framing.
        assert_eq!(derived, b"{Alice}");
        assert_eq!(hinted, derived);
    }

    /// Negative control. Proves the hint is actually consulted, so the two
    /// agreement tests above are not passing trivially.
    ///
    /// The observable effect is stronger than lost framing: `has_struct_payload`
    /// feeds *two* decisions — whether the codec adds top-level framing, and
    /// (at the `has_body_members` computation) whether the serialized body is
    /// kept at all. A member carrying `@httpPayload` is excluded from the
    /// "unbound member" scan, so once the hint wrongly says there is no struct
    /// payload, nothing marks the struct as having body members and the payload
    /// is discarded. This is why the hint's default has to mean "unknown"
    /// rather than either boolean — a wrong value in *either* direction
    /// corrupts the request.
    #[test]
    fn wrong_struct_payload_hint_changes_request() {
        assert_eq!(
            body_for(&STRUCT_PAYLOAD_MIS_HINTED, &StructPayloadStruct),
            b"",
            "a mis-recorded hint must be observable, otherwise the agreement \
             tests above prove nothing"
        );
    }

    #[test]
    fn payload_hint_defaults_to_unknown() {
        assert_eq!(
            STRUCT_PAYLOAD_UNKNOWN.payload_hint(),
            crate::PayloadHint::Unknown
        );
        assert_eq!(TEST_SCHEMA.payload_hint(), crate::PayloadHint::Unknown);
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

    // -- @httpPayload must not borrow from the caller's stack ------------------------------------

    static BLOB_PAYLOAD_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "PayloadStruct"),
        ShapeType::Blob,
        "data",
        0,
    )
    .with_http_payload();
    static BLOB_PAYLOAD_MEMBERS: &[&Schema<'_>] = &[&BLOB_PAYLOAD_MEMBER];
    static BLOB_PAYLOAD_STRUCT: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "PayloadStruct"),
        ShapeType::Structure,
        BLOB_PAYLOAD_MEMBERS,
    );

    static STRING_PAYLOAD_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "PayloadStruct"),
        ShapeType::String,
        "data",
        0,
    )
    .with_http_payload();
    static STRING_PAYLOAD_MEMBERS: &[&Schema<'_>] = &[&STRING_PAYLOAD_MEMBER];
    static STRING_PAYLOAD_STRUCT: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "PayloadStruct"),
        ShapeType::Structure,
        STRING_PAYLOAD_MEMBERS,
    );

    /// Computes its payload into a local that is dropped before `serialize_members` returns.
    ///
    /// This is legal against the public `ShapeSerializer` contract: neither `write_blob` nor
    /// `write_string` requires its argument to outlive the serializer. Generated code happens to
    /// pass values derived from the input struct, but a hand-written `SerializableStruct` — the
    /// case the dynamic-client and type-registry work exists to enable — need not.
    struct LocallyComputedPayload {
        blob: bool,
    }

    impl SerializableStruct for LocallyComputedPayload {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            if self.blob {
                // Heap-allocated here and freed at the end of this scope.
                let computed: Vec<u8> = (0u8..64).collect();
                s.write_blob(&BLOB_PAYLOAD_MEMBER, aws_smithy_types::Blob::new(computed))
            } else {
                let computed: String = (0..16).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
                s.write_string(&STRING_PAYLOAD_MEMBER, &computed)
            }
        }
    }

    /// Regression test for a use-after-free in the `@httpPayload` binding.
    ///
    /// `raw_payload` used to be `Option<&'a [u8]>`, populated by transmuting the `write_blob` /
    /// `write_string` argument's anonymous lifetime up to `'a`. `'a` is bound by the input schema
    /// and the headers, *not* by the serialized value, so the assertion was unfounded: a payload
    /// computed into a local was freed before `serialize_request` read `raw_payload` back. Miri
    /// reported `encountered a dangling reference (use-after-free)` while constructing the
    /// `Option<&[u8]>`.
    ///
    /// Both members are now owned, so the payload outlives the frame that produced it. Run this
    /// under Miri — a plain `cargo test` may well pass on freed-but-unreused memory:
    ///
    /// ```text
    /// cargo +nightly miri test -p aws-smithy-schema --lib http_payload_from_a_local
    /// ```
    #[test]
    fn http_payload_from_a_local_does_not_dangle() {
        let request = make_protocol()
            .serialize_request(
                &LocallyComputedPayload { blob: true },
                &BLOB_PAYLOAD_STRUCT,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        // Reading the bytes is the point: it dereferences what `raw_payload` retained.
        let expected: Vec<u8> = (0u8..64).collect();
        assert_eq!(request.body().bytes().unwrap(), &expected[..]);

        let request = make_protocol()
            .serialize_request(
                &LocallyComputedPayload { blob: false },
                &STRING_PAYLOAD_STRUCT,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.body().bytes().unwrap(), b"abcdefghijklmnop");
    }

    /// The blob payload path must not copy: `write_blob` takes an owned `Blob`, so the `Bytes`
    /// handed to `SdkBody` should be the very same allocation the caller built.
    ///
    /// Asserting on the pointer is the only way to observe this; a byte-equality assertion passes
    /// either way and would not notice a regression back to copying.
    #[test]
    fn blob_payload_reaches_the_body_without_copying() {
        struct OwnedBlobPayload(aws_smithy_types::Blob);
        impl SerializableStruct for OwnedBlobPayload {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_blob(&BLOB_PAYLOAD_MEMBER, self.0.clone())
            }
        }

        let payload = aws_smithy_types::Blob::new(vec![7u8; 4096]);
        let src_ptr = payload.as_ref().as_ptr();

        let request = make_protocol()
            .serialize_request(
                &OwnedBlobPayload(payload),
                &BLOB_PAYLOAD_STRUCT,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();

        let body = request.body().bytes().expect("payload body is in memory");
        assert_eq!(body.len(), 4096);
        assert_eq!(
            body.as_ptr(),
            src_ptr,
            "blob payload was copied; `write_blob` should move the `Bytes` handle all the way \
             into `SdkBody`"
        );
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

    static HEADER_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "xToken",
        0,
    )
    .with_http_header("X-Token");

    static HEADER_SCHEMA: Schema<'static> = Schema::new_struct(
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

    /// Header binding for a schema built at *runtime*: the structural strings
    /// are borrowed from a local arena (so `'a` is a function-body lifetime,
    /// not `'static`), while the header name itself is `'static` — standing in
    /// for a name a dynamic client would intern once at model-load time.
    ///
    /// This is what the `@httpHeader` constructor pin costs and what it still
    /// permits: a non-`'static` schema binds headers fine, and the insert stays
    /// allocation-free because `value_static()` is `Some`.
    #[test]
    fn http_header_on_a_runtime_built_schema() {
        // Structural strings: owned locally, dropped at end of scope. The
        // header name is interned, which is how a real runtime-built schema
        // satisfies the `'static` bound on `with_http_header`.
        let arena: Vec<String> = vec![
            String::from("token"),
            String::from("runtime-value"),
            String::from("X-Interned-Token"),
        ];

        let member: Schema<'_> = Schema::new_member(
            crate::shape_id!("test", "S"),
            ShapeType::String,
            &arena[0],
            0,
        )
        .with_http_header(crate::intern_header_name(&arena[2]));

        // The binder's fast path is available for this runtime schema.
        assert_eq!(
            member.http_header().unwrap().value_static(),
            Some("X-Interned-Token")
        );

        let members = [&member];
        let schema = Schema::new_struct(
            crate::shape_id!("test", "S"),
            ShapeType::Structure,
            &members,
        );

        struct RuntimeStruct<'a>(&'a Schema<'a>, &'a str);
        impl SerializableStruct for RuntimeStruct<'_> {
            fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                s.write_string(self.0, self.1)
            }
        }

        let request = make_protocol()
            .serialize_request(
                &RuntimeStruct(&member, &arena[1]),
                &schema,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();

        assert_eq!(
            request.headers().get("X-Interned-Token").unwrap(),
            "runtime-value"
        );
    }

    static INT_HEADER_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Integer,
        "retryCount",
        0,
    )
    .with_http_header("X-Retry-Count");

    static INT_HEADER_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static BOOL_HEADER_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Boolean,
        "verbose",
        0,
    )
    .with_http_header("X-Verbose");

    static BOOL_HEADER_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static QUERY_MEMBER: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "color", 0)
            .with_http_query("color");

    static QUERY_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static INT_QUERY_MEMBER: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Integer, "size", 0)
            .with_http_query("size");

    static INT_QUERY_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static Q1: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "a", 0)
            .with_http_query("a");
    static Q2: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "b", 1)
            .with_http_query("b");
    static MULTI_QUERY_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static LABEL_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "bucketName",
        0,
    )
    .with_http_label();

    static LABEL_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static INT_LABEL_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Integer,
        "itemId",
        0,
    )
    .with_http_label();

    static INT_LABEL_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static COMBINED_LABEL: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "id", 0)
            .with_http_label();
    static COMBINED_HEADER: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "token", 1)
            .with_http_header("X-Token");
    static COMBINED_QUERY: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "filter",
        2,
    )
    .with_http_query("filter");
    static COMBINED_BODY: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::String, "data", 3);
    static COMBINED_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static PREFIX_MEMBER: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Map, "metadata", 0)
            .with_http_prefix_headers("X-Meta-");

    static PREFIX_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static QUERY_PARAMS_MEMBER: Schema<'static> =
        Schema::new_member(crate::shape_id!("test", "S"), ShapeType::Map, "params", 0)
            .with_http_query_params();

    static QUERY_PARAMS_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static TS_HEADER_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Timestamp,
        "ifModified",
        0,
    )
    .with_http_header("If-Modified-Since");

    static TS_HEADER_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static TS_QUERY_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::Timestamp,
        "since",
        0,
    )
    .with_http_query("since");

    static TS_QUERY_SCHEMA: Schema<'static> = Schema::new_struct(
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

    static BOUND_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "headerVal",
        0,
    )
    .with_http_header("X-Val");
    static UNBOUND_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "S"),
        ShapeType::String,
        "bodyVal",
        1,
    );
    static MIXED_SCHEMA: Schema<'static> = Schema::new_struct(
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
