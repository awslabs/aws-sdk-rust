/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Downstream source-compatibility contract for [`Document`].
//!
//! This file lives in `tests/` on purpose: it is compiled as a separate
//! crate, so it sees `aws_smithy_types` exactly as a published
//! downstream consumer does. Anything that compiles here compiles for
//! them; anything that stops compiling here is a source-breaking change
//! to the `Document` API.
//!
//! The contract asserted is the one published as `aws-smithy-types`
//! 1.6.4:
//!
//! 1. `Document` has exactly six variants, in the order
//!    `Object`, `Array`, `Number`, `String`, `Bool`, `Null`.
//! 2. `Document` is **not** `#[non_exhaustive]`, so a downstream `match`
//!    with six arms and no wildcard compiles. (A `#[non_exhaustive]`
//!    enum would require a wildcard arm from outside the defining
//!    crate, so the six-arm match below would fail to compile.)
//! 3. `Document::Object` wraps a `std::collections::HashMap<String,
//!    Document>`, constructible directly by a downstream caller.
//! 4. `as_object` returns `Option<&HashMap<String, Document>>` and
//!    `as_object_mut` returns `Option<&mut HashMap<String, Document>>`
//!    — the exact released types, not an opaque wrapper.
//!
//! No `trybuild`: the compile-time half of the contract is enforced by
//! this file compiling at all, which every `cargo test` run does.

use aws_smithy_types::{Document, Number};
use std::collections::HashMap;

/// Exhaustive six-arm `match` with **no** wildcard arm.
///
/// This is the load-bearing assertion in this file. If `Document` ever
/// regains `#[non_exhaustive]`, or gains a seventh variant, this
/// function stops compiling with `E0004`.
fn variant_name(d: &Document) -> &'static str {
    match d {
        Document::Object(_) => "object",
        Document::Array(_) => "array",
        Document::Number(_) => "number",
        Document::String(_) => "string",
        Document::Bool(_) => "bool",
        Document::Null => "null",
    }
}

#[test]
fn exhaustive_six_arm_match_without_wildcard_compiles() {
    let cases: Vec<(Document, &str)> = vec![
        (Document::Object(HashMap::new()), "object"),
        (Document::Array(Vec::new()), "array"),
        (Document::Number(Number::PosInt(1)), "number"),
        (Document::String("s".to_owned()), "string"),
        (Document::Bool(true), "bool"),
        (Document::Null, "null"),
    ];
    for (doc, expected) in cases {
        assert_eq!(variant_name(&doc), expected);
    }
}

#[test]
fn object_variant_is_constructed_directly_from_a_hash_map() {
    // Direct `Document::Object(HashMap)` construction — the pattern in
    // released downstream code.
    let mut map: HashMap<String, Document> = HashMap::new();
    map.insert("k".to_owned(), Document::String("v".to_owned()));
    let doc = Document::Object(map);

    // Destructuring back out to a `HashMap` by value must also work.
    match doc {
        Document::Object(inner) => {
            let inner: HashMap<String, Document> = inner;
            assert_eq!(inner.len(), 1);
            assert_eq!(inner["k"], Document::String("v".to_owned()));
        }
        other => panic!("expected object, got {}", variant_name(&other)),
    }
}

#[test]
fn as_object_returns_a_hash_map_reference() {
    let mut map: HashMap<String, Document> = HashMap::new();
    map.insert("a".to_owned(), Document::Bool(false));
    let doc = Document::Object(map);

    // The annotation is the assertion: `as_object` must return exactly
    // `Option<&HashMap<String, Document>>`. An opaque wrapper type would
    // fail to unify here.
    let got: Option<&HashMap<String, Document>> = doc.as_object();
    assert_eq!(got.unwrap().len(), 1);

    // Non-object variants still return `None`.
    assert!(Document::Null.as_object().is_none());
}

#[test]
fn as_object_mut_returns_a_mutable_hash_map_reference() {
    let mut doc = Document::Object(HashMap::new());

    let got: Option<&mut HashMap<String, Document>> = doc.as_object_mut();
    let map = got.expect("object");
    // Mutating through the borrow uses inherent `HashMap` API.
    map.insert("added".to_owned(), Document::Number(Number::NegInt(-1)));
    map.reserve(4);

    assert_eq!(
        doc.as_object().unwrap()["added"],
        Document::Number(Number::NegInt(-1))
    );

    assert!(Document::Null.as_object_mut().is_none());
}

#[test]
fn released_accessors_predicates_and_conversions_are_unchanged() {
    // Accessors.
    let array = Document::Array(vec![Document::Null]);
    let _: Option<&Vec<Document>> = array.as_array();
    let mut array = array;
    let _: Option<&mut Vec<Document>> = array.as_array_mut();
    let _: Option<&Number> = Document::Number(Number::PosInt(1)).as_number();
    let _: Option<&str> = Document::String("s".to_owned()).as_string();
    let _: Option<bool> = Document::Bool(true).as_bool();
    let _: Option<()> = Document::Null.as_null();

    // Predicates.
    assert!(Document::Object(HashMap::new()).is_object());
    assert!(Document::Array(Vec::new()).is_array());
    assert!(Document::Number(Number::PosInt(0)).is_number());
    assert!(Document::String(String::new()).is_string());
    assert!(Document::Bool(false).is_bool());
    assert!(Document::Null.is_null());

    // `Default` is `Null`.
    assert_eq!(Document::default(), Document::Null);

    // `From` conversions, including the `HashMap` one.
    let mut map: HashMap<String, Document> = HashMap::new();
    map.insert("x".to_owned(), 1u64.into());
    let from_map: Document = map.into();
    assert!(from_map.is_object());

    let from_vec: Document = vec![Document::Null].into();
    assert!(from_vec.is_array());

    assert_eq!(Document::from("str"), Document::String("str".to_owned()));
    assert_eq!(
        Document::from("owned".to_owned()),
        Document::String("owned".to_owned())
    );
    assert_eq!(Document::from(true), Document::Bool(true));
    assert_eq!(Document::from(1u64), Document::Number(Number::PosInt(1)));
    assert_eq!(Document::from(-1i64), Document::Number(Number::NegInt(-1)));
    assert_eq!(Document::from(1.5f64), Document::Number(Number::Float(1.5)));
}
