/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! AWS REST JSON 1.0 protocol implementation.
//!
//! This module provides [`AwsRestJsonProtocol`], which constructs an
//! [`HttpBindingProtocol`] with a [`JsonCodec`] configured for the
//! `aws.protocols#restJson1` protocol:
//!
//! - Uses `@jsonName` trait for JSON property names
//! - Default timestamp format: `epoch-seconds`
//! - Content-Type: `application/json`

use crate::codec::{JsonCodec, JsonCodecSettings};
use aws_smithy_schema::http_protocol::{
    http_error_deserializer, http_output_deserializer, HttpBindingProtocol,
};
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;

static PROTOCOL_ID: ShapeId<'static> = shape_id!("aws.protocols", "restJson1");

/// AWS REST JSON 1.0 protocol (`aws.protocols#restJson1`).
///
/// This is a thin configuration wrapper that constructs an [`HttpBindingProtocol`]
/// with a [`JsonCodec`] using REST JSON settings. The `HttpBindingProtocol` handles
/// splitting members between HTTP bindings and the JSON payload.
#[derive(Debug)]
pub struct AwsRestJsonProtocol {
    inner: HttpBindingProtocol<JsonCodec>,
}

impl AwsRestJsonProtocol {
    /// Creates a new REST JSON protocol with default settings.
    pub fn new() -> Self {
        let codec = JsonCodec::new(
            JsonCodecSettings::builder()
                .use_json_name(true)
                .default_timestamp_format(aws_smithy_types::date_time::Format::EpochSeconds)
                .protocol_id(PROTOCOL_ID.clone())
                .build(),
        );
        Self {
            inner: HttpBindingProtocol::new(PROTOCOL_ID.clone(), codec, "application/json"),
        }
    }

    /// Configures the default Smithy namespace used to resolve relative
    /// shape IDs in JSON `__type` discriminator fields. Forwarded to
    /// [`JsonCodecSettings::default_namespace`] on the codec wrapped by
    /// this protocol.
    ///
    /// REST JSON services may emit relative `__type` values in document-
    /// typed members; code-generated clients call this method with the
    /// service shape's namespace so the deserializer can produce a fully-
    /// qualified discriminator.
    ///
    /// Setting this explicitly *overrides* the default, which is the
    /// [`ServiceShapeNamespace`](aws_smithy_schema::protocol::ServiceShapeNamespace) config-bag
    /// entry that generated clients store regardless of which protocol they were generated for.
    /// That fallback exists because a customer selecting restJson1 through
    /// `Config::builder().protocol(..)` has no way to know the model's namespace, and without it
    /// every relative discriminator would stay unresolved. See
    /// the internal `codec_with_bag_namespace` helper in `protocol/mod.rs` for how it is applied
    /// and what it costs.
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
        }
    }

    /// Returns a reference to the inner `HttpBindingProtocol`.
    pub fn inner(&self) -> &HttpBindingProtocol<JsonCodec> {
        &self.inner
    }
}

impl Default for AwsRestJsonProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl aws_smithy_schema::protocol::ClientProtocolInner for AwsRestJsonProtocol {
    type Request = aws_smithy_runtime_api::http::Request;
    type Response = aws_smithy_runtime_api::http::Response;

    fn protocol_id(&self) -> &ShapeId<'static> {
        self.inner.protocol_id()
    }

    fn serialize_request(
        &self,
        input: &dyn aws_smithy_schema::serde::SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<aws_smithy_runtime_api::http::Request, aws_smithy_schema::serde::SerdeError> {
        self.inner
            .serialize_request(input, input_schema, endpoint, cfg)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a aws_smithy_runtime_api::http::Response,
        output_schema: &Schema<'_>,
        cfg: &'a ConfigBag,
    ) -> Result<
        Box<dyn aws_smithy_schema::serde::ShapeDeserializer + 'a>,
        aws_smithy_schema::serde::SerdeError,
    > {
        // When no namespace was configured explicitly, fall back to the one generated clients
        // store in the config bag, so a protocol selected at runtime can still resolve relative
        // `__type` discriminators. See `crate::protocol::codec_with_bag_namespace`.
        if let Some(codec) = crate::protocol::codec_with_bag_namespace(self.inner.codec(), cfg) {
            // This branch builds its own body deserializer, so it has to wrap it the same way
            // the inner protocol would. Without that, selecting restJson1 at runtime against a
            // client that stored its namespace in the bag would silently drop every modeled
            // header, status, and payload member.
            //
            // Body extraction mirrors `HttpBindingProtocol::deserialize_response`, which carries
            // the rationale for tolerating an unreadable (streaming) body; `&[]` means "no body
            // members to read". Kept in step with that method.
            let body = response.body().bytes().unwrap_or(&[]);
            return Ok(http_output_deserializer(
                aws_smithy_schema::codec::Codec::create_deserializer(&codec, body),
                response,
                output_schema,
                cfg,
            ));
        }
        self.inner
            .deserialize_response(response, output_schema, cfg)
    }

    /// Returns a deserializer for a modeled error response.
    ///
    /// restJson1 has no error envelope, so the body root already *is* the error body and there is
    /// nothing to reposition. The override is therefore **behavior-neutral today**: the default
    /// forwarding would route the error through
    /// [`deserialize_response`](aws_smithy_schema::protocol::ClientProtocolInner::deserialize_response), and because the JSON codec reads an
    /// empty body as an empty object, the success wrapper's stricter empty-body handling is not
    /// observable here. It is stated rather than relied upon.
    ///
    /// It is overridden anyway so that error deserialization does not *depend* on two incidental
    /// properties of the default forwarding: that `prelude::DOCUMENT` happens not to qualify for
    /// the body-only fast path, and that this codec happens to accept an empty body. Neither is a
    /// contract, and an error path that silently became the success path would drop every
    /// transport-bound member. Being explicit also keeps every REST error path structurally
    /// incapable of reaching the success fast path, which is the property the protocol-level
    /// error tests assert.
    ///
    /// The namespace fallback applies here too, so the branch structure matches
    /// `deserialize_response`.
    fn deserialize_error_response<'a>(
        &self,
        response: &'a aws_smithy_runtime_api::http::Response,
        cfg: &'a ConfigBag,
    ) -> Result<
        Box<dyn aws_smithy_schema::serde::ShapeDeserializer + 'a>,
        aws_smithy_schema::serde::SerdeError,
    > {
        if let Some(codec) = crate::protocol::codec_with_bag_namespace(self.inner.codec(), cfg) {
            let body = response.body().bytes().unwrap_or(&[]);
            return Ok(http_error_deserializer(
                aws_smithy_schema::codec::Codec::create_deserializer(&codec, body),
                response,
                cfg,
            ));
        }
        self.inner.deserialize_error_response(response, cfg)
    }

    /// Extracts canonical error metadata from a `restJson1` response.
    ///
    /// restJson1 uses the same JSON error envelope as awsJson1.0 / 1.1:
    /// `__type` (or legacy `code`) for the error code, with header
    /// `X-Amzn-Errortype` taking priority; `message` / `Message` /
    /// `errorMessage` for the message.
    ///
    /// Per the
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata)
    /// contract the request id is **not** populated here — the
    /// orchestrator's request-id pipeline attaches it separately.
    ///
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
    /// `eventStreamMessageContentType` for this protocol (`RestJson.kt:90`), which supplies
    /// the fallback when a protocol declares no media type.
    fn event_stream_media_type(&self) -> Option<&str> {
        Some("application/json")
    }

    /// Parses the same JSON error envelope as
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata), from an event-stream
    /// frame's payload rather than an HTTP response body. An event-stream frame has
    /// no HTTP headers, so the `x-amzn-errortype` route is unavailable here and an
    /// empty header map is passed; the discriminator comes from the payload's
    /// `__type`.
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
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_schema::protocol::ClientProtocolInner;
    use aws_smithy_schema::serde::SerdeError;
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
    fn protocol_id_is_rest_json_1() {
        assert_eq!(
            AwsRestJsonProtocol::new().protocol_id().as_str(),
            "aws.protocols#restJson1"
        );
    }

    #[test]
    fn parse_error_metadata_extracts_code_and_message_from_body() {
        let proto = AwsRestJsonProtocol::new();
        let response = http_response(
            &[],
            r#"{"__type":"ValidationException","message":"bad input"}"#,
        );
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("ValidationException"));
        assert_eq!(meta.message(), Some("bad input"));
    }

    #[test]
    fn parse_error_metadata_header_takes_priority() {
        let proto = AwsRestJsonProtocol::new();
        let response = http_response(
            &[("x-amzn-errortype", "FromHeader")],
            r#"{"__type":"FromBody"}"#,
        );
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("FromHeader"));
    }

    #[test]
    fn parse_error_metadata_sanitizes_namespaced_code() {
        let proto = AwsRestJsonProtocol::new();
        let response = http_response(&[("x-amzn-errortype", "ns#FooError:http://example/")], "");
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert_eq!(meta.code(), Some("FooError"));
    }

    #[test]
    fn parse_error_metadata_empty_body_returns_empty_builder() {
        let proto = AwsRestJsonProtocol::new();
        let response = http_response(&[], "");
        let cfg = ConfigBag::base();
        let meta = proto.parse_error_metadata(&response, &cfg).unwrap().build();
        assert!(meta.code().is_none());
        assert!(meta.message().is_none());
    }

    #[test]
    fn parse_error_metadata_malformed_body_returns_error() {
        let proto = AwsRestJsonProtocol::new();
        let response = http_response(&[], r#"{"__type":"Foo""#);
        let cfg = ConfigBag::base();
        let err = proto.parse_error_metadata(&response, &cfg).unwrap_err();
        assert!(matches!(err, SerdeError::InvalidInput { .. }));
    }

    #[test]
    fn with_default_namespace_propagates_to_codec_settings() {
        let proto = AwsRestJsonProtocol::new().with_default_namespace("com.amazonaws.s3");
        assert_eq!(
            proto.inner.codec().settings().default_namespace(),
            Some("com.amazonaws.s3"),
        );
    }

    #[test]
    fn with_default_namespace_preserves_other_settings() {
        // Rebuilding the codec for `default_namespace` must preserve
        // the restJson1-specific `@jsonName=true` field-mapper choice
        // set in `AwsRestJsonProtocol::new`.
        let proto = AwsRestJsonProtocol::new().with_default_namespace("com.example");
        let settings = proto.inner.codec().settings();
        assert_eq!(settings.default_namespace(), Some("com.example"));
        assert_eq!(
            settings.default_timestamp_format(),
            aws_smithy_types::date_time::Format::EpochSeconds,
        );
    }

    // ---------------------------------------------------------------------------------
    // the ConfigBag-namespace branch must wrap like the inner protocol does
    //
    // That branch builds its own body deserializer and returns early, so it is the one place
    // restJson1 can silently lose every transport-bound member.
    // ---------------------------------------------------------------------------------

    use aws_smithy_schema::protocol::ServiceShapeNamespace;
    use aws_smithy_schema::ShapeType;
    use aws_smithy_types::config_bag::Layer;

    static NS_NAME: Schema<'static> =
        Schema::new_member(shape_id!("test", "Out"), ShapeType::String, "name", 0)
            .with_http_header("x-name");
    static NS_STATUS: Schema<'static> =
        Schema::new_member(shape_id!("test", "Out"), ShapeType::Integer, "status", 1)
            .with_http_response_code();
    static NS_BODY: Schema<'static> =
        Schema::new_member(shape_id!("test", "Out"), ShapeType::String, "note", 2);
    static NS_OUT: Schema<'static> = Schema::new_struct(
        shape_id!("test", "Out"),
        ShapeType::Structure,
        &[&NS_NAME, &NS_STATUS, &NS_BODY],
    );

    /// A bag carrying the namespace a generated client would have stored, which is what sends
    /// `deserialize_response` down its own branch instead of delegating to the inner protocol.
    fn bag_with_namespace() -> ConfigBag {
        let mut layer = Layer::new("test");
        layer.store_put(ServiceShapeNamespace::new("com.amazonaws.dynamodb"));
        ConfigBag::of_layers(vec![layer])
    }

    fn read_bound(
        mut deser: Box<dyn aws_smithy_schema::serde::ShapeDeserializer + '_>,
        schema: &Schema<'_>,
    ) -> (Option<String>, Option<i32>) {
        let mut name = None;
        let mut status = None;
        deser
            .read_struct(schema, &mut |member, d| {
                match member.member_name() {
                    Some("name") => name = Some(d.read_string(member)?),
                    Some("status") => status = Some(d.read_integer(member)?),
                    // Read and discard. A consumer that declines a *known* member leaves a
                    // cursor-based codec mid-value, which is the same reason the response
                    // wrapper calls `skip_value` rather than just returning.
                    _ => {
                        let _ = d.read_string(member)?;
                    }
                }
                Ok(())
            })
            .expect("deserialization succeeds");
        (name, status)
    }

    #[test]
    fn the_namespace_branch_still_reads_modeled_headers_and_status() {
        let proto = AwsRestJsonProtocol::new();
        let cfg = bag_with_namespace();
        assert!(
            crate::protocol::codec_with_bag_namespace(proto.inner().codec(), &cfg).is_some(),
            "precondition: this response must take the namespace branch"
        );
        let response = http_response(&[("x-name", "widget")], r#"{"note":"n"}"#);
        let (name, status) = read_bound(
            proto
                .deserialize_response(&response, &NS_OUT, &cfg)
                .unwrap(),
            &NS_OUT,
        );
        assert_eq!(name.as_deref(), Some("widget"));
        assert_eq!(status, Some(400));
    }

    #[test]
    fn the_error_path_reads_modeled_headers_and_status_too() {
        let proto = AwsRestJsonProtocol::new();
        let cfg = bag_with_namespace();
        let response = http_response(&[("x-name", "missing")], r#"{"note":"n"}"#);
        let (name, status) = read_bound(
            proto.deserialize_error_response(&response, &cfg).unwrap(),
            &NS_OUT,
        );
        assert_eq!(name.as_deref(), Some("missing"));
        assert_eq!(status, Some(400));
    }

    #[test]
    fn the_error_path_reads_bindings_without_the_namespace_branch_too() {
        // With no namespace in the bag the method delegates to the inner protocol's error path.
        // Both branches must behave the same way.
        let proto = AwsRestJsonProtocol::new();
        let cfg = ConfigBag::base();
        assert!(
            crate::protocol::codec_with_bag_namespace(proto.inner().codec(), &cfg).is_none(),
            "precondition: this response must delegate to the inner protocol"
        );
        let response = http_response(&[("x-name", "missing")], r#"{"note":"n"}"#);
        let (name, status) = read_bound(
            proto.deserialize_error_response(&response, &cfg).unwrap(),
            &NS_OUT,
        );
        assert_eq!(name.as_deref(), Some("missing"));
        assert_eq!(status, Some(400));
    }
}
