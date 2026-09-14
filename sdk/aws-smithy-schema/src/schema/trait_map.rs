/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{ShapeId, Trait};
use std::collections::HashMap;

/// A map of traits keyed by their Shape ID.
///
/// This provides efficient lookup of traits during serialization and deserialization.
///
/// # Cross-lifetime lookup
///
/// Internal storage is keyed by `ShapeId<'static>` because traits come from
/// codegen-emitted schemas (themselves `'static`). The public `get` and
/// `contains` accessors accept `&ShapeId<'_>` of any lifetime — runtime-
/// built shape IDs (e.g. parsed from a wire-format `__type` field) are
/// looked up against the `'static`-keyed table via `ShapeId`'s
/// `Borrow<str>` impl, which keys on the fully qualified name.
#[derive(Debug)]
pub struct TraitMap {
    // Wrapped in `Option` because `HashMap::new()` is not `const fn` in stable Rust,
    // allowing `TraitMap::EMPTY` to be used in const contexts (e.g. prelude schemas).
    traits: Option<HashMap<ShapeId<'static>, Box<dyn Trait>>>,
}

impl Default for TraitMap {
    fn default() -> Self {
        Self::new()
    }
}

impl TraitMap {
    /// An empty trait map for use in const contexts (e.g. prelude schemas).
    pub const EMPTY: Self = Self { traits: None };

    /// Creates a new empty TraitMap.
    pub fn new() -> Self {
        Self {
            traits: Some(HashMap::new()),
        }
    }

    /// Inserts a trait into the map.
    pub fn insert(&mut self, trait_obj: Box<dyn Trait>) {
        let id = trait_obj.trait_id().clone();
        self.traits
            .get_or_insert_with(HashMap::new)
            .insert(id, trait_obj);
    }

    /// Gets a trait by its Shape ID.
    ///
    /// Accepts a [`ShapeId`] of any lifetime — lookup is by fully
    /// qualified name. A `ShapeId<'static>` constructed via the
    /// `shape_id!` macro and a runtime-built `ShapeId<'_>` with the
    /// same FQN resolve to the same entry.
    pub fn get(&self, id: &ShapeId<'_>) -> Option<&dyn Trait> {
        // `id.as_str()` returns `&str`; the map's `Borrow<str>` impl
        // (via `ShapeId`'s `Borrow<str>`) makes this a direct lookup.
        self.traits.as_ref()?.get(id.as_str()).map(|t| t.as_ref())
    }

    /// Looks up a trait by its fully qualified name (e.g.
    /// `"smithy.api#sensitive"`).
    ///
    /// Equivalent to [`Self::get`] when the caller already has the FQN
    /// as a string slice — for example, after extracting the
    /// `__type` field from a JSON document.
    ///
    /// # Example
    ///
    /// ```
    /// use aws_smithy_schema::traits::JsonNameTrait;
    /// use aws_smithy_schema::{shape_id, ShapeId, TraitMap};
    ///
    /// let mut traits = TraitMap::new();
    /// traits.insert(Box::new(JsonNameTrait::new("custom_name")));
    ///
    /// // Cross-lifetime: a static-lifetime ShapeId from `shape_id!` and a
    /// // runtime-built ShapeId<'_> with the same FQN both resolve via
    /// // `Borrow<str>`. The companion `_fqn` accessor takes `&str` directly.
    /// assert!(traits.get(&shape_id!("smithy.api", "jsonName")).is_some());
    /// assert!(traits.get_fqn("smithy.api#jsonName").is_some());
    /// ```
    pub fn get_fqn(&self, fqn: &str) -> Option<&dyn Trait> {
        self.traits.as_ref()?.get(fqn).map(|t| t.as_ref())
    }

    /// Returns true if the map contains a trait with the given Shape ID.
    ///
    /// Accepts a [`ShapeId`] of any lifetime; see [`Self::get`].
    pub fn contains(&self, id: &ShapeId<'_>) -> bool {
        self.traits
            .as_ref()
            .is_some_and(|m| m.contains_key(id.as_str()))
    }

    /// Returns true if the map contains a trait with the given fully
    /// qualified name. See [`Self::get_fqn`].
    pub fn contains_fqn(&self, fqn: &str) -> bool {
        self.traits.as_ref().is_some_and(|m| m.contains_key(fqn))
    }

    /// Returns the number of traits in the map.
    pub fn len(&self) -> usize {
        self.traits.as_ref().map_or(0, |m| m.len())
    }

    /// Returns true if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.traits.as_ref().is_none_or(|m| m.is_empty())
    }
}
