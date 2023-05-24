/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::Number;
use std::collections::HashMap;

/* ANCHOR: document */

/// Document Type
///
/// Document types represents protocol-agnostic open content that is accessed like JSON data.
/// Open content is useful for modeling unstructured data that has no schema, data that can't be
/// modeled using rigid types, or data that has a schema that evolves outside of the purview of a model.
/// The serialization format of a document is an implementation detail of a protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    /// JSON object
    Object(HashMap<String, Document>),
    /// JSON array
    Array(Vec<Document>),
    /// JSON number
    Number(Number),
    /// JSON string
    String(String),
    /// JSON boolean
    Bool(bool),
    /// JSON null
    Null,
}

impl From<bool> for Document {
    fn from(value: bool) -> Self {
        Document::Bool(value)
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Document::String(value)
    }
}

impl From<Vec<Document>> for Document {
    fn from(values: Vec<Document>) -> Self {
        Document::Array(values)
    }
}

impl From<HashMap<String, Document>> for Document {
    fn from(values: HashMap<String, Document>) -> Self {
        Document::Object(values)
    }
}

impl From<u64> for Document {
    fn from(value: u64) -> Self {
        Document::Number(Number::PosInt(value))
    }
}

impl From<i64> for Document {
    fn from(value: i64) -> Self {
        Document::Number(Number::NegInt(value))
    }
}

impl From<i32> for Document {
    fn from(value: i32) -> Self {
        Document::Number(Number::NegInt(value as i64))
    }
}
