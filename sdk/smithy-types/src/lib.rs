/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0.
 */

pub mod instant;
pub mod retry;

use std::collections::HashMap;

pub use crate::instant::Instant;
use crate::retry::{ErrorKind, ProvideErrorKind};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub struct Blob {
    inner: Vec<u8>,
}

impl Blob {
    pub fn new<T: Into<Vec<u8>>>(inp: T) -> Self {
        Blob { inner: inp.into() }
    }
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

/* ANCHOR: document */

/// Document Type
///
/// Document types represents protocol-agnostic open content that is accessed like JSON data.
/// Open content is useful for modeling unstructured data that has no schema, data that can't be
/// modeled using rigid types, or data that has a schema that evolves outside of the purview of a model.
/// The serialization format of a document is an implementation detail of a protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    Object(HashMap<String, Document>),
    Array(Vec<Document>),
    Number(Number),
    String(String),
    Bool(bool),
    Null,
}

/// A number type that implements Javascript / JSON semantics, modeled on serde_json:
/// https://docs.serde.rs/src/serde_json/number.rs.html#20-22
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}

/* ANCHOR_END: document */

/// Generic Error type
///
/// For many services, Errors are modeled. However, many services only partially model errors or don't
/// model errors at all. In these cases, the SDK will return this generic error type to expose the
/// `code`, `message` and `request_id`.
#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct Error {
    pub code: Option<String>,
    pub message: Option<String>,
    pub request_id: Option<String>,
}

impl Error {
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
}

impl ProvideErrorKind for Error {
    fn retryable_error_kind(&self) -> Option<ErrorKind> {
        None
    }

    fn code(&self) -> Option<&str> {
        Error::code(self)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut fmt = f.debug_struct("Error");
        if let Some(code) = &self.code {
            fmt.field("code", code);
        }
        if let Some(message) = &self.message {
            fmt.field("message", message);
        }
        if let Some(req_id) = &self.request_id {
            fmt.field("request_id", req_id);
        }
        Ok(())
    }
}

impl std::error::Error for Error {}
