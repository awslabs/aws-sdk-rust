/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Schema-aware adapters for the [`Document`](aws_smithy_types::Document) data model.
//!
//! The [`Document`](aws_smithy_types::Document), [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument), and [`DocumentSettings`](aws_smithy_types::DocumentSettings)
//! types live in `aws-smithy-types` (the dependency direction is
//! `aws-smithy-schema → aws-smithy-types`); import them from there. This
//! module supplies the schema-crate-only adapters that connect `Document`
//! with the [`ShapeSerializer`](crate::serde::ShapeSerializer) /
//! [`ShapeDeserializer`](crate::serde::ShapeDeserializer) machinery:
//!
//! - [`serializer`] — the [`DocumentShapeSerializer`], a
//!   [`ShapeSerializer`](crate::serde::ShapeSerializer) implementation
//!   that builds a [`Document`](aws_smithy_types::Document) tree from a
//!   typed Smithy shape.
//! - [`deserializer`] — the [`DocumentShapeDeserializer`], a
//!   [`ShapeDeserializer`](crate::serde::ShapeDeserializer) implementation
//!   that walks a [`Document`](aws_smithy_types::Document) tree and
//!   dispatches to consumer callbacks defined by generated `deserialize`
//!   methods.
//! - [`discriminated_ext`] — [`DiscriminatedDocumentExt`], an extension
//!   trait that bolts `Document.from_struct` / `Document.asShape` /
//!   `Document.shape_type` onto
//!   [`DiscriminatedDocument`](aws_smithy_types::DiscriminatedDocument)
//!   (which itself lives in `aws-smithy-types` and so cannot reference
//!   [`Schema`](crate::Schema) inherently).
//!
//! See the SEP "Document Type and Type Registries" for the full
//! specification.

mod deserializer;
mod discriminated_ext;
#[cfg(test)]
mod round_trips;
mod serializer;

pub use deserializer::DocumentShapeDeserializer;
pub use discriminated_ext::DiscriminatedDocumentExt;
pub use serializer::DocumentShapeSerializer;
