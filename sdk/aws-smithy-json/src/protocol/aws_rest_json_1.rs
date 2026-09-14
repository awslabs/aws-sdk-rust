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
use aws_smithy_schema::http_protocol::HttpBindingProtocol;
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
        cfg: &ConfigBag,
    ) -> Result<
        Box<dyn aws_smithy_schema::serde::ShapeDeserializer + 'a>,
        aws_smithy_schema::serde::SerdeError,
    > {
        // When no namespace was configured explicitly, fall back to the one generated clients
        // store in the config bag, so a protocol selected at runtime can still resolve relative
        // `__type` discriminators. See `crate::protocol::codec_with_bag_namespace`.
        if let Some(codec) = crate::protocol::codec_with_bag_namespace(self.inner.codec(), cfg) {
            // Body extraction mirrors `HttpBindingProtocol::deserialize_response`, which carries
            // the rationale for tolerating an unreadable (streaming) body; `&[]` means "no body
            // members to read". Kept in step with that method.
            let body = response.body().bytes().unwrap_or(&[]);
            return Ok(Box::new(
                aws_smithy_schema::codec::Codec::create_deserializer(&codec, body),
            ));
        }
        self.inner
            .deserialize_response(response, output_schema, cfg)
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
    /// `deserialize_error_response` is **not** overridden: restJson1 has no
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
}
