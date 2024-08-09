/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use aws_smithy_types::{Blob, DateTime};

/// Macro for delegating method calls to the encoder.
///
/// This macro generates wrapper methods for calling specific encoder methods on the encoder
/// and returning a mutable reference to self for method chaining.
///
/// # Example
///
/// ```ignore
/// delegate_method! {
///     /// Wrapper method for encoding method `encode_str` on the encoder.
///     encode_str_wrapper => encode_str(data: &str);
///     /// Wrapper method for encoding method `encode_int` on the encoder.
///     encode_int_wrapper => encode_int(value: i32);
/// }
/// ```
macro_rules! delegate_method {
    ($($(#[$meta:meta])* $wrapper_name:ident => $encoder_name:ident($($param_name:ident : $param_type:ty),*);)+) => {
        $(
            pub fn $wrapper_name(&mut self, $($param_name: $param_type),*) -> &mut Self {
                self.encoder.$encoder_name($($param_name)*).expect(INFALLIBLE_WRITE);
                self
            }
        )+
    };
}

#[derive(Debug, Clone)]
pub struct Encoder {
    encoder: minicbor::Encoder<Vec<u8>>,
}

/// We always write to a `Vec<u8>`, which is infallible in `minicbor`.
/// <https://docs.rs/minicbor/latest/minicbor/encode/write/trait.Write.html#impl-Write-for-Vec%3Cu8%3E>
const INFALLIBLE_WRITE: &str = "write failed";

impl Encoder {
    pub fn new(writer: Vec<u8>) -> Self {
        Self {
            encoder: minicbor::Encoder::new(writer),
        }
    }

    delegate_method! {
        /// Used when it's not cheap to calculate the size, i.e. when the struct has one or more
        /// `Option`al members.
        begin_map => begin_map();
        /// Writes a definite length string.
        str => str(x: &str);
        /// Writes a boolean value.
        boolean => bool(x: bool);
        /// Writes a byte value.
        byte => i8(x: i8);
        /// Writes a short value.
        short => i16(x: i16);
        /// Writes an integer value.
        integer => i32(x: i32);
        /// Writes an long value.
        long => i64(x: i64);
        /// Writes an float value.
        float => f32(x: f32);
        /// Writes an double value.
        double => f64(x: f64);
        /// Writes a null tag.
        null => null();
        /// Writes an end tag.
        end => end();
    }

    pub fn blob(&mut self, x: &Blob) -> &mut Self {
        self.encoder.bytes(x.as_ref()).expect(INFALLIBLE_WRITE);
        self
    }

    /// Writes a fixed length array of given length.
    pub fn array(&mut self, len: usize) -> &mut Self {
        self.encoder
            // `.expect()` safety: `From<u64> for usize` is not in the standard library,
            // but the conversion should be infallible (unless we ever have 128-bit machines I
            // guess). <See https://users.rust-lang.org/t/cant-convert-usize-to-u64/6243>.
            .array(len.try_into().expect("`usize` to `u64` conversion failed"))
            .expect(INFALLIBLE_WRITE);
        self
    }

    /// Writes a fixed length map of given length.
    /// Used when we know the size in advance, i.e.:
    /// - when a struct has all non-`Option`al members.
    /// - when serializing `union` shapes (they can only have one member set).
    /// - when serializing a `map` shape.
    pub fn map(&mut self, len: usize) -> &mut Self {
        self.encoder
            .map(len.try_into().expect("`usize` to `u64` conversion failed"))
            .expect(INFALLIBLE_WRITE);
        self
    }

    pub fn timestamp(&mut self, x: &DateTime) -> &mut Self {
        self.encoder
            .tag(minicbor::data::Tag::from(
                minicbor::data::IanaTag::Timestamp,
            ))
            .expect(INFALLIBLE_WRITE);
        self.encoder.f64(x.as_secs_f64()).expect(INFALLIBLE_WRITE);
        self
    }

    pub fn into_writer(self) -> Vec<u8> {
        self.encoder.into_writer()
    }
}
