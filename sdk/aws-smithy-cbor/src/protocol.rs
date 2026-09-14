/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! RPC v2 CBOR protocol implementation.

use crate::codec::{CborCodec, CborCodecSettings};
use crate::Decoder;
use aws_smithy_runtime_api::client::orchestrator::Metadata;
use aws_smithy_runtime_api::http::{Headers, Request, Response};
use aws_smithy_schema::error_envelope::{parse_query_compatible_header, sanitize_error_code};
use aws_smithy_schema::http_protocol::HttpRpcProtocol;
use aws_smithy_schema::protocol::{ClientProtocolInner, ServiceShapeName};
use aws_smithy_schema::serde::{SerdeError, SerializableStruct, ShapeDeserializer};
use aws_smithy_schema::{shape_id, Schema, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::error::metadata::{Builder as ErrorMetadataBuilder, ErrorMetadata};

/// RPC v2 CBOR protocol (`smithy.protocols#rpcv2Cbor`).
#[derive(Debug)]
pub struct RpcV2CborProtocol {
    inner: HttpRpcProtocol<CborCodec>,
}

impl RpcV2CborProtocol {
    /// Creates a new RPC v2 CBOR protocol instance.
    pub fn new() -> Self {
        Self {
            inner: HttpRpcProtocol::new(
                shape_id!("smithy.protocols", "rpcv2Cbor"),
                CborCodec::new(CborCodecSettings::default()),
                "application/cbor",
            ),
        }
    }
}

impl Default for RpcV2CborProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientProtocolInner for RpcV2CborProtocol {
    type Request = Request;
    type Response = Response;

    fn protocol_id(&self) -> &ShapeId<'static> {
        self.inner.protocol_id()
    }

    fn serialize_request(
        &self,
        input: &dyn SerializableStruct,
        input_schema: &Schema<'_>,
        endpoint: &str,
        cfg: &ConfigBag,
    ) -> Result<Request, SerdeError> {
        // RPC v2 CBOR ignores HTTP binding traits entirely and routes to
        // `/service/{serviceName}/operation/{operationName}`. Resolve that route
        // here rather than trusting the path codegen passed in: because the
        // protocol can be swapped at runtime, `endpoint` may have been computed
        // for a different protocol (`/` for a client generated against awsJson,
        // or an `@http` URI for a REST one), and honoring it sends every request
        // to the wrong route. See https://github.com/smithy-lang/smithy-rs/issues/4801.
        //
        // Falls back to the supplied endpoint when the model names aren't
        // available — e.g. a hand-written caller invoking this protocol directly
        // without populating the config bag.
        let route = rpc_v2_cbor_route(cfg);
        let endpoint = route.as_deref().unwrap_or(endpoint);
        let mut request = self
            .inner
            .serialize_request(input, input_schema, endpoint, cfg)?;

        // Protocol framing, set here for the same reason as the route: it is a
        // function of the protocol, not of the operation or its model, so a client
        // generated for another protocol never emitted it. `smithy-protocol` is
        // required on every request and is what conformant servers route on —
        // smithy-rs's own server rejects its absence with
        // `WireFormatError::HeaderNotFound`.
        //
        // Codegen inserts headers *after* this returns and `Headers::insert`
        // replaces, so an operation that needs a different value (an event stream
        // sends `accept: application/vnd.amazon.eventstream, application/cbor`)
        // still overrides these without any coordination here.
        request
            .headers_mut()
            .insert("smithy-protocol", "rpc-v2-cbor");
        request.headers_mut().insert("accept", "application/cbor");

        Ok(request)
    }

    fn deserialize_response<'a>(
        &self,
        response: &'a Response,
        output_schema: &Schema<'_>,
        cfg: &ConfigBag,
    ) -> Result<Box<dyn ShapeDeserializer + 'a>, SerdeError> {
        self.inner
            .deserialize_response(response, output_schema, cfg)
    }

    fn payload_codec(&self) -> Option<&dyn aws_smithy_schema::codec::DynCodec> {
        self.inner.payload_codec()
    }

    /// This protocol labels structured event-stream payloads `application/cbor`.
    ///
    /// Must stay in agreement with the code generator's
    /// `eventStreamMessageContentType` for this protocol (`RpcV2Cbor.kt:115`), which supplies
    /// the fallback when a protocol declares no media type.
    fn event_stream_media_type(&self) -> Option<&str> {
        Some("application/cbor")
    }

    /// Parses the same CBOR error envelope as
    /// [`ClientProtocolInner::parse_error_metadata`], from an event-stream
    /// frame's payload rather than an HTTP response body. An event-stream frame has
    /// no HTTP headers, so an empty header map is passed.
    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        parse_error_envelope_metadata(payload, &Headers::new())
    }

    fn update_endpoint(
        &self,
        request: &mut Request,
        endpoint: &aws_smithy_types::endpoint::Endpoint,
        cfg: &ConfigBag,
    ) -> Result<(), SerdeError> {
        self.inner.update_endpoint(request, endpoint, cfg)
    }

    fn parse_error_metadata(
        &self,
        response: &Response,
        _cfg: &ConfigBag,
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        let body = response.body().bytes().unwrap_or(&[]);
        parse_error_envelope_metadata(body, response.headers())
    }
}

/// Resolves the canonical RPC v2 CBOR request route from the config bag.
///
/// Per the [spec](https://smithy.io/2.0/additional-specs/protocols/smithy-rpc-v2.html#requests),
/// requests are sent to `/service/{serviceName}/operation/{operationName}`, using
/// the Smithy shape names of the service and the operation.
///
/// The service shape name comes from [`ServiceShapeName`] (stored by generated
/// clients for whichever protocol ends up being used) and the operation shape
/// name from [`Metadata`]. Returns `None` when either is absent, leaving the
/// caller to fall back to the endpoint it was given.
fn rpc_v2_cbor_route(cfg: &ConfigBag) -> Option<String> {
    let service = cfg.load::<ServiceShapeName>()?;
    let operation = cfg.load::<Metadata>()?;
    Some(format!(
        "/service/{}/operation/{}",
        service.as_str(),
        operation.name()
    ))
}

/// Parses the canonical CBOR error envelope. The envelope is a CBOR map at the
/// document root containing `__type` (the error code, optionally namespaced
/// and/or URL-suffixed in the same way as JSON envelopes) and an optional
/// message field (`message`, `Message`, or `errorMessage`).
///
/// `headers` is consulted only for the queryCompatible header
/// (`X-Amzn-Query-Error`); when present, it overrides any body-derived code
/// and stores the AWS error type as a `type` extra. Per the rpcv2Cbor spec,
/// non-queryCompatible services do not emit this header, so the override is
/// a no-op for them.
///
/// Returns an empty builder when the body is empty (and the queryCompatible
/// header is absent).
pub(crate) fn parse_error_envelope_metadata(
    response_body: &[u8],
    response_headers: &Headers,
) -> Result<ErrorMetadataBuilder, SerdeError> {
    let mut builder = if response_body.is_empty() {
        ErrorMetadata::builder()
    } else {
        parse_error_body(response_body)?
    };

    // queryCompatible override — see comment on the corresponding helper in
    // `aws-smithy-json::protocol::error`.
    if let Some((qc_code, qc_type)) = parse_query_compatible_header(response_headers) {
        builder = builder.code(qc_code).custom("type", qc_type);
    }

    Ok(builder)
}

fn parse_error_body(response_body: &[u8]) -> Result<ErrorMetadataBuilder, SerdeError> {
    let decoder = &mut Decoder::new(response_body);
    let mut builder = ErrorMetadata::builder();

    match decoder.map().map_err(deser_err)? {
        // Indefinite-length map: read entries until a `Break` token.
        None => loop {
            match decoder.datatype().map_err(deser_err)? {
                crate::data::Type::Break => {
                    decoder.skip().map_err(deser_err)?;
                    break;
                }
                _ => {
                    builder = error_code_and_message(builder, decoder)?;
                }
            }
        },
        // Definite-length map: read exactly `n` entries.
        Some(n) => {
            for _ in 0..n {
                builder = error_code_and_message(builder, decoder)?;
            }
        }
    }

    Ok(builder)
}

fn error_code_and_message(
    mut builder: ErrorMetadataBuilder,
    decoder: &mut Decoder,
) -> Result<ErrorMetadataBuilder, SerdeError> {
    let key = decoder.str().map_err(deser_err)?;
    builder = match key.as_ref() {
        "__type" => {
            // Silently skip if the value isn't a string, mirroring the
            // message-key handling below. A malformed error code
            // shouldn't prevent the rest of the envelope (e.g. the
            // message) from being recovered.
            match decoder.str() {
                Ok(code) => builder.code(sanitize_error_code(&code)),
                Err(_) => builder,
            }
        }
        "message" | "Message" | "errorMessage" => {
            // Silently skip if the value isn't a string. Custom error
            // structures may use non-string types under these keys.
            match decoder.str() {
                Ok(message) => builder.message(message),
                Err(_) => builder,
            }
        }
        _ => {
            decoder.skip().map_err(deser_err)?;
            builder
        }
    };
    Ok(builder)
}

fn deser_err(e: crate::decode::DeserializeError) -> SerdeError {
    SerdeError::invalid_input(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Encoder;
    use aws_smithy_runtime_api::http::{Response, StatusCode};
    use aws_smithy_types::body::SdkBody;
    use aws_smithy_types::config_bag::ConfigBag;

    fn cbor_response(body: Vec<u8>) -> Response {
        Response::new(StatusCode::try_from(400).unwrap(), SdkBody::from(body))
    }

    fn encode_envelope(type_value: Option<&str>, message_value: Option<(&str, &str)>) -> Vec<u8> {
        let mut entries: Vec<(&str, &str)> = vec![];
        if let Some(t) = type_value {
            entries.push(("__type", t));
        }
        if let Some((k, v)) = message_value {
            entries.push((k, v));
        }
        let mut encoder = Encoder::new(Vec::new());
        encoder.map(entries.len());
        for (k, v) in &entries {
            encoder.str(k).str(v);
        }
        encoder.into_writer()
    }

    #[test]
    fn parse_error_metadata_extracts_code_and_message() {
        let body = encode_envelope(Some("InvalidGreeting"), Some(("message", "Hi")));
        let response = cbor_response(body);
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .expect("parse succeeds")
            .build();

        assert_eq!(meta.code(), Some("InvalidGreeting"));
        assert_eq!(meta.message(), Some("Hi"));
    }

    #[test]
    fn parse_error_metadata_sanitizes_namespaced_code() {
        let body = encode_envelope(
            Some("aws.protocoltests.rpcv2cbor#InvalidGreeting:http://example/"),
            None,
        );
        let response = cbor_response(body);
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .expect("parse succeeds")
            .build();

        assert_eq!(meta.code(), Some("InvalidGreeting"));
    }

    #[test]
    fn parse_error_metadata_accepts_alternate_message_keys() {
        for key in &["Message", "errorMessage"] {
            let body = encode_envelope(Some("X"), Some((key, "msg")));
            let response = cbor_response(body);
            let cfg = ConfigBag::base();
            let protocol = RpcV2CborProtocol::new();

            let meta = protocol
                .parse_error_metadata(&response, &cfg)
                .expect("parse succeeds")
                .build();

            assert_eq!(meta.message(), Some("msg"), "key={}", key);
        }
    }

    #[test]
    fn parse_error_metadata_empty_body_returns_empty_builder() {
        let response = cbor_response(Vec::new());
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .expect("parse succeeds")
            .build();

        assert_eq!(meta.code(), None);
        assert_eq!(meta.message(), None);
    }

    #[test]
    fn parse_error_metadata_malformed_body_returns_error() {
        // 0xff is a Break token at top level — not a valid CBOR document root.
        let response = cbor_response(vec![0xff]);
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let err = protocol
            .parse_error_metadata(&response, &cfg)
            .expect_err("malformed body should fail");
        assert!(
            matches!(err, SerdeError::InvalidInput { .. }),
            "expected InvalidInput, got {:?}",
            err
        );
    }

    #[test]
    fn parse_error_metadata_ignores_unknown_keys() {
        // Encode a 3-entry map with an unknown key in the middle, so the
        // decoder must `skip()` past its value to continue reading.
        let mut encoder = Encoder::new(Vec::new());
        encoder.map(3);
        encoder.str("__type").str("InvalidGreeting");
        encoder.str("not_a_known_key").str("ignore me");
        encoder.str("message").str("Hi");
        let body = encoder.into_writer();
        let response = cbor_response(body);
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .expect("parse succeeds")
            .build();

        assert_eq!(meta.code(), Some("InvalidGreeting"));
        assert_eq!(meta.message(), Some("Hi"));
    }

    #[test]
    fn parse_error_metadata_query_compat_header_overrides_body_code() {
        // The body says `__type: CustomCodeError` (the shape name); the
        // queryCompatible header says `Customized;Sender` — codegen dispatch
        // keys for queryCompatible services use the awsQueryError code
        // (`Customized`), so the header MUST win.
        let body = encode_envelope(Some("CustomCodeError"), Some(("message", "Hi")));
        let mut response = cbor_response(body);
        response.headers_mut().insert(
            "x-amzn-query-error".to_string(),
            "Customized;Sender".to_string(),
        );
        let cfg = ConfigBag::base();
        let protocol = RpcV2CborProtocol::new();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .expect("parse succeeds")
            .build();

        assert_eq!(meta.code(), Some("Customized"));
        assert_eq!(meta.message(), Some("Hi"));
        assert_eq!(meta.extra("type"), Some("Sender"));
    }

    /// A minimal input with no members, enough to drive `serialize_request`.
    struct EmptyInput;
    impl aws_smithy_schema::serde::SerializableStruct for EmptyInput {
        fn serialize_members(
            &self,
            _: &mut dyn aws_smithy_schema::serde::ShapeSerializer,
        ) -> Result<(), SerdeError> {
            Ok(())
        }
    }

    static INPUT_SCHEMA: Schema<'static> = Schema::new(
        aws_smithy_schema::shape_id!("smithy.example", "GetStatsInput"),
        aws_smithy_schema::ShapeType::Structure,
    );

    /// Builds a config bag holding the model names a generated client stores.
    fn cfg_with_names(service: &'static str, operation: &'static str) -> ConfigBag {
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(ServiceShapeName::new(service));
        layer.store_put(Metadata::new(operation, "Some Sdk Id"));
        ConfigBag::of_layers(vec![layer])
    }

    fn serialize_with(cfg: &ConfigBag, endpoint: &str) -> Request {
        RpcV2CborProtocol::new()
            .serialize_request(&EmptyInput, &INPUT_SCHEMA, endpoint, cfg)
            .expect("serialization succeeds")
    }

    #[test]
    fn serialize_request_uses_canonical_rpc_route() {
        let cfg = cfg_with_names("PokemonService", "GetServerStatistics");
        let request = serialize_with(&cfg, "");
        assert_eq!(
            "/service/PokemonService/operation/GetServerStatistics",
            request.uri()
        );
        assert_eq!("POST", request.method());
    }

    /// `smithy-protocol` and `accept` are required on every RPC v2 CBOR request and
    /// are a function of the protocol alone, so this protocol must set them itself
    /// rather than rely on codegen having emitted them — a client generated for
    /// another protocol never did. Unlike the route, these do *not* fall back to
    /// anything: they are unconditional.
    #[test]
    fn serialize_request_sets_protocol_framing_headers() {
        let cfg = cfg_with_names("PokemonService", "GetServerStatistics");
        let request = serialize_with(&cfg, "");
        assert_eq!(
            Some("rpc-v2-cbor"),
            request.headers().get("smithy-protocol")
        );
        assert_eq!(Some("application/cbor"), request.headers().get("accept"));
    }

    /// The framing headers do not depend on the config bag, so they are still set
    /// when the model names are absent and the route falls back to the supplied
    /// endpoint.
    #[test]
    fn serialize_request_sets_framing_headers_without_model_names() {
        let cfg = ConfigBag::base();
        let request = serialize_with(&cfg, "/some/path");
        assert_eq!("/some/path", request.uri());
        assert_eq!(
            Some("rpc-v2-cbor"),
            request.headers().get("smithy-protocol")
        );
        assert_eq!(Some("application/cbor"), request.headers().get("accept"));
    }

    /// The route is protocol-mandated, so a path computed by codegen for a
    /// *different* protocol must not win when this protocol is plugged in at
    /// runtime. `/` is what an awsJson-generated client passes, and `/stats` is
    /// what a REST-generated client's `@http` URI would look like.
    /// See https://github.com/smithy-lang/smithy-rs/issues/4801.
    #[test]
    fn serialize_request_overrides_endpoint_from_another_protocol() {
        let cfg = cfg_with_names("PokemonService", "GetServerStatistics");
        for endpoint in ["/", "/stats", "/service/Wrong/operation/Wrong"] {
            let request = serialize_with(&cfg, endpoint);
            assert_eq!(
                "/service/PokemonService/operation/GetServerStatistics",
                request.uri(),
                "endpoint {endpoint:?} must not override the protocol's route",
            );
        }
    }

    /// Without the model names in the config bag there is nothing to resolve, so
    /// the supplied endpoint is used as-is. This keeps direct (non-generated)
    /// callers of the protocol API working.
    #[test]
    fn serialize_request_falls_back_to_supplied_endpoint() {
        let cfg = ConfigBag::base();
        assert_eq!(
            "/service/Fallback/operation/Op",
            serialize_with(&cfg, "/service/Fallback/operation/Op").uri()
        );
        // And with no endpoint at all, `HttpRpcProtocol`'s `/` default applies.
        assert_eq!("/", serialize_with(&cfg, "").uri());
    }

    /// `Metadata` alone is not enough — the service shape name is not derivable
    /// from it, since `Metadata::service` is the sdkId rather than the shape name.
    #[test]
    fn serialize_request_falls_back_when_only_metadata_is_present() {
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(Metadata::new("GetServerStatistics", "Pokemon Service"));
        let cfg = ConfigBag::of_layers(vec![layer]);
        assert_eq!("/", serialize_with(&cfg, "").uri());
    }

    /// The service name may be materialized at runtime (e.g. read from a parsed
    /// model) rather than being a codegen-emitted literal.
    #[test]
    fn service_shape_name_accepts_runtime_strings() {
        let owned: String = ["Pokemon", "Service"].concat();
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(ServiceShapeName::new(owned));
        layer.store_put(Metadata::new("GetServerStatistics", "Pokemon Service"));
        let cfg = ConfigBag::of_layers(vec![layer]);
        assert_eq!(
            "/service/PokemonService/operation/GetServerStatistics",
            serialize_with(&cfg, "").uri()
        );
    }
}
