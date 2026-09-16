/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! XML serializer.
//!
//! Implements the schema-serde [`ShapeSerializer`] trait for the AWS REST XML
//! protocol. Serialization is single-pass with a deferred start-tag flush:
//! when a struct's start tag is written, the closing `>` is held back until
//! either an attribute is added or the first child element / text is
//! written. This lets us emit attributes inline with the start tag without
//! buffering the start tag separately.

use super::XmlCodecSettings;
use aws_smithy_schema::codec::FinishSerializer;
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
use aws_smithy_schema::Schema;
use aws_smithy_types::date_time::Format as TimestampFormat;
use aws_smithy_types::{BigDecimal, BigInteger, DateTime, Document};
use std::sync::Arc;

/// XML serializer that implements the [`ShapeSerializer`] trait.
pub struct XmlSerializer {
    output: String,
    settings: Arc<XmlCodecSettings>,
    /// Stack of open elements. Top of stack is the deepest currently-open
    /// element. The frame state distinguishes "start tag still open" from
    /// "start tag closed, inside body".
    frames: Vec<Frame>,
    /// When inside a `write_map` callback, tracks the entry/key/value state.
    map_state: Option<MapState>,
    /// When inside a `write_list` callback, overrides the element name for items.
    list_item_name: Option<String>,
    /// When inside a `write_list` callback, propagates the inner-list-member
    /// `@xmlNamespace` (uri, prefix) to scalar item writes whose schema is a
    /// generic prelude type and therefore doesn't carry the trait itself.
    list_item_namespace: Option<(String, Option<String>)>,
    /// Two-pass `serialize_members` filter for the immediately containing
    /// `write_struct` call. XML attributes must appear inside the start tag,
    /// before any child element closes it; the codegen-generated
    /// `serialize_members` does not order attribute members first, so the
    /// codec calls `serialize_members` twice — once with `AttributesOnly` to
    /// emit only `@xmlAttribute` members, and once with `NonAttributesOnly`
    /// to emit everything else. This keeps generated code protocol-neutral
    /// and preserves runtime protocol selection: the order of attribute vs.
    /// element members is the codec's concern, not the generated code's.
    ///
    /// Trade-off: `serialize_members` is iterated twice per struct. The
    /// actual byte-writing work happens once (the skipped pass exits early
    /// at each `write_*` filter check), so the overhead is proportional to
    /// member count and dominated by the work of iterating the generated
    /// `if let Some(ref val) = self.x` chain — measurable for very wide
    /// structs but not for typical shapes.
    ///
    /// Alternative considered: a single-pass buffered-children approach.
    /// `write_struct` would record `attr_insert_pos` (the byte offset just
    /// after the element name in `output`), keep that position valid even
    /// after the start tag is "flushed", and route attribute writes through
    /// `output.insert_str(attr_insert_pos, ...)` no matter when they arrive.
    /// That would let `serialize_members` run once in declaration order
    /// while still placing attributes inside the start tag. The cost is an
    /// `O(n)` shift of the output buffer per attribute write where `n` is
    /// the bytes already written after the start tag — potentially worse
    /// than two-pass for structs with many child elements followed by an
    /// attribute, and noticeably more code for the frame-tracking. We chose
    /// two-pass for simplicity and predictability; revisit if benchmarks
    /// show double-iteration is a real cost.
    member_filter: MemberFilter,
    /// One-shot override for the wrapper element name on the next
    /// document-root `write_struct`. Consumed (`take`d) on first use.
    /// Set externally (e.g. by `AwsRestXmlProtocol::serialize_request`)
    /// when the body root must be named after a member's `@xmlName` that
    /// the codec couldn't see — for example, an `@httpPayload` struct member
    /// whose codegen passes the *target* shape's `SCHEMA` (which may carry
    /// its own `@xmlName`) but whose member-level `@xmlName` should win
    /// per the Smithy spec.
    next_root_xml_name: Option<String>,
    /// One-shot override for the `xmlns` attribute on the next document-root
    /// element. Consumed on first use. Intended for protocol-layer use:
    /// REST XML services may declare a service-level `@xmlNamespace` that
    /// applies as the default xmlns to every operation's request/response
    /// XML root, but per the Smithy spec a shape- or member-level
    /// `@xmlNamespace` overrides it. Schemas only carry shape/member
    /// namespaces; the protocol pre-sets this field with the service
    /// default, and the codec consumes it on the next root write — but
    /// only if the schema itself has no `xml_namespace`.
    next_root_xml_namespace: Option<(String, Option<String>)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum MemberFilter {
    /// No filtering — emit every write call.
    None,
    /// Emit only members whose schema has `@xmlAttribute`.
    AttributesOnly,
    /// Emit only members whose schema does NOT have `@xmlAttribute`.
    NonAttributesOnly,
}

/// Tracks alternating key/value writes inside a map callback.
#[derive(Debug)]
struct MapState {
    /// Element name for each entry (e.g., "entry" or the member name for flattened).
    entry_name: String,
    /// Element name for the key (default "key", overridable via @xmlName).
    key_name: String,
    /// Element name for the value (default "value", overridable via @xmlName).
    value_name: String,
    /// `@xmlNamespace` (uri, prefix) on the key member, if any.
    key_namespace: Option<(String, Option<String>)>,
    /// `@xmlNamespace` (uri, prefix) on the value member, if any.
    value_namespace: Option<(String, Option<String>)>,
    /// Schema of this map's value member. Used by a nested inner `write_map`
    /// (when codegen passes `prelude::DOCUMENT` for the inner aggregate)
    /// to recover the inner map's own `_KEY` / `_VALUE` schemas through
    /// `value_schema.key()` / `.value()`. `None` when the value is a scalar.
    value_schema: Option<&'static Schema>,
    /// True when the next write is a key (odd writes), false for value (even writes).
    expecting_key: bool,
}

#[derive(Debug)]
enum Frame {
    /// `<name` has been written; the closing `>` is deferred so that
    /// attributes and namespaces can still be added inline with the start tag.
    /// Attributes are buffered so they can arrive in any order relative to
    /// child elements (protocol-neutral serialize_members ordering).
    StartTagPending { name: String, attrs: String },
    /// `<name attrs>` has been fully written; we are now inside the element body.
    Open { name: String },
}

/// Append the `xmlns="..."` (or `xmlns:prefix="..."`) attribute fragment
/// for `ns` directly into `out`. A no-op when `ns` is `None`.
///
/// Used by inline element emission paths (e.g., map entries) where the
/// frame-based [`XmlSerializer::write_xmlns`] helper isn't applicable
/// because the element is emitted via `write!(self.output, ...)` rather
/// than going through `open_element`.
///
/// Replaces an earlier `format_xmlns_attr -> String` helper that
/// allocated a fresh `String` per call. Per-entry map serialization can
/// call this twice per entry (key + value), so on map-heavy payloads the
/// allocations added up.
fn write_xmlns_attr(out: &mut String, ns: Option<&(String, Option<String>)>) {
    use std::fmt::Write;
    match ns {
        Some((uri, Some(prefix))) => {
            write!(out, " xmlns:{prefix}=\"{uri}\"").unwrap();
        }
        Some((uri, None)) => {
            write!(out, " xmlns=\"{uri}\"").unwrap();
        }
        None => {}
    }
}

impl XmlSerializer {
    /// Creates a new XML serializer with the given settings.
    pub(crate) fn new(settings: Arc<XmlCodecSettings>) -> Self {
        Self {
            output: String::new(),
            settings,
            frames: Vec::new(),
            map_state: None,
            list_item_name: None,
            list_item_namespace: None,
            member_filter: MemberFilter::None,
            next_root_xml_name: None,
            next_root_xml_namespace: None,
        }
    }

    /// Sets a one-shot override for the wrapper element name on the next
    /// document-root `write_struct`. Consumed on first use. Intended for
    /// protocol-layer use (e.g. REST XML routing of `@httpPayload` struct
    /// members whose member-level `@xmlName` would otherwise be invisible
    /// to the codec — codegen passes the target shape's `SCHEMA`, which
    /// carries the target's `@xmlName` but not the member's).
    pub fn set_next_root_xml_name(&mut self, name: String) {
        self.next_root_xml_name = Some(name);
    }

    /// Sets a one-shot fallback xmlns for the document-root element. Consumed
    /// on the first root-level `write_struct` only when the struct's own
    /// schema has no `xml_namespace` (per the Smithy spec, a shape-level
    /// `@xmlNamespace` overrides any service-level default). Intended for
    /// the REST XML protocol to apply the service-level `@xmlNamespace`
    /// to operation request/response root elements.
    pub fn set_next_root_xml_namespace(&mut self, uri: String, prefix: Option<String>) {
        self.next_root_xml_namespace = Some((uri, prefix));
    }

    /// Returns true if a write of `schema` should produce output under the
    /// current `member_filter`. Top-level calls (no filter set) always pass.
    fn filter_allows(&self, schema: &Schema) -> bool {
        match self.member_filter {
            MemberFilter::None => true,
            MemberFilter::AttributesOnly => schema.xml_attribute(),
            MemberFilter::NonAttributesOnly => !schema.xml_attribute(),
        }
    }

    /// Resolve the XML element name for a schema being serialized as an element.
    ///
    /// Resolution order:
    /// 1. `@xmlName` (member-level wins over shape-level via the codegen-emitted
    ///    member schema).
    /// 2. `original_name` — the synthetic shape's pre-rename name. Set only on
    ///    operation input/output synthetic shapes.
    /// 3. `member_name` — the smithy member name, set on member schemas.
    /// 4. `shape_id().shape_name()` — fallback for non-synthetic, non-member shapes.
    fn element_name(schema: &Schema) -> &str {
        schema
            .xml_name()
            .map(|t| t.value())
            .or_else(|| schema.original_name())
            .or_else(|| schema.member_name())
            .unwrap_or_else(|| schema.shape_id().shape_name())
    }

    /// If the top frame is a [`Frame::StartTagPending`], close its start tag
    /// (write `>`) and transition the frame to [`Frame::Open`]. No-op
    /// otherwise. Called before any child content (text or nested element)
    /// is written.
    fn flush_start_tag(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            if let Frame::StartTagPending { name, attrs } = frame {
                self.output.push_str(attrs);
                self.output.push('>');
                let name = std::mem::take(name);
                *frame = Frame::Open { name };
            }
        }
    }

    /// Write `<name` and push a new [`Frame::StartTagPending`]. The caller
    /// must have already flushed any parent's pending start tag (via
    /// [`Self::flush_start_tag`]) so the new element doesn't end up nested
    /// inside an unclosed tag.
    fn open_element(&mut self, name: &str) {
        self.output.push('<');
        self.output.push_str(name);
        self.frames.push(Frame::StartTagPending {
            name: name.to_owned(),
            attrs: String::new(),
        });
    }

    /// Append `xmlns` / `xmlns:prefix` attribute(s) to the most recently
    /// opened start tag. The schema's `@xmlNamespace` is preferred. If the
    /// schema has no namespace and `inherited` is `Some`, that fallback is
    /// used — this is how a list's inner-member `@xmlNamespace` reaches each
    /// scalar item write whose schema is a generic prelude type and therefore
    /// doesn't carry the trait itself.
    ///
    /// If neither schema nor `inherited` provides a namespace AND this is
    /// the document-root frame (only one frame on the stack), the
    /// one-shot [`Self::next_root_xml_namespace`] override is consumed.
    /// This is how the REST XML protocol applies a service-level
    /// `@xmlNamespace` to the request/response root element without
    /// codec-time knowledge of the service.
    fn write_xmlns(&mut self, schema: &Schema, inherited: Option<&(String, Option<String>)>) {
        use std::fmt::Write;
        let is_document_root = self.frames.len() == 1;
        // Consume the document-root override only at the root, only when
        // the schema and any caller-provided fallback don't already carry a
        // namespace. Take it eagerly so it cannot leak to a subsequent root
        // write on the same serializer (which is unusual but possible).
        let root_override =
            if is_document_root && schema.xml_namespace().is_none() && inherited.is_none() {
                self.next_root_xml_namespace.take()
            } else {
                None
            };
        let Some(Frame::StartTagPending { attrs, .. }) = self.frames.last_mut() else {
            return;
        };
        if let Some(ns) = schema.xml_namespace() {
            match ns.prefix() {
                Some(prefix) => write!(attrs, " xmlns:{prefix}=\"{}\"", ns.uri()).unwrap(),
                None => write!(attrs, " xmlns=\"{}\"", ns.uri()).unwrap(),
            }
        } else if let Some((uri, prefix)) = inherited {
            match prefix.as_deref() {
                Some(p) => write!(attrs, " xmlns:{p}=\"{uri}\"").unwrap(),
                None => write!(attrs, " xmlns=\"{uri}\"").unwrap(),
            }
        } else if let Some((uri, prefix)) = root_override {
            match prefix {
                Some(p) => write!(attrs, " xmlns:{p}=\"{uri}\"").unwrap(),
                None => write!(attrs, " xmlns=\"{uri}\"").unwrap(),
            }
        }
    }

    /// Pop the top frame and emit the closing tag.
    ///
    /// Always emits `<name attrs>...</name>` form, never `<name attrs/>`.
    /// Both forms are equivalent XML, but legacy smithy-rs (and S3's recorded
    /// request fixtures) use the explicit-close form. Matching that prevents
    /// false-positive content-length mismatches in DVR-replay tests like
    /// `s3::select_object_content::test_success`, where `<CSV></CSV>` differs
    /// from `<CSV/>` by 5 bytes.
    fn close_element(&mut self) {
        let frame = self
            .frames
            .pop()
            .expect("close_element called with empty frame stack");
        match frame {
            Frame::StartTagPending { name, attrs } => {
                self.output.push_str(&attrs);
                self.output.push('>');
                self.output.push_str("</");
                self.output.push_str(&name);
                self.output.push('>');
            }
            Frame::Open { name } => {
                self.output.push_str("</");
                self.output.push_str(&name);
                self.output.push('>');
            }
        }
    }

    /// Emit `<name>content</name>` where `content` is already safe (no
    /// XML-special chars). Used for numbers, booleans, base64 — values that
    /// are known not to need escaping. If the schema has `@xmlAttribute`,
    /// writes an attribute on the parent's pending start tag instead.
    fn write_safe_element(&mut self, schema: &Schema, content: &str) {
        if !self.filter_allows(schema) {
            return;
        }
        if self.try_write_attribute(schema, content) {
            return;
        }
        use std::fmt::Write;
        self.flush_start_tag();
        if let Some(map_state) = &mut self.map_state {
            if map_state.expecting_key {
                // Open entry element, write key (with optional key @xmlNamespace)
                let entry = &map_state.entry_name.clone();
                let key = &map_state.key_name.clone();
                write!(self.output, "<{entry}><{key}").unwrap();
                write_xmlns_attr(&mut self.output, map_state.key_namespace.as_ref());
                write!(self.output, ">{content}</{key}>").unwrap();
                map_state.expecting_key = false;
            } else {
                // Write value (with optional value @xmlNamespace), close entry element
                let entry = &map_state.entry_name.clone();
                let value = &map_state.value_name.clone();
                write!(self.output, "<{value}").unwrap();
                write_xmlns_attr(&mut self.output, map_state.value_namespace.as_ref());
                write!(self.output, ">{content}</{value}></{entry}>").unwrap();
                map_state.expecting_key = true;
            }
        } else {
            let name = if schema.xml_name().is_none() {
                self.list_item_name
                    .clone()
                    .unwrap_or_else(|| Self::element_name(schema).to_string())
            } else {
                Self::element_name(schema).to_string()
            };
            // Use open_element/close_element so namespace attributes can be
            // emitted via write_xmlns into the still-pending start tag.
            self.open_element(&name);
            let inherited = self.list_item_namespace.clone();
            self.write_xmlns(schema, inherited.as_ref());
            self.flush_start_tag();
            self.output.push_str(content);
            self.close_element();
        }
    }

    /// If the schema has `@xmlAttribute` and the parent frame's start tag is
    /// still pending, write ` name="escaped_value"` into the attrs buffer and
    /// return `true`. Otherwise return `false` (caller should emit a child
    /// element instead).
    fn try_write_attribute(&mut self, schema: &Schema, value: &str) -> bool {
        use std::fmt::Write;
        if !schema.xml_attribute() {
            return false;
        }
        // Buffer the attribute on the nearest pending start tag frame.
        // If the frame was already flushed to Open, we can't add attributes —
        // but with the current design, flush only happens when opening a nested
        // element, and attributes are always scalars, so this case shouldn't occur
        // in practice. We handle it gracefully by checking.
        if let Some(Frame::StartTagPending { attrs, .. }) = self.frames.last_mut() {
            let name = Self::element_name(schema);
            let escaped = crate::escape::escape(value);
            write!(attrs, " {name}=\"{escaped}\"").unwrap();
            return true;
        }
        false
    }

    /// Resolve the timestamp format for a member. Member-level
    /// `@timestampFormat` wins; otherwise the codec's default (`date-time`
    /// for REST XML body bindings).
    fn resolve_timestamp_format(&self, schema: &Schema) -> TimestampFormat {
        schema
            .timestamp_format()
            .map(|t| match t.format() {
                aws_smithy_schema::traits::TimestampFormat::EpochSeconds => {
                    TimestampFormat::EpochSeconds
                }
                aws_smithy_schema::traits::TimestampFormat::DateTime => TimestampFormat::DateTime,
                aws_smithy_schema::traits::TimestampFormat::HttpDate => TimestampFormat::HttpDate,
            })
            .unwrap_or(self.settings.default_timestamp_format())
    }
}

impl FinishSerializer for XmlSerializer {
    fn finish(self) -> Vec<u8> {
        debug_assert!(
            self.frames.is_empty(),
            "XmlSerializer::finish called with {} unclosed frame(s)",
            self.frames.len()
        );
        self.output.into_bytes()
    }
}

impl ShapeSerializer for XmlSerializer {
    fn write_struct(
        &mut self,
        schema: &Schema,
        value: &dyn SerializableStruct,
    ) -> Result<(), SerdeError> {
        if schema.xml_attribute() {
            return Err(SerdeError::custom(
                "@xmlAttribute is not supported on aggregate types",
            ));
        }
        if !self.filter_allows(schema) {
            return Ok(());
        }
        self.flush_start_tag();

        // Handle map entry framing when this struct is a map value.
        let in_map_value = if let Some(map_state) = &mut self.map_state {
            if map_state.expecting_key {
                false
            } else {
                // This struct is the map value — open entry's value element.
                // The struct's members are serialized directly inside <value>,
                // without the struct's own element wrapper (per Smithy XML spec).
                // Note: <entry> was already opened by the preceding key write.
                use std::fmt::Write;
                let val_name = &map_state.value_name.clone();
                write!(self.output, "<{val_name}>").unwrap();
                map_state.expecting_key = true;
                true
            }
        } else {
            false
        };

        // Temporarily clear map_state so nested writes don't get map framing
        let saved_map_state = if in_map_value {
            self.map_state.take()
        } else {
            None
        };

        if in_map_value {
            // Map value struct: serialize members directly (no struct element wrapper).
            // Two-pass for attribute ordering: pass 1 emits attribute members,
            // pass 2 emits non-attribute members. The first pass is skipped
            // entirely when no member of this struct has `@xmlAttribute` —
            // every `write_*` call would otherwise no-op via `filter_allows`,
            // but the codegen-generated `serialize_members` body still
            // iterates each value field on the way to those no-ops. Avoiding
            // a full pass over value fields when there's nothing to emit
            // halves the per-struct serialization cost on the common case
            // (Smithy structures with no attributes).
            let saved_filter = self.member_filter;
            // See the comment on the corresponding `has_attrs` check in the
            // non-map-value branch below: member schemas (`Schema::new_member`)
            // leave `members()` empty regardless of the target's shape type,
            // so we run the AttributesOnly pass conservatively when the
            // schema is a member.
            let has_attrs = schema.member_name().is_some()
                || schema.members().iter().any(|m| m.xml_attribute());
            if has_attrs {
                self.member_filter = MemberFilter::AttributesOnly;
                value.serialize_members(self)?;
            }
            self.member_filter = MemberFilter::NonAttributesOnly;
            value.serialize_members(self)?;
            self.member_filter = saved_filter;
            // Close value element and entry element
            use std::fmt::Write;
            let saved = saved_map_state.as_ref().unwrap();
            let val_name = &saved.value_name;
            let entry = &saved.entry_name;
            write!(self.output, "</{val_name}></{entry}>").unwrap();
            self.map_state = saved_map_state;
        } else {
            // Document-root override: if a protocol layer pre-set
            // `next_root_xml_name` (e.g. REST XML resolving an
            // `@httpPayload` member's `@xmlName` that codegen couldn't
            // surface), consume it here. Only applies at the document root —
            // nested struct calls always have at least one open frame.
            let root_override = if self.frames.is_empty() {
                self.next_root_xml_name.take()
            } else {
                None
            };
            let name = if let Some(override_name) = root_override {
                override_name
            } else if schema.xml_name().is_none() {
                self.list_item_name
                    .clone()
                    .unwrap_or_else(|| Self::element_name(schema).to_string())
            } else {
                Self::element_name(schema).to_string()
            };
            let saved_list_item = self.list_item_name.take();
            self.open_element(&name);
            self.write_xmlns(schema, None);
            // Two-pass: attributes first (so they land in the still-pending
            // start tag), non-attributes second (which flushes the start tag
            // and emits child elements). The attributes pass is skipped
            // entirely when no member has `@xmlAttribute` — see comment in
            // the map-value branch above.
            //
            // Member schemas (`Schema::new_member`) carry the target shape's
            // `shape_type` but leave `members()` empty — the target struct's
            // member list lives on the target schema, not on the member
            // wrapper. When `schema` is a member schema, `members().iter()`
            // can't see the target's `@xmlAttribute` members, so we
            // conservatively run the AttributesOnly pass and let the
            // per-member `filter_allows` checks gate emission. Root-struct
            // calls (`schema` is a struct schema with `members()`
            // populated) keep the optimization. The discriminator is
            // `member_name()` — `Some(_)` only when the schema was built
            // via `Schema::new_member`.
            let saved_filter = self.member_filter;
            let has_attrs = schema.member_name().is_some()
                || schema.members().iter().any(|m| m.xml_attribute());
            if has_attrs {
                self.member_filter = MemberFilter::AttributesOnly;
                value.serialize_members(self)?;
            }
            self.member_filter = MemberFilter::NonAttributesOnly;
            value.serialize_members(self)?;
            self.member_filter = saved_filter;
            self.close_element();
            self.list_item_name = saved_list_item;
        }
        Ok(())
    }

    fn write_list(
        &mut self,
        schema: &Schema,
        write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        if schema.xml_attribute() {
            return Err(SerdeError::custom(
                "@xmlAttribute is not supported on aggregate types",
            ));
        }
        if !self.filter_allows(schema) {
            return Ok(());
        }

        self.flush_start_tag();

        // Handle being called as a map value
        let in_map_value = if let Some(map_state) = &mut self.map_state {
            if !map_state.expecting_key {
                use std::fmt::Write;
                let val_name = &map_state.value_name.clone();
                write!(self.output, "<{val_name}>").unwrap();
                map_state.expecting_key = true;
                true
            } else {
                false
            }
        } else {
            false
        };
        let saved_map_state = if in_map_value {
            self.map_state.take()
        } else {
            None
        };

        // Resolve the wrapper element name. If the schema has @xmlName, use it.
        // Otherwise, when this list is itself a list element of an outer call
        // (parent set `self.list_item_name`), use the parent's child-name. This
        // lets nested write_list calls produce the correct wrapper name even
        // when codegen passes a generic placeholder schema (e.g. prelude::DOCUMENT)
        // for the inner aggregate. Only matters for non-flattened, non-map-value
        // wrappers.
        let wrapper_name = if schema.xml_name().is_none() {
            self.list_item_name
                .clone()
                .unwrap_or_else(|| Self::element_name(schema).to_string())
        } else {
            Self::element_name(schema).to_string()
        };

        // Resolve the item element name.
        // Per the Smithy XML spec:
        //  - For wrapped lists, items are emitted using the inner list member's
        //    name (default "member", overridable via @xmlName on the list's
        //    member shape).
        //  - For flattened lists, the wrapper element is omitted and items are
        //    emitted using the OUTER member's name (the list member's xml_name
        //    or its smithy member_name) — the inner list member's name is
        //    ignored because there is no wrapper to host it.
        let item_name = if schema.xml_flattened() {
            // Flattened: outer member's resolved element name (xml_name → list_item_name → member_name).
            if schema.xml_name().is_none() {
                self.list_item_name
                    .clone()
                    .unwrap_or_else(|| Self::element_name(schema).to_string())
            } else {
                Self::element_name(schema).to_string()
            }
        } else {
            schema
                .member()
                .and_then(|m| m.xml_name().map(|n| n.value().to_string()))
                .or_else(|| {
                    schema
                        .member()
                        .and_then(|m| m.member_name().map(|s| s.to_string()))
                })
                .unwrap_or_else(|| "member".to_string())
        };

        let saved_list_item = self.list_item_name.take();
        self.list_item_name = Some(item_name);

        // Propagate the list's inner-member @xmlNamespace so scalar item
        // writes (whose schema is typically a generic prelude type) can
        // inherit the namespace declaration. Only wrap-level lists set this;
        // for flattened lists the items are at the parent level and use the
        // parent member's own namespace if any.
        let saved_list_item_ns = self.list_item_namespace.take();
        self.list_item_namespace = schema.member().and_then(|m| {
            m.xml_namespace()
                .map(|ns| (ns.uri().to_owned(), ns.prefix().map(|p| p.to_owned())))
        });

        if schema.xml_flattened() || in_map_value {
            write_elements(self)?;
        } else {
            self.open_element(&wrapper_name);
            // The wrapper element gets the OUTER list member's @xmlNamespace
            // (which is on `schema` itself), not the inner-member namespace.
            self.write_xmlns(schema, None);
            write_elements(self)?;
            self.close_element();
        }

        self.list_item_name = saved_list_item;
        self.list_item_namespace = saved_list_item_ns;

        if in_map_value {
            use std::fmt::Write;
            let saved = saved_map_state.as_ref().unwrap();
            let val_name = &saved.value_name;
            let entry = &saved.entry_name;
            write!(self.output, "</{val_name}></{entry}>").unwrap();
            self.map_state = saved_map_state;
        }
        Ok(())
    }

    fn write_map(
        &mut self,
        schema: &Schema,
        write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        if schema.xml_attribute() {
            return Err(SerdeError::custom(
                "@xmlAttribute is not supported on aggregate types",
            ));
        }
        if !self.filter_allows(schema) {
            return Ok(());
        }

        self.flush_start_tag();

        // If we're being called as a value of an outer map and the schema we
        // were given has no map members chained (e.g. codegen passed
        // `prelude::DOCUMENT` for the inner aggregate), substitute in the
        // outer map's saved `value_schema`, which carries the inner map's
        // own `_KEY` / `_VALUE` chain (set up by the outer `write_map`).
        // This is what lets nested-map element-name overrides (`@xmlName`
        // on inner key / value) reach the runtime without making codegen
        // recurse into the body emission path.
        let effective_schema: &Schema = if schema.key().is_none() && schema.member().is_none() {
            self.map_state
                .as_ref()
                .and_then(|s| s.value_schema)
                .unwrap_or(schema)
        } else {
            schema
        };
        let schema = effective_schema;

        // Handle being called as a map value (nested maps)
        let in_map_value = if let Some(map_state) = &mut self.map_state {
            if !map_state.expecting_key {
                use std::fmt::Write;
                let val_name = &map_state.value_name.clone();
                write!(self.output, "<{val_name}>").unwrap();
                map_state.expecting_key = true;
                true
            } else {
                false
            }
        } else {
            false
        };

        let outer_map_state = self.map_state.take();

        // Resolve entry/key/value element names from the schema's map members.
        let entry_name = if schema.xml_flattened() {
            Self::element_name(schema).to_string()
        } else {
            "entry".to_string()
        };
        let key_name = schema
            .key()
            .and_then(|k| k.xml_name().map(|n| n.value().to_string()))
            .unwrap_or_else(|| {
                schema
                    .key()
                    .and_then(|k| k.member_name().map(|s| s.to_string()))
                    .unwrap_or_else(|| "key".to_string())
            });
        let value_name = schema
            .member()
            .and_then(|v| v.xml_name().map(|n| n.value().to_string()))
            .unwrap_or_else(|| {
                schema
                    .member()
                    .and_then(|v| v.member_name().map(|s| s.to_string()))
                    .unwrap_or_else(|| "value".to_string())
            });

        self.map_state = Some(MapState {
            entry_name,
            key_name,
            value_name,
            key_namespace: schema.key().and_then(|k| {
                k.xml_namespace()
                    .map(|ns| (ns.uri().to_owned(), ns.prefix().map(|p| p.to_owned())))
            }),
            value_namespace: schema.member().and_then(|v| {
                v.xml_namespace()
                    .map(|ns| (ns.uri().to_owned(), ns.prefix().map(|p| p.to_owned())))
            }),
            // Save the value member's full schema so a nested inner write_map
            // (which codegen may invoke with `prelude::DOCUMENT`) can recover
            // the inner aggregate's own `_KEY`/`_VALUE` (or `_MEMBER`) chain.
            value_schema: schema.member_static(),
            expecting_key: true,
        });

        // Resolve the wrapper element name. If the schema has @xmlName, use it.
        // Otherwise, when this map is itself an element of an outer list (parent set
        // `self.list_item_name`), use the parent's child-name. This lets nested
        // write_map calls produce the correct wrapper name even when codegen
        // passes a generic placeholder schema (e.g. prelude::DOCUMENT) for the
        // inner aggregate. Only matters for non-flattened, non-map-value wrappers.
        let wrapper_name = if schema.xml_name().is_none() {
            self.list_item_name
                .clone()
                .unwrap_or_else(|| Self::element_name(schema).to_string())
        } else {
            Self::element_name(schema).to_string()
        };

        // Don't propagate parent list_item_name into map entries — entries are
        // framed by map_state instead.
        let saved_list_item = self.list_item_name.take();

        if schema.xml_flattened() || in_map_value {
            write_entries(self)?;
        } else {
            self.open_element(&wrapper_name);
            self.write_xmlns(schema, None);
            write_entries(self)?;
            self.close_element();
        }

        self.list_item_name = saved_list_item;

        self.map_state = outer_map_state;

        if in_map_value {
            use std::fmt::Write;
            let saved = self.map_state.as_ref().unwrap();
            let val_name = &saved.value_name;
            let entry = &saved.entry_name;
            write!(self.output, "</{val_name}></{entry}>").unwrap();
        }
        Ok(())
    }

    fn write_boolean(&mut self, schema: &Schema, value: bool) -> Result<(), SerdeError> {
        self.write_safe_element(schema, if value { "true" } else { "false" });
        Ok(())
    }

    fn write_byte(&mut self, schema: &Schema, value: i8) -> Result<(), SerdeError> {
        self.write_safe_element(schema, &value.to_string());
        Ok(())
    }

    fn write_short(&mut self, schema: &Schema, value: i16) -> Result<(), SerdeError> {
        self.write_safe_element(schema, &value.to_string());
        Ok(())
    }

    fn write_integer(&mut self, schema: &Schema, value: i32) -> Result<(), SerdeError> {
        self.write_safe_element(schema, &value.to_string());
        Ok(())
    }

    fn write_long(&mut self, schema: &Schema, value: i64) -> Result<(), SerdeError> {
        self.write_safe_element(schema, &value.to_string());
        Ok(())
    }

    fn write_float(&mut self, schema: &Schema, value: f32) -> Result<(), SerdeError> {
        let text = if value.is_nan() {
            "NaN".to_owned()
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                "Infinity".to_owned()
            } else {
                "-Infinity".to_owned()
            }
        } else {
            value.to_string()
        };
        self.write_safe_element(schema, &text);
        Ok(())
    }

    fn write_double(&mut self, schema: &Schema, value: f64) -> Result<(), SerdeError> {
        let text = if value.is_nan() {
            "NaN".to_owned()
        } else if value.is_infinite() {
            if value.is_sign_positive() {
                "Infinity".to_owned()
            } else {
                "-Infinity".to_owned()
            }
        } else {
            value.to_string()
        };
        self.write_safe_element(schema, &text);
        Ok(())
    }

    fn write_big_integer(&mut self, schema: &Schema, value: &BigInteger) -> Result<(), SerdeError> {
        self.write_safe_element(schema, value.as_ref());
        Ok(())
    }

    fn write_big_decimal(&mut self, schema: &Schema, value: &BigDecimal) -> Result<(), SerdeError> {
        self.write_safe_element(schema, value.as_ref());
        Ok(())
    }

    fn write_string(&mut self, schema: &Schema, value: &str) -> Result<(), SerdeError> {
        if !self.filter_allows(schema) {
            return Ok(());
        }
        if self.try_write_attribute(schema, value) {
            return Ok(());
        }
        use std::fmt::Write;
        self.flush_start_tag();
        let escaped = crate::escape::escape(value);
        if let Some(map_state) = &mut self.map_state {
            if map_state.expecting_key {
                let entry = &map_state.entry_name.clone();
                let key = &map_state.key_name.clone();
                write!(self.output, "<{entry}><{key}").unwrap();
                write_xmlns_attr(&mut self.output, map_state.key_namespace.as_ref());
                write!(self.output, ">{escaped}</{key}>").unwrap();
                map_state.expecting_key = false;
            } else {
                let entry = &map_state.entry_name.clone();
                let val_name = &map_state.value_name.clone();
                write!(self.output, "<{val_name}").unwrap();
                write_xmlns_attr(&mut self.output, map_state.value_namespace.as_ref());
                write!(self.output, ">{escaped}</{val_name}></{entry}>").unwrap();
                map_state.expecting_key = true;
            }
        } else {
            let name = if schema.xml_name().is_none() {
                self.list_item_name
                    .clone()
                    .unwrap_or_else(|| Self::element_name(schema).to_string())
            } else {
                Self::element_name(schema).to_string()
            };
            // Use open_element/close_element so namespace attributes can be
            // emitted via write_xmlns into the still-pending start tag.
            self.open_element(&name);
            let inherited = self.list_item_namespace.clone();
            self.write_xmlns(schema, inherited.as_ref());
            self.flush_start_tag();
            self.output.push_str(&escaped);
            self.close_element();
        }
        Ok(())
    }

    fn write_blob(&mut self, schema: &Schema, value: &[u8]) -> Result<(), SerdeError> {
        let encoded = aws_smithy_types::base64::encode(value);
        self.write_safe_element(schema, &encoded);
        Ok(())
    }

    fn write_timestamp(&mut self, schema: &Schema, value: &DateTime) -> Result<(), SerdeError> {
        let format = self.resolve_timestamp_format(schema);
        let formatted = value
            .fmt(format)
            .map_err(|e| SerdeError::custom(e.to_string()))?;
        // Timestamp text is safe (digits, dashes, colons, T, Z, dots) — no escaping needed.
        self.write_safe_element(schema, &formatted);
        Ok(())
    }

    fn write_document(&mut self, _schema: &Schema, _value: &Document) -> Result<(), SerdeError> {
        Err(SerdeError::custom(
            "document types are not supported by REST XML",
        ))
    }

    fn write_null(&mut self, _schema: &Schema) -> Result<(), SerdeError> {
        // XML represents null/absent members by omitting the element entirely.
        // Generated code skips None fields, so this should rarely be called.
        // If it is (e.g. sparse collections), we simply emit nothing.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::{prelude, shape_id, Schema, ShapeType};
    use aws_smithy_types::Blob;

    /// Renders a struct with one string member named `name`.
    static NAME_MEMBER: Schema = Schema::new_member(
        shape_id!("test", "Person$name"),
        ShapeType::String,
        "name",
        0,
    );
    static PERSON_SCHEMA: Schema = Schema::new_struct(
        shape_id!("test", "Person"),
        ShapeType::Structure,
        &[&NAME_MEMBER],
    );

    struct Person<'a> {
        name: &'a str,
    }

    impl SerializableStruct for Person<'_> {
        fn serialize_members(
            &self,
            serializer: &mut dyn ShapeSerializer,
        ) -> Result<(), SerdeError> {
            serializer.write_string(&NAME_MEMBER, self.name)
        }
    }

    fn serialize<F>(write: F) -> String
    where
        F: FnOnce(&mut XmlSerializer) -> Result<(), SerdeError>,
    {
        let mut ser = XmlSerializer::new(Arc::new(XmlCodecSettings::default()));
        write(&mut ser).expect("serialization failed");
        String::from_utf8(<XmlSerializer as FinishSerializer>::finish(ser)).unwrap()
    }

    #[test]
    fn struct_with_string_member() {
        let p = Person { name: "Iago" };
        let out = serialize(|ser| ser.write_struct(&PERSON_SCHEMA, &p));
        assert_eq!(out, "<Person><name>Iago</name></Person>");
    }

    #[test]
    fn struct_with_no_members_self_closes() {
        struct Empty;
        impl SerializableStruct for Empty {
            fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                Ok(())
            }
        }
        static EMPTY_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "Empty"), ShapeType::Structure, &[]);

        let out = serialize(|ser| ser.write_struct(&EMPTY_SCHEMA, &Empty));
        assert_eq!(out, "<Empty></Empty>");
    }

    #[test]
    fn struct_string_value_is_escaped() {
        let p = Person { name: "<a&b>" };
        let out = serialize(|ser| ser.write_struct(&PERSON_SCHEMA, &p));
        assert_eq!(out, "<Person><name>&lt;a&amp;b&gt;</name></Person>");
    }

    #[test]
    fn struct_string_value_eol_is_encoded() {
        // Per the XML EOL Encoding SEP, \r and \n must be escaped as
        // numeric character references to survive XML EOL normalization.
        let p = Person { name: "a\r\nb" };
        let out = serialize(|ser| ser.write_struct(&PERSON_SCHEMA, &p));
        assert_eq!(out, "<Person><name>a&#xD;&#xA;b</name></Person>");
    }

    #[test]
    fn nested_structs_close_correctly() {
        // Schemas: Outer { inner: Inner { name: String } }.
        // Note: the inner-struct schema is not exercised directly because
        // member dispatch in this codec uses the *member* schema, not the
        // target shape's schema. The Inner type's `SerializableStruct` impl
        // is what drives inner serialization.
        static INNER_NAME: Schema = Schema::new_member(
            shape_id!("test", "Inner$name"),
            ShapeType::String,
            "name",
            0,
        );
        static OUTER_INNER: Schema = Schema::new_member(
            shape_id!("test", "Outer$inner"),
            ShapeType::Structure,
            "inner",
            0,
        );
        static OUTER_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "Outer"),
            ShapeType::Structure,
            &[&OUTER_INNER],
        );

        struct Inner<'a> {
            name: &'a str,
        }
        impl SerializableStruct for Inner<'_> {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&INNER_NAME, self.name)
            }
        }
        struct Outer<'a> {
            inner: Inner<'a>,
        }
        impl SerializableStruct for Outer<'_> {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                // For nested structs, dispatch on the *member* schema so that
                // the resolved element name is the field's name (or its
                // @xmlName), not the target shape's name.
                ser.write_struct(&OUTER_INNER, &self.inner)
            }
        }

        let o = Outer {
            inner: Inner { name: "v" },
        };
        let out = serialize(|ser| ser.write_struct(&OUTER_SCHEMA, &o));
        assert_eq!(out, "<Outer><inner><name>v</name></inner></Outer>");
    }

    #[test]
    fn xml_name_overrides_member_name() {
        static RENAMED_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "Person$name"),
            ShapeType::String,
            "name",
            0,
        )
        .with_xml_name("FullName");
        static PERSON_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "Person"),
            ShapeType::Structure,
            &[&RENAMED_MEMBER],
        );

        struct P;
        impl SerializableStruct for P {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&RENAMED_MEMBER, "v")
            }
        }

        let out = serialize(|ser| ser.write_struct(&PERSON_SCHEMA, &P));
        assert_eq!(out, "<Person><FullName>v</FullName></Person>");
    }

    #[test]
    fn original_name_overrides_id_for_synthetic_root() {
        // Synthetic shapes have id name "OperationInput" but original_name is
        // "OperationRequest" (the user-authored name). The codec should use
        // the original name for the root element when there's no @xmlName.
        static SYNTHETIC: Schema = Schema::new_struct(
            shape_id!("test.synthetic", "FooInput"),
            ShapeType::Structure,
            &[],
        )
        .with_original_name("FooRequest");

        struct Empty;
        impl SerializableStruct for Empty {
            fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                Ok(())
            }
        }

        let out = serialize(|ser| ser.write_struct(&SYNTHETIC, &Empty));
        assert_eq!(out, "<FooRequest></FooRequest>");
    }

    // Scalar member writes (boolean, ints, floats, blob, timestamp).

    static SCALAR_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "S$v"), ShapeType::Integer, "v", 0);

    #[test]
    fn write_boolean_true() {
        let out = serialize(|ser| ser.write_boolean(&SCALAR_MEMBER, true));
        assert_eq!(out, "<v>true</v>");
    }

    #[test]
    fn write_boolean_false() {
        let out = serialize(|ser| ser.write_boolean(&SCALAR_MEMBER, false));
        assert_eq!(out, "<v>false</v>");
    }

    #[test]
    fn write_integer_negative() {
        let out = serialize(|ser| ser.write_integer(&SCALAR_MEMBER, -42));
        assert_eq!(out, "<v>-42</v>");
    }

    #[test]
    fn write_long_large() {
        let out = serialize(|ser| ser.write_long(&SCALAR_MEMBER, i64::MAX));
        assert_eq!(out, format!("<v>{}</v>", i64::MAX));
    }

    #[test]
    fn write_float_special_values() {
        let out = serialize(|ser| ser.write_float(&SCALAR_MEMBER, f32::NAN));
        assert_eq!(out, "<v>NaN</v>");
        let out = serialize(|ser| ser.write_float(&SCALAR_MEMBER, f32::INFINITY));
        assert_eq!(out, "<v>Infinity</v>");
        let out = serialize(|ser| ser.write_float(&SCALAR_MEMBER, f32::NEG_INFINITY));
        assert_eq!(out, "<v>-Infinity</v>");
    }

    #[test]
    fn write_double_normal() {
        let out = serialize(|ser| ser.write_double(&SCALAR_MEMBER, 1.5));
        assert_eq!(out, "<v>1.5</v>");
    }

    #[test]
    fn write_blob_base64() {
        let blob = Blob::new(b"hello");
        let out = serialize(|ser| ser.write_blob(&SCALAR_MEMBER, blob.as_ref()));
        assert_eq!(out, "<v>aGVsbG8=</v>");
    }

    #[test]
    fn write_timestamp_default_datetime() {
        // Default format for REST XML is date-time (ISO 8601).
        let ts = DateTime::from_secs(1515531081);
        let out = serialize(|ser| ser.write_timestamp(&SCALAR_MEMBER, &ts));
        assert_eq!(out, "<v>2018-01-09T20:51:21Z</v>");
    }

    #[test]
    fn write_timestamp_epoch_seconds_override() {
        use aws_smithy_schema::traits::TimestampFormat as SchemaTimestampFormat;
        static TS_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$t"), ShapeType::Timestamp, "t", 0)
                .with_timestamp_format(SchemaTimestampFormat::EpochSeconds);
        let ts = DateTime::from_secs(1515531081);
        let out = serialize(|ser| ser.write_timestamp(&TS_MEMBER, &ts));
        assert_eq!(out, "<t>1515531081</t>");
    }

    #[test]
    fn write_null_emits_nothing() {
        let out = serialize(|ser| ser.write_null(&SCALAR_MEMBER));
        assert_eq!(out, "");
    }

    // `@xmlAttribute` emission and ordering relative to child elements.

    #[test]
    fn attribute_string_on_struct() {
        static ATTR_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "X$id"), ShapeType::String, "id", 0)
                .with_xml_attribute();
        static CHILD_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "X$name"), ShapeType::String, "name", 1);
        static X_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "X"),
            ShapeType::Structure,
            &[&ATTR_MEMBER, &CHILD_MEMBER],
        );

        struct X;
        impl SerializableStruct for X {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&ATTR_MEMBER, "42")?;
                ser.write_string(&CHILD_MEMBER, "hello")
            }
        }

        let out = serialize(|ser| ser.write_struct(&X_SCHEMA, &X));
        assert_eq!(out, "<X id=\"42\"><name>hello</name></X>");
    }

    #[test]
    fn attribute_integer_on_struct() {
        static ATTR: Schema =
            Schema::new_member(shape_id!("test", "X$count"), ShapeType::Integer, "count", 0)
                .with_xml_attribute();
        static X_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "X"), ShapeType::Structure, &[&ATTR]);

        struct X;
        impl SerializableStruct for X {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_integer(&ATTR, 7)
            }
        }

        let out = serialize(|ser| ser.write_struct(&X_SCHEMA, &X));
        assert_eq!(out, "<X count=\"7\"></X>");
    }

    #[test]
    fn attribute_value_is_escaped() {
        static ATTR: Schema =
            Schema::new_member(shape_id!("test", "X$v"), ShapeType::String, "v", 0)
                .with_xml_attribute();
        static X_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "X"), ShapeType::Structure, &[&ATTR]);

        struct X;
        impl SerializableStruct for X {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&ATTR, "a\"b&c")
            }
        }

        let out = serialize(|ser| ser.write_struct(&X_SCHEMA, &X));
        assert_eq!(out, "<X v=\"a&quot;b&amp;c\"></X>");
    }

    #[test]
    fn attribute_on_struct_returns_error() {
        static ATTR_STRUCT: Schema = Schema::new_member(
            shape_id!("test", "X$inner"),
            ShapeType::Structure,
            "inner",
            0,
        )
        .with_xml_attribute();

        struct Empty;
        impl SerializableStruct for Empty {
            fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                Ok(())
            }
        }

        let mut ser = XmlSerializer::new(Arc::new(XmlCodecSettings::default()));
        let result = ser.write_struct(&ATTR_STRUCT, &Empty);
        assert!(result.is_err());
    }

    // `@xmlNamespace` emission on root and child elements.

    #[test]
    fn namespace_on_struct() {
        static NS_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "X"), ShapeType::Structure, &[])
                .with_xml_namespace("https://example.com", None);

        struct Empty;
        impl SerializableStruct for Empty {
            fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                Ok(())
            }
        }

        let out = serialize(|ser| ser.write_struct(&NS_SCHEMA, &Empty));
        assert_eq!(out, "<X xmlns=\"https://example.com\"></X>");
    }

    #[test]
    fn namespace_with_prefix() {
        static NS_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "X"), ShapeType::Structure, &[])
                .with_xml_namespace("https://example.com", Some("ex"));

        struct Empty;
        impl SerializableStruct for Empty {
            fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                Ok(())
            }
        }

        let out = serialize(|ser| ser.write_struct(&NS_SCHEMA, &Empty));
        assert_eq!(out, "<X xmlns:ex=\"https://example.com\"></X>");
    }

    #[test]
    fn namespace_with_children() {
        static CHILD: Schema =
            Schema::new_member(shape_id!("test", "X$v"), ShapeType::String, "v", 0);
        static NS_SCHEMA: Schema =
            Schema::new_struct(shape_id!("test", "X"), ShapeType::Structure, &[&CHILD])
                .with_xml_namespace("urn:foo", None);

        struct X;
        impl SerializableStruct for X {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&CHILD, "hi")
            }
        }

        let out = serialize(|ser| ser.write_struct(&NS_SCHEMA, &X));
        assert_eq!(out, "<X xmlns=\"urn:foo\"><v>hi</v></X>");
    }

    // List serialization: wrapped, flattened, and `@xmlName` overrides.

    #[test]
    fn list_wrapped() {
        // Schema: struct S { items: List<String> }
        // List member schema defaults to name "member".
        static LIST_ITEM: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 0);

        let out = serialize(|ser| {
            ser.write_list(&LIST_MEMBER, &|ser| {
                ser.write_string(&LIST_ITEM, "a")?;
                ser.write_string(&LIST_ITEM, "b")
            })
        });
        assert_eq!(out, "<items><member>a</member><member>b</member></items>");
    }

    #[test]
    fn list_wrapped_with_xml_name_on_item() {
        // @xmlName("Item") on the list's member schema.
        static LIST_ITEM: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("Item");
        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 0);

        let out = serialize(|ser| {
            ser.write_list(&LIST_MEMBER, &|ser| {
                ser.write_string(&LIST_ITEM, "a")?;
                ser.write_string(&LIST_ITEM, "b")
            })
        });
        assert_eq!(out, "<items><Item>a</Item><Item>b</Item></items>");
    }

    #[test]
    fn list_flattened() {
        // @xmlFlattened on the struct member. Items use the member schema
        // passed to write_string (which carries the item element name).
        static LIST_ITEM: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("item");
        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 0)
                .with_xml_flattened();

        let out = serialize(|ser| {
            ser.write_list(&LIST_MEMBER, &|ser| {
                ser.write_string(&LIST_ITEM, "a")?;
                ser.write_string(&LIST_ITEM, "b")
            })
        });
        // Flattened: no wrapper, items emitted directly.
        assert_eq!(out, "<item>a</item><item>b</item>");
    }

    // Map serialization: wrapped, flattened, and `@xmlName` on key/value.

    #[test]
    fn map_wrapped() {
        static KEY_SCHEMA: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static VALUE_SCHEMA: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static MAP_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$myMap"), ShapeType::Map, "myMap", 0)
                .with_map_members(&KEY_SCHEMA, &VALUE_SCHEMA);

        let out = serialize(|ser| {
            ser.write_map(&MAP_MEMBER, &|ser| {
                ser.write_string(&prelude::STRING, "k1")?;
                ser.write_string(&prelude::STRING, "v1")?;
                ser.write_string(&prelude::STRING, "k2")?;
                ser.write_string(&prelude::STRING, "v2")
            })
        });
        assert_eq!(
            out,
            "<myMap><entry><key>k1</key><value>v1</value></entry><entry><key>k2</key><value>v2</value></entry></myMap>"
        );
    }

    #[test]
    fn map_wrapped_with_renamed_key_value() {
        // @xmlName on key/value schemas.
        static REN_KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0)
                .with_xml_name("Attribute");
        static REN_VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1)
                .with_xml_name("Setting");

        static MAP_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$m"), ShapeType::Map, "m", 0)
                .with_map_members(&REN_KEY, &REN_VALUE);

        let out = serialize(|ser| {
            ser.write_map(&MAP_MEMBER, &|ser| {
                ser.write_string(&prelude::STRING, "k")?;
                ser.write_string(&prelude::STRING, "v")
            })
        });
        assert_eq!(
            out,
            "<m><entry><Attribute>k</Attribute><Setting>v</Setting></entry></m>"
        );
    }

    #[test]
    fn map_flattened() {
        static KEY_SCHEMA: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static VALUE_SCHEMA: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static MAP_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "S$m"), ShapeType::Map, "m", 0)
                .with_xml_flattened()
                .with_map_members(&KEY_SCHEMA, &VALUE_SCHEMA);

        let out = serialize(|ser| {
            ser.write_map(&MAP_MEMBER, &|ser| {
                ser.write_string(&prelude::STRING, "a")?;
                ser.write_string(&prelude::STRING, "1")?;
                ser.write_string(&prelude::STRING, "b")?;
                ser.write_string(&prelude::STRING, "2")
            })
        });
        // Flattened: no wrapper, entries use member name as entry element.
        assert_eq!(
            out,
            "<m><key>a</key><value>1</value></m><m><key>b</key><value>2</value></m>"
        );
    }

    #[test]
    fn write_document_returns_error() {
        let mut ser = XmlSerializer::new(Arc::new(XmlCodecSettings::default()));
        let result = ser.write_document(&SCALAR_MEMBER, &Document::Object(Default::default()));
        assert_eq!(
            result.unwrap_err().to_string(),
            "document types are not supported by REST XML"
        );
    }

    #[test]
    fn map_with_struct_value() {
        static HI: Schema =
            Schema::new_member(shape_id!("test", "G$hi"), ShapeType::String, "hi", 0);
        static GREETING: Schema = Schema::new_struct(
            shape_id!("test", "GreetingStruct"),
            ShapeType::Structure,
            &[&HI],
        );
        static KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static MAP: Schema =
            Schema::new_member(shape_id!("test", "S$myMap"), ShapeType::Map, "myMap", 0)
                .with_map_members(&KEY, &VALUE);

        struct G;
        impl SerializableStruct for G {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&HI, "bye")
            }
        }

        let out = serialize(|ser| {
            ser.write_map(&MAP, &|ser| {
                ser.write_string(&prelude::STRING, "baz")?;
                ser.write_struct(&GREETING, &G)
            })
        });
        assert_eq!(
            out,
            "<myMap><entry><key>baz</key><value><hi>bye</hi></value></entry></myMap>"
        );
    }

    /// Regression test for a `has_attrs` short-circuit bug:
    /// `write_struct(member_schema, value)` where `member_schema` was
    /// produced by `Schema::new_member` and the target struct has
    /// `@xmlAttribute` members.
    ///
    /// Pre-fix: `has_attrs = schema.members().iter().any(|m| m.xml_attribute())`
    /// looked at `member_schema.members()`, which is empty for member
    /// schemas regardless of the target's actual member list. The
    /// AttributesOnly pass was therefore skipped and the attribute was
    /// dropped.
    ///
    /// Post-fix: when `schema.member_name().is_some()` (i.e. the schema
    /// was built via `Schema::new_member`), the codec runs the
    /// AttributesOnly pass conservatively. The per-member
    /// `filter_allows` check then correctly emits the attribute.
    ///
    /// Surfaced by the Smithy `XmlNamespaceSimpleScalarProperties`
    /// protocol test where `<Nested xsi:someName="...">` was missing
    /// the `someName` attribute when emitted via a member schema.
    #[test]
    fn member_schema_struct_emits_attribute_member() {
        // Inner struct with an attribute member.
        static ATTR_FIELD_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "Inner$attrField"),
            ShapeType::String,
            "attrField",
            0,
        )
        .with_xml_name("xsi:someName")
        .with_xml_attribute();
        static INNER_TARGET: Schema = Schema::new_struct(
            shape_id!("test", "Inner"),
            ShapeType::Structure,
            &[&ATTR_FIELD_MEMBER],
        );
        // Outer struct with a member pointing at `Inner`. Crucially this
        // is a member schema (Schema::new_member), not the target
        // struct's schema.
        static INNER_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "Outer$nested"),
            ShapeType::Structure,
            "nested",
            0,
        );
        static OUTER_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "Outer"),
            ShapeType::Structure,
            &[&INNER_MEMBER],
        );

        struct Inner<'a> {
            attr: &'a str,
        }
        impl SerializableStruct for Inner<'_> {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                ser.write_string(&ATTR_FIELD_MEMBER, self.attr)
            }
        }
        struct Outer<'a> {
            inner: Inner<'a>,
        }
        impl SerializableStruct for Outer<'_> {
            fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
                // Pass the *member* schema (matches codegen output), not
                // the target struct's `INNER_TARGET`. This is the
                // configuration that triggered the original bug.
                let _ = &INNER_TARGET;
                ser.write_struct(&INNER_MEMBER, &self.inner)
            }
        }

        let outer = Outer {
            inner: Inner { attr: "v" },
        };
        let out = serialize(|ser| ser.write_struct(&OUTER_SCHEMA, &outer));
        assert_eq!(out, r#"<Outer><nested xsi:someName="v"></nested></Outer>"#);
    }
}
