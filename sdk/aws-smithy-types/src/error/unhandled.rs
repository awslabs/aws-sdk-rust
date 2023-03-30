/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Unhandled error type.

use crate::error::{metadata::ProvideErrorMetadata, ErrorMetadata};
use std::error::Error as StdError;

/// Builder for [`Unhandled`]
#[derive(Default, Debug)]
pub struct Builder {
    source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    meta: Option<ErrorMetadata>,
}

impl Builder {
    /// Sets the error source
    pub fn source(mut self, source: impl Into<Box<dyn StdError + Send + Sync + 'static>>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the error source
    pub fn set_source(
        &mut self,
        source: Option<Box<dyn StdError + Send + Sync + 'static>>,
    ) -> &mut Self {
        self.source = source;
        self
    }

    /// Sets the error metadata
    pub fn meta(mut self, meta: ErrorMetadata) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Sets the error metadata
    pub fn set_meta(&mut self, meta: Option<ErrorMetadata>) -> &mut Self {
        self.meta = meta;
        self
    }

    /// Builds the unhandled error
    pub fn build(self) -> Unhandled {
        Unhandled {
            source: self.source.expect("unhandled errors must have a source"),
            meta: self.meta.unwrap_or_default(),
        }
    }
}

/// An unexpected error occurred (e.g., invalid JSON returned by the service or an unknown error code).
///
/// When logging an error from the SDK, it is recommended that you either wrap the error in
/// [`DisplayErrorContext`](crate::error::display::DisplayErrorContext), use another
/// error reporter library that visits the error's cause/source chain, or call
/// [`Error::source`](std::error::Error::source) for more details about the underlying cause.
#[derive(Debug)]
pub struct Unhandled {
    source: Box<dyn StdError + Send + Sync + 'static>,
    meta: ErrorMetadata,
}

impl Unhandled {
    /// Returns a builder to construct an unhandled error.
    pub fn builder() -> Builder {
        Default::default()
    }
}

impl std::fmt::Display for Unhandled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "unhandled error")
    }
}

impl StdError for Unhandled {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref() as _)
    }
}

impl ProvideErrorMetadata for Unhandled {
    fn meta(&self) -> &ErrorMetadata {
        &self.meta
    }
}
