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
    protocol_id: ShapeId<'static>,
    codec: C,
    content_type: &'static str,
}

impl<C: Codec> HttpRpcProtocol<C> {
    /// Creates a new HTTP RPC protocol.
    pub fn new(protocol_id: ShapeId<'static>, codec: C, content_type: &'static str) -> Self {
        Self {
            protocol_id,
            codec,
            content_type,
        }
    }

    /// Returns a reference to the body codec. Used by wrapper protocols
    /// that need to read the codec's settings before rebuilding it via
    /// [`Self::with_codec`].
    pub fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns the Content-Type string this protocol stamps onto the
    /// outgoing request. Used by wrapper protocols that rebuild the
    /// inner [`HttpRpcProtocol`] when reconfiguring the codec.
    pub fn content_type(&self) -> &'static str {
        self.content_type
    }

    /// Replaces the body codec, returning a new protocol instance
    /// with all other fields preserved. Used by wrapper protocols
    /// (e.g. AWS JSON RPC) that need to swap in a reconfigured codec.
    pub fn with_codec(self, codec: C) -> Self {
        Self {
            protocol_id: self.protocol_id,
            codec,
            content_type: self.content_type,
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

    fn protocol_id(&self) -> &ShapeId<'static> {
        &self.protocol_id
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
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

        // A presigning interceptor (or any other caller that stored a
        // `SharedHeaderOmitSettings` in the config bag) can request that these
        // protocol-default headers be suppressed so they don't end up in the
        // signed-header set of a presigned URL. Mirrors
        // `HttpBindingProtocol::serialize_request_with_body`.
        let omit = cfg.load::<crate::header_omit_settings::SharedHeaderOmitSettings>();
        let omit_content_type = omit
            .map(|s| s.should_omit_default_content_type())
            .unwrap_or(false);
        let omit_content_length = omit
            .map(|s| s.should_omit_default_content_length())
            .unwrap_or(false);
        if !omit_content_type {
            request
                .headers_mut()
                .insert("Content-Type", self.content_type);
        }
        if !omit_content_length {
            if let Some(len) = request.body().content_length() {
                request
                    .headers_mut()
                    .insert("Content-Length", len.to_string());
            }
        }
        Ok(request)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        _output_schema: &Schema<'_>,
        _cfg: &'a ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        // A streaming output's body has already been taken by the caller, so
        // `bytes()` is `None`. The `&[]` slice reads as an empty structure, so
        // no body member is populated; for event streams the initial-response
        // members arrive later in the first frame. HTTP bindings are
        // deliberately ignored: this protocol is body-only.
        let body = response.body().bytes().unwrap_or(&[]);
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
            _: &Schema<'_>,
            value: &dyn SerializableStruct,
        ) -> Result<(), SerdeError> {
            self.output.push(b'{');
            value.serialize_members(self)?;
            self.output.push(b'}');
            Ok(())
        }
        fn write_list(
            &mut self,
            _: &Schema<'_>,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_map(
            &mut self,
            _: &Schema<'_>,
            _: &dyn Fn(&mut dyn ShapeSerializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_boolean(&mut self, _: &Schema<'_>, _: bool) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_byte(&mut self, _: &Schema<'_>, _: i8) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_short(&mut self, _: &Schema<'_>, _: i16) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_integer(&mut self, _: &Schema<'_>, _: i32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_long(&mut self, _: &Schema<'_>, _: i64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_float(&mut self, _: &Schema<'_>, _: f32) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_double(&mut self, _: &Schema<'_>, _: f64) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_integer(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::BigInteger,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_big_decimal(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::BigDecimal,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_string(&mut self, _: &Schema<'_>, v: &str) -> Result<(), SerdeError> {
            self.output.extend_from_slice(v.as_bytes());
            Ok(())
        }
        fn write_blob(
            &mut self,
            _: &Schema<'_>,
            _: aws_smithy_types::Blob,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_timestamp(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::DateTime,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_document(
            &mut self,
            _: &Schema<'_>,
            _: &aws_smithy_types::Document,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn write_null(&mut self, _: &Schema<'_>) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    struct TestDeserializer<'a> {
        input: &'a [u8],
    }

    impl ShapeDeserializer for TestDeserializer<'_> {
        /// Reports every member of the schema, as a real codec would for a body that carried
        /// them all.
        ///
        /// A stub that ignored the consumer could not discriminate *where* a member's value
        /// came from, which is exactly what the body-only protocol tests below need to observe:
        /// an RPC protocol must read an `@httpHeader` member from the body representation and
        /// never from the header.
        fn read_struct(
            &mut self,
            schema: &Schema<'_>,
            consumer: &mut dyn FnMut(
                &Schema<'_>,
                &mut dyn ShapeDeserializer,
            ) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            for member in schema.members() {
                consumer(member, self)?;
            }
            Ok(())
        }
        fn read_list(
            &mut self,
            _: &Schema<'_>,
            _: &mut dyn FnMut(&mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_map(
            &mut self,
            _: &Schema<'_>,
            _: &mut dyn FnMut(String, &mut dyn ShapeDeserializer) -> Result<(), SerdeError>,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
        fn read_boolean(&mut self, _: &Schema<'_>) -> Result<bool, SerdeError> {
            Ok(false)
        }
        fn read_byte(&mut self, _: &Schema<'_>) -> Result<i8, SerdeError> {
            Ok(0)
        }
        fn read_short(&mut self, _: &Schema<'_>) -> Result<i16, SerdeError> {
            Ok(0)
        }
        fn read_integer(&mut self, _: &Schema<'_>) -> Result<i32, SerdeError> {
            Ok(0)
        }
        fn read_long(&mut self, _: &Schema<'_>) -> Result<i64, SerdeError> {
            Ok(0)
        }
        fn read_float(&mut self, _: &Schema<'_>) -> Result<f32, SerdeError> {
            Ok(0.0)
        }
        fn read_double(&mut self, _: &Schema<'_>) -> Result<f64, SerdeError> {
            Ok(0.0)
        }
        fn read_big_integer(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigInteger, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigInteger::from_str("0").unwrap())
        }
        fn read_big_decimal(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::BigDecimal, SerdeError> {
            use std::str::FromStr;
            Ok(aws_smithy_types::BigDecimal::from_str("0").unwrap())
        }
        fn read_string(&mut self, _: &Schema<'_>) -> Result<String, SerdeError> {
            Ok(String::from_utf8_lossy(self.input).into_owned())
        }
        fn read_blob(&mut self, _: &Schema<'_>) -> Result<aws_smithy_types::Blob, SerdeError> {
            Ok(aws_smithy_types::Blob::new(vec![]))
        }
        fn read_timestamp(
            &mut self,
            _: &Schema<'_>,
        ) -> Result<aws_smithy_types::DateTime, SerdeError> {
            Ok(aws_smithy_types::DateTime::from_secs(0))
        }
        fn read_document(
            &mut self,
            _: &Schema<'_>,
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

    static TEST_SCHEMA: Schema<'static> =
        Schema::new(crate::shape_id!("test", "TestStruct"), ShapeType::Structure);

    struct EmptyStruct;
    impl SerializableStruct for EmptyStruct {
        fn schema(&self) -> &Schema<'_> {
            &TEST_SCHEMA
        }

        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static NAME_MEMBER: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::String,
        "name",
        0,
    );
    static MEMBERS: &[&Schema<'_>] = &[&NAME_MEMBER];
    static STRUCT_WITH_MEMBER: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "TestStruct"),
        ShapeType::Structure,
        MEMBERS,
    );

    struct NameStruct;
    impl SerializableStruct for NameStruct {
        fn schema(&self) -> &Schema<'_> {
            &STRUCT_WITH_MEMBER
        }

        fn serialize_members(&self, s: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            s.write_string(&NAME_MEMBER, "Alice")
        }
    }

    // Fixtures for the body-only guard below: a structure whose members carry the response
    // bindings an RPC protocol is required to ignore.
    static BOUND_NAME: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "BoundStruct"),
        ShapeType::String,
        "name",
        0,
    )
    .with_http_header("x-name");
    static BOUND_STATUS: Schema<'static> = Schema::new_member(
        crate::shape_id!("test", "BoundStruct"),
        ShapeType::Integer,
        "status",
        1,
    )
    .with_http_response_code();
    static BOUND_MEMBERS: &[&Schema<'_>] = &[&BOUND_NAME, &BOUND_STATUS];
    static BOUND_STRUCT: Schema<'static> = Schema::new_struct(
        crate::shape_id!("test", "BoundStruct"),
        ShapeType::Structure,
        BOUND_MEMBERS,
    );

    /// A response carrying values for both bound members, so a test can tell whether the
    /// protocol read them from the transport or from the body.
    fn bound_response(body: &'static str) -> Response {
        let response = http::Response::builder()
            .status(418)
            .header("x-name", "from-header")
            .body(SdkBody::from(body))
            .expect("response");
        Response::try_from(response).expect("convertible")
    }

    fn collect<'c>(
        strings: &'c mut Vec<(String, String)>,
        integers: &'c mut Vec<(String, i32)>,
    ) -> impl FnMut(&Schema<'_>, &mut dyn ShapeDeserializer) -> Result<(), SerdeError> + 'c {
        move |member, deser| {
            let name = member.member_name().unwrap_or("?").to_string();
            match member.shape_type() {
                ShapeType::Integer => integers.push((name, deser.read_integer(member)?)),
                _ => strings.push((name, deser.read_string(member)?)),
            }
            Ok(())
        }
    }

    /// An RPC protocol must ignore HTTP response bindings and read every member from the body.
    ///
    /// This is the complement of the REST behavior and the reason binding applicability has to
    /// be owned by the protocol selected at runtime rather than by the generated shape: the same
    /// schema, carrying the same `@httpHeader` and `@httpResponseCode` traits, must produce
    /// transport-sourced members under a REST protocol and body-sourced members here. See the
    /// codec settings table in the Serialization and Schema Decoupling SEP, under which
    /// awsJson, awsQuery, ec2Query and rpcv2Cbor all ignore HTTP bindings.
    #[test]
    fn deserialize_response_ignores_http_response_bindings() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let response = bound_response("body-value");
        let mut strings = Vec::new();
        let mut integers = Vec::new();
        protocol
            .deserialize_response(&response, &BOUND_STRUCT, &ConfigBag::base())
            .unwrap()
            .read_struct(&BOUND_STRUCT, &mut collect(&mut strings, &mut integers))
            .unwrap();

        // The stub codec answers `read_string` with the body bytes, so "body-value" here means
        // the member was read from the body. "from-header" would mean the composite had been
        // wired in and the transport had won.
        assert_eq!(
            strings,
            vec![("name".to_string(), "body-value".to_string())],
            "a bound member must be read from the body, not from the header"
        );
        // Likewise the response code member: 0 is the stub codec's integer, 418 is the
        // response's status.
        assert_eq!(integers, vec![("status".to_string(), 0)]);
    }

    /// The error path must be body-only too, including through the default forwarding.
    ///
    /// `HttpRpcProtocol` does not override `deserialize_error_response`, so this exercises
    /// [`ClientProtocolInner::deserialize_error_response`]'s default, which forwards to
    /// `deserialize_response` against `prelude::DOCUMENT`. That forwarding must stay body-only
    /// for an RPC protocol even though the identical default is what a REST protocol has to
    /// avoid.
    #[test]
    fn deserialize_error_response_ignores_http_response_bindings() {
        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let response = bound_response("body-value");
        let mut strings = Vec::new();
        let mut integers = Vec::new();
        protocol
            .deserialize_error_response(&response, &ConfigBag::base())
            .unwrap()
            .read_struct(&BOUND_STRUCT, &mut collect(&mut strings, &mut integers))
            .unwrap();

        assert_eq!(
            strings,
            vec![("name".to_string(), "body-value".to_string())],
            "a bound error member must be read from the body, not from the header"
        );
        assert_eq!(integers, vec![("status".to_string(), 0)]);
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
        let base_cfg = ConfigBag::base();
        let mut deser = protocol
            .deserialize_response(&response, &TEST_SCHEMA, &base_cfg)
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

    #[test]
    fn serialize_honors_header_omit_settings() {
        use crate::header_omit_settings::{HeaderOmitSettings, SharedHeaderOmitSettings};
        use aws_smithy_types::config_bag::Layer;

        #[derive(Debug)]
        struct OmitBoth;
        impl HeaderOmitSettings for OmitBoth {
            fn should_omit_default_content_type(&self) -> bool {
                true
            }
            fn should_omit_default_content_length(&self) -> bool {
                true
            }
        }

        let protocol = HttpRpcProtocol::new(
            crate::shape_id!("test", "rpc"),
            TestCodec,
            "application/x-amz-json-1.0",
        );
        let mut layer = Layer::new("test");
        layer.store_put(SharedHeaderOmitSettings::new(OmitBoth));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let request = protocol
            .serialize_request(
                &NameStruct,
                &STRUCT_WITH_MEMBER,
                "https://example.com",
                &cfg,
            )
            .unwrap();
        assert!(request.headers().get("Content-Type").is_none());
        assert!(request.headers().get("Content-Length").is_none());
    }
}
