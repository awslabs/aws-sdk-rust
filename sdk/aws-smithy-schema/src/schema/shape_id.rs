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
/// const ID: ShapeId = shape_id!("smithy.api", "String");
/// assert_eq!(ID.as_str(), "smithy.api#String");
/// ```
#[macro_export]
macro_rules! shape_id {
    ($ns:literal, $name:literal) => {
        $crate::ShapeId::from_static(concat!($ns, "#", $name), $ns, $name)
    };
    ($ns:literal, $name:literal, $member:literal) => {
        $crate::ShapeId::from_static_with_member(
            concat!($ns, "#", $name, "$", $member),
            $ns,
            $name,
            $member,
        )
    };
}

/// A Smithy Shape ID.
///
/// Shape IDs uniquely identify shapes in a Smithy model.
/// Use the [`shape_id!`] macro to construct instances — it computes the
/// fully qualified name at compile time from the namespace and shape name,
/// preventing the parts from getting out of sync.
///
/// # Examples
/// ```
/// use aws_smithy_schema::{shape_id, ShapeId};
///
/// const ID: ShapeId = shape_id!("smithy.api", "String");
/// assert_eq!(ID.namespace(), "smithy.api");
/// assert_eq!(ID.shape_name(), "String");
/// assert_eq!(ID.as_str(), "smithy.api#String");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeId {
    fqn: &'static str,
    namespace: &'static str,
    shape_name: &'static str,
    member_name: Option<&'static str>,
}

impl ShapeId {
    /// Creates a ShapeId from pre-computed static strings.
    ///
    /// Prefer the [`shape_id!`] macro which computes `fqn` via `concat!`
    /// to prevent the parts from getting out of sync.
    #[doc(hidden)]
    pub const fn from_static(
        fqn: &'static str,
        namespace: &'static str,
        shape_name: &'static str,
    ) -> Self {
        Self {
            fqn,
            namespace,
            shape_name,
            member_name: None,
        }
    }

    /// Creates a ShapeId with a member name from pre-computed static strings.
    ///
    /// Prefer the [`shape_id!`] macro which computes `fqn` via `concat!`
    /// to prevent the parts from getting out of sync.
    #[doc(hidden)]
    pub const fn from_static_with_member(
        fqn: &'static str,
        namespace: &'static str,
        shape_name: &'static str,
        member_name: &'static str,
    ) -> Self {
        Self {
            fqn,
            namespace,
            shape_name,
            member_name: Some(member_name),
        }
    }

    /// Returns the fully qualified string representation (e.g. `"smithy.api#String"`).
    pub fn as_str(&self) -> &str {
        self.fqn
    }

    /// Returns the namespace portion of the ShapeId.
    pub fn namespace(&self) -> &str {
        self.namespace
    }

    /// Returns the shape name portion of the ShapeId.
    pub fn shape_name(&self) -> &str {
        self.shape_name
    }

    /// Returns the member name if this is a member shape ID.
    pub fn member_name(&self) -> Option<&str> {
        self.member_name
    }
}

impl fmt::Display for ShapeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.fqn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_id_macro() {
        const ID: ShapeId = shape_id!("smithy.api", "String");
        assert_eq!(ID.as_str(), "smithy.api#String");
        assert_eq!(ID.namespace(), "smithy.api");
        assert_eq!(ID.shape_name(), "String");
        assert_eq!(ID.member_name(), None);
    }

    #[test]
    fn test_shape_id_macro_with_member() {
        const ID: ShapeId = shape_id!("com.example", "MyStruct", "field");
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
}
