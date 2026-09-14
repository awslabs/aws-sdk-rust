/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! AWS JSON RPC protocol implementation (`awsJson1_0` and `awsJson1_1`).
//!
//! # Protocol behaviors
//!
//! - HTTP method: always POST, path: always `/`
//! - `X-Amz-Target`: `{ServiceName}.{OperationName}` (required)
//! - Does **not** use `@jsonName` trait
//! - Default timestamp format: `epoch-seconds`
//! - Ignores HTTP binding traits
//!
//! # Differences between 1.0 and 1.1
//!
//! - Content-Type: `application/x-amz-json-1.0` vs `application/x-amz-json-1.1`
//! - Error `__type` serialization differs on the server side, but clients MUST
//!   accept either format for both versions.

use crate::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_runtime_api::client::orchestrator::Metadata;
use aws_smithy_schema::http_protocol::HttpRpcProtocol;
use aws_smithy_schema::protocol::ServiceShapeName;
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;

/// AWS JSON RPC protocol (`awsJson1_0` / `awsJson1_1`).
#[derive(Debug)]
pub struct AwsJsonRpcProtocol {
    inner: HttpRpcProtocol<JsonCodec>,
    /// Prefix of the `X-Amz-Target` header. `None` means "resolve from the config bag", which is
    /// the normal case — see [`Self::with_target_prefix`].
    target_prefix: Option<String>,
}

impl AwsJsonRpcProtocol {
    /// Creates an AWS JSON 1.0 protocol instance.
    ///
    /// The `X-Amz-Target` prefix defaults to the Smithy service shape name from the config bag;
    /// use [`Self::with_target_prefix`] to override it.
    pub fn aws_json_1_0() -> Self {
        Self::new(
            shape_id!("aws.protocols", "awsJson1_0"),
            "application/x-amz-json-1.0",
        )
    }

    /// Creates an AWS JSON 1.1 protocol instance.
    ///
    /// The `X-Amz-Target` prefix defaults to the Smithy service shape name from the config bag;
    /// use [`Self::with_target_prefix`] to override it.
    pub fn aws_json_1_1() -> Self {
        Self::new(
            shape_id!("aws.protocols", "awsJson1_1"),
            "application/x-amz-json-1.1",
        )
    }

    fn new(protocol_id: ShapeId<'static>, content_type: &'static str) -> Self {
        let codec = JsonCodec::new(
            JsonCodecSettings::builder()
                .use_json_name(false)
                .default_timestamp_format(aws_smithy_types::date_time::Format::EpochSeconds)
                .protocol_id(protocol_id.clone())
                .build(),
        );
        Self {
            inner: HttpRpcProtocol::new(protocol_id, codec, content_type),
            target_prefix: None,
        }
    }

    /// Overrides the prefix of the `X-Amz-Target` header.
    ///
    /// By default the prefix is the Smithy service shape name, read from the
    /// [`ServiceShapeName`] config-bag entry that generated clients store regardless of which
    /// protocol they were generated for. That default exists because a customer selecting this
    /// protocol through `Config::builder().protocol(..)` has no way to know the shape name, and an
    /// incorrect `X-Amz-Target` is unroutable.
    ///
    /// Override it when a service's target prefix is not its shape name.
    ///
    /// Note this is *not* the sdkId carried by
    /// [`Metadata::service`](aws_smithy_runtime_api::client::orchestrator::Metadata::service) —
    /// an sdkId may contain spaces (`"JSON RPC 10"`) and is not a valid header value here.
    pub fn with_target_prefix(mut self, target_prefix: impl Into<String>) -> Self {
        self.target_prefix = Some(target_prefix.into());
        self
    }

    /// Resolves the `X-Amz-Target` prefix: an explicit override wins, otherwise the service shape
    /// name from the config bag. `None` when neither is available, in which case there is no
    /// correct value to send.
    fn resolved_target_prefix<'a>(&'a self, cfg: &'a ConfigBag) -> Option<&'a str> {
        self.target_prefix
            .as_deref()
            .or_else(|| cfg.load::<ServiceShapeName>().map(ServiceShapeName::as_str))
    }

    /// Configures the default Smithy namespace used to resolve relative
    /// shape IDs in JSON `__type` discriminator fields. Forwarded to
    /// [`JsonCodecSettings::default_namespace`] on the codec wrapped by
    /// this protocol.
    ///
    /// AWS JSON 1.0 / 1.1 services typically emit relative `__type`
    /// values (the shape name without a namespace prefix). Code-generated
    /// clients call this method with the service shape's namespace so
    /// that [`crate::codec::JsonDeserializer::read_discriminated_document`]
    /// can produce a fully-qualified discriminator.
    ///
    /// Unlike the `X-Amz-Target` prefix, this is a response-parsing concern rather than request
    /// shaping — nothing on the serialization path reads it, because a serializer must always
    /// emit an absolute shape ID.
    ///
    /// Setting it explicitly *overrides* the default, which is the
    /// [`ServiceShapeNamespace`](aws_smithy_schema::protocol::ServiceShapeNamespace) config-bag
    /// entry that generated clients store regardless of which protocol they were generated for.
    /// A caller selecting this protocol at runtime therefore gets relative `__type` resolution
    /// without having to know the model's namespace; previously it had to be set by hand or
    /// discriminators stayed relative, which made the type registry silently miss the shape.
    ///
    /// The fallback is applied per response because `JsonCodec` holds an `Arc<JsonCodecSettings>`
    /// that `create_deserializer` clones by pointer, so there is nowhere to store the resolved
    /// value on an immutable protocol. That costs ~46 ns per response and only when no explicit
    /// namespace was configured. The internal `codec_with_bag_namespace` helper in
    /// `protocol/mod.rs` carries the
    /// measurement and the memoization option if it ever matters.
    pub fn with_default_namespace(self, namespace: impl Into<String>) -> Self {
        let new_settings = self
            .inner
            .codec()
            .settings()
            .to_builder()
            .default_namespace(namespace)
            .build();
        let new_codec = JsonCodec::new(new_settings);
        Self {
            inner: self.inner.with_codec(new_codec),
            target_prefix: self.target_prefix,
        }
    }
}

impl aws_smithy_schema::protocol::ClientProtocolInner for AwsJsonRpcProtocol {
    type Request = aws_smithy_runtime_api::http::Request;
    type Response = aws_smithy_runtime_api::http::Response;

    fn protocol_id(&self) -> &ShapeId<'static> {
        self.inner.protocol_id()
    }

    /// Serializes an awsJson1_0 / awsJson1_1 request.
    ///
    /// `_endpoint` is deliberately ignored: both protocols fix the request path at `/`, so the
    /// route is a function of the protocol rather than of the operation, and a path computed by
    /// codegen for a different protocol must not leak through when this protocol is selected at
    /// runtime via `Config::builder().protocol(..)`. `apply_http_endpoint` merges the scheme and
    /// authority afterwards.
    ///
    /// The assertion belongs here rather than in [`HttpRpcProtocol`] because only the concrete
    /// protocol knows whether its route is constant — `RpcV2CborProtocol` legitimately passes
    /// `HttpRpcProtocol` a computed `/service/{service}/operation/{operation}` route.
    fn serialize_request(
        &self,
        input: &dyn aws_smithy_schema::serde::SerializableStruct,
        input_schema: &Schema<'_>,
        _endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<aws_smithy_runtime_api::http::Request, aws_smithy_schema::serde::SerdeError> {
        let mut request = self
            .inner
            .serialize_request(input, input_schema, "/", cfg)?;
        if let (Some(prefix), Some(metadata)) =
            (self.resolved_target_prefix(cfg), cfg.load::<Metadata>())
        {
            request
                .headers_mut()
                .insert("X-Amz-Target", format!("{}.{}", prefix, metadata.name()));
        }
        Ok(request)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a aws_smithy_runtime_api::http::Response,
        output_schema: &Schema<'_>,
        cfg: &ConfigBag,
    ) -> Result<
        Box<dyn aws_smithy_schema::serde::ShapeDeserializer + 'a>,
        aws_smithy_schema::serde::SerdeError,
    > {
        // When no namespace was configured explicitly, fall back to the one generated clients
        // store in the config bag, so a protocol selected at runtime can still resolve relative
        // `__type` discriminators. See `crate::protocol::codec_with_bag_namespace`.
        if let Some(codec) = crate::protocol::codec_with_bag_namespace(self.inner.codec(), cfg) {
            // Body extraction mirrors `HttpRpcProtocol::deserialize_response`, which defers to
            // `HttpBindingProtocol::deserialize_response` for the rationale behind tolerating an
            // unreadable (streaming) body. Kept in step with those methods.
            let body = response.body().bytes().unwrap_or(&[]);
            return Ok(Box::new(
                aws_smithy_schema::codec::Codec::create_deserializer(&codec, body),
            ));
        }
        self.inner
            .deserialize_response(response, output_schema, cfg)
    }

    /// Extracts canonical error metadata from an `awsJson1_0` / `awsJson1_1`
    /// response.
    ///
    /// awsJson protocols carry the error code in the `__type` (or legacy
    /// `code`) field of the JSON body, with `X-Amzn-Errortype` taking
    /// priority. The error message comes from `message`, `Message`, or
    /// `errorMessage` body keys.
    ///
    /// Per the
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata)
    /// contract the request id is **not** populated here — the
    /// orchestrator's request-id pipeline attaches it separately.
    ///
    /// `deserialize_error_response` is **not** overridden: awsJson has no
    /// error envelope, so the default (which forwards to
    /// `deserialize_response` against
    /// [`prelude::DOCUMENT`](aws_smithy_schema::prelude::DOCUMENT)) is
    /// already correct — the body root IS the error body.
    fn parse_error_metadata(
        &self,
        response: &aws_smithy_runtime_api::http::Response,
        _cfg: &ConfigBag,
    ) -> Result<aws_smithy_types::error::metadata::Builder, aws_smithy_schema::serde::SerdeError>
    {
        let body = response.body().bytes().unwrap_or(&[]);
        crate::protocol::error::parse_error_envelope_metadata(body, response.headers())
    }

    fn payload_codec(&self) -> Option<&dyn aws_smithy_schema::codec::DynCodec> {
        self.inner.payload_codec()
    }

    /// This protocol labels structured event-stream payloads `application/json`.
    ///
    /// Must stay in agreement with the code generator's
    /// `eventStreamMessageContentType` for this protocol (`AwsJson.kt:95`), which supplies
    /// the fallback when a protocol declares no media type.
    fn event_stream_media_type(&self) -> Option<&str> {
        Some("application/json")
    }

    /// Parses the same JSON error envelope as
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata), from an event-stream
    /// frame's payload rather than an HTTP response body. An event-stream frame has
    /// no HTTP headers, so an empty header map is passed; the discriminator comes
    /// from the payload's `__type`.
    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<aws_smithy_types::error::metadata::Builder, aws_smithy_schema::serde::SerdeError>
    {
        crate::protocol::error::parse_error_envelope_metadata(
            payload,
            &aws_smithy_runtime_api::http::Headers::new(),
        )
    }

    fn update_endpoint(
        &self,
        request: &mut aws_smithy_runtime_api::http::Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), aws_smithy_schema::serde::SerdeError> {
        self.inner.update_endpoint(request, endpoint, cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::protocol::ClientProtocolInner;
    use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeSerializer};
    use aws_smithy_schema::ShapeType;
    use aws_smithy_types::config_bag::Layer;

    struct EmptyStruct;
    impl SerializableStruct for EmptyStruct {
        fn serialize_members(&self, _: &mut dyn ShapeSerializer) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static TEST_SCHEMA: aws_smithy_schema::Schema =
        aws_smithy_schema::Schema::new(shape_id!("test", "Input"), ShapeType::Structure);

    fn cfg_with_metadata(service: &str, operation: &str) -> ConfigBag {
        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new(operation.to_string(), service.to_string()));
        ConfigBag::of_layers(vec![layer])
    }

    /// The bag a schema-serde generated client actually builds: operation `Metadata` plus the
    /// model's service shape name.
    fn cfg_with_service_shape_name(shape_name: &'static str, operation: &str) -> ConfigBag {
        let mut layer = Layer::new("test");
        layer.store_put(Metadata::new(
            operation.to_string(),
            "Some Sdk Id".to_string(),
        ));
        layer.store_put(aws_smithy_schema::protocol::ServiceShapeName::new(
            shape_name,
        ));
        ConfigBag::of_layers(vec![layer])
    }

    /// A customer selecting awsJson at runtime via `Config::builder().protocol(..)` has no way to
    /// know the Smithy service shape name that belongs in `X-Amz-Target`, so the protocol defaults
    /// it from the config-bag entry that every schema-serde client stores regardless of the
    /// protocol it was generated for.
    #[test]
    fn target_prefix_defaults_to_service_shape_name_from_config_bag() {
        let cfg = cfg_with_service_shape_name("MyService", "DoThing");
        let request = AwsJsonRpcProtocol::aws_json_1_0()
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "/", &cfg)
            .unwrap();
        assert_eq!(
            request.headers().get("X-Amz-Target").unwrap(),
            "MyService.DoThing"
        );
    }

    /// An explicit prefix still wins, so a caller whose service uses a target prefix that is not
    /// the shape name (DynamoDB's `DynamoDB_20120810`) can override it.
    #[test]
    fn with_target_prefix_overrides_config_bag() {
        let cfg = cfg_with_service_shape_name("MyService", "DoThing");
        let request = AwsJsonRpcProtocol::aws_json_1_0()
            .with_target_prefix("DynamoDB_20120810")
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "/", &cfg)
            .unwrap();
        assert_eq!(
            request.headers().get("X-Amz-Target").unwrap(),
            "DynamoDB_20120810.DoThing"
        );
    }

    /// With neither an override nor a bag entry there is no correct value, so the header is
    /// omitted rather than guessed — matching the existing behavior when `Metadata` is absent.
    #[test]
    fn x_amz_target_omitted_when_prefix_is_unknown() {
        let cfg = cfg_with_metadata("Some Sdk Id", "DoThing");
        let request = AwsJsonRpcProtocol::aws_json_1_0()
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "/", &cfg)
            .unwrap();
        assert!(request.headers().get("X-Amz-Target").is_none());
    }

    #[test]
    fn json_1_0_content_type() {
        let request = AwsJsonRpcProtocol::aws_json_1_0()
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
    fn json_1_1_content_type() {
        let request = AwsJsonRpcProtocol::aws_json_1_1()
            .serialize_request(
                &EmptyStruct,
                &TEST_SCHEMA,
                "https://example.com",
                &ConfigBag::base(),
            )
            .unwrap();
        assert_eq!(
            request.headers().get("Content-Type").unwrap(),
            "application/x-amz-json-1.1"
        );
    }

    #[test]
    fn sets_x_amz_target() {
        let cfg = cfg_with_metadata("MyService", "DoThing");
        let request = AwsJsonRpcProtocol::aws_json_1_0()
            .with_target_prefix("MyService")
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "https://example.com", &cfg)
            .unwrap();
        assert_eq!(
            request.headers().get("X-Amz-Target").unwrap(),
            "MyService.DoThing"
        );
    }

    /// The body deserializer handed out by `deserialize_response` must reject malformed JSON
    /// bodies end to end, not only inside containers. These are the bodies of Smithy protocol
    /// tests `RestJsonInvalidJsonBody_case1` and `RestJsonInvalidJsonBody_case7`.
    #[test]
    fn malformed_response_bodies_are_rejected() {
        use aws_smithy_runtime_api::http::Response;
        use aws_smithy_types::body::SdkBody;

        for body in [r#"{ "int": 10 }abc"#, r#"{"int": 10,}"#] {
            let response = Response::new(200u16.try_into().unwrap(), SdkBody::from(body));
            let mut deser = AwsJsonRpcProtocol::aws_json_1_0()
                .deserialize_response(&response, &TEST_SCHEMA, &ConfigBag::base())
                .unwrap();
            let result = deser.read_struct(&TEST_SCHEMA, &mut |_, _| Ok(()));
            assert!(result.is_err(), "body {body:?} must be rejected");
        }

        let response = Response::new(
            200u16.try_into().unwrap(),
            SdkBody::from(r#"{ "int": 10 }"#),
        );
        let mut deser = AwsJsonRpcProtocol::aws_json_1_0()
            .deserialize_response(&response, &TEST_SCHEMA, &ConfigBag::base())
            .unwrap();
        deser
            .read_struct(&TEST_SCHEMA, &mut |_, _| Ok(()))
            .expect("well-formed body is accepted");
    }

    /// A quoted number for a double member must be rejected through the real response path.
    /// This is the body of Smithy protocol test `RestJsonBodyDoubleMalformedValueRejected_case0`.
    #[test]
    fn quoted_number_for_double_member_is_rejected() {
        use aws_smithy_runtime_api::http::Response;
        use aws_smithy_types::body::SdkBody;

        static DOUBLE: aws_smithy_schema::Schema = aws_smithy_schema::Schema::new_member(
            shape_id!("test", "Output", "doubleInBody"),
            ShapeType::Double,
            "doubleInBody",
            0,
        );
        static OUTPUT: aws_smithy_schema::Schema = aws_smithy_schema::Schema::new_struct(
            shape_id!("test", "Output"),
            ShapeType::Structure,
            &[&DOUBLE],
        );
        let read = |body: &str| {
            let response = Response::new(200u16.try_into().unwrap(), SdkBody::from(body));
            let mut deser = AwsJsonRpcProtocol::aws_json_1_0()
                .deserialize_response(&response, &OUTPUT, &ConfigBag::base())
                .unwrap();
            let mut value = None;
            deser
                .read_struct(&OUTPUT, &mut |member, d| {
                    value = Some(d.read_double(member)?);
                    Ok(())
                })
                .map(|()| value)
        };
        assert_eq!(read(r#"{ "doubleInBody" : 123 }"#).unwrap(), Some(123.0));
        assert!(read(r#"{ "doubleInBody" : "123" }"#).is_err());
    }

    #[test]
    fn json_1_0_protocol_id() {
        assert_eq!(
            AwsJsonRpcProtocol::aws_json_1_0().protocol_id().as_str(),
            "aws.protocols#awsJson1_0"
        );
    }

    // ---- route ---------------------------------------------------------

    /// awsJson1_0/1_1 fix the request path at `/`, so a path computed by codegen for a
    /// *different* protocol must not win when this protocol is selected at runtime via
    /// `Config::builder().protocol(..)`. The rpcv2Cbor route below is what an
    /// rpcv2Cbor-generated client passes.
    ///
    /// This is the mirror image of https://github.com/smithy-lang/smithy-rs/issues/4801,
    /// where the CBOR protocol failed to apply its own route.
    #[test]
    fn serialize_request_ignores_a_route_computed_for_another_protocol() {
        let cfg = cfg_with_metadata("MyService", "DoThing");
        for foreign_route in ["/service/MyService/operation/DoThing", "/stats"] {
            let request = AwsJsonRpcProtocol::aws_json_1_0()
                .serialize_request(&EmptyStruct, &TEST_SCHEMA, foreign_route, &cfg)
                .unwrap();
            assert_eq!(
                "/",
                request.uri(),
                "awsJson must POST to / regardless of the path it is handed"
            );
            assert_eq!("POST", request.method());
        }
    }

    /// The empty endpoint a generated awsJson client passes must keep resolving to `/`.
    #[test]
    fn serialize_request_defaults_to_slash() {
        let cfg = cfg_with_metadata("MyService", "DoThing");
        let request = AwsJsonRpcProtocol::aws_json_1_0()
            .serialize_request(&EmptyStruct, &TEST_SCHEMA, "", &cfg)
            .unwrap();
        assert_eq!("/", request.uri());
    }

    #[test]
    fn json_1_1_protocol_id() {
        assert_eq!(
            AwsJsonRpcProtocol::aws_json_1_1().protocol_id().as_str(),
            "aws.protocols#awsJson1_1"
        );
    }

    // ---- parse_error_metadata overrides --------------------------------

    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;

    fn http_response(headers: &[(&str, &str)], body: &str) -> Response {
        let mut response = Response::new(StatusCode::try_from(400).unwrap(), SdkBody::from(body));
        for (name, value) in headers {
            response
                .headers_mut()
                .insert(name.to_string(), value.to_string());
        }
        response
    }

    #[test]
    fn parse_error_metadata_extracts_code_and_message_from_body() {
        let proto = AwsJsonRpcProtocol::aws_json_1_0();
        let response = http_response(&[], r#"{"__type":"InvalidGreeting","message":"hi"}"#);
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("InvalidGreeting"));
        assert_eq!(meta.message(), Some("hi"));
    }

    #[test]
    fn parse_error_metadata_header_takes_priority() {
        let proto = AwsJsonRpcProtocol::aws_json_1_1();
        let response = http_response(
            &[("x-amzn-errortype", "FromHeader")],
            r#"{"__type":"FromBody","message":"go"}"#,
        );
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("FromHeader"));
        assert_eq!(meta.message(), Some("go"));
    }

    #[test]
    fn parse_error_metadata_sanitizes_namespaced_code() {
        let proto = AwsJsonRpcProtocol::aws_json_1_0();
        let response = http_response(&[], r#"{"__type":"aws.protocoltests.json#FooError"}"#);
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn parse_error_metadata_empty_body_returns_empty_builder() {
        let proto = AwsJsonRpcProtocol::aws_json_1_0();
        let response = http_response(&[], "");
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert!(meta.code().is_none());
        assert!(meta.message().is_none());
    }

    #[test]
    fn parse_error_metadata_malformed_body_returns_error() {
        let proto = AwsJsonRpcProtocol::aws_json_1_0();
        let response = http_response(&[], r#"{"__type":"FooError""#); // truncated
        let cfg = ConfigBag::base();
        let err = proto.parse_error_metadata(&response, &cfg).unwrap_err();
        assert!(matches!(err, SerdeError::InvalidInput { .. }));
    }

    #[test]
    fn with_default_namespace_propagates_to_codec_settings() {
        // The protocol's `with_default_namespace` builder must surface
        // the namespace on the codec's settings — this is the wiring
        // codegen relies on so that wire-bytes `__type:"Capacity"`
        // lifts to a fully-qualified `com.amazonaws.dynamodb#Capacity`
        // discriminator on the resulting [`DiscriminatedDocument`].
        let proto = AwsJsonRpcProtocol::aws_json_1_0()
            .with_target_prefix("DynamoDB_20120810")
            .with_default_namespace("com.amazonaws.dynamodb");
        assert_eq!(
            proto.inner.codec().settings().default_namespace(),
            Some("com.amazonaws.dynamodb"),
        );
    }

    #[test]
    fn with_default_namespace_preserves_other_settings() {
        // Sanity-check that rebuilding the codec to set
        // `default_namespace` doesn't reset other configured fields
        // — the AwsJsonRpc constructor already disables `@jsonName`
        // and sets epoch-seconds as the default timestamp format.
        let proto = AwsJsonRpcProtocol::aws_json_1_0().with_default_namespace("com.example");
        let settings = proto.inner.codec().settings();
        assert_eq!(settings.default_namespace(), Some("com.example"));
        assert_eq!(
            settings.default_timestamp_format(),
            aws_smithy_types::date_time::Format::EpochSeconds,
        );
    }
}
