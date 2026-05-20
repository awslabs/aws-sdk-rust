/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Codec trait for creating shape serializers and deserializers.
//!
//! A codec represents a specific serialization format (e.g., JSON, XML, CBOR)
//! and provides methods to create serializers and deserializers for that format.

pub mod http_string;

use crate::serde::{ShapeDeserializer, ShapeSerializer};

/// Trait for serializers that can produce a final byte output.
///
/// This is separate from [`ShapeSerializer`] to preserve object safety on
/// [`ShapeSerializer`] (which is used as `&mut dyn ShapeSerializer` in generated code).
///
/// # Why isn't `FinishSerializer` itself object-safe?
///
/// [`FinishSerializer::finish`] takes `self` by value so it can consume and tear down the
/// serializer (e.g., return an owned `Vec<u8>` without a leftover borrow on the serializer's
/// internal buffer). Methods that receive `self` by value are not dispatchable through a
/// trait object: `dyn FinishSerializer` doesn't know the concrete size of `Self`, so it
/// cannot move it. This is the standard Rust object-safety restriction.
///
/// The consequence is that `FinishSerializer` can only be used with a statically-known
/// serializer type, which is fine for generated code that knows the concrete [`Codec`].
/// For call sites that need dynamic dispatch (e.g., event stream marshallers that receive
/// a `Box<dyn PayloadSerializer>` from `ClientProtocol::payload_codec`), use
/// [`PayloadSerializer::finish_boxed`] instead — it takes `self: Box<Self>`, which *is*
/// object-safe because the `Box` owns the value and knows how to drop it.
pub trait FinishSerializer {
    /// Consumes the serializer and returns the serialized bytes.
    fn finish(self) -> Vec<u8>;
}

/// A codec for a specific serialization format.
///
/// Codecs are responsible for creating [`ShapeSerializer`] and [`ShapeDeserializer`]
/// instances that can serialize and deserialize shapes to and from a specific format.
///
/// # Examples
///
/// Implementing a custom codec:
///
/// ```ignore
/// use aws_smithy_schema::codec::Codec;
/// use aws_smithy_schema::serde::{ShapeSerializer, ShapeDeserializer};
///
/// struct MyCodec {
///     // codec configuration
/// }
///
/// impl Codec for MyCodec {
///     type Serializer = MySerializer;
///     type Deserializer = MyDeserializer;
///
///     fn create_serializer(&self) -> Self::Serializer {
///         MySerializer::new()
///     }
///
///     fn create_deserializer(&self, input: &[u8]) -> Self::Deserializer {
///         MyDeserializer::new(input)
///     }
/// }
/// ```
pub trait Codec {
    /// The serializer type for this codec.
    type Serializer: ShapeSerializer + FinishSerializer;

    /// The deserializer type for this codec.
    type Deserializer<'a>: ShapeDeserializer + 'a;

    /// Creates a new serializer for this codec.
    fn create_serializer(&self) -> Self::Serializer;

    /// Creates a new deserializer for this codec from the given input bytes.
    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a>;
}

/// Object-safe view of a codec's serializer.
///
/// Combines [`ShapeSerializer`] with an object-safe finish operation. [`FinishSerializer::finish`]
/// takes `self` by value, which cannot be called through `&mut dyn ShapeSerializer` — so this
/// trait exposes `finish(self: Box<Self>)` instead, which is object-safe.
///
/// A blanket impl is provided for every `ShapeSerializer + FinishSerializer`, so every concrete
/// codec serializer (e.g., `JsonSerializer`) is automatically usable through `Box<dyn PayloadSerializer>`
/// without requiring codec authors to write extra impls.
pub trait PayloadSerializer: ShapeSerializer {
    /// Consumes this boxed serializer and returns the serialized bytes.
    fn finish_boxed(self: Box<Self>) -> Vec<u8>;
}

impl<S> PayloadSerializer for S
where
    S: ShapeSerializer + FinishSerializer,
{
    fn finish_boxed(self: Box<Self>) -> Vec<u8> {
        <S as FinishSerializer>::finish(*self)
    }
}

/// Object-safe sibling of [`Codec`] exposing dynamic deserializer creation.
///
/// # Why a sibling trait?
///
/// [`Codec`] uses associated types (`Serializer`, `Deserializer<'a>`) and
/// returns them by value from its methods. This gives codec consumers
/// zero-cost static dispatch — the compiler can inline and monomorphize
/// serializer/deserializer creation at call sites that know the concrete
/// codec type. That is the right choice for the common case (generated code
/// that knows the protocol statically).
///
/// However, some features require accessing a codec through a trait object.
/// The SEP-specified `ClientProtocol::payload_codec()` method returns "the
/// codec" in a context where the `ClientProtocol` itself is accessed via
/// `dyn ClientProtocol` (see [`SharedClientProtocol`](crate::protocol::SharedClientProtocol),
/// which stores `Arc<dyn ClientProtocol>` for runtime protocol swapping).
/// Returning a [`Codec`] through `dyn` is not possible in Rust because
/// associated types and by-value returns are not object-safe.
///
/// `DynCodec` is the minimal object-safe view that covers the operations
/// needed through a trait object. It exists purely as a Rust adaptation of
/// the SEP's object-oriented `Codec` design; it is not additional user-facing
/// API. A blanket `impl<C: Codec> DynCodec for C` makes every concrete codec
/// automatically usable through `&dyn DynCodec` without any extra work from
/// codec authors.
///
/// Both deserializer and serializer creation are exposed through this trait
/// to support event-stream marshalling (input streams) and unmarshalling
/// (output streams) through `dyn ClientProtocol`.
///
/// # Returning (de)serializers as boxed trait objects
///
/// [`ShapeDeserializer`] implementations typically hold cursor state over an
/// input byte slice (e.g., `JsonDeserializer` holds `input: &'a [u8]` and a
/// `position: usize`). Producing a fresh deserializer positioned at the start
/// of the input is the standard way to read independent messages — as is
/// required for event-stream frames, where each frame is an independent
/// serialized payload. The returned `Box<dyn ShapeDeserializer + 'a>`
/// borrows from `input`, so the caller retains ownership of the bytes for
/// the duration of deserialization. Similarly each event frame requires a
/// fresh serializer.
pub trait DynCodec: Send + Sync + std::fmt::Debug {
    /// Creates a new deserializer over the given input bytes.
    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Box<dyn ShapeDeserializer + 'a>;

    /// Creates a new serializer. Use [`PayloadSerializer::finish_boxed`] to
    /// consume the serializer and obtain the serialized bytes.
    fn create_serializer(&self) -> Box<dyn PayloadSerializer + '_>;
}

// Blanket implementation: any statically-dispatched `Codec` is automatically
// available as a `DynCodec`. The boxed (de)serializer here incurs one heap
// allocation per call, which is acceptable for the per-event-frame use case.
// Callers using `Codec` directly pay no such cost.
impl<C> DynCodec for C
where
    C: Codec + Send + Sync + std::fmt::Debug,
{
    fn create_deserializer<'a>(&self, input: &'a [u8]) -> Box<dyn ShapeDeserializer + 'a> {
        Box::new(<C as Codec>::create_deserializer(self, input))
    }

    fn create_serializer(&self) -> Box<dyn PayloadSerializer + '_> {
        Box::new(<C as Codec>::create_serializer(self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
    use crate::{prelude::*, Schema};

    // Mock serializer
    struct MockSerializer {
        output: Vec<u8>,
    }

    impl MockSerializer {
        fn finish(self) -> Vec<u8> {
            self.output
        }
    }

    impl FinishSerializer for MockSerializer {
        fn finish(self) -> Vec<u8> {
            self.output
        }
    }

    impl ShapeSerializer for MockSerializer {
        fn write_struct(
            &mut self,
            _schema: &Schema,
            _value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_list(
            &mut self,
            _schema: &Schema,
            _write_elements: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_map(
            &mut self,
            _schema: &Schema,
            _write_entries: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_boolean(&mut self, _schema: &Schema, _value: bool) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_byte(&mut self, _schema: &Schema, _value: i8) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_short(&mut self, _schema: &Schema, _value: i16) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_integer(&mut self, _schema: &Schema, _value: i32) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_long(&mut self, _schema: &Schema, _value: i64) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_float(&mut self, _schema: &Schema, _value: f32) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_double(&mut self, _schema: &Schema, _value: f64) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_big_integer(
            &mut self,
            _schema: &Schema,
            _value: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_big_decimal(
            &mut self,
            _schema: &Schema,
            _value: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_string(&mut self, _schema: &Schema, _value: &str) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_blob(
            &mut self,
            _schema: &Schema,
            _value: &aws_smithy_types::Blob,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_timestamp(
            &mut self,
            _schema: &Schema,
            _value: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_document(
            &mut self,
            _schema: &Schema,
            _value: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn write_null(&mut self, _schema: &Schema) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    // Mock deserializer
    struct MockDeserializer<'a> {
        #[allow(dead_code)]
        input: &'a [u8],
    }

    impl<'a> ShapeDeserializer for MockDeserializer<'a> {
        fn read_struct(
            &mut self,
            _schema: &Schema,
            _consumer: &mut dyn FnMut(
                &Schema,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_list(
            &mut self,
            _schema: &Schema,
            _consumer: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_map(
            &mut self,
            _schema: &Schema,
            _consumer: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }

        fn read_boolean(&mut self, _schema: &Schema) -> Result<bool, SerdeError> {
            Ok(false)
        }

        fn read_byte(&mut self, _schema: &Schema) -> Result<i8, SerdeError> {
            Ok(0)
        }

        fn read_short(&mut self, _schema: &Schema) -> Result<i16, SerdeError> {
            Ok(0)
        }

        fn read_integer(&mut self, _schema: &Schema) -> Result<i32, SerdeError> {
            Ok(0)
        }

        fn read_long(&mut self, _schema: &Schema) -> Result<i64, SerdeError> {
            Ok(0)
        }

        fn read_float(&mut self, _schema: &Schema) -> Result<f32, SerdeError> {
            Ok(0.0)
        }

        fn read_double(&mut self, _schema: &Schema) -> Result<f64, SerdeError> {
            Ok(0.0)
        }

        fn read_big_integer(
            &mut self,
            _schema: &Schema,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }

        fn read_big_decimal(
            &mut self,
            _schema: &Schema,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }

        fn read_string(&mut self, _schema: &Schema) -> Result<String, SerdeError> {
            Ok(String::new())
        }

        fn read_blob(&mut self, _schema: &Schema) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(Vec::new()))
        }

        fn read_timestamp(
            &mut self,
            _schema: &Schema,
        ) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }

        fn read_document(
            &mut self,
            _schema: &Schema,
        ) -> Result<aws_smithy_types::Document, SerdeError> {
            Ok(aws_smithy_types::Document::Null)
        }

        fn is_null(&self) -> bool {
            false
        }

        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    // Mock codec
    struct MockCodec;

    impl Codec for MockCodec {
        type Serializer = MockSerializer;
        type Deserializer<'a> = MockDeserializer<'a>;

        fn create_serializer(&self) -> Self::Serializer {
            MockSerializer { output: Vec::new() }
        }

        fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
            MockDeserializer { input }
        }
    }

    #[test]
    fn test_codec_create_serializer() {
        let codec = MockCodec;
        let mut serializer = codec.create_serializer();

        // Test that we can use the serializer
        serializer.write_string(&STRING, "test").unwrap();
        let output = serializer.finish();
        assert_eq!(output, Vec::<u8>::new());
    }

    #[test]
    fn test_codec_create_deserializer() {
        let codec = MockCodec;
        let input = b"test data";
        let mut deserializer = codec.create_deserializer(input);

        // Test that we can use the deserializer
        let result = deserializer.read_string(&STRING).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_codec_roundtrip() {
        let codec = MockCodec;

        // Serialize
        let mut serializer = codec.create_serializer();
        serializer.write_integer(&INTEGER, 42).unwrap();
        let bytes = serializer.finish();

        // Deserialize
        let mut deserializer = codec.create_deserializer(&bytes);
        let value = deserializer.read_integer(&INTEGER).unwrap();
        assert_eq!(value, 0); // Mock deserializer always returns 0
    }
}
