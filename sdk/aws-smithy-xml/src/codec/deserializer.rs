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
use aws_smithy_types::{BigDecimal, BigInteger, Blob, DateTime, Document as SmithyDocument};
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
    /// Pre-extracted leaf text. When `Some`, scalar reads consume it
    /// directly without re-parsing `input`; aggregate reads error.
    text: Option<Cow<'a, str>>,
    settings: Arc<XmlCodecSettings>,
    /// Optional schema override consulted by the next aggregate read
    /// (`read_struct` / `read_list` / `read_map`) when codegen passes a
    /// shapeless placeholder schema (e.g. `prelude::DOCUMENT`) for an
    /// inner aggregate. Used to thread the outer aggregate's
    /// `value_schema` / `member_schema` (with its own
    /// `with_map_members`/`with_list_member` chain) into nested reads so
    /// nested element-name overrides (`@xmlName` on inner key/value) can
    /// be honored.
    schema_override: Option<&'static Schema>,
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
            schema_override: None,
            depth: 0,
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
            text: Some(text),
            settings,
            schema_override: None,
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
        Ok(Document::try_from(self.input).unwrap_or_else(|_| Document::new("")))
    }

    /// Extract the leaf text content. If `text` was pre-set, returns it
    /// directly; otherwise parses `input`, navigates to the root element,
    /// and reads its text content.
    fn take_text(&mut self) -> Result<Cow<'a, str>, SerdeError> {
        if let Some(t) = self.text.take() {
            return Ok(t);
        }
        let mut doc = Document::try_from(self.input).unwrap_or_else(|_| Document::new(""));
        let mut root = doc
            .root_element()
            .map_err(|e| SerdeError::custom(e.to_string()))?;
        decode::try_data(&mut root).map_err(|e| SerdeError::custom(e.to_string()))
    }

    /// Run `f` against `self` after temporarily repointing it at a sub-slice
    /// of the parent input. State (input, text, schema_override) is
    /// saved on entry and restored on return so the deserializer can be
    /// reused for sibling dispatches without per-iteration allocation.
    fn dispatch_subslice<R>(
        &mut self,
        sub: &'a [u8],
        schema_override: Option<&'static Schema>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let saved_input = std::mem::replace(&mut self.input, sub);
        let saved_text = self.text.take();
        let saved_override = std::mem::replace(&mut self.schema_override, schema_override);
        let r = f(self);
        self.input = saved_input;
        self.text = saved_text;
        self.schema_override = saved_override;
        r
    }

    /// Run `f` against `self` after temporarily setting pre-extracted leaf
    /// text. State is restored on return.
    fn dispatch_text<R>(&mut self, text: Cow<'a, str>, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_input = std::mem::replace(&mut self.input, b"");
        let saved_text = self.text.replace(text);
        let saved_override = self.schema_override.take();
        let r = f(self);
        self.input = saved_input;
        self.text = saved_text;
        self.schema_override = saved_override;
        r
    }

    /// Resolve a child element name to a member schema by matching against
    /// @xmlName (if present) or member_name.
    fn resolve_member<'s>(schema: &'s Schema, element_name: &str) -> Option<&'s Schema> {
        schema.members().iter().copied().find(|m| {
            if let Some(xml_name) = m.xml_name() {
                xml_name.value() == element_name
            } else {
                m.member_name() == Some(element_name)
            }
        })
    }

    /// Find the byte slice in `input` that contains the element whose local name
    /// pointer `el_local` points into `input`. Uses pointer arithmetic to locate
    /// the `<` before the element name, then scans forward for the matching close
    /// tag with depth tracking. Returns the sub-slice `<tag...>...</tag>`.
    ///
    /// Operates purely on byte slices — `<`, `>`, `/`, and `?` are all single-
    /// byte ASCII characters, so the byte-level scanning we do here is correct
    /// regardless of the multi-byte UTF-8 sequences that may appear in element
    /// content (e.g. attribute values, text nodes containing non-ASCII chars).
    /// Previous versions converted to `&str` and panicked on
    /// `start byte index N is not a char boundary` when a byte-level `pos += 1`
    /// landed inside a multi-byte sequence; sticking to bytes throughout
    /// avoids the issue.
    pub(crate) fn find_element_slice(input: &'a [u8], el_local: &str) -> &'a [u8] {
        // Invariant: `el_local` must be a sub-slice of `input` (typically
        // returned by xmlparser as a borrow into the underlying bytes).
        // The pointer-arithmetic below assumes containment; passing a
        // separately-allocated `String` would compute a meaningless
        // offset. Caught by `.saturating_sub.min` clamping at runtime
        // (so we don't UB) but the result is silently wrong. The assert
        // surfaces the misuse in debug builds.
        debug_assert!(
            {
                let lo = input.as_ptr() as usize;
                let hi = lo + input.len();
                let p = el_local.as_ptr() as usize;
                p >= lo && p + el_local.len() <= hi
            },
            "find_element_slice: el_local must point into input"
        );
        let name_ptr = el_local.as_ptr() as usize;
        let input_start = input.as_ptr() as usize;
        let name_offset = name_ptr.saturating_sub(input_start).min(input.len());

        // The element name is inside the input. Find the `<` immediately
        // preceding it.
        let el_start = input[..name_offset]
            .iter()
            .rposition(|&b| b == b'<')
            .unwrap_or(0);

        // Scan forward, byte by byte, tracking nesting of elements with the
        // same local name. `<`, `>`, `/`, `?` are single-byte ASCII so the
        // byte-level cursor is always at the start of a UTF-8 char.
        let tag_name = el_local.as_bytes();
        let remaining = &input[el_start..];
        let mut depth = 0i32;
        let mut pos = 0;
        while pos < remaining.len() {
            if remaining[pos..].starts_with(b"</") {
                // Close tag — check if it matches our tag name.
                let after_slash = pos + 2;
                if remaining[after_slash..].starts_with(tag_name) {
                    let after_name = after_slash + tag_name.len();
                    if remaining.get(after_name) == Some(&b'>') {
                        depth -= 1;
                        if depth == 0 {
                            let end = el_start + after_name + 1;
                            return &input[el_start..end];
                        }
                    }
                }
                pos = after_slash;
            } else if remaining[pos] == b'<'
                && remaining.get(pos + 1) != Some(&b'/')
                && remaining.get(pos + 1) != Some(&b'?')
            {
                // Open tag — check if self-closing or matches our name.
                if let Some(gt) = remaining[pos..].iter().position(|&b| b == b'>') {
                    let tag_content = &remaining[pos + 1..pos + gt];
                    let is_self_closing = tag_content.last() == Some(&b'/');
                    let opens_our_tag = tag_content.starts_with(tag_name)
                        && tag_content
                            .get(tag_name.len())
                            .is_none_or(|&b| b == b' ' || b == b'>' || b == b'/');
                    if opens_our_tag && is_self_closing && depth == 0 {
                        // The target element is itself self-closing (e.g.
                        // `<Foo/>`) — there is no matching close tag, so the
                        // element slice ends just past this `>`.
                        let end = el_start + pos + gt + 1;
                        return &input[el_start..end];
                    }
                    if opens_our_tag && !is_self_closing {
                        depth += 1;
                    }
                    pos += gt + 1;
                } else {
                    pos += 1;
                }
            } else {
                // Any other byte (text content, attribute byte, multi-byte
                // UTF-8 lead/continuation, etc.) — advance by one byte. This
                // is correct because we never `&str`-index into `remaining`,
                // only byte-slice it, and byte slicing on a `&[u8]` accepts
                // any offset.
                pos += 1;
            }
        }
        // Fallback: return from el_start to end
        &input[el_start..]
    }

    fn resolve_timestamp_format(&self, schema: &Schema) -> TimestampFormat {
        schema
            .timestamp_format()
            .map(|t| match t.format() {
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
            })
            .unwrap_or_else(|| match self.settings.default_timestamp_format() {
                TimestampFormat::DateTime => TimestampFormat::DateTimeWithOffset,
                other => other,
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
            // pointer-containment invariant of `find_element_slice`.
            return Some(XmlDeserializer::find_element_slice(body, local));
        }
    }
    None
}

impl ShapeDeserializer for XmlDeserializer<'_> {
    fn read_struct(
        &mut self,
        schema: &Schema,
        consumer: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
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
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;

            if self.settings.enforce_strictness && self.depth == 1 {
                let expected = schema
                    .xml_name()
                    .map(|t| t.value())
                    .or_else(|| schema.original_name())
                    .or_else(|| schema.member_name())
                    .unwrap_or_else(|| schema.shape_id().shape_name());
                if !root.start_el().matches(expected) {
                    return Err(SerdeError::InvalidInput {
                        message: format!("expected XML root {expected}"),
                    });
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
                // Capture the local element name and find its byte range
                // BEFORE dropping `root` / `doc`, because:
                //   - `find_element_slice`'s pointer-arithmetic invariant
                //     requires `el_local` to be a sub-slice of `input`;
                //     `root.start_el().local()` returns exactly that.
                //   - The owned `String` is only used by `resolve_member`,
                //     after the parser borrows are released. A previous
                //     version of this code passed the owned `String` to
                //     `find_element_slice`, silently producing offset=0;
                //     correct only by happy accident when the input
                //     buffer started with the target element.
                let el_local = root.start_el().local();
                let sub = Self::find_element_slice(input, el_local);
                let local = el_local.to_owned();
                // Release the iterator borrow on `doc` so we can mutate `self`.
                // `root` is a `ScopedDecoder` (whose `Drop` advances the tokenizer
                // past the close tag) and is dropped explicitly. `doc` is a
                // `decode::Document` which has no `Drop` impl; binding to `_`
                // consumes it without firing clippy's `drop_non_drop` lint.
                drop(root);
                let _ = doc;
                if let Some(member) = Self::resolve_member(schema, &local) {
                    self.dispatch_subslice(sub, None, |this| consumer(member, this))?;
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
                        self.dispatch_text(text, |this| consumer(member, this))?;
                    }
                }
            }

            // Track flattened-aggregate members: their wire format is repeated sibling
            // elements that must be accumulated and dispatched as a single read_list /
            // read_map call. Map: member_index -> (member_schema, accumulated XML bytes).
            let mut flattened_groups: std::collections::HashMap<usize, (&Schema, Vec<u8>)> =
                std::collections::HashMap::new();

            // Dispatch child elements.
            while let Some(mut child_scope) = root.next_tag() {
                let local = child_scope.start_el().local().to_owned();
                let Some(member) = Self::resolve_member(schema, &local) else {
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
                    let el_local = child_scope.start_el().local();
                    let sub = Self::find_element_slice(input, el_local);
                    drop(child_scope);
                    self.dispatch_subslice(sub, None, |this| consumer(member, this))?;
                } else if is_aggregate {
                    // Flattened aggregate: capture this sibling's slice; dispatch
                    // the merged group below.
                    let el_local = child_scope.start_el().local();
                    let sub = Self::find_element_slice(input, el_local);
                    drop(child_scope);
                    let idx = member.member_index().unwrap_or(usize::MAX);
                    let entry = flattened_groups
                        .entry(idx)
                        .or_insert_with(|| (member, Vec::new()));
                    entry.1.extend_from_slice(sub);
                } else {
                    let text = decode::try_data(&mut child_scope)
                        .map_err(|e| SerdeError::custom(e.to_string()))?;
                    drop(child_scope);
                    self.dispatch_text(text, |this| consumer(member, this))?;
                }
            }

            // Dispatch each accumulated flattened-aggregate group as a single call.
            // We synthesize a `<__flat>...</__flat>` wrapper so the consumer's
            // `read_list` / `read_map` sees the collected siblings as
            // wrapper-children and iterates them normally. The wrapper buffer is
            // owned locally (lifetime is shorter than `'a`), so we cannot route
            // it through `dispatch_subslice` — keep a fresh deserializer for
            // this case only.
            for (_idx, (member, bytes)) in flattened_groups {
                let mut wrapped = Vec::with_capacity(bytes.len() + 16);
                wrapped.extend_from_slice(b"<__flat>");
                wrapped.extend_from_slice(&bytes);
                wrapped.extend_from_slice(b"</__flat>");
                let mut child_deser = XmlDeserializer::new(&wrapped, self.settings.clone());
                consumer(member, &mut child_deser)?;
            }
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_list(
        &mut self,
        _schema: &Schema,
        consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
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

            // Each child tag is a list item. Provide each item to the consumer
            // by re-pointing `self` at the item's sub-slice via dispatch_subslice.
            // Scalar consumers will navigate to the text via `take_text()`;
            // aggregate consumers (read_list / read_struct / read_map) will
            // descend into the element's content. This keeps the deserializer
            // compatible with both scalar list elements and nested aggregate
            // elements (e.g. list-of-lists, list-of-structs) without per-element
            // type sniffing.
            while let Some(child_scope) = root.next_tag() {
                let el_local = child_scope.start_el().local();
                let sub = Self::find_element_slice(input, el_local);
                drop(child_scope);
                self.dispatch_subslice(sub, None, |this| consumer(this))?;
            }
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_map(
        &mut self,
        schema: &Schema,
        consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
    ) -> Result<(), SerdeError> {
        self.enter_aggregate()?;
        // IIFE: any `?` inside falls through to `leave_aggregate` below
        // (see `read_string_list` for rationale).
        let result = (|| -> Result<(), SerdeError> {
            // If a parent aggregate read installed a `schema_override` on us, it
            // takes priority. Codegen generates inner `read_map` calls reusing
            // the outer `member` schema (closure variable shadowing) — so the
            // arg here is the outer map's schema, not the inner's. The override
            // carries the outer's `_VALUE` schema (which chains the inner map's
            // `_KEY` / `_VALUE`) and is the right one for nested element-name
            // resolution.
            let effective_schema: &Schema =
                self.schema_override.map(|s| s as &Schema).unwrap_or(schema);
            // Once we've read this override, clear it so it doesn't leak to
            // sibling reads on the same deserializer.
            self.schema_override = None;
            let schema = effective_schema;

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
            // For aggregate values, save the value member's `'static` schema so a
            // nested inner aggregate read (which codegen invokes with
            // `prelude::DOCUMENT`) can recover its own `_KEY`/`_VALUE`
            // chain via the `schema_override` parameter on `dispatch_subslice`.
            let value_schema_static = schema.member_static();

            // Each child tag is an entry (e.g. <entry><key>k</key><value>v</value></entry>).
            while let Some(mut entry_scope) = root.next_tag() {
                let mut key: Option<String> = None;
                // For scalar values we capture the text upfront; for aggregate
                // values we capture the element sub-slice. At most one is set.
                let mut value_text: Option<Cow<'_, str>> = None;
                let mut value_slice: Option<&'_ [u8]> = None;
                while let Some(mut field_scope) = entry_scope.next_tag() {
                    let local = field_scope.start_el().local().to_owned();
                    if local == key_name {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        key = Some(text.into_owned());
                    } else if local == value_name {
                        if value_is_aggregate {
                            let el_local = field_scope.start_el().local();
                            let sub = Self::find_element_slice(input, el_local);
                            drop(field_scope);
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
                        self.dispatch_subslice(slice, value_schema_static, |this| {
                            consumer(k, this)
                        })?;
                    } else if let Some(t) = value_text {
                        // Re-borrow t as 'a — the text was extracted from `doc`
                        // which borrows `self.input: &'a [u8]`, so its lifetime
                        // is `'a`.
                        let t: Cow<'_, str> = t;
                        self.dispatch_text(t.into_owned().into(), |this| consumer(k, this))?;
                    }
                }
            }
            Ok(())
        })();
        self.leave_aggregate();
        result
    }

    fn read_boolean(&mut self, _schema: &Schema) -> Result<bool, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(SerdeError::custom(format!("invalid boolean: {other}"))),
        }
    }

    fn read_byte(&mut self, _schema: &Schema) -> Result<i8, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_short(&mut self, _schema: &Schema) -> Result<i16, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_integer(&mut self, _schema: &Schema) -> Result<i32, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_long(&mut self, _schema: &Schema) -> Result<i64, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_float(&mut self, _schema: &Schema) -> Result<f32, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "NaN" => Ok(f32::NAN),
            "Infinity" => Ok(f32::INFINITY),
            "-Infinity" => Ok(f32::NEG_INFINITY),
            _ => text.parse().map_err(|e| SerdeError::custom(format!("{e}"))),
        }
    }

    fn read_double(&mut self, _schema: &Schema) -> Result<f64, SerdeError> {
        let text = self.take_text()?;
        match text.as_ref() {
            "NaN" => Ok(f64::NAN),
            "Infinity" => Ok(f64::INFINITY),
            "-Infinity" => Ok(f64::NEG_INFINITY),
            _ => text.parse().map_err(|e| SerdeError::custom(format!("{e}"))),
        }
    }

    fn read_big_integer(&mut self, _schema: &Schema) -> Result<BigInteger, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_big_decimal(&mut self, _schema: &Schema) -> Result<BigDecimal, SerdeError> {
        let text = self.take_text()?;
        text.parse().map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_string(&mut self, _schema: &Schema) -> Result<String, SerdeError> {
        let text = self.take_text()?;
        Ok(text.into_owned())
    }

    fn read_blob(&mut self, _schema: &Schema) -> Result<Blob, SerdeError> {
        let text = self.take_text()?;
        let bytes = aws_smithy_types::base64::decode(text.as_ref())
            .map_err(|e| SerdeError::custom(format!("{e}")))?;
        Ok(Blob::new(bytes))
    }

    fn read_timestamp(&mut self, schema: &Schema) -> Result<DateTime, SerdeError> {
        let text = self.take_text()?;
        let format = self.resolve_timestamp_format(schema);
        DateTime::from_str(text.as_ref(), format).map_err(|e| SerdeError::custom(format!("{e}")))
    }

    fn read_document(&mut self, _schema: &Schema) -> Result<SmithyDocument, SerdeError> {
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
    //      a save/restore of (`input`, `text`, `schema_override`).
    //
    // The overrides below walk the existing tokenizer once and extract
    // text inline via `decode::try_data`, eliminating all three costs.
    // They preserve the default behavior of accepting any child element
    // name (matching `XmlDeserializer::read_list` / `read_map` which do
    // not validate element names against the schema's expected member
    // name — element-name dispatch is the deserializer's responsibility
    // for structs only).
    //
    // Sparse lists are not routed here: `SchemaGenerator` only emits
    // `read_string_list` / `read_blob_list` / `read_integer_list` /
    // `read_long_list` / `read_string_string_map` for non-sparse element
    // shapes (see SchemaGenerator.kt line ~1497).

    fn read_string_list(&mut self, _schema: &Schema) -> Result<Vec<String>, SerdeError> {
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
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                out.push(text.into_owned());
            }
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_blob_list(&mut self, _schema: &Schema) -> Result<Vec<Blob>, SerdeError> {
        use aws_smithy_types::base64;
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<Blob>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let bytes = base64::decode(text.as_ref())
                    .map_err(|e| SerdeError::custom(format!("invalid base64: {e}")))?;
                out.push(Blob::new(bytes));
            }
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_integer_list(&mut self, _schema: &Schema) -> Result<Vec<i32>, SerdeError> {
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<i32>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let v: i32 = text
                    .parse()
                    .map_err(|e| SerdeError::custom(format!("{e}")))?;
                out.push(v);
            }
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_long_list(&mut self, _schema: &Schema) -> Result<Vec<i64>, SerdeError> {
        self.enter_aggregate()?;
        let result = (|| -> Result<Vec<i64>, SerdeError> {
            let mut doc = self.document()?;
            let mut root = doc
                .root_element()
                .map_err(|e| SerdeError::custom(e.to_string()))?;
            let mut out = Vec::new();
            while let Some(mut child_scope) = root.next_tag() {
                let text = decode::try_data(&mut child_scope)
                    .map_err(|e| SerdeError::custom(e.to_string()))?;
                let v: i64 = text
                    .parse()
                    .map_err(|e| SerdeError::custom(format!("{e}")))?;
                out.push(v);
            }
            Ok(out)
        })();
        self.leave_aggregate();
        result
    }

    fn read_string_string_map(
        &mut self,
        schema: &Schema,
    ) -> Result<std::collections::HashMap<String, String>, SerdeError> {
        // Mirror the schema_override / key-name / value-name resolution
        // used by `read_map` so the override is a behavioral drop-in.
        let effective_schema: &Schema =
            self.schema_override.map(|s| s as &Schema).unwrap_or(schema);
        self.schema_override = None;
        let schema = effective_schema;

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
                let mut k: Option<String> = None;
                let mut v: Option<String> = None;
                while let Some(mut field_scope) = entry_scope.next_tag() {
                    let local = field_scope.start_el().local().to_owned();
                    if local == key_name {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        k = Some(text.into_owned());
                    } else if local == value_name {
                        let text = decode::try_data(&mut field_scope)
                            .map_err(|e| SerdeError::custom(e.to_string()))?;
                        v = Some(text.into_owned());
                    }
                }
                if let (Some(k), Some(v)) = (k, v) {
                    out.insert(k, v);
                }
            }
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

    fn container_size(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::{shape_id, Schema, ShapeType};

    static STRING_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "S$v"), ShapeType::String, "v", 0);

    #[test]
    fn strict_roots_only_validate_document_boundary() {
        static CHILD: Schema = Schema::new_struct(
            aws_smithy_schema::shape_id!("test", "Child"),
            aws_smithy_schema::ShapeType::Structure,
            &[],
        );
        static RENAMED: Schema = Schema::new_member(
            aws_smithy_schema::shape_id!("test", "Root", "child"),
            aws_smithy_schema::ShapeType::Structure,
            "child",
            0,
        )
        .with_xml_name("Renamed");
        static ROOT: Schema = Schema::new_struct(
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
        static V_MEMBER: Schema =
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

    static NAME_MEMBER: Schema = Schema::new_member(
        shape_id!("test", "Person$name"),
        ShapeType::String,
        "name",
        0,
    );
    static AGE_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "Person$age"), ShapeType::String, "age", 1);
    static RENAMED_MEMBER: Schema = Schema::new_member(
        shape_id!("test", "Person$nick"),
        ShapeType::String,
        "nick",
        2,
    )
    .with_xml_name("Nickname");

    static PERSON_SCHEMA: Schema = Schema::new_struct(
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

    static ATTR_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "X$id"), ShapeType::String, "id", 0)
            .with_xml_attribute();
    static ELEM_MEMBER: Schema =
        Schema::new_member(shape_id!("test", "X$name"), ShapeType::String, "name", 1);

    static X_SCHEMA: Schema = Schema::new_struct(
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

        static LIST_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

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

        static MAP_KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0);
        static MAP_SCHEMA: Schema = Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

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

        static MAP_KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0)
                .with_xml_name("Attribute");
        static MAP_VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0)
                .with_xml_name("Setting");
        static MAP_SCHEMA: Schema = Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

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

        static S_NAME: Schema =
            Schema::new_member(shape_id!("test", "S$name"), ShapeType::String, "name", 0);
        static S_ITEMS: Schema =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 1)
                .with_xml_flattened()
                .with_xml_name("item");
        static S_SCHEMA: Schema = Schema::new_struct(
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

    #[test]
    fn read_struct_flattened_list_intermixed() {
        // Flattened list elements intermixed with other members.
        let xml = b"<S><item>x</item><name>n</name><item>y</item></S>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static S_NAME: Schema =
            Schema::new_member(shape_id!("test", "S$name"), ShapeType::String, "name", 0);
        static S_ITEMS: Schema =
            Schema::new_member(shape_id!("test", "S$items"), ShapeType::List, "items", 1)
                .with_xml_flattened()
                .with_xml_name("item");
        static S_SCHEMA: Schema = Schema::new_struct(
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
    // on `deserialize_with_response` skipping the body deserializer
    // entirely (codegen passes `_deserializer`), so they never reach
    // `read_struct`. Operations that DO have body-bound members and
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
        static R_MEMBER_SELF: Schema =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema = Schema::new_struct(
            shape_id!("test", "R"),
            ShapeType::Structure,
            &[&R_MEMBER_SELF],
        );

        // Tighter limit so the test is fast.
        let max = 4;
        let xml = nested_struct_xml(max + 2);
        let mut deser = XmlDeserializer::new(&xml, settings_with_max_depth(max));

        // Recursive consumer: each entry into `<r>` calls read_struct again.
        fn consume(_m: &Schema, d: &mut dyn ShapeDeserializer) -> Result<(), SerdeError> {
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
        static R_MEMBER_SELF: Schema =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema = Schema::new_struct(
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
            _m: &Schema,
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
        static L_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "L$member"), ShapeType::List, "member", 0);
        static L_SCHEMA: Schema = Schema::new_list(shape_id!("test", "L"), &L_MEMBER);

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
        static R_MEMBER_SELF: Schema =
            Schema::new_member(shape_id!("test", "R$r"), ShapeType::Structure, "r", 0);
        static R_SCHEMA: Schema = Schema::new_struct(
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
        static L_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static L_SCHEMA: Schema = Schema::new_list(shape_id!("test", "L"), &L_MEMBER);

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
        // earlier version called `find_element_slice` with a heap-allocated
        // `String` instead of a `&str` borrowing from `input`. The
        // pointer-arithmetic invariant broke; only the
        // `.saturating_sub.min` clamping prevented UB. The result was
        // silently correct only when the target element happened to start
        // at offset 0 of the input.
        //
        // Constructing a payload with an XML prolog (so the element is NOT
        // at offset 0) verifies the fixed code passes a real sub-slice of
        // `input` to `find_element_slice`. The `debug_assert!` in
        // `find_element_slice` would also fire under the old code in
        // debug builds.
        static MEMBER: Schema = Schema::new_member(
            shape_id!("test", "U$location"),
            ShapeType::String,
            "LocationConstraint",
            0,
        );
        static SCHEMA: Schema =
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

    /// Regression test for a UTF-8 char-boundary panic in `find_element_slice`.
    ///
    /// Found via `schema_xml_roundtrip` fuzz target on input
    /// `StringStringMap([("Б", "")])`. The serialized payload contains a
    /// Cyrillic `Б` (UTF-8 bytes `0xD0 0x91`) inside a map-key element. The
    /// previous implementation operated on `&str` and advanced its scan
    /// cursor by one byte per non-`<` character, which landed mid-char on
    /// the second byte of `Б` and panicked with
    /// `start byte index N is not a char boundary`. The fix moves all
    /// scanning to byte slices since `<`, `>`, `/`, and `?` are single-byte
    /// ASCII and the multi-byte content is opaque to the search.
    #[test]
    fn find_element_slice_handles_multibyte_utf8() {
        // Build a struct-with-map XML payload containing Cyrillic text in a
        // map-key element. The exact wire form matches what `XmlSerializer`
        // emits for `StringStringMap([("Б", "")])` wrapped in
        // `WRAPPER_STRING_STRING_MAP_SCHEMA`.
        static KEY_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "Wrapper$k"), ShapeType::Map, "entries", 0);
        static MAP_SCHEMA: Schema = Schema::new_map(
            shape_id!("test", "MyMap"),
            &aws_smithy_schema::prelude::STRING,
            &aws_smithy_schema::prelude::STRING,
        );
        static WRAPPER: Schema = Schema::new_struct(
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

        static LIST_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

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

        static LIST_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "L$member"),
            ShapeType::String,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "L"), &LIST_MEMBER);

        let out = deser.read_string_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec!["x".to_owned(), "y".to_owned()]);
    }

    #[test]
    fn read_blob_list_helper() {
        // Each element's text is base64-decoded into a Blob.
        let xml = b"<blobs><member>aGVsbG8=</member><member>d29ybGQ=</member></blobs>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "B$member"), ShapeType::Blob, "member", 0);
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "B"), &LIST_MEMBER);

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

        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "B$member"), ShapeType::Blob, "member", 0);
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "B"), &LIST_MEMBER);

        let err = deser.read_blob_list(&LIST_SCHEMA).unwrap_err();
        assert!(format!("{err}").contains("base64"));
    }

    #[test]
    fn read_integer_list_helper() {
        let xml = b"<nums><member>1</member><member>-42</member><member>0</member></nums>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema = Schema::new_member(
            shape_id!("test", "I$member"),
            ShapeType::Integer,
            "member",
            0,
        );
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "I"), &LIST_MEMBER);

        let out = deser.read_integer_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec![1i32, -42, 0]);
    }

    #[test]
    fn read_long_list_helper() {
        let xml = b"<nums><member>9223372036854775807</member><member>-1</member></nums>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static LIST_MEMBER: Schema =
            Schema::new_member(shape_id!("test", "Lo$member"), ShapeType::Long, "member", 0);
        static LIST_SCHEMA: Schema = Schema::new_list(shape_id!("test", "Lo"), &LIST_MEMBER);

        let out = deser.read_long_list(&LIST_SCHEMA).unwrap();
        assert_eq!(out, vec![i64::MAX, -1]);
    }

    #[test]
    fn read_string_string_map_helper() {
        let xml = b"<m><entry><key>a</key><value>1</value></entry>\
                    <entry><key>b</key><value>2</value></entry></m>";
        let settings = Arc::new(XmlCodecSettings::default());
        let mut deser = XmlDeserializer::new(xml, settings);

        static MAP_KEY: Schema =
            Schema::new_member(shape_id!("test", "M$key"), ShapeType::String, "key", 0);
        static MAP_VALUE: Schema =
            Schema::new_member(shape_id!("test", "M$value"), ShapeType::String, "value", 0);
        static MAP_SCHEMA: Schema = Schema::new_map(shape_id!("test", "M"), &MAP_KEY, &MAP_VALUE);

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

        static MAP_KEY: Schema =
            Schema::new_member(shape_id!("test", "M2$key"), ShapeType::String, "key", 0)
                .with_xml_name("K");
        static MAP_VALUE: Schema =
            Schema::new_member(shape_id!("test", "M2$value"), ShapeType::String, "value", 0)
                .with_xml_name("V");
        static MAP_SCHEMA: Schema = Schema::new_map(shape_id!("test", "M2"), &MAP_KEY, &MAP_VALUE);

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
    // for aggregate sub-trees via `find_element_slice` and `dispatch_subslice`,
    // so neither bug should reproduce — these tests lock that in.
    #[test]
    fn nested_struct_three_levels_deep() {
        static LEAF: Schema =
            Schema::new_member(shape_id!("t", "Inner"), ShapeType::String, "Leaf", 0);
        static INNER_SCHEMA: Schema =
            Schema::new_struct(shape_id!("t", "Inner"), ShapeType::Structure, &[&LEAF]);
        static INNER_MEMBER: Schema =
            Schema::new_member(shape_id!("t", "Middle"), ShapeType::Structure, "Inner", 0);
        static MIDDLE_SCHEMA: Schema = Schema::new_struct(
            shape_id!("t", "Middle"),
            ShapeType::Structure,
            &[&INNER_MEMBER],
        );
        static MIDDLE_MEMBER: Schema =
            Schema::new_member(shape_id!("t", "Outer"), ShapeType::Structure, "Middle", 0);
        static OUTER_SCHEMA: Schema = Schema::new_struct(
            shape_id!("t", "Outer"),
            ShapeType::Structure,
            &[&MIDDLE_MEMBER],
        );
        static OUTER_MEMBER: Schema =
            Schema::new_member(shape_id!("t", "Root"), ShapeType::Structure, "Outer", 0);
        static ROOT: Schema = Schema::new_struct(
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
        static VALUE: Schema =
            Schema::new_member(shape_id!("t", "Body"), ShapeType::String, "value", 0);
        static BODY_SCHEMA: Schema =
            Schema::new_struct(shape_id!("t", "Body"), ShapeType::Structure, &[&VALUE]);
        static PAYLOAD_MEMBER: Schema = Schema::new_member(
            shape_id!("t", "Envelope"),
            ShapeType::Structure,
            "payload",
            0,
        );
        static ENVELOPE_SCHEMA: Schema = Schema::new_struct(
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
        static KEY: Schema =
            Schema::new_member(shape_id!("t", "M$key"), ShapeType::String, "key", 0);
        static VALUE: Schema =
            Schema::new_member(shape_id!("t", "M$value"), ShapeType::String, "value", 1);
        static MAP: Schema = Schema::new_map(shape_id!("t", "M"), &KEY, &VALUE);

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
}
