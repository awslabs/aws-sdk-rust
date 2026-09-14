/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Normative round-trip tests from the
//! Document Type and Type Registries SEP §"Test Cases", adapted to
//! smithy-rs's typed runtime.
//!
//! For every test data set, four representations are constructed:
//!
//! 1. `canonical_serialization` — the JSON wire bytes
//! 2. `canonical_data_object` — the same data as a typed `OmniWidget`
//! 3. `document_from_serialization` — a [`Document`] obtained by
//!    parsing `canonical_serialization` (carries protocol settings)
//! 4. `document_from_data_object` — a [`Document`] obtained by
//!    `Document::from_struct(&SCHEMA, &canonical_data_object)`
//!    (protocol-agnostic per SEP §Document Types rule 8)
//!
//! And six assertions are verified:
//!
//! 1. data object from canonical_serialization == canonical_data_object
//!    (setup — also confirms the deserializer is consistent)
//! 2. data object from document_from_data_object == canonical_data_object
//! 3. data object from document_from_serialization == canonical_data_object
//! 4. serialize(canonical_data_object) == canonical_serialization
//! 5. serialize(document_from_serialization) via write_document ==
//!    canonical_serialization
//! 6. serialize(document_from_data_object) via write_document ==
//!    canonical_serialization
//!
//! Coverage scope is intentionally bounded by the SEP's design: every
//! timestamp member here uses the codec's *default* format
//! (`epoch-seconds`). The SEP allows mixing per-member
//! `@timestampFormat` overrides on the wire (the deserialization path
//! handles that) but a [`Document`] produced via
//! `Document::from_struct(...)` is protocol-agnostic — it carries no
//! per-member format trait — so a Document constructed from a struct
//! with mixed-format timestamps cannot round-trip through assertion 6
//! without losing format info.
//!
//! Document interface coverage is via the simple types and aggregates
//! (string, blob, number, boolean, null, list, map, nested struct).

use std::collections::HashMap;

use aws_smithy_json::codec::JsonCodec;
use aws_smithy_json::codec::JsonCodecSettings;
use aws_smithy_schema::codec::Codec;
use aws_smithy_schema::document::DiscriminatedDocumentExt;
use aws_smithy_schema::prelude;
use aws_smithy_schema::serde::{
    SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer,
};
use aws_smithy_schema::{shape_id, Schema, ShapeId, ShapeType};
use aws_smithy_types::{Blob, DateTime, DiscriminatedDocument, Document};

// -- Test fixture: `OmniWidget` schema and typed shape ----------------

const OMNI_WIDGET_ID: ShapeId<'static> = shape_id!("smithy.example", "OmniWidget");

static M_VALUE_STRING: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_string"),
    ShapeType::String,
    "value_string",
    0,
);

static M_VALUE_BLOB: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_blob"),
    ShapeType::Blob,
    "value_blob",
    1,
);

static M_VALUE_TIMESTAMP_DEFAULT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_timestamp_default"),
    ShapeType::Timestamp,
    "value_timestamp_default",
    2,
);

static M_VALUE_LIST_STRINGS: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_list_strings"),
    ShapeType::List,
    "value_list_strings",
    3,
);

static M_VALUE_MAP_STRINGS: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_map_strings"),
    ShapeType::Map,
    "value_map_strings",
    4,
);

static M_VALUE_STRUCT: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_struct"),
    ShapeType::Structure,
    "value_struct",
    5,
);

// Sparse list member: a list whose `member` is itself sparse (i.e.
// `null` entries are valid values, not absent values). The schema
// itself doesn't carry a sparse flag — `@sparse` is handled at the
// call site via `write_null` / `is_null`. We pick a member-element
// schema of `prelude::STRING` for the same reason as the dense list
// above.
static M_VALUE_SPARSE_LIST: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "OmniWidget", "value_sparse_list"),
    ShapeType::List,
    "value_sparse_list",
    6,
);

static OMNI_WIDGET_SCHEMA: Schema<'static> = Schema::new_struct(
    OMNI_WIDGET_ID,
    ShapeType::Structure,
    &[
        &M_VALUE_STRING,
        &M_VALUE_BLOB,
        &M_VALUE_TIMESTAMP_DEFAULT,
        &M_VALUE_LIST_STRINGS,
        &M_VALUE_MAP_STRINGS,
        &M_VALUE_STRUCT,
        &M_VALUE_SPARSE_LIST,
    ],
);

// Nested struct for the `value_struct` member.
const NESTED_ID: ShapeId<'static> = shape_id!("smithy.example", "Nested");

static M_INNER_STRING: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "Nested", "inner_string"),
    ShapeType::String,
    "inner_string",
    0,
);

static NESTED_SCHEMA: Schema<'static> =
    Schema::new_struct(NESTED_ID, ShapeType::Structure, &[&M_INNER_STRING]);

#[derive(Debug, Default, Clone, PartialEq)]
struct OmniWidget {
    value_string: Option<String>,
    value_blob: Option<Blob>,
    value_timestamp_default: Option<DateTime>,
    value_list_strings: Option<Vec<String>>,
    // The Smithy data model maps to a `HashMap<String, String>` here, but the
    // typed shape uses `Vec<(K, V)>` so canonical wire bytes are deterministic
    // for the round-trip assertion below — the user-data path goes through
    // serde with `String` values, where the reader produces a HashMap whose
    // iteration order is unspecified. The deserialize side normalizes the read
    // HashMap into a sorted Vec so equality comparisons stay stable.
    value_map_strings: Option<Vec<(String, String)>>,
    value_struct: Option<Nested>,
    // Sparse list: a `null` entry in the wire form means a present
    // value that happens to be null (Some(None) on the typed side
    // would be the model, but flattening to `Option<Vec<Option<T>>>`
    // matches how Smithy code generation emits sparse list members
    // today). Outer Option = "is the field set at all?", inner Option
    // = "is this entry null?".
    value_sparse_list: Option<Vec<Option<String>>>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Nested {
    inner_string: Option<String>,
}

impl SerializableStruct for OmniWidget {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        if let Some(v) = &self.value_string {
            ser.write_string(&M_VALUE_STRING, v)?;
        }
        if let Some(v) = &self.value_blob {
            ser.write_blob(&M_VALUE_BLOB, v.clone())?;
        }
        if let Some(v) = self.value_timestamp_default {
            ser.write_timestamp(&M_VALUE_TIMESTAMP_DEFAULT, &v)?;
        }
        if let Some(items) = &self.value_list_strings {
            ser.write_list(&M_VALUE_LIST_STRINGS, &|inner| {
                for s in items {
                    inner.write_string(&prelude::STRING, s)?;
                }
                Ok(())
            })?;
        }
        if let Some(entries) = &self.value_map_strings {
            ser.write_map(&M_VALUE_MAP_STRINGS, &|inner| {
                for (k, v) in entries {
                    inner.write_string(&prelude::STRING, k)?;
                    inner.write_string(&prelude::STRING, v)?;
                }
                Ok(())
            })?;
        }
        if let Some(nested) = &self.value_struct {
            ser.write_struct(&M_VALUE_STRUCT, nested)?;
        }
        if let Some(items) = &self.value_sparse_list {
            ser.write_list(&M_VALUE_SPARSE_LIST, &|inner| {
                for entry in items {
                    match entry {
                        Some(s) => inner.write_string(&prelude::STRING, s)?,
                        // Sparse-list null is written via `write_null`
                        // — the wire form must include explicit `null`
                        // entries (not be elided as if absent).
                        None => inner.write_null(&prelude::STRING)?,
                    }
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

impl SerializableStruct for Nested {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        if let Some(v) = &self.inner_string {
            ser.write_string(&M_INNER_STRING, v)?;
        }
        Ok(())
    }
}

impl OmniWidget {
    fn deserialize(deser: &mut dyn ShapeDeserializer) -> Result<Self, SerdeError> {
        let mut out = OmniWidget::default();
        deser.read_struct(&OMNI_WIDGET_SCHEMA, &mut |member, sub| {
            match member.member_index() {
                Some(0) => out.value_string = Some(sub.read_string(member)?),
                Some(1) => out.value_blob = Some(sub.read_blob(member)?),
                Some(2) => out.value_timestamp_default = Some(sub.read_timestamp(member)?),
                Some(3) => {
                    let mut items = Vec::new();
                    sub.read_list(member, &mut |item| {
                        items.push(item.read_string(&prelude::STRING)?);
                        Ok(())
                    })?;
                    out.value_list_strings = Some(items);
                }
                Some(4) => {
                    let mut h: HashMap<String, String> = HashMap::new();
                    sub.read_map(member, &mut |k, v| {
                        h.insert(k, v.read_string(&prelude::STRING)?);
                        Ok(())
                    })?;
                    let mut entries: Vec<(String, String)> = h.into_iter().collect();
                    entries.sort_by(|a, b| a.0.cmp(&b.0));
                    out.value_map_strings = Some(entries);
                }
                Some(5) => {
                    out.value_struct = Some(Nested::deserialize(sub)?);
                }
                Some(6) => {
                    let mut items: Vec<Option<String>> = Vec::new();
                    sub.read_list(member, &mut |item| {
                        // `is_null()` peeks; it does NOT consume the
                        // `null` literal. For the typed-cursor
                        // deserializer (`JsonDeserializer`), failing to
                        // advance leaves the cursor on `null` and the
                        // outer `read_list` loop spins forever (it
                        // treats commas as whitespace, so once we're
                        // past the comma we re-see the same `null`).
                        // Call `read_null()` to advance past the four
                        // literal bytes.
                        //
                        // This is *not* needed for the document-walking
                        // deserializer in
                        // `aws-smithy-schema/src/schema/document/round_trips.rs`
                        // because tree iteration advances on its own.
                        if item.is_null() {
                            item.read_null()?;
                            items.push(None);
                        } else {
                            items.push(Some(item.read_string(&prelude::STRING)?));
                        }
                        Ok(())
                    })?;
                    out.value_sparse_list = Some(items);
                }
                _ => {}
            }
            Ok(())
        })?;
        Ok(out)
    }
}

impl Nested {
    fn deserialize(deser: &mut dyn ShapeDeserializer) -> Result<Self, SerdeError> {
        let mut out = Nested::default();
        deser.read_struct(&NESTED_SCHEMA, &mut |member, sub| {
            if member.member_index() == Some(0) {
                out.inner_string = Some(sub.read_string(member)?);
            }
            Ok(())
        })?;
        Ok(out)
    }
}

// -- The 6-assertion harness ------------------------------------------

/// Runs the SEP §"Test Cases" 6-assertion matrix on `value`.
///
/// Returns the canonical JSON bytes for inspection by callers that want
/// to additionally assert on the wire form.
fn run_sep_assertions(label: &str, value: OmniWidget) -> Vec<u8> {
    let codec = JsonCodec::default();

    // --- Build the four representations -----------------------------

    // (a) canonical_serialization: serialize the typed value through
    //     the JSON codec's `ShapeSerializer`.
    let canonical_serialization: Vec<u8> = {
        let mut ser = codec.create_serializer();
        ser.write_struct(&OMNI_WIDGET_SCHEMA, &value)
            .unwrap_or_else(|e| panic!("[{label}] write_struct failed: {e:?}"));
        ser.finish()
    };

    // (b) canonical_data_object: round-trip the canonical bytes back
    //     through the deserializer to get the typed value the SEP
    //     considers the source of truth. Per the SEP this also serves
    //     as the implicit "assertion 1": parsing the canonical bytes
    //     yields the typed value.
    let canonical_data_object: OmniWidget = {
        let mut deser = codec.create_deserializer(&canonical_serialization);
        OmniWidget::deserialize(&mut deser)
            .unwrap_or_else(|e| panic!("[{label}] OmniWidget::deserialize failed: {e:?}"))
    };
    assert_eq!(
        canonical_data_object, value,
        "[{label}] assertion 1: round-tripping canonical_serialization \
         must reproduce the original typed value",
    );

    // (c) document_from_serialization: parse the canonical bytes into
    //     a DiscriminatedDocument. This is the "deserialize-side"
    //     wrapper — it carries `JsonDocumentSettings` so
    //     format-aware coercion (e.g. base64 string → blob) works on
    //     the read side. No `__type` is present in the canonical
    //     bytes, so the discriminator slot remains `None`.
    let document_from_serialization: DiscriminatedDocument = {
        let mut deser = codec.create_deserializer(&canonical_serialization);
        // The trait method `read_document` returns plain `Document`;
        // the inherent `read_discriminated_document` is the one that
        // attaches settings and lifts a top-level `__type` field if
        // present. We use the inherent method because the SEP test
        // matrix's read-side expectations match the discriminated-
        // document semantics (settings attached, discriminator-aware).
        deser
            .read_discriminated_document()
            .unwrap_or_else(|e| panic!("[{label}] read_discriminated_document failed: {e:?}"))
    };

    // (d) document_from_data_object: produce a DiscriminatedDocument
    //     from the typed value. This is the "serialize-side" wrapper —
    //     protocol-agnostic per SEP §Document Types rule 8. The
    //     discriminator slot is populated with the FQN of the
    //     OmniWidget schema.
    let document_from_data_object: DiscriminatedDocument =
        DiscriminatedDocument::from_struct(&OMNI_WIDGET_SCHEMA, &canonical_data_object)
            .unwrap_or_else(|e| {
                panic!("[{label}] DiscriminatedDocument::from_struct failed: {e:?}")
            });

    // --- Run the 6 assertions ---------------------------------------

    // Assertion 2: data object from document_from_data_object ==
    //              canonical_data_object.
    let typed_from_doc_from_data: OmniWidget = document_from_data_object
        .as_shape(OmniWidget::deserialize)
        .unwrap_or_else(|e| panic!("[{label}] as_shape on document_from_data_object: {e:?}"));
    assert_eq!(
        typed_from_doc_from_data, canonical_data_object,
        "[{label}] assertion 2: typed shape recovered from \
         document_from_data_object must equal canonical_data_object",
    );

    // Assertion 3: data object from document_from_serialization ==
    //              canonical_data_object.
    let typed_from_doc_from_ser: OmniWidget = document_from_serialization
        .as_shape(OmniWidget::deserialize)
        .unwrap_or_else(|e| panic!("[{label}] as_shape on document_from_serialization: {e:?}"));
    assert_eq!(
        typed_from_doc_from_ser, canonical_data_object,
        "[{label}] assertion 3: typed shape recovered from \
         document_from_serialization must equal canonical_data_object",
    );

    // Assertion 4: serialize(canonical_data_object) ==
    //              canonical_serialization. Already implicit via the
    //              construction of canonical_serialization, but
    //              re-verify here so the assertion is explicit and
    //              self-documenting.
    let reser_typed: Vec<u8> = {
        let mut ser = codec.create_serializer();
        ser.write_struct(&OMNI_WIDGET_SCHEMA, &canonical_data_object)
            .unwrap_or_else(|e| panic!("[{label}] re-serialize: {e:?}"));
        ser.finish()
    };
    assert_eq!(
        reser_typed, canonical_serialization,
        "[{label}] assertion 4: re-serializing canonical_data_object \
         must reproduce canonical_serialization (typed-shape \
         serialization is deterministic)",
    );

    // Assertion 5: serialize(document_from_serialization) via
    //              write_document is *semantically* equivalent to
    //              canonical_serialization.
    //
    // Byte-for-byte comparison would require a `Document` whose map
    // representation preserves insertion order. The current
    // implementation uses `HashMap` — so when canonical bytes get
    // parsed into a Document and then re-serialized, the member order
    // can shift. Parse both as `serde_json::Value` and compare
    // structurally — the SEP's intent is "produces equivalent JSON",
    // not "produces byte-identical JSON".
    let reser_doc_from_ser: Vec<u8> = {
        let mut ser = codec.create_serializer();
        // `write_document` takes a plain `&Document` (no
        // discriminator); extract via `.document()` from the wrapper.
        // The wrapper's discriminator slot is empty here anyway —
        // canonical_serialization doesn't carry `__type` — so this is
        // semantically equivalent to using `write_discriminated_document`.
        ser.write_document(&prelude::DOCUMENT, document_from_serialization.document())
            .unwrap_or_else(|e| panic!("[{label}] write_document on doc_from_ser: {e:?}"));
        ser.finish()
    };
    assert_json_equivalent(
        label,
        "assertion 5",
        &reser_doc_from_ser,
        &canonical_serialization,
        "serializing document_from_serialization via write_document \
         must produce JSON semantically equivalent to canonical_serialization",
    );

    // Assertion 6: serialize(document_from_data_object) via
    //              write_document is *semantically* equivalent to
    //              canonical_serialization.
    //
    // `DiscriminatedDocument::from_struct` attaches the schema's shape
    // ID as the *wrapper's* discriminator (the SEP "typed Document"
    // model — a Document derived from a typed shape carries the
    // typed-shape identity). Only the top-level wrapper carries a
    // discriminator; nested struct nodes in the data tree do not.
    //
    // The trait method `write_document` takes a plain `&Document`
    // (data-only, no discriminator emission). Calling it on the
    // wrapper's `.document()` view produces canonical bytes without
    // `__type`. The discriminator-emission contract is exercised by
    // the separate `sep_write_document_emits_type_discriminator`
    // test below via the inherent `write_discriminated_document`
    // method.
    let reser_doc_from_data: Vec<u8> = {
        let mut ser = codec.create_serializer();
        ser.write_document(&prelude::DOCUMENT, document_from_data_object.document())
            .unwrap_or_else(|e| panic!("[{label}] write_document on doc_from_data: {e:?}"));
        ser.finish()
    };
    assert_json_equivalent(
        label,
        "assertion 6",
        &reser_doc_from_data,
        &canonical_serialization,
        "serializing document_from_data_object via write_document \
         (which doesn't emit `__type`) must produce JSON semantically \
         equivalent to canonical_serialization",
    );

    canonical_serialization
}

/// Asserts that two JSON byte slices are structurally equivalent (same
/// values, regardless of object key order). Used for the assertions
/// that serialize via the Document path, where `HashMap` iteration
/// order makes byte-for-byte comparison non-deterministic.
fn assert_json_equivalent(
    label: &str,
    assertion_name: &str,
    actual: &[u8],
    expected: &[u8],
    description: &str,
) {
    let actual_value: serde_json::Value = serde_json::from_slice(actual).unwrap_or_else(|e| {
        panic!(
            "[{label}] {assertion_name}: actual bytes are not valid JSON: {e}\nactual: {}",
            String::from_utf8_lossy(actual),
        )
    });
    let expected_value: serde_json::Value = serde_json::from_slice(expected).unwrap_or_else(|e| {
        panic!(
            "[{label}] {assertion_name}: expected bytes are not valid JSON: {e}\nexpected: {}",
            String::from_utf8_lossy(expected),
        )
    });
    assert_eq!(
        actual_value,
        expected_value,
        "[{label}] {assertion_name}: {description}\n\
         actual:   {}\n\
         expected: {}",
        String::from_utf8_lossy(actual),
        String::from_utf8_lossy(expected),
    );
}

// -- Test cases -------------------------------------------------------

#[test]
fn sep_round_trip_empty() {
    // All members `None`: an empty struct serializes to `{}` and
    // round-trips through every representation cleanly. Verifies the
    // baseline shape framing.
    let value = OmniWidget::default();
    let bytes = run_sep_assertions("empty", value);
    assert_eq!(
        bytes, b"{}",
        "empty OmniWidget must serialize to the empty JSON object"
    );
}

#[test]
fn sep_round_trip_string_only() {
    // Plain string scalar — exercises the `write_string` /
    // `read_string` path for both typed-shape and document modes.
    let value = OmniWidget {
        value_string: Some("hello".to_owned()),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("string_only", value);
    assert_eq!(bytes, br#"{"value_string":"hello"}"#);
}

#[test]
fn sep_round_trip_blob_only() {
    // Blob — verifies that base64 round-trips through every path.
    // `b"abcd"` encodes as `YWJjZA==`. Critically, `write_document`
    // on a `Document::Blob` must emit the same base64 string the
    // typed-shape `write_blob` would; this is the assertion 6 case
    // for the extended variants.
    let value = OmniWidget {
        value_blob: Some(Blob::new(b"abcd".to_vec())),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("blob_only", value);
    assert_eq!(bytes, br#"{"value_blob":"YWJjZA=="}"#);
}

#[test]
fn sep_round_trip_timestamp_default_format() {
    // Timestamp at codec-default format (`epoch-seconds`). Without a
    // member-level `@timestampFormat`, both the typed-shape path and
    // the `write_document` path resolve to the same format —
    // `JsonSerializer::write_timestamp` reads the codec default when
    // the member schema has no override, and `write_document` always
    // uses the codec default since the document carries no per-member
    // schema info.
    //
    // The SEP example uses 1970-01-01T00:00:00Z (epoch zero) for this
    // case; we match it.
    let value = OmniWidget {
        value_timestamp_default: Some(DateTime::from_secs(0)),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("timestamp_default_format", value);
    assert_eq!(bytes, br#"{"value_timestamp_default":0}"#);
}

#[test]
fn sep_round_trip_list_strings() {
    // List aggregate of strings — exercises `write_list` /
    // `read_list` framing for both modes.
    let value = OmniWidget {
        value_list_strings: Some(vec!["red".to_owned(), "blue".to_owned()]),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("list_strings", value);
    assert_eq!(bytes, br#"{"value_list_strings":["red","blue"]}"#);
}

#[test]
fn sep_round_trip_map_strings() {
    // Map aggregate. Iteration order of the typed `Vec<(K,V)>` is
    // declaration order, which yields a deterministic canonical wire
    // form. The deserialize side normalizes the resulting HashMap into
    // a sorted Vec to keep equality comparisons stable.
    let value = OmniWidget {
        value_map_strings: Some(vec![
            ("alpha".to_owned(), "1".to_owned()),
            ("beta".to_owned(), "2".to_owned()),
        ]),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("map_strings", value);
    assert_eq!(bytes, br#"{"value_map_strings":{"alpha":"1","beta":"2"}}"#);
}

#[test]
fn sep_round_trip_nested_struct() {
    // Nested structure — a struct member whose target is itself a
    // structure shape. Verifies that the recursive `Document` walk
    // and `Document::from_struct` produce identical wire forms for a
    // multi-level shape graph.
    let value = OmniWidget {
        value_struct: Some(Nested {
            inner_string: Some("nested!".to_owned()),
        }),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("nested_struct", value);
    assert_eq!(bytes, br#"{"value_struct":{"inner_string":"nested!"}}"#);
}

#[test]
fn sep_round_trip_sparse_list_with_nulls() {
    // Sparse list whose wire form contains explicit `null` entries
    // intermixed with strings. Exercises:
    //   - typed-shape `write_null` framing inside `write_list`
    //   - typed-shape `is_null` detection inside `read_list`
    //   - `Document::from_struct` walking a list whose elements
    //     include null (the resulting Document tree has
    //     `Document::Null` for those positions)
    //   - `write_document` recursing into a list containing
    //     `Document::Null` entries (per the SEP, Document::Null
    //     serializes as the JSON literal `null`)
    let value = OmniWidget {
        value_sparse_list: Some(vec![
            Some("alpha".to_owned()),
            None,
            Some("gamma".to_owned()),
            None,
        ]),
        ..OmniWidget::default()
    };
    let bytes = run_sep_assertions("sparse_list_with_nulls", value);
    assert_eq!(
        bytes, br#"{"value_sparse_list":["alpha",null,"gamma",null]}"#,
        "sparse list must serialize null entries as the JSON `null` \
         literal in their original positions, not elide them",
    );
}

#[test]
fn sep_round_trip_full_payload() {
    // All member kinds populated together — the most stringent case
    // because it exercises the interleaving between scalar and
    // aggregate writers (e.g. that returning from `write_list` doesn't
    // corrupt the serializer's `needs_comma` state for the next
    // member).
    let value = OmniWidget {
        value_string: Some("hi".to_owned()),
        value_blob: Some(Blob::new(b"abcd".to_vec())),
        value_timestamp_default: Some(DateTime::from_secs(0)),
        value_list_strings: Some(vec!["red".to_owned()]),
        value_map_strings: Some(vec![("k".to_owned(), "v".to_owned())]),
        value_struct: Some(Nested {
            inner_string: Some("n".to_owned()),
        }),
        value_sparse_list: Some(vec![Some("a".to_owned()), None]),
    };
    let bytes = run_sep_assertions("full_payload", value);
    let expected = br#"{"value_string":"hi","value_blob":"YWJjZA==","value_timestamp_default":0,"value_list_strings":["red"],"value_map_strings":{"k":"v"},"value_struct":{"inner_string":"n"},"value_sparse_list":["a",null]}"#;
    assert_eq!(bytes.as_slice(), expected.as_slice());
}

// -- Discriminator emission contract ----------------------------------

#[test]
fn sep_write_document_emits_type_discriminator() {
    // `DiscriminatedDocument::from_struct(schema, value)` attaches the
    // schema's shape ID as the wrapper's discriminator (per the SEP
    // "typed Document" model — a Document derived from a typed shape
    // carries the typed-shape identity).
    //
    // When such a discriminator-tagged DiscriminatedDocument is
    // serialized via `write_discriminated_document`, `JsonSerializer`
    // emits a `__type` field with the absolute shape id per SEP §
    // Typed Document Serialization rule 1
    // ("`"__type": "com.absolute#ShapeId"`"). This test pins that
    // contract down — it's deliberately separate from the SEP
    // 6-assertion matrix because the typed-shape `write_struct` path
    // does NOT emit `__type` and the two are structurally different.
    //
    // The discriminator's serialized form must be the *absolute*
    // shape id even when the codec is configured for a default
    // namespace (per SEP § Typed Document Serialization rule 2).
    let value = OmniWidget {
        value_string: Some("hi".to_owned()),
        ..OmniWidget::default()
    };
    let codec = JsonCodec::default();
    let document = DiscriminatedDocument::from_struct(&OMNI_WIDGET_SCHEMA, &value)
        .expect("DiscriminatedDocument::from_struct succeeds for OmniWidget");

    let mut ser = codec.create_serializer();
    ser.write_discriminated_document(&prelude::DOCUMENT, &document)
        .expect("write_discriminated_document succeeds for a discriminated Document");
    let bytes = ser.finish();

    // The `__type` field appears first (the implementation emits the
    // discriminator before iterating regular members — this lets a
    // streaming JSON consumer dispatch on type before reading the
    // body) and uses the absolute shape id form
    // "namespace#ShapeName".
    let expected = br#"{"__type":"smithy.example#OmniWidget","value_string":"hi"}"#;
    assert_eq!(
        bytes.as_slice(),
        expected.as_slice(),
        "write_discriminated_document must emit the discriminator as \
         `__type` with the absolute shape id, ahead of the member fields",
    );
}

#[test]
fn sep_write_document_no_discriminator_omits_type() {
    // Symmetric negative: a Document constructed directly (no typed-
    // shape origin) has no discriminator. The trait method
    // `write_document` never emits `__type` for *any* input — it
    // operates purely on the data tree. This pins down the contract
    // that `write_document` is the discriminator-free emission path.
    let mut entries = aws_smithy_types::document::DocumentObject::new();
    entries.insert("value_string".to_owned(), Document::String("hi".into()));
    let document = Document::Object(entries);

    let codec = JsonCodec::default();
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &document)
        .expect("write_document succeeds for a plain Document");
    let bytes = ser.finish();

    let expected = br#"{"value_string":"hi"}"#;
    assert_eq!(
        bytes.as_slice(),
        expected.as_slice(),
        "write_document on a plain Document must NOT emit `__type` \
         (the trait method is the discriminator-free emission path; \
         use write_discriminated_document for typed-Document emission)",
    );
}

// -- @jsonName interaction --------------------------------------------
//
// `@jsonName` is a JSON-codec-specific protocol trait. Two kinds of
// behavior must be pinned down separately:
//
// 1. On the typed-shape `write_struct` / `read_struct` path, the
//    codec's `use_json_name` setting decides whether the wire keys
//    use the member's `@jsonName` value (restJson1 → true) or the
//    Smithy member name (awsJson1.0 / 1.1 → false).
//
// 2. On the Document path, `Document::from_struct(...)` always stores
//    *Smithy member names* as map keys (per SEP § Document Types
//    rule 8: serialize-side Documents are protocol-agnostic).
//    Consequently `write_document` emits the member name regardless
//    of `use_json_name`.
//
// The 6-assertion matrix would fail assertion 6 for any fixture
// carrying `@jsonName` whenever the codec has `use_json_name=true`,
// because the typed wire form (with jsonName) and the Document wire
// form (without) genuinely differ. So these tests sit outside the
// matrix and exercise each contract in isolation.

const ALIAS_HOLDER_ID: ShapeId<'static> = shape_id!("smithy.example", "AliasHolder");

static M_ALTERNATE_NAME: Schema<'static> = Schema::new_member(
    shape_id!("smithy.example", "AliasHolder", "alternate_name"),
    ShapeType::String,
    "alternate_name",
    0,
)
.with_json_name("AlternateName");

static ALIAS_HOLDER_SCHEMA: Schema<'static> =
    Schema::new_struct(ALIAS_HOLDER_ID, ShapeType::Structure, &[&M_ALTERNATE_NAME]);

#[derive(Debug, Default, Clone, PartialEq)]
struct AliasHolder {
    alternate_name: Option<String>,
}

impl SerializableStruct for AliasHolder {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        if let Some(v) = &self.alternate_name {
            ser.write_string(&M_ALTERNATE_NAME, v)?;
        }
        Ok(())
    }
}

#[test]
fn sep_typed_shape_uses_json_name_on_serialize() {
    // Codec configured for restJson1-style behavior (`use_json_name=true`,
    // matching how `AwsRestJsonProtocol` is built). Typed-shape
    // serialization MUST emit the `@jsonName` value as the JSON key.
    use aws_smithy_json::codec::JsonCodecSettings;

    let codec = JsonCodec::new(JsonCodecSettings::builder().use_json_name(true).build());
    let value = AliasHolder {
        alternate_name: Some("hello".to_owned()),
    };

    let mut ser = codec.create_serializer();
    ser.write_struct(&ALIAS_HOLDER_SCHEMA, &value)
        .expect("write_struct must succeed");
    let bytes = ser.finish();

    assert_eq!(
        bytes, br#"{"AlternateName":"hello"}"#,
        "with use_json_name=true the wire form must use the @jsonName \
         override (\"AlternateName\"), not the Smithy member name",
    );
}

#[test]
fn sep_typed_shape_ignores_json_name_when_disabled() {
    // Codec configured for awsJson-style behavior (`use_json_name=false`,
    // matching how `AwsJsonRpcProtocol` is built for both 1.0 and 1.1).
    // Typed-shape serialization MUST emit the Smithy member name and
    // ignore the `@jsonName` override.
    use aws_smithy_json::codec::JsonCodecSettings;

    let codec = JsonCodec::new(JsonCodecSettings::builder().use_json_name(false).build());
    let value = AliasHolder {
        alternate_name: Some("hello".to_owned()),
    };

    let mut ser = codec.create_serializer();
    ser.write_struct(&ALIAS_HOLDER_SCHEMA, &value)
        .expect("write_struct must succeed");
    let bytes = ser.finish();

    assert_eq!(
        bytes, br#"{"alternate_name":"hello"}"#,
        "with use_json_name=false the wire form must use the Smithy \
         member name, ignoring the @jsonName override",
    );
}

#[test]
fn sep_write_document_ignores_json_name() {
    // `Document::from_struct(...)` uses `member.member_name()` as the
    // map key (the codebase comment confirms this:
    // "uses `member_schema.member_name()` as the map key" in
    // `aws-smithy-schema/src/schema/document/serializer.rs`). So the
    // Document tree always carries the Smithy member name regardless
    // of any `@jsonName` on the original schema member.
    //
    // `write_document` therefore emits the Smithy member name as the
    // JSON key — even when the codec has `use_json_name=true`. This
    // is the protocol-agnostic-Document contract from SEP § Document
    // Types rule 8: a serialize-side Document is independent of the
    // protocol it will eventually be transmitted under.
    use aws_smithy_json::codec::JsonCodecSettings;

    let codec = JsonCodec::new(JsonCodecSettings::builder().use_json_name(true).build());
    let value = AliasHolder {
        alternate_name: Some("hello".to_owned()),
    };
    let document = DiscriminatedDocument::from_struct(&ALIAS_HOLDER_SCHEMA, &value)
        .expect("DiscriminatedDocument::from_struct must succeed");

    let mut ser = codec.create_serializer();
    // Use plain `write_document` on the wrapper's `.document()` view
    // so we don't emit `__type` here — this test's contract is about
    // member-name handling, not about the discriminator. The data
    // tree never carries discriminators on individual nodes; only
    // the wrapper does.
    ser.write_document(&prelude::DOCUMENT, document.document())
        .expect("write_document must succeed");
    let bytes = ser.finish();

    assert_eq!(
        bytes, br#"{"alternate_name":"hello"}"#,
        "write_document must emit the Smithy member name even when \
         the codec is configured with use_json_name=true — Documents \
         are protocol-agnostic on the serialize side",
    );
}

// -- Big-number Document numerics -------------------------------------
//
// `BigInteger` and `BigDecimal` are arbitrary-precision numerics that
// cannot be represented as `serde_json::Number` (which is f64 under
// the hood). The Document path stores them as their own
// `Document::BigInteger` / `Document::BigDecimal` variants;
// `JsonSerializer::write_json_value` emits the underlying decimal
// string verbatim as a raw JSON number — without any precision loss
// that would occur if it were routed through `serde_json::Number`.
//
// These tests pin the wire form down. They sit outside the SEP
// 6-assertion matrix because there's no typed-shape carrier in
// `OmniWidget` for these types — the Document is constructed
// directly.

#[test]
fn sep_write_document_big_integer_preserves_precision() {
    use std::str::FromStr;

    use aws_smithy_types::BigInteger;

    // A value far beyond i64::MAX (which is ~9.2 × 10^18). If
    // `write_document` routed BigIntegers through any f64-typed
    // intermediate it would lose the trailing digits — the f64
    // round-trip of this number is `1.2345678901234568e+30`, which
    // would emit at minimum a different string and at worst a
    // mismatched exponent form. We assert the exact decimal string
    // reaches the wire intact.
    let value = BigInteger::from_str("1234567890123456789012345678901").expect("valid BigInteger");
    let document = Document::BigInteger(value);

    let codec = JsonCodec::default();
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &document)
        .expect("write_document on BigInteger must succeed");
    let bytes = ser.finish();

    assert_eq!(
        bytes, b"1234567890123456789012345678901",
        "write_document must emit a BigInteger as a raw JSON number, \
         preserving the full decimal precision of the source string",
    );
}

#[test]
fn sep_write_document_big_decimal_preserves_precision() {
    use std::str::FromStr;

    use aws_smithy_types::BigDecimal;

    // High-precision decimal that exceeds f64's ~15-17 significant
    // digits. f64 round-trip of `0.123456789012345678901234567890`
    // would produce `0.12345678901234568` — losing 14 trailing
    // digits. The wire form must keep every digit.
    let value = BigDecimal::from_str("0.123456789012345678901234567890").expect("valid BigDecimal");
    let document = Document::BigDecimal(value);

    let codec = JsonCodec::default();
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &document)
        .expect("write_document on BigDecimal must succeed");
    let bytes = ser.finish();

    assert_eq!(
        bytes, b"0.123456789012345678901234567890",
        "write_document must emit a BigDecimal as a raw JSON number, \
         preserving the full decimal precision of the source string \
         (no f64 round-trip)",
    );
}

#[test]
fn sep_write_document_big_numbers_inside_aggregate() {
    // BigInteger and BigDecimal nested inside list and map containers
    // — verifies the recursive walk in `write_json_value` visits big-
    // number variants without going through `serde_json::Number` even
    // when wrapped in container types.
    use std::str::FromStr;

    use aws_smithy_types::{BigDecimal, BigInteger};

    let big_int = BigInteger::from_str("999999999999999999999").expect("valid BigInteger");
    let big_dec = BigDecimal::from_str("1.234567890123456789012345").expect("valid BigDecimal");

    let mut entries = aws_smithy_types::document::DocumentObject::new();
    entries.insert(
        "list_of_big_ints".to_owned(),
        Document::Array(vec![Document::BigInteger(big_int.clone())]),
    );
    entries.insert(
        "single_big_dec".to_owned(),
        Document::BigDecimal(big_dec.clone()),
    );
    let document = Document::Object(entries);

    let codec = JsonCodec::default();
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &document)
        .expect("write_document on a map containing big numbers must succeed");
    let bytes = ser.finish();

    // HashMap iteration order is non-deterministic, so compare
    // structurally via serde_json::Value rather than byte-for-byte.
    // Note that `serde_json` itself preserves arbitrary precision via
    // the `arbitrary_precision` feature; in its absence, big numbers
    // round-trip through f64 and we'd lose precision. To sidestep that
    // entirely, locate each precision-sensitive substring in the
    // emitted bytes directly.
    let bytes_str = std::str::from_utf8(&bytes).expect("output is valid UTF-8");
    assert!(
        bytes_str.contains(r#""list_of_big_ints":[999999999999999999999]"#),
        "BigInteger inside a list must be emitted as a raw JSON number \
         with full precision; got: {bytes_str}",
    );
    assert!(
        bytes_str.contains(r#""single_big_dec":1.234567890123456789012345"#),
        "BigDecimal inside a map must be emitted as a raw JSON number \
         with full precision; got: {bytes_str}",
    );
}

// =============================================================================
// SEP "Document Type and Type Registries" §"`Document` interface and type
// coercion" point 9: "Document implementations SHOULD iterate map entries in
// insertion order if possible. For a document created from serialized data,
// insertion order is the order in which the entries appear in the source data."
// =============================================================================

#[test]
fn sep_map_document_iterates_in_source_order() {
    // Three keys whose hash order would not match either source order or
    // alphabetical order under a typical SipHasher seed. The test asserts the
    // document iterates them in source order, not hash order.
    let source = br#"{"zebra":1,"alpha":2,"middle":3}"#;

    let codec = JsonCodec::default();
    let mut deser = codec.create_deserializer(source);

    let document: Document = deser
        .read_document(&prelude::DOCUMENT)
        .expect("source bytes are valid JSON");

    let object = document
        .as_object()
        .expect("read_document on a JSON object yields Document::Object");

    let observed: Vec<&str> = object.keys().map(String::as_str).collect();
    assert_eq!(
        observed,
        ["zebra", "alpha", "middle"],
        "DocumentObject must iterate keys in the order they appeared in the \
         source bytes (SEP Document Type and Type Registries §`Document` \
         interface point 9)"
    );

    // Round-trip back through write_document and assert wire bytes match the
    // source order too — confirms that serialization preserves the same
    // insertion order on output.
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &document)
        .expect("write_document succeeds");
    let bytes = ser.finish();
    let bytes_str = std::str::from_utf8(&bytes).expect("valid UTF-8");

    assert_eq!(
        bytes_str, r#"{"zebra":1,"alpha":2,"middle":3}"#,
        "Round-trip serialization must preserve insertion order. Got: {bytes_str}",
    );
}

// -- use_string_for_arbitrary_precision wire-form round-trip ----------
//
// SEP §"Number coercion" + the JSON codec's behavior under the
// `use_string_for_arbitrary_precision` setting. Sender configured for
// strings → receiver decodes from strings → re-emits as strings:
// lossless. Cross-config round-trip (sender writes strings, receiver
// is on default) also works because read is unconditionally lenient.

#[test]
fn arbitrary_precision_string_form_round_trip() {
    use aws_smithy_types::{BigDecimal, BigInteger};
    use std::str::FromStr;

    let codec = JsonCodec::new(
        JsonCodecSettings::builder()
            .use_string_for_arbitrary_precision(true)
            .build(),
    );

    let bi = BigInteger::from_str("99999999999999999999999").unwrap();
    let bd = BigDecimal::from_str("0.123456789012345678901234567890").unwrap();
    let bi_doc = Document::BigInteger(bi.clone());
    let bd_doc = Document::BigDecimal(bd.clone());

    // Write — wire form is a JSON string.
    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &bi_doc).unwrap();
    let bi_wire = ser.finish();
    assert_eq!(bi_wire, br#""99999999999999999999999""#);

    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &bd_doc).unwrap();
    let bd_wire = ser.finish();
    assert_eq!(bd_wire, br#""0.123456789012345678901234567890""#);

    // Read with a typed schema-aware accessor — both wire forms accepted.
    let mut deser = codec.create_deserializer(&bi_wire);
    let parsed_bi = deser.read_big_integer(&prelude::BIG_INTEGER).unwrap();
    assert_eq!(parsed_bi.as_ref(), bi.as_ref());

    let mut deser = codec.create_deserializer(&bd_wire);
    let parsed_bd = deser.read_big_decimal(&prelude::BIG_DECIMAL).unwrap();
    assert_eq!(parsed_bd.as_ref(), bd.as_ref());
}

#[test]
fn arbitrary_precision_default_form_round_trip() {
    // Default `use_string_for_arbitrary_precision = false` writes raw
    // JSON numbers. Confirms the existing baseline test
    // `sep_write_document_big_integer_preserves_precision` covers this
    // direction; here we additionally verify that
    // `read_big_integer`/`read_big_decimal` still parse the number form.
    use aws_smithy_types::{BigDecimal, BigInteger};
    use std::str::FromStr;

    let codec = JsonCodec::default();

    let bi = BigInteger::from_str("99999999999999999999999").unwrap();
    let bd = BigDecimal::from_str("1.234567890123456789012345").unwrap();

    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &Document::BigInteger(bi.clone()))
        .unwrap();
    let bi_wire = ser.finish();
    assert_eq!(bi_wire, b"99999999999999999999999"); // No quotes.

    let mut deser = codec.create_deserializer(&bi_wire);
    let parsed_bi = deser.read_big_integer(&prelude::BIG_INTEGER).unwrap();
    assert_eq!(parsed_bi.as_ref(), bi.as_ref());

    let mut ser = codec.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &Document::BigDecimal(bd.clone()))
        .unwrap();
    let bd_wire = ser.finish();
    assert_eq!(bd_wire, b"1.234567890123456789012345");

    let mut deser = codec.create_deserializer(&bd_wire);
    let parsed_bd = deser.read_big_decimal(&prelude::BIG_DECIMAL).unwrap();
    assert_eq!(parsed_bd.as_ref(), bd.as_ref());
}

#[test]
fn arbitrary_precision_cross_config_interop() {
    // Sender configured for string form, receiver on default — verifies
    // the read side is unconditionally lenient on wire form regardless
    // of its own setting.
    use aws_smithy_types::BigInteger;
    use std::str::FromStr;

    let strict_sender = JsonCodec::new(
        JsonCodecSettings::builder()
            .use_string_for_arbitrary_precision(true)
            .build(),
    );
    let default_receiver = JsonCodec::default();

    let bi = BigInteger::from_str("99999999999999999999999").unwrap();

    let mut ser = strict_sender.create_serializer();
    ser.write_document(&prelude::DOCUMENT, &Document::BigInteger(bi.clone()))
        .unwrap();
    let wire = ser.finish();
    assert!(wire.starts_with(b"\""));

    let mut deser = default_receiver.create_deserializer(&wire);
    let parsed = deser.read_big_integer(&prelude::BIG_INTEGER).unwrap();
    assert_eq!(parsed.as_ref(), bi.as_ref());
}
