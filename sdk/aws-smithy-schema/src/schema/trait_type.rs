/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::ShapeId;
use std::any::Any;
use std::fmt;

/// Trait representing a Smithy trait at runtime.
///
/// Traits provide additional metadata about shapes that affect serialization,
/// validation, and other behaviors.
pub trait Trait: Any + Send + Sync + fmt::Debug {
    /// Returns the Shape ID of this trait.
    ///
    /// Returns `&ShapeId<'static>` rather than `&ShapeId<'_>` because the
    /// `Any` supertrait forces all `Trait` implementors to be `'static`, and
    /// `Any` is required for the `Schema::with_traits(LazyLock<TraitMap>)`
    /// downcast fallback.
    ///
    /// The practical consequence: the typed trait wrappers are generic over
    /// `'a` and so usable on runtime-materialized schemas, but they can only
    /// `impl Trait` for their `'static` instantiation. Custom and unknown
    /// traits, which reach a schema through the `dyn Trait` map rather than a
    /// typed accessor, therefore remain `'static`-only. Lifting that means
    /// replacing `Any`-based downcasting with an explicit tagged
    /// representation — a larger change, deliberately out of scope here.
    fn trait_id(&self) -> &ShapeId<'static>;

    /// Returns this trait as `&dyn Any` for downcasting.
    fn as_any(&self) -> &dyn Any;
}

/// An annotation trait (no value), e.g. `@sensitive`, `@sparse`, `@httpPayload`.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used by generated code
pub struct AnnotationTrait {
    id: ShapeId<'static>,
}

#[allow(dead_code)]
impl AnnotationTrait {
    /// Creates a new annotation trait.
    pub fn new(id: ShapeId<'static>) -> Self {
        Self { id }
    }
}

impl Trait for AnnotationTrait {
    fn trait_id(&self) -> &ShapeId<'static> {
        &self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A trait with a string value, e.g. `@jsonName("foo")`, `@xmlName("bar")`.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used by generated code
pub struct StringTrait {
    id: ShapeId<'static>,
    value: String,
}

#[allow(dead_code)]
impl StringTrait {
    /// Creates a new string-valued trait.
    pub fn new(id: ShapeId<'static>, value: impl Into<String>) -> Self {
        Self {
            id,
            value: value.into(),
        }
    }

    /// Returns the string value of this trait.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Trait for StringTrait {
    fn trait_id(&self) -> &ShapeId<'static> {
        &self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A trait with a Document value, used for unknown/custom traits.
///
/// When a trait is included in a schema but has no typed Rust representation,
/// its value is stored as a [`Document`](aws_smithy_types::Document).
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used by generated code
pub struct DocumentTrait {
    id: ShapeId<'static>,
    value: aws_smithy_types::Document,
}

#[allow(dead_code)]
impl DocumentTrait {
    /// Creates a new document-valued trait.
    pub fn new(id: ShapeId<'static>, value: aws_smithy_types::Document) -> Self {
        Self { id, value }
    }

    /// Returns the document value of this trait.
    pub fn value(&self) -> &aws_smithy_types::Document {
        &self.value
    }
}

impl Trait for DocumentTrait {
    fn trait_id(&self) -> &ShapeId<'static> {
        &self.id
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
