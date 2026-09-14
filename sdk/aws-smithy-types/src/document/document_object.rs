/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! [`DocumentObject`]: insertion-order-preserving map type used as the
//! inner storage of [`Document::Object`].
//!
//! The Smithy "Document Type and Type Registries" SEP requires that a
//! `Document` constructed from serialized data iterate its map entries
//! in the order they appeared in the source data. Rust's
//! [`std::collections::HashMap`] uses hash-randomized iteration order
//! and therefore cannot satisfy that requirement.
//!
//! `DocumentObject` exposes a HashMap-shaped surface — `insert`,
//! `get`, `contains_key`, `len`, `iter`, `Index<&str>`, etc. — while
//! preserving insertion order on iteration. The underlying storage is
//! an internal implementation detail and is not part of the public API,
//! so the implementation can change in the future (for example, to a
//! different ordered-map representation) without breaking callers.
//!
//! # Example
//!
//! ```
//! use aws_smithy_types::{Document, Number, document::DocumentObject};
//!
//! let mut object = DocumentObject::new();
//! object.insert("greeting".to_string(), Document::String("hello".to_string()));
//! object.insert("count".to_string(), Document::Number(Number::PosInt(42)));
//!
//! let doc = Document::Object(object);
//! let map = doc.as_object().unwrap();
//!
//! assert_eq!(map.len(), 2);
//! assert!(map.contains_key("greeting"));
//!
//! // Iteration order matches insertion order.
//! let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
//! assert_eq!(keys, ["greeting", "count"]);
//! ```

use super::Document;
use std::ops::Index;

/// Insertion-order-preserving map from string keys to [`Document`] values.
///
/// See the [module-level documentation](self) for context.
///
/// `DocumentObject` provides a HashMap-shaped public surface. The
/// underlying ordered-map implementation is private and may change in a
/// future release; only the methods documented on this type are part of
/// the public API.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DocumentObject {
    inner: indexmap::IndexMap<String, Document>,
}

impl DocumentObject {
    /// Creates an empty `DocumentObject`.
    pub fn new() -> Self {
        Self {
            inner: indexmap::IndexMap::new(),
        }
    }

    /// Creates an empty `DocumentObject` with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: indexmap::IndexMap::with_capacity(capacity),
        }
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Inserts a key-value pair.
    ///
    /// If a value already existed for the key, the old value is replaced
    /// in place at its existing iteration position and returned.
    /// Otherwise the new entry is appended at the end of the iteration
    /// order, and `None` is returned.
    pub fn insert(&mut self, key: String, value: Document) -> Option<Document> {
        self.inner.insert(key, value)
    }

    /// Returns a reference to the value for `key`, if present.
    pub fn get(&self, key: &str) -> Option<&Document> {
        self.inner.get(key)
    }

    /// Returns a mutable reference to the value for `key`, if present.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Document> {
        self.inner.get_mut(key)
    }

    /// Returns `true` if the map contains an entry for `key`.
    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    /// Removes the entry for `key`, preserving the iteration order of the
    /// remaining entries. Returns the removed value, or `None` if no
    /// entry existed.
    pub fn remove(&mut self, key: &str) -> Option<Document> {
        self.inner.shift_remove(key)
    }

    /// Returns an iterator over the entries in insertion order.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self.inner.iter(),
        }
    }

    /// Returns a mutable iterator over the entries in insertion order.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            inner: self.inner.iter_mut(),
        }
    }

    /// Returns an iterator over the keys in insertion order.
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            inner: self.inner.keys(),
        }
    }

    /// Returns an iterator over the values in insertion order.
    pub fn values(&self) -> Values<'_> {
        Values {
            inner: self.inner.values(),
        }
    }

    /// Returns a mutable iterator over the values in insertion order.
    pub fn values_mut(&mut self) -> ValuesMut<'_> {
        ValuesMut {
            inner: self.inner.values_mut(),
        }
    }
}

// ---------- Iterator types ----------------------------------------------
// Wrap indexmap's iterators so the implementation type doesn't appear in
// our public API. `impl Iterator` would also work but a named iterator
// type plays better with downstream code that wants to name the type.

/// Borrowing iterator over the entries of a [`DocumentObject`].
#[derive(Debug)]
pub struct Iter<'a> {
    inner: indexmap::map::Iter<'a, String, Document>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a String, &'a Document);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Iter<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Mutably borrowing iterator over the entries of a [`DocumentObject`].
#[derive(Debug)]
pub struct IterMut<'a> {
    inner: indexmap::map::IterMut<'a, String, Document>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a String, &'a mut Document);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for IterMut<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Iterator over the keys of a [`DocumentObject`].
#[derive(Debug)]
pub struct Keys<'a> {
    inner: indexmap::map::Keys<'a, String, Document>,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a String;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Keys<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Iterator over the values of a [`DocumentObject`].
#[derive(Debug)]
pub struct Values<'a> {
    inner: indexmap::map::Values<'a, String, Document>,
}

impl<'a> Iterator for Values<'a> {
    type Item = &'a Document;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Values<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Mutably borrowing iterator over the values of a [`DocumentObject`].
#[derive(Debug)]
pub struct ValuesMut<'a> {
    inner: indexmap::map::ValuesMut<'a, String, Document>,
}

impl<'a> Iterator for ValuesMut<'a> {
    type Item = &'a mut Document;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for ValuesMut<'_> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Owning iterator over the entries of a [`DocumentObject`].
#[derive(Debug)]
pub struct IntoIter {
    inner: indexmap::map::IntoIter<String, Document>,
}

impl Iterator for IntoIter {
    type Item = (String, Document);
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

// ---------- IntoIterator impls ------------------------------------------

impl IntoIterator for DocumentObject {
    type Item = (String, Document);
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            inner: self.inner.into_iter(),
        }
    }
}

impl<'a> IntoIterator for &'a DocumentObject {
    type Item = (&'a String, &'a Document);
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut DocumentObject {
    type Item = (&'a String, &'a mut Document);
    type IntoIter = IterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// ---------- Index<&str> -------------------------------------------------

impl Index<&str> for DocumentObject {
    type Output = Document;
    fn index(&self, key: &str) -> &Document {
        self.get(key)
            .unwrap_or_else(|| panic!("DocumentObject: no entry for key {key:?}"))
    }
}

impl Index<&String> for DocumentObject {
    type Output = Document;
    fn index(&self, key: &String) -> &Document {
        &self[key.as_str()]
    }
}

// ---------- FromIterator / Extend / From --------------------------------

impl FromIterator<(String, Document)> for DocumentObject {
    fn from_iter<I: IntoIterator<Item = (String, Document)>>(iter: I) -> Self {
        Self {
            inner: indexmap::IndexMap::from_iter(iter),
        }
    }
}

impl Extend<(String, Document)> for DocumentObject {
    fn extend<I: IntoIterator<Item = (String, Document)>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}

impl<const N: usize> From<[(String, Document); N]> for DocumentObject {
    fn from(entries: [(String, Document); N]) -> Self {
        Self {
            inner: indexmap::IndexMap::from(entries),
        }
    }
}

impl From<std::collections::HashMap<String, Document>> for DocumentObject {
    /// Converts a [`HashMap`](std::collections::HashMap) to a `DocumentObject`.
    ///
    /// Iteration order in the resulting `DocumentObject` follows the
    /// (unspecified) iteration order of the source `HashMap`. For
    /// callers that care about iteration order, build the
    /// `DocumentObject` directly with `insert` instead.
    fn from(map: std::collections::HashMap<String, Document>) -> Self {
        Self {
            inner: map.into_iter().collect(),
        }
    }
}

// ---------- Optional serde support --------------------------------------

#[cfg(all(aws_sdk_unstable, feature = "serde-serialize"))]
impl serde::Serialize for DocumentObject {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.inner.len()))?;
        for (k, v) in &self.inner {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(all(aws_sdk_unstable, feature = "serde-deserialize"))]
impl<'de> serde::Deserialize<'de> for DocumentObject {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = DocumentObject;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a Document object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut access: M,
            ) -> Result<Self::Value, M::Error> {
                let mut out = DocumentObject::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((k, v)) = access.next_entry::<String, Document>()? {
                    out.insert(k, v);
                }
                Ok(out)
            }
        }
        deserializer.deserialize_map(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_order_is_preserved_on_iteration() {
        let mut obj = DocumentObject::new();
        obj.insert("zebra".to_string(), Document::Bool(true));
        obj.insert("alpha".to_string(), Document::Bool(false));
        obj.insert("middle".to_string(), Document::Null);

        let keys: Vec<&String> = obj.keys().collect();
        assert_eq!(
            keys,
            [
                &"zebra".to_string(),
                &"alpha".to_string(),
                &"middle".to_string()
            ]
        );
    }

    #[test]
    fn replacing_a_key_keeps_its_position() {
        let mut obj = DocumentObject::new();
        obj.insert("first".to_string(), Document::Bool(false));
        obj.insert("second".to_string(), Document::Bool(false));
        obj.insert("third".to_string(), Document::Bool(false));

        let prev = obj.insert("second".to_string(), Document::Bool(true));
        assert_eq!(prev, Some(Document::Bool(false)));

        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(keys, ["first", "second", "third"]);
        assert_eq!(obj["second"], Document::Bool(true));
    }

    #[test]
    fn from_array_preserves_order() {
        let obj = DocumentObject::from([
            ("z".to_string(), Document::Null),
            ("a".to_string(), Document::Null),
            ("m".to_string(), Document::Null),
        ]);
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(keys, ["z", "a", "m"]);
    }

    #[test]
    fn remove_preserves_remaining_order() {
        let mut obj = DocumentObject::from([
            ("first".to_string(), Document::Bool(true)),
            ("second".to_string(), Document::Bool(true)),
            ("third".to_string(), Document::Bool(true)),
        ]);

        obj.remove("second");
        let keys: Vec<&str> = obj.keys().map(String::as_str).collect();
        assert_eq!(keys, ["first", "third"]);
    }

    #[test]
    fn index_panics_on_missing_key() {
        let obj = DocumentObject::new();
        let result = std::panic::catch_unwind(|| {
            let _ = obj["missing"].clone();
        });
        assert!(result.is_err());
    }
}
