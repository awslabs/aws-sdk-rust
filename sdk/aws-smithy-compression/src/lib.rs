/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

/* Automatically managed default lints */
#![cfg_attr(docsrs, feature(doc_cfg))]
/* End of automatically managed default lints */
#![allow(clippy::derive_partial_eq_without_eq)]
#![warn(
    missing_docs,
    rustdoc::missing_crate_level_docs,
    unreachable_pub,
    rust_2018_idioms
)]

//! Compression-related code.

use aws_smithy_runtime_api::box_error::BoxError;
use aws_smithy_types::config_bag::{Storable, StoreReplace};
use std::io::Write;
use std::str::FromStr;

pub mod body;
mod gzip;
pub mod http;

// Valid compression algorithm names
/// The name of the `gzip` algorithm.
pub const GZIP_NAME: &str = "gzip";

/// The maximum-allowable value per internal standards is 10 Megabytes.
const MAX_MIN_COMPRESSION_SIZE_BYTES: u32 = 10_485_760;

/// Types implementing this trait can compress data.
///
/// Compression algorithms are used reduce the size of data. This trait
/// requires Send + Sync because trait implementors are often used in an
/// async context.
pub trait Compress: Send + Sync {
    /// Given a slice of bytes, and a [Write] implementor, compress and write
    /// bytes to the writer until done.
    // I wanted to use `impl Write` but that's not object-safe
    fn compress_bytes(&mut self, bytes: &[u8], writer: &mut dyn Write) -> Result<(), BoxError>;
}

/// Options for configuring request compression.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct CompressionOptions {
    /// Valid values are 0-9 with lower values configuring less (but faster) compression
    level: u32,
    min_compression_size_bytes: u32,
    enabled: bool,
}

impl Default for CompressionOptions {
    fn default() -> Self {
        Self {
            level: 6,
            min_compression_size_bytes: 10240,
            enabled: true,
        }
    }
}

impl CompressionOptions {
    /// The compression level to use.
    pub fn level(&self) -> u32 {
        self.level
    }

    /// The minimum size of data to compress.
    ///
    /// Data smaller than this will not be compressed.
    pub fn min_compression_size_bytes(&self) -> u32 {
        self.min_compression_size_bytes
    }

    /// Whether compression is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set whether compression is enabled.
    pub fn with_enabled(self, enabled: bool) -> Self {
        Self { enabled, ..self }
    }

    /// Set the compression level.
    ///
    /// Valid values are `0..=9` with lower values configuring less _(but faster)_ compression
    pub fn with_level(self, level: u32) -> Result<Self, BoxError> {
        Self::validate_level(level)?;
        Ok(Self { level, ..self })
    }

    /// Set the minimum size of data to compress.
    ///
    /// Data smaller than this will not be compressed.
    /// Valid values are `0..=10_485_760`. The default is `10_240`.
    pub fn with_min_compression_size_bytes(
        self,
        min_compression_size_bytes: u32,
    ) -> Result<Self, BoxError> {
        Self::validate_min_compression_size_bytes(min_compression_size_bytes)?;
        Ok(Self {
            min_compression_size_bytes,
            ..self
        })
    }

    fn validate_level(level: u32) -> Result<(), BoxError> {
        if level > 9 {
            return Err(
                format!("compression level `{level}` is invalid, valid values are 0..=9").into(),
            );
        };
        Ok(())
    }

    fn validate_min_compression_size_bytes(
        min_compression_size_bytes: u32,
    ) -> Result<(), BoxError> {
        if min_compression_size_bytes > MAX_MIN_COMPRESSION_SIZE_BYTES {
            return Err(format!(
                "min compression size `{min_compression_size_bytes}` is invalid, valid values are 0..=10_485_760"
            )
            .into());
        };
        Ok(())
    }
}

impl Storable for CompressionOptions {
    type Storer = StoreReplace<Self>;
}

/// An enum encompassing all supported compression algorithms.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CompressionAlgorithm {
    /// The [gzip](https://en.wikipedia.org/wiki/Gzip) compression algorithm
    Gzip,
}

impl FromStr for CompressionAlgorithm {
    type Err = BoxError;

    /// Create a new `CompressionAlgorithm` from an algorithm name.
    ///
    /// Valid algorithm names are:
    /// - "gzip"
    ///
    /// Passing an invalid name will return an error.
    fn from_str(compression_algorithm: &str) -> Result<Self, Self::Err> {
        if compression_algorithm.eq_ignore_ascii_case(GZIP_NAME) {
            Ok(Self::Gzip)
        } else {
            Err(format!("unknown compression algorithm `{compression_algorithm}`").into())
        }
    }
}

impl CompressionAlgorithm {
    /// Return the `HttpChecksum` implementor for this algorithm.
    pub fn into_impl_http_body_1_x(
        self,
        options: &CompressionOptions,
    ) -> Box<dyn http::CompressRequest> {
        match self {
            Self::Gzip => Box::new(gzip::Gzip::from(options)),
        }
    }

    /// Return the name of this algorithm in string form
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gzip { .. } => GZIP_NAME,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CompressionAlgorithm;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_compression_algorithm_from_str_unknown() {
        let error = "some unknown compression algorithm"
            .parse::<CompressionAlgorithm>()
            .expect_err("it should error");
        assert_eq!(
            "unknown compression algorithm `some unknown compression algorithm`",
            error.to_string()
        );
    }

    #[test]
    fn test_compression_algorithm_from_str_gzip() {
        let algo = "gzip".parse::<CompressionAlgorithm>().unwrap();
        assert_eq!("gzip", algo.as_str());
    }
}
