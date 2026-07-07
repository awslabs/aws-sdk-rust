/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! HTTP RPC protocol for body-only APIs.

use crate::codec::{Codec, FinishSerializer};
use crate::protocol::{apply_http_endpoint, ClientProtocolInner};
use crate::serde::{SerdeError, SerializableStruct, ShapeDeserializer, ShapeSerializer};
use crate::{Schema, ShapeId};
use aws_smithy_runtime_api::http::{Request, Response};
use aws_smithy_types::body::SdkBody;
use aws_smithy_types::config_bag::ConfigBag;

/// An HTTP protocol for RPC-style APIs that put everything in the body.
///
/// This protocol ignores HTTP binding traits and serializes the entire input
/// into the request body using the provided codec. Used by protocols like
/// `awsJson1_0`, `awsJson1_1`, and `rpcv2Cbor`.
///
/// # Type parameters
///
/// * `C` — the payload codec (ex: `JsonCodec`, `CborCodec`)
#[derive(Debug)]
pub struct HttpRpcProtocol<C> {
    protocol_id: ShapeId,
    codec: C,
    content_type: &'static str,
}

impl<C: Codec> HttpRpcProtocol<C> {
    /// Creates a new HTTP RPC protocol.
    pub fn new(protocol_id: ShapeId, codec: C, content_type: &'static str) -> Self {
        Self {
            protocol_id,
            codec,
            content_type,
        }
    }
}

impl<C> ClientProtocolInner for HttpRpcProtocol<C>
where
    C: Codec + Send + Sync + std::fmt::Debug + 'static,
    for<'a> C::Deserializer<'a>: ShapeDeserializer,
{
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId {
        &self.protocol_id
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema,
        endpoint: &str,
        _cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        let mut serializer = self.codec.create_serializer();
        serializer.write_struct(input_schema, input)?;
        let body = serializer.finish();

        let mut request = Request::new(SdkBody::from(body));
        request
            .set_method("POST")
            .map_err(|e| SerdeError::custom(format!("invalid HTTP method: {e}")))?;
        let uri = if endpoint.is_empty() { "/" } else { endpoint };
        request
            .set_uri(uri)
            .map_err(|e| SerdeError::custom(format!("invalid endpoint URI: {e}")))?;
        request
            .headers_mut()
            .insert("Content-Type", self.content_type);
        if let Some(len) = request.body().content_length() {
            request
                .headers_mut()
                .insert("Content-Length", len.to_string());
        }
        Ok(request)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        _output_schema: &Schema,
        _cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        let body = response
            .body()
            .bytes()
            .ok_or_else(|| SerdeError::custom("response body is not available as bytes"))?;
        Ok(Box::new(self.codec.create_deserializer(body)))
    }

    fn payload_codec(&self) -> Option<&dyn crate::codec::DynCodec> {
        Some(&self.codec)
    }

    fn update_endpoint(
        &self,
        request: &mut Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        apply_http_endpoint(request, endpoint, cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serde::SerializableStruct;
    use crate::{prelude::*, ShapeType};

    struct TestSerializer {
        output: Vec<u8>,
    }

    impl FinishSerializer for TestSerializer {
        fn finish(self) -> Vec<u8> {
            self.output
        }
    }

    impl ShapeSerializer for TestSerializer {
        fn write_struct(
            &mut self,
            _: &Schema,
            value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            self.output.push(b'{');
            value.serialize_members(self)?;
            self.output.push(b'}');
            Ok(())
        }
        fn write_list(
            &mut self,
            _: &Schema,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_map(
            &mut self,
            _: &Schema,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_boolean(&mut self, _: &Schema, _: bool) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_byte(&mut self, _: &Schema, _: i8) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_short(&mut self, _: &Schema, _: i16) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_integer(&mut self, _: &Schema, _: i32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_long(&mut self, _: &Schema, _: i64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_float(&mut self, _: &Schema, _: f32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_double(&mut self, _: &Schema, _: f64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_integer(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_decimal(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_string(&mut self, _: &Schema, v: &str) -> Result<(), SerdeError> {
            self.output.extend_from_slice(v.as_bytes());
            Ok(())
        }
        fn write_blob(&mut self, _: &Schema, _: &aws_smithy_types::Blob) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_timestamp(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_document(
            &mut self,
            _: &Schema,
            _: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_null(&mut self, _: &Schema) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    struct TestDeserializer<'a> {
        input: &'a [u8],
    }

    impl ShapeDeserializer for TestDeserializer<'_> {
        fn read_struct(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(&Schema, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_list(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _: &Schema,
            _: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _: &Schema) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _: &Schema) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _: &Schema) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _: &Schema) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _: &Schema) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _: &Schema) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _: &Schema) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(
            &mut self,
            _: &Schema,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }
        fn read_big_decimal(
            &mut self,
            _: &Schema,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }
        fn read_string(&mut self, _: &Schema) -> Result<String, SerdeError> {
            Ok(String::from_utf8_lossy(self.input).into_owned())
        }
        fn read_blob(&mut self, _: &Schema) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(vec![]))
        }
        fn read_timestamp(&mut self, _: &Schema) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }
        fn read_document(&mut self, _: &Schema) -> Result<aws_smithy_types::Document, SerdeError> {
            Ok(aws_smithy_types::Document::Null)
        }
        fn is_null(&self) -> bool {
            false
        }
        fn container_size(&self) -> Option<usize> {
            None
        }
    }

    #[derive(Debug)]
    struct TestCodec;

    impl Codec for TestCodec {
        type Serializer = TestSerializer;
        type Deserializer<'a> = TestDeserializer<'a>;
        fn create_serializer(&self) -> Self::Serializer {
            TestSerializer { output: Vec::new() }
        }
        fn create_deserializer<'a>(&self, input: &'a [u8]) -> Self::Deserializer<'a> {
            TestDeserializer { input }
        }
    }

    static TEST_SCHEMA: Schema =
        Schema::new(crate::shape_id!("test", "TestStruct"), ShapeType::Structure);

    struct EmptyStruct;
    impl SerializableStruct for EmptyStruct {
        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static NAME_MEMBER: Schema = Schema::new_member(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::String,
        "name",
        0,
    );
    static MEMBERS: &[&Schema] = &[&NAME_MEMBER];
    static STRUCT_WITH_MEMBER: Schema = Schema::new_struct(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::Structure,
        MEMBERS,
    );

    struct NameStruct;
    impl SerializableStruct for NameStruct {
        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&NAME_MEMBER, "Alice")
        }
    }

    #[test]
    fn serialize_sets_content_type() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let request = protocol
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/x-amz-json-1.0"
        );
    }

    #[test]
    fn serialize_body() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let request = protocol
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(request.body().bytes().unwrap(), b"{Alice}");
    }

    #[test]
    fn serialize_empty_endpoint_defaults_to_root() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let request = protocol
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "", &ConfigBag::base())
            .unwrap();
        assert_eq!(request.uri(), "/");
    }

    #[test]
    fn deserialize_response() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let response = Response::new(
            200u16.try_into().unwrap(),
            SdkBody::from(r#"{"result":42}"#),
        );
        let mut deser = protocol
            .deserialize_response(&response, &TEST_SCHEMA, &ConfigBag::base())
            .unwrap();
        assert_eq!(deser.read_string(&STRING).unwrap(), r#"{"result":42}"#);
    }

    #[test]
    fn update_endpoint() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let mut request = protocol
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://old.example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        let endpoint = aws_smithy_types::endpoint::Endpoint::builder()
            .url("https://new.example.com")
            .build();
        protocol
            .update_endpoint(&mut request, &endpoint, &ConfigBag::base())
            .unwrap();
        assert_eq!(request.uri(), "https://new.example.com/");
    }

    #[test]
    fn protocol_id() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("aws.protocols", "awsJson1_0"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        assert_eq!(protocol.protocol_id().as_str(), "aws.protocols#awsJson1_0");
    }
}
