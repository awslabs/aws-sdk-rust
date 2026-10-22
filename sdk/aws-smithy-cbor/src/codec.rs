/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! CBOR codec implementation for schema-based serialization.

use aws_smithy_schema::codec::Codec;

mod deserializer;
mod serializer;

pub use deserializer::CborDeserializer;
pub use serializer::{CborSerializer, StructPrefix};

/// Configuration for CBOR codec behavior.
#[derive(Debug, Clone)]
pub struct CborCodecSettings {
    max_depth: u32,
    enforce_strictness: bool,
    use_definite_lengths: bool,
}

impl CborCodecSettings {
    /// Uses definite-length encoding for collections whose element or entry
    /// count is supplied by the caller. Unknown lengths remain indefinite.
    /// Disabled by default.
    pub fn use_definite_lengths(mut self, value: bool) -> Self {
        self.use_definite_lengths = value;
        self
    }

    /// Rejects trailing bytes after top-level containers and truncated nested structures.
    /// Disabled by default to preserve client deserialization behavior.
    pub fn enforce_strictness(mut self, value: bool) -> Self {
        self.enforce_strictness = value;
        self
    }

    /// Maximum aggregate nesting depth the deserializer will accept.
    pub fn max_depth(&self) -> u32 {
        self.max_depth
    }
}

impl Default for CborCodecSettings {
    fn default() -> Self {
        Self {
            max_depth: 128,
            enforce_strictness: false,
            use_definite_lengths: false,
        }
    }
}

/// CBOR codec for schema-based serialization and deserialization.
#[derive(Debug)]
pub struct CborCodec {
    settings: CborCodecSettings,
}

impl CborCodec {
    /// Creates a new CBOR codec with the given settings.
    pub fn new(settings: CborCodecSettings) -> Self {
        Self { settings }
    }
}

impl Default for CborCodec {
    fn default() -> Self {
        Self::new(CborCodecSettings::default())
    }
}

impl Codec for CborCodec {
    type Serializer = CborSerializer;
    type Deserializer<'a> = CborDeserializer<'a>;

    fn create_serializer(&self) -> Self::Serializer {
        CborSerializer::new().with_definite_lengths(self.settings.use_definite_lengths)
    }

    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
        CborDeserializer::new(input, self.settings.max_depth)
            .with_strictness(self.settings.enforce_strictness)
    }
}
