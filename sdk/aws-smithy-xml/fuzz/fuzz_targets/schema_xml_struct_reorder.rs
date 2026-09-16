/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Fuzz target: element-order tolerance for the schema-based XML deserializer.
//!
//! # Strategy
//!
//! REST XML's wire format does not guarantee that child elements of a struct
//! appear in their declaration order. The deserializer must dispatch each
//! child element to the right member by name, regardless of position. This
//! target catches any latent ordering assumption.
//!
//! Per fuzz iteration:
//! 1. `Arbitrary` produces an `OrderingProbe` carrying explicit values for
//!    every member of a small schema, plus a permutation seed.
//! 2. Serialize the value via `XmlSerializer` (which emits members in
//!    declaration order).
//! 3. Parse the output with `xmlparser`, slicing the document into:
//!    - the outer-element prefix `<Wrapper>`,
//!    - one byte range per child element (`<a>...</a>`, `<b>...</b>`, etc.),
//!    - the outer-element suffix `</Wrapper>`.
//! 4. Apply a deterministic permutation to the child-element list using a
//!    seeded Fisher-Yates shuffle.
//! 5. Reassemble into a permuted XML document.
//! 6. Deserialize the permuted document with the same schema, and assert
//!    every member round-tripped to its original value.
//!
//! # Why this matters
//!
//! A reordering bug in the deserializer would show up as either a missing
//! field (the consumer was never invoked for one of the members) or a
//! wrong value (the wrong member was populated). Both manifest as round-
//! trip mismatches the assert below catches.

#![no_main]
use libfuzzer_sys::fuzz_target;

mod schema_common;

use arbitrary::Arbitrary;
use aws_smithy_schema::codec::{Codec, FinishSerializer};
use aws_smithy_schema::serde::{
    SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer,
};
use aws_smithy_schema::{shape_id, Schema, ShapeType};
use schema_common::default_codec;

// Minimal multi-member struct schema. Five distinct members of distinct
// scalar types so any swap is detectable. Members deliberately have short
// names to keep the serialized output small and the fuzz throughput high.

static M_A: Schema = Schema::new_member(shape_id!("test", "Probe$a"), ShapeType::Boolean, "a", 0);
static M_B: Schema = Schema::new_member(shape_id!("test", "Probe$b"), ShapeType::Integer, "b", 1);
static M_C: Schema = Schema::new_member(shape_id!("test", "Probe$c"), ShapeType::String, "c", 2);
static M_D: Schema = Schema::new_member(shape_id!("test", "Probe$d"), ShapeType::Long, "d", 3);
static M_E: Schema = Schema::new_member(shape_id!("test", "Probe$e"), ShapeType::Short, "e", 4);

static PROBE_SCHEMA: Schema = Schema::new_struct(
    shape_id!("test", "Probe"),
    ShapeType::Structure,
    &[&M_A, &M_B, &M_C, &M_D, &M_E],
);

#[derive(Debug, Clone, Arbitrary)]
struct OrderingProbe {
    a: bool,
    b: i32,
    c: String,
    d: i64,
    e: i16,
    /// Permutation seed used to shuffle child elements before deserialization.
    seed: u64,
}

impl SerializableStruct for OrderingProbe {
    fn serialize_members(&self, ser: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
        ser.write_boolean(&M_A, self.a)?;
        ser.write_integer(&M_B, self.b)?;
        ser.write_string(&M_C, &self.c)?;
        ser.write_long(&M_D, self.d)?;
        ser.write_short(&M_E, self.e)?;
        Ok(())
    }
}

fuzz_target!(|probe: OrderingProbe| {
    // Skip strings whose contents would require XML 1.0 escaping the parser
    // can't represent (matches the roundtrip target's skip — same reason).
    if probe.c.chars().any(|c| {
        let v = c as u32;
        !(v == 0x9
            || v == 0xA
            || v == 0xD
            || (0x20..=0xD7FF).contains(&v)
            || (0xE000..=0xFFFD).contains(&v)
            || (0x10000..=0x10FFFF).contains(&v))
    }) {
        return;
    }

    let codec = default_codec();
    let mut ser = codec.create_serializer();
    if ser.write_struct(&PROBE_SCHEMA, &probe).is_err() {
        return;
    }
    let bytes = ser.finish();
    let s = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Slice the serialized bytes into:
    //   prefix + [child element ranges, in document order] + suffix
    // The prefix is everything up to and including the start tag's `>`.
    // The suffix is the closing `</Probe>`.
    let (prefix, children, suffix) = match split_children(s) {
        Some(parts) => parts,
        None => return, // Empty or unsplittable — skip.
    };
    if children.is_empty() {
        return; // Nothing to permute.
    }

    // Permute children with a deterministic Fisher-Yates seeded by `probe.seed`.
    let mut indices: Vec<usize> = (0..children.len()).collect();
    let mut rng = SplitMix64(probe.seed);
    for i in (1..indices.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        indices.swap(i, j);
    }

    let mut reordered = String::with_capacity(s.len());
    reordered.push_str(prefix);
    for &i in &indices {
        reordered.push_str(children[i]);
    }
    reordered.push_str(suffix);

    // Deserialize the reordered XML; assert every field round-trips to
    // its declared value.
    let mut got_a: Option<bool> = None;
    let mut got_b: Option<i32> = None;
    let mut got_c: Option<String> = None;
    let mut got_d: Option<i64> = None;
    let mut got_e: Option<i16> = None;

    let mut deser = codec.create_deserializer(reordered.as_bytes());
    let read_result = deser.read_struct(&PROBE_SCHEMA, &mut |member, d| {
        match member.member_index() {
            Some(0) => got_a = Some(d.read_boolean(member)?),
            Some(1) => got_b = Some(d.read_integer(member)?),
            Some(2) => got_c = Some(d.read_string(member)?),
            Some(3) => got_d = Some(d.read_long(member)?),
            Some(4) => got_e = Some(d.read_short(member)?),
            _ => {}
        }
        Ok(())
    });

    if let Err(e) = read_result {
        panic!(
            "Reordered deserialization failed!\n\
             Probe: {:?}\nOriginal: {:?}\nReordered: {:?}\nError: {:?}",
            probe, s, reordered, e,
        );
    }

    assert_eq!(
        got_a,
        Some(probe.a),
        "field `a` lost in reordering!\nReordered: {:?}",
        reordered
    );
    assert_eq!(
        got_b,
        Some(probe.b),
        "field `b` lost in reordering!\nReordered: {:?}",
        reordered
    );
    assert_eq!(
        got_c.as_deref(),
        Some(probe.c.as_str()),
        "field `c` lost in reordering!\nReordered: {:?}",
        reordered
    );
    assert_eq!(
        got_d,
        Some(probe.d),
        "field `d` lost in reordering!\nReordered: {:?}",
        reordered
    );
    assert_eq!(
        got_e,
        Some(probe.e),
        "field `e` lost in reordering!\nReordered: {:?}",
        reordered
    );
});

/// Split the document `<Probe>...</Probe>` into a prefix, an ordered list of
/// child-element substrings, and a suffix. Returns None if `xmlparser`
/// rejects the input, no `<Probe>` element exists, or there are no children.
fn split_children(s: &str) -> Option<(&str, Vec<&str>, &str)> {
    use xmlparser::Token;
    let mut tokenizer = xmlparser::Tokenizer::from(s);

    // Find the first ElementStart for `Probe` and its closing `>`.
    let prefix_end;
    let depth;
    loop {
        let tok = tokenizer.next()?;
        let tok = tok.ok()?;
        if let Token::ElementStart {
            ref local, span, ..
        } = tok
        {
            if local.as_str() == "Probe" {
                // Walk forward to the open's `>` to know where the prefix ends.
                let _ = span; // unused; walk via next token
                loop {
                    let inner = tokenizer.next()?.ok()?;
                    if let Token::ElementEnd { end, span } = inner {
                        match end {
                            xmlparser::ElementEnd::Open => {
                                prefix_end = span.end();
                                depth = 1i32;
                                break;
                            }
                            xmlparser::ElementEnd::Empty => return None, // empty Probe
                            xmlparser::ElementEnd::Close(_, _) => return None, // unexpected
                        }
                    }
                }
                break;
            }
        }
    }
    let mut depth = depth;

    // Walk children, recording the byte range of each top-level child element.
    // Children are anything between depth=1 and depth=0 again at the outer
    // close tag.
    let mut children: Vec<(usize, usize)> = Vec::new();
    let mut current_start: Option<usize> = None;
    let mut suffix_start = s.len();
    while let Some(tok) = tokenizer.next() {
        let tok = tok.ok()?;
        match tok {
            Token::ElementStart { span, .. } => {
                if depth == 1 {
                    current_start = Some(span.start());
                }
                depth += 1;
            }
            Token::ElementEnd { end, span } => match end {
                xmlparser::ElementEnd::Open => { /* `>` for an open element */ }
                xmlparser::ElementEnd::Close(_, _) => {
                    depth -= 1;
                    if depth == 1 {
                        // Closed a child element; record range.
                        if let Some(start) = current_start.take() {
                            children.push((start, span.end()));
                        }
                    } else if depth == 0 {
                        // Closed the outer Probe.
                        suffix_start = span.start();
                        break;
                    }
                }
                xmlparser::ElementEnd::Empty => {
                    if depth == 1 {
                        // Self-closing child like `<x/>`.
                        if let Some(start) = current_start.take() {
                            children.push((start, span.end()));
                        }
                    }
                    depth -= 1;
                }
            },
            _ => {}
        }
    }

    let prefix = &s[..prefix_end];
    let suffix = &s[suffix_start..];
    let child_slices: Vec<&str> = children.iter().map(|&(a, b)| &s[a..b]).collect();
    Some((prefix, child_slices, suffix))
}

/// SplitMix64 — small and deterministic.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}
