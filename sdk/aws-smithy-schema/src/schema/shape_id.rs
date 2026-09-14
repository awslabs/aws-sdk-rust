/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use std::fmt;

/// Creates a [`ShapeId`] from a namespace and shape name at compile time.
///
/// The fully qualified name (`namespace#ShapeName`) is computed via `concat!`,
/// eliminating the risk of the FQN getting out of sync with the parts.
///
/// # Examples
/// ```
/// use aws_smithy_schema::{shape_id, ShapeId};
///
/// const ID: ShapeId<'static> = shape_id!("smithy.api", "String");
/// assert_eq!(ID.as_str(), "smithy.api#String");
/// ```
#[macro_export]
macro_rules! shape_id {
    ($ns:literal, $name:literal) => {
        $crate::ShapeId::from_parts(concat!($ns, "#", $name), $ns, $name)
    };
    ($ns:literal, $name:literal, $member:literal) => {
        $crate::ShapeId::from_parts_with_member(
            concat!($ns, "#", $name, "$", $member),
            $ns,
            $name,
            $member,
        )
    };
}

/// A Smithy Shape ID, parameterized over the lifetime of its component
/// strings.
///
/// `ShapeId<'static>` is the codegen-emitted form (zero-allocation,
/// const-constructible via the [`shape_id!`] macro). `ShapeId<'a>` for
/// shorter `'a` is constructible from runtime-parsed bytes — for example,
/// from a wire-format `__type` field on a JSON response.
///
/// Use the [`shape_id!`] macro to construct `'static` instances — it
/// computes the fully qualified name at compile time from the namespace
/// and shape name, preventing the parts from getting out of sync.
///
/// # Examples
/// ```
/// use aws_smithy_schema::{shape_id, ShapeId};
///
/// const ID: ShapeId<'static> = shape_id!("smithy.api", "String");
/// assert_eq!(ID.namespace(), "smithy.api");
/// assert_eq!(ID.shape_name(), "String");
/// assert_eq!(ID.as_str(), "smithy.api#String");
/// ```
///
/// # Variance
///
/// `ShapeId<'a>` is **covariant** in `'a` because every field is of the
/// form `&'a str`. This means a `ShapeId<'static>` (codegen-emitted) is
/// usable everywhere a `ShapeId<'a>` is expected, regardless of `'a`.
///
/// # Equality, hashing, and borrow
///
/// Two `ShapeId`s compare equal when their fully qualified names
/// ([`Self::as_str`]) compare equal. The other fields are determined by
/// the FQN for any `ShapeId` constructed via the [`shape_id!`] macro or
/// [`Self::from_parts`] with consistent parts, so this is the same
/// equality you would get from comparing every field.
///
/// FQN-based equality lets [`ShapeId`] act as `Borrow<str>`: a
/// `HashMap<ShapeId<'static>, V>` can be looked up by `&str` (or by
/// any `&ShapeId<'_>` via [`Self::as_str`]) without going through
/// the `'static`-bound storage key. This is what the cross-lifetime
/// lookup APIs on `TypeRegistry` and `TraitMap` rely on.
#[derive(Debug, Clone)]
pub struct ShapeId<'a> {
    fqn: &'a str,
    namespace: &'a str,
    shape_name: &'a str,
    member_name: Option<&'a str>,
}

impl PartialEq for ShapeId<'_> {
    /// Two `ShapeId`s are equal iff their fully qualified names match.
    /// See the type-level docs for why this is sound.
    fn eq(&self, other: &Self) -> bool {
        self.fqn == other.fqn
    }
}

impl Eq for ShapeId<'_> {}

impl std::hash::Hash for ShapeId<'_> {
    /// Hashes the fully qualified name so that `hash(id)` agrees with
    /// `hash(id.as_str())` — required for the `Borrow<str>` impl.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.fqn.hash(state);
    }
}

impl std::borrow::Borrow<str> for ShapeId<'_> {
    /// Borrows the fully qualified name. Combined with the FQN-based
    /// `Hash`/`Eq` impls, this lets `HashMap<ShapeId<'static>, V>::get`
    /// accept `&str` keys, enabling cross-lifetime lookup against a
    /// `'static`-keyed table.
    fn borrow(&self) -> &str {
        self.fqn
    }
}

impl<'a> ShapeId<'a> {
    /// Creates a `ShapeId` from its already-split component strings.
    ///
    /// `const`-callable, so the [`shape_id!`] macro and codegen use it for
    /// `'static` ids; it also accepts borrowed parts for runtime-built ids
    /// (e.g. from a wire-format `__type`). `fqn` must agree with
    /// `namespace`/`shape_name` — the macro guarantees this via `concat!`.
    ///
    /// Prefer the [`shape_id!`] macro for literal ids so the parts cannot get
    /// out of sync.
    #[doc(hidden)]
    pub const fn from_parts(fqn: &'a str, namespace: &'a str, shape_name: &'a str) -> Self {
        Self {
            fqn,
            namespace,
            shape_name,
            member_name: None,
        }
    }

    /// Creates a member `ShapeId` from its already-split component strings.
    ///
    /// See [`Self::from_parts`]; this is the member-shape variant.
    ///
    /// Prefer the [`shape_id!`] macro for literal ids so the parts cannot get
    /// out of sync.
    #[doc(hidden)]
    pub const fn from_parts_with_member(
        fqn: &'a str,
        namespace: &'a str,
        shape_name: &'a str,
        member_name: &'a str,
    ) -> Self {
        Self {
            fqn,
            namespace,
            shape_name,
            member_name: Some(member_name),
        }
    }

    /// Returns the fully qualified string representation (e.g. `"smithy.api#String"`).
    ///
    /// The return type is `&'a str` (the lifetime of the data, not the
    /// receiver). Calling this on `ShapeId<'static>` yields `&'static str`,
    /// preserving the codegen-side `'static` accessor guarantee.
    pub fn as_str(&self) -> &'a str {
        self.fqn
    }

    /// Returns the namespace portion of the ShapeId.
    ///
    /// Lifetime is `'a` — see [`Self::as_str`] for rationale.
    pub fn namespace(&self) -> &'a str {
        self.namespace
    }

    /// Returns the shape name portion of the ShapeId.
    ///
    /// Lifetime is `'a` — see [`Self::as_str`] for rationale.
    pub fn shape_name(&self) -> &'a str {
        self.shape_name
    }

    /// Returns the member name if this is a member shape ID.
    ///
    /// Lifetime is `'a` — see [`Self::as_str`] for rationale.
    pub fn member_name(&self) -> Option<&'a str> {
        self.member_name
    }
}

impl fmt::Display for ShapeId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.fqn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_id_macro() {
        const ID: ShapeId<'static> = shape_id!("smithy.api", "String");
        assert_eq!(ID.as_str(), "smithy.api#String");
        assert_eq!(ID.namespace(), "smithy.api");
        assert_eq!(ID.shape_name(), "String");
        assert_eq!(ID.member_name(), None);
    }

    #[test]
    fn test_shape_id_macro_with_member() {
        const ID: ShapeId<'static> = shape_id!("com.example", "MyStruct", "field");
        assert_eq!(ID.as_str(), "com.example#MyStruct$field");
        assert_eq!(ID.namespace(), "com.example");
        assert_eq!(ID.shape_name(), "MyStruct");
        assert_eq!(ID.member_name(), Some("field"));
    }

    #[test]
    fn test_display() {
        let id = shape_id!("smithy.api", "String");
        assert_eq!(format!("{id}"), "smithy.api#String");
    }

    #[test]
    fn test_equality() {
        let a = shape_id!("smithy.api", "String");
        let b = shape_id!("smithy.api", "String");
        assert_eq!(a, b);

        let c = shape_id!("smithy.api", "String", "foo");
        let d = shape_id!("smithy.api", "String", "foo");
        assert_eq!(c, d);
    }

    /// Construct a `ShapeId<'a>` from non-`'static` data. Compile-tests
    /// that the lifetime parameter does what we want — without this the
    /// runtime-built ShapeId path would not be exercised by tests.
    #[test]
    fn test_runtime_lifetime() {
        let fqn = String::from("ns#Foo");
        let ns = String::from("ns");
        let name = String::from("Foo");
        let id: ShapeId<'_> = ShapeId::from_parts(&fqn, &ns, &name);
        assert_eq!(id.as_str(), "ns#Foo");
        assert_eq!(id.namespace(), "ns");
        assert_eq!(id.shape_name(), "Foo");
    }

    /// `ShapeId<'static>` and a runtime `ShapeId<'a>` with the same FQN
    /// must compare equal and hash identically. This is what makes
    /// cross-lifetime lookup against a `'static`-keyed table sound.
    #[test]
    fn equality_across_lifetimes() {
        let static_id: ShapeId<'static> = shape_id!("ns", "Foo");
        let owned_fqn = String::from("ns#Foo");
        let owned_ns = String::from("ns");
        let owned_name = String::from("Foo");
        let runtime_id: ShapeId<'_> = ShapeId::from_parts(&owned_fqn, &owned_ns, &owned_name);

        assert_eq!(static_id, runtime_id);

        // Hash agreement.
        let mut h1 = std::collections::hash_map::DefaultHasher::new();
        let mut h2 = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(&static_id, &mut h1);
        std::hash::Hash::hash(&runtime_id, &mut h2);
        assert_eq!(
            std::hash::Hasher::finish(&h1),
            std::hash::Hasher::finish(&h2),
        );
    }

    /// `Borrow<str>` plus FQN-only `Hash`/`Eq` lets us look up a
    /// `HashMap<ShapeId<'static>, V>` with a bare `&str` key.
    #[test]
    fn hash_map_lookup_by_str_and_runtime_id() {
        use std::collections::HashMap;

        let static_id: ShapeId<'static> = shape_id!("ns", "Foo");
        let mut map: HashMap<ShapeId<'static>, u32> = HashMap::new();
        map.insert(static_id, 42);

        // Lookup by `&str` (cross-lifetime).
        assert_eq!(map.get("ns#Foo"), Some(&42));

        // Lookup by a runtime-built `ShapeId<'_>` (cross-lifetime).
        let owned_fqn = String::from("ns#Foo");
        let owned_ns = String::from("ns");
        let owned_name = String::from("Foo");
        let runtime_id: ShapeId<'_> = ShapeId::from_parts(&owned_fqn, &owned_ns, &owned_name);
        assert_eq!(map.get(runtime_id.as_str()), Some(&42));

        // Negative case.
        assert_eq!(map.get("ns#Bar"), None);
    }
}
