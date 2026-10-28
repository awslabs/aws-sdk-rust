/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! XML deserializer implementing the `ShapeDeserializer` trait.

use super::XmlCodecSettings;
use crate::decode::{self, Document};
use aws_smithy_schema::serde::{SerdeError, ShapeDeserializer};
use aws_smithy_schema::Schema;
use aws_smithy_types::date_time::Format as TimestampFormat;
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime};

use aws_smithy_types::Document as SmithyDocument;
use std::borrow::Cow;
use std::sync::Arc;

/// Maximum recursion depth for deserialization. Payloads nested deeper than
/// this will produce a [`SerdeError`] instead of risking a stack overflow.
/// Matches the default used by the JSON and CBOR codecs.
pub(crate) const MAX_DESERIALIZE_DEPTH: u32 = 128;

/// XML deserializer that implements the `ShapeDeserializer` trait.
///
/// Wraps the existing `aws_smithy_xml::decode` SAX-like API and provides
/// schema-driven dispatch for struct members, lists, and maps.
///
/// The deserializer holds the input as `&'a [u8]` throughout. For aggregate
/// reads (`read_struct`, `read_list`, `read_map`) we construct a fresh
/// `Document` over `input` on demand. For scalar reads we either parse
/// `input` to extract the root element's text, or — when a parent
/// aggregate has already extracted the leaf text from its child element —
/// take the pre-extracted text directly via `text`.
///
/// Sibling dispatch within an aggregate read avoids per-iteration
/// `XmlDeserializer` construction by reusing `&mut self` through
/// `dispatch_subslice` / `dispatch_text` helpers, which save and restore
/// the relevant state across the closure call.
pub struct XmlDeserializer<'a> {
    /// XML bytes for this deserializer. For the document root and for
    /// aggregate sub-deserializers this is the slice of the parent input
    /// covering the relevant element. Ignored when `text` is set.
    input: &'a [u8],
    /// Pre-extracted leaf text, including any extraction error. Scalar reads
    /// consume this result; a consumer that skips the value also skips errors
    /// about its leaf shape. Aggregate reads error when this is present.
    text: Option<Result<Cow<'a, str>, SerdeError>>,
    settings: Arc<XmlCodecSettings>,
    /// Aggregate nesting depth. Incremented at the top of each
    /// `read_struct` / `read_list` / `read_map` and decremented before
    /// they return so sibling reads on the same deserializer don't
    /// accumulate. Compared against [`XmlCodecSettings::max_depth`] to
    /// reject deeply-nested payloads before they exhaust the stack.
    depth: u32,
}

impl<'a> XmlDeserializer<'a> {
    /// Creates a new XML deserializer over raw bytes.
    pub(crate) fn new(input: &'a [u8], settings: Arc<XmlCodecSettings>) -> Self {
        Self {
            input,
            text: None,
            settings,
            depth: 0,
        }
    }

    /// Spawn a deserializer for owned sub-content that cannot be reached via
    /// [`dispatch_subslice`](Self::dispatch_subslice) — e.g. the synthesized
    /// `<__flat>` wrapper around a flattened-aggregate group, whose buffer is
    /// owned locally and so has a shorter lifetime than `'a`.
    ///
    /// Inherits the parent's [`depth`](Self::depth) so the recursion-depth
    /// guard stays continuous across the boundary, exactly as it does for the
    /// non-flattened path (which reuses `self` via `dispatch_subslice`).
    /// Using [`new`](Self::new) here instead would reset `depth` to 0 and let
    /// a shape that recurses through a flattened member (e.g.
    /// `structure Node { @xmlFlattened kids: NodeList }`) nest without bound,
    /// overflowing the stack.
    fn new_child<'b>(&self, input: &'b [u8]) -> XmlDeserializer<'b> {
        XmlDeserializer {
            input,
            text: None,
            settings: self.settings.clone(),
            depth: self.depth,
        }
    }

    /// Creates a deserializer pre-loaded with leaf text content. Used by
    /// tests; runtime dispatch uses [`dispatch_text`](Self::dispatch_text)
    /// to repoint an existing deserializer at leaf text rather than
    /// constructing a new instance.
    #[cfg(test)]
    fn from_text(text: Cow<'a, str>, settings: Arc<XmlCodecSettings>) -> Self {
        Self {
            input: b"",
            text: Some(Ok(text)),
            settings,
            depth: 0,
        }
    }

    /// Increment the recursion-depth counter and return an error if the
    /// configured maximum would be exceeded. Caller is responsible for
    /// decrementing on the way out (see [`Self::leave_aggregate`]).
    ///
    /// Order matters: the depth bound is checked *before* the increment so
    /// the error path doesn't leave the counter incremented. Together with
    /// the IIFE pattern around each aggregate body (which guarantees
    /// `leave_aggregate` always runs after a successful `enter_aggregate`),
    /// this keeps `depth` consistent across `?` propagation.
    fn enter_aggregate(&mut self) -> Result<(), SerdeError> {
        if self.depth >= self.settings.max_depth() {
            return Err(SerdeError::custom("maximum nesting depth exceeded"));
        }
        self.depth += 1;
        Ok(())
    }

    /// Decrement the recursion-depth counter. Pair with each successful
    /// [`Self::enter_aggregate`] call.
    fn leave_aggregate(&mut self) {
        debug_assert!(self.depth > 0, "leave_aggregate without enter_aggregate");
        self.depth = self.depth.saturating_sub(1);
    }

    /// Construct a fresh `Document` over `self.input`. Errors if the
    /// deserializer holds pre-extracted text (an aggregate read was
    /// expected on this deserializer).
    fn document(&self) -> Result<Document<'a>, SerdeError> {
        if self.text.is_some() {
            return Err(SerdeError::custom("expected XML element, found text"));
        }
        let mut doc = Document::try_from(self.input).unwrap_or_else(|_| Document::new(""));
        if self.settings.validate_document && self.depth == 1 {
            doc.check_well_formedness();
        }
        Ok(doc)
    }

    /// Extract the leaf text content. If `text` was pre-set, returns it
    /// directly; otherwise parses `input`, navigates to the root element,
    /// and reads its text content.
    fn take_text(&mut self) -> Result<Cow<'a, str>, SerdeError> {
        if let Some(t) = self.text.take() {
            return t;
        }
        let mut doc = Document::try_from(self.input).unwrap_or_else(|_| Document::new(""));
        let top_level_strict = self.settings.validate_document && self.depth == 0;
        if top_level_strict {
            doc.check_well_formedness();
        }
        let mut root = doc
            .root_element()
            .map_err(|e| SerdeError::custom(e.to_string()))?;
        let text = decode::try_data(&mut root).map_err(|e| SerdeError::custom(e.to_string()))?;
        drop(root);
        if top_level_strict {
            Self::finish_well_formed(&mut doc)?;
        }
        Ok(text)
    }

    /// Run `f` against `self` after temporarily repointing it at a sub-slice
    /// of the parent input. State (input, text) is saved on entry and
    /// restored on return so the deserializer can be reused for sibling
    /// dispatches without per-iteration allocation.
    fn dispatch_subslice<R>(&mut self, sub: &'a [u8], f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_input = std::mem::replace(&mut self.input, sub);
        let saved_text = self.text.take();
        let r = f(self);
        self.input = saved_input;
        self.text = saved_text;
        r
    }

    /// Run `f` against `self` after temporarily setting a leaf-text result.
    /// Extraction errors surface only when the consumer reads the value.
    /// State is restored on return.
    fn dispatch_text<R>(
        &mut self,
        text: Result<Cow<'a, str>, SerdeError>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let saved_input = std::mem::replace(&mut self.input, b"");
        let saved_text = self.text.replace(text);
        let r = f(self);
        self.input = saved_input;
        self.text = saved_text;
        r
    }

    /// Resolve a child element name to a member schema by matching against
    /// @xmlName (if present) or member_name. Returns the first matching member, so
    /// a name shared by several members always resolves to the same one.
    fn resolve_member<'s>(schema: &'s Schema<'s>, element_name: &str) -> Option<&'s Schema<'s>> {
        schema.members().iter().copied().find(|m| {
            if let Some(xml_name) = m.xml_name() {
                xml_name.value() == element_name
            } else {
                m.member_name() == Some(element_name)
            }
        })
    }

    /// The element name the items of a wrapped list must carry when the codec
    /// checks collection element names (`check_names`, the
    /// `strict_collection_element_names` setting): `@xmlName` on the list's
    /// member, else the member's name (`member`).
    ///
    /// `None` means every child element is an item. That is the case when the
    /// setting is off; for a flattened list, whose siblings `read_struct` has
    /// already selected by name before handing them over; and for a schema
    /// that does not describe the list's member, such as the
    /// `prelude::DOCUMENT` placeholder codegen falls back to when it has no
    /// sub-schema for a nested aggregate, where the name is not known.
    fn list_item_name<'s>(check_names: bool, schema: &'s Schema<'_>) -> Option<&'s str> {
        if !check_names || schema.xml_flattened() {
            return None;
        }
        let member = schema.member()?;
        Some(
            member
                .xml_name()
                .map(|t| t.value())
                .or(member.member_name())
                .unwrap_or("member"),
        )
    }

    /// Whether `el` is an item of a list whose items are named `item_name`
    /// (see [`list_item_name`](Self::list_item_name)). Other elements are
    /// skipped, as unknown structure members are.
    fn is_list_item(item_name: Option<&str>, el: &decode::StartEl<'_>) -> bool {
        item_name.is_none_or(|name| el.matches(name))
    }

    /// Whether `el` is an entry of the map `schema` describes. With
    /// `check_names` (the `strict_collection_element_names` setting) only the
    /// `entry` children of a wrapped map are; without it every child is. The
    /// siblings of a flattened map were selected by name in `read_struct`, so
    /// each of them is an entry.
    fn is_map_entry(check_names: bool, schema: &Schema<'_>, el: &decode::StartEl<'_>) -> bool {
        !check_names || schema.xml_flattened() || el.matches("entry")
    }

    /// Byte offset of the `<` that opens the element whose local name
    /// `el_local` borrows from `input` (xmlparser hands out names as borrows
    /// into the document). Only the element's own `prefix:` sits between that
    /// `<` and the name, so a short backwards search finds it.
    ///
    /// The matching end offset is *not* computed here: callers take it from
    /// [`ScopedDecoder::end_offset`](decode::ScopedDecoder::end_offset), which
    /// runs the tokenizer through the matching close tag. Taking both ends from the
    /// tokenizer means comments, CDATA, processing instructions and `>` in
    /// attribute values are handled exactly as the parser sees them, and each
    /// input byte belongs to at most one sibling slice. An earlier byte
    /// scanner disagreed with the tokenizer on such input, fell back to "rest
    /// of document" for every sibling, and made flattened lists quadratic in
    /// memory and CPU.
    fn element_start(input: &[u8], el_local: &str) -> usize {
        let input_start = input.as_ptr() as usize;
        let name_ptr = el_local.as_ptr() as usize;
        debug_assert!(
            name_ptr >= input_start && name_ptr + el_local.len() <= input_start + input.len(),
            "element_start: el_local must point into input"
        );
        let name_offset = name_ptr.saturating_sub(input_start).min(input.len());
        input[..name_offset]
            .iter()
            .rposition(|&b| b == b'<')
            .unwrap_or(0)
    }

    /// Surface any well-formedness violation recorded by a document in
    /// checking mode (see [`Document::check_well_formedness`]).
    fn finish_well_formed(doc: &mut Document<'_>) -> Result<(), SerdeError> {
        doc.finish_well_formed()
            .map_err(|e| SerdeError::invalid_input(format!("ill-formed XML: {e}")))
    }

    /// Only the outer aggregate owns the document boundary. Its tokenizer also
    /// drains skipped children, so nested reads do not need separate validation.
    fn finish_document(&self, doc: &mut Document<'_>) -> Result<(), SerdeError> {
        if self.settings.validate_document && self.depth == 1 {
            Self::finish_well_formed(doc)?;
        }
        Ok(())
    }

    fn resolve_timestamp_format(&self, schema: &Schema<'_>) -> Result<TimestampFormat, SerdeError> {
        let Some(t) = schema.timestamp_format() else {
            return Ok(match self.settings.default_timestamp_format() {
                TimestampFormat::DateTime => TimestampFormat::DateTimeWithOffset,
                other => other,
            });
        };
        Ok(match t.format() {
            aws_smithy_schema::traits::TimestampFormat::EpochSeconds => {
                TimestampFormat::EpochSeconds
            }
            // Use the lenient `DateTimeWithOffset` so timezone-suffixed
            // RFC-3339 strings (e.g. `2019-12-17T00:48:18+01:00`) parse —
            // matches the Smithy `date-time` protocol-test expectations
            // and the JSON codec's behavior.
            aws_smithy_schema::traits::TimestampFormat::DateTime => {
                TimestampFormat::DateTimeWithOffset
            }
            aws_smithy_schema::traits::TimestampFormat::HttpDate => TimestampFormat::HttpDate,
            other => {
                return Err(SerdeError::unsupported(format!(
                    "unsupported timestamp format {other:?}"
                )))
            }
        })
    }
}

/// Locate a depth-2 XML element (a direct child of the document root) whose
/// local name satisfies `predicate`, returning the byte slice covering it
/// (`<El>...</El>`, inclusive of tags). The document root itself is also
/// considered, so an "unwrapped" envelope whose root already matches is found.
///
/// Returns `None` if the body is not valid UTF-8, is not parseable as XML, or
/// contains no matching element — callers decide how to fall back.
///
/// Robust to start-tag attributes (e.g. `<Error xmlns="...">`), nested
/// same-name elements, comments, and CDATA, which a naive substring search
/// would mishandle. Both the AWS REST XML error path (`name == "Error"`) and
/// the awsQuery response path (`name.ends_with("Result") || name == "Error"`)
/// build on this.
pub fn find_depth2_element_slice_by(
    body: &[u8],
    predicate: impl Fn(&str) -> bool,
) -> Option<&[u8]> {
    let mut doc = Document::try_from(body).ok()?;
    let mut root = doc.root_element().ok()?;
    // Unwrapped envelope: the root element itself matches. Its start/end tags
    // are already at the body boundaries, so return the whole body.
    if predicate(root.start_el().local()) {
        return Some(body);
    }
    // Wrapped envelope: scan the root's direct children for a match.
    while let Some(tag) = root.next_tag() {
        let local = tag.start_el().local();
        if predicate(local) {
            // `local` is a `&str` borrowed from `body`, satisfying the
            // pointer-containment invariant of `element_start`.
            let start = XmlDeserializer::element_start(body, local);
            return Some(&body[start..tag.end_offset()]);
        }
    }
    None
}

impl ShapeDeserializer for XmlDeserializer<'_> {
    fn read_struct(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.enter_aggregate()?;
        // IIFE: any `?` (or early `return`) inside falls through to
        // `leave_aggregate` below — preserving the depth counter on the
        // error path. `return` inside a Rust closure returns from the
        // closure, not the enclosing function, so the unwrapped-output
        // early-return below correctly produces `Ok(())` for the IIFE.
        let result = (|| -> Result<(), SerdeError> {
            // Build a Document over `self.input` locally. Doing it here (rather
            // than as part of `XmlDeserializer` state) keeps the iteration
            // borrow scoped to this stack frame, which lets us mutate `self`
            // (via `dispatch_*`) for child-consumer dispatches without fighting
            // a long-lived borrow on `self.state`.
            let input = self.input;
            // The top-level struct of a strict (server) read owns the whole
            // body: it rejects ill-formed XML rather than recovering from it,
            // so nothing in front of the service can read the bytes
            // differently from the handler. Every byte of the body passes
            // through this `doc`'s tokenizer (child elements are drained
            // through it), so checking here covers nested content too.
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;

            if self.settings.validate_root_name && self.depth == 1 {
                let expected = schema
                    .xml_name()
                    .map(|t| t.value())
                    .or_else(|| schema.original_name())
                    .or_else(|| schema.member_name())
                    .unwrap_or_else(|| schema.shape_id().shape_name());
                if !root.start_el().matches(expected) {
                    return Err(SerdeError::invalid_input(format!(
                        "expected XML root {expected}"
                    )));
                }
            }

            // Unwrapped XML output (e.g. S3 `GetBucketLocation` whose body is
            // `<LocationConstraint>...</LocationConstraint>` rather than
            // `<GetBucketLocationOutput><LocationConstraint>...`). The body's
            // root element IS the (sole) member element — dispatch it directly
            // and skip the normal "enter wrapper, iterate children" path. The
            // schema flag is set by codegen for operations with the
            // `S3UnwrappedXmlOutputTrait` AWS customization; non-XML codecs
            // ignore the field, preserving runtime protocol-swap compatibility.
            if schema.xml_unwrapped_output() {
                // Capture the element's start offset BEFORE consuming `root`:
                // `element_start`'s pointer-arithmetic invariant requires
                // `el_local` to be a sub-slice of `input`, which
                // `root.start_el().local()` is. A previous version passed an
                // owned `String` here, silently producing offset=0; correct
                // only by happy accident when the input buffer started with
                // the target element.
                let el_local = root.start_el().local();
                let start = Self::element_start(input, el_local);
                let local = el_local.to_owned();
                // Consuming `root` advances the tokenizer past the close tag,
                // which gives the end offset and releases the iterator borrow
                // on `doc` so we can mutate `self`.
                let sub = &input[start..root.end_offset()];
                self.finish_document(&mut doc)?;
                if let Some(member) = Self::resolve_member(schema, &local) {
                    self.dispatch_subslice(sub, |this| consumer(member, this))?;
                }
                return Ok(());
            }

            // Dispatch @xmlAttribute members from the start element's attributes.
            for member in schema.members() {
                if member.xml_attribute() {
                    let attr_name = member
                        .xml_name()
                        .map(|t| t.value())
                        .or(member.member_name())
                        .unwrap_or("");
                    if let Some(value) = root.start_el().attr(attr_name) {
                        let text = Cow::Owned(value.to_owned());
                        self.dispatch_text(Ok(text), |this| consumer(member, this))?;
                    }
                }
            }

            // Track flattened-aggregate members: their wire format is repeated sibling
            // elements that must be accumulated and dispatched as a single read_list /
            // read_map call. Map: member_index -> (member_schema, accumulated XML bytes).
            let mut flattened_groups: std::collections::HashMap<usize, (&Schema<'_>, Vec<u8>)> =
                std::collections::HashMap::new();

            // Dispatch child elements.
            while let Some(mut child_scope) = root.next_tag() {
                let Some(member) = Self::resolve_member(schema, child_scope.start_el().local())
                else {
                    continue;
                };
                // For non-flattened aggregate members, the child element IS the
                // aggregate container (e.g. `<myList><member>...</member></myList>`).
                // Use a sub-slice into `input` so the consumer's read_list / read_map
                // / read_struct can build its own Document over the child element.
                // For flattened aggregate members, accumulate the bytes of each
                // matching sibling and dispatch them together below.
                // For scalars (including flattened scalars), extract text inline.
                let is_aggregate = member.shape_type().is_aggregate();
                if is_aggregate && !member.xml_flattened() {
                    let start = Self::element_start(input, child_scope.start_el().local());
                    let sub = &input[start..child_scope.end_offset()];
                    self.dispatch_subslice(sub, |this| consumer(member, this))?;
                } else if is_aggregate {
                    // Flattened aggregate: capture this sibling's slice; dispatch
                    // the merged group below.
                    let start = Self::element_start(input, child_scope.start_el().local());
                    let sub = &input[start..child_scope.end_offset()];
                    let idx = member.member_index().unwrap_or(usize::MAX);
                    let entry = flattened_groups
                        .entry(idx)
                        .or_insert_with(|| (member, Vec::new()));
                    entry.1.extend_from_slice(sub);
                } else {
                    // Keep extraction errors with the value: a transport adapter may
                    // discard this member without reading its modeled scalar type.
                    let text = decode::try_data(&mut child_scope)
                        .map_err(|e| SerdeError::custom(e.to_string()));
                    drop(child_scope);
                    self.dispatch_text(text, |this| consumer(member, this))?;
                }
            }
            drop(root);
            self.finish_document(&mut doc)?;

            // Dispatch each accumulated flattened-aggregate group as a single call.
            // We synthesize a `<__flat>...</__flat>` wrapper so the consumer's
            // `read_list` / `read_map` sees the collected siblings as
            // wrapper-children and iterates them normally. The wrapper buffer is
            // owned locally (lifetime is shorter than `'a`), so we cannot route
            // it through `dispatch_subslice`; `new_child` instead spawns a
            // deserializer that inherits the current depth, keeping the
            // recursion-depth guard continuous across the flattened boundary.
            for (_idx, (member, bytes)) in flattened_groups {
                let mut wrapped = Vec::with_capacity(bytes.len() + 16);
                wrapped.extend_from_slice(b"<__flat>");
                wrapped.extend_from_slice(&bytes);
                wrapped.extend_from_slice(b"</__flat>");
                let mut child_deser = self.new_child(&wrapped);
                consumer(member, &mut child_deser)?;
            }
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_list(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        let item_name = Self::list_item_name(self.settings.strict_collection_element_names, schema);
        self.enter_aggregate()?;
        // IIFE: any `?` inside falls through to `leave_aggregate` below
        // (see `read_string_list` for rationale).
        let result = (|| -> Result<(), SerdeError> {
            let input = self.input;
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;

            // Each child tag is a list item (with `strict_collection_element_names`,
            // each one named as an item). Provide each item to the consumer
            // by re-pointing `self` at the item's sub-slice via dispatch_subslice.
            // Scalar consumers will navigate to the text via `take_text()`;
            // aggregate consumers (read_list / read_struct / read_map) will
            // descend into the element's content. This keeps the deserializer
            // compatible with both scalar list elements and nested aggregate
            // elements (e.g. list-of-lists, list-of-structs) without per-element
            // type sniffing.
            while let Some(child_scope) = root.next_tag() {
                if !Self::is_list_item(item_name, child_scope.start_el()) {
                    continue;
                }
                let start = Self::element_start(input, child_scope.start_el().local());
                let sub = &input[start..child_scope.end_offset()];
                self.dispatch_subslice(sub, |this| consumer(this))?;
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_map(
        &mut self,
        schema: &Schema<'_>,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.enter_aggregate()?;
        // IIFE: any `?` inside falls through to `leave_aggregate` below
        // (see `read_string_list` for rationale).
        let result = (|| -> Result<(), SerdeError> {
            let input = self.input;
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;

            // Resolve key/value element names from the schema.
            let key_name = schema
                .key()
                .and_then(|k| k.xml_name().map(|t| t.value()))
                .unwrap_or("key");
            let value_name = schema
                .member()
                .and_then(|v| v.xml_name().map(|t| t.value()))
                .unwrap_or("value");

            // Aggregate (struct/list/map) value types need a sub-document deserializer
            // since their content includes nested elements rather than just text.
            let value_is_aggregate = schema
                .member()
                .map(|v| v.shape_type().is_aggregate())
                .unwrap_or(false);

            // Each child tag is an entry (e.g. <entry><key>k</key><value>v</value></entry>);
            // with `strict_collection_element_names`, only the `entry` children of a
            // wrapped map are.
            while let Some(mut entry_scope) = root.next_tag() {
                if !Self::is_map_entry(
                    self.settings.strict_collection_element_names,
                    schema,
                    entry_scope.start_el(),
                ) {
                    continue;
                }
                let mut key: Option<String> = None;
                // For scalar values we capture the text upfront; for aggregate
                // values we capture the element sub-slice. At most one is set.
                let mut value_text: Option<Cow<'_, str>> = None;
                let mut value_slice: Option<&'_ [u8]> = None;
                while let Some(mut field_scope) = entry_scope.next_tag() {
                    let local = field_scope.start_el().local();
                    let (is_key, is_value) = (local == key_name, local == value_name);
                    if is_key {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        key = Some(text.into_owned());
                    } else if is_value {
                        if value_is_aggregate {
                            let start = Self::element_start(input, field_scope.start_el().local());
                            let sub = &input[start..field_scope.end_offset()];
                            value_slice = Some(sub);
                        } else {
                            let text = decode::try_data(&mut field_scope)
                                .map_err(|e| SerdeError::custom(e.to_string()))?;
                            value_text = Some(text);
                        }
                    }
                }
                drop(entry_scope);
                if let Some(k) = key {
                    if let Some(slice) = value_slice {
                        self.dispatch_subslice(slice, |this| consumer(k, this))?;
                    } else if let Some(t) = value_text {
                        // Re-borrow t as 'a — the text was extracted from `doc`
                        // which borrows `self.input: &'a [u8]`, so its lifetime
                        // is `'a`.
                        let t: Cow<'_, str> = t;
                        self.dispatch_text(Ok(t.into_owned().into()), |this| consumer(k, this))?;
                    }
                }
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_boolean(&mut self, _schema: &Schema<'_>) -> Result<bool, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(SerdeError::custom(format!("invalid boolean: {other}"))),
        }
    }

    fn read_byte(&mut self, _schema: &Schema<'_>) -> Result<i8, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_short(&mut self, _schema: &Schema<'_>) -> Result<i16, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_integer(&mut self, _schema: &Schema<'_>) -> Result<i32, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_long(&mut self, _schema: &Schema<'_>) -> Result<i64, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_float(&mut self, _schema: &Schema<'_>) -> Result<f32, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "NaN" => Ok(f32::NAN),
            "Infinity" => Ok(f32::INFINITY),
            "-Infinity" => Ok(f32::NEG_INFINITY),
            _ => text.parse().map_err(|e| SerdeError::custom(format!("{e}"))),
        }
    }

    fn read_double(&mut self, _schema: &Schema<'_>) -> Result<f64, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "NaN" => Ok(f64::NAN),
            "Infinity" => Ok(f64::INFINITY),
            "-Infinity" => Ok(f64::NEG_INFINITY),
            _ => text.parse().map_err(|e| SerdeError::custom(format!("{e}"))),
        }
    }

    fn read_big_integer(&mut self, _schema: &Schema<'_>) -> Result<BigInteger, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_big_decimal(&mut self, _schema: &Schema<'_>) -> Result<BigDecimal, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_string(&mut self, _schema: &Schema<'_>) -> Result<String, SerdeError> {
        let text = self.take_text()?;
        Ok(text.into_owned())
    }

    fn read_blob(&mut self, _schema: &Schema<'_>) -> Result<Blob, SerdeError> {
        let text = self.take_text()?;
        let bytes = aws_smithy_types::base64::decode(text.as_ref())
            .map_err(|e| SerdeError::custom(format!("{e}")))?;
        Ok(Blob::new(bytes))
    }

    fn read_timestamp(&mut self, schema: &Schema<'_>) -> Result<DateTime, SerdeError> {
        let text = self.take_text()?;
        let format = self.resolve_timestamp_format(schema)?;
        DateTime::from_str(text.as_ref(), format).map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_document(&mut self, _schema: &Schema<'_>) -> Result<SmithyDocument, SerdeError> {
        Err(SerdeError::custom(
            "document types are not supported by REST XML",
        ))
    }

    // -------- Specialized collection overrides --------
    //
    // The default trait impls of `read_*_list` / `read_string_string_map`
    // call `self.read_list` / `self.read_map` with a `&mut dyn FnMut` consumer
    // that itself calls `&mut dyn ShapeDeserializer::read_X` per element.
    // For XML this is doubly wasteful: each list element pays
    //
    //   1. one `&mut dyn FnMut` indirect call,
    //   2. one `&mut dyn ShapeDeserializer` virtual call,
    //   3. and — because the consumer goes through `dispatch_subslice` —
    //      a fresh `Document::try_from` over the element's sub-slice plus
    //      a save/restore of (`input`, `text`).
    //
    // The overrides below walk the existing tokenizer once and extract
    // text inline via `decode::try_data`, eliminating all three costs.
    // Like `XmlDeserializer::read_list` / `read_map`, they take every child
    // element as an item or entry unless `strict_collection_element_names`
    // is set, in which case they take only the children named as the schema
    // says items and entries are and skip the others.
    //
    // Sparse lists are not routed here: `SchemaGenerator` only emits
    // `read_string_list` / `read_blob_list` / `read_integer_list` /
    // `read_long_list` / `read_string_string_map` for non-sparse element
    // shapes (see SchemaGenerator.kt line ~1497).

    fn read_string_list(&mut self, schema: &Schema<'_>) -> Result<Vec<String>, SerdeError> {
        let item_name = Self::list_item_name(self.settings.strict_collection_element_names, schema);
        self.enter_aggregate()?;
        // IIFE so that any `?` short-circuit still falls through to
        // `leave_aggregate` below — preserving the depth counter on the
        // error path. Same pattern in `read_blob_list`, `read_integer_list`,
        // `read_long_list`, and `read_string_string_map`.
        let result = (|| -> Result<Vec<String>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                if !Self::is_list_item(item_name, child_scope.start_el()) {
                    continue;
                }
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                out.push(text.into_owned());
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_blob_list(&mut self, schema: &Schema<'_>) -> Result<Vec<Blob>, SerdeError> {
        use aws_smithy_types::base64;
        let item_name = Self::list_item_name(self.settings.strict_collection_element_names, schema);
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<Blob>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                if !Self::is_list_item(item_name, child_scope.start_el()) {
                    continue;
                }
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let bytes = base64::decode(text.as_ref())
                    .map_err(|e| SerdeError::custom(format!("invalid base64: {e}")))?;
                out.push(Blob::new(bytes));
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_integer_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i32>, SerdeError> {
        let item_name = Self::list_item_name(self.settings.strict_collection_element_names, schema);
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<i32>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                if !Self::is_list_item(item_name, child_scope.start_el()) {
                    continue;
                }
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let v: i32 = text
                    .parse()
                    .map_err(|e| SerdeError::custom(format!("{e}")))?;
                out.push(v);
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_long_list(&mut self, schema: &Schema<'_>) -> Result<Vec<i64>, SerdeError> {
        let item_name = Self::list_item_name(self.settings.strict_collection_element_names, schema);
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<i64>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                if !Self::is_list_item(item_name, child_scope.start_el()) {
                    continue;
                }
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let v: i64 = text
                    .parse()
                    .map_err(|e| SerdeError::custom(format!("{e}")))?;
                out.push(v);
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_string_string_map(
        &mut self,
        schema: &Schema<'_>,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        let key_name = schema
            .key()
            .and_then(|k| k.xml_name().map(|t| t.value()))
            .unwrap_or("key");
        let value_name = schema
            .member()
            .and_then(|v| v.xml_name().map(|t| t.value()))
            .unwrap_or("value");

        self.enter_aggregate()?;
        let result = (|| -> Result<std::collections::HashMap<String, String>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = std::collections::HashMap::new();
            while let Some(mut entry_scope) = root.next_tag() {
                if !Self::is_map_entry(
                    self.settings.strict_collection_element_names,
                    schema,
                    entry_scope.start_el(),
                ) {
                    continue;
                }
                let mut k: Option<String> = None;
                let mut v: Option<String> = None;
                while let Some(mut field_scope) = entry_scope.next_tag() {
                    let local = field_scope.start_el().local();
                    let (is_key, is_value) = (local == key_name, local == value_name);
                    if is_key {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        k = Some(text.into_owned());
                    } else if is_value {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        v = Some(text.into_owned());
                    }
                }
                if let (Some(k), Some(v)) = (k, v) {
                    out.insert(k, v);
                }
            }
            drop(root);
            self.finish_document(&mut doc)?;
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn is_null(&self) -> bool {
        // XML represents absence by omitting the element entirely.
        // If we have a deserializer, the element exists, so it's not null.
        false
    }

    /// Nothing to advance. `read_struct` delimits each child before dispatching it —
    /// either as an owned sub-slice or as pre-extracted text — and the child's
    /// `ScopedDecoder` is dropped, which moves the parent's tokenizer past the closing
    /// tag regardless of what the consumer did. Advancing again here would consume a
    /// sibling element.
    fn skip_value(&mut self) -> Result<(), SerdeError> {
        Ok(())
    }

    fn container_size(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::{shape_id, Schema, ShapeType};

    static STRING_MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("test", "S$v"), ShapeType::String, "v", 0);

    #[test]
    fn read_string_from_text_state() {
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::from_text(Cow::Borrowed("hello"), settings);
        let result = deser.read_string(&STRING_MEMBER).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn read_string_from_doc_text_content() {
        // Verify a Doc-state deserializer can extract leaf text via the
        // public `read_string` API (which goes through `take_text` →
        // lazy Document construction over `self.input`).
        static V_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "X$v"), ShapeType::String, "v", 0);
        let xml = b"<root>world</root>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);
        let result = deser.read_string(&V_MEMBER).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn is_null_always_false() {
        let settings = Arc::new(XmlCodecSettings::default());
        let deser = XmlDeserializer::new(b"<r/>", settings);
        assert!(!deser.is_null());
    }

    // Struct member dispatch by element name (`@xmlName` and member name).

    static NAME_MEMBER: Schema<'static> = Schema::new_member(
        shape_id!("test", "Person$name"),
        ShapeType::String,
        "name",
        0,
    );
    static AGE_MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("test", "Person$age"), ShapeType::String, "age", 1);
    static RENAMED_MEMBER: Schema<'static> = Schema::new_member(
        shape_id!("test", "Person$nick"),
        ShapeType::String,
        "nick",
        2,
    )
    .with_xml_name("Nickname");

    static PERSON_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "Person"),
        ShapeType::Structure,
        &[&NAME_MEMBER, &AGE_MEMBER, &RENAMED_MEMBER],
    );

    #[test]
    fn read_struct_dispatches_members() {
        let xml = b"<Person><name>Alice</name><age>30</age></Person>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        let mut name = String::new();
        let mut age = String::new();
        deser
            .read_struct(&PERSON_SCHEMA, &mut |member, d| {
                match member.member_name().unwrap() {
                    "name" => name = d.read_string(member)?,
                    "age" => age = d.read_string(member)?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(name, "Alice");
        assert_eq!(age, "30");
    }

    #[test]
    fn read_struct_skips_unknown_elements() {
        let xml = b"<Person><unknown>x</unknown><name>Bob</name></Person>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        let mut name = String::new();
        deser
            .read_struct(&PERSON_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("name") {
                    name = d.read_string(member)?;
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(name, "Bob");
    }

    /// A member's own name can collide with another member's `@xmlName`. Every
    /// occurrence of the shared element resolves to the first matching member, wherever
    /// it appears and however often it repeats.
    #[test]
    fn colliding_element_names_resolve_to_the_first_member() {
        static STATUS: Schema<'static> = Schema::new_member(
            shape_id!("ns", "S", "status"),
            ShapeType::String,
            "status",
            0,
        );
        static STATUS_TEXT: Schema<'static> = Schema::new_member(
            shape_id!("ns", "S", "statusText"),
            ShapeType::String,
            "statusText",
            1,
        )
        .with_xml_name("status");
        static OTHER: Schema<'static> =
            Schema::new_member(shape_id!("ns", "S", "other"), ShapeType::String, "other", 2);
        static S: Schema<'static> = Schema::new_struct(
            shape_id!("ns", "S"),
            ShapeType::Structure,
            &[&STATUS, &STATUS_TEXT, &OTHER],
        );

        fn read(xml: &[u8]) -> Vec<(usize, String)> {
            let mut deser = XmlDeserializer::new(xml, Arc::new(XmlCodecSettings::default()));
            let mut seen = Vec::new();
            deser
                .read_struct(&S, &mut |member, d| {
                    seen.push((member.member_index().unwrap(), d.read_string(member)?));
                    Ok(())
                })
                .unwrap();
            seen
        }

        assert_eq!(
            read(b"<S><status>a</status><status>b</status></S>"),
            [(0, "a".to_owned()), (0, "b".to_owned())]
        );
        assert_eq!(
            read(b"<S><other>x</other><status>a</status></S>"),
            [(2, "x".to_owned()), (0, "a".to_owned())]
        );
    }

    #[test]
    fn strict_read_rejects_ill_formed_xml() {
        static P_STRING: Schema<'static> =
            Schema::new_member(shape_id!("test", "P$s"), ShapeType::String, "s", 0);
        static P_LIST: Schema<'static> =
            Schema::new_member(shape_id!("test", "P$list"), ShapeType::List, "list", 1);
        static P_FLAT: Schema<'static> =
            Schema::new_member(shape_id!("test", "P$flat"), ShapeType::List, "flat", 2)
                .with_xml_flattened();
        static P_MAP: Schema<'static> =
            Schema::new_member(shape_id!("test", "P$map"), ShapeType::Map, "map", 3);
        static P_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "P"),
            ShapeType::Structure,
            &[&P_STRING, &P_LIST, &P_FLAT, &P_MAP],
        );
        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        fn read(input: &str, strict: bool) -> Result<(), SerdeError> {
            let settings = Arc::new(
                XmlCodecSettings::builder()
                    .enforce_strictness(strict)
                    .build(),
            );
            XmlDeserializer::new(input.as_bytes(), settings).read_struct(&P_SCHEMA, &mut |m, d| {
                match m.member_name().unwrap() {
                    "s" => d.read_string(m).map(|_| ()),
                    "list" | "flat" => d.read_list(m, &mut |d| d.read_string(m).map(|_| ())),
                    _ => d.read_map(&MAP_SCHEMA, &mut |_, d| d.read_string(m).map(|_| ())),
                }
            })
        }

        for ok in [
            "<P><s>x</s><list><member>a</member></list><flat>b</flat><flat>c</flat></P>",
            "<?xml version=\"1.0\"?><!-- c --><P><s><![CDATA[<x>]]></s></P>\n<!-- trailing -->",
            r#"<a:P xmlns:a="u"><a:s>x</a:s><list ><member/></list ></a:P>"#,
            "<P><unknown><deep/></unknown><map><entry><key>k</key><value>v</value></entry></map></P>",
            "<P/>",
        ] {
            read(ok, true).unwrap_or_else(|e| panic!("well-formed {ok:?} rejected: {e}"));
        }

        for bad in [
            // proof 55: mismatched end tags
            "<P><map><gntry><key>k</key><value>v</value></entry></map><s>s</s></P>",
            "<P><list><member>a</member></l><member>b</member></list></P>",
            // mismatched end tag inside a flattened member / an unknown member
            "<P><flat>a</flatx><flat>b</flat></P>",
            "<P><unknown><deep></unknown></P>",
            // prefix must match too
            r#"<P xmlns:a="u" xmlns:b="v"><a:s>x</b:s></P>"#,
            r#"<P xmlns:a="u"><s>x</a:s></P>"#,
            r#"<P xmlns:a="u"><a:s>x</s></P>"#,
            // unclosed root, unclosed child
            "<P><s>x</s>",
            "<P><s>x</P>",
            // stray close tag after the root, second root
            "<P><s>x</s></P></P>",
            "<P></P><P></P>",
        ] {
            assert!(read(bad, true).is_err(), "ill-formed {bad:?} accepted");
            // The lenient (client) read still recovers as before.
            let _ = read(bad, false);
        }
        // Lenient reads keep recovering rather than rejecting.
        read("<P><s>x</s>", false).expect("lenient read accepts an unclosed root");
    }

    #[test]
    fn strict_roots_only_validate_document_boundary() {
        static CHILD: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Child"),
            aws_smithy_schema::ShapeType::Structure,
            &[],
        );
        static RENAMED: Schema<'static> = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Root", "child"),
            aws_smithy_schema::ShapeType::Structure,
            "child",
            0,
        )
        .with_xml_name("Renamed");
        static ROOT: Schema<'static> = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Synthetic"),
            aws_smithy_schema::ShapeType::Structure,
            &[&RENAMED],
        )
        .with_original_name("Original")
        .with_xml_name("WireRoot");
        for strict in [false, true] {
            let settings = Arc::new(
                XmlCodecSettings::builder()
                    .enforce_strictness(strict)
                    .build(),
            );
            for (input, matches) in [
                (b"<WireRoot><Renamed/></WireRoot>".as_slice(), true),
                (b"<Wrong><Renamed/></Wrong>", false),
            ] {
                let mut visited = false;
                let result = XmlDeserializer::new(input, settings.clone()).read_struct(
                    &ROOT,
                    &mut |_, d| {
                        visited = true;
                        d.read_struct(&CHILD, &mut |_, _| Ok(()))
                    },
                );
                assert_eq!(result.is_ok(), !strict || matches);
                assert_eq!(visited, !strict || matches);
            }
        }
    }

    #[test]
    fn read_struct_resolves_xml_name() {
        let xml = b"<Person><Nickname>Ally</Nickname></Person>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        let mut nick = String::new();
        deser
            .read_struct(&PERSON_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("nick") {
                    nick = d.read_string(member)?;
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(nick, "Ally");
    }

    // `@xmlAttribute` dispatch from the start element's attributes.

    static ATTR_MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("test", "X$id"), ShapeType::String, "id", 0)
            .with_xml_attribute();
    static ELEM_MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("test", "X$name"), ShapeType::String, "name", 1);

    static X_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "X"),
        ShapeType::Structure,
        &[&ATTR_MEMBER, &ELEM_MEMBER],
    );

    #[test]
    fn read_struct_dispatches_attributes() {
        let xml = b"<X id=\"42\"><name>hello</name></X>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        let mut id = String::new();
        let mut name = String::new();
        deser
            .read_struct(&X_SCHEMA, &mut |member, d| {
                match member.member_name().unwrap() {
                    "id" => id = d.read_string(member)?,
                    "name" => name = d.read_string(member)?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(id, "42");
        assert_eq!(name, "hello");
    }

    // Wrapped list / map reads with element-name resolution.

    #[test]
    fn read_list_wrapped() {
        let xml = b"<items><member>a</member><member>b</member></items>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

        let mut items = Vec::new();
        deser
            .read_list(&LIST_SCHEMA, &mut |d| {
                items.push(d.read_string(&LIST_MEMBER)?);
                Ok(())
            })
            .unwrap();

        assert_eq!(items, vec!["a", "b"]);
    }

    #[test]
    fn read_map_wrapped() {
        let xml = b"<myMap><entry><key>k1</key><value>v1</value></entry><entry><key>k2</key><value>v2</value></entry></myMap>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0);
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        let mut entries = Vec::new();
        deser
            .read_map(&MAP_SCHEMA, &mut |k, d| {
                entries.push((k, d.read_string(&MAP_VALUE)?));
                Ok(())
            })
            .unwrap();

        assert_eq!(
            entries,
            vec![
                ("k1".to_owned(), "v1".to_owned()),
                ("k2".to_owned(), "v2".to_owned())
            ]
        );
    }

    #[test]
    fn read_map_with_renamed_key_value() {
        let xml = b"<m><entry><Attribute>a</Attribute><Setting>s</Setting></entry></m>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0)
                .with_xml_name("Attribute");
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0)
                .with_xml_name("Setting");
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        let mut entries = Vec::new();
        deser
            .read_map(&MAP_SCHEMA, &mut |k, d| {
                entries.push((k, d.read_string(&MAP_VALUE)?));
                Ok(())
            })
            .unwrap();

        assert_eq!(entries, vec![("a".to_owned(), "s".to_owned())]);
    }

    // Flattened collections: repeated sibling elements accumulated into one read.

    #[test]
    fn read_struct_flattened_list() {
        // Flattened list: repeated <item> siblings inside the struct.
        let xml = b"<S><name>hi</name><item>a</item><item>b</item></S>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static S_NAME: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$name"), ShapeType::String, "name", 0);
        static S_ITEMS: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 1)
                .with_xml_flattened()
                .with_xml_name("item");
        static S_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "S"),
            ShapeType::Structure,
            &[&S_NAME, &S_ITEMS],
        );

        let mut name = String::new();
        let mut items = Vec::new();
        deser
            .read_struct(&S_SCHEMA, &mut |member, d| {
                match member.member_name().unwrap() {
                    "name" => name = d.read_string(member)?,
                    "items" => d.read_list(member, &mut |d| {
                        items.push(d.read_string(member)?);
                        Ok(())
                    })?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(name, "hi");
        assert_eq!(items, vec!["a", "b"]);
    }

    /// Regression (proofs 51/52): markup-like text in comments, CDATA,
    /// processing instructions and attribute values inside a flattened list,
    /// a list of structs and a map with struct values must not change what is
    /// decoded. Values follow `try_data`: the first text node wins, and CDATA
    /// is skipped (matching the legacy decoder).
    #[test]
    fn read_struct_ignores_markup_in_comments_cdata_pi_attrs() {
        static S_ITEMS: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 0)
                .with_xml_flattened()
                .with_xml_name("item");
        static S_PEOPLE: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$people"), ShapeType::List, "people", 1);
        static S_MAP: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$m"), ShapeType::Map, "m", 2);
        static S_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "S"),
            ShapeType::Structure,
            &[&S_ITEMS, &S_PEOPLE, &S_MAP],
        );
        static PEOPLE_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "People$member"),
            ShapeType::Structure,
            "member",
            0,
        );
        static PEOPLE_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "People"), &PEOPLE_MEMBER);
        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> = Schema::new_member(
            shape_id!("test", "M$value"),
            ShapeType::Structure,
            "value",
            1,
        );
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        let xml = concat!(
            "<S>",
            "<item><!--><item>-->a</item>",
            "<item><![CDATA[><item>]]></item>",
            "<item><?pi <item>?>c</item>",
            r#"<item x="/>">d</item>"#,
            "<people>",
            "<member><!--><member>--><name>p1</name></member>",
            r#"<member x="/>"><![CDATA[<member>]]><name>p2</name></member>"#,
            "</people>",
            "<m><entry><key>k</key><value><!--><value>--><name>v</name></value></entry></m>",
            "</S>",
        );
        let mut deser = XmlDeserializer::new(xml.as_bytes(), Arc::new(XmlCodecSettings::default()));

        fn read_name(d: &mut dyn ShapeDeserializer) -> Result<String, SerdeError> {
            let mut name = String::new();
            d.read_struct(&PERSON_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("name") {
                    name = d.read_string(member)?;
                }
                Ok(())
            })?;
            Ok(name)
        }

        let (mut items, mut people, mut map) = (Vec::new(), Vec::new(), Vec::new());
        deser
            .read_struct(&S_SCHEMA, &mut |member, d| {
                match member.member_name().unwrap() {
                    "items" => d.read_list(member, &mut |d| {
                        items.push(d.read_string(member)?);
                        Ok(())
                    })?,
                    "people" => d.read_list(&PEOPLE_SCHEMA, &mut |d| {
                        people.push(read_name(d)?);
                        Ok(())
                    })?,
                    "m" => d.read_map(&MAP_SCHEMA, &mut |k, d| {
                        map.push((k, read_name(d)?));
                        Ok(())
                    })?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(items, vec!["a", "", "c", "d"]);
        assert_eq!(people, vec!["p1", "p2"]);
        assert_eq!(map, vec![("k".to_owned(), "v".to_owned())]);
    }

    #[test]
    fn read_struct_flattened_list_intermixed() {
        // Flattened list elements intermixed with other members.
        let xml = b"<S><item>x</item><name>n</name><item>y</item></S>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static S_NAME: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$name"), ShapeType::String, "name", 0);
        static S_ITEMS: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 1)
                .with_xml_flattened()
                .with_xml_name("item");
        static S_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "S"),
            ShapeType::Structure,
            &[&S_NAME, &S_ITEMS],
        );

        let mut name = String::new();
        let mut items = Vec::new();
        deser
            .read_struct(&S_SCHEMA, &mut |member, d| {
                match member.member_name().unwrap() {
                    "name" => name = d.read_string(member)?,
                    "items" => d.read_list(member, &mut |d| {
                        items.push(d.read_string(member)?);
                        Ok(())
                    })?,
                    _ => {}
                }
                Ok(())
            })
            .unwrap();

        assert_eq!(name, "n");
        assert_eq!(items, vec!["x", "y"]);
    }

    #[test]
    fn read_struct_flattened_map() {
        // Flattened map: each `<attr>` sibling is an entry, whatever its name;
        // an `<entry>` sibling is not a member of the structure and is skipped.
        let xml = b"<S><attr><key>a</key><value>1</value></attr><name>n</name>\
                    <entry><key>x</key><value>9</value></entry>\
                    <attr><key>b</key><value>2</value></attr></S>";

        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static S_NAME: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$name"), ShapeType::String, "name", 0);
        static S_ATTRS: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$attrs"), ShapeType::Map, "attrs", 1)
                .with_map_members(&MAP_KEY, &MAP_VALUE)
                .with_xml_flattened()
                .with_xml_name("attr");
        static S_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "S"),
            ShapeType::Structure,
            &[&S_NAME, &S_ATTRS],
        );

        for (helper, check_names) in [(false, false), (true, false), (false, true), (true, true)] {
            let settings = XmlCodecSettings::builder()
                .strict_collection_element_names(check_names)
                .build();
            let mut deser = XmlDeserializer::new(xml, Arc::new(settings));
            let mut attrs = Vec::new();
            deser
                .read_struct(&S_SCHEMA, &mut |member, d| {
                    if member.member_name() == Some("attrs") {
                        if helper {
                            attrs.extend(d.read_string_string_map(member)?);
                        } else {
                            d.read_map(member, &mut |k, d| {
                                attrs.push((k, d.read_string(&MAP_VALUE)?));
                                Ok(())
                            })?;
                        }
                    }
                    Ok(())
                })
                .unwrap();
            attrs.sort();
            assert_eq!(
                attrs,
                vec![
                    ("a".to_owned(), "1".to_owned()),
                    ("b".to_owned(), "2".to_owned())
                ]
            );
        }
    }

    // Scalar reads (booleans, ints, floats, blob, timestamp) and document rejection.

    #[test]
    fn read_scalars() {
        let settings = Arc::new(XmlCodecSettings::default());

        let mut d = XmlDeserializer::from_text(Cow::Borrowed("true"), settings.clone());
        assert!(d.read_boolean(&STRING_MEMBER).unwrap());

        let mut d = XmlDeserializer::from_text(Cow::Borrowed("-42"), settings.clone());
        assert_eq!(d.read_integer(&STRING_MEMBER).unwrap(), -42);

        let mut d = XmlDeserializer::from_text(Cow::Borrowed("NaN"), settings.clone());
        assert!(d.read_float(&STRING_MEMBER).unwrap().is_nan());

        let mut d = XmlDeserializer::from_text(Cow::Borrowed("Infinity"), settings.clone());
        assert_eq!(d.read_double(&STRING_MEMBER).unwrap(), f64::INFINITY);

        let mut d = XmlDeserializer::from_text(Cow::Borrowed("aGVsbG8="), settings.clone());
        assert_eq!(d.read_blob(&STRING_MEMBER).unwrap().as_ref(), b"hello");

        let mut d =
            XmlDeserializer::from_text(Cow::Borrowed("2023-04-01T12:00:00Z"), settings.clone());
        let ts = d.read_timestamp(&STRING_MEMBER).unwrap();
        assert_eq!(ts.secs(), 1680350400);
    }

    #[test]
    fn read_document_returns_error() {
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::from_text(Cow::Borrowed("x"), settings);
        assert_eq!(
            deser.read_document(&STRING_MEMBER).unwrap_err().to_string(),
            "document types are not supported by REST XML"
        );
    }

    // Empty body → error. The XML codec is strict here because XML 1.0
    // requires every document to have a root element. Consumers (e.g., S3
    // HEAD operations) whose output struct has no body-bound members rely
    // on the HTTP response composite skipping the body deserializer
    // entirely, so they never reach `read_struct`. Operations that DO have body-bound members and
    // receive an empty body are responding to a malformed wire format —
    // the deserializer surfaces that as an error rather than silently
    // returning a default-built struct (which the legacy XML parser also
    // did not do).
    #[test]
    fn read_struct_empty_body_errors() {
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(b"", settings);
        let err = deser
            .read_struct(&PERSON_SCHEMA, &mut |_member, _d| Ok(()))
            .expect_err("empty body must be rejected by read_struct");
        let _ = format!("{err}");
    }

    #[test]
    fn read_list_empty_body_errors() {
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(b"", settings);
        deser
            .read_list(&PERSON_SCHEMA, &mut |_d| Ok(()))
            .expect_err("empty body must be rejected by read_list");
    }

    #[test]
    fn read_map_empty_body_errors() {
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(b"", settings);
        deser
            .read_map(&PERSON_SCHEMA, &mut |_k, _d| Ok(()))
            .expect_err("empty body must be rejected by read_map");
    }

    // Recursion-depth guard tests. These pin the read_struct / read_list /
    // read_map paths to a small custom `max_depth` so we can exercise the
    // overflow path without constructing pathologically large XML.

    /// Lower max_depth to `n` for the test deserializer.
    fn settings_with_max_depth(n: u32) -> Arc<XmlCodecSettings> {
        Arc::new(XmlCodecSettings::builder().max_depth(n).build())
    }

    /// Build a `<r>(<r>)*N(value)(</r>)*N` chain `depth` levels deep.
    fn nested_struct_xml(depth: u32) -> Vec<u8> {
        let mut s = String::new();
        for _ in 0..depth {
            s.push_str("<r>");
        }
        s.push('v');
        for _ in 0..depth {
            s.push_str("</r>");
        }
        s.into_bytes()
    }

    #[test]
    fn read_struct_rejects_overdeep_payloads() {
        // Self-referential schema: `R { r: R }`. Each `read_struct` with the
        // same schema increments depth and recurses one level via the
        // member dispatch.
        static R_MEMBER_SELF: Schema<'static> =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "R"),
            ShapeType::Structure,
            &[&R_MEMBER_SELF],
        );

        // Tighter limit so the test is fast.
        let max = 4;
        let xml = nested_struct_xml(max + 2);
        let mut deser = XmlDeserializer::new(&xml, settings_with_max_depth(max));

        // Recursive consumer: each entry into `<r>` calls read_struct again.
        fn consume(_m: &Schema<'_>, d: &mut dyn ShapeDeserializer) -> Result<(), SerdeError> {
            d.read_struct(&R_SCHEMA, &mut consume)
        }
        let err = deser
            .read_struct(&R_SCHEMA, &mut consume)
            .expect_err("must reject payload exceeding max_depth");
        assert!(
            format!("{err}").contains("maximum nesting depth exceeded"),
            "expected depth-exceeded error, got: {err}"
        );
    }

    #[test]
    fn read_struct_accepts_payloads_up_to_max_depth() {
        // Inverse of the above: at exactly `max_depth` we should succeed.
        // Walking the chain `max_depth` times consumes `max_depth` enter
        // calls (the outermost is the test's own call, inner consumer
        // recursions add one each).
        static R_MEMBER_SELF: Schema<'static> =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "R"),
            ShapeType::Structure,
            &[&R_MEMBER_SELF],
        );

        // 4 nested `<r>` opens; the consumer recurses for each `<r>` it
        // encounters as a child member. At depth=max we stop recursing
        // (consumer only recurses when it sees an `<r>` child element).
        let max = 4;
        let xml = nested_struct_xml(max);
        let mut deser = XmlDeserializer::new(&xml, settings_with_max_depth(max));

        let mut depth_seen = 0u32;
        fn consume(
            _m: &Schema<'_>,
            d: &mut dyn ShapeDeserializer,
            depth_seen: &mut u32,
        ) -> Result<(), SerdeError> {
            *depth_seen += 1;
            d.read_struct(&R_SCHEMA, &mut |m, d2| consume(m, d2, depth_seen))
        }

        deser
            .read_struct(&R_SCHEMA, &mut |m, d| consume(m, d, &mut depth_seen))
            .expect("payload at exactly max_depth must succeed");
    }

    #[test]
    fn read_list_rejects_overdeep_payloads() {
        // Nested lists: `<l><l><l>...</l></l></l>` exceeds max_depth.
        static L_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "L$member"), ShapeType::List, "member", 0);
        static L_SCHEMA: Schema<'static> = Schema::new_list(shape_id!("test", "L"), &L_MEMBER);

        let max = 3;
        // 5 levels of `<l>` — exceeds 3.
        let xml = b"<l><l><l><l><l/></l></l></l></l>";
        let mut deser = XmlDeserializer::new(xml, settings_with_max_depth(max));

        fn consume(d: &mut dyn ShapeDeserializer) -> Result<(), SerdeError> {
            d.read_list(&L_SCHEMA, &mut consume)
        }
        let err = deser
            .read_list(&L_SCHEMA, &mut consume)
            .expect_err("nested-list payload exceeding max_depth must error");
        assert!(
            format!("{err}").contains("maximum nesting depth exceeded"),
            "expected depth-exceeded error, got: {err}"
        );
    }

    #[test]
    fn depth_resets_between_sibling_reads() {
        // After a successful aggregate read, the depth counter must return
        // to its prior value so the next sibling read isn't poisoned.
        static R_MEMBER_SELF: Schema<'static> =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "R"),
            ShapeType::Structure,
            &[&R_MEMBER_SELF],
        );

        let max = 4;
        let xml = nested_struct_xml(2);
        let mut deser = XmlDeserializer::new(&xml, settings_with_max_depth(max));

        // First read: 2 levels deep, well under max.
        deser
            .read_struct(&R_SCHEMA, &mut |_m, d| {
                d.read_struct(&R_SCHEMA, &mut |_, _| Ok(()))
            })
            .expect("first read at depth=2 must succeed");

        // Second read on the same deserializer with the same payload:
        // would fail if depth had leaked from the first call.
        deser
            .read_struct(&R_SCHEMA, &mut |_m, d| {
                d.read_struct(&R_SCHEMA, &mut |_, _| Ok(()))
            })
            .expect("sibling read on the same deserializer must succeed");
    }

    #[test]
    fn read_struct_rejects_overdeep_payloads_through_flattened_member() {
        // Regression: a shape that recurses through an @xmlFlattened list
        // member — `structure Node { @xmlFlattened kids: NodeList }` where
        // `NodeList` is a list of `Node` — must still be depth-limited.
        //
        // Flattened groups are dispatched through a freshly spawned
        // deserializer (the synthesized `<__flat>` wrapper is owned locally
        // and can't route through `dispatch_subslice`). That child must
        // inherit the parent's depth via `new_child`; if it instead started
        // at 0 (as a plain `XmlDeserializer::new` would), the counter would
        // reset on every flattened hop and this recursion would nest without
        // bound, overflowing the worker stack instead of returning an error.
        static NODE_KIDS: Schema<'static> =
            Schema::new_member(shape_id!("test", "Node$kids"), ShapeType::List, "kids", 0)
                .with_xml_flattened()
                .with_xml_name("kids");
        static NODE_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("test", "Node"),
            ShapeType::Structure,
            &[&NODE_KIDS],
        );

        // Deeply nested flattened `<kids>` chain, far beyond `max`. Each
        // `Node` level costs two enter_aggregate calls (its `read_struct`
        // plus the flattened list's `read_list`), so `max = 4` rejects within
        // a handful of levels; 50 levels guarantees we cross it.
        let max = 4;
        let depth = 50usize;
        let mut xml = Vec::new();
        xml.extend_from_slice(b"<Node>");
        for _ in 0..depth {
            xml.extend_from_slice(b"<kids>");
        }
        for _ in 0..depth {
            xml.extend_from_slice(b"</kids>");
        }
        xml.extend_from_slice(b"</Node>");

        let mut deser = XmlDeserializer::new(&xml, settings_with_max_depth(max));

        // Each `<kids>` list item is itself a `Node`, so the list consumer
        // recurses back into `read_struct`.
        fn consume(_m: &Schema<'_>, d: &mut dyn ShapeDeserializer) -> Result<(), SerdeError> {
            d.read_struct(&NODE_SCHEMA, &mut |member, d| {
                d.read_list(member, &mut |d| consume(&NODE_SCHEMA, d))
            })
        }

        let err = consume(&NODE_SCHEMA, &mut deser)
            .expect_err("recursion through a flattened member must be depth-limited");
        assert!(
            format!("{err}").contains("maximum nesting depth exceeded"),
            "expected depth-exceeded error, got: {err}"
        );
    }

    #[test]
    fn depth_resets_after_consumer_error() {
        // Regression: prior to the IIFE refactor in `read_struct` /
        // `read_list` / `read_map` / the 5 collection helpers, a `?`
        // propagating out of an aggregate read body skipped the trailing
        // `self.leave_aggregate()`, leaking +1 on the depth counter. A
        // second read on the same deserializer would then fail with
        // "maximum nesting depth exceeded" on a payload well within
        // limits.
        //
        // We force the error via a consumer that always returns Err
        // (rather than relying on a malformed XML payload — which would
        // also fail on the second read for the same wire-level reason
        // and so wouldn't isolate the depth-counter bug).
        static L_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static L_SCHEMA: Schema<'static> = Schema::new_list(shape_id!("test", "L"), &L_MEMBER);

        // max_depth=2 means we can enter at most 2 levels of aggregates
        // before erroring. With the leak, after one errored aggregate
        // read at depth=1, depth stays at 1, and a second read enters
        // at depth=2 — still OK. After two errored reads we'd be at
        // depth=2, and a third would push to depth=3 and trip the limit.
        let max = 2;
        let xml = b"<l><member>a</member><member>b</member></l>";
        let mut deser = XmlDeserializer::new(xml, settings_with_max_depth(max));

        // Force an error from the consumer on the very first element.
        let mut force_err = |_: &mut dyn ShapeDeserializer| -> Result<(), SerdeError> {
            Err(SerdeError::custom("forced"))
        };
        deser
            .read_list(&L_SCHEMA, &mut force_err)
            .expect_err("forced consumer error must propagate");

        // Repeat enough times that, with a leak of +1 per call, the depth
        // counter would exceed max_depth. With the fix, every iteration
        // ends with depth=0 and this loop is fine.
        for i in 0..(max as usize + 5) {
            deser
                .read_list(&L_SCHEMA, &mut force_err)
                .expect_err(&format!("forced consumer error must propagate (iter {i})"));
        }

        // A successful (no-op) read must still work — proving the counter
        // was decremented correctly through all the error iterations.
        let mut count = 0usize;
        deser
            .read_list(&L_SCHEMA, &mut |d| {
                count += 1;
                d.read_string(&L_MEMBER).map(|_| ())
            })
            .expect("subsequent successful read must not be poisoned by prior errors");
        assert_eq!(count, 2);
    }

    #[test]
    fn read_struct_unwrapped_output_with_prolog() {
        // Regression: in the unwrapped-output path of `read_struct`, an
        // earlier version called the element-slicing helper with a heap-allocated
        // `String` instead of a `&str` borrowing from `input`. The
        // pointer-arithmetic invariant broke; only the
        // `.saturating_sub.min` clamping prevented UB. The result was
        // silently correct only when the target element happened to start
        // at offset 0 of the input.
        //
        // Constructing a payload with an XML prolog (so the element is NOT
        // at offset 0) verifies the fixed code passes a real sub-slice of
        // `input` to `element_start`. The `debug_assert!` in
        // `element_start` would also fire under the old code in
        // debug builds.
        static MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "U$location"),
            ShapeType::String,
            "LocationConstraint",
            0,
        );
        static SCHEMA: Schema<'static> =
            Schema::new_struct(shape_id!("test", "U"), ShapeType::Structure, &[&MEMBER])
                .with_xml_unwrapped_output();

        let xml = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<LocationConstraint>us-west-2</LocationConstraint>";
        let mut deser = XmlDeserializer::new(xml, Arc::new(XmlCodecSettings::default()));

        let mut got = String::new();
        deser
            .read_struct(&SCHEMA, &mut |member, d| {
                got = d.read_string(member)?;
                Ok(())
            })
            .expect("unwrapped output must deserialize through prolog");
        assert_eq!(got, "us-west-2");
    }

    /// Regression test for a UTF-8 char-boundary panic in an earlier byte
    /// scanner that located element boundaries.
    ///
    /// Found via `schema_xml_roundtrip` fuzz target on input
    /// `StringStringMap([("Б", "")])`. The serialized payload contains a
    /// Cyrillic `Б` (UTF-8 bytes `0xD0 0x91`) inside a map-key element. The
    /// previous implementation operated on `&str` and advanced its scan
    /// cursor by one byte per non-`<` character, which landed mid-char on
    /// the second byte of `Б` and panicked with
    /// `start byte index N is not a char boundary`. Element boundaries now
    /// come from the tokenizer's offsets, which always land on a `<` or just
    /// past a `>`.
    #[test]
    fn element_slicing_handles_multibyte_utf8() {
        // Build a struct-with-map XML payload containing Cyrillic text in a
        // map-key element. The exact wire form matches what `XmlSerializer`
        // emits for `StringStringMap([("Б", "")])` wrapped in
        // `WRAPPER_STRING_STRING_MAP_SCHEMA`.
        static KEY_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "Wrapper$k"), ShapeType::Map, "entries", 0);
        static MAP_SCHEMA: Schema<'static> = Schema::new_map(
            shape_id!("test", "MyMap"),
            &aws_smithy_schema::prelude::STRING,
            &aws_smithy_schema::prelude::STRING,
        );
        static WRAPPER: Schema<'static> = Schema::new_struct(
            shape_id!("test", "Wrapper"),
            ShapeType::Structure,
            &[&KEY_MEMBER],
        );
        let _ = MAP_SCHEMA; // referenced for documentation; not directly used

        let xml =
            "<Wrapper><entries><entry><key>Б</key><value></value></entry></entries></Wrapper>";

        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml.as_bytes(), settings);

        let mut entries = Vec::new();
        deser
            .read_struct(&WRAPPER, &mut |member, d| {
                d.read_map(member, &mut |k, d| {
                    entries.push((k, d.read_string(&aws_smithy_schema::prelude::STRING)?));
                    Ok(())
                })
            })
            .expect("must not panic on multi-byte UTF-8 inside map elements");

        assert_eq!(entries, vec![("Б".to_owned(), String::new())]);
    }

    /// Slice each child of the root element the way the aggregate readers do:
    /// start from `element_start`, end at the child scope's `end_offset`.
    fn child_slices(input: &[u8]) -> Vec<&[u8]> {
        let mut doc = Document::try_from(input).unwrap();
        let mut root = doc.root_element().unwrap();
        let mut out = Vec::new();
        while let Some(child) = root.next_tag() {
            let start = XmlDeserializer::element_start(input, child.start_el().local());
            out.push(&input[start..child.end_offset()]);
        }
        out
    }

    /// Element slices must match the element exactly for prefixed names,
    /// whitespace in tags, and mismatched prefixes on the close tag.
    #[test]
    fn element_slices_prefixed_and_whitespace() {
        let input = br#"<Root><a:flatList xmlns:a="u">x</a:flatList><a:flatList xmlns:a="u">y</a:flatList></Root>"#;
        assert_eq!(
            child_slices(input),
            vec![
                &br#"<a:flatList xmlns:a="u">x</a:flatList>"#[..],
                &br#"<a:flatList xmlns:a="u">y</a:flatList>"#[..],
            ],
        );
        let ws = b"<Root><flatList >x</flatList ><flatList>y</flatList></Root>";
        assert_eq!(
            child_slices(ws),
            vec![
                &b"<flatList >x</flatList >"[..],
                &b"<flatList>y</flatList>"[..]
            ],
        );
        let empty = b"<Root><a/><b x=\"1\" /></Root>";
        assert_eq!(
            child_slices(empty),
            vec![&b"<a/>"[..], &b"<b x=\"1\" />"[..]]
        );
    }

    /// Regression (proofs 51/52): markup-like text inside comments, CDATA,
    /// processing instructions and attribute values must not shift element
    /// boundaries. The old byte scanner saw `<flatList>` inside `<!-- -->` as
    /// an open tag, never found the matching close, and returned the rest of
    /// the document for every sibling (quadratic memory and CPU).
    #[test]
    fn element_slices_ignore_markup_in_comments_cdata_pi_attrs() {
        for item in [
            &b"<flatList><!--><flatList>--></flatList>"[..],
            b"<flatList><![CDATA[><flatList>]]></flatList>",
            b"<flatList><?pi <flatList>?>x</flatList>",
            b"<flatList a=\"/>\">x</flatList>",
            b"<flatList><!-- </flatList> -->x</flatList>",
        ] {
            let mut input = b"<Root>".to_vec();
            for _ in 0..3 {
                input.extend_from_slice(item);
            }
            input.extend_from_slice(b"</Root>");
            assert_eq!(
                child_slices(&input),
                vec![item; 3],
                "{}",
                String::from_utf8_lossy(item)
            );
        }
    }

    #[test]
    fn find_depth2_by_predicate_wrapped() {
        let xml = b"<Resp><FooResult><A>1</A></FooResult><Metadata/></Resp>";
        let got = find_depth2_element_slice_by(xml, |n| n.ends_with("Result"));
        assert_eq!(got, Some(&b"<FooResult><A>1</A></FooResult>"[..]));
    }

    #[test]
    fn find_depth2_by_predicate_self_closing() {
        // Regression: a self-closing target element must return just `<Foo/>`,
        // not everything from the element to the end of the document.
        let xml = b"<Resp><FooResult/><Metadata><Id>r</Id></Metadata></Resp>";
        let got = find_depth2_element_slice_by(xml, |n| n.ends_with("Result"));
        assert_eq!(got, Some(&b"<FooResult/>"[..]));
    }

    #[test]
    fn find_depth2_by_predicate_root_match() {
        // Unwrapped envelope: the root itself matches — return the whole body.
        let xml = b"<Error><Code>Boom</Code></Error>";
        let got = find_depth2_element_slice_by(xml, |n| n == "Error");
        assert_eq!(got, Some(&xml[..]));
    }

    #[test]
    fn find_depth2_by_predicate_no_match() {
        let xml = b"<Resp><Metadata/></Resp>";
        assert_eq!(find_depth2_element_slice_by(xml, |n| n == "Error"), None);
    }

    #[test]
    fn find_depth2_by_predicate_invalid_xml() {
        assert_eq!(find_depth2_element_slice_by(b"not xml", |_| true), None);
    }

    // -------- Specialized collection-helper override tests --------
    //
    // These exercise the inlined `read_*_list` / `read_string_string_map`
    // overrides on `XmlDeserializer`, confirming behavioral parity with
    // the trait's default-impl path (which goes through `read_list` /
    // `read_map` + `&mut dyn ShapeDeserializer`).

    #[test]
    fn read_string_list_helper() {
        let xml = b"<items><member>a</member><member>b</member><member></member></items>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

        let out = deser.read_string_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec!["a".to_owned(), "b".to_owned(), String::new()]);
    }

    #[test]
    fn read_string_list_helper_renamed_member_name() {
        // Codegen-emitted call site for a list whose member shape has
        // `@xmlName("Item")`. Like `read_list`, the helper does not
        // validate child element names against the schema — it accepts
        // whatever the wire form provides. This matches the default impl.
        let xml = b"<items><Item>x</Item><Item>y</Item></items>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

        let out = deser.read_string_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec!["x".to_owned(), "y".to_owned()]);
    }

    /// Settings with `strict_collection_element_names` set to `check_names`.
    /// Without it they still enforce strictness, as a server does: strictness
    /// alone does not make a read check collection element names.
    fn element_name_settings(check_names: bool) -> Arc<XmlCodecSettings> {
        Arc::new(
            XmlCodecSettings::builder()
                .enforce_strictness(!check_names)
                .strict_collection_element_names(check_names)
                .build(),
        )
    }

    /// With `strict_collection_element_names`, a wrapped list takes only the
    /// children named as its member, as the legacy generated parsers do.
    /// Without it every child is an item, whether or not strictness is
    /// enforced.
    #[test]
    fn element_name_check_skips_list_children_not_named_as_the_member() {
        static STRING_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static STRING_LIST: Schema<'static> =
            Schema::new_list(shape_id!("test", "L"), &STRING_MEMBER);
        // A structure member targeting the list, as codegen emits it.
        static STRUCT_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$tags"), ShapeType::List, "tags", 0)
                .with_list_member(&STRING_MEMBER);
        static RENAMED_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "R$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("Item");
        static RENAMED_LIST: Schema<'static> =
            Schema::new_list(shape_id!("test", "R"), &RENAMED_MEMBER);
        static BLOB_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "B$member"), ShapeType::Blob, "member", 0);
        static BLOB_LIST: Schema<'static> = Schema::new_list(shape_id!("test", "B"), &BLOB_MEMBER);
        static INT_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "I$member"),
            ShapeType::Integer,
            "member",
            0,
        );
        static INT_LIST: Schema<'static> = Schema::new_list(shape_id!("test", "I"), &INT_MEMBER);
        static LONG_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "Lo$member"), ShapeType::Long, "member", 0);
        static LONG_LIST: Schema<'static> = Schema::new_list(shape_id!("test", "Lo"), &LONG_MEMBER);

        fn deser(xml: &'static str, check_names: bool) -> XmlDeserializer<'static> {
            XmlDeserializer::new(xml.as_bytes(), element_name_settings(check_names))
        }
        fn read_list(xml: &'static str, check_names: bool, schema: &Schema<'_>) -> Vec<String> {
            let mut items = Vec::new();
            deser(xml, check_names)
                .read_list(schema, &mut |d| {
                    items.push(d.read_string(&STRING_MEMBER)?);
                    Ok(())
                })
                .unwrap();
            items
        }

        let strings =
            "<tags><item>a</item><member>b</member><Member>c</Member><p:member>d</p:member></tags>";
        for schema in [&STRING_LIST, &STRUCT_MEMBER] {
            assert_eq!(
                deser(strings, true).read_string_list(schema).unwrap(),
                vec!["b", "d"]
            );
            assert_eq!(read_list(strings, true, schema), vec!["b", "d"]);
            assert_eq!(
                deser(strings, false).read_string_list(schema).unwrap(),
                vec!["a", "b", "c", "d"]
            );
            assert_eq!(read_list(strings, false, schema), vec!["a", "b", "c", "d"]);
        }

        let renamed = "<l><Item>x</Item><member>skipped</member><Item>y</Item></l>";
        assert_eq!(
            deser(renamed, true)
                .read_string_list(&RENAMED_LIST)
                .unwrap(),
            vec!["x", "y"]
        );
        assert_eq!(
            deser(renamed, false)
                .read_string_list(&RENAMED_LIST)
                .unwrap(),
            vec!["x", "skipped", "y"]
        );

        // A skipped child is not decoded, so its content cannot fail the read.
        let blobs = deser("<l><x>!!!</x><member>aGVsbG8=</member></l>", true)
            .read_blob_list(&BLOB_LIST)
            .unwrap();
        assert_eq!(blobs, vec![Blob::new("hello")]);
        assert_eq!(
            deser("<l><x>nan</x><member>7</member></l>", true)
                .read_integer_list(&INT_LIST)
                .unwrap(),
            vec![7]
        );
        assert_eq!(
            deser("<l><x><deep/></x><member>-9</member></l>", true)
                .read_long_list(&LONG_LIST)
                .unwrap(),
            vec![-9]
        );
    }

    /// A schema that does not describe the list's member (the placeholder
    /// codegen falls back to when it has no sub-schema for a nested aggregate)
    /// gives no item name to check.
    #[test]
    fn element_name_check_without_a_member_schema_reads_every_child() {
        let xml = b"<l><a>1</a><b>2</b></l>";
        let mut deser = XmlDeserializer::new(xml, element_name_settings(true));
        let mut items = Vec::new();
        deser
            .read_list(&aws_smithy_schema::prelude::DOCUMENT, &mut |d| {
                items.push(d.read_string(&aws_smithy_schema::prelude::STRING)?);
                Ok(())
            })
            .unwrap();
        assert_eq!(items, vec!["1", "2"]);
    }

    /// With `strict_collection_element_names`, a wrapped map takes only its
    /// `entry` children. Without it every child is an entry, whether or not
    /// strictness is enforced.
    #[test]
    fn element_name_check_skips_map_children_not_named_entry() {
        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 1);
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        let xml = "<m><item><key>x</key><value>1</value></item>\
                   <entry><key>a</key><value>2</value></entry></m>";

        for (settings, expected) in [
            (element_name_settings(true), vec![("a", "2")]),
            (element_name_settings(false), vec![("a", "2"), ("x", "1")]),
            (
                Arc::new(XmlCodecSettings::default()),
                vec![("a", "2"), ("x", "1")],
            ),
        ] {
            let expected: Vec<(String, String)> = expected
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();

            let mut out: Vec<_> = XmlDeserializer::new(xml.as_bytes(), settings.clone())
                .read_string_string_map(&MAP_SCHEMA)
                .unwrap()
                .into_iter()
                .collect();
            out.sort();
            assert_eq!(out, expected);

            let mut entries = Vec::new();
            XmlDeserializer::new(xml.as_bytes(), settings)
                .read_map(&MAP_SCHEMA, &mut |k, d| {
                    entries.push((k, d.read_string(&MAP_VALUE)?));
                    Ok(())
                })
                .unwrap();
            entries.sort();
            assert_eq!(entries, expected);
        }
    }

    /// The element-name check applies at every nesting level: each nested
    /// list or map is read against the sub-schema that names its own items.
    #[test]
    fn element_name_check_applies_to_nested_collections() {
        // `grid: list<list<String>>`, the inner member renamed to `Cell`.
        static CELL: Schema<'static> = Schema::new_member(
            shape_id!("test", "Row$member"),
            ShapeType::String,
            "member",
            0,
        )
        .with_xml_name("Cell");
        static ROW: Schema<'static> = Schema::new_member(
            shape_id!("test", "Grid$member"),
            ShapeType::List,
            "member",
            0,
        )
        .with_list_member(&CELL);
        static GRID: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$grid"), ShapeType::List, "grid", 0)
                .with_list_member(&ROW);
        // `attrs: map<String, list<String>>`.
        static ATTR_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "Attrs$key"), ShapeType::String, "key", 0);
        static ATTR_ITEM: Schema<'static> = Schema::new_member(
            shape_id!("test", "Values$member"),
            ShapeType::String,
            "member",
            0,
        );
        static ATTR_VALUE: Schema<'static> = Schema::new_member(
            shape_id!("test", "Attrs$value"),
            ShapeType::List,
            "value",
            1,
        )
        .with_list_member(&ATTR_ITEM);
        static ATTRS: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$attrs"), ShapeType::Map, "attrs", 1)
                .with_map_members(&ATTR_KEY, &ATTR_VALUE);

        fn grid(check_names: bool) -> Vec<Vec<String>> {
            let xml = b"<grid>\
                <member><Cell>1</Cell><member>x</member></member>\
                <row><Cell>9</Cell></row>\
                <member><Cell>2</Cell></member>\
                </grid>";
            let mut rows = Vec::new();
            XmlDeserializer::new(xml, element_name_settings(check_names))
                .read_list(&GRID, &mut |d| {
                    let mut row = Vec::new();
                    d.read_list(&ROW, &mut |d| {
                        row.push(d.read_string(&CELL)?);
                        Ok(())
                    })?;
                    rows.push(row);
                    Ok(())
                })
                .unwrap();
            rows
        }
        assert_eq!(grid(true), vec![vec!["1"], vec!["2"]]);
        assert_eq!(grid(false), vec![vec!["1", "x"], vec!["9"], vec!["2"]]);

        fn attrs(check_names: bool) -> Vec<(String, Vec<String>)> {
            let xml = b"<attrs>\
                <entry><key>k</key><value><member>a</member><item>b</item></value></entry>\
                <other><key>z</key><value><member>q</member></value></other>\
                </attrs>";
            let mut entries = Vec::new();
            XmlDeserializer::new(xml, element_name_settings(check_names))
                .read_map(&ATTRS, &mut |key, d| {
                    entries.push((key, d.read_string_list(&ATTR_VALUE)?));
                    Ok(())
                })
                .unwrap();
            entries.sort();
            entries
        }
        assert_eq!(attrs(true), vec![("k".to_owned(), vec!["a".to_owned()])]);
        assert_eq!(
            attrs(false),
            vec![
                ("k".to_owned(), vec!["a".to_owned(), "b".to_owned()]),
                ("z".to_owned(), vec!["q".to_owned()]),
            ]
        );
    }

    /// A recursive structure (`Node { name, children: list<Node> }`) is read
    /// against its own schema at every depth, so the check applies at each
    /// level of the tree.
    #[test]
    fn element_name_check_applies_to_recursive_structures() {
        static NODE_NAME: Schema<'static> =
            Schema::new_member(shape_id!("test", "Node$name"), ShapeType::String, "name", 0);
        static NODE_LIST_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "NodeList$member"),
            ShapeType::Structure,
            "member",
            0,
        );
        static NODE_CHILDREN: Schema<'static> = Schema::new_member(
            shape_id!("test", "Node$children"),
            ShapeType::List,
            "children",
            1,
        )
        .with_list_member(&NODE_LIST_MEMBER);
        static NODE: Schema<'static> = Schema::new_struct(
            shape_id!("test", "Node"),
            ShapeType::Structure,
            &[&NODE_NAME, &NODE_CHILDREN],
        );

        #[derive(Debug, PartialEq)]
        struct Node(String, Vec<Node>);

        fn read_node(d: &mut dyn ShapeDeserializer) -> Result<Node, SerdeError> {
            let mut node = Node(String::new(), Vec::new());
            d.read_struct(&NODE, &mut |member, d| {
                match member.member_name() {
                    Some("name") => node.0 = d.read_string(member)?,
                    Some("children") => d.read_list(member, &mut |d| {
                        node.1.push(read_node(d)?);
                        Ok(())
                    })?,
                    _ => {}
                }
                Ok(())
            })?;
            Ok(node)
        }
        fn leaf(name: &str) -> Node {
            Node(name.to_owned(), Vec::new())
        }

        let xml = b"<Node><name>root</name><children>\
            <member><name>a</name><children>\
                <bogus><name>x</name></bogus>\
                <member><name>a1</name></member>\
            </children></member>\
            <item><name>skipped</name></item>\
            <member><name>b</name></member>\
            </children></Node>";
        let read = |check_names: bool| {
            read_node(&mut XmlDeserializer::new(
                xml,
                element_name_settings(check_names),
            ))
            .unwrap()
        };
        assert_eq!(
            read(true),
            Node(
                "root".to_owned(),
                vec![Node("a".to_owned(), vec![leaf("a1")]), leaf("b")]
            )
        );
        assert_eq!(
            read(false),
            Node(
                "root".to_owned(),
                vec![
                    Node("a".to_owned(), vec![leaf("x"), leaf("a1")]),
                    leaf("skipped"),
                    leaf("b")
                ]
            )
        );
    }

    #[test]
    fn read_blob_list_helper() {
        // Each element's text is base64-decoded into a Blob.
        let xml = b"<blobs><member>aGVsbG8=</member><member>d29ybGQ=</member></blobs>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "B$member"), ShapeType::Blob, "member", 0);
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "B"), &LIST_MEMBER);

        let out = deser.read_blob_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].as_ref(), b"hello");
        assert_eq!(out[1].as_ref(), b"world");
    }

    #[test]
    fn read_blob_list_helper_rejects_invalid_base64() {
        let xml = b"<blobs><member>!!!not-base64!!!</member></blobs>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "B$member"), ShapeType::Blob, "member", 0);
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "B"), &LIST_MEMBER);

        let err = deser.read_blob_list(&LIST_SCHEMA).unwrap_err();
        assert!(format!("{err}").contains("base64"));
    }

    #[test]
    fn read_integer_list_helper() {
        let xml = b"<nums><member>1</member><member>-42</member><member>0</member></nums>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("test", "I$member"),
            ShapeType::Integer,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "I"), &LIST_MEMBER);

        let out = deser.read_integer_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec![1i32, -42, 0]);
    }

    #[test]
    fn read_long_list_helper() {
        let xml = b"<nums><member>9223372036854775807</member><member>-1</member></nums>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("test", "Lo$member"), ShapeType::Long, "member", 0);
        static LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "Lo"), &LIST_MEMBER);

        let out = deser.read_long_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec![i64::MAX, -1]);
    }

    #[test]
    fn read_string_string_map_helper() {
        let xml = b"<m><entry><key>a</key><value>1</value></entry>\
                    <entry><key>b</key><value>2</value></entry></m>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0);
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

        let out = deser.read_string_string_map(&MAP_SCHEMA).unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out.get("a").map(String::as_str), Some("1"));
        assert_eq!(out.get("b").map(String::as_str), Some("2"));
    }

    #[test]
    fn read_string_string_map_helper_with_renamed_key_value() {
        // @xmlName overrides on the map's key / value members — the
        // helper resolves these from the schema, mirroring `read_map`.
        let xml = b"<m><entry><K>a</K><V>1</V></entry></m>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static MAP_KEY: Schema<'static> =
            Schema::new_member(shape_id!("test", "M2$key"), ShapeType::String, "key", 0)
                .with_xml_name("K");
        static MAP_VALUE: Schema<'static> =
            Schema::new_member(shape_id!("test", "M2$value"), ShapeType::String, "value", 0)
                .with_xml_name("V");
        static MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "M2"), &MAP_KEY, &MAP_VALUE);

        let out = deser.read_string_string_map(&MAP_SCHEMA).unwrap();
        assert_eq!(out.get("a").map(String::as_str), Some("1"));
    }

    // Regression tests adapted from review comments on PR #4668 against an
    // earlier schema-XML deserializer that reconstructed sub-trees with
    // `try_data` + `format!("<{}>{}</{}>", ...)`. That pattern broke (a) on
    // structs with element children >1 level deep (because `try_data` errors
    // on a non-text token) and (b) on text containing `&` or `<` (because
    // unescape ran before re-emitting into fabricated tags, producing
    // invalid XML on re-parse). Our deserializer propagates raw byte slices
    // for aggregate sub-trees via `element_start` and `dispatch_subslice`,
    // so neither bug should reproduce — these tests lock that in.
    #[test]
    fn nested_struct_three_levels_deep() {
        static LEAF: Schema<'static> =
            Schema::new_member(shape_id!("t", "Inner"), ShapeType::String, "Leaf", 0);
        static INNER_SCHEMA: Schema<'static> =
            Schema::new_struct(shape_id!("t", "Inner"), ShapeType::Structure, &[&LEAF]);
        static INNER_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("t", "Middle"), ShapeType::Structure, "Inner", 0);
        static MIDDLE_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("t", "Middle"),
            ShapeType::Structure,
            &[&INNER_MEMBER],
        );
        static MIDDLE_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("t", "Outer"), ShapeType::Structure, "Middle", 0);
        static OUTER_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("t", "Outer"),
            ShapeType::Structure,
            &[&MIDDLE_MEMBER],
        );
        static OUTER_MEMBER: Schema<'static> =
            Schema::new_member(shape_id!("t", "Root"), ShapeType::Structure, "Outer", 0);
        static ROOT: Schema<'static> = Schema::new_struct(
            shape_id!("t", "Root"),
            ShapeType::Structure,
            &[&OUTER_MEMBER],
        );

        let xml = b"<Root><Outer><Middle><Inner><Leaf>value</Leaf></Inner></Middle></Outer></Root>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);
        let mut leaf = String::new();
        deser
            .read_struct(&ROOT, &mut |outer_m, d_outer| {
                assert_eq!(outer_m.member_name(), Some("Outer"));
                d_outer.read_struct(&OUTER_SCHEMA, &mut |middle_m, d_middle| {
                    assert_eq!(middle_m.member_name(), Some("Middle"));
                    d_middle.read_struct(&MIDDLE_SCHEMA, &mut |inner_m, d_inner| {
                        assert_eq!(inner_m.member_name(), Some("Inner"));
                        d_inner.read_struct(&INNER_SCHEMA, &mut |leaf_m, d_leaf| {
                            if leaf_m.member_name() == Some("Leaf") {
                                leaf = d_leaf.read_string(leaf_m)?;
                            }
                            Ok(())
                        })
                    })
                })
            })
            .expect("3-level nested struct should round-trip");
        assert_eq!(leaf, "value");
    }

    #[test]
    fn struct_member_with_escaped_text_round_trips() {
        static VALUE: Schema<'static> =
            Schema::new_member(shape_id!("t", "Body"), ShapeType::String, "value", 0);
        static BODY_SCHEMA: Schema<'static> =
            Schema::new_struct(shape_id!("t", "Body"), ShapeType::Structure, &[&VALUE]);
        static PAYLOAD_MEMBER: Schema<'static> = Schema::new_member(
            shape_id!("t", "Envelope"),
            ShapeType::Structure,
            "payload",
            0,
        );
        static ENVELOPE_SCHEMA: Schema<'static> = Schema::new_struct(
            shape_id!("t", "Envelope"),
            ShapeType::Structure,
            &[&PAYLOAD_MEMBER],
        );

        // Server response with an XML-escaped `&` in the leaf value.
        // After `unescape`, the original value is `foo&bar`. The PR's
        // deserializer would re-emit the unescaped text into reconstructed
        // markup, producing invalid XML and an error or wrong value.
        let xml = b"<Envelope><payload><value>foo&amp;bar</value></payload></Envelope>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);
        let mut value_str = String::new();
        deser
            .read_struct(&ENVELOPE_SCHEMA, &mut |payload_m, d_payload| {
                let _ = payload_m;
                d_payload.read_struct(&BODY_SCHEMA, &mut |inner_m, d_inner| {
                    if inner_m.member_name() == Some("value") {
                        value_str = d_inner.read_string(inner_m)?;
                    }
                    Ok(())
                })
            })
            .expect("escaped text inside a nested struct should round-trip");
        assert_eq!(value_str, "foo&bar");
    }

    #[test]
    fn read_map_preserves_empty_string_key() {
        // The PR's deserializer had a guard that dropped entries with empty
        // keys. Empty string is a valid map key per Smithy semantics.
        static KEY: Schema<'static> =
            Schema::new_member(shape_id!("t", "M$key"), ShapeType::String, "key", 0);
        static VALUE: Schema<'static> =
            Schema::new_member(shape_id!("t", "M$value"), ShapeType::String, "value", 1);
        static MAP: Schema<'static> = Schema::new_map(shape_id!("t", "M"), &KEY, &VALUE);

        let xml = b"<Root><entry><key></key><value>v1</value></entry></Root>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        let mut got: std::collections::HashMap<String, String> = Default::default();
        deser
            .read_map(&MAP, &mut |k, d| {
                let v = d.read_string(&VALUE)?;
                got.insert(k, v);
                Ok(())
            })
            .expect("empty-key entry should be preserved");
        assert_eq!(got.get("").map(String::as_str), Some("v1"));
    }

    /// Outer map whose value member targets a map with `@xmlName` on
    /// the inner key/value. Mirrors the wire format produced by the
    /// serializer's `map_value_is_inner_map_with_renamed_inner_key_value`
    /// test and verifies the nested read uses the inner map's renamed
    /// element names.
    #[test]
    fn read_map_value_is_inner_map_with_renamed_inner_key_value() {
        let xml = b"<outerMap><entry><key>ok</key><value><entry><InnerKey>ik</InnerKey><InnerVal>iv</InnerVal></entry></value></entry></outerMap>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static INNER_KEY: Schema<'static> = Schema::new_member(
            shape_id!("test", "InnerMap$key"),
            ShapeType::String,
            "key",
            0,
        )
        .with_xml_name("InnerKey");
        static INNER_VAL: Schema<'static> = Schema::new_member(
            shape_id!("test", "InnerMap$value"),
            ShapeType::String,
            "value",
            1,
        )
        .with_xml_name("InnerVal");

        // The outer map's value member, with its target's _KEY/_VALUE
        // chain attached. This is what codegen now emits at the inner
        // `read_map` call site.
        static OUTER_VALUE: Schema<'static> = Schema::new_member(
            shape_id!("test", "OuterMap$value"),
            ShapeType::Map,
            "value",
            1,
        )
        .with_map_members(&INNER_KEY, &INNER_VAL);

        static OUTER_KEY: Schema<'static> = Schema::new_member(
            shape_id!("test", "OuterMap$key"),
            ShapeType::String,
            "key",
            0,
        );
        static OUTER_MAP_SCHEMA: Schema<'static> =
            Schema::new_map(shape_id!("test", "OuterMap"), &OUTER_KEY, &OUTER_VALUE);

        let mut got: Vec<(String, Vec<(String, String)>)> = Vec::new();
        deser
            .read_map(&OUTER_MAP_SCHEMA, &mut |outer_key, d| {
                let mut inner: Vec<(String, String)> = Vec::new();
                d.read_map(&OUTER_VALUE, &mut |inner_key, d2| {
                    let inner_val = d2.read_string(&INNER_VAL)?;
                    inner.push((inner_key, inner_val));
                    Ok(())
                })?;
                got.push((outer_key, inner));
                Ok(())
            })
            .expect("nested map deserialization should succeed");

        assert_eq!(
            got,
            vec![("ok".to_owned(), vec![("ik".to_owned(), "iv".to_owned())])]
        );
    }

    /// List whose member targets a map with `@xmlName` on inner
    /// key/value. Mirrors the serializer's
    /// `list_member_is_inner_map_with_renamed_inner_key_value` wire
    /// format.
    #[test]
    fn read_list_member_is_inner_map_with_renamed_inner_key_value() {
        let xml = b"<items><member><entry><Attr>k1</Attr><Set>v1</Set></entry></member></items>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static INNER_KEY: Schema<'static> = Schema::new_member(
            shape_id!("test", "InnerMap$key"),
            ShapeType::String,
            "key",
            0,
        )
        .with_xml_name("Attr");
        static INNER_VAL: Schema<'static> = Schema::new_member(
            shape_id!("test", "InnerMap$value"),
            ShapeType::String,
            "value",
            1,
        )
        .with_xml_name("Set");

        // The list's member: target shape is InnerMap. Carries the
        // chained inner map _KEY/_VALUE schemas.
        static LIST_ITEM: Schema<'static> = Schema::new_member(
            shape_id!("test", "OuterList$member"),
            ShapeType::Map,
            "member",
            0,
        )
        .with_map_members(&INNER_KEY, &INNER_VAL);

        static OUTER_LIST_SCHEMA: Schema<'static> =
            Schema::new_list(shape_id!("test", "OuterList"), &LIST_ITEM);

        let mut got: Vec<Vec<(String, String)>> = Vec::new();
        deser
            .read_list(&OUTER_LIST_SCHEMA, &mut |d| {
                let mut entries: Vec<(String, String)> = Vec::new();
                d.read_map(&LIST_ITEM, &mut |inner_key, d2| {
                    let inner_val = d2.read_string(&INNER_VAL)?;
                    entries.push((inner_key, inner_val));
                    Ok(())
                })?;
                got.push(entries);
                Ok(())
            })
            .expect("list-of-map deserialization should succeed");

        assert_eq!(got, vec![vec![("k1".to_owned(), "v1".to_owned())]]);
    }
}

/// Tests for the [`ShapeDeserializer::skip_value`] contract.
///
/// XML is the opposite case from JSON and CBOR: `read_struct` delimits each child before
/// dispatching it, and the child's `ScopedDecoder` drop advances the parent's tokenizer
/// past the closing tag. So the override must do nothing. These tests are written so that
/// an implementation which *did* advance would consume a sibling and fail.
#[cfg(test)]
mod skip_value_contract {
    use super::*;
    use aws_smithy_schema::serde::ShapeDeserializer;
    use aws_smithy_schema::{shape_id, Schema, ShapeType};

    static BOUND: Schema<'static> = Schema::new_member(
        shape_id!("test", "Output$bound"),
        ShapeType::String,
        "bound",
        0,
    );
    static BODY: Schema<'static> = Schema::new_member(
        shape_id!("test", "Output$body"),
        ShapeType::String,
        "body",
        1,
    );
    static TAIL: Schema<'static> = Schema::new_member(
        shape_id!("test", "Output$tail"),
        ShapeType::String,
        "tail",
        2,
    );
    static OUTPUT: Schema<'static> = Schema::new_struct(
        shape_id!("test", "Output"),
        ShapeType::Structure,
        &[&BOUND, &BODY, &TAIL],
    );

    /// Reads `OUTPUT` from `xml`, skipping `bound` and collecting the other two members.
    fn skip_bound(xml: &[u8]) -> Result<(Option<String>, Option<String>), SerdeError> {
        let mut deser = XmlDeserializer::new(xml, Arc::new(XmlCodecSettings::default()));
        let (mut body, mut tail) = (None, None);
        deser.read_struct(&OUTPUT, &mut |member, d| {
            match member.member_index() {
                Some(0) => d.skip_value()?,
                Some(1) => body = Some(d.read_string(member)?),
                Some(2) => tail = Some(d.read_string(member)?),
                _ => {}
            }
            Ok(())
        })?;
        Ok((body, tail))
    }

    #[test]
    fn skipping_a_member_does_not_consume_its_siblings() {
        // `bound` sits between the two members we read. An implementation that advanced
        // the tokenizer here would swallow `<body>`.
        let xml = b"<Output><bound>skipped</bound><body>real</body><tail>end</tail></Output>";
        let (body, tail) = skip_bound(xml).unwrap();
        assert_eq!(body.as_deref(), Some("real"));
        assert_eq!(tail.as_deref(), Some("end"));
    }

    #[test]
    fn skipping_works_regardless_of_element_order() {
        for xml in [
            &b"<Output><bound>s</bound><body>real</body><tail>end</tail></Output>"[..],
            &b"<Output><body>real</body><bound>s</bound><tail>end</tail></Output>"[..],
            &b"<Output><body>real</body><tail>end</tail><bound>s</bound></Output>"[..],
        ] {
            let (body, tail) = skip_bound(xml).unwrap();
            assert_eq!(
                (body.as_deref(), tail.as_deref()),
                (Some("real"), Some("end")),
                "failed for {}",
                String::from_utf8_lossy(xml)
            );
        }
    }

    #[test]
    fn skipping_an_aggregate_member_with_nested_content_does_not_consume_siblings() {
        // Here `bound` is a structure, so the parent hands the consumer a sub-slice
        // view. Skipping that must not disturb the outer iteration.
        // Member 0 is aggregate-typed this time, which routes it through
        // `dispatch_subslice`; members 1 and 2 stay scalars. It needs no sub-members of
        // its own because the consumer never reads it.
        static AGG_BOUND: Schema<'static> = Schema::new_member(
            shape_id!("test", "Output$bound"),
            ShapeType::Structure,
            "bound",
            0,
        );
        static AGG_OUTPUT: Schema<'static> = Schema::new_struct(
            shape_id!("test", "Output"),
            ShapeType::Structure,
            &[&AGG_BOUND, &BODY, &TAIL],
        );

        // The skipped element contains a decoy `<body>`. Because the parent delimited the
        // whole `<bound>` element, the decoy must never surface and the outer `<body>`
        // must still be read.
        let xml = b"<Output><bound><inner>x</inner><body>decoy</body></bound>\
                    <body>real</body><tail>end</tail></Output>";
        let mut deser = XmlDeserializer::new(xml, Arc::new(XmlCodecSettings::default()));
        let (mut body, mut tail) = (None, None);
        deser
            .read_struct(&AGG_OUTPUT, &mut |member, d| {
                match member.member_index() {
                    Some(0) => d.skip_value()?,
                    Some(1) => body = Some(d.read_string(member)?),
                    Some(2) => tail = Some(d.read_string(member)?),
                    _ => {}
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(body.as_deref(), Some("real"));
        assert_eq!(tail.as_deref(), Some("end"));
    }

    #[test]
    fn skipping_a_scalar_member_ignores_its_nested_content() {
        let xml = b"<Output><bound><body>decoy</body></bound>\
                    <body>real</body><tail>end</tail></Output>";
        let (body, tail) = skip_bound(xml).unwrap();
        assert_eq!(body.as_deref(), Some("real"));
        assert_eq!(tail.as_deref(), Some("end"));
    }

    #[test]
    fn reading_a_scalar_member_still_rejects_nested_content() {
        let xml = b"<Output><bound><body>decoy</body></bound>\
                    <body>real</body><tail>end</tail></Output>";
        let mut deser = XmlDeserializer::new(xml, Arc::new(XmlCodecSettings::default()));
        let result = deser.read_struct(&OUTPUT, &mut |member, value| {
            value.read_string(member)?;
            Ok(())
        });
        assert!(result.is_err());
    }

    #[test]
    fn skipping_a_self_closing_and_empty_element_is_fine() {
        for xml in [
            &b"<Output><bound/><body>real</body><tail>end</tail></Output>"[..],
            &b"<Output><bound></bound><body>real</body><tail>end</tail></Output>"[..],
        ] {
            let (body, tail) = skip_bound(xml).unwrap();
            assert_eq!(
                (body.as_deref(), tail.as_deref()),
                (Some("real"), Some("end")),
                "failed for {}",
                String::from_utf8_lossy(xml)
            );
        }
    }

    #[test]
    fn skip_value_is_a_no_op_on_a_text_deserializer() {
        // The leaf case: a deserializer holding pre-extracted text. Skipping must neither
        // fail nor consume anything, and must not allocate a discarded document (the
        // trait default would call `read_document`).
        static V: Schema<'static> =
            Schema::new_member(shape_id!("test", "S$v"), ShapeType::String, "v", 0);
        let mut deser =
            XmlDeserializer::new(b"<v>hello</v>", Arc::new(XmlCodecSettings::default()));
        let dynamic: &mut dyn ShapeDeserializer = &mut deser;
        dynamic.skip_value().unwrap();
        // Still readable afterwards, confirming nothing was consumed.
        assert_eq!(deser.read_string(&V).unwrap(), "hello");
    }
}
