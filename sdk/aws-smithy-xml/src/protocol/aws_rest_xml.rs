/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! AWS REST XML protocol implementation (`aws.protocols#restXml`).

use std::sync::Arc;

use crate::codec::{XmlCodec, XmlCodecSettings, XmlDeserializer};
use crate::decode::{try_data, Document};
use aws_smithy_schema::http_protocol::HttpBindingProtocol;
use aws_smithy_schema::serde::SerdeError;
use aws_smithy_schema::Schema;
use aws_smithy_schema::{shape_id, ShapeId};
use aws_smithy_types::config_bag::ConfigBag;
use aws_smithy_types::date_time::Format as TimestampFormat;
use aws_smithy_types::error::metadata::{Builder as ErrorMetadataBuilder, ErrorMetadata};

static PROTOCOL_ID: ShapeId<'static> = shape_id!("aws.protocols", "restXml");

/// AWS REST XML protocol (`aws.protocols#restXml`).
#[derive(Debug)]
pub struct AwsRestXmlProtocol {
    inner: HttpBindingProtocol<XmlCodec>,
    settings: Arc<XmlCodecSettings>,
    /// True if the service has `@restXml(noErrorWrapping: true)`.
    no_error_wrapping: bool,
    /// Service-level `@xmlNamespace` URI/prefix. Per the Smithy spec, this is
    /// the default xmlns applied to operation request/response root elements
    /// when the operation's input/output struct (or its `@httpPayload` target)
    /// doesn't declare its own `@xmlNamespace`.
    service_xml_namespace: Option<(String, Option<String>)>,
}

impl AwsRestXmlProtocol {
    /// Construct a new `AwsRestXmlProtocol` with default settings:
    /// Content-Type `application/xml`, `date-time` default timestamp
    /// format, error envelopes wrapped in `<ErrorResponse>` (i.e.
    /// `noErrorWrapping = false`), and no service-level `@xmlNamespace`.
    ///
    /// Use [`Self::with_no_error_wrapping`] to opt in to the `noErrorWrapping`
    /// variant of the `@restXml` trait, and
    /// [`Self::with_service_xml_namespace`] to set a service-level default
    /// xmlns. Both methods are typically called by code generation based on
    /// the service shape's traits; manual construction is fine for tests
    /// or custom protocols.
    pub fn new() -> Self {
        let settings = XmlCodecSettings::builder()
            .default_timestamp_format(TimestampFormat::DateTime)
            .build();
        let settings = Arc::new(settings);
        let codec = XmlCodec::from_shared_settings(settings.clone());
        Self {
            inner: HttpBindingProtocol::new(PROTOCOL_ID.clone(), codec, "application/xml"),
            settings,
            no_error_wrapping: false,
            service_xml_namespace: None,
        }
    }

    /// Configure whether error responses use the `noErrorWrapping` variant
    /// of the `@restXml` trait. When `true`, error response bodies have
    /// `<Error>` as the document root; when `false` (default), they are
    /// wrapped in `<ErrorResponse><Error>...</Error>...</ErrorResponse>`.
    /// Drives the parsing branch in `parse_error_metadata` and
    /// `deserialize_error_response`.
    ///
    /// Unlike the service XML namespace, this has **no config-bag default and cannot have one**:
    /// it is read from `@restXml(noErrorWrapping)`, and a model that does not use restXml carries
    /// no `@restXml` trait for a generated client to store. So when restXml is selected at runtime
    /// on, say, a CBOR-modeled service, there is nothing to recover — the caller must set it, and
    /// the documented default of `false` (wrapped envelopes, the spec default) applies otherwise.
    ///
    /// This is the general limit of runtime protocol swapping: a protocol's own trait data cannot
    /// be recovered from a model that does not use that protocol.
    pub fn with_no_error_wrapping(mut self, v: bool) -> Self {
        self.no_error_wrapping = v;
        self
    }

    /// Returns the current `noErrorWrapping` setting. See
    /// [`Self::with_no_error_wrapping`] for semantics.
    pub fn no_error_wrapping(&self) -> bool {
        self.no_error_wrapping
    }

    /// Configures the service-level `@xmlNamespace` declared on the Smithy
    /// service shape. Applied to request/response XML root elements that
    /// don't carry their own `@xmlNamespace` (e.g. S3's
    /// `http://s3.amazonaws.com/doc/2006-03-01/`).
    ///
    /// Overrides the default, which is the
    /// [`ServiceXmlNamespace`](aws_smithy_schema::protocol::ServiceXmlNamespace) config-bag entry
    /// that generated clients store regardless of which protocol they were generated for. That
    /// default exists because a customer selecting restXml through
    /// `Config::builder().protocol(..)` has no way to know the model's namespace.
    pub fn with_service_xml_namespace(
        mut self,
        uri: impl Into<String>,
        prefix: Option<String>,
    ) -> Self {
        self.service_xml_namespace = Some((uri.into(), prefix));
        self
    }

    /// Resolves the service-level `@xmlNamespace`: an explicit configuration wins, otherwise the
    /// config-bag entry.
    ///
    /// Returns owned values because the codec takes ownership; the explicit branch clones exactly
    /// as it did before this fell back to the bag.
    fn resolved_service_xml_namespace(&self, cfg: &ConfigBag) -> Option<(String, Option<String>)> {
        if let Some((uri, prefix)) = &self.service_xml_namespace {
            return Some((uri.clone(), prefix.clone()));
        }
        cfg.load::<aws_smithy_schema::protocol::ServiceXmlNamespace>()
            .map(|ns| (ns.uri().to_owned(), ns.prefix().map(str::to_owned)))
    }
}

/// Locate the `<Error>` element within an AWS REST XML error response body
/// and return a byte slice covering it (`<Error>...</Error>`).
///
/// Handles both wrapped and unwrapped error envelopes:
/// - **Wrapped** (`<ErrorResponse>...<Error>...</Error>...</ErrorResponse>`):
///   returns the inner `<Error>` element's slice.
/// - **Unwrapped** (`<Error>...</Error>` as the document root): returns the
///   full body unchanged (the root *is* the `<Error>` element).
///
/// Falls back to the full `body` if it isn't valid UTF-8, isn't parseable as
/// XML, or contains no `<Error>` element. Returning the body unchanged on
/// failure lets downstream error deserialization surface a generic /
/// "unhandled" error variant rather than panicking on malformed responses.
///
/// Robust to start-tag attributes (e.g. `<Error xmlns="..."`), nested
/// same-name elements, comments, and CDATA sections — all of which a naive
/// `body_str.find("<Error>")` substring match would mishandle.
pub fn find_error_element_slice(body: &[u8]) -> &[u8] {
    // Depth-2 `<Error>` (wrapped) or the root itself (unwrapped). Fall back to
    // the full body when the response isn't valid UTF-8/XML or has no `<Error>`,
    // so downstream parsing produces a malformed-error result rather than
    // panicking.
    crate::codec::find_depth2_element_slice_by(body, |name| name == "Error").unwrap_or(body)
}

impl Default for AwsRestXmlProtocol {
    fn default() -> Self {
        Self::new()
    }
}

impl aws_smithy_schema::protocol::ClientProtocolInner for AwsRestXmlProtocol {
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
        // XML-specific pre-scan: if the input has an `@httpPayload` struct or
        // union member with its own `@xmlName`, the body's wrapper element
        // must be that name. Codegen passes the *target* shape's `SCHEMA`
        // for the payload member's `write_struct` call (so the codec sees
        // the target's `@xmlName`, not the member's), so the codec on its
        // own would emit the wrong wrapper. Look up the member here, where
        // we have the input schema in hand, and pre-set the override on the
        // body serializer; the XmlSerializer consumes it on the first
        // root-level `write_struct`.
        let payload_xml_name = input_schema.members().iter().find_map(|m| {
            if m.http_payload().is_some()
                && matches!(
                    m.shape_type(),
                    aws_smithy_schema::ShapeType::Structure | aws_smithy_schema::ShapeType::Union
                )
            {
                m.xml_name().map(|n| n.value().to_owned())
            } else {
                None
            }
        });
        let mut body = aws_smithy_schema::codec::Codec::create_serializer(self.inner.codec());
        if let Some(name) = payload_xml_name {
            body.set_next_root_xml_name(name);
        }
        // Apply service-level `@xmlNamespace` as the document-root xmlns
        // fallback. Consumed by the codec on the first root-level
        // `write_struct` only if the struct's schema has no own
        // `@xmlNamespace`.
        if let Some((uri, prefix)) = self.resolved_service_xml_namespace(cfg) {
            body.set_next_root_xml_namespace(uri, prefix);
        }
        self.inner
            .serialize_request_with_body(body, input, input_schema, endpoint, cfg)
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
        self.inner
            .deserialize_response(response, output_schema, cfg)
    }

    /// Extracts canonical error metadata from a REST XML response.
    ///
    /// Walks the error envelope structurally to extract `Code` / `Message` /
    /// `Type` from inside `<Error>` and (for the wrapped variant) `RequestId`
    /// from the outer `<ErrorResponse>`.
    ///
    /// Honors [`Self::with_no_error_wrapping`]:
    ///
    /// - **Wrapped** (default): expects
    ///   `<ErrorResponse><Error><Code>...</Code>...</Error><RequestId>...</RequestId></ErrorResponse>`.
    /// - **Unwrapped** (`@restXml(noErrorWrapping: true)`): expects
    ///   `<Error><Code>...</Code>...</Error>` as the document root.
    ///
    /// Per the
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata)
    /// contract the request id is **not** populated here — the
    /// orchestrator's request-id pipeline attaches it separately. The
    /// `RequestId` element is captured into the builder's `extra` slot
    /// under the key `request_id` for callers that need it.
    fn parse_error_metadata(
        &self,
        response: &aws_smithy_runtime_api::http::Response,
        _cfg: &ConfigBag,
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        parse_error_envelope_metadata(
            response.body().bytes().unwrap_or(&[]),
            self.no_error_wrapping,
        )
    }

    /// Returns a [`ShapeDeserializer`](aws_smithy_schema::serde::ShapeDeserializer)
    /// positioned inside `<Error>` so generated
    /// `SpecificError::deserialize` reads land on the right elements.
    ///
    /// For the wrapped variant, uses [`find_error_element_slice`] to locate
    /// the `<Error>` sub-element. For `noErrorWrapping`, the body root IS
    /// `<Error>`, so the body is used unchanged.
    fn deserialize_error_response<'a>(
        &self,
        response: &'a aws_smithy_runtime_api::http::Response,
        _cfg: &ConfigBag,
    ) -> Result<
        Box<dyn aws_smithy_schema::serde::ShapeDeserializer + 'a>,
        aws_smithy_schema::serde::SerdeError,
    > {
        let body = response.body().bytes().unwrap_or(&[]);
        let inner = if self.no_error_wrapping {
            body
        } else {
            find_error_element_slice(body)
        };
        Ok(Box::new(XmlDeserializer::new(inner, self.settings.clone())))
    }

    fn payload_codec(&self) -> Option<&dyn aws_smithy_schema::codec::DynCodec> {
        self.inner.payload_codec()
    }

    /// This protocol labels structured event-stream payloads `application/xml`.
    ///
    /// Must stay in agreement with the code generator's
    /// `eventStreamMessageContentType` for this protocol (`RestXml.kt:47`), which supplies
    /// the fallback when a protocol declares no media type.
    fn event_stream_media_type(&self) -> Option<&str> {
        Some("application/xml")
    }

    /// Parses the same restXml error envelope as
    /// [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata), from an event-stream
    /// frame's payload rather than an HTTP response body.
    fn parse_event_stream_error_metadata(
        &self,
        payload: &[u8],
    ) -> Result<ErrorMetadataBuilder, SerdeError> {
        parse_error_envelope_metadata(payload, self.no_error_wrapping)
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

/// Parses a restXml error envelope out of a response or event-stream payload.
///
/// Shared by [`ClientProtocolInner::parse_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_error_metadata) and
/// [`ClientProtocolInner::parse_event_stream_error_metadata`](aws_smithy_schema::protocol::ClientProtocolInner::parse_event_stream_error_metadata) so the two cannot
/// drift. Takes `no_error_wrapping` rather than `&self` because it needs no other
/// protocol state, and takes the bytes rather than a response because an
/// event-stream frame is not one.
fn parse_error_envelope_metadata(
    body: &[u8],
    no_error_wrapping: bool,
) -> Result<ErrorMetadataBuilder, SerdeError> {
    // An empty body carries no error metadata. Some services signal errors
    // with a status code and an empty body (e.g. S3 HEAD operations, whose
    // responses have no body to hold an error code). Return an empty builder
    // rather than failing to locate a root XML element; the error code for
    // such responses is derived from the HTTP status by the generated
    // dispatch's error-metadata customizations.
    if body.is_empty() {
        return Ok(ErrorMetadata::builder());
    }
    let body_str =
        std::str::from_utf8(body).map_err(|e| SerdeError::custom(format!("invalid UTF-8: {e}")))?;
    let mut doc = Document::new(body_str);
    let mut root = doc
        .root_element()
        .map_err(|e| SerdeError::custom(format!("{e}")))?;
    let mut builder = ErrorMetadata::builder();

    if no_error_wrapping {
        // <Error><Code>...</Code><Message>...</Message>...members...</Error>
        while let Some(mut tag) = root.next_tag() {
            match tag.start_el().local() {
                "Code" => {
                    builder = builder
                        .code(try_data(&mut tag).map_err(|e| SerdeError::custom(format!("{e}")))?);
                }
                "Message" => {
                    builder = builder.message(
                        try_data(&mut tag).map_err(|e| SerdeError::custom(format!("{e}")))?,
                    );
                }
                _ => {}
            }
        }
    } else {
        // <ErrorResponse><Error><Code>...</Code>...</Error><RequestId>...</RequestId></ErrorResponse>
        while let Some(mut tag) = root.next_tag() {
            match tag.start_el().local() {
                "Error" => {
                    while let Some(mut error_field) = tag.next_tag() {
                        match error_field.start_el().local() {
                            "Code" => {
                                builder = builder.code(
                                    try_data(&mut error_field)
                                        .map_err(|e| SerdeError::custom(format!("{e}")))?,
                                );
                            }
                            "Message" => {
                                builder = builder.message(
                                    try_data(&mut error_field)
                                        .map_err(|e| SerdeError::custom(format!("{e}")))?,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                "RequestId" => {
                    builder = builder.custom(
                        "request_id",
                        try_data(&mut tag).map_err(|e| SerdeError::custom(format!("{e}")))?,
                    );
                }
                _ => {}
            }
        }
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_smithy_schema::protocol::ClientProtocolInner;
    use aws_smithy_schema::serde::{SerializableStruct, ShapeSerializer};
    use aws_smithy_schema::{shape_id, ShapeType};

    use aws_smithy_schema::traits::HttpTrait;

    static NAME_MEMBER: Schema<'static> =
        Schema::new_member(shape_id!("test", "Op$name"), ShapeType::String, "name", 0);
    static OP_SCHEMA: Schema<'static> = Schema::new_struct(
        shape_id!("test", "OpRequest"),
        ShapeType::Structure,
        &[&NAME_MEMBER],
    )
    .with_original_name("OpRequest")
    .with_http(HttpTrait::new("PUT", "/op", None));

    struct TestInput;
    impl SerializableStruct for TestInput {
        fn serialize_members(
            &self,
            ser: &mut dyn ShapeSerializer,
        ) -> Result<(), aws_smithy_schema::serde::SerdeError> {
            ser.write_string(&NAME_MEMBER, "Alice")
        }
    }

    #[test]
    fn serialize_request_produces_xml_body() {
        let protocol = AwsRestXmlProtocol::new();
        let cfg = ConfigBag::base();
        let request = protocol
            .serialize_request(&TestInput, &OP_SCHEMA, "", &cfg)
            .unwrap();

        assert_eq!(request.method(), "PUT");
        assert_eq!(request.uri(), "/op");
        assert_eq!(
            request.headers().get("content-type").unwrap(),
            "application/xml"
        );
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert_eq!(body, "<OpRequest><name>Alice</name></OpRequest>");
    }

    /// The service-level `@xmlNamespace` becomes the root `xmlns`, and a customer selecting restXml
    /// through `Config::builder().protocol(..)` has no way to know it, so it defaults from the
    /// config-bag entry generated clients store regardless of their protocol.
    #[test]
    fn service_xml_namespace_defaults_from_config_bag() {
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(aws_smithy_schema::protocol::ServiceXmlNamespace::new(
            "http://example.com/from-bag/",
            None,
        ));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let request = AwsRestXmlProtocol::new()
            .serialize_request(&TestInput, &OP_SCHEMA, "", &cfg)
            .unwrap();
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert_eq!(
            body,
            "<OpRequest xmlns=\"http://example.com/from-bag/\"><name>Alice</name></OpRequest>"
        );
    }

    /// A prefix in the bag entry is honored too.
    #[test]
    fn service_xml_namespace_from_config_bag_honors_prefix() {
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(aws_smithy_schema::protocol::ServiceXmlNamespace::new(
            "http://example.com/from-bag/",
            Some("ex".into()),
        ));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let request = AwsRestXmlProtocol::new()
            .serialize_request(&TestInput, &OP_SCHEMA, "", &cfg)
            .unwrap();
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert_eq!(
            body,
            "<OpRequest xmlns:ex=\"http://example.com/from-bag/\"><name>Alice</name></OpRequest>"
        );
    }

    /// An explicitly configured namespace still wins over the bag.
    #[test]
    fn with_service_xml_namespace_overrides_config_bag() {
        let mut layer = aws_smithy_types::config_bag::Layer::new("test");
        layer.store_put(aws_smithy_schema::protocol::ServiceXmlNamespace::new(
            "http://example.com/from-bag/",
            None,
        ));
        let cfg = ConfigBag::of_layers(vec![layer]);

        let request = AwsRestXmlProtocol::new()
            .with_service_xml_namespace("http://example.com/explicit/".to_owned(), None)
            .serialize_request(&TestInput, &OP_SCHEMA, "", &cfg)
            .unwrap();
        let body = std::str::from_utf8(request.body().bytes().unwrap()).unwrap();
        assert_eq!(
            body,
            "<OpRequest xmlns=\"http://example.com/explicit/\"><name>Alice</name></OpRequest>"
        );
    }

    #[test]
    fn deserialize_response_round_trips() {
        let protocol = AwsRestXmlProtocol::new();
        let cfg = ConfigBag::base();
        let body = b"<OpResponse><name>Bob</name></OpResponse>";
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(200).unwrap(),
            aws_smithy_types::body::SdkBody::from(&body[..]),
        );

        let mut deser = protocol
            .deserialize_response(&response, &OP_SCHEMA, &cfg)
            .unwrap();

        let mut name = String::new();
        deser
            .read_struct(&OP_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("name") {
                    name = d.read_string(member)?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(name, "Bob");
    }

    fn http_response(body: &[u8]) -> aws_smithy_runtime_api::http::Response {
        aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(400).unwrap(),
            aws_smithy_types::body::SdkBody::from(body),
        )
    }

    #[test]
    fn protocol_id_is_rest_xml() {
        assert_eq!(
            AwsRestXmlProtocol::new().protocol_id().as_str(),
            "aws.protocols#restXml"
        );
    }

    #[test]
    fn parse_error_metadata_wrapped() {
        let protocol = AwsRestXmlProtocol::new();
        let response = http_response(b"<ErrorResponse><Error><Type>Sender</Type><Code>InvalidGreeting</Code><Message>Hi</Message><Greeting>Howdy</Greeting></Error><RequestId>req-1</RequestId></ErrorResponse>");
        let cfg = ConfigBag::base();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("InvalidGreeting"));
        assert_eq!(meta.message(), Some("Hi"));
        assert_eq!(meta.extra("request_id"), Some("req-1"));
    }

    #[test]
    fn parse_error_metadata_unwrapped() {
        let protocol = AwsRestXmlProtocol::new().with_no_error_wrapping(true);
        let response = http_response(
            b"<Error><Code>NotFound</Code><Message>Gone</Message><Detail>extra</Detail></Error>",
        );
        let cfg = ConfigBag::base();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .unwrap()
            .build();
        assert_eq!(meta.code(), Some("NotFound"));
        assert_eq!(meta.message(), Some("Gone"));
        // No outer envelope means no RequestId on the unwrapped path.
        assert_eq!(meta.extra("request_id"), None);
    }

    #[test]
    fn parse_error_metadata_wrapped_ignores_literal_error_in_cdata() {
        // Regression: a previous implementation used
        // `body_str.find("<Error>")` to locate the envelope's `<Error>`
        // sub-element, which would also match the literal bytes of
        // `<Error>` inside an unrelated CDATA section. The new
        // structural walk uses the XML parser, which treats CDATA as
        // opaque text and only matches a real `<Error>` element.
        let protocol = AwsRestXmlProtocol::new();
        let response = http_response(
            b"<ErrorResponse>\
            <Description><![CDATA[<Error><Code>FAKE</Code></Error>]]></Description>\
            <Error><Code>RealCode</Code><Message>Real message</Message></Error>\
            <RequestId>req-1</RequestId>\
            </ErrorResponse>",
        );
        let cfg = ConfigBag::base();

        let meta = protocol
            .parse_error_metadata(&response, &cfg)
            .unwrap()
            .build();
        // The structural walk skips the CDATA content and finds the
        // real <Error>'s <Code>.
        assert_eq!(meta.code(), Some("RealCode"));
        assert_eq!(meta.message(), Some("Real message"));
        assert_eq!(meta.extra("request_id"), Some("req-1"));
    }

    #[test]
    fn deserialize_error_response_positions_inside_error_wrapped() {
        // The deserializer returned by the trait method must be
        // positioned over the `<Error>` element so generated
        // `SpecificError::deserialize` reads its members directly.
        let protocol = AwsRestXmlProtocol::new();
        let response = http_response(
            b"<ErrorResponse>\
                <Error xmlns=\"http://example.com/\"><name>Carol</name></Error>\
                <RequestId>req-1</RequestId>\
                </ErrorResponse>",
        );
        let cfg = ConfigBag::base();

        let mut deser = protocol
            .deserialize_error_response(&response, &cfg)
            .unwrap();

        let mut name = String::new();
        deser
            .read_struct(&OP_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("name") {
                    name = d.read_string(member)?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(name, "Carol");
    }

    #[test]
    fn deserialize_error_response_positions_inside_error_unwrapped() {
        // For `noErrorWrapping`, the body root IS `<Error>` so the
        // deserializer reads from the body unchanged.
        let protocol = AwsRestXmlProtocol::new().with_no_error_wrapping(true);
        let response = http_response(b"<Error><name>Dave</name></Error>");
        let cfg = ConfigBag::base();

        let mut deser = protocol
            .deserialize_error_response(&response, &cfg)
            .unwrap();

        let mut name = String::new();
        deser
            .read_struct(&OP_SCHEMA, &mut |member, d| {
                if member.member_name() == Some("name") {
                    name = d.read_string(member)?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(name, "Dave");
    }

    // Regression tests for `find_error_element_slice` covering the cases
    // where a naive `body.find("<Error>")` substring match would
    // mishandle the input.
    #[test]
    fn find_error_element_slice_strips_wrapped_envelope() {
        let body =
            b"<ErrorResponse><Error><Code>X</Code></Error><RequestId>r</RequestId></ErrorResponse>";
        let slice = find_error_element_slice(body);
        assert_eq!(slice, b"<Error><Code>X</Code></Error>");
    }

    #[test]
    fn find_error_element_slice_handles_xmlns_on_inner_error() {
        // The exact case the substring-find fails on: `xmlns` directly
        // on the inner `<Error>` start tag means `body.find("<Error>")`
        // returns `None` and the previous codegen returned the full
        // body, breaking downstream error-code lookup.
        let body = br#"<ErrorResponse><Error xmlns="http://example.com/"><Code>X</Code></Error></ErrorResponse>"#;
        let slice = find_error_element_slice(body);
        assert_eq!(
            slice,
            br#"<Error xmlns="http://example.com/"><Code>X</Code></Error>"#
        );
    }

    #[test]
    fn find_error_element_slice_returns_body_when_root_is_error() {
        // `noErrorWrapping` mode: the root element IS `<Error>`. The
        // slice should be the full body, since the root's tags are
        // already at the boundaries.
        let body = b"<Error><Code>X</Code><Message>m</Message></Error>";
        let slice = find_error_element_slice(body);
        assert_eq!(slice, body);
    }

    #[test]
    fn find_error_element_slice_falls_back_for_missing_error() {
        // Defensive: if the body doesn't contain an `<Error>` element
        // anywhere, return the body unchanged so downstream parsing
        // produces a malformed/unhandled error rather than panicking.
        let body = b"<SomethingElse><Code>X</Code></SomethingElse>";
        let slice = find_error_element_slice(body);
        assert_eq!(slice, body);
    }

    #[test]
    fn find_error_element_slice_falls_back_for_invalid_xml() {
        // Defensive: malformed XML returns the body unchanged.
        let body = b"not xml at all";
        let slice = find_error_element_slice(body);
        assert_eq!(slice, body);
    }

    #[test]
    fn find_error_element_slice_falls_back_for_invalid_utf8() {
        // Defensive: non-UTF-8 bytes return the body unchanged. The
        // `Document::try_from(&[u8])` path validates UTF-8.
        let body = &[0xFFu8, 0xFE, 0xFD][..];
        let slice = find_error_element_slice(body);
        assert_eq!(slice, body);
    }
}
